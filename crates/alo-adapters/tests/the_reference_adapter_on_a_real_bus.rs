//! The reference adapter end to end, on a real bus.
//!
//! `docs/autonomy/software-and-the-web-plan.md`, task 5: *one reference
//! adapter for an application task 2 ships is built end to end as the proof,
//! with the application's own automation interface.* Here the whole road runs:
//! the shipped verbs, a grant, a proposal approved once, the authority redeemed,
//! [`Driving`], and [`SessionBus`] sending the message over a real session bus
//! started by this test — to a service that holds the text editor's own name and
//! answers `org.freedesktop.Application` on `/org/gnome/TextEditor`, as GNOME
//! Text Editor does. What the service received is asserted from the service's
//! side.
//!
//! **What it is not** is GNOME Text Editor itself: no machine this ran on has
//! it installed, and the report says what the on-machine acceptance is. The bus
//! is `dbus-daemon`, started from a configuration naming no service directory,
//! so nothing can be activated on it and nothing of the machine's own session
//! is reached.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::HashMap;
use std::io::{BufRead as _, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use alo_adapters::{Driving, SessionBus, adapter_verbs, shipped_adapters};
use alo_applications::{Application, Installed};
use alo_capability::{Approvals, Authorised, Call, Given, Grant, Grantee, Grants, Proposal, Reach};
use alo_record::{Entry, Record};
use alo_strings::{Strings, Vocabulary};
use zbus::zvariant::OwnedValue;

/// A bus of this test's own, which nothing else can reach.
struct ABusOfOurOwn {
    place: PathBuf,
    daemon: Child,
    address: String,
}

impl ABusOfOurOwn {
    fn started(what: &str) -> Self {
        let place = std::env::temp_dir().join(format!(
            "alo-adapters-{what}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&place).unwrap();
        let config = place.join("bus.conf");
        std::fs::write(
            &config,
            format!(
                "<!DOCTYPE busconfig PUBLIC \"-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN\" \
                 \"http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd\">\n\
                 <busconfig>\n\
                 <type>session</type>\n\
                 <listen>unix:path={}</listen>\n\
                 <auth>EXTERNAL</auth>\n\
                 <policy context=\"default\">\n\
                 <allow send_destination=\"*\" eavesdrop=\"true\"/>\n\
                 <allow eavesdrop=\"true\"/>\n\
                 <allow own=\"*\"/>\n\
                 </policy>\n\
                 </busconfig>\n",
                place.join("bus").display()
            ),
        )
        .unwrap();
        let mut daemon = Command::new("dbus-daemon")
            .arg(format!("--config-file={}", config.display()))
            .arg("--nofork")
            .arg("--print-address")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect(
                "dbus-daemon is not installed — this test needs a real bus, and skipping would \
                 report green on a machine with none",
            );
        let mut address = String::new();
        BufReader::new(daemon.stdout.take().unwrap())
            .read_line(&mut address)
            .unwrap();
        let address = address.trim().to_owned();
        assert!(!address.is_empty(), "dbus-daemon printed no address");
        Self {
            place,
            daemon,
            address,
        }
    }
}

impl Drop for ABusOfOurOwn {
    fn drop(&mut self) {
        let _ = self.daemon.kill();
        let _ = self.daemon.wait();
        let _ = std::fs::remove_dir_all(&self.place);
    }
}

/// What the stand-in application was asked.
#[derive(Default, Clone)]
struct Asked(Arc<Mutex<Vec<String>>>);

impl Asked {
    fn heard(&self, what: String) {
        self.0.lock().unwrap().push(what);
    }

    fn everything(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

/// The text editor, as far as its bus interface goes.
struct TheTextEditor {
    asked: Asked,
}

#[zbus::interface(name = "org.freedesktop.Application")]
impl TheTextEditor {
    fn activate(&self, _platform_data: HashMap<String, OwnedValue>) {
        self.asked.heard("Activate".to_owned());
    }

    fn open(&self, uris: Vec<String>, platform_data: HashMap<String, OwnedValue>) {
        self.asked
            .heard(format!("Open {uris:?} {}", platform_data.len()));
    }

    fn activate_action(
        &self,
        action_name: String,
        parameter: Vec<OwnedValue>,
        platform_data: HashMap<String, OwnedValue>,
    ) {
        self.asked.heard(format!(
            "ActivateAction {action_name} {} {}",
            parameter.len(),
            platform_data.len()
        ));
    }
}

/// The text editor, holding its own name on the bus and answering on its own
/// object.
fn the_text_editor_on(bus: &ABusOfOurOwn) -> (zbus::blocking::Connection, Asked) {
    let asked = Asked::default();
    let connection = zbus::blocking::connection::Builder::address(bus.address.as_str())
        .unwrap()
        .name("org.gnome.TextEditor")
        .unwrap()
        .serve_at(
            "/org/gnome/TextEditor",
            TheTextEditor {
                asked: asked.clone(),
            },
        )
        .unwrap()
        .build()
        .unwrap();
    (connection, asked)
}

fn noon() -> SystemTime {
    SystemTime::now()
}

fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

fn agent() -> Grantee {
    Grantee::named("@alo")
}

fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_applications::words::declare_into(&mut vocabulary).unwrap();
    alo_adapters::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

fn grants(at: SystemTime) -> Grants {
    let mut grants = Grants::default();
    for reach in [
        Reach::Application("org.gnome.TextEditor".to_owned()),
        Reach::Folder(PathBuf::from("/home/anna/Notes")),
    ] {
        grants.grant(Grant::checked("@alo", reach, at, hour()).unwrap());
    }
    grants
}

fn installed() -> Installed {
    Installed::holding([Application::called("org.gnome.TextEditor", "Text Editor").unwrap()])
}

fn approved(call: &Call, grants: &Grants, at: SystemTime) -> Authorised {
    let mut approvals = Approvals::default();
    let id = approvals.propose(Proposal::checked(call, &agent(), grants, at, hour()).unwrap());
    approvals
        .approve(id, at)
        .unwrap()
        .redeem(grants, at)
        .unwrap()
}

/// The recording-interface fixture on the private session bus.
struct RecordingControls {
    /// Calls observed independently at the public interface.
    asked: Asked,
    /// The status currently returned by the fixture.
    status: Arc<Mutex<String>>,
    /// The literal countdown requests received over D-Bus.
    countdowns: Arc<Mutex<Vec<i64>>>,
}

#[zbus::interface(name = "org.aloworld.Studio.Recorder1")]
impl RecordingControls {
    /// Return the actual fixture status as a string.
    fn get_status(&self) -> String {
        self.asked.heard("GetStatus".to_owned());
        self.status.lock().unwrap().clone()
    }

    fn new_recording(&self) {
        self.asked.heard("NewRecording".to_owned());
    }

    fn choose_screen(&self) {
        self.asked.heard("ChooseScreen".to_owned());
    }

    fn choose_window(&self) {
        self.asked.heard("ChooseWindow".to_owned());
    }

    fn start(&self) {
        self.asked.heard("Start".to_owned());
    }

    fn pause(&self) {
        self.asked.heard("Pause".to_owned());
    }

    fn resume(&self) {
        self.asked.heard("Resume".to_owned());
    }

    fn finish(&self) {
        self.asked.heard("Finish".to_owned());
    }

    fn enable_microphone(&self) {
        self.asked.heard("EnableMicrophone".to_owned());
    }

    fn disable_microphone(&self) {
        self.asked.heard("DisableMicrophone".to_owned());
    }

    fn cancel_countdown(&self) {
        self.asked.heard("CancelCountdown".to_owned());
    }

    fn show_library(&self) {
        self.asked.heard("ShowLibrary".to_owned());
    }

    fn export_recording(&self) {
        self.asked.heard("ExportRecording".to_owned());
    }

    fn set_quality(&self, quality: String) {
        self.asked.heard(format!("SetQuality {quality}"));
    }

    fn review_recording(&self, recording: String) {
        self.asked.heard(format!("ReviewRecording {recording}"));
    }

    fn rename_recording(&self, title: String) {
        self.asked.heard(format!("RenameRecording {title}"));
    }

    /// Record the fixed numeric argument actually delivered.
    fn set_countdown(&self, seconds: i64) {
        self.asked.heard(format!("SetCountdown {seconds}"));
        self.countdowns.lock().unwrap().push(seconds);
    }
}

#[test]
fn studio_reads_actual_bounded_status_and_sends_only_the_approved_fixed_countdown() {
    let bus = ABusOfOurOwn::started("studio-controls");
    let status = Arc::new(Mutex::new("{\"phase\":\"selected\"}".to_owned()));
    let countdowns = Arc::new(Mutex::new(Vec::new()));
    let service = zbus::blocking::connection::Builder::address(bus.address.as_str())
        .unwrap()
        .name(alo_adapters::studio::APPLICATION)
        .unwrap()
        .serve_at(
            "/org/aloworld/Studio/Recorder",
            RecordingControls {
                asked: Asked::default(),
                status: status.clone(),
                countdowns: countdowns.clone(),
            },
        )
        .unwrap()
        .build()
        .unwrap();
    let transport = SessionBus::at(&bus.address).unwrap();
    let strings = in_english();
    let at = noon();
    let adapters = shipped_adapters().unwrap();
    let verbs = adapter_verbs().unwrap();
    let installed =
        Installed::holding([
            Application::called(alo_adapters::studio::APPLICATION, "alo Studio").unwrap(),
        ]);
    let mut grants = Grants::default();
    let permission = grants.grant(
        Grant::checked(
            "@alo",
            Reach::Application(alo_adapters::studio::APPLICATION.to_owned()),
            at,
            hour(),
        )
        .unwrap(),
    );
    let read = || {
        let call = verbs.call("studio.status", &[]).unwrap();
        Authorised::read(&call, &agent(), &grants, at).unwrap()
    };
    let answer = Driving::of(read(), &adapters, &grants, &installed, &strings)
        .unwrap()
        .deliver(&transport, &strings)
        .unwrap();
    assert_eq!(answer.reply(), Some("{\"phase\":\"selected\"}"));
    assert!(!answer.unanswered());
    let call = verbs.call("studio.countdown_three", &[]).unwrap();
    assert!(call.what_it_changes(&strings).is_some());
    let authorised = approved(&call, &grants, at);
    assert!(countdowns.lock().unwrap().is_empty());
    Driving::of(authorised, &adapters, &grants, &installed, &strings)
        .unwrap()
        .deliver(&transport, &strings)
        .unwrap();
    assert_eq!(*countdowns.lock().unwrap(), [3]);
    let boundary = "x".repeat(8192);
    *status.lock().unwrap() = boundary.clone();
    let driving = Driving::of(read(), &adapters, &grants, &installed, &strings).unwrap();
    assert_eq!(
        alo_adapters::Delivers::read_text(&transport, driving.message()),
        Ok(boundary.clone())
    );
    let answer = driving.deliver(&transport, &strings).unwrap();
    assert_eq!(answer.reply(), Some(boundary.as_str()));
    *status.lock().unwrap() = "x".repeat(8193);
    let driving = Driving::of(read(), &adapters, &grants, &installed, &strings).unwrap();
    assert_eq!(
        alo_adapters::Delivers::read_text(&transport, driving.message()),
        Err(alo_adapters::NotDelivered::Refused)
    );
    assert!(
        Driving::of(read(), &adapters, &grants, &installed, &strings)
            .unwrap()
            .deliver(&transport, &strings)
            .is_err()
    );
    // Drop can leave asynchronous bus work owning the connection temporarily.
    // Confirm that the name is released before testing an unavailable app.
    assert!(
        service
            .release_name(alo_adapters::studio::APPLICATION)
            .unwrap()
    );
    service.close().unwrap();
    assert!(
        Driving::of(read(), &adapters, &grants, &installed, &strings)
            .unwrap()
            .deliver(&transport, &strings)
            .is_err()
    );
    assert!(grants.revoke(permission));
    let call = verbs.call("studio.status", &[]).unwrap();
    let authority = Authorised::read(&call, &agent(), &grants, at).unwrap();
    assert!(Driving::of(authority, &adapters, &grants, &installed, &strings).is_err());
}

/// These wire expectations come from the public automation contract, rather
/// than ACTION_NAMES or the adapter declaration. A swapped action or countdown
/// constant must fail even when the model fixture agrees with that declaration.
#[test]
fn every_studio_control_delivers_its_contract_method_and_arguments_once() {
    let bus = ABusOfOurOwn::started("studio-all-controls");
    let asked = Asked::default();
    let _service = zbus::blocking::connection::Builder::address(bus.address.as_str())
        .unwrap()
        .name("org.aloworld.Studio")
        .unwrap()
        .serve_at(
            "/org/aloworld/Studio/Recorder",
            RecordingControls {
                asked: asked.clone(),
                status: Arc::new(Mutex::new("{\"phase\":\"paused\"}".to_owned())),
                countdowns: Arc::default(),
            },
        )
        .unwrap()
        .build()
        .unwrap();
    let transport = SessionBus::at(&bus.address).unwrap();
    let strings = in_english();
    let at = noon();
    let adapters = shipped_adapters().unwrap();
    let verbs = adapter_verbs().unwrap();
    let installed =
        Installed::holding([Application::called("org.aloworld.Studio", "alo Studio").unwrap()]);
    let cases = [
        ("studio.status", None, "GetStatus"),
        ("studio.new_recording", None, "NewRecording"),
        ("studio.choose_screen", None, "ChooseScreen"),
        ("studio.choose_window", None, "ChooseWindow"),
        ("studio.start", None, "Start"),
        ("studio.pause", None, "Pause"),
        ("studio.resume", None, "Resume"),
        ("studio.finish", None, "Finish"),
        ("studio.enable_microphone", None, "EnableMicrophone"),
        ("studio.disable_microphone", None, "DisableMicrophone"),
        (
            "studio.set_quality",
            Some(("quality", "lightweight")),
            "SetQuality lightweight",
        ),
        ("studio.countdown_off", None, "SetCountdown 0"),
        ("studio.countdown_three", None, "SetCountdown 3"),
        ("studio.countdown_five", None, "SetCountdown 5"),
        ("studio.cancel_countdown", None, "CancelCountdown"),
        ("studio.show_library", None, "ShowLibrary"),
        (
            "studio.review",
            Some(("recording", "1000-20-1")),
            "ReviewRecording 1000-20-1",
        ),
        (
            "studio.rename",
            Some(("title", "Product demonstration")),
            "RenameRecording Product demonstration",
        ),
        ("studio.export", None, "ExportRecording"),
    ];
    for (name, argument, expected) in cases {
        let mut grants = Grants::default();
        let permission = grants.grant(
            Grant::checked(
                "@alo",
                Reach::Application("org.aloworld.Studio".to_owned()),
                at,
                hour(),
            )
            .unwrap(),
        );
        let given: Vec<_> = argument
            .into_iter()
            .map(|(name, value)| (name, Given::text(value)))
            .collect();
        let call = verbs.call(name, &given).unwrap();
        let authority = || {
            if name == "studio.status" {
                Authorised::read(&call, &agent(), &grants, at).unwrap()
            } else {
                approved(&call, &grants, at)
            }
        };
        let before = asked.everything();
        let accepted = authority();
        assert_eq!(asked.everything(), before, "authorisation sent {name}");
        let answer = Driving::of(accepted, &adapters, &grants, &installed, &strings)
            .unwrap()
            .deliver(&transport, &strings)
            .unwrap();
        assert!(!answer.unanswered(), "{name}");
        let mut expected_calls = before;
        expected_calls.push(expected.to_owned());
        assert_eq!(asked.everything(), expected_calls, "{name}");
        if name == "studio.status" {
            assert_eq!(answer.reply(), Some("{\"phase\":\"paused\"}"));
        }
        // Keep the service alive: an unavailable app cannot stand in for the
        // grant check between authorisation and sending the message.
        let already_authorised = authority();
        assert!(grants.revoke(permission));
        assert!(
            Driving::of(already_authorised, &adapters, &grants, &installed, &strings).is_err(),
            "revoked authority still reached {name}"
        );
        assert_eq!(asked.everything(), expected_calls, "revocation sent {name}");
    }
}

/// **The text editor receives what a person approved, once, over a real bus**:
/// the document's file address for *open*, and the action written into the
/// adapter for *new window* — and both are recorded as run.
#[test]
fn the_text_editor_receives_what_was_approved_and_it_is_recorded() {
    let bus = ABusOfOurOwn::started("receives");
    let (_editor, asked) = the_text_editor_on(&bus);
    let strings = in_english();
    let at = noon();
    let grants = grants(at);
    let adapters = shipped_adapters().unwrap();
    let verbs = adapter_verbs().unwrap();
    let session = SessionBus::at(&bus.address).unwrap();
    let mut record = Record::default();

    let open = verbs
        .call(
            "text_editor.open_document",
            &[("document", Given::text("/home/anna/Notes/März & more.txt"))],
        )
        .unwrap();
    let new_window = verbs.call("text_editor.new_window", &[]).unwrap();

    for call in [&open, &new_window] {
        let driven = Driving::of(
            approved(call, &grants, at),
            &adapters,
            &grants,
            &installed(),
            &strings,
        )
        .unwrap()
        .deliver(&session, &strings)
        .unwrap();
        assert!(!driven.unanswered());
        record.keep(Entry::ran(&driven.into_authorised(), &strings));
    }

    assert_eq!(
        asked.everything(),
        [
            "Open [\"file:///home/anna/Notes/M%C3%A4rz%20%26%20more.txt\"] 0".to_owned(),
            "ActivateAction new-window 0 0".to_owned(),
        ]
    );
    assert_eq!(record.len(), 2);
    assert!(record.everything().all(|entry| entry.happened().ran()));
}

/// **An application that is not on the bus is refused in words, and so is one
/// that does not offer what was asked** — both from what the real bus answered.
#[test]
fn an_application_not_there_or_not_offering_it_is_refused_in_words() {
    let strings = in_english();
    let at = noon();
    let grants = grants(at);
    let adapters = shipped_adapters().unwrap();
    let call = adapter_verbs()
        .unwrap()
        .call(
            "text_editor.open_document",
            &[("document", Given::text("/home/anna/Notes/march.txt"))],
        )
        .unwrap();

    // Nobody holds the name, and nothing can be activated on this bus.
    let empty = ABusOfOurOwn::started("empty");
    let refused = Driving::of(
        approved(&call, &grants, at),
        &adapters,
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .deliver(&SessionBus::at(&empty.address).unwrap(), &strings)
    .unwrap_err();
    assert!(
        refused
            .said(&strings)
            .text()
            .contains("could not be reached"),
        "{}",
        refused.said(&strings).text()
    );

    // Something holds the name and answers on another object only — a release
    // whose interface moved.
    let other = ABusOfOurOwn::started("other");
    let _holder = zbus::blocking::connection::Builder::address(other.address.as_str())
        .unwrap()
        .name("org.gnome.TextEditor")
        .unwrap()
        .serve_at(
            "/org/gnome/SomethingElse",
            TheTextEditor {
                asked: Asked::default(),
            },
        )
        .unwrap()
        .build()
        .unwrap();
    let refused = Driving::of(
        approved(&call, &grants, at),
        &adapters,
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .deliver(&SessionBus::at(&other.address).unwrap(), &strings)
    .unwrap_err();
    assert!(
        refused
            .said(&strings)
            .text()
            .contains("does not offer what was approved"),
        "{}",
        refused.said(&strings).text()
    );

    let mut record = Record::default();
    record.keep(Entry::refused(&refused, &agent(), &strings, at));
    assert!(record.everything().next().unwrap().happened().was_stopped());
}
