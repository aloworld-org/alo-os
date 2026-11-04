// Protocol checks on a real bus; the picker is explicitly a controlled fixture.
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

const CHOOSER: &str = "org.freedesktop.portal.FileChooser";
const NATIVE: &str = "org.freedesktop.impl.portal.desktop.gtk";
const REQUEST: &str = "org.freedesktop.portal.Request";
const STUDIO: &str = "org.aloworld.Studio";

#[derive(Default)]
struct DialogState {
    ready: AtomicBool,
    complete: AtomicBool,
    applications: Mutex<Vec<String>>,
}

struct ControlledChooser {
    state: Arc<DialogState>,
    code: u32,
    uris: Vec<String>,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl ControlledChooser {
    async fn save_file(&self, _handle: OwnedObjectPath, application: &str, _parent: &str, _title: &str, _options: HashMap<String, OwnedValue>) -> (u32, HashMap<String, OwnedValue>) {
        self.state.applications.lock().unwrap().push(application.to_owned());
        self.state.ready.store(true, Ordering::Release);
        let deadline = Instant::now() + Duration::from_secs(5);
        while !self.state.complete.load(Ordering::Acquire) && Instant::now() < deadline {
            async_io::Timer::after(Duration::from_millis(5)).await;
        }
        let uris = OwnedValue::try_from(Value::from(self.uris.clone())).unwrap();
        (self.code, HashMap::from([("uris".into(), uris)]))
    }
}

fn chooser(serving: &Serving, code: u32, uris: Vec<String>, wait: bool) -> (Connection, Arc<DialogState>) {
    let state = Arc::new(DialogState::default());
    state.complete.store(!wait, Ordering::Release);
    let connection = zbus::blocking::connection::Builder::address(serving._bus.address().as_str()).unwrap()
        .name(NATIVE).unwrap().serve_at(PORTALS, ControlledChooser { state: state.clone(), code, uris }).unwrap().build().unwrap();
    (connection, state)
}

fn until(check: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(4);
    while !check() {
        assert!(Instant::now() < deadline, "the request did not finish");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn save_on(client: &Connection, token: &str) -> OwnedObjectPath {
    client.call_method(Some(DESKTOP), PORTALS, Some(CHOOSER), "SaveFile", &("", "Save your recording", options(token))).unwrap().body().deserialize().unwrap()
}

#[test]
fn another_connection_for_the_same_application_cannot_cancel_its_save_dialog() {
    let serving = Serving::started("save-dialog-ownership");
    serving.become_(Some(STUDIO));
    let (_chooser, state) = chooser(&serving, 1, Vec::new(), true);
    let before = Instant::now();
    let handle = save_on(&serving.client, "recording_save");
    assert!(before.elapsed() < Duration::from_secs(1), "the method waited for its dialog");
    until(|| state.ready.load(Ordering::Acquire));
    assert_eq!(state.applications.lock().unwrap().as_slice(), [STUDIO]);
    let stranger = zbus::blocking::connection::Builder::address(serving._bus.address().as_str()).unwrap().build().unwrap();
    let refused = stranger.call_method(Some(DESKTOP), handle.as_str(), Some(REQUEST), "Close", &()).unwrap_err();
    assert!(matches!(refused, zbus::Error::MethodError(name, _, _) if name.as_str() == "org.freedesktop.DBus.Error.AccessDenied"));
    assert!(serving.record.everything().is_empty());
    serving.client.call_method(Some(DESKTOP), handle.as_str(), Some(REQUEST), "Close", &()).unwrap();
    until(|| !serving.record.everything().is_empty());
    assert!(matches!(serving.last_answer().outcome(), Outcome::Cancelled));
    assert!(serving.machine.grants.read().unwrap().is_empty());
    state.complete.store(true, Ordering::Release);
}

#[test]
fn a_disconnected_caller_ends_its_pending_dialog() {
    let serving = Serving::started("save-dialog-disconnection");
    serving.become_(Some(STUDIO));
    let (_chooser, state) = chooser(&serving, 1, Vec::new(), true);
    let caller = zbus::blocking::connection::Builder::address(serving._bus.address().as_str()).unwrap().build().unwrap();
    save_on(&caller, "disconnected_save");
    until(|| state.ready.load(Ordering::Acquire));
    drop(caller);
    until(|| !serving.record.everything().is_empty());
    assert!(matches!(serving.last_answer().outcome(), Outcome::Cancelled));
    assert!(serving.machine.grants.read().unwrap().is_empty());
    state.complete.store(true, Ordering::Release);
}

#[test]
fn native_cancellation_is_recorded_as_cancellation_without_a_file_grant() {
    let serving = Serving::started("save-dialog-cancelled");
    serving.become_(Some(STUDIO));
    let (_chooser, _state) = chooser(&serving, 1, Vec::new(), false);
    let code = serving.ask("cancelled_save", |client| client.call_method(Some(DESKTOP), PORTALS, Some(CHOOSER), "SaveFile", &("", "Save your recording", options("cancelled_save"))));
    assert_eq!(code, 1);
    assert!(matches!(serving.last_answer().outcome(), Outcome::Cancelled));
    assert!(serving.machine.grants.read().unwrap().is_empty());
}

#[test]
fn a_missing_native_backend_is_a_service_failure_without_a_file_grant() {
    let serving = Serving::started("save-dialog-unavailable");
    serving.become_(Some(STUDIO));
    let code = serving.ask("unavailable_save", |client| client.call_method(Some(DESKTOP), PORTALS, Some(CHOOSER), "SaveFile", &("", "Save your recording", options("unavailable_save"))));
    assert_eq!(code, 2);
    assert!(matches!(serving.last_answer().outcome(), Outcome::Unanswered(alo_portals::Unanswered::DialogUnavailable)));
    assert!(serving.machine.grants.read().unwrap().is_empty());
}

#[test]
fn a_failed_document_export_rolls_back_the_file_grant_and_creates_no_file() {
    let serving = Serving::started("save-document-unavailable");
    serving.become_(Some(STUDIO));
    let target = serving.files.join("demo.webm");
    let uri: String = url::Url::from_file_path(&target).unwrap().into();
    let (_chooser, _state) = chooser(&serving, 0, vec![uri], false);
    let code = serving.ask("unexported_save", |client| client.call_method(Some(DESKTOP), PORTALS, Some(CHOOSER), "SaveFile", &("", "Save your recording", options("unexported_save"))));
    assert_eq!(code, 2);
    assert!(serving.record.everything().iter().any(|answer| matches!(answer.outcome(), Outcome::FilesChosen(_))));
    assert!(matches!(serving.last_answer().outcome(), Outcome::Unanswered(alo_portals::Unanswered::DocumentUnavailable)));
    assert!(serving.machine.grants.read().unwrap().is_empty());
    assert!(!target.exists());
}
