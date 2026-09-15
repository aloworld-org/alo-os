//! **A private IPv4 departure held to an interface is one address on one
//! network**, decided by the kernel on a running machine with the real programme
//! loaded.
//!
//! [ADR 0042](../../../docs/decisions/0042-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md).
//! `192.168.1.20` on the wired network and `192.168.1.20` on the Wi-Fi are two
//! machines whenever two routers hand out the same range. A paired machine found
//! on one of them is dialled from a socket held to that interface, and the
//! departure the person was shown is held to the same one — so a turn reaches
//! the studio on the cable and **must not** reach the same address on the other
//! network, whether by a socket held there, a socket held to nothing that the
//! route would send there, or a datagram whose `IP_PKTINFO` names the other
//! interface. `a_link_local_departure_names_its_interface.rs` holds the same
//! for an IPv6 link-local address.
//!
//! # Where it runs, and why the refused half is a real witness
//!
//! The turn is this binary run again inside a **network namespace of its own**,
//! made by util-linux's `unshare` as root and in no user namespace, because it
//! puts itself into a control group of the first namespace's. Inside it are two
//! `veth` cables, `cable0` and `elsewhere0`, both carrying `10.64.0.1/24`, and
//! at the far end of each — in two more namespaces nested inside — **a machine
//! at `10.64.0.20`**, which is this binary a fourth and fifth time, listening on
//! the port for connections and datagrams and counting every byte that
//! arrives. So every attempt below *would* arrive somewhere without the
//! boundary, and the far ends say where: an attempt refused with `EACCES` is
//! refused by the programme, and a byte that reached the machine nobody showed
//! is counted by that machine.
//!
//! (A `dummy` interface would not do here as it did for IPv6: an IPv4 address
//! on two interfaces of one namespace is local to the namespace, answered by
//! loopback, and a socket held to the second interface is never answered.)
//!
//! # What is tried
//!
//! | Attempt | How the interface is chosen | Hook |
//! |---|---|---|
//! | `tcp` | a socket held to the interface, connected and written on | `socket_connect`, then `socket_sendmsg` on the peer |
//! | `udp` | a socket held to the interface, a datagram sent to the address | `socket_sendmsg` |
//! | `joined` | a socket held to the interface and joined **before** the turn began, written on inside it | `socket_sendmsg` on the peer |
//! | `route` | a socket held to nothing, which leaves by the route | `socket_connect` |
//! | `pktinfo` | a socket held to one interface, a datagram whose `IP_PKTINFO` names another | `socket_sendmsg`, with a control message |

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    env, fs,
    io::{BufRead, BufReader, IoSlice, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket},
    num::NonZeroU32,
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use alo_bounding::{Cgroup, Departure, Departures, Family, places_of};
use socket2::{Domain, MsgHdr, SockAddr, Socket, Type};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Where the turn is told its control group is.
const THE_CGROUP: &str = "ALO_PRIVATE_IPV4_TEST_CGROUP";

/// The attempts the turn makes, in order, separated by commas.
const THE_ATTEMPTS: &str = "ALO_PRIVATE_IPV4_TEST_ATTEMPTS";

/// Set on a far end, naming which it is.
const FAR_END: &str = "ALO_PRIVATE_IPV4_TEST_FAR_END";

/// What the turn says with the two interfaces' numbers after it.
const INTERFACES: &str = "alo:interfaces ";

/// What the turn says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the turn says about one attempt, followed by the outcome.
const WENT: &str = "alo:went ";

/// What the turn says with how many bytes each far end heard.
const HEARD: &str = "alo:heard ";

/// What a far end says when it is listening, and with its counts.
const LISTENING: &str = "alo:listening";

/// What a far end says with the bytes it has counted.
const COUNTED: &str = "alo:counted ";

/// `EACCES`, which is the only thing a refusal by this boundary looks like.
const REFUSED: i32 = 13;

/// The address both far ends have, which is the studio's on one network and
/// somebody else's on the other.
const THE_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 64, 0, 20);

/// The address the turn's end of both cables has.
const THIS_END: &str = "10.64.0.1/24";

/// The port both far ends listen on.
const THE_PORT: u16 = 7_610;

/// The interface a departure is shown on.
const THE_CABLE: &str = "cable0";

/// The other network, carrying the same address.
const ELSEWHERE: &str = "elsewhere0";

/// `IPPROTO_IP` and `IP_PKTINFO`, as Linux numbers them.
const IP_PKTINFO: (i32, i32) = (0, 8);

/// How long anything here waits for a network to settle.
const PATIENCE: Duration = Duration::from_secs(20);

/// What the kernel made of one attempt.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The boundary refused it before anything was sent.
    Refused,

    /// It was made: the connection was made and written on, or the datagram
    /// was sent.
    Reached,

    /// Anything else, which is neither and is always a failure here.
    Otherwise(String),
}

impl Outcome {
    /// What the turn wrote, read back.
    fn from(text: &str) -> Self {
        match (text, text.parse::<i32>()) {
            ("reached", _) => Self::Reached,
            (_, Ok(REFUSED)) => Self::Refused,
            (other, _) => Self::Otherwise(other.to_owned()),
        }
    }
}

use Outcome::{Reached, Refused};

/// The two interfaces' numbers, as the kernel inside the namespace gave them.
#[derive(Debug, Clone, Copy)]
struct Interfaces {
    /// [`THE_CABLE`]'s.
    cable: u32,
    /// [`ELSEWHERE`]'s.
    elsewhere: u32,
}

/// What reached each far end, in bytes of connections and datagrams together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Heard {
    /// At the far end of [`THE_CABLE`].
    cable: usize,
    /// At the far end of [`ELSEWHERE`].
    elsewhere: usize,
}

/// The address on the network behind `interface`, as a departure held to it.
fn held_to(interface: u32) -> Departure {
    Departure::on(
        Family::Four,
        u128::from(THE_ADDRESS.to_bits()),
        THE_PORT,
        interface,
    )
}

/// A granted folder, so that a turn has a bound at all.
fn a_folder_to_grant(what: &str) -> std::path::PathBuf {
    let root = std::path::PathBuf::from("/tmp")
        .join(format!("alo-private-ipv4-{}-{what}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Invoices")).expect("a temporary directory can be made");
    root
}

/// One turn inside a network of its own, with two networks carrying the same
/// address: a cgroup, a bound with the destinations `shown` makes of the turn's
/// interfaces, and each attempt in turn — then what each far end heard.
fn a_turn_on_two_networks(
    named: &str,
    shown: impl FnOnce(Interfaces) -> Departures,
    attempts: &[&str],
) -> (Vec<Outcome>, Heard) {
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel(named);
    let boundary = &mut kernel.boundary;
    let cgroup = Cgroup::made(&format!("{named}-{}", std::process::id()))
        .expect("a control group can be made");
    let turn = cgroup.id().expect("a control group has an identifier");
    let folder = a_folder_to_grant(named);

    let script = "ip link set lo up \
         && exec \"$0\" --exact --ignored --nocapture the_work_a_turn_does_on_two_networks";
    let mut child = Command::new("unshare")
        .args(["--net", "--", "sh", "-c", script])
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
        .unwrap_or_else(|| panic!("the turn never said its interfaces: {numbers}"));
    let ready = next_line_from(&mut saying);
    assert_eq!(ready.trim(), READY, "the turn never reached its cgroup");

    let places = places_of(&[&folder.join("Invoices")]).expect("the granted folder is there");
    boundary
        .bound(turn, places.and_shown(shown(interfaces)))
        .expect("the kernel takes the entry");

    telling.write_all(b"go\n").expect("the turn is listening");
    telling.flush().expect("the turn is listening");

    let mut went = Vec::new();
    let heard = loop {
        let said = next_line_from(&mut saying);
        let said = said.trim();
        if let Some(counts) = said.strip_prefix(HEARD) {
            break counts
                .split_once(' ')
                .and_then(|(cable, elsewhere)| {
                    Some(Heard {
                        cable: cable.parse().ok()?,
                        elsewhere: elsewhere.parse().ok()?,
                    })
                })
                .unwrap_or_else(|| panic!("the turn said what was heard unreadably: {said}"));
        }
        let what = said
            .strip_prefix(WENT)
            .expect("every line is an outcome or what was heard");
        went.push(Outcome::from(what));
    };

    let finished = child.wait().expect("the turn finishes");
    assert!(finished.success(), "the turn failed: {finished}");
    boundary.released(turn).expect("the kernel takes it back");
    cgroup
        .removed()
        .expect("an empty control group can be taken away");
    let _ = fs::remove_dir_all(&folder);
    (went, heard)
}

/// The next line a child says that is meant for us.
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

/// **Shown the studio on the cable, a turn reaches it there by every road — and
/// is refused the same address on the other network by every road**, and by a
/// socket held to nothing and a datagram whose control message names the other
/// interface. The machine at the far end of the other cable hears nothing.
#[test]
fn a_private_ipv4_departure_reaches_its_network_and_not_the_same_address_on_another() {
    let attempts = [
        "tcp cable0",
        "udp cable0",
        "joined cable0",
        "tcp elsewhere0",
        "udp elsewhere0",
        "joined elsewhere0",
        "route",
        "pktinfo cable0 elsewhere0",
        "pktinfo cable0 cable0",
    ];
    let (went, heard) = a_turn_on_two_networks(
        "alo-private-ipv4-cable",
        |interfaces| Departures::of(&[held_to(interfaces.cable)]).expect("one is not too many"),
        &attempts,
    );
    assert_eq!(
        went,
        [
            Reached, Reached, Reached, Refused, Refused, Refused, Refused, Refused, Refused
        ],
        "tried, in order: {attempts:?}"
    );
    assert_eq!(
        heard,
        Heard {
            cable: 3,
            elsewhere: 0
        },
        "a byte reached a network nobody was shown, or the studio missed one"
    );
}

/// **And the other way round**: shown the address on the other network, the
/// cable is the one refused. The interface is what decides, not which network
/// the route would have chosen.
#[test]
fn shown_on_the_other_network_the_cable_is_the_one_refused() {
    let attempts = [
        "tcp cable0",
        "udp cable0",
        "joined cable0",
        "tcp elsewhere0",
        "udp elsewhere0",
        "joined elsewhere0",
        "pktinfo elsewhere0 cable0",
    ];
    let (went, heard) = a_turn_on_two_networks(
        "alo-private-ipv4-elsewhere",
        |interfaces| Departures::of(&[held_to(interfaces.elsewhere)]).expect("one is not too many"),
        &attempts,
    );
    assert_eq!(
        went,
        [
            Refused, Refused, Refused, Reached, Reached, Reached, Refused
        ],
        "tried, in order: {attempts:?}"
    );
    assert_eq!(
        heard,
        Heard {
            cable: 0,
            elsewhere: 3
        }
    );
}

/// **A provider's departure is unchanged**: held to no interface, the address is
/// reached however the socket is held — by the route, on either network, and by
/// a datagram carrying a control message — exactly as every IPv4 departure was
/// before ADR 0042.
#[test]
fn a_departure_held_to_no_interface_is_reached_as_it_always_was() {
    let attempts = [
        "route",
        "tcp cable0",
        "udp elsewhere0",
        "pktinfo cable0 elsewhere0",
    ];
    let (went, heard) = a_turn_on_two_networks(
        "alo-private-ipv4-provider",
        |_| Departures::of(&[held_to(0)]).expect("one is not too many"),
        &attempts,
    );
    assert_eq!(
        went,
        [Reached, Reached, Reached, Reached],
        "tried, in order: {attempts:?}"
    );
    // Where the route sent the first is the route's business; the rest are
    // counted where they were held, and the control message was followed.
    assert_eq!(heard.cable + heard.elsewhere, 4, "{heard:?}");
    assert!(heard.cable >= 1 && heard.elsewhere >= 2, "{heard:?}");
}

/// **Default deny holds on both networks**: a turn shown nothing reaches
/// neither, by a connection made inside the turn, one joined before it, the
/// route, or a control message.
#[test]
fn a_turn_shown_nothing_reaches_neither_network() {
    let attempts = [
        "tcp cable0",
        "joined elsewhere0",
        "route",
        "pktinfo cable0 cable0",
    ];
    let (went, heard) =
        a_turn_on_two_networks("alo-private-ipv4-none", |_| Departures::none(), &attempts);
    assert_eq!(went, [Refused, Refused, Refused, Refused]);
    assert_eq!(
        heard,
        Heard {
            cable: 0,
            elsewhere: 0
        }
    );
}

/// Run `ip` with `args`, inside the network namespace of `pid` when one is
/// named, and fail with what it said if it refuses.
fn ip(pid: Option<u32>, args: &[&str]) {
    let mut command = match pid {
        Some(pid) => {
            let mut command = Command::new("nsenter");
            command.args(["-t", &pid.to_string(), "-n", "ip"]);
            command
        }
        None => Command::new("ip"),
    };
    let ran = command
        .args(args)
        .output()
        .expect("iproute2's `ip` and util-linux's `nsenter` make the cables");
    assert!(
        ran.status.success(),
        "ip {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&ran.stderr)
    );
}

/// The kernel's number for the interface `name`, inside this namespace, from
/// what `ip` prints first on its line.
fn index_of(name: &str) -> u32 {
    let ran = Command::new("ip")
        .args(["-o", "link", "show", "dev", name])
        .output()
        .expect("iproute2's `ip` reads the interfaces");
    let said = String::from_utf8_lossy(&ran.stdout);
    said.split_once(':')
        .and_then(|(index, _)| index.trim().parse().ok())
        .unwrap_or_else(|| panic!("no interface called {name}: {said}"))
}

/// A far end, started in a network namespace nested inside this one, with the
/// cable `name` laid to it: the far end at [`THE_ADDRESS`], this end at
/// [`THIS_END`].
fn a_far_end_on(name: &str) -> (Child, ChildStdin, BufReader<ChildStdout>) {
    let script =
        "ip link set lo up && exec \"$0\" --exact --ignored --nocapture the_machine_at_the_far_end";
    let mut far = Command::new("unshare")
        .args(["--net", "--", "sh", "-c", script])
        .arg(env::current_exe().expect("a test binary knows where it is"))
        .env(FAR_END, name)
        .env_remove(THE_CGROUP)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("util-linux's `unshare` makes the far end");
    let ours = fs::read_link("/proc/self/ns/net").expect("this process has a network");
    let theirs = format!("/proc/{}/ns/net", far.id());
    let until = Instant::now() + PATIENCE;
    while fs::read_link(&theirs).ok().as_ref() == Some(&ours) || !Path::new(&theirs).exists() {
        assert!(
            Instant::now() < until,
            "the far end never left this network"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let pid = far.id();
    let pid_text = pid.to_string();
    ip(
        None,
        &[
            "link", "add", name, "type", "veth", "peer", "name", "far0", "netns", &pid_text,
        ],
    );
    ip(Some(pid), &["addr", "add", "10.64.0.20/24", "dev", "far0"]);
    ip(Some(pid), &["link", "set", "far0", "up"]);
    ip(None, &["addr", "add", THIS_END, "dev", name]);
    ip(None, &["link", "set", name, "up"]);
    let telling = far.stdin.take().expect("stdin was asked for");
    let mut hearing = BufReader::new(far.stdout.take().expect("stdout was asked for"));
    assert_eq!(next_line_from(&mut hearing).trim(), LISTENING);
    (far, telling, hearing)
}

/// How many bytes a far end has heard, once it has had a moment to hear them.
fn counted_by(telling: &mut ChildStdin, hearing: &mut BufReader<ChildStdout>) -> usize {
    telling
        .write_all(b"count\n")
        .expect("the far end is listening");
    telling.flush().expect("the far end is listening");
    let said = next_line_from(hearing);
    said.trim()
        .strip_prefix(COUNTED)
        .and_then(|count| count.parse().ok())
        .unwrap_or_else(|| panic!("the far end said something else: {said}"))
}

/// A socket of `kind` held to the interface the kernel numbers `interface`.
fn a_socket_held_to(kind: Type, interface: u32) -> std::io::Result<Socket> {
    let socket = Socket::new(Domain::IPV4, kind, None)?;
    socket.bind_device_by_index_v4(NonZeroU32::new(interface))?;
    Ok(socket)
}

/// Where both far ends listen.
fn the_far_end() -> SockAddr {
    SockAddr::from(SocketAddr::new(THE_ADDRESS.into(), THE_PORT))
}

/// A connection from a socket held to `interface`, and a byte written on it.
fn connected_and_written(interface: u32) -> std::io::Result<()> {
    let socket = a_socket_held_to(Type::STREAM, interface)?;
    socket.connect_timeout(&the_far_end(), Duration::from_secs(2))?;
    TcpStream::from(socket).write_all(b"x")
}

/// A datagram from a socket held to `interface`.
fn sent_from(interface: u32) -> std::io::Result<()> {
    a_socket_held_to(Type::DGRAM, interface)?
        .send_to(b"x", &the_far_end())
        .map(drop)
}

/// A datagram from a socket held to `held`, carrying an `IP_PKTINFO` that names
/// `named` — which on IPv4 the kernel follows ahead of the socket.
fn sent_with_a_control_message(held: u32, named: u32) -> std::io::Result<()> {
    let socket = a_socket_held_to(Type::DGRAM, held)?;
    // A `cmsghdr` — its length as a `size_t`, then level and type as `int`s —
    // and an `in_pktinfo`: the interface, and two addresses left as nothing.
    let (level, kind) = IP_PKTINFO;
    let mut control = Vec::with_capacity(32);
    control.extend_from_slice(&28_u64.to_ne_bytes());
    control.extend_from_slice(&level.to_ne_bytes());
    control.extend_from_slice(&kind.to_ne_bytes());
    control.extend_from_slice(&named.to_ne_bytes());
    control.extend_from_slice(&[0; 8]);
    control.extend_from_slice(&[0; 4]);
    let bytes = [IoSlice::new(b"x")];
    let to = the_far_end();
    let message = MsgHdr::new()
        .with_addr(&to)
        .with_buffers(&bytes)
        .with_control(&control);
    socket.sendmsg(&message, 0).map(drop)
}

/// A connection by the route, from a socket held to nothing.
fn connected_by_the_route() -> std::io::Result<()> {
    let mut stream = TcpStream::connect_timeout(
        &SocketAddr::new(THE_ADDRESS.into(), THE_PORT),
        Duration::from_secs(2),
    )?;
    stream.write_all(b"x")
}

/// What one attempt came to, as the turn writes it.
fn written(outcome: std::io::Result<()>) -> String {
    match outcome {
        Ok(()) => "reached".to_owned(),
        Err(why) => why
            .raw_os_error()
            .map_or_else(|| why.to_string(), |errno| errno.to_string()),
    }
}

/// The work of a turn, run inside its own network by the tests above.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name inside the namespace it made. Run without the environment that names a
/// cgroup it does nothing.
#[test]
#[ignore = "this is the turn half of the tests above, and the parent runs it by name"]
fn the_work_a_turn_does_on_two_networks() {
    let (Ok(cgroup), Ok(attempts)) = (env::var(THE_CGROUP), env::var(THE_ATTEMPTS)) else {
        return;
    };
    let attempts: Vec<&str> = attempts.split(',').collect();

    // The two networks, each with a machine at the same address at the far end.
    let (mut cable_end, mut telling_cable, mut hearing_cable) = a_far_end_on(THE_CABLE);
    let (mut elsewhere_end, mut telling_elsewhere, mut hearing_elsewhere) = a_far_end_on(ELSEWHERE);
    let (cable, elsewhere) = (index_of(THE_CABLE), index_of(ELSEWHERE));
    let scope = |interface: &str| {
        if interface == THE_CABLE {
            cable
        } else {
            elsewhere
        }
    };

    // Outside the turn, each network carries a connection to its far end — which
    // is also what the joined attempts write on inside it. A cable that is still
    // coming up is waited for rather than failed on.
    let joined_on = |interface: u32| {
        let until = Instant::now() + PATIENCE;
        loop {
            let tried = a_socket_held_to(Type::STREAM, interface).and_then(|socket| {
                socket.connect_timeout(&the_far_end(), Duration::from_secs(2))?;
                Ok(TcpStream::from(socket))
            });
            match tried {
                Ok(stream) => return stream,
                Err(why) => assert!(
                    Instant::now() < until,
                    "outside a turn the far end on {interface} was never reached: {why}"
                ),
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    };
    let mut joined_before = Vec::new();
    for interface in [cable, elsewhere] {
        drop(joined_on(interface));
    }
    for attempt in &attempts {
        if let Some(interface) = attempt.strip_prefix("joined ") {
            joined_before.push(joined_on(scope(interface)));
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
    // arrive at one far end or the other: what is read is whether it was refused.
    let mut joined = joined_before.into_iter();
    for attempt in &attempts {
        let words: Vec<&str> = attempt.split(' ').collect();
        let outcome = match words.as_slice() {
            ["tcp", interface] => connected_and_written(scope(interface)),
            ["udp", interface] => sent_from(scope(interface)),
            ["joined", _] => joined.next().map_or_else(
                || Err(std::io::Error::other("nothing joined")),
                |mut stream| stream.write_all(b"x"),
            ),
            ["route"] => connected_by_the_route(),
            ["pktinfo", held, named] => sent_with_a_control_message(scope(held), scope(named)),
            _ => Err(std::io::Error::other(format!("no such attempt: {attempt}"))),
        };
        writeln!(saying, "{WENT}{}", written(outcome)).expect("the parent is listening");
    }

    // The far ends are outside the turn's grant only in their pipes, which
    // were open before it began — so they are asked, after a moment to hear.
    std::thread::sleep(Duration::from_millis(500));
    let heard_on_the_cable = counted_by(&mut telling_cable, &mut hearing_cable);
    let heard_elsewhere = counted_by(&mut telling_elsewhere, &mut hearing_elsewhere);
    writeln!(saying, "{HEARD}{heard_on_the_cable} {heard_elsewhere}")
        .expect("the parent is listening");
    saying.flush().expect("the parent is listening");
    drop((telling_cable, telling_elsewhere));
    drop((cable_end.kill(), elsewhere_end.kill()));
    drop((cable_end.wait(), elsewhere_end.wait()));

    // Left rather than returned to the harness, which would print a summary and
    // may touch files this process is no longer allowed to open.
    std::process::exit(0);
}

/// A machine at [`THE_ADDRESS`] at the far end of one cable, counting every byte
/// of every connection and datagram that reaches it, and saying the count when
/// asked.
#[test]
#[ignore = "this is the far end of the tests above, and the turn runs it by name"]
fn the_machine_at_the_far_end() {
    if env::var(FAR_END).is_err() {
        return;
    }
    let heard = Arc::new(AtomicUsize::new(0));
    let listener = TcpListener::bind(("0.0.0.0", THE_PORT)).expect("a listener at the far end");
    let datagrams =
        UdpSocket::bind(("0.0.0.0", THE_PORT)).expect("a datagram socket at the far end");

    let counting = Arc::clone(&heard);
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let counting = Arc::clone(&counting);
            std::thread::spawn(move || {
                let mut stream = stream;
                let mut buffer = [0_u8; 64];
                while let Ok(read @ 1..) = stream.read(&mut buffer) {
                    counting.fetch_add(read, Ordering::SeqCst);
                }
            });
        }
    });
    let counting = Arc::clone(&heard);
    std::thread::spawn(move || {
        let mut buffer = [0_u8; 64];
        while let Ok((read, _)) = datagrams.recv_from(&mut buffer) {
            counting.fetch_add(read, Ordering::SeqCst);
        }
    });

    let mut saying = std::io::stdout();
    writeln!(saying, "{LISTENING}").expect("the turn is listening");
    saying.flush().expect("the turn is listening");
    for line in std::io::stdin().lines() {
        if line.is_err() {
            break;
        }
        writeln!(saying, "{COUNTED}{}", heard.load(Ordering::SeqCst))
            .expect("the turn is listening");
        saying.flush().expect("the turn is listening");
    }
    std::process::exit(0);
}
