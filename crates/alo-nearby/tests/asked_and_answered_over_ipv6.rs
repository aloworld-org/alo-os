//! *Machines find each other with zero configuration* — on a network nobody gave
//! an IPv4 address too.
//!
//! Two machines on one cable with no DHCP server between them, or an office
//! whose router is down for the afternoon, have no IPv4 address in common; each
//! interface still has an IPv6 link-local one. This is the crate's half of that:
//!
//! | The acceptance | The test |
//! |---|---|
//! | asked and answered over IPv6 as over IPv4, with the same closed advertisement, byte for byte | [`asked_and_answered_over_ipv6_as_over_ipv4`] |
//! | a packet saying more than presence is refused whichever family carried it | the same test, second part |
//! | a machine heard on a link-local address carries the interface it was heard on | the same test, third part |
//!
//! # Where it runs
//!
//! In a network namespace of its own, made by util-linux's `unshare` inside a
//! user namespace, with two `veth` interfaces joined to each other and nothing
//! else. That is not a nicety: a development machine may have IPv6 switched off
//! outright (the WSL2 kernel this was written on does, in its first namespace),
//! and a fresh namespace has it on — so what is measured here is this crate, not
//! whatever the host happens to be configured as. Nothing outside the namespace
//! is touched.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeSet;
use std::env;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6, UdpSocket};
use std::num::NonZeroU16;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use alo_nearby::{
    Answering, Looking, MachineId, NotNearby, Presence, THE_IPV6_ADDRESS, WorkspacePresence,
    advertising, reading,
};

/// Set on the binary run again inside the namespace.
const INSIDE: &str = "ALO_NEARBY_OVER_IPV6_INSIDE";

/// The machine answering.
fn the_studio() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// **Asked and answered over IPv6 as over IPv4**: the outer test, which makes
/// the namespace and reads what happened in it.
#[test]
fn asked_and_answered_over_ipv6_as_over_ipv4() {
    let exe = env::current_exe().expect("a test binary knows where it is");
    let script = "ip link set lo up \
        && ip link add near0 type veth peer name near1 \
        && ip link set near0 up && ip link set near1 up \
        && exec \"$0\" --exact --ignored --nocapture inside_a_network_of_its_own";
    let ran = Command::new("unshare")
        .args(["--map-root-user", "--net", "--", "sh", "-c", script])
        .arg(exe)
        .env(INSIDE, "yes")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .expect("util-linux's `unshare` makes the namespace, and this machine does not have it");
    let said = String::from_utf8_lossy(&ran.stdout).into_owned();
    assert!(
        ran.status.success(),
        "inside failed ({}):\n{said}",
        ran.status
    );
    for step in [
        "the same bytes in both families",
        "found over ipv6",
        "refused over ipv6",
        "a link-local address carries its interface",
    ] {
        assert!(said.contains(step), "never said `{step}`:\n{said}");
    }
}

/// A socket bound at `at`, with a read timeout.
fn bound(at: SocketAddr) -> UdpSocket {
    let socket = UdpSocket::bind(at).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_millis(800)))
        .unwrap();
    socket
}

/// Every datagram that came back to `socket` before it went quiet.
fn everything_heard(socket: &UdpSocket) -> Vec<(SocketAddr, Vec<u8>)> {
    let mut heard = Vec::new();
    let mut datagram = [0_u8; 1_500];
    while let Ok((read, from)) = socket.recv_from(&mut datagram) {
        heard.push((from, datagram.get(..read).unwrap().to_vec()));
    }
    heard
}

/// The link-local address and the kernel's index for the interface `name`,
/// once the kernel has finished checking nobody else holds it — read out of
/// `/proc/net/if_inet6`, whose fifth column is the address's flags.
fn link_local_on(name: &str) -> (Ipv6Addr, u32) {
    /// `IFA_F_TENTATIVE`: the kernel has not finished duplicate address
    /// detection, and nothing can be bound to the address yet.
    const TENTATIVE: u32 = 0x40;
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        let table = std::fs::read_to_string("/proc/net/if_inet6").unwrap();
        let found = table.lines().find_map(|line| {
            let columns: Vec<&str> = line.split_whitespace().collect();
            let [address, index, _, _, flags, interface] = columns.as_slice() else {
                return None;
            };
            let flags = u32::from_str_radix(flags, 16).ok()?;
            let octets: Vec<u8> = (0..16)
                .map(|at| u8::from_str_radix(address.get(at * 2..at * 2 + 2)?, 16).ok())
                .collect::<Option<_>>()?;
            let ip = Ipv6Addr::from(<[u8; 16]>::try_from(octets).ok()?);
            (*interface == name && ip.segments()[0] == 0xfe80 && flags & TENTATIVE == 0)
                .then(|| u32::from_str_radix(index, 16).ok().map(|index| (ip, index)))
                .flatten()
        });
        if let Some(found) = found {
            return found;
        }
        assert!(
            Instant::now() < until,
            "{name} never had a usable link-local address:\n{table}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Inside the namespace: loopback in both families, a stranger saying too much,
/// and a question over link-local between the two ends of a cable.
#[test]
#[ignore = "run inside a network namespace by asked_and_answered_over_ipv6_as_over_ipv4"]
fn inside_a_network_of_its_own() {
    assert_eq!(
        env::var(INSIDE).ok().as_deref(),
        Some("yes"),
        "this test runs inside the namespace the outer test makes; run the outer one"
    );
    let workspace_port = NonZeroU16::new(8_443).unwrap();
    let presence = Presence::of(the_studio(), 7_610);

    // 1. The same responder in both families answers with the same bytes.
    let expected: BTreeSet<Vec<u8>> = [
        advertising::about(&presence).unwrap(),
        advertising::about_a_workspace(&WorkspacePresence::of(the_studio(), 8_443)).unwrap(),
    ]
    .into();
    for loopback in [
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(Ipv6Addr::LOCALHOST),
    ] {
        let socket = bound(SocketAddr::new(loopback, 0));
        let at = socket.local_addr().unwrap();
        let answering =
            Answering::on(socket, presence.clone()).hosting_a_workspace_at(workspace_port);
        let asking = bound(SocketAddr::new(loopback, 0));
        asking
            .send_to(&advertising::a_question().unwrap(), at)
            .unwrap();
        asking
            .send_to(&advertising::a_question_for_workspaces().unwrap(), at)
            .unwrap();
        assert!(answering.answer_one().unwrap().is_some());
        assert!(answering.answer_one().unwrap().is_some());
        let heard: BTreeSet<Vec<u8>> = everything_heard(&asking)
            .into_iter()
            .map(|(_, bytes)| bytes)
            .collect();
        assert_eq!(heard, expected, "over {loopback}");
    }
    println!("the same bytes in both families");

    // And a look over IPv6 finds the machine and the workspace at the address
    // they answered from.
    let socket = bound(SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0));
    let at = socket.local_addr().unwrap();
    let answering = Answering::on(socket, presence.clone()).hosting_a_workspace_at(workspace_port);
    let answered = std::thread::spawn(move || {
        (0..2)
            .map(|_| answering.answer_one().unwrap().is_some())
            .collect::<Vec<_>>()
    });
    let looking = Looking::from(bound(SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0)));
    looking.ask(at).unwrap();
    looking.ask_for_workspaces(at).unwrap();
    let around = looking.around(Duration::from_millis(800)).unwrap();
    assert_eq!(answered.join().unwrap(), vec![true, true]);
    assert_eq!(around.machines.len(), 1, "{around:?}");
    let machine = around.machines.first().unwrap();
    assert_eq!(machine.machine, the_studio());
    assert_eq!(
        machine.where_it_answers(),
        SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 7_610)
    );
    assert_eq!(around.workspaces.len(), 1, "{around:?}");
    assert_eq!(
        around.workspaces.first().unwrap().where_it_answers(),
        SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 8_443)
    );
    println!("found over ipv6");

    // 2. A stranger over IPv6 saying more than presence is refused, exactly as
    //    over IPv4, and found as nothing. The TXT record is the last thing in
    //    the packet: its data length, then `v=1`; one entry more goes beside it.
    let mut saying_more = advertising::about(&presence).unwrap();
    let whole = saying_more.len();
    saying_more.truncate(whole - 6);
    let more = b"who=disan";
    saying_more.extend_from_slice(&[0, u8::try_from(4 + 1 + more.len()).unwrap()]);
    saying_more.extend_from_slice(&[3, b'v', b'=', b'1']);
    saying_more.push(u8::try_from(more.len()).unwrap());
    saying_more.extend_from_slice(more);
    let refused = NotNearby::SaysMoreThanPresence("who".to_owned());
    assert_eq!(
        reading::a_machine_in(&saying_more, Ipv4Addr::LOCALHOST.into()).unwrap_err(),
        refused
    );
    assert_eq!(
        reading::a_machine_heard(
            &saying_more,
            SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 5_353)
        )
        .unwrap_err(),
        refused
    );
    let stranger = bound(SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0));
    let strangers_at = stranger.local_addr().unwrap();
    let answering = std::thread::spawn(move || {
        let mut heard = [0_u8; 1_500];
        let (_, who) = stranger.recv_from(&mut heard).unwrap();
        stranger.send_to(&saying_more, who).unwrap();
    });
    let looking = Looking::from(bound(SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0)));
    looking.ask(strangers_at).unwrap();
    let found = looking.found(Duration::from_millis(800)).unwrap();
    answering.join().unwrap();
    assert!(
        found.is_empty(),
        "a machine saying more was found: {found:?}"
    );
    println!("refused over ipv6");

    // 3. Over link-local, between the two ends of one cable: the question goes
    //    to the group on one end's interface, the other end has joined the group,
    //    and what is found carries the interface it was heard on.
    let (asking_from, asking_on) = link_local_on("near0");
    let (answering_ip, answering_on) = link_local_on("near1");
    let socket = bound(SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0));
    socket
        .join_multicast_v6(&THE_IPV6_ADDRESS, answering_on)
        .unwrap();
    let port = socket.local_addr().unwrap().port();
    let answering = Answering::on(socket, presence);
    let answered = std::thread::spawn(move || answering.answer_one());
    let looking = Looking::from(bound(SocketAddr::V6(SocketAddrV6::new(
        asking_from,
        0,
        0,
        asking_on,
    ))));
    looking
        .ask(SocketAddr::V6(SocketAddrV6::new(
            THE_IPV6_ADDRESS,
            port,
            0,
            asking_on,
        )))
        .unwrap();
    let found = looking.found(Duration::from_secs(2)).unwrap();
    assert!(answered.join().unwrap().unwrap().is_some());
    assert_eq!(found.len(), 1, "{found:?}");
    let one = found.first().unwrap();
    assert_eq!(one.machine, the_studio());
    assert_eq!(one.address.ip(), IpAddr::V6(answering_ip));
    assert_eq!(one.address.scope(), Some(asking_on));
    assert_eq!(
        one.where_it_answers(),
        SocketAddr::V6(SocketAddrV6::new(answering_ip, 7_610, 0, asking_on))
    );
    println!("a link-local address carries its interface");
}
