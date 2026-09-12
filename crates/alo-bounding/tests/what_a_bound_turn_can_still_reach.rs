//! What a bound turn can still reach, reproduced rather than argued — and the
//! two ways it no longer can.
//!
//! `what_a_turn_can_reach_on_the_network.rs` holds the two halves of law 1
//! meeting: a file nobody granted refused, and a destination nobody showed
//! refused, by the same boundary in the same breath. **This file is the other
//! list** — the ways a bound turn could still reach something past that, each
//! one run against the real loaded programme so that a gap is a failing
//! expectation somebody can watch rather than a paragraph in a report.
//!
//! Each is asserted in the direction it behaves **today**. When one of them was
//! closed, its assertion failed and said where to come and what to change,
//! which is what the rename and delete gaps did before they were closed, and
//! what the two socket gaps below did on 2026-09-12.
//!
//! # Three ways, and what became of each
//!
//! - **A socket that was already open.** `socket_connect` decides when a
//!   connection is *made*, so a socket connected before the turn began was
//!   invisible to it and stayed writable inside. **Closed**: `socket_sendmsg`
//!   decides on every message, asked of the sending thread's control group, so
//!   the inherited connection is refused the moment the turn writes on it —
//!   and a connection inherited to a destination the turn *was* shown carries
//!   on, which is the test that stops the first from being a boundary that
//!   refuses everything and looks like it works.
//! - **A datagram sent without connecting.** `sendto` on an unconnected socket
//!   reaches no `connect` hook at all. **Closed** by the same hook: the address
//!   the message names is what is decided about, a datagram to a destination
//!   the turn was shown still goes, and one to loopback goes unshown for the
//!   reason `socket_connect` gives.
//! - **A proxy on loopback.** Loopback is deliberately unchecked (ADR 0007) so
//!   that a model on this machine works. A proxy listening there and forwarding
//!   elsewhere is therefore a bound turn's way out, and this is the one of the
//!   three that a person could set up today without writing any code. **Still
//!   open**, and reproduced below as it was: closing it needs enforcement that
//!   is not turn-scoped, which is ADR 0021's decision and not this hook's.
//!
//! `docs/autonomy/updates/sockets-and-datagrams-inside-the-boundary.md` is
//! where the two closures are placed against the plan, and
//! `docs/autonomy/updates/publication-hardening-and-egress-coverage.md` where
//! all three were first placed against a release.
//!
//! # Nothing here reaches a network
//!
//! Every address is a listener **this test owns**, on a port the operating
//! system chose, on either loopback or this machine's own interface. The
//! "elsewhere" the proxy forwards to is this same machine on `eth0` — which is
//! not loopback, and is therefore something the boundary really rules on, and is
//! still inside this virtual machine. Nothing resolves a name, nothing leaves,
//! and no host-wide networking is touched. A refusal here is `EACCES` from the
//! kernel before a byte is sent, and the listener on the other end hearing
//! nothing is asserted beside it.
//!
//! # The control comes first, and it is not decoration
//!
//! A turn that reached something proves nothing if the boundary was never
//! applied — an unbounded process reaches everything too. So the same child in
//! the same turn is first refused a destination nobody showed it, and every
//! result below is thrown away rather than believed unless that happened.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    env, fs,
    io::{BufRead as _, BufReader, Read as _, Write as _},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use alo_bounding::{Cgroup, Departure, Departures, Family, places_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Where the child is told its turn's control group is.
const THE_CGROUP: &str = "ALO_STILL_CGROUP";

/// A destination the child must be refused, whatever else it manages.
const THE_CONTROL: &str = "ALO_STILL_CONTROL";

/// Which of the subjects the child is being asked to do.
const THE_SUBJECT: &str = "ALO_STILL_SUBJECT";

/// Where the subject reaches, whatever reaching means for it.
const THE_ADDRESS: &str = "ALO_STILL_ADDRESS";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says about the control, followed by `allowed` or a number.
const THE_CONTROL_WENT: &str = "alo:control ";

/// What the child says about the subject, followed by `allowed` or a number.
const THE_SUBJECT_WENT: &str = "alo:subject ";

/// What travels, so a server can say it was this and not a stray connection.
const THE_BYTES: &[u8] = b"a question nobody was shown\n";

/// `EACCES`, which is the only thing a refusal by this boundary looks like.
const REFUSED: i32 = 13;

/// Long enough for a machine under load, short enough that a hang is a failure.
const NOT_FOREVER: Duration = Duration::from_secs(10);

/// Long enough to be sure a datagram that was going to arrive has, and short
/// enough that waiting for one that was refused does not hold the suite.
const A_MOMENT: Duration = Duration::from_secs(2);

/// What one attempt came to.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The machine allowed it.
    Allowed,

    /// The machine refused, with the number it gave.
    Refused(i32),
}

impl Outcome {
    /// What the child wrote, read back.
    fn from(text: &str) -> Self {
        text.parse().map_or(Self::Allowed, Self::Refused)
    }
}

/// This machine's own address on the interface it has, which is not loopback.
///
/// Connecting a datagram socket to somewhere unreachable picks the interface
/// this machine would use and tells us its address, without sending anything.
fn our_own_address() -> IpAddr {
    let asking = UdpSocket::bind("0.0.0.0:0").expect("a socket of our own");
    asking
        .connect("192.0.2.1:9")
        .expect("a route to somewhere can be chosen");
    asking
        .local_addr()
        .expect("a socket knows its own address")
        .ip()
}

/// A destination as the map holds one, from an address a listener of ours has.
fn shown(at: SocketAddr) -> Departure {
    match at.ip() {
        IpAddr::V4(four) => Departure::of(Family::Four, u128::from(four.to_bits()), at.port()),
        IpAddr::V6(six) => Departure::of(Family::Six, six.to_bits(), at.port()),
    }
}

/// A listener of this test's own that reads one message and hands it back.
///
/// [`None`] when the connection arrived and the message never did — which is
/// what a refused write looks like from the other end.
fn a_server_at(address: IpAddr) -> (SocketAddr, thread::JoinHandle<Option<Vec<u8>>>) {
    let listener = TcpListener::bind(SocketAddr::new(address, 0)).expect("a listener of our own");
    let at = listener.local_addr().expect("a listener knows where it is");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().ok()?;
        stream.set_read_timeout(Some(NOT_FOREVER)).ok()?;
        let mut read = vec![0u8; THE_BYTES.len()];
        stream.read_exact(&mut read).ok()?;
        // Answered so that the writer's own `write` cannot be the only witness.
        stream.write_all(b"heard\n").ok()?;
        Some(read)
    });
    (at, handle)
}

/// A datagram listener of this test's own, which gives up after a moment.
fn a_datagram_listener_at(address: IpAddr) -> UdpSocket {
    let listening =
        UdpSocket::bind(SocketAddr::new(address, 0)).expect("a datagram socket of our own");
    listening
        .set_read_timeout(Some(A_MOMENT))
        .expect("a socket can be made to give up");
    listening
}

/// What one datagram listener heard, if anything.
fn heard_by(listening: &UdpSocket) -> Option<Vec<u8>> {
    let mut arrived = vec![0u8; THE_BYTES.len()];
    let (read, _) = listening.recv_from(&mut arrived).ok()?;
    arrived.truncate(read);
    Some(arrived)
}

/// A relay on loopback that carries whatever it is given to somewhere else.
///
/// Somebody's own proxy, in eleven lines. It runs in **this** process, which is
/// in no control group, so its outbound connection is not a turn's and nothing
/// decides about it — which is precisely the shape of the hole.
fn a_proxy_on_loopback_to(elsewhere: SocketAddr) -> (SocketAddr, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener can be made");
    let at = listener.local_addr().expect("a listener knows where it is");
    let handle = thread::spawn(move || {
        let Ok((mut from, _)) = listener.accept() else {
            return;
        };
        let Ok(mut onward) = TcpStream::connect_timeout(&elsewhere, NOT_FOREVER) else {
            return;
        };
        drop(from.set_read_timeout(Some(NOT_FOREVER)));
        let mut carried = vec![0u8; THE_BYTES.len()];
        if from.read_exact(&mut carried).is_ok() {
            drop(onward.write_all(&carried));
            drop(onward.flush());
        }
        drop(from.write_all(b"heard\n"));
    });
    (at, handle)
}

/// One turn: a cgroup, a grant, the destinations it was shown, and a child
/// inside it.
///
/// What comes back is the control's outcome and the subject's, in that order.
/// The grant is a folder of this test's own so that the bound is a real one —
/// `places_of` refuses a path that is not there rather than writing zeroes.
fn a_turn(named: &str, subject: &str, address: &str, shown_them: Departures) -> (Outcome, Outcome) {
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel(named);
    let granted = PathBuf::from("/tmp").join(format!("alo-still-{}-{named}", std::process::id()));
    let _ = fs::remove_dir_all(&granted);
    fs::create_dir_all(&granted).expect("a temporary directory can be made");

    // A destination nobody showed this turn, which it must be refused. It is on
    // this machine's own interface rather than loopback, because loopback is
    // not checked and a control that was never going to be refused is not one.
    let (control, refusing) = a_server_at(our_own_address());

    let cgroup = Cgroup::made(named).expect("a control group can be made");
    let turn = cgroup.id().expect("a control group has an identifier");

    let mut child = Command::new(env::current_exe().expect("a test binary knows where it is"))
        .args([
            "--exact",
            "--ignored",
            "--nocapture",
            "the_work_a_turn_does",
        ])
        .env(THE_CGROUP, cgroup.at())
        .env(THE_CONTROL, control.to_string())
        .env(THE_SUBJECT, subject)
        .env(THE_ADDRESS, address)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("this binary can be run again");

    let mut saying = BufReader::new(child.stdout.take().expect("stdout was asked for"));
    let mut telling = child.stdin.take().expect("stdin was asked for");

    let ready = next_line_from(&mut saying);
    assert_eq!(ready.trim(), READY, "the child never reached its cgroup");

    // Bound to one folder and shown exactly what the test said. Every other
    // destination is refused from here, which is what makes the control
    // meaningful.
    let places = places_of(&[granted.as_path()]).expect("the granted folder is there");
    kernel
        .boundary
        .bound(turn, places.and_shown(shown_them))
        .expect("the kernel takes the entry");

    telling.write_all(b"go\n").expect("the child is listening");
    telling.flush().expect("the child is listening");

    let control_went = said_about(&mut saying, THE_CONTROL_WENT);
    let subject_went = said_about(&mut saying, THE_SUBJECT_WENT);

    child.wait().expect("the child finishes");
    kernel
        .boundary
        .released(turn)
        .expect("the kernel takes it back");
    cgroup
        .removed()
        .expect("an empty control group can be taken away");
    drop(refusing);
    let _ = fs::remove_dir_all(&granted);
    (control_went, subject_went)
}

/// One turn shown nothing, which is the state every turn begins in.
fn a_turn_shown_no_destination(named: &str, subject: &str, address: &str) -> (Outcome, Outcome) {
    a_turn(named, subject, address, Departures::none())
}

/// The next line the child says that begins with this, read as an outcome.
fn said_about(saying: &mut BufReader<std::process::ChildStdout>, about: &str) -> Outcome {
    let said = next_line_from(saying);
    let said = said.trim();
    match said.strip_prefix(about) {
        Some(what) => Outcome::from(what),
        None => panic!("the child said `{said}` where `{about}...` was expected"),
    }
}

/// The next line the child says that is meant for us.
fn next_line_from(saying: &mut BufReader<std::process::ChildStdout>) -> String {
    let mut line = String::new();
    loop {
        line.clear();
        let read = saying.read_line(&mut line).expect("the child is talking");
        assert!(
            read > 0,
            "the child stopped talking before it said anything"
        );
        if line.starts_with("alo:") {
            return line;
        }
    }
}

/// **A connection made before the turn began is refused the moment the turn
/// writes on it.**
///
/// The child opens the socket, *then* joins the control group, and the boundary
/// is written after that. `socket_connect` never saw the connection — and
/// `socket_sendmsg` sees the write, asks whose control group is writing, and
/// refuses it with `EACCES` before a byte goes. The server accepted the
/// connection, because that happened outside the turn, and heard nothing.
///
/// Until 2026-09-12 this asserted the opposite, and it was the network's
/// version of the already-open-descriptor gap. A socket was the first of the
/// two to be closed, because a message has a destination and a read has none;
/// a file descriptor opened before a turn began followed the same day, decided
/// by `file_permission` on every use — `what_a_turn_inherits.rs` holds that
/// half, and this file holds the sockets: `a socket already connected` is the
/// row of that table that lives here.
#[test]
fn a_connection_made_before_the_boundary_is_refused_inside_it() {
    let (server, listening) = a_server_at(our_own_address());
    let (control, subject) =
        a_turn_shown_no_destination("alo-still-early", "early", &server.to_string());

    assert_eq!(
        control,
        Outcome::Refused(REFUSED),
        "the boundary was not in force, so what the inherited socket did means nothing"
    );
    assert_eq!(
        subject,
        Outcome::Refused(REFUSED),
        "a bound turn wrote on a connection made before the turn began, to a destination \
         nobody showed it, and the kernel let the bytes go — socket_sendmsg is not deciding, \
         or is not asking the sending thread's control group"
    );
    assert_eq!(
        listening.join().expect("the server thread finishes"),
        None,
        "the message arrived, so it was not refused before it left"
    );
}

/// **And a connection inherited to a destination the turn was shown carries
/// on.** The test that stops the one above from being a boundary that refuses
/// every inherited socket and looks like it works: what is decided is where the
/// bytes go, and a destination the person was shown is permitted however the
/// socket to it came to exist.
#[test]
fn a_connection_made_before_the_boundary_to_a_shown_destination_carries_on() {
    let (server, listening) = a_server_at(our_own_address());
    let (control, subject) = a_turn(
        "alo-still-early-shown",
        "early",
        &server.to_string(),
        Departures::of(&[shown(server)]).expect("one is not too many"),
    );

    assert_eq!(
        control,
        Outcome::Refused(REFUSED),
        "the boundary was not in force, so what the inherited socket did means nothing"
    );
    assert_eq!(
        subject,
        Outcome::Allowed,
        "a bound turn was refused a write to the destination it was shown, so the message \
         hook is refusing more than the connect hook permits and a question could never be sent"
    );
    assert_eq!(
        listening.join().expect("the server thread finishes"),
        Some(THE_BYTES.to_vec()),
        "the message did not arrive, so this proves nothing either way"
    );
}

/// **A datagram sent without connecting is refused, and nothing arrives.**
///
/// `sendto` on an unconnected socket makes no `connect`, so `socket_connect`
/// never runs. `socket_sendmsg` runs on the message, reads the address it
/// names, and refuses it with `EACCES` — a kernel refusal, not an egress event:
/// `alo-egress` was never involved, nothing was shown, nothing was recorded by
/// this test as leaving, and the listener heard nothing.
///
/// The parent then sends the same datagram from outside any turn and it
/// arrives, which is two things at once: a process that is not a turn is
/// unaffected, and the listener the turn was refused was alive to hear it.
#[test]
fn an_unconnected_datagram_is_refused_inside_a_bound_turn() {
    let listening = a_datagram_listener_at(our_own_address());
    let at = listening.local_addr().expect("a socket knows where it is");

    let (control, subject) =
        a_turn_shown_no_destination("alo-still-datagram", "datagram", &at.to_string());

    assert_eq!(
        control,
        Outcome::Refused(REFUSED),
        "the boundary was not in force, so what the datagram did means nothing"
    );
    assert_eq!(
        subject,
        Outcome::Refused(REFUSED),
        "a bound turn sent a datagram to a destination nobody showed it without connecting, \
         and the kernel let it go — socket_sendmsg is not reading the address a message names"
    );
    assert_eq!(
        heard_by(&listening),
        None,
        "the datagram arrived, so it was not refused before it left"
    );

    let ours = UdpSocket::bind("0.0.0.0:0").expect("a socket of our own");
    ours.send_to(THE_BYTES, at)
        .expect("this process is not a turn, and sends as it always could");
    assert_eq!(
        heard_by(&listening),
        Some(THE_BYTES.to_vec()),
        "a datagram from a process that is not a turn did not arrive, so either the listener \
         was never reachable or the boundary is deciding about processes that are not turns"
    );
}

/// **And a datagram to a destination the turn was shown still goes.** One
/// departure is one address and one port, and it permits a datagram there
/// exactly as it permits a connection.
#[test]
fn a_datagram_to_a_shown_destination_still_goes() {
    let listening = a_datagram_listener_at(our_own_address());
    let at = listening.local_addr().expect("a socket knows where it is");

    let (control, subject) = a_turn(
        "alo-still-datagram-shown",
        "datagram",
        &at.to_string(),
        Departures::of(&[shown(at)]).expect("one is not too many"),
    );

    assert_eq!(
        control,
        Outcome::Refused(REFUSED),
        "the boundary was not in force, so what the datagram did means nothing"
    );
    assert_eq!(
        subject,
        Outcome::Allowed,
        "a bound turn was refused a datagram to the destination it was shown"
    );
    assert_eq!(
        heard_by(&listening),
        Some(THE_BYTES.to_vec()),
        "the datagram was allowed and never arrived"
    );
}

/// **A datagram to this machine goes without having been shown**, for the
/// reason `socket_connect` gives: ADR 0007 makes a model on this machine the
/// default and `alo-egress` does not call a question answered here a departure
/// at all. The message hook keeps the same exemption, with the same cost.
#[test]
fn a_datagram_to_this_machine_goes_without_having_been_shown() {
    let listening = a_datagram_listener_at(IpAddr::from([127, 0, 0, 1]));
    let at = listening.local_addr().expect("a socket knows where it is");

    let (control, subject) =
        a_turn_shown_no_destination("alo-still-datagram-loopback", "datagram", &at.to_string());

    assert_eq!(
        control,
        Outcome::Refused(REFUSED),
        "the boundary was not in force, so what the datagram did means nothing"
    );
    assert_eq!(
        subject,
        Outcome::Allowed,
        "loopback was refused to a bound turn's datagram, which breaks a model running on this \
         machine — ADR 0007 says it must not be"
    );
    assert_eq!(
        heard_by(&listening),
        Some(THE_BYTES.to_vec()),
        "the datagram was allowed and never arrived"
    );
}

/// **A proxy on loopback carries a bound turn somewhere nobody showed it.**
///
/// The one of the three a person could set up today without writing code, and
/// the one that is production-reachable: a provider endpoint pointed at a local
/// proxy is an ordinary thing to configure, and `alo_models::address` treats
/// loopback at face value on purpose.
///
/// The turn connects to `127.0.0.1`, which is unchecked by ADR 0007 so that a
/// model on this machine works, and the proxy — a process in no control group —
/// carries the bytes to this machine's own `eth0` address, which the boundary
/// would have refused the turn directly. **The far server receiving them is the
/// assertion**: a bound turn reached a destination it was never shown.
///
/// Closing it needs enforcement that is not turn-scoped, which is a decision
/// nobody has taken and which the network-request approval explicitly did not
/// cover. The message hook changes nothing here: the turn's own writes go to
/// loopback, and the proxy's go from a process that is not a turn.
#[test]
fn a_proxy_on_loopback_carries_a_bound_turn_somewhere_nobody_showed_it() {
    let (elsewhere, listening) = a_server_at(our_own_address());
    let (proxy, relaying) = a_proxy_on_loopback_to(elsewhere);

    let (control, subject) =
        a_turn_shown_no_destination("alo-still-proxy", "proxy", &proxy.to_string());

    assert_eq!(
        control,
        Outcome::Refused(REFUSED),
        "the boundary was not in force, so what went through the proxy means nothing"
    );
    assert_eq!(
        subject,
        Outcome::Allowed,
        "loopback was refused to a bound turn, which would break a model running on this \
         machine — ADR 0007 says it must not be, so this is a real regression rather than a \
         gap being closed"
    );

    relaying.join().expect("the proxy thread finishes");
    assert_eq!(
        listening.join().expect("the server thread finishes"),
        Some(THE_BYTES.to_vec()),
        "the far server heard nothing, so the proxy did not carry it and this test proves \
         nothing about the hole it is named after"
    );
}

/// **A bound turn may reach a Unix socket nobody showed it, and write on it.**
///
/// Deliberate, and `deciding.rs` says why: *a Unix socket is not egress, and
/// refusing it would be enforcing something no policy claims.* `alo-egress`
/// decides about what leaves the machine, and a local socket does not. The
/// write matters as much as the connect since 2026-09-12: the daemon answers
/// the person on a Unix socket it opened before any turn, and a message hook
/// that refused it would refuse the person their own answer.
///
/// It is reproduced here because of what it means for **where a credential is
/// kept**. A store reached over a Unix socket — the D-Bus session bus, a
/// keyring daemon's own socket — is reachable *from inside a bound turn*, by
/// the turn, directly. The boundary offers no protection there at all, and an
/// architecture that assumed "D-Bus, therefore isolated" would be assuming the
/// opposite of what this machine does.
///
/// [ADR 0022](../../../docs/decisions/0022-where-a-providers-key-is-kept.md)
/// carries the consequence: what protects a credential is that it is fetched
/// **outside** the boundary and crosses in as a value that cannot be read —
/// never that the store is hard to reach from inside.
///
/// The socket is one this test owns, in a directory of its own, and nothing
/// listens on it beyond accepting once and reading what arrives.
#[test]
fn a_bound_turn_may_reach_a_unix_socket_nobody_showed_it() {
    let at = PathBuf::from("/tmp").join(format!("alo-still-unix-{}", std::process::id()));
    drop(fs::remove_file(&at));
    let listening = std::os::unix::net::UnixListener::bind(&at).expect("a socket of our own");
    let heard = thread::spawn(move || {
        let (mut stream, _) = listening.accept().ok()?;
        stream.set_read_timeout(Some(NOT_FOREVER)).ok()?;
        let mut read = vec![0u8; THE_BYTES.len()];
        stream.read_exact(&mut read).ok()?;
        Some(read)
    });

    let (control, subject) =
        a_turn_shown_no_destination("alo-still-unix", "unix", &at.to_string_lossy());

    assert_eq!(
        control,
        Outcome::Refused(REFUSED),
        "the boundary was not in force, so what the Unix socket did means nothing"
    );
    assert_eq!(
        subject,
        Outcome::Allowed,
        "a bound turn was refused a Unix socket — which would be this machine enforcing \
         something no policy claims, and would break every local socket alo OS uses"
    );
    assert_eq!(
        heard.join().expect("the listener thread finishes"),
        Some(THE_BYTES.to_vec()),
        "the connection was allowed and the message never arrived"
    );
    drop(fs::remove_file(&at));
}

/// The work of a turn, which is a second process because a cgroup holds
/// processes.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name. Run without the environment that names a cgroup it does nothing.
#[test]
#[ignore = "this is the child half of the tests above, and each parent runs it by name"]
fn the_work_a_turn_does() {
    let (Ok(cgroup), Ok(control), Ok(subject), Ok(address)) = (
        env::var(THE_CGROUP),
        env::var(THE_CONTROL),
        env::var(THE_SUBJECT),
        env::var(THE_ADDRESS),
    ) else {
        return;
    };
    // A path for the Unix subject, an address for every other one — parsed
    // where it is an address, so that a path is never silently read as one.
    let reaching: SocketAddr = if subject == "unix" {
        SocketAddr::from(([127, 0, 0, 1], 9))
    } else {
        address.parse().expect("the parent named an address")
    };

    // **Before the control group**, for the one subject that is about what a
    // turn inherits. Everything else opens what it opens from inside.
    let inherited = (subject == "early").then(|| {
        TcpStream::connect_timeout(&reaching, NOT_FOREVER).expect("a connection made outside")
    });

    fs::write(
        std::path::Path::new(&cgroup).join("cgroup.procs"),
        std::process::id().to_string(),
    )
    .expect("a process can put itself in a control group");

    let mut saying = std::io::stdout();
    writeln!(saying, "{READY}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    let mut go = String::new();
    std::io::stdin()
        .read_line(&mut go)
        .expect("the parent says when");

    // From here the boundary is in force. The control first, because it is what
    // says the boundary is in force at all.
    let control: SocketAddr = control.parse().expect("the parent named an address");
    let went = match TcpStream::connect_timeout(&control, NOT_FOREVER) {
        Ok(_) => "allowed".to_owned(),
        Err(why) => why.raw_os_error().unwrap_or(0).to_string(),
    };
    writeln!(saying, "{THE_CONTROL_WENT}{went}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    let went = match subject.as_str() {
        // A conversation on a socket that was open before any of this.
        "early" => {
            let mut held = inherited.expect("the connection made outside");
            match held.write_all(THE_BYTES).and_then(|()| held.flush()) {
                Ok(()) => "allowed".to_owned(),
                Err(why) => why.raw_os_error().unwrap_or(0).to_string(),
            }
        }
        // One datagram, to the address the parent named, on a socket that
        // never connects to anything.
        "datagram" => match UdpSocket::bind("0.0.0.0:0")
            .and_then(|sending| sending.send_to(THE_BYTES, reaching))
        {
            Ok(_) => "allowed".to_owned(),
            Err(why) => why.raw_os_error().unwrap_or(0).to_string(),
        },
        // A Unix socket, which is not egress and is not this programme's to
        // decide about — connected to and written on. `ALO_STILL_ADDRESS` is a
        // path rather than an address for this one.
        "unix" => match std::os::unix::net::UnixStream::connect(&address)
            .and_then(|mut stream| stream.write_all(THE_BYTES).and_then(|()| stream.flush()))
        {
            Ok(()) => "allowed".to_owned(),
            Err(why) => why.raw_os_error().unwrap_or(0).to_string(),
        },
        // Loopback, which is unchecked, to something that forwards.
        _ => match TcpStream::connect_timeout(&reaching, NOT_FOREVER) {
            Ok(mut through) => match through.write_all(THE_BYTES).and_then(|()| through.flush()) {
                Ok(()) => "allowed".to_owned(),
                Err(why) => why.raw_os_error().unwrap_or(0).to_string(),
            },
            Err(why) => why.raw_os_error().unwrap_or(0).to_string(),
        },
    };
    writeln!(saying, "{THE_SUBJECT_WENT}{went}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");
}
