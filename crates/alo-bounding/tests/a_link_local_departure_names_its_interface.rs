//! **A link-local departure is one address on one interface**, decided by the
//! kernel on a running machine with the real programme loaded.
//!
//! [ADR 0041](../../../docs/decisions/0041-a-link-local-departure-names-its-interface.md).
//! Since task 23 of the local-network plan a paired machine on a network with no
//! IPv4 address is dialled at `fe80::…%3`, and `fe80::…` is not one machine: it
//! is one machine on each link this machine is on. So a turn shown the studio on
//! the cable must reach it there and **must not** reach the same address on any
//! other interface — `the_kernel_refuses_a_departure.rs` holds the address and
//! the port, and this holds the interface.
//!
//! # Where it runs, and why the refused half is a real witness
//!
//! The turn is a process of this binary run again inside a **network namespace
//! of its own**, made by util-linux's `unshare` as root: a namespace, because
//! the first namespace of the WSL2 kernel this was written on has IPv6 switched
//! off and a fresh one has it on; as root and in no user namespace, because the
//! process has to put itself into a control group of the first namespace's
//! cgroup filesystem. Nothing outside the namespace is touched, and the
//! namespace dies with the process.
//!
//! Inside it are **two interfaces carrying the same link-local address**, each
//! a `dummy` holding `fe80::a1` with duplicate detection off, and one listener on
//! every address. So every attempt below *would* succeed without the boundary —
//! a connection to the address on either interface is accepted by this process
//! — and an attempt that fails with `EACCES` failed because the programme
//! refused it. A refusal cannot be mistaken for a network that was not there.
//!
//! # What is tried
//!
//! Every road the kernel takes an interface from, for both network hooks:
//!
//! | Attempt | What names the interface | Hook |
//! |---|---|---|
//! | `tcp` | the scope in the address `connect` names, then the socket it set | `socket_connect`, then `socket_sendmsg` on the peer |
//! | `udp` | the scope in the address `sendto` names | `socket_sendmsg` |
//! | `held` | nothing in the address; the socket was bound to a scoped address first | `socket_sendmsg`, by the socket |
//! | `joined` | a socket joined **before** the turn began, written on inside it | `socket_sendmsg` on the peer |
//! | `nowhere` | nothing at all | `socket_sendmsg`, and refused as naming no network |

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    net::{Ipv6Addr, SocketAddr, SocketAddrV6, TcpListener, TcpStream, UdpSocket},
    path::Path,
    process::{ChildStdout, Command, Stdio},
    time::Duration,
};

use alo_bounding::{Cgroup, Departure, Departures, Family, places_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Where the child is told its turn's control group is.
const THE_CGROUP: &str = "ALO_LINK_LOCAL_TEST_CGROUP";

/// The attempts the child makes, in order, separated by commas.
const THE_ATTEMPTS: &str = "ALO_LINK_LOCAL_TEST_ATTEMPTS";

/// What the child says with the two interfaces' numbers after it.
const INTERFACES: &str = "alo:interfaces ";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says about one attempt, followed by the outcome.
const WENT: &str = "alo:went ";

/// What the child says when it has tried them all.
const DONE: &str = "alo:done";

/// `EACCES`, which is the only thing a refusal by this boundary looks like.
const REFUSED: i32 = 13;

/// The address both interfaces carry.
const THE_ADDRESS: Ipv6Addr = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0xa1);

/// The port the child listens on, which is its own namespace's.
const THE_PORT: u16 = 7_610;

/// The interface a departure is shown on.
const THE_CABLE: &str = "cable0";

/// The other interface, carrying the same address.
const ELSEWHERE: &str = "elsewhere0";

/// What the kernel made of one attempt.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The boundary refused it before anything was sent.
    Refused,

    /// It arrived: the connection was made and written on, or the datagram
    /// was sent.
    Reached,

    /// Anything else, which is neither and is always a failure here: every
    /// attempt in this file either reaches this process or is refused.
    Otherwise(String),
}

impl Outcome {
    /// What the child wrote, read back.
    fn from(text: &str) -> Self {
        match (text, text.parse::<i32>()) {
            ("reached", _) => Self::Reached,
            (_, Ok(REFUSED)) => Self::Refused,
            (other, _) => Self::Otherwise(other.to_owned()),
        }
    }
}

/// The two interfaces' numbers, as the kernel inside the namespace gave them.
#[derive(Debug, Clone, Copy)]
struct Interfaces {
    /// [`THE_CABLE`]'s.
    cable: u32,
    /// [`ELSEWHERE`]'s.
    elsewhere: u32,
}

/// The studio's address on one interface, as a departure.
fn on(interface: u32) -> Departure {
    Departure::on(Family::Six, THE_ADDRESS.to_bits(), THE_PORT, interface)
}

/// A granted folder, so that a turn has a bound at all.
fn a_folder_to_grant(what: &str) -> std::path::PathBuf {
    let root = std::path::PathBuf::from("/tmp")
        .join(format!("alo-link-local-{}-{what}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Invoices")).expect("a temporary directory can be made");
    root
}

/// One turn inside a network of its own: a cgroup, a bound with the
/// destinations `shown` makes of the child's interfaces, and a child that makes
/// each attempt in turn.
fn a_turn_on_two_links(
    named: &str,
    shown: impl FnOnce(Interfaces) -> Departures,
    attempts: &[&str],
) -> Vec<Outcome> {
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel(named);
    let boundary = &mut kernel.boundary;
    // Named for this run as well as the test, so a run that was killed part of
    // the way through leaves nothing the next one trips over.
    let cgroup = Cgroup::made(&format!("{named}-{}", std::process::id()))
        .expect("a control group can be made");
    let turn = cgroup.id().expect("a control group has an identifier");
    let folder = a_folder_to_grant(named);

    let script = format!(
        "ip link set lo up \
         && ip link add {THE_CABLE} type dummy && ip link set {THE_CABLE} up \
         && ip link add {ELSEWHERE} type dummy && ip link set {ELSEWHERE} up \
         && ip -6 addr add {THE_ADDRESS}/64 dev {THE_CABLE} nodad \
         && ip -6 addr add {THE_ADDRESS}/64 dev {ELSEWHERE} nodad \
         && exec \"$0\" --exact --ignored --nocapture the_work_a_turn_does_on_two_links"
    );
    let mut child = Command::new("unshare")
        .args(["--net", "--", "sh", "-c", &script])
        .arg(env::current_exe().expect("a test binary knows where it is"))
        .env(THE_CGROUP, cgroup.at())
        .env(THE_ATTEMPTS, attempts.join(","))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("util-linux's `unshare` makes the network, and this machine does not have it");

    let mut saying = BufReader::new(child.stdout.take().expect("stdout was asked for"));
    let mut telling = child.stdin.take().expect("stdin was asked for");

    let numbers = next_line_from(&mut saying);
    let interfaces = numbers
        .trim()
        .strip_prefix(INTERFACES)
        .and_then(|numbers| numbers.split_once(' '))
        .and_then(|(cable, elsewhere)| {
            Some(Interfaces {
                cable: cable.parse().ok()?,
                elsewhere: elsewhere.parse().ok()?,
            })
        })
        .unwrap_or_else(|| panic!("the child never said its interfaces: {numbers}"));
    let ready = next_line_from(&mut saying);
    assert_eq!(ready.trim(), READY, "the child never reached its cgroup");

    let places = places_of(&[&folder.join("Invoices")]).expect("the granted folder is there");
    boundary
        .bound(turn, places.and_shown(shown(interfaces)))
        .expect("the kernel takes the entry");

    telling.write_all(b"go\n").expect("the child is listening");
    telling.flush().expect("the child is listening");

    let mut went = Vec::new();
    loop {
        let said = next_line_from(&mut saying);
        let said = said.trim();
        if said == DONE {
            break;
        }
        let what = said
            .strip_prefix(WENT)
            .expect("every line is an outcome or the end");
        went.push(Outcome::from(what));
    }

    let finished = child.wait().expect("the child finishes");
    assert!(finished.success(), "the child failed: {finished}");
    boundary.released(turn).expect("the kernel takes it back");
    cgroup
        .removed()
        .expect("an empty control group can be taken away");
    let _ = fs::remove_dir_all(&folder);
    went
}

/// The next line the child says that is meant for us.
fn next_line_from(saying: &mut BufReader<ChildStdout>) -> String {
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

/// Every attempt the child can make, each once, on the interface named.
fn on_both(interface: &str) -> [String; 4] {
    [
        format!("tcp {interface}"),
        format!("udp {interface}"),
        format!("held {interface}"),
        format!("joined {interface}"),
    ]
}

/// **Shown the studio on the cable, a turn reaches it there by every road —
/// and is refused the same address on the other interface by every road.**
///
/// The acceptance's two ways, in one turn: the registered interface, and the
/// same address on an interface nobody showed. If a departure matched the
/// address alone, the second half would reach this process's own listener.
#[test]
fn a_link_local_departure_reaches_its_interface_and_not_the_same_address_on_another() {
    let cable = on_both(THE_CABLE);
    let elsewhere = on_both(ELSEWHERE);
    let attempts: Vec<&str> = cable.iter().chain(&elsewhere).map(String::as_str).collect();
    let went = a_turn_on_two_links(
        "alo-link-local-cable",
        |interfaces| Departures::of(&[on(interfaces.cable)]).expect("one is not too many"),
        &attempts,
    );
    assert_eq!(
        went,
        [
            Outcome::Reached,
            Outcome::Reached,
            Outcome::Reached,
            Outcome::Reached,
            Outcome::Refused,
            Outcome::Refused,
            Outcome::Refused,
            Outcome::Refused,
        ],
        "tried, in order: {attempts:?}"
    );
}

/// **And the other way round**: shown the address on the other interface, the
/// cable is the one refused. The interface is what decides, not which one
/// happens to be first.
#[test]
fn shown_on_the_other_interface_the_cable_is_the_one_refused() {
    let cable = on_both(THE_CABLE);
    let elsewhere = on_both(ELSEWHERE);
    let attempts: Vec<&str> = cable.iter().chain(&elsewhere).map(String::as_str).collect();
    let went = a_turn_on_two_links(
        "alo-link-local-elsewhere",
        |interfaces| Departures::of(&[on(interfaces.elsewhere)]).expect("one is not too many"),
        &attempts,
    );
    assert_eq!(
        went,
        [
            Outcome::Refused,
            Outcome::Refused,
            Outcome::Refused,
            Outcome::Refused,
            Outcome::Reached,
            Outcome::Reached,
            Outcome::Reached,
            Outcome::Reached,
        ],
        "tried, in order: {attempts:?}"
    );
}

/// **A link-local address with no interface is refused, even to a turn whose
/// entry holds that very nothing** — and a datagram naming no interface is
/// refused to a turn shown the address on the cable, because the kernel would
/// choose its interface from a socket option no departure describes.
#[test]
fn a_link_local_address_with_no_interface_is_refused_however_it_was_shown() {
    let unscoped = Departure::of(Family::Six, THE_ADDRESS.to_bits(), THE_PORT);
    let went = a_turn_on_two_links(
        "alo-link-local-unscoped",
        |_| Departures::of(&[unscoped]).expect("one is not too many"),
        &["tcp cable0", "udp cable0", "nowhere"],
    );
    assert_eq!(went, [Outcome::Refused, Outcome::Refused, Outcome::Refused]);

    let went = a_turn_on_two_links(
        "alo-link-local-nowhere",
        |interfaces| Departures::of(&[on(interfaces.cable)]).expect("one is not too many"),
        &["nowhere", "tcp cable0"],
    );
    assert_eq!(
        went,
        [Outcome::Refused, Outcome::Reached],
        "a datagram naming no interface reached past a departure shown on one"
    );
}

/// **Default deny holds on a link**: a turn shown nothing reaches neither
/// interface, by a connection made inside the turn or one joined before it.
#[test]
fn a_turn_shown_nothing_reaches_no_link_local_destination() {
    let went = a_turn_on_two_links(
        "alo-link-local-none",
        |_| Departures::none(),
        &["tcp cable0", "joined cable0", "udp elsewhere0"],
    );
    assert_eq!(went, [Outcome::Refused, Outcome::Refused, Outcome::Refused]);
}

/// The kernel's number for the interface `name`, inside this namespace.
///
/// Read from `/proc/net/if_inet6`, which answers for the namespace of the
/// process reading it — `/sys/class/net` answers for the namespace `sysfs` was
/// mounted in, which is the first one, and has never heard of these interfaces.
fn index_of(name: &str) -> u32 {
    let table = fs::read_to_string("/proc/net/if_inet6").expect("this kernel has IPv6 here");
    table
        .lines()
        .find_map(|line| {
            let columns: Vec<&str> = line.split_whitespace().collect();
            match columns.as_slice() {
                [_, index, _, _, _, interface] if *interface == name => {
                    u32::from_str_radix(index, 16).ok()
                }
                _ => None,
            }
        })
        .unwrap_or_else(|| panic!("the script made no address on {name}: {table}"))
}

/// The address on the interface the kernel numbers `scope`, or on none for zero.
fn the_address_on(scope: u32) -> SocketAddr {
    SocketAddr::V6(SocketAddrV6::new(THE_ADDRESS, THE_PORT, 0, scope))
}

/// What one attempt came to, as the child writes it.
fn written(outcome: std::io::Result<()>) -> String {
    match outcome {
        Ok(()) => "reached".to_owned(),
        Err(why) => why
            .raw_os_error()
            .map_or_else(|| why.to_string(), |errno| errno.to_string()),
    }
}

/// A connection to `to`, and a byte written on it.
fn connected_and_written(to: SocketAddr) -> std::io::Result<()> {
    let mut stream = TcpStream::connect_timeout(&to, Duration::from_secs(2))?;
    stream.write_all(b"x")
}

/// A datagram to `to`, from a socket bound to nothing in particular.
fn sent(to: SocketAddr) -> std::io::Result<()> {
    let socket = UdpSocket::bind("[::]:0")?;
    socket.send_to(b"x", to).map(|_| ())
}

/// A datagram to the address with no scope, from a socket bound to the address
/// on the interface `scope` — so the interface comes from the socket and not the
/// address.
fn sent_from_a_socket_held_to(scope: u32) -> std::io::Result<()> {
    let socket = UdpSocket::bind(SocketAddr::V6(SocketAddrV6::new(THE_ADDRESS, 0, 0, scope)))?;
    socket.send_to(b"x", the_address_on(0)).map(|_| ())
}

/// The work of a turn, run inside its own network by the tests above.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name inside the namespace it made. Run without the environment that names a
/// cgroup it does nothing.
#[test]
#[ignore = "this is the child half of the tests above, and the parent runs it by name"]
fn the_work_a_turn_does_on_two_links() {
    let (Ok(cgroup), Ok(attempts)) = (env::var(THE_CGROUP), env::var(THE_ATTEMPTS)) else {
        return;
    };
    let attempts: Vec<&str> = attempts.split(',').collect();
    // Read now: once this process is a turn, `/sys` is outside its grant.
    let (cable, elsewhere) = (index_of(THE_CABLE), index_of(ELSEWHERE));
    let scope = |interface: &str| {
        if interface == THE_CABLE {
            cable
        } else {
            elsewhere
        }
    };

    // Everything that has to exist before the turn begins: the listener every
    // attempt would reach, and the connections joined before the turn.
    let listening = TcpListener::bind("[::]:7610").expect("a listener in this namespace");
    let mut joined_before = Vec::new();
    for attempt in &attempts {
        if let Some(interface) = attempt.strip_prefix("joined ") {
            joined_before.push(
                TcpStream::connect_timeout(
                    &the_address_on(scope(interface)),
                    Duration::from_secs(2),
                )
                .expect("outside a turn the address on either interface is reached"),
            );
        }
    }

    let mut saying = std::io::stdout();
    writeln!(saying, "{INTERFACES}{cable} {elsewhere}").expect("the parent is listening");

    fs::write(
        Path::new(&cgroup).join("cgroup.procs"),
        std::process::id().to_string(),
    )
    .expect("a process can put itself in a control group");
    writeln!(saying, "{READY}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    let mut go = String::new();
    std::io::stdin()
        .read_line(&mut go)
        .expect("the parent says when");

    // From here the boundary is in force, and every attempt would otherwise
    // reach `listening` or be sent: what is read is whether it was refused.
    let mut joined = joined_before.into_iter();
    for attempt in &attempts {
        let outcome = match attempt.split_once(' ') {
            Some(("tcp", interface)) => connected_and_written(the_address_on(scope(interface))),
            Some(("udp", interface)) => sent(the_address_on(scope(interface))),
            Some(("held", interface)) => sent_from_a_socket_held_to(scope(interface)),
            Some(("joined", _)) => joined.next().map_or_else(
                || Err(std::io::Error::other("nothing joined")),
                |mut stream| stream.write_all(b"x"),
            ),
            _ if *attempt == "nowhere" => sent(the_address_on(0)),
            _ => Err(std::io::Error::other(format!("no such attempt: {attempt}"))),
        };
        writeln!(saying, "{WENT}{}", written(outcome)).expect("the parent is listening");
    }
    writeln!(saying, "{DONE}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");
    drop(listening);

    // Left rather than returned to the harness, which would print a summary and
    // may touch files this process is no longer allowed to open.
    std::process::exit(0);
}
