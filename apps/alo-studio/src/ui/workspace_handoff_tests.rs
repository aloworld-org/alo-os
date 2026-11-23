//! Real GTK recording-bar lifetime at the production workspace handoff.
//!
//! Install as a child module of `ui::controller`; it deliberately calls the
//! actual `show_bar`, `show_workspace` and error path, with real GTK windows.
//! These are local window-lifecycle tests, not capture or installed Dock proof.
//! Run on an owned display/session bus. No recorder, portal or media is faked.

#![expect(
    clippy::expect_used,
    reason = "a missing real GTK fixture window or display is a test failure"
)]

use super::*;

/// Own all GTK windows used by one isolated handoff case, including on failure.
struct Handoff {
    application: gtk::Application,
    state: Rc<State>,
}

impl Handoff {
    /// Construct the normal production layout without starting capture or a bus API.
    fn new(suffix: &str) -> Self {
        let application = gtk::Application::builder()
            .application_id(format!("org.aloworld.Studio.HandoffFixture.{suffix}"))
            .flags(gio::ApplicationFlags::NON_UNIQUE)
            .build();
        application
            .register(None::<&gio::Cancellable>)
            .expect("the exclusively owned GTK application registers");
        let state = Rc::new(State {
            view: super::super::layout::build(&application),
            session: RefCell::new(Session::default()),
            capture: RefCell::new(None),
            engine: RefCell::new(None),
            clip: RefCell::new(None),
            library: None,
            kind: Cell::new(SourceKind::Screen),
            finish_started: Cell::new(None),
            close_after_save: Cell::new(false),
            closed: Cell::new(false),
            export_pending: Cell::new(false),
            automation: RefCell::new(None),
            bar: RefCell::new(None),
        });
        Self { application, state }
    }

    /// Map the actual production bar and observe that it hides the workspace.
    fn mapped_bar(&self) -> gtk::ApplicationWindow {
        self.state.show_bar();
        let bar = self
            .state
            .bar
            .borrow()
            .as_ref()
            .expect("show_bar retains its real window")
            .0
            .clone();
        let deadline = Instant::now() + Duration::from_secs(3);
        while !bar.is_mapped() && Instant::now() < deadline {
            while glib::MainContext::default().pending() {
                glib::MainContext::default().iteration(false);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(
            bar.is_mapped(),
            "the premise requires a real mapped recording bar"
        );
        assert!(bar.is_visible());
        assert!(
            !self.state.view.window.is_visible(),
            "recording hides the workspace"
        );
        assert!(
            !self.state.view.window.is_active(),
            "the hidden workspace has no focus"
        );
        bar
    }

    /// Obsolete controls must retire at handoff, without awaiting WM activation.
    fn handed_off(&self, bar: &gtk::ApplicationWindow) {
        assert!(
            self.state.view.window.is_visible(),
            "the workspace must be presented first"
        );
        assert!(
            !bar.is_visible(),
            "obsolete recording controls remain visible until activation/timeout"
        );
        assert!(
            !bar.is_mapped(),
            "obsolete controls must not remain a second mapped Studio window"
        );
        assert!(self.state.bar.borrow().is_none());
        let windows = self.application.windows();
        assert_eq!(
            windows.len(),
            1,
            "handoff leaves only the normal workspace in GtkApplication"
        );
        assert_eq!(windows.first(), Some(self.state.view.window.upcast_ref()));
    }
}

impl Drop for Handoff {
    fn drop(&mut self) {
        self.state.session.borrow_mut().reset();
        self.state.close_bar();
        for window in self.application.windows() {
            window.set_child(None::<&gtk::Widget>);
            window.set_application(None::<&gtk::Application>);
            window.destroy();
        }
    }
}

/// Finishing has selected Review before this method: old controls are now obsolete.
#[gtk::test]
#[ignore = "requires an owned GTK display and session bus"]
fn review_handoff_unmaps_the_recording_bar_without_waiting_for_activation() {
    let fixture = Handoff::new("Review");
    let bar = fixture.mapped_bar();
    fixture.state.session.borrow_mut().reviewing();
    fixture.state.show_workspace();
    // No main-loop iteration, external focus command or five-second sleep here.
    fixture.handed_off(&bar);
    assert_eq!(fixture.state.session.borrow().phase(), Phase::Review);
}

/// The actual error path must also expose the workspace and retire stale controls.
#[gtk::test]
#[ignore = "requires an owned GTK display and session bus"]
fn error_handoff_unmaps_the_recording_bar_and_keeps_the_notice() {
    let fixture = Handoff::new("Error");
    let bar = fixture.mapped_bar();
    fixture.state.failed("Test-only owned recording failure");
    fixture.handed_off(&bar);
    assert_eq!(fixture.state.session.borrow().phase(), Phase::Ready);
    assert_eq!(
        fixture.state.view.banner_text.text(),
        "Test-only owned recording failure"
    );
    assert!(fixture.state.view.banner.reveals_child());
}

/// Repeated handoffs neither recreate controls nor destroy the normal workspace.
#[gtk::test]
#[ignore = "requires an owned GTK display and session bus"]
fn workspace_handoff_is_idempotent_with_and_without_an_existing_bar() {
    let fixture = Handoff::new("Idempotent");
    fixture.state.show_workspace();
    fixture.state.show_workspace();
    assert!(fixture.state.view.window.is_visible());
    assert_eq!(fixture.application.windows().len(), 1);
    let bar = fixture.mapped_bar();
    fixture.state.session.borrow_mut().reviewing();
    fixture.state.show_workspace();
    fixture.state.show_workspace();
    fixture.handed_off(&bar);
    assert_eq!(fixture.state.session.borrow().phase(), Phase::Review);
}
