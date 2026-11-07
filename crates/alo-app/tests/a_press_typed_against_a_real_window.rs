//! **`alo-app alo press check_box "Remember me"`, typed against a real GTK
//! window on a real accessibility bus** — the whole road a person's press
//! takes, through the real binary, with nothing standing in for the tree.
//!
//! The same session `alo-adapters`' own real-application test starts: a
//! session bus of the test's own, **at-spi2's own bus launcher**, GTK's
//! Broadway display, and `gtk-builder-tool` showing a window from an interface
//! file written here. A desktop door of the test's own answers `alo-tree` with
//! the unique name that window's tree answers as on the accessibility bus —
//! which is how `alo` is reached on a machine, from the compositor that
//! publishes the shell's tree.
//!
//! **Why `alo` and not the window's application.** On a machine an application
//! is told apart by its sandbox, which the kernel holds; a window shown by a
//! test has no sandbox, and giving the binary a folder of pretend sandboxes to
//! read would be a way to tell it which program is which that a person's own
//! command must not have. `alo` is found by its unique name instead, which no
//! other connection can hold, so this reaches a real window through exactly the
//! code a person's press runs, with nothing added to the binary for the test.
//!
//! If any of the programs is missing the test fails rather than skipping — a
//! skipped test here would be green on a machine that proved nothing.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, Write as _};
use std::net::TcpListener;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use alo_adapters::{AccessibilityTree, AccessibleSession, Whose, read_by_hand};
use alo_portals::Sandboxes;

/// What the window is known as through its pretend sandbox, which only this
/// test reads.
const FIXTURE: &str = "org.example.Fixture";

/// How long anything here is waited for.
const PATIENCE: Duration = Duration::from_secs(40);

/// A window with a check box, a button and a label.
const THE_WINDOW: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<interface>
  <object class="GtkWindow" id="window">
    <property name="title">Preferences</property>
    <child>
      <object class="GtkBox">
        <property name="visible">True</property>
        <property name="orientation">vertical</property>
        <child><object class="GtkLabel"><property name="visible">True</property><property name="label">Keep me signed in</property></object></child>
        <child><object class="GtkCheckButton"><property name="visible">True</property><property name="label">Remember me</property></object></child>
        <child><object class="GtkButton"><property name="visible">True</property><property name="label">Close</property></object></child>
      </object>
    </child>
  </object>
</interface>
"#;

/// The first of these programs that is here, or the test fails.
fn found(candidates: &[&str], what: &str) -> PathBuf {
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
        .unwrap_or_else(|| panic!("{what} is not installed (looked at {candidates:?})"))
}

/// A program on the search path, or the test fails.
fn on_the_path(program: &str) -> PathBuf {
    std::env::var_os("PATH")
        .and_then(|path| {
            std::env::split_paths(&path)
                .map(|dir| dir.join(program))
                .find(|candidate| candidate.exists())
        })
        .unwrap_or_else(|| panic!("{program} is not installed"))
}

/// Wait for `ready` to answer, or fail saying what never came.
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

/// A signed-in session of the test's own.
struct ASession {
    /// Where everything is.
    place: PathBuf,
    /// The session bus's address.
    session: String,
    /// The accessibility bus's address.
    accessibility: String,
    /// Everything started, in order.
    processes: Vec<Child>,
}

impl ASession {
    /// A session bus, an accessibility bus, a display and the window.
    fn started() -> Self {
        let place = std::env::temp_dir().join(format!(
            "alo-app-press-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(place.join("proc")).unwrap();
        std::fs::create_dir_all(place.join("doors")).unwrap();
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
        let mut this = Self {
            place,
            session: session.trim().to_owned(),
            accessibility: String::new(),
            processes: vec![session_daemon],
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
            .args(["--address", "127.0.0.1", "--port", &port.to_string(), ":6"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut listening = String::new();
        BufReader::new(display.stdout.take().unwrap())
            .read_line(&mut listening)
            .unwrap();
        assert!(listening.starts_with("Listening on"), "{listening:?}");
        this.processes.push(display);

        let interface = this.place.join("window.ui");
        std::fs::write(&interface, THE_WINDOW).unwrap();
        let window = this
            .environment(on_the_path("gtk-builder-tool"))
            .env("GDK_BACKEND", "broadway")
            .env("BROADWAY_DISPLAY", ":6")
            .arg("preview")
            .arg(&interface)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let root = this
            .place
            .join("proc")
            .join(window.id().to_string())
            .join("root");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(".flatpak-info"),
            format!(
                "[Application]\nname={FIXTURE}\nruntime=runtime/org.gnome.Platform/x86_64/48\n"
            ),
        )
        .unwrap();
        this.processes.push(window);
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

    /// The tree as only this test reads it: with the window's pretend sandbox.
    fn tree(&self) -> AccessibleSession {
        AccessibleSession::at(
            &self.accessibility,
            Sandboxes::under(self.place.join("proc")),
        )
        .unwrap()
    }

    /// The unique name the window's tree answers as, once it shows something.
    fn where_the_window_is(&self) -> String {
        let tree = self.tree();
        waiting_for("the window appearing on the tree", || {
            let running = tree.applications().ok()?;
            let root = running
                .into_iter()
                .find(|running| running.is.as_deref() == Some(FIXTURE))?;
            let window = tree.facts(&root.at).ok()?.children.into_iter().next()?;
            (!tree.facts(&window).ok()?.children.is_empty()).then_some(root.at.holder)
        })
    }

    /// Whether the check box is ticked, read through the tree.
    fn ticked(&self, holder: &str) -> Option<bool> {
        let shown = read_by_hand(&self.tree(), Whose::AlosOwn { answers_as: holder }, None).ok()?;
        shown
            .everything()
            .find(|seen| seen.name() == "Remember me")
            .and_then(alo_adapters::Seen::is_on)
    }
}

impl Drop for ASession {
    fn drop(&mut self) {
        for child in self.processes.iter_mut().rev() {
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

/// A desktop door that answers `alo-tree` with this name, and nothing else.
fn a_door_saying_alos_tree_is_at(session: &ASession, holder: &str) {
    let listener = UnixListener::bind(session.place.join("doors").join("shell.sock")).unwrap();
    let answer = format!("at {holder}");
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { break };
            let mut line = String::new();
            let mut reading = BufReader::new(stream.try_clone().unwrap());
            if reading.read_line(&mut line).unwrap_or(0) == 0 {
                continue;
            }
            let mut writing = stream;
            let said = if line.trim_end() == "alo-tree" {
                answer.as_str()
            } else {
                "could-not-ask"
            };
            let _ = writeln!(writing, "{said}");
        }
    });
}

#[test]
fn a_typed_press_ticks_a_real_check_box_and_writes_it_down() {
    let session = ASession::started();
    let holder = session.where_the_window_is();
    assert_eq!(session.ticked(&holder), Some(false), "it began ticked");
    a_door_saying_alos_tree_is_at(&session, &holder);

    let typed = |line: &[&str]| {
        session
            .environment(PathBuf::from(env!("CARGO_BIN_EXE_alo-app")))
            .env("ALO_APP_DOORS_IN", session.place.join("doors"))
            .args(line)
            .output()
            .expect("the command this crate builds runs")
    };

    let pressed = typed(&["alo", "press", "check_box", "Remember me"]);
    let said = String::from_utf8_lossy(&pressed.stdout).into_owned();
    let warned = String::from_utf8_lossy(&pressed.stderr).into_owned();
    assert_eq!(pressed.status.code(), Some(0), "{said}{warned}");
    waiting_for("the check box being ticked", || {
        (session.ticked(&holder) == Some(true)).then_some(())
    });

    // **And it is written down**, as alo's own surface, by the tree.
    let record = std::fs::read_to_string(session.place.join(".local/state/alo/commands.jsonl"))
        .unwrap_or_default();
    let entries: Vec<&str> = record
        .lines()
        .filter(|line| line.contains("asked-an-application"))
        .collect();
    assert_eq!(entries.len(), 1, "{record}");
    let entry = entries.first().unwrap();
    assert!(entry.contains(r#""road":"tree""#), "{entry}");
    assert!(entry.contains(r#""application":"alo""#), "{entry}");

    // **A control that is not there is refused, and nothing is pressed.**
    let refused = typed(&["alo", "press", "button", "Sign out"]);
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(session.ticked(&holder), Some(true));
}
