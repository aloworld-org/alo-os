//! A machine on two networks is found on each of them — on a real kernel, with
//! two real networks between two machines.
//!
//! *Machines find each other with zero configuration*, and a machine in an
//! office is often on more than one network at once. The rules are each tested
//! with no kernel in them (`crate::networks`, `crate::route_messages`,
//! `alo_nearby::Around::heard_on_each`); this is the join, the way a machine has
//! it: the studio's `Wire::bound` on two networks, reception's look on each.
//!
//! # The two machines
//!
//! **Reception** is this test binary run again inside a network namespace of its
//! own, made by util-linux's `unshare` as `an_office_that_cannot_connect` makes
//! its office. **The studio** is the binary run a third time, in a namespace
//! nested inside reception's. Between them are two `veth` pairs — the wired
//! network, `10.61.1.0/24`, and the wireless one, `10.61.2.0/24` — made by
//! iproute2, so each machine has two interfaces that are up, carry multicast and
//! have an address, exactly as a docked laptop does.
//!
//! **Nothing here touches kernel-global state.** Both namespaces are private to
//! a user namespace this test made, the interfaces live and die with them, and
//! nothing on the host's own network moves — so this does not take
//! `alo_bounding::Waited::on_this_kernel()`, which is for fixtures that attach
//! BPF programs or change what the other checkout's tests would see.
//!
//! # In this order
//!
//! 1. The studio starts with **one** network — the wired one — and binds its
//!    wire as `src/main.rs` does, hosting a workspace.
//! 2. Reception makes the wireless network and looks: the studio is heard on the
//!    wired network **only**, because it has not yet joined the other — which is
//!    the failure this task exists for, seen before it is fixed.
//! 3. The kernel tells the studio a network appeared; the studio's wire follows
//!    it, as the service does when that socket is ready, and is joined on both.
//! 4. Reception looks again: one machine, with the address it answered from on
//!    each network; one workspace likewise; and the bytes the studio answered
//!    with are the same on both networks.
//! 5. The studio's `advertised` answer is the same bytes it would be with one
//!    network, or none.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeSet;
use std::env;
use std::io::{BufRead as _, BufReader, Write as _};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, UdpSocket};
use std::num::NonZeroU16;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use alo_agentd::{Hosted, THE_WIRE_PORT, Wire, around_at, discovery_networks, found_by_name};
use alo_nearby::{MachineId, Presence, THE_ADDRESS, THE_PORT, WorkspacePresence, advertising};

/// Set on the binary run again inside a namespace, naming which machine it is.
const INSIDE: &str = "ALO_AGENTD_TWO_NETWORKS_INSIDE";

/// The port the studio's workspace answers on.
const WORKSPACE: u16 = 8_443;

/// Reception's address on the wired network, and the studio's.
const RECEPTION_WIRED: Ipv4Addr = Ipv4Addr::new(10, 61, 1, 1);
/// The studio's address on the wired network.
const STUDIO_WIRED: Ipv4Addr = Ipv4Addr::new(10, 61, 1, 2);
/// Reception's address on the wireless network.
const RECEPTION_WIRELESS: Ipv4Addr = Ipv4Addr::new(10, 61, 2, 1);
/// The studio's address on the wireless network.
const STUDIO_WIRELESS: Ipv4Addr = Ipv4Addr::new(10, 61, 2, 2);

/// The studio.
fn the_studio() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// Where a look for who is here is asked on a real network.
fn the_group() -> SocketAddr {
    SocketAddr::new(THE_ADDRESS.into(), THE_PORT)
}

/// **A machine on two networks is found on each of them**: the outer test,
/// which makes reception and reads what it saw.
#[test]
fn a_machine_on_two_networks_is_found_on_each_of_them() {
    let exe = env::current_exe().expect("a test binary knows where it is");
    let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture reception_inside_its_own_network";
    let ran = Command::new("unshare")
        .args(["--map-root-user", "--net", "--", "sh", "-c", script])
        .arg(exe)
        .env(INSIDE, "reception")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .expect("util-linux's `unshare` makes the two machines, and this machine does not have it");
    let said = String::from_utf8_lossy(&ran.stdout).into_owned();
    assert!(
        ran.status.success(),
        "reception failed ({}):\n{said}",
        ran.status
    );
    for step in [
        "heard on one network before joining the second",
        "one machine, an address on each network",
        "one workspace, an address on each network",
        "the same bytes on each network",
        "advertised is the same bytes",
    ] {
        assert!(
            said.contains(step),
            "reception never said `{step}`:\n{said}"
        );
    }
}

/// Refuse to run unless this is the binary re-run as `which`.
fn must_be(which: &str) {
    assert_eq!(
        env::var(INSIDE).ok().as_deref(),
        Some(which),
        "this test runs inside the namespace the outer test makes; run the outer one"
    );
}

/// Run iproute2 with `args`, and fail with what it said if it refuses.
fn ip(args: &[&str]) {
    let ran = Command::new("ip")
        .args(args)
        .output()
        .expect("iproute2's `ip` makes the networks, and this machine does not have it");
    assert!(
        ran.status.success(),
        "ip {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&ran.stderr)
    );
}

/// Run iproute2 with `args` inside the studio's network namespace.
fn ip_in(studio: u32, args: &[&str]) {
    let pid = studio.to_string();
    let ran = Command::new("nsenter")
        .args(["-t", &pid, "-n", "ip"])
        .args(args)
        .output()
        .expect("util-linux's `nsenter` reaches the studio's network, and this machine does not have it");
    assert!(
        ran.status.success(),
        "nsenter ip {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&ran.stderr)
    );
}

/// One network between reception and the studio: a `veth` pair, one end in
/// each namespace, each addressed and up.
fn a_network(studio: u32, name: &str, reception: Ipv4Addr, studios: Ipv4Addr) {
    let here = format!("{name}0");
    let there = format!("{name}1");
    let pid = studio.to_string();
    ip(&[
        "link", "add", &here, "type", "veth", "peer", "name", &there, "netns", &pid,
    ]);
    ip(&["addr", "add", &format!("{reception}/24"), "dev", &here]);
    ip(&["link", "set", &here, "up"]);
    ip_in(
        studio,
        &["addr", "add", &format!("{studios}/24"), "dev", &there],
    );
    ip_in(studio, &["link", "set", &there, "up"]);
}

/// The next line the studio printed under `prefix`, waiting for it.
fn the_studio_says(from: &mut BufReader<ChildStdout>, prefix: &str) -> String {
    let mut line = String::new();
    loop {
        line.clear();
        let read = from.read_line(&mut line).unwrap();
        assert!(read > 0, "the studio ended before saying `{prefix}`");
        if let Some(rest) = line.trim_end().strip_prefix(prefix) {
            return rest.to_owned();
        }
    }
}

/// Wait until the studio's process is in a network namespace that is not this
/// one — until then, a link moved to it would land here.
fn until_it_has_its_own_network(studio: &Child) {
    let ours = std::fs::read_link("/proc/self/ns/net").unwrap();
    let theirs = format!("/proc/{}/ns/net", studio.id());
    let until = Instant::now() + Duration::from_secs(10);
    while std::fs::read_link(&theirs).ok().as_ref() == Some(&ours)
        || !std::path::Path::new(&theirs).exists()
    {
        assert!(Instant::now() < until, "the studio never left this network");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Every datagram that came back within a second to `socket`, with where from.
fn everything_heard(socket: &UdpSocket) -> Vec<(IpAddr, Vec<u8>)> {
    socket
        .set_read_timeout(Some(Duration::from_millis(1_000)))
        .unwrap();
    let mut heard = Vec::new();
    let mut datagram = [0_u8; 1_500];
    while let Ok((read, from)) = socket.recv_from(&mut datagram) {
        heard.push((from.ip(), datagram.get(..read).unwrap().to_vec()));
    }
    heard
}

/// Reception, inside a network of its own: it starts the studio, makes the two
/// networks, and looks.
#[test]
#[ignore = "run inside its own network by a_machine_on_two_networks_is_found_on_each_of_them"]
fn reception_inside_its_own_network() {
    must_be("reception");
    let exe = env::current_exe().unwrap();
    let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture the_studio_inside_its_own_network";
    let mut studio = Command::new("unshare")
        .args(["--net", "--", "sh", "-c", script])
        .arg(exe)
        .env(INSIDE, "studio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("util-linux's `unshare` makes the studio");
    until_it_has_its_own_network(&studio);
    let pid = studio.id();
    let mut hearing = BufReader::new(studio.stdout.take().unwrap());
    let mut telling = studio.stdin.take().unwrap();

    // 1. The studio, on the wired network only.
    a_network(pid, "wired", RECEPTION_WIRED, STUDIO_WIRED);
    telling.write_all(b"wired\n").unwrap();
    the_studio_says(&mut hearing, "bound on ");

    // 2. The wireless network appears; the studio has not joined it yet.
    a_network(pid, "wireless", RECEPTION_WIRELESS, STUDIO_WIRELESS);
    let networks =
        discovery_networks(&alo_agentd::route_messages::reported_by_the_kernel().unwrap());
    assert_eq!(
        networks.len(),
        2,
        "reception is on two networks: {networks:?}"
    );
    let before = found_by_name(&the_studio(), the_group()).expect("the studio is found");
    assert_eq!(
        before.addresses().collect::<Vec<_>>(),
        vec![IpAddr::from(STUDIO_WIRED)],
        "{before:?}"
    );
    println!("heard on one network before joining the second");

    // 3. The studio follows the kernel.
    telling.write_all(b"wireless\n").unwrap();
    let joined = the_studio_says(&mut hearing, "joined on ");
    assert_eq!(joined, "2", "the studio joined on {joined} networks");

    // 4. One machine, one workspace, the same bytes on each network.
    let around = around_at(the_group());
    let both: BTreeSet<IpAddr> = [STUDIO_WIRED.into(), STUDIO_WIRELESS.into()].into();
    assert_eq!(around.machines.len(), 1, "{around:?}");
    let machine = around.machines.first().unwrap();
    assert_eq!(machine.machine, the_studio());
    assert_eq!(machine.port, THE_WIRE_PORT);
    assert_eq!(
        machine.addresses().collect::<BTreeSet<_>>(),
        both,
        "{machine:?}"
    );
    assert_eq!(machine.also_at.len(), 1, "{machine:?}");
    let by_name = found_by_name(&the_studio(), the_group()).unwrap();
    assert_eq!(by_name.addresses().collect::<BTreeSet<_>>(), both);
    println!("one machine, an address on each network");

    assert_eq!(around.workspaces.len(), 1, "{around:?}");
    let workspace = around.workspaces.first().unwrap();
    assert_eq!(workspace.host(), &the_studio());
    assert_eq!(workspace.port(), WORKSPACE);
    assert_eq!(
        workspace.addresses().collect::<BTreeSet<_>>(),
        both,
        "{workspace:?}"
    );
    println!("one workspace, an address on each network");

    let expected: BTreeSet<Vec<u8>> = [
        advertising::about(&Presence::of(the_studio(), THE_WIRE_PORT)).unwrap(),
        advertising::about_a_workspace(&WorkspacePresence::of(the_studio(), WORKSPACE)).unwrap(),
    ]
    .into();
    for (here, studios) in [
        (RECEPTION_WIRED, STUDIO_WIRED),
        (RECEPTION_WIRELESS, STUDIO_WIRELESS),
    ] {
        let asking = UdpSocket::bind((here, 0)).unwrap();
        asking
            .send_to(&advertising::a_question().unwrap(), the_group())
            .unwrap();
        asking
            .send_to(
                &advertising::a_question_for_workspaces().unwrap(),
                the_group(),
            )
            .unwrap();
        let heard = everything_heard(&asking);
        assert!(
            heard.iter().all(|(from, _)| *from == IpAddr::from(studios)),
            "an answer on the network at {here} came from elsewhere: {heard:?}"
        );
        let bytes: BTreeSet<Vec<u8>> = heard.into_iter().map(|(_, bytes)| bytes).collect();
        assert_eq!(bytes, expected, "the network at {here}");
    }
    println!("the same bytes on each network");

    // 5. And the studio's own answer to `advertised`.
    drop(telling);
    let advertised = the_studio_says(&mut hearing, "advertised ");
    assert_eq!(advertised, "the same bytes");
    println!("advertised is the same bytes");
    assert!(studio.wait().unwrap().success(), "the studio failed");
}

/// The studio, inside a network of its own nested in reception's: it binds its
/// wire as a machine does, follows the kernel when told to, and says what it
/// advertises.
#[test]
#[ignore = "run inside its own network by reception_inside_its_own_network"]
fn the_studio_inside_its_own_network() {
    must_be("studio");
    let strings = alo_agentd::what_this_machine_says().unwrap().into_strings();
    let hosted = Hosted::At(NonZeroU16::new(WORKSPACE).unwrap());
    let told = |wire: &Wire| {
        alo_agentd::what_is_advertised::told(&wire.advertising(), &strings)
            .written()
            .unwrap()
    };
    let mut lines = std::io::stdin().lock().lines();

    // What a wire with no network at all would say, at the wire's own port.
    let alone = Wire::on(
        TcpListener::bind((Ipv4Addr::LOCALHOST, THE_WIRE_PORT)).unwrap(),
        UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
        the_studio(),
        0,
    )
    .unwrap()
    .hosting(hosted);
    let with_none = told(&alone);
    drop(alone);

    // 1. One network, and the wire bound on it.
    assert_eq!(lines.next().unwrap().unwrap(), "wired");
    let until = Instant::now() + Duration::from_secs(10);
    while discovery_networks(&alo_agentd::route_messages::reported_by_the_kernel().unwrap())
        .is_empty()
    {
        assert!(Instant::now() < until, "the wired network never came up");
        std::thread::sleep(Duration::from_millis(20));
    }
    let wire = Wire::bound(the_studio()).unwrap().hosting(hosted);
    let with_one = told(&wire);
    assert_eq!(wire.joined().len(), 1, "{:?}", wire.joined());
    println!("bound on {}", wire.joined().len());

    std::thread::scope(|scope| {
        scope.spawn(|| while wire.answer_discovery().is_ok() {});

        // 3. Told the second network is there: wait for the kernel to say so on
        //    the wire's own socket, then do what the service does.
        assert_eq!(lines.next().unwrap().unwrap(), "wireless");
        let told_fd = wire
            .networks_waiting_on()
            .expect("the kernel says when a network changes")
            .try_clone_to_owned()
            .unwrap();
        let notifications = UdpSocket::from(told_fd);
        let until = Instant::now() + Duration::from_secs(10);
        let mut peeked = [0_u8; 64];
        while notifications.peek(&mut peeked).is_err() {
            assert!(
                Instant::now() < until,
                "the kernel never said a network appeared"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        let until = Instant::now() + Duration::from_secs(10);
        while wire.joined().len() < 2 {
            assert!(Instant::now() < until, "joined on {:?}", wire.joined());
            wire.networks_changed();
            std::thread::sleep(Duration::from_millis(20));
        }
        println!("joined on {}", wire.joined().len());

        // 5. Reception is done looking.
        while lines.next().is_some() {}
        let with_two = told(&wire);
        assert_eq!(with_two, with_one);
        assert_eq!(with_two, with_none);
        println!("advertised the same bytes");
        // The answering thread is blocked on a socket with no timeout, as the
        // service's is, and nothing can end that wait from here — so the
        // studio ends as a process does, with reception reading its status.
        std::process::exit(0);
    });
}
