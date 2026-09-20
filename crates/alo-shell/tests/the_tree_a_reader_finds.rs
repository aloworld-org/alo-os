//! **Every surface this machine draws, read back off the accessibility bus.**
//!
//! `docs/autonomy/v0-5-the-shell-plan.md` task 12: *the shell exposes every
//! surface's role, name and state to AT-SPI as `alo-access` decides them, and a
//! test reads the exposed tree over the bus for each surface the shell draws*.
//!
//! So nothing here reads `alo_shell::ReadAloudTree`. The tree is served on **a
//! real accessibility bus**, started by **at-spi2's own bus launcher** exactly
//! as a signed-in session starts it, embedded in **its registry** the way every
//! toolkit's bridge embeds one, and read back with `alo-adapters` — the agent's
//! own reader, which knows nothing about this crate and asks the questions a
//! screen reader asks. What is asserted is what crossed the bus.
//!
//! **That the agent's reader is the one used is the point, not a convenience.**
//! `alo_access::tree` argues that a blind person and an agent read one
//! description of this machine rather than two; if the agent's reader can read
//! every surface, the claim is measured rather than asserted. It also holds the
//! role numbers to the only other place in this workspace that has them.
//!
//! If a program this needs is missing the test fails rather than skipping: a
//! skipped test here would be green on a machine that proved nothing.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use alo_access::{Control, Surface};
use alo_adapters::{AccessibilityTree, AccessibleSession, Facts, NodeAt, Role};
use alo_portals::Sandboxes;
use alo_shell::{ReadAloudBus, ReadAloudTree};
use alo_strings::{Filling, Strings};

/// How long anything here is waited for.
const PATIENCE: Duration = Duration::from_secs(40);

/// The first of the two words `GetState` answers with holds the low states.
const FOCUSABLE: u32 = 11;
/// `ATSPI_STATE_FOCUSED`.
const FOCUSED: u32 = 12;
/// `ATSPI_STATE_IS_DEFAULT`.
const IS_DEFAULT: u32 = 39;
/// `ATSPI_STATE_CHECKED`.
const CHECKED: u32 = 4;

/// A program this test needs, wherever the machine keeps it.
fn found(candidates: &[&str], what: &str) -> PathBuf {
    for candidate in candidates {
        let at = PathBuf::from(candidate);
        if at.exists() {
            return at;
        }
    }
    panic!("{what} is not on this machine, so nothing here proved anything");
}

/// A program on the path.
fn on_the_path(program: &str) -> PathBuf {
    let path = std::env::var_os("PATH").unwrap_or_default();
    for place in std::env::split_paths(&path) {
        let at = place.join(program);
        if at.exists() {
            return at;
        }
    }
    panic!("{program} is not on this machine, so nothing here proved anything");
}

/// Wait for something that takes a moment to be true.
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

/// A signed-in session of the test's own: its session bus, and on it the
/// accessibility bus and registry at-spi2 runs.
struct ASession {
    /// Where its files are.
    place: PathBuf,
    /// Its session bus.
    session: String,
    /// Its accessibility bus.
    accessibility: String,
    /// What it started, killed when it is dropped.
    processes: Vec<Child>,
}

impl ASession {
    /// A session of this test's own, started.
    fn started() -> Self {
        let place = std::env::temp_dir().join(format!(
            "alo-read-aloud-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&place).unwrap();
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
        let mut daemon = Command::new(on_the_path("dbus-daemon"))
            .arg(format!("--config-file={}", config.display()))
            .args(["--nofork", "--print-address"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut session = String::new();
        BufReader::new(daemon.stdout.take().unwrap())
            .read_line(&mut session)
            .unwrap();
        let session = session.trim().to_owned();
        assert!(!session.is_empty(), "dbus-daemon printed no address");

        let mut this = Self {
            place,
            session,
            accessibility: String::new(),
            processes: vec![daemon],
        };
        let launcher = found(
            &[
                "/usr/libexec/at-spi-bus-launcher",
                "/usr/lib/at-spi2-core/at-spi-bus-launcher",
                "/usr/lib/at-spi-bus-launcher",
            ],
            "at-spi-bus-launcher",
        );
        let mut command = Command::new(launcher);
        command
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("HOME", &this.place)
            .env("XDG_RUNTIME_DIR", &this.place)
            .env("DBUS_SESSION_BUS_ADDRESS", &this.session)
            .env("GSETTINGS_BACKEND", "memory")
            .env("LANG", "C.UTF-8")
            .arg("--launch-immediately")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        this.processes.push(command.spawn().unwrap());
        this.accessibility = waiting_for("the accessibility bus being announced", || {
            AccessibleSession::announced_on(&this.session).ok()
        });
        this
    }

    /// A reader of this session's tree.
    fn reader(&self) -> AccessibleSession {
        AccessibleSession::at(
            &self.accessibility,
            Sandboxes::under(self.place.join("proc")),
        )
        .expect("the accessibility bus this test started")
    }
}

impl Drop for ASession {
    fn drop(&mut self) {
        for process in &mut self.processes {
            let _ = process.kill();
            let _ = process.wait();
        }
        let _ = std::fs::remove_dir_all(&self.place);
    }
}

/// Everything this machine can say, in English.
fn words() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// One word, as a person reads it.
fn said(strings: &Strings, control: &Control) -> String {
    strings
        .say(&control.name.key(), &Filling::nothing())
        .text()
        .to_owned()
}

/// Whether a set of states from the bus carries a bit.
fn carries(states: &[u32], bit: u32) -> bool {
    let low = u64::from(states.first().copied().unwrap_or(0));
    let high = u64::from(states.get(1).copied().unwrap_or(0));
    (low | (high << 32)) & (1 << bit) != 0
}

/// Everything under `at`, itself first, as the reader reads it.
fn everything(reader: &AccessibleSession, at: &NodeAt) -> Vec<(NodeAt, Facts)> {
    let facts = reader.facts(at).expect("a thing the tree said was there");
    let mut all = vec![(at.clone(), facts.clone())];
    for child in &facts.children {
        all.extend(everything(reader, child));
    }
    all
}

/// The shell's own tree, on a real bus, read back by the agent's reader.
fn the_tree_read_back() -> (ASession, Vec<(NodeAt, Facts)>) {
    let session = ASession::started();
    let strings = words();
    // Every surface up at once is not a machine; it is how one reading of the
    // bus can be held to every surface the shell draws. Which are really up is
    // the shell's own answer, and `the_showing_bit_says_which_surfaces_are_up`
    // is where that is held.
    let tree = ReadAloudTree::of(&strings, &Surface::ALL);
    let bus = ReadAloudBus::serving(&session.accessibility, &tree).expect("the tree is served");
    bus.embedded().expect("the registry embedded this machine");
    let reader = session.reader();
    let ours = waiting_for(
        "this machine's own tree being listed by the registry",
        || {
            reader
                .applications()
                .ok()?
                .into_iter()
                .find(|running| running.at.holder == bus.answers_as())
        },
    );
    let read = everything(&reader, &ours.at);
    // The bus is kept alive until the reading is done, and no longer.
    drop(bus);
    (session, read)
}

/// **Every surface the shell draws is on the bus, with the role, the name and
/// the state `alo-access` decided** — read back through the registry by the
/// agent's own reader.
#[test]
fn every_surface_is_read_back_as_alo_access_decided_it() {
    let (_session, read) = the_tree_read_back();
    let strings = words();
    let named: Vec<(&str, u32)> = read
        .iter()
        .map(|(_, facts)| (facts.name.as_str(), facts.role))
        .collect();
    for surface in Surface::ALL {
        for control in surface.read_aloud() {
            let name = said(&strings, &control);
            let (_, role) = named
                .iter()
                .find(|(said, _)| *said == name)
                .unwrap_or_else(|| {
                    panic!("{surface:?}: nothing on the bus is called {name:?}");
                });
            let kind = Role::of_the_tree(*role);
            let expected = match control.role {
                alo_access::Role::Window | alo_access::Role::Dialogue => Role::Window,
                alo_access::Role::Button => Role::Button,
                alo_access::Role::Label => Role::Label,
                alo_access::Role::Entry => Role::TextField,
                alo_access::Role::PasswordEntry => Role::PasswordField,
                alo_access::Role::Switch => Role::Switch,
                // A list, a line of a list and a strip of state are laid out
                // rather than read or pressed, and the agent's reader calls
                // every one of those a part it walks through.
                alo_access::Role::List
                | alo_access::Role::ListItem
                | alo_access::Role::StatusBar => Role::Part,
            };
            assert_eq!(
                kind, expected,
                "{name:?} is read as the wrong kind of thing"
            );
        }
    }
}

/// **A password field is a password field on the bus**, which is what stops
/// anything reading it as a field whose contents are read.
#[test]
fn the_password_field_crosses_the_bus_as_a_password_field() {
    let (_session, read) = the_tree_read_back();
    let strings = words();
    let password = said(
        &strings,
        &Control {
            role: alo_access::Role::PasswordEntry,
            name: alo_access::words::THE_PASSWORD,
            state: alo_access::State::CanBeUsed,
        },
    );
    let (_, facts) = read
        .iter()
        .find(|(_, facts)| facts.name == password)
        .expect("the password field is on the bus");
    assert_eq!(Role::of_the_tree(facts.role), Role::PasswordField);
    assert!(
        !facts.has_text,
        "the password field offers its text to anything that asks"
    );
}

/// **Nothing arrives chosen** (ADR 0001): nothing on the bus reads as the
/// default, as focused, or as already on — on the approval surface above all,
/// where one of the two answers carries a change out.
#[test]
fn nothing_on_the_bus_reads_as_already_chosen() {
    let (_session, read) = the_tree_read_back();
    for (at, facts) in &read {
        for (bit, what) in [
            (IS_DEFAULT, "the default"),
            (FOCUSED, "focused"),
            (CHECKED, "already on"),
        ] {
            assert!(
                !carries(&facts.states, bit),
                "{} ({}) is read as {what}",
                facts.name,
                at.object
            );
        }
    }
}

/// **What a reader is offered is what the keyboard reaches**: everything
/// `alo-access` says can be used is focusable on the bus, and nothing else is.
#[test]
fn what_is_focusable_on_the_bus_is_where_the_keyboard_stops() {
    let (_session, read) = the_tree_read_back();
    let strings = words();
    for surface in Surface::ALL {
        for control in surface.read_aloud() {
            let name = said(&strings, &control);
            let (_, facts) = read
                .iter()
                .find(|(_, facts)| facts.name == name)
                .unwrap_or_else(|| panic!("{name:?} is not on the bus"));
            assert_eq!(
                carries(&facts.states, FOCUSABLE),
                Control::can_be_used(&control),
                "{name:?} is offered and unreachable, or reachable and not offered"
            );
        }
    }
}

/// **The approval surface is the sentence, then its two answers** — in that
/// order, on the bus, with nothing chosen (ADR 0001).
#[test]
fn the_approval_surface_reads_as_the_sentence_and_its_two_answers() {
    let (_session, read) = the_tree_read_back();
    let strings = words();
    let asked = said(
        &strings,
        &alo_access::the_approval_in_reading_order()[0].clone(),
    );
    let (_, dialogue) = read
        .iter()
        .find(|(_, facts)| facts.name == asked)
        .expect("the approval surface is on the bus");
    let inside: Vec<String> = dialogue
        .children
        .iter()
        .map(|child| {
            read.iter()
                .find(|(at, _)| at.object == child.object)
                .map(|(_, facts)| facts.name.clone())
                .unwrap_or_default()
        })
        .collect();
    let expected: Vec<String> = alo_access::the_approval_in_reading_order()
        .iter()
        .skip(1)
        .map(|control| said(&strings, control))
        .collect();
    assert_eq!(
        inside, expected,
        "the approval surface is read in another order"
    );
}
