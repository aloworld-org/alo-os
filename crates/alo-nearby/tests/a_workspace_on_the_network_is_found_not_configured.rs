//! *A self-hosted workspace on the network is discovered, not configured — no
//! DNS step* — and [ADR 0003]'s limit on what that may cost: discovery reveals
//! presence and nothing else, and finding something confers nothing.
//!
//! | The acceptance | The test |
//! |---|---|
//! | a workspace advertised under its own service is found, with which workspace, where it answers and the version | [`a_workspace_advertised_on_the_network_is_found_with_exactly_the_closed_list`] |
//! | an advertisement carrying one field more is refused rather than read around | [`a_workspace_advertisement_carrying_one_field_more_is_refused_and_not_found`] |
//! | finding a workspace contacts nothing | [`finding_a_workspace_contacts_nothing_and_pairs_nothing`] |
//! | an address a person types has no road into a found workspace | [`only_an_advertisement_heard_makes_a_found_workspace`] |
//!
//! # What no test here shows
//!
//! Both sides are on this host, over ordinary datagrams. **Whether a second
//! physical machine on an office network hears a workspace served by
//! `alo-workplace` is owed to two machines and that repository's responder**,
//! and is not claimed anywhere in this file.
//!
//! [ADR 0003]: https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0003-the-network-is-not-authority.md

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::net::{Ipv4Addr, TcpListener, UdpSocket};
use std::time::Duration;

use alo_nearby::{
    Looking, MachineId, NotNearby, Standing, VERSION, WORKSPACE_SERVICE, WorkspacePresence,
    advertising, reading,
};

/// A socket bound to this machine and nowhere else.
fn a_socket() -> UdpSocket {
    let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    socket
}

/// A workspace host that answers the first question it hears with `packet`,
/// on a thread of its own, at the address it returns.
fn a_host_answering_with(packet: Vec<u8>) -> (std::net::SocketAddr, std::thread::JoinHandle<()>) {
    let host = a_socket();
    let at = host.local_addr().unwrap();
    let answering = std::thread::spawn(move || {
        let mut heard = [0_u8; 1_500];
        let (_, who) = host.recv_from(&mut heard).unwrap();
        host.send_to(&packet, who).unwrap();
    });
    (at, answering)
}

/// **A workspace advertised under its own service is found by another
/// machine, and what is read is exactly the closed list** — which workspace,
/// where it answers (the address it was heard from, the port it advertised)
/// and the version — paired with nothing.
#[test]
fn a_workspace_advertised_on_the_network_is_found_with_exactly_the_closed_list() {
    let host = MachineId::made().unwrap();
    let packet =
        advertising::about_a_workspace(&WorkspacePresence::of(host.clone(), 8_443)).unwrap();
    let (at, answering) = a_host_answering_with(packet);

    let looking = Looking::from(a_socket());
    looking.ask_for_workspaces(at).unwrap();
    let around = looking.around(Duration::from_millis(800)).unwrap();
    answering.join().unwrap();

    assert!(around.machines.is_empty(), "{around:?}");
    assert_eq!(around.workspaces.len(), 1, "{around:?}");
    let found = around.workspaces.first().unwrap();
    assert_eq!(found.host(), &host);
    assert_eq!(
        found.where_it_answers(),
        std::net::SocketAddr::new(at.ip(), 8_443)
    );
    assert_eq!(found.version(), VERSION);
    assert_eq!(found.standing(), Standing::NotPaired);
    assert_eq!(WORKSPACE_SERVICE, "_alo-workspace._tcp.local");
}

/// **One field more is refused, not read around**, and a workspace saying it
/// is not found at all: the list a workspace may advertise is closed, and a
/// key nobody reads is how it would quietly start saying more.
#[test]
fn a_workspace_advertisement_carrying_one_field_more_is_refused_and_not_found() {
    let host = MachineId::made().unwrap();
    let mut packet = advertising::about_a_workspace(&WorkspacePresence::of(host, 8_443)).unwrap();
    // The `TXT` record is the last thing in the packet: its data length, then
    // `v=1` behind its own length. Put one entry more beside it.
    let whole = packet.len();
    packet.truncate(whole - 6);
    let more = b"org=axon";
    packet.extend_from_slice(&[0, u8::try_from(4 + 1 + more.len()).unwrap()]);
    packet.extend_from_slice(&[3, b'v', b'=', b'1']);
    packet.push(u8::try_from(more.len()).unwrap());
    packet.extend_from_slice(more);

    assert_eq!(
        reading::a_workspace_in(&packet, Ipv4Addr::LOCALHOST.into()).unwrap_err(),
        NotNearby::SaysMoreThanPresence("org".to_owned())
    );

    let (at, answering) = a_host_answering_with(packet);
    let looking = Looking::from(a_socket());
    looking.ask_for_workspaces(at).unwrap();
    let around = looking.around(Duration::from_millis(800)).unwrap();
    answering.join().unwrap();
    assert!(
        around.workspaces.is_empty(),
        "a workspace saying more than the list was found: {around:?}"
    );
}

/// **Finding a workspace contacts nothing.** Where it answers is a listener
/// this test holds; after it was advertised and found on a network where
/// nothing is paired, nothing has connected to it.
#[test]
fn finding_a_workspace_contacts_nothing_and_pairs_nothing() {
    let workspace = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    workspace.set_nonblocking(true).unwrap();
    let port = workspace.local_addr().unwrap().port();

    let packet =
        advertising::about_a_workspace(&WorkspacePresence::of(MachineId::made().unwrap(), port))
            .unwrap();
    let (at, answering) = a_host_answering_with(packet);
    let looking = Looking::from(a_socket());
    looking.ask_for_workspaces(at).unwrap();
    let around = looking.around(Duration::from_millis(800)).unwrap();
    answering.join().unwrap();

    let found = around.workspaces.first().unwrap();
    assert_eq!(found.where_it_answers().port(), port);
    assert_eq!(found.standing(), Standing::NotPaired);
    assert_eq!(
        workspace.accept().map(|_| ()).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock,
        "something connected to a workspace because it was found"
    );
}

/// **An address a person types has no road into a found workspace.** The type
/// has no public constructor (its `compile_fail` example holds that), and in
/// the crate's shipped code it is made in exactly one place — reading an
/// advertisement — which this reads off the source.
#[test]
fn only_an_advertisement_heard_makes_a_found_workspace() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut making = Vec::new();
    for file in std::fs::read_dir(&source).unwrap() {
        let file = file.unwrap().path();
        let written = std::fs::read_to_string(&file).unwrap();
        let ships = written
            .split_once("#[cfg(test)]")
            .map_or(written.as_str(), |(before, _)| before);
        for line in ships.lines() {
            if line.contains("FoundWorkspace::heard(") && !line.trim_start().starts_with("//") {
                making.push(file.file_name().unwrap().to_string_lossy().into_owned());
            }
        }
    }
    assert_eq!(
        making,
        ["reading.rs"],
        "a found workspace is made somewhere else"
    );

    // And what reads an advertisement never reads an address out of one:
    // where a found workspace answers is where its answer was heard from.
    let packet =
        advertising::about_a_workspace(&WorkspacePresence::of(MachineId::made().unwrap(), 443))
            .unwrap();
    let heard_from = Ipv4Addr::new(10, 9, 8, 7).into();
    assert_eq!(
        reading::a_workspace_in(&packet, heard_from)
            .unwrap()
            .where_it_answers()
            .ip(),
        heard_from
    );
}
