//! **The shell's tree, read by the library Orca reads it through.**
//!
//! `the_tree_a_reader_finds` reads the shell's tree back with `alo-adapters`,
//! the agent's own reader — and it passed for weeks while Orca, on a booted
//! machine, logged *AT-SPI: Called _atspi_dbus_return_accessible_from_message
//! with strange signature so* for every child it asked for, and *Error in
//! GetItems … Unknown object '/org/a11y/atspi/cache'* for the tree as a whole.
//! The agent's reader is built on zbus, and zbus reads a reply of two loose
//! values and a reply of one structure into the same Rust tuple, so it could
//! not tell the shell's wrong answer from a right one
//! (`docs/misreadings/a-reader-we-wrote-agreed-with-the-tree-we-wrote.md`).
//!
//! So here the reader is **libatspi itself**, the C library every screen reader
//! on this desktop reads through, driven by its own GObject bindings — the road
//! Orca takes. Nothing in this file decides what a right answer looks like:
//! libatspi does, by warning on what it cannot read and by reading it as
//! nothing. The tree is walked twice:
//!
//! - **one question at a time**, libatspi's cache off, which is
//!   `GetChildAtIndex`, `Parent`, `GetIndexInParent`, `GetRole` and `Name` for
//!   every node — the questions the walk on 2026-11-13 failed;
//! - **through the cache**, which is the one `GetItems` libatspi asks every
//!   application it meets, and what Orca reads once its main loop runs.
//!
//! Both walks must read the same tree the agent's reader reads, with every
//! child's parent the node that listed it, and libatspi must say nothing at all
//! on its standard error. A monitor on the accessibility bus holds the wire to
//! the same: each answer carries the signature at-spi2-core's XML gives it.
//!
//! **What this needs**, beyond `the_tree_a_reader_finds`': `python3`, PyGObject
//! and the `Atspi-2.0` typelib (`python3-gi`, `gir1.2-atspi-2.0` on Ubuntu).
//! The few lines of Python below are the bindings' calls and nothing else — the
//! library being exercised is libatspi, which is C, rented and unpatched. If any
//! of it is missing the test fails rather than skipping: a skipped test here
//! would be green on a machine that proved nothing.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

#[path = "support/one_heavy_fixture_at_a_time.rs"]
mod one_heavy_fixture_at_a_time;

use std::io::{BufRead as _, BufReader};
use std::os::unix::process::CommandExt as _;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use alo_access::{Surface, TurnedOn};
use alo_adapters::{AccessibilityTree, AccessibleSession, Facts, NodeAt};
use alo_portals::Sandboxes;
use alo_shell::{ReadAloudBus, ReadAloudTree};
use alo_strings::{Filling, Strings};
use zbus::blocking::MessageIterator;
use zbus::blocking::connection::Builder;
use zbus::message::Type;

/// How long anything here is waited for.
const PATIENCE: Duration = Duration::from_secs(40);

/// libatspi, through its bindings: find the application whose toolkit is
/// `alo`, let the main loop take in what it was sent (`GetItems`' answer among
/// it), and walk it — with the cache on when asked to. One line per thing:
/// depth, role, child count, index in parent, whether its parent is the thing
/// that listed it, and its name.
const WALK: &str = r#"
import sys
import gi
gi.require_version("Atspi", "2.0")
from gi.repository import Atspi, GLib

cached = sys.argv[1] == "cached"
Atspi.init()
context = GLib.MainContext.default()

def pump(seconds):
    until = GLib.get_monotonic_time() + int(seconds * 1000000)
    while GLib.get_monotonic_time() < until:
        context.iteration(False)

desktop = Atspi.get_desktop(0)
ours = None
for index in range(desktop.get_child_count()):
    application = desktop.get_child_at_index(index)
    if application is not None and application.get_toolkit_name() == "alo":
        ours = application
if ours is None:
    print("no application with the toolkit alo on the desktop")
    sys.exit(3)
pump(2)
if cached:
    ours.set_cache_mask(Atspi.Cache.DEFAULT)

def walk(node, depth, listed_by, as_index):
    count = node.get_child_count()
    parent = node.get_parent()
    index = node.get_index_in_parent()
    where = listed_by is None or (parent == listed_by and index == as_index)
    print("%d\t%d\t%d\t%d\t%s\t%s" % (
        depth,
        int(node.get_role()),
        count,
        index,
        "parent" if where else "stray",
        node.get_name(),
    ))
    for index in range(count):
        child = node.get_child_at_index(index)
        if child is None:
            print("missing\t%d\t%d" % (depth + 1, index))
            continue
        walk(child, depth + 1, node, index)

walk(ours, 0, None, -1)
"#;

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
/// accessibility bus and registry at-spi2 runs. The shape of
/// `the_tree_a_reader_finds`' own, process group and all, so that dropping it
/// reaches the bus the launcher forked and the registry that bus started.
struct ASession {
    /// Where its files are.
    place: PathBuf,
    /// Its session bus.
    session: String,
    /// Its accessibility bus.
    accessibility: String,
    /// What it started, killed when it is dropped.
    processes: Vec<Child>,
    /// The process group everything above runs in.
    group: Option<rustix::process::Pid>,
    /// Held while this session's processes are up; dropped last.
    _heavy: one_heavy_fixture_at_a_time::TheOnlyHeavyFixture,
}

impl ASession {
    /// A session of this test's own, started.
    fn started() -> Self {
        let heavy = one_heavy_fixture_at_a_time::TheOnlyHeavyFixture::held();
        let place = std::env::temp_dir().join(format!(
            "alo-libatspi-{}-{}",
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
            .process_group(0)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let group = rustix::process::Pid::from_child(&daemon);
        let printed = daemon.stdout.take();
        let mut this = Self {
            place,
            session: String::new(),
            accessibility: String::new(),
            processes: vec![daemon],
            group: Some(group),
            _heavy: heavy,
        };
        let mut session = String::new();
        BufReader::new(printed.unwrap())
            .read_line(&mut session)
            .unwrap();
        this.session = session.trim().to_owned();
        assert!(!this.session.is_empty(), "dbus-daemon printed no address");

        let launcher = found(
            &[
                "/usr/libexec/at-spi-bus-launcher",
                "/usr/lib/at-spi2-core/at-spi-bus-launcher",
                "/usr/lib/at-spi-bus-launcher",
            ],
            "at-spi-bus-launcher",
        );
        let mut command = this.environment(launcher);
        command
            .arg("--launch-immediately")
            .process_group(group.as_raw_nonzero().get())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        this.processes.push(command.spawn().unwrap());
        this.accessibility = waiting_for("the accessibility bus being announced", || {
            AccessibleSession::announced_on(&this.session).ok()
        });
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
            .env("DBUS_SESSION_BUS_ADDRESS", &self.session)
            .env("GSETTINGS_BACKEND", "memory")
            .env("LANG", "C.UTF-8");
        command
    }

    /// The agent's own reader of this session's tree.
    fn reader(&self) -> AccessibleSession {
        AccessibleSession::at(
            &self.accessibility,
            Sandboxes::under(self.place.join("proc")),
        )
        .expect("the accessibility bus this test started")
    }

    /// libatspi's walk of the shell's tree, and everything it said on its
    /// standard error.
    fn libatspi_walks(&self, how: &str) -> (Vec<String>, String) {
        let ran = self
            .environment(on_the_path("python3"))
            .args(["-I", "-c", WALK, how])
            .output()
            .expect("python3 started");
        let walked = String::from_utf8_lossy(&ran.stdout).into_owned();
        let said = String::from_utf8_lossy(&ran.stderr).into_owned();
        assert!(
            ran.status.success(),
            "libatspi's walk ({how}) did not finish: {:?}\n{walked}\n{said}",
            ran.status
        );
        (walked.lines().map(str::to_owned).collect(), said)
    }
}

impl Drop for ASession {
    /// Stop everything this session started, including what its children
    /// started — `the_tree_a_reader_finds` says why the group goes first.
    fn drop(&mut self) {
        if let Some(group) = self.group.take() {
            let _ = rustix::process::kill_process_group(group, rustix::process::Signal::KILL);
        }
        for process in &mut self.processes {
            let _ = process.kill();
            let _ = process.wait();
        }
        let _ = std::fs::remove_dir_all(&self.place);
    }
}

/// One message on the accessibility bus, as much of it as this test reads.
#[derive(Debug, Clone)]
struct Heard {
    /// Call, return, error or signal.
    kind: Type,
    /// Who sent it.
    sender: String,
    /// Who it was for.
    destination: String,
    /// What was called.
    member: String,
    /// Its own number.
    serial: u32,
    /// The call a return or an error answers.
    answers: Option<u32>,
    /// What its body is.
    signature: String,
}

/// Everything that crosses the accessibility bus from the moment this exists.
struct AMonitor {
    /// What was heard, in order.
    heard: Arc<Mutex<Vec<Heard>>>,
}

impl AMonitor {
    /// A monitor of the bus at `address`.
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
                    member: text(header.member().map(ToString::to_string)),
                    serial: header.primary().serial_num().get(),
                    answers: header.reply_serial().map(std::num::NonZeroU32::get),
                    signature: message.body().signature().to_string(),
                });
            }
        });
        Self { heard }
    }

    /// Every call made to `to`, and the signature of what answered it —
    /// `None` where nothing did, the member's name where an error did.
    fn answers_to(&self, to: &str) -> Vec<(String, Result<String, String>)> {
        // A moment for the last answers to reach the monitor's thread.
        std::thread::sleep(Duration::from_millis(500));
        let heard = self.heard.lock().unwrap().clone();
        heard
            .iter()
            .filter(|call| call.kind == Type::MethodCall && call.destination == to)
            .map(|call| {
                let answer = heard.iter().find(|reply| {
                    reply.sender == to
                        && reply.destination == call.sender
                        && reply.answers == Some(call.serial)
                });
                let said = match answer {
                    Some(reply) if reply.kind == Type::MethodReturn => Ok(reply.signature.clone()),
                    Some(reply) => Err(format!("{:?} {}", reply.kind, reply.signature)),
                    None => Err("no answer".to_owned()),
                };
                (call.member.clone(), said)
            })
            .collect()
    }
}

/// Everything under `at`, itself first, as the agent's reader reads it.
fn everything(reader: &AccessibleSession, at: &NodeAt, depth: usize) -> Vec<(usize, Facts)> {
    let facts = reader.facts(at).expect("a thing the tree said was there");
    let mut all = vec![(depth, facts.clone())];
    for child in &facts.children {
        all.extend(everything(reader, child, depth + 1));
    }
    all
}

/// One line of libatspi's walk, as the agent's reader would have it.
fn as_libatspi_says(depth: usize, facts: &Facts) -> String {
    format!(
        "{depth}\t{}\t{}\t*\tparent\t{}",
        facts.role,
        facts.children.len(),
        facts.name
    )
}

/// libatspi's line with the index in parent masked, which the agent's reader
/// does not ask — the walk's own `parent` column holds it instead, being
/// `stray` where a child's parent or index is not where it was listed.
fn masked(line: &str) -> String {
    let mut fields: Vec<&str> = line.splitn(6, '\t').collect();
    if fields.len() == 6 {
        fields[3] = "*";
    }
    fields.join("\t")
}

/// **libatspi reads the shell's whole tree — Settings among it — with the
/// names, roles and children the shell serves, one question at a time and
/// through the cache, and says nothing about any of it.**
#[test]
fn libatspi_reads_the_whole_tree_with_nothing_to_complain_of() {
    let session = ASession::started();
    let strings = Strings::of(alo_saying::everything_this_machine_can_say().unwrap());
    let tree = ReadAloudTree::of(&strings, &Surface::ALL, &TurnedOn::nothing());
    let bus = ReadAloudBus::serving(&session.accessibility, &tree).expect("the tree is served");
    let monitor = AMonitor::of(&session.accessibility);
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
    let expected: Vec<String> = everything(&reader, &ours.at, 0)
        .iter()
        .map(|(depth, facts)| as_libatspi_says(*depth, facts))
        .collect();
    assert!(expected.len() > 1, "the agent's reader read no tree");

    for how in ["one-question-at-a-time", "cached"] {
        let (walked, said) = session.libatspi_walks(how);
        assert!(
            said.trim().is_empty(),
            "libatspi complained while reading the tree ({how}):\n{said}"
        );
        let read: Vec<String> = walked.iter().map(|line| masked(line)).collect();
        assert_eq!(
            read, expected,
            "libatspi ({how}) read a different tree from the one served"
        );
        for control in Surface::Settings.read_aloud() {
            let name = strings
                .say(&control.name.key(), &Filling::nothing())
                .text()
                .to_owned();
            assert!(
                walked
                    .iter()
                    .any(|line| line.ends_with(&format!("\t{name}"))),
                "libatspi ({how}) did not read Settings' {name:?}"
            );
        }
    }

    let answers = monitor.answers_to(bus.answers_as());
    let of = |member: &str| -> Vec<Result<String, String>> {
        answers
            .iter()
            .filter(|(asked, _)| asked == member)
            .map(|(_, said)| said.clone())
            .collect()
    };
    let children = of("GetChildAtIndex");
    assert!(
        !children.is_empty(),
        "libatspi never asked for a child one at a time, so that road was not read"
    );
    assert!(
        children.iter().all(|said| said.as_deref() == Ok("(so)")),
        "GetChildAtIndex was not answered as (so): {children:?}"
    );
    let items = of("GetItems");
    assert!(
        !items.is_empty()
            && items
                .iter()
                .all(|said| said.as_deref() == Ok("a((so)(so)(so)iiassusau)")),
        "GetItems was not asked, or not answered as Cache.xml gives it: {items:?}"
    );
    for (asked, said) in &answers {
        assert!(said.is_ok(), "{asked} was refused: {said:?}");
    }
    drop(bus);
}
