//! What a bound turn can still reach, reproduced rather than argued.
//!
//! `what_a_turn_can_reach_on_the_network.rs` holds the two halves of law 1
//! meeting: a file nobody granted refused, and a destination nobody showed
//! refused, by the same boundary in the same breath. **This file is the other
//! list** — the three ways a bound turn still reaches something, each one run
//! against the real loaded programme so that the gap is a failing expectation
//! somebody can watch rather than a paragraph in a report.
//!
//! Each is asserted in the direction it behaves **today**. The day one of them
//! is closed, its assertion fails and says where to come and what to change,
//! which is what the rename and delete gaps did before they were closed.
//!
//! # The three, and why they are different in kind
//!
//! - **A socket that was already open.** `socket_connect` decides when a
//!   connection is *made*. A socket connected before the turn began is invisible
//!   to it, and stays writable inside. This is the network's version of the
//!   already-open-descriptor gap, and it has the same answer: a hook that
//!   decides at the moment of opening says nothing afterwards.
//! - **A datagram sent without connecting.** `sendto` on an unconnected socket
//!   reaches no `connect` hook at all. Closing it needs `socket_sendmsg`, which
//!   fires on every message rather than every connection.
//! - **A proxy on loopback.** Loopback is deliberately unchecked (ADR 0007) so
//!   that a model on this machine works. A proxy listening there and forwarding
//!   elsewhere is therefore a bound turn's way out, and this is the one of the
//!   three that a person could set up today without writing any code.
//!
//! `docs/autonomy/updates/publication-hardening-and-egress-coverage.md` is where
//! each is placed against a release and against what production can reach.
//!
//! # Nothing here reaches a network
//!
//! Every address is a listener **this test owns**, on a port the operating
//! system chose, on either loopback or this machine's own interface. The
//! "elsewhere" the proxy forwards to is this same machine on `eth0` — which is
//! not loopback, and is therefore something the boundary really rules on, and is
//! still inside this virtual machine. Nothing resolves a name, nothing leaves,
//! and no host-wide networking is touched.
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

use alo_bounding::{Cgroup, places_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Where the child is told its turn's control group is.
const THE_CGROUP: &str = "ALO_STILL_CGROUP";

/// A destination the child must be refused, whatever else it manages.
const THE_CONTROL: &str = "ALO_STILL_CONTROL";

/// Which of the three the child is being asked to do.
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

/// Long enough for a machine under load, short enough that a hang is a failure.
const NOT_FOREVER: Duration = Duration::from_secs(10);

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

/// A listener of this test's own that reads one message and hands it back.
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

/// One turn: a cgroup, a grant, no destination shown, and a child inside it.
///
/// What comes back is the control's outcome and the subject's, in that order.
/// The grant is a folder of this test's own so that the bound is a real one —
/// `places_of` refuses a path that is not there rather than writing zeroes.
fn a_turn_shown_no_destination(named: &str, subject: &str, address: &str) -> (Outcome, Outcome) {
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

    // Bound to one folder and shown **nothing** to reach. Every destination is
    // refused from here, which is what makes the control meaningful.
    kernel
        .boundary
        .bound(
            turn,
            places_of(&[granted.as_path()]).expect("the granted folder is there"),
        )
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

/// **A connection made before the turn began stays usable inside it.**
///
/// The child opens the socket, *then* joins the control group, and the boundary
/// is written after that. `socket_connect` never saw the connection, and there
/// is no hook that sees the writes — so a turn refused every destination
/// carries a conversation on one it inherited.
///
/// Not reachable through any verb alo OS ships: a turn's verbs are file verbs,
/// and none of them hands a model a descriptor. It is the floor under a verb
/// with a bug in it that has the hole, which is why this is v0.5 work beside the
/// already-open-descriptor gap rather than a v0.01 one.
#[test]
fn a_connection_made_before_the_boundary_stays_usable_inside_it() {
    let (server, listening) = a_server_at(our_own_address());
    let (control, subject) =
        a_turn_shown_no_destination("alo-still-early", "early", &server.to_string());

    assert_eq!(
        control,
        Outcome::Refused(13),
        "the boundary was not in force, so what the inherited socket did means nothing"
    );
    assert_eq!(
        subject,
        Outcome::Allowed,
        "an inherited connection was refused inside a turn, which means something now watches \
         a socket after it is opened — say so in crates/alo-bounding/src/lib.rs and in the \
         coverage table of the egress report, and turn this into the refusal it should be"
    );
    assert_eq!(
        listening.join().expect("the server thread finishes"),
        Some(THE_BYTES.to_vec()),
        "the message did not arrive, so this proves nothing either way"
    );
}

/// **A datagram sent without connecting leaves a bound turn unchecked.**
///
/// `sendto` on an unconnected socket makes no `connect`, so the one hook that
/// decides about destinations never runs. Closing it needs `socket_sendmsg` —
/// a hook on every message rather than every connection — which is a different
/// decision about cost and about what a security module sees.
///
/// Nothing alo OS ships sends one from inside a turn: a question is TCP, and
/// resolution happens outside the boundary by ADR 0020. So this is v0.5 too,
/// and it is named rather than implied.
#[test]
fn an_unconnected_datagram_leaves_a_bound_turn_unchecked() {
    let listening = UdpSocket::bind(SocketAddr::new(our_own_address(), 0))
        .expect("a datagram socket of our own");
    listening
        .set_read_timeout(Some(NOT_FOREVER))
        .expect("a socket can be made to give up");
    let at = listening.local_addr().expect("a socket knows where it is");

    let (control, subject) =
        a_turn_shown_no_destination("alo-still-datagram", "datagram", &at.to_string());

    assert_eq!(
        control,
        Outcome::Refused(13),
        "the boundary was not in force, so what the datagram did means nothing"
    );
    assert_eq!(
        subject,
        Outcome::Allowed,
        "an unconnected datagram was refused, which means a message-level hook has landed — \
         say so in crates/alo-bounding/src/deciding.rs and in the egress report's coverage \
         table, and turn this into the refusal it should be"
    );

    let mut arrived = vec![0u8; THE_BYTES.len()];
    let (read, from) = listening
        .recv_from(&mut arrived)
        .expect("the datagram arrived");
    assert_eq!(
        read,
        THE_BYTES.len(),
        "a stray datagram arrived from {from}"
    );
    assert_eq!(
        arrived, THE_BYTES,
        "what arrived was not what the turn sent"
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
/// cover.
#[test]
fn a_proxy_on_loopback_carries_a_bound_turn_somewhere_nobody_showed_it() {
    let (elsewhere, listening) = a_server_at(our_own_address());
    let (proxy, relaying) = a_proxy_on_loopback_to(elsewhere);

    let (control, subject) =
        a_turn_shown_no_destination("alo-still-proxy", "proxy", &proxy.to_string());

    assert_eq!(
        control,
        Outcome::Refused(13),
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
    let reaching: SocketAddr = address.parse().expect("the parent named an address");

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
        // One datagram, to an address nobody showed this turn, on a socket that
        // never connects to anything.
        "datagram" => match UdpSocket::bind("0.0.0.0:0")
            .and_then(|sending| sending.send_to(THE_BYTES, reaching))
        {
            Ok(_) => "allowed".to_owned(),
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
