//! The accessibility fallback against real applications' windows.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 6: *a password
//! field's contents are never read, held by a test against a real
//! application's password field.* Here the whole road runs on this machine,
//! with nothing standing in for the rented parts:
//!
//! - a session bus of the test's own, and on it **at-spi2's own bus
//!   launcher**, which starts the accessibility bus and its registry exactly as
//!   a signed-in session does;
//! - **GTK's own `gtk-builder-tool`**, showing two windows from interface
//!   files written here — a sign-in form with a real `GtkEntry` whose text is
//!   hidden and holds a password, and a payroll window nobody granted — each on
//!   GTK's Broadway display, which needs no screen;
//! - each window's process named by a sandbox file under a directory the test
//!   hands to `alo_portals::Sandboxes`, read through the process held by its
//!   descriptor — the same road a Flatpak application's is.
//!
//! **What is asserted about the wire is taken from the bus, not from this
//! crate.** A second connection becomes a monitor of the accessibility bus
//! before anything is read, and keeps every message: so *the password field is
//! never asked for its text*, *nothing is asked about positions*, *nothing is
//! subscribed to*, *the ungranted window is never addressed* and *no answer to
//! the fallback carries the password* are each read off what actually crossed
//! the bus.
//!
//! GTK 3 and not GTK 4: GTK 4 connects its accessibility only on an X11 or a
//! Wayland display, never on Broadway (`docs/quirks.md`). What this test is
//! about is the accessibility layer's wire, which is the same one both speak.
//! If any of the programs is missing the test fails rather than skipping — a
//! skipped test here would be green on a machine that proved nothing.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use alo_adapters::{
    ACTIVATE_CONTROL, AccessibilityTree, AccessibleSession, Activating, Adapters, Contents, Limit,
    NodeAt, READ_WINDOW, ReadingWindows, Role, fallback_verbs,
};
use alo_applications::{Application, Installed};
use alo_capability::{Approvals, Authorised, Call, Given, Grant, Grantee, Grants, Proposal, Reach};
use alo_portals::Sandboxes;
use alo_record::{Entry, Record};
use alo_strings::{Strings, Vocabulary};
use zbus::blocking::connection::Builder;
use zbus::blocking::{Connection, MessageIterator};
use zbus::message::Type;

/// The application whose window is granted.
const FIXTURE: &str = "org.example.Fixture";

/// The application whose window is not.
const PAYROLL: &str = "org.example.Payroll";

/// What is in the sign-in form's password field.
const SECRET: &str = "correct-horse-battery-staple";

/// What is in the payroll window, which nobody granted.
const SALARY: &str = "Anna earns 4,200";

/// How long anything here is waited for.
const PATIENCE: Duration = Duration::from_secs(40);

/// The sign-in form: words, a field, a password field, controls, a control
/// with no name, a greyed-out control, two controls of one name, and an area
/// the window draws itself.
const SIGN_IN_FORM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<interface>
  <object class="GtkWindow" id="window">
    <property name="title">Sign in to the fixture</property>
    <child>
      <object class="GtkBox">
        <property name="visible">True</property>
        <property name="orientation">vertical</property>
        <child><object class="GtkLabel"><property name="visible">True</property><property name="label">Account</property></object></child>
        <child><object class="GtkEntry"><property name="visible">True</property><property name="text">anna</property></object></child>
        <child><object class="GtkEntry"><property name="visible">True</property><property name="visibility">False</property><property name="text">correct-horse-battery-staple</property></object></child>
        <child><object class="GtkCheckButton"><property name="visible">True</property><property name="label">Remember me</property></object></child>
        <child><object class="GtkButton"><property name="visible">True</property><property name="label">Sign in</property></object></child>
        <child><object class="GtkButton"><property name="visible">True</property></object></child>
        <child><object class="GtkButton"><property name="visible">True</property><property name="sensitive">False</property><property name="label">Delete account</property></object></child>
        <child><object class="GtkButton"><property name="visible">True</property><property name="label">Help</property></object></child>
        <child><object class="GtkButton"><property name="visible">True</property><property name="label">Help</property></object></child>
        <child><object class="GtkDrawingArea"><property name="visible">True</property></object></child>
      </object>
    </child>
  </object>
</interface>
"#;

/// A window nobody granted.
const PAYROLL_WINDOW: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<interface>
  <object class="GtkWindow" id="window">
    <property name="title">Payroll</property>
    <child>
      <object class="GtkLabel"><property name="visible">True</property><property name="label">Anna earns 4,200</property></object>
    </child>
  </object>
</interface>
"#;

// ---------------------------------------------------------------------------
// A session of the test's own.
// ---------------------------------------------------------------------------

/// The first of these programs that is here.
fn found(candidates: &[&str], what: &str) -> PathBuf {
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
        .unwrap_or_else(|| {
            panic!(
                "{what} is not installed (looked at {candidates:?}) — this test needs the real \
                 program, and skipping would report green on a machine with none"
            )
        })
}

/// A program on the search path, or the test fails.
fn on_the_path(program: &str) -> PathBuf {
    std::env::var_os("PATH")
        .and_then(|path| {
            std::env::split_paths(&path)
                .map(|dir| dir.join(program))
                .find(|candidate| candidate.exists())
        })
        .unwrap_or_else(|| {
            panic!(
                "{program} is not installed — this test needs it, and skipping would report \
                 green on a machine with none"
            )
        })
}

/// Wait for `ready` to answer something, or fail saying what never came.
fn waiting_for<T>(what: &str, mut ready: impl FnMut() -> Option<T>) -> T {
    let started = Instant::now();
    loop {
        if let Some(answer) = ready() {
            return answer;
        }
        assert!(
            started.elapsed() < PATIENCE,
            "{what} did not happen in {PATIENCE:?}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// A signed-in session of the test's own: its session bus, its accessibility
/// bus, a display, and the windows shown on it.
struct ASession {
    place: PathBuf,
    session: String,
    accessibility: String,
    processes: Vec<Child>,
    windows: Vec<Child>,
}

impl ASession {
    fn started(what: &str) -> Self {
        let place = std::env::temp_dir().join(format!(
            "alo-fallback-{what}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(place.join("proc")).unwrap();
        let config = place.join("session.conf");
        std::fs::write(
            &config,
            format!(
                "<!DOCTYPE busconfig PUBLIC \"-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN\" \
                 \"http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd\">\n\
                 <busconfig>\n<type>session</type>\n<listen>unix:path={}</listen>\n\
                 <auth>EXTERNAL</auth>\n<policy context=\"default\">\n\
                 <allow send_destination=\"*\" eavesdrop=\"true\"/>\n<allow eavesdrop=\"true\"/>\n\
                 <allow own=\"*\"/>\n</policy>\n</busconfig>\n",
                place.join("session").display()
            ),
        )
        .unwrap();
        let mut session_daemon = Command::new(on_the_path("dbus-daemon"))
            .arg(format!("--config-file={}", config.display()))
            .args(["--nofork", "--print-address"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut session = String::new();
        BufReader::new(session_daemon.stdout.take().unwrap())
            .read_line(&mut session)
            .unwrap();
        let session = session.trim().to_owned();
        assert!(!session.is_empty(), "dbus-daemon printed no address");
        let mut this = Self {
            place,
            session,
            accessibility: String::new(),
            processes: vec![session_daemon],
            windows: Vec::new(),
        };

        let launcher = found(
            &[
                "/usr/libexec/at-spi-bus-launcher",
                "/usr/lib/at-spi2-core/at-spi-bus-launcher",
                "/usr/lib/at-spi-bus-launcher",
            ],
            "at-spi-bus-launcher",
        );
        let launched = this
            .environment(launcher)
            .arg("--launch-immediately")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        this.processes.push(launched);
        this.accessibility = waiting_for("the accessibility bus being announced", || {
            AccessibleSession::announced_on(&this.session).ok()
        });

        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let mut display = this
            .environment(on_the_path("broadwayd"))
            .args(["--address", "127.0.0.1", "--port", &port.to_string(), ":5"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        // Its socket is an abstract one, with no file to wait for: it says when
        // it is listening.
        let mut listening = String::new();
        BufReader::new(display.stdout.take().unwrap())
            .read_line(&mut listening)
            .unwrap();
        assert!(
            listening.starts_with("Listening on"),
            "broadwayd did not start: {listening:?}"
        );
        this.processes.push(display);
        this
    }

    /// A command with nothing of the machine's own session in its environment.
    fn environment(&self, program: PathBuf) -> Command {
        let mut command = Command::new(program);
        command
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("HOME", &self.place)
            .env("XDG_RUNTIME_DIR", &self.place)
            .env("XDG_CONFIG_HOME", self.place.join("config"))
            .env("XDG_CACHE_HOME", self.place.join("cache"))
            .env("XDG_DATA_HOME", self.place.join("data"))
            .env("DBUS_SESSION_BUS_ADDRESS", &self.session)
            .env("GSETTINGS_BACKEND", "memory")
            .env("LANG", "C.UTF-8");
        command
    }

    /// Show a window from this interface file, as the application its sandbox
    /// names.
    fn showing(&mut self, file: &str, interface: &str, application: &str) -> u32 {
        let at = self.place.join(file);
        std::fs::write(&at, interface).unwrap();
        let window = self
            .environment(on_the_path("gtk-builder-tool"))
            .env("GDK_BACKEND", "broadway")
            .env("BROADWAY_DISPLAY", ":5")
            .arg("preview")
            .arg(&at)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let process = window.id();
        let root = self
            .place
            .join("proc")
            .join(process.to_string())
            .join("root");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(".flatpak-info"),
            format!(
                "[Application]\nname={application}\nruntime=runtime/org.gnome.Platform/x86_64/48\n"
            ),
        )
        .unwrap();
        self.windows.push(window);
        process
    }

    /// Where sandboxes are read here.
    fn sandboxes(&self) -> Sandboxes {
        Sandboxes::under(self.place.join("proc"))
    }

    /// The fallback's tree, for one turn.
    fn tree(&self) -> AccessibleSession {
        AccessibleSession::at(&self.accessibility, self.sandboxes()).unwrap()
    }

    /// Wait until this application's window is on the accessibility bus and
    /// has something in it.
    fn waiting_for_window_of(&self, application: &str) {
        let tree = self.tree();
        waiting_for(&format!("{application}'s window appearing"), || {
            let running = tree.applications().ok()?;
            let root = running
                .into_iter()
                .find(|running| running.is.as_deref() == Some(application))?;
            let window = tree.facts(&root.at).ok()?.children.into_iter().next()?;
            (!tree.facts(&window).ok()?.children.is_empty()).then_some(())
        });
    }
}

impl Drop for ASession {
    fn drop(&mut self) {
        for child in self
            .windows
            .iter_mut()
            .chain(self.processes.iter_mut().rev())
        {
            // Asked to end, so the launcher ends the bus it started.
            let _ = Command::new("kill")
                .args(["-TERM", &child.id().to_string()])
                .status();
            let started = Instant::now();
            while child.try_wait().ok().flatten().is_none()
                && started.elapsed() < Duration::from_secs(5)
            {
                std::thread::sleep(Duration::from_millis(50));
            }
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = std::fs::remove_dir_all(&self.place);
    }
}

// ---------------------------------------------------------------------------
// What crossed the accessibility bus.
// ---------------------------------------------------------------------------

/// One message a monitor heard.
#[derive(Debug, Clone)]
struct Heard {
    kind: Type,
    sender: String,
    destination: String,
    object: String,
    interface: String,
    member: String,
    bytes: Vec<u8>,
}

/// A monitor of the accessibility bus, keeping every message on it.
struct AMonitor {
    heard: Arc<Mutex<Vec<Heard>>>,
    probe: Connection,
}

impl AMonitor {
    fn of(address: &str) -> Self {
        let monitor = Builder::address(address).unwrap().build().unwrap();
        monitor
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus.Monitoring"),
                "BecomeMonitor",
                &(Vec::<&str>::new(), 0_u32),
            )
            .expect("the accessibility bus refused a monitor");
        let heard = Arc::new(Mutex::new(Vec::new()));
        let keeping = Arc::clone(&heard);
        std::thread::spawn(move || {
            for message in MessageIterator::from(monitor) {
                let Ok(message) = message else { break };
                let header = message.header();
                let text = |value: Option<String>| value.unwrap_or_default();
                keeping.lock().unwrap().push(Heard {
                    kind: header.message_type(),
                    sender: text(header.sender().map(ToString::to_string)),
                    destination: text(header.destination().map(ToString::to_string)),
                    object: text(header.path().map(ToString::to_string)),
                    interface: text(header.interface().map(ToString::to_string)),
                    member: text(header.member().map(ToString::to_string)),
                    bytes: message.data().bytes().to_vec(),
                });
            }
        });
        Self {
            heard,
            probe: Builder::address(address).unwrap().build().unwrap(),
        }
    }

    /// Everything heard, once everything sent before now has been heard.
    fn everything(&self) -> Vec<Heard> {
        let marker = format!(
            "/alo/marker/n{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let _ = self.probe.call_method(
            Some("org.freedesktop.DBus"),
            marker.as_str(),
            Some("org.freedesktop.DBus.Peer"),
            "Ping",
            &(),
        );
        waiting_for("the monitor hearing everything", || {
            let heard = self.heard.lock().unwrap();
            heard
                .iter()
                .any(|message| message.object == marker)
                .then(|| heard.clone())
        })
    }

    /// Every connection on the bus now.
    fn names(&self) -> Vec<String> {
        self.probe
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "ListNames",
                &(),
            )
            .unwrap()
            .body()
            .deserialize::<Vec<String>>()
            .unwrap()
    }

    /// The events anybody has asked the registry to deliver.
    fn registered_events(&self) -> Vec<(String, String)> {
        self.probe
            .call_method(
                Some("org.a11y.atspi.Registry"),
                "/org/a11y/atspi/registry",
                Some("org.a11y.atspi.Registry"),
                "GetRegisteredEvents",
                &(),
            )
            .unwrap()
            .body()
            .deserialize::<Vec<(String, String)>>()
            .unwrap()
    }
}

/// Where the first thing of this role is in `application`'s windows — found
/// through a tree of the test's own, not the one under test.
fn where_the_role_is(tree: &AccessibleSession, application: &str, role: u32) -> NodeAt {
    let root = tree
        .applications()
        .unwrap()
        .into_iter()
        .find(|running| running.is.as_deref() == Some(application))
        .unwrap()
        .at;
    let mut waiting = vec![root];
    while let Some(at) = waiting.pop() {
        let facts = tree.facts(&at).unwrap();
        if facts.role == role {
            return at;
        }
        waiting.extend(facts.children);
    }
    panic!("{application} shows nothing of role {role}");
}

// ---------------------------------------------------------------------------
// The machine's side.
// ---------------------------------------------------------------------------

fn noon() -> SystemTime {
    SystemTime::now()
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

fn the_fixture_granted(at: SystemTime) -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@alo",
            Reach::Application(FIXTURE.to_owned()),
            at,
            Duration::from_secs(3600),
        )
        .unwrap(),
    );
    grants
}

fn installed() -> Installed {
    Installed::holding([
        Application::called(FIXTURE, "Fixture").unwrap(),
        Application::called(PAYROLL, "Payroll").unwrap(),
    ])
}

fn pressing(kind: &str, name: &str) -> Call {
    fallback_verbs()
        .unwrap()
        .call(
            ACTIVATE_CONTROL,
            &[
                ("application", Given::text(FIXTURE)),
                ("kind", Given::text(kind)),
                ("name", Given::text(name)),
            ],
        )
        .unwrap()
}

fn approved(call: &Call, grants: &Grants, at: SystemTime) -> Authorised {
    let mut approvals = Approvals::default();
    let id = approvals
        .propose(Proposal::checked(call, &agent(), grants, at, Duration::from_secs(3600)).unwrap());
    approvals
        .approve(id, at)
        .unwrap()
        .redeem(grants, at)
        .unwrap()
}

/// Read the fixture's windows through a tree of its own, for one turn.
fn read_the_fixture(
    session: &ASession,
    grants: &Grants,
    at: SystemTime,
    strings: &Strings,
) -> (Authorised, alo_adapters::Shown, String) {
    let call = fallback_verbs()
        .unwrap()
        .call(READ_WINDOW, &[("application", Given::text(FIXTURE))])
        .unwrap();
    let tree = session.tree();
    let asks_as = tree.asks_as().unwrap();
    let (authorised, shown) = ReadingWindows::of(
        Authorised::read(&call, &agent(), grants, at).unwrap(),
        &Adapters::default(),
        grants,
        &installed(),
        strings,
    )
    .unwrap()
    .read(&tree, strings)
    .unwrap()
    .into_parts();
    (authorised, shown, asks_as)
}

/// Everything a connection sent, as the monitor heard it.
fn sent_by<'h>(heard: &'h [Heard], asks_as: &'h str) -> impl Iterator<Item = &'h Heard> {
    heard
        .iter()
        .filter(move |message| message.sender == asks_as && message.kind == Type::MethodCall)
}

/// **A real application's password field: its contents are never read.** The
/// sign-in form's hidden `GtkEntry` holds a password; the fallback reads the
/// window, finds the field, says it is a password field — and the bus shows it
/// was never asked for its text, no answer carried the password, nothing was
/// asked about positions or subscribed to, the payroll window nobody granted
/// was never addressed, and the connection was gone when the turn was.
#[test]
fn a_real_applications_password_field_is_never_read() {
    let mut session = ASession::started("password");
    session.showing("sign-in.ui", SIGN_IN_FORM, FIXTURE);
    session.showing("payroll.ui", PAYROLL_WINDOW, PAYROLL);
    session.waiting_for_window_of(FIXTURE);
    session.waiting_for_window_of(PAYROLL);

    let probe = session.tree();
    let password = where_the_role_is(&probe, FIXTURE, 40);
    let payroll_holder = probe
        .applications()
        .unwrap()
        .into_iter()
        .find(|running| running.is.as_deref() == Some(PAYROLL))
        .unwrap()
        .at
        .holder;
    drop(probe);

    let monitor = AMonitor::of(&session.accessibility);
    let strings = in_english();
    let at = noon();
    let grants = the_fixture_granted(at);
    let (authorised, shown, asks_as) = read_the_fixture(&session, &grants, at, &strings);

    // What was read.
    assert_eq!(
        shown
            .windows()
            .iter()
            .map(alo_adapters::ShownWindow::title)
            .collect::<Vec<_>>(),
        ["Sign in to the fixture"]
    );
    let field = shown
        .everything()
        .find(|seen| seen.role() == Role::PasswordField)
        .expect("the password field was not found at all");
    assert_eq!(field.contents(), &Contents::Withheld);
    assert!(
        shown.everything().any(|seen| seen.role() == Role::TextField
            && seen.contents() == &Contents::Text("anna".to_owned())),
        "the ordinary field was not read — so the test would prove nothing about the other"
    );
    assert!(shown.everything().any(|seen| seen.name() == "Account"));
    let described = format!("{shown:?}");
    assert!(
        !described.contains(SECRET),
        "the password is in what was read"
    );
    assert!(!described.contains(SALARY), "the payroll window was read");

    // What crossed the bus.
    let heard = monitor.everything();
    let sent: Vec<&Heard> = sent_by(&heard, &asks_as).collect();
    assert!(
        sent.len() > 10,
        "the monitor heard almost nothing: {sent:?}"
    );
    assert!(
        sent.iter().any(|message| message.member == "GetText"),
        "no text was asked for at all, so its absence below proves nothing"
    );
    for message in &sent {
        assert!(
            !(message.object == password.object
                && message.destination == password.holder
                && (message.interface == "org.a11y.atspi.Text"
                    || message.interface == "org.a11y.atspi.EditableText")),
            "the password field was asked for its contents: {message:?}"
        );
        assert!(
            !matches!(
                message.interface.as_str(),
                "org.a11y.atspi.Component" | "org.a11y.atspi.Image"
            ),
            "something was asked where it is: {message:?}"
        );
        assert!(
            !matches!(
                message.member.as_str(),
                "RegisterEvent" | "AddMatch" | "BecomeMonitor"
            ),
            "something was subscribed to: {message:?}"
        );
        assert_ne!(
            message.destination, payroll_holder,
            "the payroll window, never granted, was addressed: {message:?}"
        );
    }
    let secret = SECRET.as_bytes();
    for message in heard
        .iter()
        .filter(|message| message.destination == asks_as)
    {
        assert!(
            !message
                .bytes
                .windows(secret.len())
                .any(|bytes| bytes == secret),
            "an answer to the fallback carried the password: {message:?}"
        );
    }
    assert!(monitor.registered_events().is_empty());

    // The turn is over: nothing of it is still on the bus.
    waiting_for("the reading connection closing", || {
        (!monitor.names().contains(&asks_as)).then_some(())
    });

    let mut record = Record::default();
    record.keep(Entry::ran(&authorised, &strings));
    assert!(record.everything().next().unwrap().happened().ran());
    drop(session);
}

/// **A real control is pressed by its kind and name, once, for one approval** —
/// the form's check box is ticked afterwards — and on the same window a
/// control that is not there, one greyed out and one of two of a name are
/// refused in words; a control with no name and an area the window draws
/// itself are said, never guessed at; and nothing at all is asked about where
/// anything is.
#[test]
fn a_real_control_is_pressed_by_its_kind_and_name() {
    let mut session = ASession::started("press");
    session.showing("sign-in.ui", SIGN_IN_FORM, FIXTURE);
    session.waiting_for_window_of(FIXTURE);
    let monitor = AMonitor::of(&session.accessibility);
    let strings = in_english();
    let at = noon();
    let grants = the_fixture_granted(at);

    let (_, before, _) = read_the_fixture(&session, &grants, at, &strings);
    let remember = |shown: &alo_adapters::Shown| {
        shown
            .everything()
            .find(|seen| seen.name() == "Remember me")
            .map(alo_adapters::Seen::is_on)
            .expect("the check box was not read")
    };
    assert_eq!(remember(&before), Some(false));
    let limits: Vec<(Role, Limit)> = before
        .everything()
        .filter_map(|seen| seen.limit().map(|limit| (seen.role(), limit)))
        .collect();
    assert!(
        limits.contains(&(Role::Button, Limit::NoName)),
        "{limits:?}"
    );
    assert!(
        limits.contains(&(Role::Canvas, Limit::NotDescribed)),
        "{limits:?}"
    );

    let tree = session.tree();
    let asks_as = tree.asks_as().unwrap();
    let pressed = Activating::of(
        approved(&pressing("check_box", "Remember me"), &grants, at),
        &Adapters::default(),
        &grants,
        &installed(),
        &strings,
    )
    .unwrap()
    .press(&tree, &strings)
    .unwrap();
    assert!(!pressed.unanswered());
    let heard = monitor.everything();
    assert_eq!(
        sent_by(&heard, &asks_as)
            .filter(|message| message.member == "DoAction")
            .count(),
        1,
        "one approval, one press"
    );
    drop(tree);
    let mut record = Record::default();
    record.keep(Entry::ran(&pressed.into_authorised(), &strings));

    let after = waiting_for("the check box being ticked", || {
        let (_, after, _) = read_the_fixture(&session, &grants, at, &strings);
        (remember(&after) == Some(true)).then_some(after)
    });
    assert_eq!(remember(&after), Some(true));

    for (kind, name, words) in [
        (
            "button",
            "Forgot password",
            "org.example.Fixture shows no button named “Forgot password” now, so nothing was \
             pressed",
        ),
        (
            "button",
            "Delete account",
            "The button named “Delete account” in org.example.Fixture cannot be used right now, \
             so it was not pressed",
        ),
        (
            "button",
            "Help",
            "org.example.Fixture shows more than one button named “Help”, so nothing was pressed \
             rather than guessing which",
        ),
    ] {
        let tree = session.tree();
        let asks_as = tree.asks_as().unwrap();
        let refused = Activating::of(
            approved(&pressing(kind, name), &grants, at),
            &Adapters::default(),
            &grants,
            &installed(),
            &strings,
        )
        .unwrap()
        .press(&tree, &strings)
        .unwrap_err();
        assert_eq!(refused.said(&strings).text(), words);
        record.keep(Entry::refused(&refused, &agent(), &strings, at));
        let heard = monitor.everything();
        assert_eq!(
            sent_by(&heard, &asks_as)
                .filter(|message| message.member == "DoAction")
                .count(),
            0,
            "{name} was pressed"
        );
    }

    // Nobody on this bus — the fallback's every connection included — asked
    // where anything is.
    for message in monitor
        .everything()
        .iter()
        .filter(|message| message.kind == Type::MethodCall)
    {
        assert!(
            !matches!(
                message.interface.as_str(),
                "org.a11y.atspi.Component" | "org.a11y.atspi.Image"
            ),
            "something was asked where it is: {message:?}"
        );
    }
    assert_eq!(record.everything().count(), 4);
}
