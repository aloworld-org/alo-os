//! **The shell's tree, read by the library Orca reads it through.**
//!
//! `alo-shell`'s `the_tree_a_reader_finds` reads the shell's tree back with
//! `alo-adapters`, the agent's own reader — and it passed for weeks while Orca,
//! on a booted machine, logged *AT-SPI: Called
//! _atspi_dbus_return_accessible_from_message with strange signature so* for
//! every child it asked for, and *Error in GetItems … Unknown object
//! '/org/a11y/atspi/cache'* for the tree as a whole. The agent's reader is
//! built on zbus, as the shell is, and could not tell the shell's wrong answer
//! from a right one
//! (`docs/misreadings/a-reader-we-wrote-agreed-with-the-tree-we-wrote.md`).
//!
//! So here the reader is **libatspi itself**, the C library every screen reader
//! on this desktop reads through, called through its C API by this package's
//! own `alo-libatspi-walk` — the road Orca takes. Nothing in this file decides
//! what a right answer looks like: libatspi does, by warning on what it cannot
//! read and by reading it as nothing. The tree is walked twice, each time by a
//! fresh process, as a fresh screen reader would:
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
//! **What this needs** is what `the_tree_a_reader_finds` needs — `dbus-daemon`
//! and at-spi2's bus launcher — and libatspi's runtime library, which comes
//! with at-spi2-core. If any of it is missing the test fails rather than
//! skipping: a skipped test here would be green on a machine that proved
//! nothing.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

#[path = "../../alo-shell/tests/support/one_heavy_fixture_at_a_time.rs"]
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
use alo_shell::{ReadAloudBus, ReadAloudTree, SignInShows};
use alo_strings::{Filling, Strings};
use zbus::blocking::MessageIterator;
use zbus::blocking::connection::Builder;
use zbus::message::Type;

/// How long anything here is waited for.
const PATIENCE: Duration = Duration::from_secs(40);

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
            .environment(PathBuf::from(env!("CARGO_BIN_EXE_alo-libatspi-walk")))
            .arg(how)
            .output()
            .expect("the walk started");
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
    for (how, walked) in read_by_libatspi(&session, &tree) {
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
}

/// Every sentence the screens before sign-in say: the machine's, and the
/// shell's own two lists, which `alo-saying` cannot collect
/// (`alo_shell::what_this_machine_can_say` gives the reason).
fn the_words_before_sign_in() -> Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    alo_shell::sign_in_words::declare_sign_in_words(&mut vocabulary).unwrap();
    alo_shell::first_start_words::declare_first_start_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// Whether libatspi's walk read a thing called `name`.
fn read_a_thing_called(walked: &[String], name: &str) -> bool {
    walked
        .iter()
        .any(|line| line.ends_with(&format!("\t{name}")))
}

/// **libatspi reads the first screen of a fresh machine** — the question, its
/// three rows, the way on and *Accessibility* — as the shell serves it, and
/// says nothing about any of it (ADR 0155). Before this nothing was served
/// before sign-in at all, so a blind person could not set a machine up.
#[test]
fn libatspi_reads_the_first_screen_of_a_fresh_machine() {
    use alo_shell::first_start_words as say;

    let session = ASession::started();
    let strings = the_words_before_sign_in();
    let said = |word: alo_strings::Word| {
        strings
            .say(&word.key(), &Filling::nothing())
            .text()
            .to_owned()
    };
    let first_start = alo_shell::the_first_start::TheFirstStart::arriving(None);
    let tree = ReadAloudTree::before_sign_in(&strings, SignInShows::FirstStart(&first_start));
    let wanted = [
        said(say::OVERVIEW_TITLE),
        said(say::ROW_LANGUAGE),
        said(say::ROW_REGION),
        said(say::ROW_KEYBOARD),
        said(say::CONTINUE),
        said(alo_shell::sign_in_words::ACCESSIBILITY),
    ];
    for (how, walked) in read_by_libatspi(&session, &tree) {
        for name in &wanted {
            assert!(
                read_a_thing_called(&walked, name),
                "libatspi ({how}) did not read {name:?} on the first screen:\n{}",
                walked.join("\n")
            );
        }
    }
}

/// **libatspi reads the sign-in screen**: the greeting and the one field, and
/// nothing of what is typed into it.
#[test]
fn libatspi_reads_the_sign_in_screen() {
    let session = ASession::started();
    let strings = the_words_before_sign_in();
    let tree = ReadAloudTree::before_sign_in(
        &strings,
        SignInShows::Fields {
            whose: "disan",
            name: "",
            password_typed: true,
            field: alo_shell::SignInField::Password,
            said: None,
        },
    );
    let pin = strings
        .say(
            &alo_shell::sign_in_words::ENTER_YOUR_PIN.key(),
            &Filling::nothing(),
        )
        .text()
        .to_owned();
    for (how, walked) in read_by_libatspi(&session, &tree) {
        assert!(
            read_a_thing_called(&walked, &pin),
            "libatspi ({how}) did not read the PIN field:\n{}",
            walked.join("\n")
        );
        assert!(
            walked.iter().any(|line| line.contains("disan")),
            "libatspi ({how}) did not read the greeting:\n{}",
            walked.join("\n")
        );
    }
}

/// **What libatspi read of `tree`**, served on `session`'s accessibility bus:
/// each walk's lines, beside how it walked — after holding what must be true
/// of any tree alo serves. Both walks read the same tree the agent's reader
/// reads, libatspi says nothing on its standard error, and every answer on
/// the wire carries the signature at-spi2-core's XML gives it.
fn read_by_libatspi(session: &ASession, tree: &ReadAloudTree) -> Vec<(&'static str, Vec<String>)> {
    let bus = ReadAloudBus::serving(&session.accessibility, tree).expect("the tree is served");
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

    let mut walks = Vec::new();
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
        walks.push((how, walked));
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
    walks
}
