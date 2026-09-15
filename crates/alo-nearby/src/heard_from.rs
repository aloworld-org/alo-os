//! The address an answer came from, and — where that address is link-local —
//! the network it came from.
//!
//! *Machines find each other with zero configuration*, and a network nobody
//! configured often has no IPv4 address on it at all: two machines joined by one
//! cable with no DHCP server between them, or an office whose router is down for
//! the afternoon. Each interface still gives itself an IPv6 **link-local**
//! address (`fe80::/10`), and discovery is asked and answered on those
//! (RFC 6762 §3's `ff02::fb`).
//!
//! # A link-local address without its interface names no network
//!
//! Every interface on a machine has an address in `fe80::/10`, so `fe80::1`
//! is an address on *some* link and says nothing about which. The kernel keeps
//! the answer beside the address as a **scope** — the index of the interface
//! the datagram arrived on — and a link-local address can only be dialled with
//! it. So [`HeardFrom`] is the measured address and, for a link-local one, the
//! interface it was heard on: [`HeardFrom::at`] is where a machine answers,
//! scope and all, and a link-local address that arrived with no scope is
//! refused where it is read ([`crate::reading::a_machine_heard`]) rather than
//! written down as something nothing could reach.
//!
//! The scope is **measured**, never advertised, for
//! [`Found::address`](crate::Found::address)'s reason: an advertisement carries
//! no address, and the interface a machine was heard on is a fact about this
//! machine's own networks, not something another machine could say.
//!
//! An IPv4 address, and an IPv6 address that is not link-local, names its
//! network by itself; a scope arriving beside one is not kept, so two
//! measurements of the same global address are the same address. An
//! IPv4-mapped IPv6 address (`::ffff:192.0.2.1`, how a dual-stack socket reports
//! an IPv4 peer) is read as the IPv4 address it is.

use std::fmt;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6};

/// An address an answer came from, with the interface it was heard on where
/// the address is link-local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HeardFrom {
    /// The address.
    ip: IpAddr,
    /// The kernel's index for the interface a link-local address was heard on,
    /// and zero for an address that names its own network.
    scope: u32,
}

impl HeardFrom {
    /// Where a datagram or a connection came from, as the kernel reported it:
    /// the scope kept for a link-local IPv6 address, and dropped for everything
    /// else.
    #[must_use]
    pub fn of(from: SocketAddr) -> Self {
        match from {
            SocketAddr::V4(v4) => Self::named(IpAddr::V4(*v4.ip())),
            SocketAddr::V6(v6) => match v6.ip().to_ipv4_mapped() {
                Some(v4) => Self::named(IpAddr::V4(v4)),
                None if is_link_local(v6.ip()) => Self {
                    ip: IpAddr::V6(*v6.ip()),
                    scope: v6.scope_id(),
                },
                None => Self::named(IpAddr::V6(*v6.ip())),
            },
        }
    }

    /// An address with no interface beside it.
    ///
    /// Right for an IPv4 address and a global IPv6 one; a link-local address
    /// made this way names no network ([`names_a_network`](Self::names_a_network)
    /// says so), which is why nothing read off a network is made through here.
    #[must_use]
    pub const fn named(ip: IpAddr) -> Self {
        Self { ip, scope: 0 }
    }

    /// The address.
    #[must_use]
    pub const fn ip(&self) -> IpAddr {
        self.ip
    }

    /// The kernel's index for the interface a link-local address was heard on,
    /// or `None` for an address that needs none — or a link-local one that
    /// arrived without one.
    #[must_use]
    pub const fn scope(&self) -> Option<u32> {
        if self.scope == 0 {
            None
        } else {
            Some(self.scope)
        }
    }

    /// Whether this is an IPv6 link-local address.
    #[must_use]
    pub const fn is_link_local(&self) -> bool {
        match self.ip {
            IpAddr::V4(_) => false,
            IpAddr::V6(v6) => is_link_local(&v6),
        }
    }

    /// Whether this address says which network it is on: always, except for a
    /// link-local address with no interface beside it, which could be on any
    /// of a machine's links and can be dialled on none.
    #[must_use]
    pub const fn names_a_network(&self) -> bool {
        !self.is_link_local() || self.scope != 0
    }

    /// Where something at this address answers on `port`, with the interface
    /// for a link-local address — which is what makes it dialable.
    #[must_use]
    pub const fn at(&self, port: u16) -> SocketAddr {
        match self.ip {
            IpAddr::V4(v4) => SocketAddr::new(IpAddr::V4(v4), port),
            IpAddr::V6(v6) => SocketAddr::V6(SocketAddrV6::new(v6, port, 0, self.scope)),
        }
    }
}

impl From<IpAddr> for HeardFrom {
    /// As [`HeardFrom::named`].
    fn from(ip: IpAddr) -> Self {
        Self::named(ip)
    }
}

impl PartialEq<IpAddr> for HeardFrom {
    /// The same address, whichever interface it was heard on — for a caller
    /// holding only an address, such as the source of a connection.
    fn eq(&self, other: &IpAddr) -> bool {
        self.ip == *other
    }
}

impl fmt::Display for HeardFrom {
    /// The address, and `%` and the interface's index for a link-local one — the
    /// spelling RFC 4007 §11 gives a scoped address.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.scope() {
            Some(scope) => write!(formatter, "{}%{scope}", self.ip),
            None => write!(formatter, "{}", self.ip),
        }
    }
}

/// Whether `ip` is in `fe80::/10`.
const fn is_link_local(ip: &Ipv6Addr) -> bool {
    ip.segments()[0] & 0xffc0 == 0xfe80
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6};

    use super::HeardFrom;

    /// The studio's link-local address on the cable.
    const LINK_LOCAL: Ipv6Addr = Ipv6Addr::new(0xfe80, 0, 0, 0, 0x30a1, 0x6ff, 0xfe98, 0xb3c5);

    /// **A link-local address keeps the interface it was heard on**, and where
    /// it answers is dialable: the scope is in the socket address.
    #[test]
    fn a_link_local_address_keeps_the_interface_it_was_heard_on() {
        let heard = HeardFrom::of(SocketAddr::V6(SocketAddrV6::new(LINK_LOCAL, 5_353, 0, 3)));
        assert_eq!(heard.ip(), IpAddr::V6(LINK_LOCAL));
        assert_eq!(heard.scope(), Some(3));
        assert!(heard.is_link_local());
        assert!(heard.names_a_network());
        let at = heard.at(7_610);
        assert_eq!(
            at,
            SocketAddr::V6(SocketAddrV6::new(LINK_LOCAL, 7_610, 0, 3))
        );
        assert_eq!(at.to_string(), "[fe80::30a1:6ff:fe98:b3c5%3]:7610");
        assert_eq!(heard.to_string(), "fe80::30a1:6ff:fe98:b3c5%3");
    }

    /// **A link-local address with no interface names no network**, and is
    /// said to name none — nothing could dial it.
    #[test]
    fn a_link_local_address_with_no_interface_names_no_network() {
        let unscoped = HeardFrom::of(SocketAddr::V6(SocketAddrV6::new(LINK_LOCAL, 5_353, 0, 0)));
        assert!(unscoped.is_link_local());
        assert_eq!(unscoped.scope(), None);
        assert!(!unscoped.names_a_network());
        assert!(!HeardFrom::named(IpAddr::V6(LINK_LOCAL)).names_a_network());
    }

    /// **An address that names its own network keeps no scope**, so two
    /// measurements of one address are one address; and an IPv4 peer reported
    /// by a dual-stack socket is its IPv4 address.
    #[test]
    fn an_address_that_names_its_own_network_keeps_no_scope() {
        let global: Ipv6Addr = "2001:db8::7".parse().unwrap();
        let heard = HeardFrom::of(SocketAddr::V6(SocketAddrV6::new(global, 1, 0, 4)));
        assert_eq!(heard.scope(), None);
        assert_eq!(heard, HeardFrom::named(IpAddr::V6(global)));
        assert!(heard.names_a_network());

        let v4 = Ipv4Addr::new(10, 61, 1, 2);
        let mapped = HeardFrom::of(SocketAddr::V6(SocketAddrV6::new(
            v4.to_ipv6_mapped(),
            7_610,
            0,
            0,
        )));
        assert_eq!(mapped, HeardFrom::named(IpAddr::V4(v4)));
        assert_eq!(mapped.at(7_610).to_string(), "10.61.1.2:7610");
        assert!(!mapped.is_link_local());
        assert_eq!(mapped, IpAddr::V4(v4));
    }
}
