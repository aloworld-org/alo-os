//! Which of this machine's networks a connection arrived on, read off the
//! connection itself.
//!
//! *Machines find each other with zero configuration, and trust none of them
//! for it.* A proposal is judged against what discovery on this machine has
//! measured about the machine that sent it, and the measurement is made at the
//! moment the proposal arrives (`crate::looking::found_at`): the machine at the
//! connection's source address is asked who it is. Since
//! [ADR 0044](../../../docs/decisions/0044-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md)
//! a machine this one **dials** at a private IPv4 address is held to the network
//! it was found on — but a connection that **arrives** from `192.168.1.20` was,
//! until this file, measured from a socket held to nothing, so the question
//! *who are you* left by whatever the route said. On a machine on two networks
//! whose routers hand out the same range that is the wrong machine: the proposal
//! from the studio on the cable measured against somebody else on the Wi-Fi.
//!
//! So the network a connection arrived on is read here, and the measurement is
//! held to it ([`ArrivedOn`]).
//!
//! # How it is read, and what that costs
//!
//! Three readings were available, and this is the one built:
//!
//! - **The interface that owns the accepting socket's local address**, which is
//!   what this file does. `getsockname` on the accepted connection is the
//!   address on **this** machine the other machine connected to, the kernel
//!   measured it, and it is there for every connection with no option set
//!   beforehand and no change to how a message is read. What it costs is the
//!   case where this machine's *own* address is the same on both networks —
//!   `10.65.0.1` on the cable and on the Wi-Fi, which is what the fixture for
//!   ADR 0044 deliberately built: then two interfaces own it, the reading is
//!   ambiguous, and [`ArrivedOn::NothingCouldSay`] is the honest answer. A
//!   proposal is then measured nowhere and refused as *not found* — never
//!   measured by the route, which is the failure this file exists to stop.
//! - **`IP_PKTINFO` read back from the accepted socket**, which would name the
//!   interface even where the address is shared. It costs the option being set
//!   on the listener before the connection is accepted and the interface being
//!   read with `recvmsg` — and `crate::wire` reads every message once, with
//!   `read`, through the framing both wires share. On TCP the control message
//!   describes the segment that last arrived rather than the connection, so the
//!   reading would have to be taken from the first `recvmsg` of the request and
//!   carried alongside it: a second way to read a message, in the one file
//!   whose single reader is the reason the wires can share a port. It buys the
//!   shared-address case, which this machine refuses rather than guesses, and
//!   it is a change to reading a message that a later task can make on its own
//!   merits.
//! - **The route to the source address**, which is what the code did before and
//!   is exactly the thing ADR 0044 refused: the route is not what the person was
//!   shown, and it changes underneath an open connection.
//!
//! # What the kernel does, measured — and why the reading is exact today
//!
//! `docs/quirks.md` carries both measurements. A namespace with `10.66.0.1` on
//! one cable and `10.66.0.3` on another, the same address `10.66.0.2` at each
//! far end:
//!
//! - A machine on **either** network can open a connection to **either** of
//!   this machine's addresses, and the accepted socket's local address is the
//!   address that was dialled, not the one that belongs to the interface the
//!   packet came in on. That is Linux's weak host model, and on its own it
//!   would let a machine on the other network be measured on this one.
//! - But an **unheld listener answers every handshake by the route**: a
//!   connection from the network the route does not point at never completes,
//!   to either address. Measured, in both directions.
//!
//! The two together are why the reading here is exact rather than
//! approximate: while `crate::wire` listens on one socket held to nothing,
//! **the only connections that complete are the ones that arrived on the
//! route's network**, and the address they were made to belongs to an
//! interface of this machine. What that costs is written down for the task
//! that gives the wire a held listener per network: once a connection can
//! arrive on a network the route does not point at, the interface must come
//! from the listener that accepted it, not from the address it was made to.
//!
//! # Loopback is one network and needs no holding
//!
//! ADR 0020 leaves loopback the only unchecked destination, and a connection
//! that arrived at `127.0.0.1` arrived from this machine. There is one loopback
//! network, nothing can be confused with it, and a question back to `127.0.0.1`
//! reaches the machine that asked. So loopback is [`ArrivedOn::ItsOwnNetwork`]
//! rather than an interface to hold to.
//!
//! # And in the other family
//!
//! An IPv6 address carries its own network already: a link-local one arrives
//! with the interface it was heard on beside it (ADR 0041,
//! [`alo_nearby::HeardFrom`]), and a global one names its network in the
//! address. Neither needs this reading, and both are
//! [`ArrivedOn::ItsOwnNetwork`]. An IPv4 peer a dual-stack listener reports as
//! `::ffff:10.65.0.2` is read as the IPv4 connection it is.
//!
//! # Nothing is kept
//!
//! The interfaces are read from the kernel at the moment the connection is
//! judged, for `crate::looking`'s reason: a list kept would age, and an
//! interface that went away since would be held to by a question that could
//! never leave.

use std::net::{IpAddr, SocketAddr, TcpStream};
use std::num::NonZeroU32;

use crate::networks::Interface;
use crate::route_messages::reported_by_the_kernel;

/// Which network a connection arrived on, as far as this machine can read it.
///
/// What a measurement about the machine at the other end is held to
/// (`crate::looking::found_at`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArrivedOn {
    /// The network behind the interface the kernel numbers this: a question to
    /// the machine that connected leaves from a socket held to it, and what
    /// answers is written down on it
    /// ([`alo_nearby::HeardFrom::on_the_network`]).
    TheNetwork(NonZeroU32),
    /// The address the connection came from names its own network — an IPv6
    /// address, or loopback — and a question to it needs no interface beside
    /// it.
    ItsOwnNetwork,
    /// Which network it arrived on could not be read: this machine's address on
    /// it is on two interfaces at once, on none the kernel reports, or the
    /// kernel would not say. Nothing is measured, and what needed the
    /// measurement is refused.
    NothingCouldSay,
}

/// The network `connection` arrived on, read from the address on this machine
/// it was made to and the interfaces the kernel reports at this moment.
///
/// [`ArrivedOn::NothingCouldSay`] when the kernel will not say where the
/// connection ends or what this machine's interfaces are — both of which are
/// *nothing could be read*, which is refused rather than guessed.
#[must_use]
pub fn the_network_it_arrived_on(connection: &TcpStream) -> ArrivedOn {
    let Ok(here) = connection.local_addr() else {
        return ArrivedOn::NothingCouldSay;
    };
    let Ok(reported) = reported_by_the_kernel() else {
        return ArrivedOn::NothingCouldSay;
    };
    the_network_of(here, &reported)
}

/// The network a connection that arrived at `here` on this machine is on, out
/// of the interfaces `reported`.
///
/// The rule with no kernel in it, so each way of reading it is a test: an IPv6
/// or loopback address names its own network; an IPv4 address owned by exactly
/// one reported interface is that interface's network; and an address two
/// interfaces own, or none does, is [`ArrivedOn::NothingCouldSay`].
#[must_use]
pub fn the_network_of(here: SocketAddr, reported: &[Interface]) -> ArrivedOn {
    let here = match here.ip() {
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => v4,
            None => return ArrivedOn::ItsOwnNetwork,
        },
        IpAddr::V4(v4) => v4,
    };
    if here.is_loopback() {
        return ArrivedOn::ItsOwnNetwork;
    }
    let mut owning = reported
        .iter()
        .filter(|interface| interface.addresses.contains(&here));
    let Some(one) = owning.next() else {
        return ArrivedOn::NothingCouldSay;
    };
    if owning.next().is_some() {
        return ArrivedOn::NothingCouldSay;
    }
    NonZeroU32::new(one.index).map_or(ArrivedOn::NothingCouldSay, ArrivedOn::TheNetwork)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6, TcpListener, TcpStream};
    use std::num::NonZeroU32;

    use super::{ArrivedOn, the_network_it_arrived_on, the_network_of};
    use crate::networks::{IFF_MULTICAST, IFF_RUNNING, IFF_UP, Interface};

    /// This machine's address on the cable.
    const ON_THE_CABLE: Ipv4Addr = Ipv4Addr::new(10, 65, 0, 1);
    /// This machine's address on the other network, which carries the same
    /// private range.
    const ON_THE_OTHER_NETWORK: Ipv4Addr = Ipv4Addr::new(10, 65, 0, 3);

    /// An interface the kernel reports, up and carrying multicast.
    fn interface(index: u32, name: &str, addresses: &[Ipv4Addr]) -> Interface {
        Interface {
            index,
            name: name.to_owned(),
            flags: IFF_UP | IFF_RUNNING | IFF_MULTICAST,
            addresses: addresses.to_vec(),
            ipv6: Vec::new(),
        }
    }

    /// Two networks with an address of this machine's on each.
    fn two_networks() -> Vec<Interface> {
        vec![
            interface(1, "lo", &[Ipv4Addr::LOCALHOST]),
            interface(3, "cable0", &[ON_THE_CABLE]),
            interface(4, "other0", &[ON_THE_OTHER_NETWORK]),
        ]
    }

    /// Where a connection arrived, as a socket address on this machine.
    fn arrived_at(here: Ipv4Addr) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(here), 7_610)
    }

    /// **A connection is on the network of the interface that owns the address
    /// it arrived at**, and the two networks are told apart.
    #[test]
    fn a_connection_is_on_the_network_of_the_address_it_arrived_at() {
        let reported = two_networks();
        assert_eq!(
            the_network_of(arrived_at(ON_THE_CABLE), &reported),
            ArrivedOn::TheNetwork(NonZeroU32::new(3).unwrap())
        );
        assert_eq!(
            the_network_of(arrived_at(ON_THE_OTHER_NETWORK), &reported),
            ArrivedOn::TheNetwork(NonZeroU32::new(4).unwrap())
        );
    }

    /// **An address on two interfaces at once says nothing**: this machine has
    /// the same address on both networks, the reading is ambiguous, and it is
    /// refused rather than resolved by the route.
    #[test]
    fn an_address_on_two_interfaces_says_nothing() {
        let both = vec![
            interface(3, "cable0", &[ON_THE_CABLE]),
            interface(4, "other0", &[ON_THE_CABLE]),
        ];
        assert_eq!(
            the_network_of(arrived_at(ON_THE_CABLE), &both),
            ArrivedOn::NothingCouldSay
        );
    }

    /// **An address no interface owns says nothing**, and so does an address on
    /// an interface the kernel numbers zero, which is no interface.
    #[test]
    fn an_address_no_interface_owns_says_nothing() {
        assert_eq!(
            the_network_of(arrived_at(Ipv4Addr::new(10, 65, 9, 9)), &two_networks()),
            ArrivedOn::NothingCouldSay
        );
        assert_eq!(
            the_network_of(
                arrived_at(ON_THE_CABLE),
                &[interface(0, "nowhere", &[ON_THE_CABLE])]
            ),
            ArrivedOn::NothingCouldSay
        );
        assert_eq!(
            the_network_of(arrived_at(ON_THE_CABLE), &[]),
            ArrivedOn::NothingCouldSay
        );
    }

    /// **Loopback is its own network**, and needs no interface held to: a
    /// connection that arrived there arrived from this machine.
    #[test]
    fn loopback_is_its_own_network() {
        assert_eq!(
            the_network_of(arrived_at(Ipv4Addr::LOCALHOST), &two_networks()),
            ArrivedOn::ItsOwnNetwork
        );
        // And with no interface reported at all, which is what a machine that
        // could say nothing about its networks looks like.
        assert_eq!(
            the_network_of(arrived_at(Ipv4Addr::LOCALHOST), &[]),
            ArrivedOn::ItsOwnNetwork
        );
    }

    /// **An IPv6 address names its own network**, in either scope — and an IPv4
    /// peer a dual-stack listener reports as an IPv4-mapped address is read as
    /// the IPv4 connection it is.
    #[test]
    fn an_ipv6_address_names_its_own_network_and_a_mapped_one_does_not() {
        let link_local: Ipv6Addr = "fe80::30a1:6ff:fe98:b3c5".parse().unwrap();
        assert_eq!(
            the_network_of(
                SocketAddr::V6(SocketAddrV6::new(link_local, 7_610, 0, 3)),
                &two_networks()
            ),
            ArrivedOn::ItsOwnNetwork
        );
        let global: Ipv6Addr = "2001:db8::1".parse().unwrap();
        assert_eq!(
            the_network_of(SocketAddr::new(global.into(), 7_610), &two_networks()),
            ArrivedOn::ItsOwnNetwork
        );
        assert_eq!(
            the_network_of(
                SocketAddr::V6(SocketAddrV6::new(
                    ON_THE_CABLE.to_ipv6_mapped(),
                    7_610,
                    0,
                    0
                )),
                &two_networks()
            ),
            ArrivedOn::TheNetwork(NonZeroU32::new(3).unwrap())
        );
    }

    /// **A real connection on loopback reads as its own network**, which is
    /// what every connection in a test on one machine is.
    #[test]
    fn a_real_loopback_connection_is_its_own_network() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = listener.local_addr().unwrap();
        let knocking = std::thread::spawn(move || TcpStream::connect(at).unwrap());
        let (accepted, _) = listener.accept().unwrap();
        assert_eq!(
            the_network_it_arrived_on(&accepted),
            ArrivedOn::ItsOwnNetwork
        );
        drop(knocking.join().unwrap());
    }
}
