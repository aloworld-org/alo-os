//! **Nothing leaves before the desktop is told.**
//!
//! Law 1: *every network egress an agent causes is visible at the moment it
//! happens.* CLAUDE.md promises it as a test — *no agent-caused egress escapes
//! the indicator* — and ADR 0134 is how the indicator reaches the screen: a
//! desktop sends `watch-what-leaves` on the person's door, and every change to
//! what is leaving is written to it **inside the call that made the change,
//! before the connection to the network is opened**.
//!
//! `alo-egress` tests that rule on an indicator alone, and `crate::watchers`
//! tests the writing over a socket pair. Neither is the daemon. This is: an
//! agent's question goes to a provider through the service `src/main.rs` runs,
//! and the provider — a listener this test owns — looks at what the desktop had
//! been told **at the moment the question's connection reached it**. The line
//! naming that provider must already be there; afterwards the list must be
//! empty again.
//!
//! # What runs, and what stands in
//!
//! | | |
//! |---|---|
//! | the service | `alo_agentd::starting::until_stopped`, the function `src/main.rs` calls, in a process of its own |
//! | the indicator and the desktops it tells | made **inside** that function, as on a machine — this file wires nothing |
//! | the person's settings | a real `settings.toml`, read from `$XDG_CONFIG_HOME` as on a machine |
//! | the provider | a listener on this machine's own address, which never answers |
//! | **which door a connection is on** | **told**, by `TheDoors` below |
//! | **the boundary** | **none**, by `NoKernelHere` below |
//!
//! **The two substitutions, and why each is honest.** A door is decided by the
//! login that connected, and a test is one login, so an agent's door cannot be
//! reached by a real connection from here; `alo_agentd::knocking` says the same
//! and `tests/two_doors_on_one_socket.rs` tests the real mapping. The boundary
//! needs root, a BPF filesystem and a kernel that started the BPF LSM, and it
//! is imposed **before** the indicator is asked (`alo_turn`'s
//! `a_question_that_leaves_this_machine`), so on a machine without one the
//! question is refused before anything could be shown — there would be nothing
//! to measure. What it decides is *which address*, not *who is told*, and
//! `tests/a_question_is_bounded_by_the_kernel.rs` measures it where it exists.
//!
//! The daemon runs in a child process — this test binary again, told so by
//! [`SERVING_AT`] — because `Questions::of_this_process` reads the person's
//! settings from the environment, and a test may not change its own.
//!
//! # What it does not prove
//!
//! Pixels. That the desktop draws what it is told is `alo-indicator`'s and the
//! compositor's. Nor does it prove anything about a desktop that stopped
//! reading: `crate::watchers` drops one rather than wait, by design.
//!
//! # It runs on every build
//!
//! As an ordinary user or as root, with no kernel feature beyond Unix sockets
//! and a non-loopback address — which every CI runner and every developer's
//! machine has. It is not ignored.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, ErrorKind, Read as _, Write as _};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, UdpSocket};
use std::os::fd::{AsFd as _, BorrowedFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use alo_agentd::knocking::Knocking;
use alo_agentd::names::{NothingKeepsNames, TheNames};
use alo_agentd::network::NothingKeepsPairings;
use alo_agentd::refusing::NotACaller;
use alo_agentd::rereading::{ThePersonsFile, WhatIsGranted};
use alo_agentd::stopping::Waking;
use alo_agentd::unix::{our_group, us};
use alo_agentd::wire::Wire;
use alo_agentd::{Bounds, Described, Lasting, Side, Sides, Uid, starting};
use alo_capability::{Grantee, Grants};
use alo_egress::{Destination, Reported, Why};
use alo_keeping::Keeping;
use alo_protocol::ToAPerson;
use alo_record::Record;

/// Set in the child process: where its socket goes. Its presence is what
/// makes a test in this file the daemon rather than the person.
const SERVING_AT: &str = "ALO_TEST_SERVING_AT";

/// The agent this machine holds turns for.
const THE_AGENT: &str = "@files";

/// The provider the person chose, by their own name for it.
const THE_PROVIDER: &str = "Somewhere";

/// The login this test gives the agent, which is not the one it runs as.
const AN_AGENT: u32 = 989;

/// The one it gives the agent if this test happens to run as [`AN_AGENT`].
const ANOTHER_AGENT: u32 = 990;

/// How long anything here waits before calling it a failure.
const PATIENCE: Duration = Duration::from_secs(20);

/// A folder of this test's own.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-agentd-told-first-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).expect("a temporary folder can be made");
    folder
}

/// This machine's own address on the interface it would route by.
///
/// Not loopback, because a provider at loopback is *this machine* and the
/// provider road refuses it before anything is shown. Connecting a datagram
/// socket picks the interface without sending anything.
fn our_own_address() -> IpAddr {
    let asking = UdpSocket::bind("0.0.0.0:0").expect("a socket of our own");
    asking
        .connect("192.0.2.1:9")
        .expect("this machine has a route somewhere, so it has an address that is not loopback");
    let ours = asking
        .local_addr()
        .expect("a socket knows its own address")
        .ip();
    assert!(
        !ours.is_loopback(),
        "this machine's only address is loopback, so no provider can be stood up on it"
    );
    ours
}

// ---------------------------------------------------------------------------
// The daemon's side: the child process.
// ---------------------------------------------------------------------------

/// A door that hands out real connections and is told which side each is on:
/// the person's first (the desktop watching), the agent's after it.
///
/// A real `UnixListener`, so what the service reads is a real connection. Past
/// the end of the list is a stranger, as on the real socket.
struct TheDoors {
    /// The socket.
    listener: UnixListener,
    /// How many connections have arrived.
    so_far: AtomicUsize,
}

impl Knocking for TheDoors {
    fn waiting_on(&self) -> BorrowedFd<'_> {
        self.listener.as_fd()
    }

    fn next(&self) -> Result<(Side, UnixStream), NotACaller> {
        let (connection, _) = self
            .listener
            .accept()
            .map_err(|why| NotACaller::NotAccepted { why })?;
        match self.so_far.fetch_add(1, Ordering::Relaxed) {
            0 => Ok((Side::Person, connection)),
            1 => Ok((Side::Agent, connection)),
            _ => Err(NotACaller::Stranger { uid: 65534 }),
        }
    }
}

/// No boundary: a departure is carried out where it stands.
///
/// Written here, where a reader of this test sees it, for the reason this
/// file's header gives.
struct NoKernelHere;

impl alo_turn::Bounding for NoKernelHere {
    fn carrying_out(
        &mut self,
        _reaching: &alo_files::Reaching,
        doing: alo_turn::Doing<'_>,
    ) -> Result<alo_turn::Done, alo_turn::NoBoundary> {
        Ok(doing.done())
    }

    fn carrying_out_a_departure(
        &mut self,
        _to: &[SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), alo_turn::NoBoundary> {
        doing();
        Ok(())
    }
}

/// When this process is the daemon, serve until the parent closes our standard
/// input, and answer `true`.
fn the_daemon_if_this_is_one() -> bool {
    let Some(at) = std::env::var_os(SERVING_AT) else {
        return false;
    };
    serving_as_a_machine_does(Path::new(&at));
    true
}

/// `starting::until_stopped`, handed what `src/main.rs` hands it with the two
/// substitutions this file's header names.
fn serving_as_a_machine_does(at: &Path) {
    let folder = at.parent().expect("the socket is in a folder");
    let person = us().expect("this process runs as somebody");
    let agent = Uid::of(if person.raw() == AN_AGENT {
        ANOTHER_AGENT
    } else {
        AN_AGENT
    })
    .expect("a login that is not root");
    let described = Described::of(
        Sides::of(
            person,
            agent,
            our_group().expect("this process has a group"),
        )
        .expect("two logins"),
        THE_AGENT,
        Lasting::of_seconds(3600, "turn").expect("an hour"),
        Lasting::of_seconds(3600, "proposal").expect("an hour"),
        &folder.join("record.jsonl"),
        Keeping::Forever,
        Bounds::nobodys(),
    )
    .expect("a machine described");
    let knocking = TheDoors {
        listener: UnixListener::bind(at).expect("a socket of our own"),
        so_far: AtomicUsize::new(0),
    };
    let (waking, stop) = Waking::made().expect("a way to stop");
    let asked_at = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).expect("a socket of our own");
    let wire = Wire::on(
        TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("a port of our own"),
        UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).expect("a socket of our own"),
        alo_nearby::MachineId::read("aaaabbbbccccddddeeeeffff00001111").expect("an identity"),
        asked_at
            .local_addr()
            .expect("a socket knows its port")
            .port(),
    )
    .expect("a wire");
    let strings = starting::what_this_machine_says()
        .expect("this machine's own words")
        .into_strings();
    let remembering = ThePersonsFile::at(&folder.join("nothing-granted.toml"));
    let mut grants = Grants::default();

    // The parent closing our standard input is the stop.
    thread::spawn(move || {
        let mut nothing = Vec::new();
        drop(std::io::stdin().read_to_end(&mut nothing));
        stop.stop();
    });

    starting::until_stopped(
        &described,
        &knocking,
        &waking,
        &wire,
        &strings,
        &mut WhatIsGranted::of(&mut grants, &remembering),
        alo_nearby::Pairings::none(),
        Box::new(NothingKeepsPairings),
        TheNames::remembering(
            alo_remembering::MachineNames::default(),
            Box::new(NothingKeepsNames),
        ),
        &mut NoKernelHere,
        &mut Record::default(),
    )
    .expect("the service served until it was asked to stop");
}

// ---------------------------------------------------------------------------
// The person's side: this test.
// ---------------------------------------------------------------------------

/// The daemon, running in a process of its own.
struct ADaemon {
    /// The process.
    child: Child,
    /// Where it listens.
    at: PathBuf,
}

impl ADaemon {
    /// This test binary again, as the daemon, for the test named `test`, with
    /// the person's settings at `config_home`.
    fn started(test: &str, folder: &Path, config_home: &Path) -> Self {
        let at = folder.join("agentd.sock");
        let child = Command::new(std::env::current_exe().expect("this test knows where it is"))
            .args(["--exact", test, "--nocapture", "--test-threads=1"])
            .env(SERVING_AT, &at)
            .env(alo_choosing::CONFIG_HOME, config_home)
            .env("HOME", folder)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("the daemon starts");
        Self { child, at }
    }

    /// The first connection, once the socket is there — which is the person's
    /// door, and is the desktop.
    fn first_connection(&mut self) -> UnixStream {
        let until = Instant::now() + PATIENCE;
        while Instant::now() < until {
            if let Ok(connection) = UnixStream::connect(&self.at) {
                return connection;
            }
            if let Ok(Some(ended)) = self.child.try_wait() {
                panic!("the daemon ended before it listened: {ended}");
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("the daemon never listened at {}", self.at.display());
    }

    /// Ask it to stop, and say whether it stopped as a service should.
    fn stopped(mut self) {
        drop(self.child.stdin.take());
        let ended = self.child.wait().expect("the daemon can be waited for");
        assert!(ended.success(), "the daemon did not stop cleanly: {ended}");
    }
}

impl Drop for ADaemon {
    /// A test that failed part way must not leave a daemon behind.
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        if matches!(self.child.try_wait(), Ok(None)) {
            drop(self.child.kill());
            drop(self.child.wait());
        }
    }
}

/// A desktop watching what leaves, and every list it has been told.
struct Watching {
    /// The connection the daemon gave over to the watchers.
    connection: UnixStream,
    /// Bytes of a line not finished yet.
    pending: Vec<u8>,
    /// Every list, in the order it arrived.
    told: Vec<Vec<Reported>>,
}

impl Watching {
    /// Ask to watch on this connection, and read the list as it stands.
    fn began(mut connection: UnixStream) -> Self {
        connection
            .write_all(b"{\"format\":1,\"asks\":{\"watch-what-leaves\":{}}}\n")
            .expect("the request is written");
        let mut watching = Self {
            connection,
            pending: Vec::new(),
            told: Vec::new(),
        };
        assert!(
            watching.until(|told| !told.is_empty()),
            "the daemon never answered watch-what-leaves"
        );
        watching
    }

    /// Take every whole line already on the connection, without waiting.
    ///
    /// **What *already there* means is the point of this test**: called at the
    /// moment the provider accepts, it reads exactly what the daemon had
    /// written before the question's connection was made.
    fn take_what_is_there(&mut self) {
        self.connection
            .set_nonblocking(true)
            .expect("a socket can be made not to wait");
        let mut buffer = [0u8; 4096];
        loop {
            match self.connection.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => self.heard(buffer.get(..read).unwrap_or_default()),
                Err(why) if why.kind() == ErrorKind::WouldBlock => break,
                Err(why) => panic!("the desktop's connection failed: {why}"),
            }
        }
        self.connection
            .set_nonblocking(false)
            .expect("a socket can be made to wait again");
    }

    /// Read until `enough` says so, or [`PATIENCE`] runs out; answer which.
    fn until(&mut self, enough: impl Fn(&[Vec<Reported>]) -> bool) -> bool {
        let until = Instant::now() + PATIENCE;
        self.waiting_until(until, enough)
    }

    /// The same, giving up at `until`.
    fn waiting_until(&mut self, until: Instant, enough: impl Fn(&[Vec<Reported>]) -> bool) -> bool {
        let mut buffer = [0u8; 4096];
        while !enough(&self.told) {
            let left = until.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            self.connection
                .set_read_timeout(Some(left))
                .expect("a socket can be given a timeout");
            match self.connection.read(&mut buffer) {
                Ok(0) => return enough(&self.told),
                Ok(read) => self.heard(buffer.get(..read).unwrap_or_default()),
                Err(why) if matches!(why.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                    return enough(&self.told);
                }
                Err(why) => panic!("the desktop's connection failed: {why}"),
            }
        }
        true
    }

    /// Bytes arrived: every whole line among them is a list.
    fn heard(&mut self, bytes: &[u8]) {
        self.pending.extend_from_slice(bytes);
        while let Some(end) = self.pending.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=end).collect();
            let line = String::from_utf8(line).expect("the daemon writes text");
            let told = ToAPerson::read(line.trim_end())
                .unwrap_or_else(|why| panic!("not a message: {why:?}: {line}"));
            let lines = told
                .lines_leaving()
                .unwrap_or_else(|| panic!("a desktop was told something other than a list: {line}"))
                .to_vec();
            self.told.push(lines);
        }
    }
}

/// Settings in which this person chose one provider, at this address, which
/// takes no key — so no keyring stands between the question and the socket.
fn a_person_who_chose(folder: &Path, provider: SocketAddr) -> PathBuf {
    let config = folder.join("config");
    let settings = config.join(alo_choosing::THE_FOLDER);
    std::fs::create_dir_all(&settings).expect("a settings folder can be made");
    std::fs::write(
        settings.join(alo_choosing::THE_SETTINGS),
        format!(
            "format = 2

[answers]
provider = {{ name = \"{THE_PROVIDER}\", model = \"a-model\" }}

[[provider]]
name = \"{THE_PROVIDER}\"
endpoint = \"https://{provider}\"
region = \"the EU\"
needs-a-key = false
"
        ),
    )
    .expect("the settings can be written");
    config
}

/// A provider that, the moment a connection reaches it, reads what the desktop
/// has been told — and then hangs up without answering.
///
/// Answers every list the desktop held at that moment, or [`None`] if nothing
/// ever connected.
fn a_provider_that_looks_at_the_desktop(
    listener: TcpListener,
    watching: Arc<Mutex<Watching>>,
) -> JoinHandle<Option<Vec<Vec<Reported>>>> {
    listener
        .set_nonblocking(true)
        .expect("a listener can be made not to wait");
    thread::spawn(move || {
        let until = Instant::now() + PATIENCE;
        while Instant::now() < until {
            if let Ok((connection, _)) = listener.accept() {
                let mut watching = watching.lock().unwrap_or_else(PoisonError::into_inner);
                watching.take_what_is_there();
                let seen = watching.told.clone();
                // Hung up unanswered: the handshake fails and the departure ends.
                drop(connection);
                return Some(seen);
            }
            thread::sleep(Duration::from_millis(5));
        }
        None
    })
}

/// The question an agent asks, as a line of the protocol.
const THE_QUESTION: &str =
    "{\"format\":1,\"asks\":{\"ask\":{\"question\":\"may the tenant sublet?\"}}}\n";

/// An agent's question, on the agent's door, and the line it was answered with.
fn an_agent_asks(at: &Path, question: &str) -> String {
    let mut agent = UnixStream::connect(at).expect("the agent's door");
    agent
        .write_all(question.as_bytes())
        .expect("the question is written");
    agent
        .set_read_timeout(Some(PATIENCE * 2))
        .expect("a socket can be given a timeout");
    let mut answer = String::new();
    BufReader::new(agent)
        .read_line(&mut answer)
        .expect("the agent is answered");
    answer
}

/// Whether this list is exactly one line: the agent asking the provider.
fn the_agent_asking_the_provider(lines: &[Reported]) -> bool {
    matches!(
        lines,
        [Reported::Agent {
            agent,
            why: Why::Asking,
            destination: Destination::Provider { provider, .. },
        }] if *agent == Grantee::named(THE_AGENT) && provider == THE_PROVIDER
    )
}

/// **The desktop is told before the question leaves, and told again when it
/// is over.**
///
/// The desktop watches from before the turn begins, so a line that arrived
/// late is distinguishable from one that arrived in time: the provider reads
/// what the desktop held at the moment the connection reached it, and the
/// daemon cannot have written anything after that and had it counted.
#[test]
fn the_desktop_is_told_before_an_agents_question_reaches_the_provider() {
    if the_daemon_if_this_is_one() {
        return;
    }
    told_before_it_reaches_the_provider(
        "the_desktop_is_told_before_an_agents_question_reaches_the_provider",
        "told-first",
        THE_QUESTION,
    );
}

/// **A follow-up's earlier conversation leaves under the same indicator**
/// (ADR 0138). It travels inside the agent's question and by no other road,
/// so the desktop is told before the connection that carries it reaches the
/// provider, exactly as for a first request — and it is sent only because an
/// agent asked, which is only because a person pressed Enter.
#[test]
fn a_follow_ups_earlier_conversation_is_told_to_the_desktop_before_it_leaves() {
    if the_daemon_if_this_is_one() {
        return;
    }
    let earlier = alo_instructing::TheConversationSoFar::of(vec![
        alo_instructing::AnEarlierTurn::ThePersonSaid("list the files in /home/anna".to_owned()),
        alo_instructing::AnEarlierTurn::AloAnswered("march.pdf and april.pdf".to_owned()),
    ]);
    let question = format!("{}\n\nThe request: and the April one too", earlier.told());
    let line = alo_protocol::FromAnAgent::Ask {
        question,
        answered: alo_protocol::Answered::AsTheNextRequest,
    }
    .written()
    .expect("a question can be written");
    told_before_it_reaches_the_provider(
        "a_follow_ups_earlier_conversation_is_told_to_the_desktop_before_it_leaves",
        "told-first-follow-up",
        &format!("{line}\n"),
    );
}

/// The guarantee, for one agent's question: what the desktop held at the moment
/// the question's connection reached the provider.
fn told_before_it_reaches_the_provider(test: &str, what: &str, question: &str) {
    let folder = a_folder_of_our_own(what);
    let listener =
        TcpListener::bind(SocketAddr::new(our_own_address(), 0)).expect("a listener of our own");
    let provider = listener.local_addr().expect("a listener knows where it is");
    let config = a_person_who_chose(&folder, provider);

    let mut daemon = ADaemon::started(test, &folder, &config);
    let watching = Arc::new(Mutex::new(Watching::began(daemon.first_connection())));
    let reached = a_provider_that_looks_at_the_desktop(listener, Arc::clone(&watching));

    let answer = an_agent_asks(&daemon.at, question);
    let seen = reached.join().expect("the provider's thread finishes");

    let mut watching = Arc::try_unwrap(watching)
        .unwrap_or_else(|_| panic!("the provider's thread still holds the desktop"))
        .into_inner()
        .unwrap_or_else(PoisonError::into_inner);
    let at_the_moment = seen.clone().unwrap_or_default().len();
    let over =
        watching.until(|told| told.len() > at_the_moment && told.last().is_some_and(Vec::is_empty));
    daemon.stopped();

    // **The control on the question itself**: if nothing reached the provider,
    // nothing below proves anything.
    let seen = seen.unwrap_or_else(|| {
        panic!("the agent's question never reached the provider, so nothing was measured: {answer}")
    });
    assert!(
        answer.contains("refused"),
        "a provider that never answered answered a question: {answer}"
    );

    // Before the turn, the desktop was told that nothing was leaving — which is
    // what makes *already there* mean *told in time* rather than *told at all*.
    assert_eq!(
        seen.first().map(Vec::as_slice),
        Some(&[][..]),
        "the desktop's first list was not the empty one: {seen:?}"
    );

    // **The guarantee.** At the moment the connection reached the provider,
    // the desktop held a line naming it.
    assert!(
        seen.last()
            .is_some_and(|lines| the_agent_asking_the_provider(lines.as_slice())),
        "the question reached the provider before the desktop was told it was leaving: {seen:?}"
    );

    // And afterwards the line came off.
    assert!(
        over,
        "the desktop was never told the question was over: {:?}",
        watching.told
    );
}

/// **The control: a turn whose question goes nowhere tells the desktop
/// nothing.**
///
/// The same daemon, the same desktop and the same agent's question, on a
/// machine where the person chose nothing to answer questions. Every list the
/// desktop receives is empty — so the reader above does not find a line where
/// there was no departure, and the test above cannot pass by always seeing one.
#[test]
fn a_question_that_goes_nowhere_tells_the_desktop_nothing() {
    if the_daemon_if_this_is_one() {
        return;
    }
    let folder = a_folder_of_our_own("nowhere");
    let nothing_chosen = folder.join("config");
    std::fs::create_dir_all(&nothing_chosen).expect("an empty settings folder can be made");

    let mut daemon = ADaemon::started(
        "a_question_that_goes_nowhere_tells_the_desktop_nothing",
        &folder,
        &nothing_chosen,
    );
    let mut watching = Watching::began(daemon.first_connection());
    let answer = an_agent_asks(&daemon.at, THE_QUESTION);

    // Anything more the daemon was going to tell has had a second to arrive.
    let _ = watching.waiting_until(Instant::now() + Duration::from_secs(1), |_| false);
    daemon.stopped();

    assert!(
        answer.contains("refused"),
        "a machine with nothing chosen answered a question: {answer}"
    );
    assert!(
        !watching.told.is_empty() && watching.told.iter().all(Vec::is_empty),
        "the desktop was told something left when nothing did: {:?}",
        watching.told
    );
}
