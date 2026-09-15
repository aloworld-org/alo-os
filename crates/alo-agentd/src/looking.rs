//! Discovery, done honestly, at the moment a proposal arrives.
//!
//! `alo_nearby::Proposals::arrived` refuses a proposal from an address
//! discovery on this machine has never measured — task 7's *a proposal that
//! arrived from an address discovery never measured is refused before anybody
//! is shown anything* — and takes the list of what was measured as an
//! argument. This daemon keeps no such list, because a list kept would age:
//! a machine found this morning at one address may be at another by the
//! afternoon, and a proposal judged against the morning's list would be
//! judged against a stale fact.
//!
//! So the measurement is made at the moment: the machine the proposal came
//! from is asked, at the address the connection came from and the port its
//! own discovery answers at, whether it exists, and what it answers is what
//! the proposal is judged against. A machine that does not answer was not
//! found, and its proposal is refused as such — which is the true thing.
//!
//! **Nothing here is a lookup of anything typed.** The address is the
//! connection's, measured by the kernel; the port is the wire's constant.
//!
//! # And for a proposal the person here makes
//!
//! The person's door proposes a pairing to a machine by its identity
//! (`crate::pairing`), and the identity is all it carries: no address, so
//! nothing typed can be dialled. [`found_by_name`] asks the link who is here
//! — the multicast group on a real machine, the socket the other side of a
//! test bound — and answers with the one machine that said it was the one
//! asked for, at the address it answered from. A machine that does not
//! answer is not found, and nothing is proposed to it. [`LookingFor`] is that
//! question as the door asks it, so the door can be tested against a network
//! with nobody on it.
//!
//! # And for the workspaces on the network
//!
//! The person's door lists the workspaces discovery finds
//! (`crate::listing_workspaces`), and [`around_at`] is that look: both
//! questions — who is here, and which workspaces — asked at once on the link,
//! and everything that answered in the one window. Again nothing is kept, so
//! the list is the network at the moment it was asked about.
//!
//! # And on every network this machine is on
//!
//! A question to the discovery group is asked on **each** network this machine
//! is on at the moment of asking (`crate::networks`), from that network's own
//! address, rather than once by the default route — so a docked laptop looks on
//! the wired network and the Wi-Fi both, and a machine heard on two of them is
//! one machine with the address it answered from on each
//! (`alo_nearby::Around::heard_on_each`). The networks are read again at every
//! look, for the same reason nothing else here is kept.
//!
//! # And in both families
//!
//! Each network with an IPv6 link-local address is asked as well, at `ff02::fb`
//! on its interface from that link-local address — so two machines on a cable
//! with no IPv4 address between them are still heard — and every IPv4 network
//! is asked before any IPv6 one is handed to the merge. A machine heard in both
//! families is one machine with an address in each, **written down at its IPv4
//! address**, which is the one a pairing dials when both answered: it is the
//! address that machine was reached at before a second family was asked, it
//! needs no interface beside it to be dialled, and it is the same address the
//! machine's own person sees for it. Where only IPv6 answered, the link-local
//! address is dialled with the interface it was heard on.
//!
//! # And an IPv4 address is heard on the network it answered on
//!
//! [ADR 0044](../../../docs/decisions/0044-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md).
//! Two networks can hand out the same private range, so `192.168.1.20` heard on
//! the wired network and on the Wi-Fi are two machines. A look on an IPv4
//! network is therefore asked from a socket **held to that network's
//! interface** — so an answer counted as heard there arrived there, even when
//! this machine's own address is the same on both — and every machine it heard
//! is written down on that interface (`alo_nearby::HeardFrom::on_the_network`).
//! A question to it is then held to the same interface (`crate::corridor`).
//!
//! **A proposal's measurement follows its connection's family, and the network
//! its connection arrived on.** The asking machine is asked at the address its
//! connection came from — with the interface, for a link-local one
//! ([`found_at`]) — because a link-local address without its interface names no
//! network and a datagram to it would go nowhere; and an IPv4 connection is
//! measured from a socket **held to the network it arrived on**
//! (`crate::arrived_on`), for the reason above: `192.168.1.20` on the cable and
//! `192.168.1.20` on the Wi-Fi are two machines, and the one that connected is
//! the one the proposal is judged against. A connection whose arriving network
//! could not be read is measured nowhere.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6, UdpSocket};
use std::num::NonZeroU32;
use std::time::Duration;

use alo_nearby::{Around, Found, HeardFrom, Looking, MachineId, THE_IPV6_ADDRESS};

use crate::arrived_on::ArrivedOn;
use crate::networks::{Network, every_discovery_network};
use crate::route_messages::reported_by_the_kernel;

/// Somewhere a machine can be looked for by its identity, at the moment.
///
/// One method, answering what discovery measured just now or nothing. The
/// `Wire` is the implementation that ships (`crate::wire`), asking the link
/// it is bound to; a test hands the door a network with nobody on it.
pub trait LookingFor: std::fmt::Debug {
    /// The machine with this identity, if it answered on the network just
    /// now, at the address it answered from.
    fn look_for(&self, machine: &MachineId) -> Option<Found>;

    /// Every machine and every workspace that answered on the network just
    /// now, each at the address it answered from — nothing at all on a link
    /// with nobody on it, which is an answer rather than a failure.
    fn look_around(&self) -> Around;
}

/// Ask `at` which machines and which workspaces are here, and answer with
/// everything that answered within [`WHILE_LOOKING`].
///
/// One socket and one window for both questions, so a workspace and the
/// machine that says it serves it are heard at the same moment
/// (`alo_nearby::Looking::around`). Nothing that answered is contacted: this
/// sends two questions and reads what comes back, and a socket that will not
/// bind or a question that cannot be sent is *nothing found*.
#[must_use]
pub fn around_at(at: SocketAddr) -> Around {
    heard_at(at, true)
}

/// Ask who is here at `at`, and answer with the machine named `machine` if
/// it answered, at the address it answered from — on a machine on several
/// networks, the address on the first network it was heard on, with the others
/// beside it.
///
/// `None` when nothing answered by that name, nothing could be asked, or the
/// socket would not bind — every one of which the caller reads as *not
/// found*: a proposal to a machine this one cannot find is refused before
/// anything is sent, which is the whole of what the measurement is for.
#[must_use]
pub fn found_by_name(machine: &MachineId, at: SocketAddr) -> Option<Found> {
    heard_at(at, false)
        .machines
        .into_iter()
        .find(|found| found.machine == *machine)
}

/// Where a look at `at` is asked from: every network this machine is on for
/// the discovery group, and one socket otherwise.
///
/// **A question to the group leaves on each network**, from a socket bound to
/// that network's own address — which on Linux is what sends a multicast
/// datagram out of the interface that owns the address rather than by the
/// default route (`docs/quirks.md`). What each network heard is its own window,
/// all of them at once, and [`Around::heard_on_each`] makes one answer of them:
/// a machine heard on two networks is one machine with an address on each.
///
/// A question to one address — a test's socket, or loopback — is one socket and
/// one window, as it always was: a unicast datagram leaves by the route to it.
/// A machine whose networks cannot be read is looked for on none, and the
/// service log says why: that is *nothing found*, which is true of a machine
/// that cannot say where it is.
fn heard_at(at: SocketAddr, workspaces_too: bool) -> Around {
    if !at.ip().is_multicast() {
        return heard_from(asked_from(at), at, workspaces_too);
    }
    let networks = match reported_by_the_kernel() {
        Ok(reported) => every_discovery_network(&reported),
        Err(why) => {
            eprintln!(
                "alo-agentd: this machine's networks could not be read ({why}); nothing was asked"
            );
            return Around::default();
        }
    };
    Around::heard_on_each(heard_on(&networks, at, workspaces_too))
}

/// What each of `networks` heard when asked at the group `at` names, in the
/// same order, each asked at once and heard for [`WHILE_LOOKING`].
fn heard_on(networks: &[Network], at: SocketAddr, workspaces_too: bool) -> Vec<Around> {
    std::thread::scope(|scope| {
        let asking: Vec<_> = networks
            .iter()
            .map(|network| scope.spawn(move || heard_on_one(network, at.port(), workspaces_too)))
            .collect();
        asking
            .into_iter()
            .map(|asked| asked.join().unwrap_or_default())
            .collect()
    })
}

/// What one network heard when asked at the group on `port`.
///
/// An IPv4 network is asked from a socket held to its interface, and every
/// machine that answered is written down on that interface (ADR 0044); a
/// link-local network is asked from its own scoped address, whose answers carry
/// their interface already (ADR 0041). Nothing at all when the socket cannot be
/// made — *nothing found*, for [`around_at`]'s reason.
fn heard_on_one(network: &Network, port: u16, workspaces_too: bool) -> Around {
    let (here, there) = asked_on(network, port);
    if here.is_ipv6() {
        return heard_from(here, there, workspaces_too);
    }
    let Ok(socket) = held_to(network.index(), here) else {
        return Around::default();
    };
    let mut heard = heard_by(socket, there, workspaces_too);
    for found in &mut heard.machines {
        found.address = found.address.on_the_network(network.index());
    }
    heard
}

/// A datagram socket bound to `here` and held to the interface the kernel
/// numbers `interface`, so that it sends and hears on that network alone.
///
/// # Errors
/// What the kernel answered: an interface that went away since the networks
/// were read, and everything a bind can fail with.
fn held_to(interface: u32, here: SocketAddr) -> std::io::Result<UdpSocket> {
    let socket = socket2::Socket::new(
        socket2::Domain::for_address(here),
        socket2::Type::DGRAM,
        None,
    )?;
    socket.bind_device_by_index_v4(NonZeroU32::new(interface))?;
    socket.bind(&here.into())?;
    Ok(socket.into())
}

/// Where a question on `network` leaves from and goes to: from the network's
/// IPv4 address to `224.0.0.251`, or from its link-local address to `ff02::fb`
/// on its interface — both at `port`.
fn asked_on(network: &Network, port: u16) -> (SocketAddr, SocketAddr) {
    match network.address() {
        IpAddr::V4(address) => (
            SocketAddr::new(address.into(), 0),
            SocketAddr::new(alo_nearby::THE_ADDRESS.into(), port),
        ),
        IpAddr::V6(address) => (
            SocketAddr::V6(SocketAddrV6::new(address, 0, 0, network.index())),
            SocketAddr::V6(SocketAddrV6::new(
                THE_IPV6_ADDRESS,
                port,
                0,
                network.index(),
            )),
        ),
    }
}

/// Where a question to one address leaves from: loopback for loopback, and any
/// address otherwise — in that address's family.
const fn asked_from(at: SocketAddr) -> SocketAddr {
    let loopback = at.ip().is_loopback();
    let here = match (at, loopback) {
        (SocketAddr::V4(_), true) => IpAddr::V4(Ipv4Addr::LOCALHOST),
        (SocketAddr::V4(_), false) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        (SocketAddr::V6(_), true) => IpAddr::V6(Ipv6Addr::LOCALHOST),
        (SocketAddr::V6(_), false) => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
    };
    SocketAddr::new(here, 0)
}

/// Ask at `at` from a socket bound to `here`, and hear for [`WHILE_LOOKING`].
///
/// Nothing at all when the socket will not bind or a question will not send —
/// *nothing found*, for [`around_at`]'s reason.
fn heard_from(here: SocketAddr, at: SocketAddr, workspaces_too: bool) -> Around {
    let Ok(socket) = UdpSocket::bind(here) else {
        return Around::default();
    };
    heard_by(socket, at, workspaces_too)
}

/// Ask at `at` from `socket`, and hear for [`WHILE_LOOKING`].
fn heard_by(socket: UdpSocket, at: SocketAddr, workspaces_too: bool) -> Around {
    let looking = Looking::from(socket);
    if looking.ask(at).is_err() || (workspaces_too && looking.ask_for_workspaces(at).is_err()) {
        return Around::default();
    }
    looking.around(WHILE_LOOKING).unwrap_or_default()
}

/// How long this machine waits for the machine that proposed to say it
/// exists.
///
/// Two seconds: it is on the same link, it is the machine that just opened a
/// connection here, and a proposal is a person waiting at a screen rather
/// than an agent in a loop.
pub const WHILE_LOOKING: Duration = Duration::from_secs(2);

/// Ask the machine at `from` whether it exists, at the port its discovery
/// answers on, and answer with what was found.
///
/// `from` is the address the connection came from, with the interface for a
/// link-local one, and the question goes to it in its own family. `arrived` is
/// the network the connection came in on (`crate::arrived_on`), and it is what
/// the question is **held to**: an IPv4 connection that arrived on the cable is
/// measured on the cable, so the machine that answers is the one that
/// connected, and not somebody else at the same private address on the Wi-Fi
/// (ADR 0044). Everything found is written down on that network
/// ([`HeardFrom::on_the_network`]), which is what a question to it is then held
/// to in turn.
///
/// Empty when nothing answered or nothing could be asked — which includes a
/// link-local address with no interface, which names no network to ask on, and
/// a connection whose arriving network could not be read
/// ([`ArrivedOn::NothingCouldSay`]), which is **measured nowhere** rather than
/// measured by the route. The caller reads all of those as *not found*: a
/// proposal from a machine this one cannot find is refused by
/// `alo_nearby::Proposals::arrived`, and refusing it is the whole of what this
/// measurement is for.
#[must_use]
pub fn found_at(from: impl Into<HeardFrom>, arrived: ArrivedOn, asking_at: u16) -> Vec<Found> {
    let from = from.into();
    if !from.names_a_network() {
        return Vec::new();
    }
    let at = from.at(asking_at);
    let here = asked_from(at);
    let socket = match arrived {
        ArrivedOn::NothingCouldSay => return Vec::new(),
        ArrivedOn::ItsOwnNetwork => UdpSocket::bind(here),
        ArrivedOn::TheNetwork(interface) => held_to(interface.get(), here),
    };
    let Ok(socket) = socket else {
        return Vec::new();
    };
    let looking = Looking::from(socket);
    if looking.ask(at).is_err() {
        return Vec::new();
    }
    let mut found = looking.found(WHILE_LOOKING).unwrap_or_default();
    if let ArrivedOn::TheNetwork(interface) = arrived {
        for one in &mut found {
            one.address = one.address.on_the_network(interface.get());
            for also in &mut one.also_at {
                *also = also.on_the_network(interface.get());
            }
        }
    }
    found
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::{Ipv4Addr, UdpSocket};

    use alo_nearby::{Answering, MachineId, Presence};

    use super::found_at;
    use crate::arrived_on::ArrivedOn;

    /// The machine that proposed, for these tests.
    fn reception() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// **A machine that answers at the address it came from is found there**,
    /// with the port it advertises.
    #[test]
    fn a_machine_that_answers_is_found_at_the_address_it_came_from() {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = socket.local_addr().unwrap();
        let answering = Answering::on(socket, Presence::of(reception(), 7_610));
        let answered = std::thread::spawn(move || answering.answer_one());

        let found = found_at(at.ip(), ArrivedOn::ItsOwnNetwork, at.port());
        assert!(answered.join().unwrap().unwrap().is_some());
        let one = found.iter().find(|one| one.machine == reception()).unwrap();
        assert_eq!(one.address, at.ip());
        assert_eq!(one.port, 7_610);
    }

    /// **A proposal from a link-local address that says no interface is
    /// measured nowhere**: the address names no network, nothing is asked, and
    /// the machine is not found — so its proposal is refused.
    #[test]
    fn a_link_local_address_with_no_interface_is_looked_for_nowhere() {
        let unscoped: std::net::IpAddr = "fe80::a406:e5ff:fe4b:ac9e".parse().unwrap();
        let started = std::time::Instant::now();
        assert!(found_at(unscoped, ArrivedOn::ItsOwnNetwork, alo_nearby::THE_PORT).is_empty());
        assert!(
            started.elapsed() < super::WHILE_LOOKING,
            "a question was asked and waited on"
        );
    }

    /// **A connection whose arriving network could not be read is measured
    /// nowhere**: nothing is asked at all, so the machine that would have
    /// answered never hears the question, and its proposal is refused as *not
    /// found* rather than judged against whoever the route reaches (ADR 0044).
    #[test]
    fn a_connection_whose_network_could_not_be_read_is_measured_nowhere() {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        socket
            .set_read_timeout(Some(std::time::Duration::from_millis(250)))
            .unwrap();
        let at = socket.local_addr().unwrap();
        let answering = Answering::on(socket, Presence::of(reception(), 7_610));
        let asked = std::thread::spawn(move || answering.answer_one());

        let started = std::time::Instant::now();
        assert!(found_at(at.ip(), ArrivedOn::NothingCouldSay, at.port()).is_empty());
        assert!(
            started.elapsed() < super::WHILE_LOOKING,
            "a question was asked and waited on"
        );
        assert!(
            asked.join().unwrap().is_err(),
            "a machine measured nowhere was asked anyway"
        );
    }

    /// **What a measurement on one network found is written down on that
    /// network**, so a question to the machine it found is held there in turn
    /// (ADR 0044) — asked here on loopback, which is the one network a test on
    /// one machine has.
    #[test]
    fn what_was_measured_on_a_network_is_written_down_on_it() {
        let loopback = crate::route_messages::reported_by_the_kernel()
            .unwrap()
            .into_iter()
            .find(|interface| interface.addresses.contains(&Ipv4Addr::LOCALHOST))
            .unwrap();
        let held_to = std::num::NonZeroU32::new(loopback.index).unwrap();

        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = socket.local_addr().unwrap();
        let answering = Answering::on(socket, Presence::of(reception(), 7_610));
        let answered = std::thread::spawn(move || answering.answer_one());

        let found = found_at(at.ip(), ArrivedOn::TheNetwork(held_to), at.port());
        assert!(answered.join().unwrap().unwrap().is_some());
        let one = found.iter().find(|one| one.machine == reception()).unwrap();
        assert_eq!(one.address, at.ip());
        assert_eq!(one.address.interface(), Some(loopback.index));
    }

    /// **A machine that does not answer is not found**, and nothing is
    /// invented in its place.
    #[test]
    fn a_machine_that_does_not_answer_is_not_found() {
        let quiet = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = quiet.local_addr().unwrap();
        assert!(found_at(at.ip(), ArrivedOn::ItsOwnNetwork, at.port()).is_empty());
    }

    /// **A machine looked for by its identity is found only if it is the one
    /// that answered**: the machine that is there is found at the address it
    /// answered from, and another identity on the same link is not.
    #[test]
    fn a_machine_looked_for_by_name_is_found_only_if_it_is_the_one_that_answered() {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = socket.local_addr().unwrap();
        let answering = Answering::on(socket, Presence::of(reception(), 7_610));
        let answered =
            std::thread::spawn(move || (0..2).map(|_| answering.answer_one()).collect::<Vec<_>>());

        let found = super::found_by_name(&reception(), at).unwrap();
        assert_eq!(found.machine, reception());
        assert_eq!(found.where_it_answers().ip(), at.ip());
        let somebody_else = MachineId::read("99998888777766665555444433332222").unwrap();
        assert!(super::found_by_name(&somebody_else, at).is_none());
        drop(answered.join().unwrap());
    }
}
