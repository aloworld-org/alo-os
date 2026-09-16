//! The IPv6 discovery group, joined on every link-local network this machine is
//! on — and kept joined as networks come and go, for every socket that follows
//! the kernel.
//!
//! `crate::networks` decides which interfaces discovery is joined on out of what
//! the kernel reports; this is the socket actually joined on them, at start and
//! afterwards. **The IPv4 groups are not here**: since `crate::responding` the
//! machine answers IPv4 discovery on one socket per network, each held to that
//! network's interface, and each joins the group on its own network — a join that
//! belongs to the socket that is held there. What stays here is the IPv6 socket,
//! which is held to nothing, and the kernel's notification that a network
//! changed, which `crate::wire` hands to the responders and the listeners as
//! well.
//!
//! # A network that appears after start is followed, not waited for
//!
//! A laptop is started on the Wi-Fi at home and docked at the office an hour
//! later. A daemon that read its interfaces once at start would be found on the
//! Wi-Fi all day and silently absent from the wired network it was plugged into —
//! the very failure a machine on two networks has — until somebody restarted a
//! service they have never heard of. So the kernel's own notification is
//! followed: [`Joining`] holds a routing socket subscribed to links and their
//! addresses (`crate::unix::told_when_networks_change`), the service waits on it
//! beside everything else, and when it speaks the interfaces are asked again and
//! every network not yet joined is joined. Nothing wakes on an interval: a
//! machine whose networks do not change costs nothing for following them.
//!
//! # A network that goes is left, and a membership is never assumed
//!
//! A network that **goes** is left, and not only forgotten. Measured on this
//! kernel (`docs/quirks.md`): a link set down keeps its interface in the group,
//! but a link **deleted** — a cable re-laid, a USB adapter pulled, the far end's
//! namespace ending — takes the interface's membership with it and leaves the
//! **socket's** behind, at an interface number nothing has. A second join at that
//! number is refused `EADDRINUSE` before the kernel looks for the interface at
//! all. So a socket that only forgot would, on an interface given that number
//! again, count itself joined while no interface was in the group — a machine
//! nobody on that cable ever finds — and would carry one more dead membership
//! for every cable pulled.
//!
//! For the same reason a join refused `EADDRINUSE` is **taken afresh** — left
//! and joined again — rather than read as *already joined*: this file joins only
//! networks it does not hold, so a membership the socket holds there anyway is
//! one it did not see go, and whether an interface is behind it is exactly what
//! the refusal does not say. The two halves overlap on purpose: leaving is what
//! keeps dead memberships from piling up, and taking a join afresh is what
//! covers a cable deleted and re-laid between two readings of the interfaces,
//! where nothing was seen to go and so nothing was left.
//! `crate::two_machines_with_no_ipv4_find_each_other_again` is the measurement:
//! with both halves removed, a cable re-laid at the numbers it first had is
//! never found; with either one alone, it is.
//!
//! A routing socket that will not open at start is a line in the service log and
//! a machine joined on the networks it had at start — found where it was, and
//! told so, rather than a stopped service.
//!
//! # And an interface the kernel says went is left, even where its number is back
//!
//! A cable deleted and laid again at the same number **between two readings of
//! the interfaces** is, in both readings, the same network — often down to its
//! link-local address, where the new interface has the old one's hardware
//! address. So what the kernel said in the round is read as well
//! ([`crate::interfaces_that_went`]): a network whose interface it said was
//! deleted is left and joined afresh whether or not it is reported again, which
//! is the same rule `crate::responding` follows for the IPv4 group.
//!
//! # Over IPv6, on that notification
//!
//! Where the machine could open an IPv6 discovery socket, every IPv6 link-local
//! network (`crate::networks::link_local_networks`) is joined at `ff02::fb` on
//! its interface, by that following: a link-local address is usable a second or
//! two after its link comes up, and the kernel says so on the same routing
//! socket, subscribed to IPv6 addresses too. Where it could not — a kernel built
//! without IPv6 — nothing is joined at all rather than failing to join once per
//! notification, and the routing socket is still followed for everything else
//! that reads it.

use std::net::UdpSocket;
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::sync::{Mutex, PoisonError};

use alo_nearby::THE_IPV6_ADDRESS;

use crate::interfaces_that_went::Went;
use crate::networks::{Network, joined_on, link_local_networks};
use crate::route_messages::reported_by_the_kernel;

/// The IPv6 discovery group, joined on every link-local network this machine is
/// on.
#[derive(Debug)]
pub struct Joining {
    /// A handle onto the IPv6 socket discovery is answered on, where this
    /// machine could open one — a second handle onto the one `crate::responding`
    /// answers with, so that a join made here is a join on that socket.
    discovery: Option<UdpSocket>,
    /// The routing socket the kernel says a network changed on, when it opened.
    told: Option<OwnedFd>,
    /// The networks the group is joined on now.
    joined: Mutex<Vec<Network>>,
}

impl Joining {
    /// Join `discovery`, where there is one, to the group on every link-local
    /// network this machine is on now, and listen for the kernel saying that
    /// changed.
    ///
    /// The routing socket is opened whether or not there is a socket to join
    /// with: it is what `crate::wire` hands to the responders and the listeners
    /// as well, and a machine with no IPv6 in its kernel still follows its
    /// networks.
    ///
    /// Every failure on the way — the kernel's interfaces unreadable, one
    /// network refusing the join, the routing socket refusing to open — is a
    /// line handed to `said` and never a refusal to start.
    pub fn on_every_network(discovery: Option<&UdpSocket>, said: &mut dyn FnMut(&str)) -> Self {
        let told = match crate::unix::told_when_networks_change() {
            Ok(told) => Some(told),
            Err(why) => {
                said(&format!(
                    "the kernel will not say when a network changes ({why}); discovery is joined on the networks this machine is on now, and a network that appears later is not joined until the service starts again"
                ));
                None
            }
        };
        let discovery = match discovery.map(UdpSocket::try_clone) {
            Some(Ok(socket)) => Some(socket),
            None => None,
            Some(Err(why)) => {
                said(&format!(
                    "the socket discovery is answered on over IPv6 could not be taken a second time to join a group on ({why}); this machine is not found on a network with no IPv4 address"
                ));
                None
            }
        };
        let joining = Self {
            discovery,
            told,
            joined: Mutex::new(Vec::new()),
        };
        joining.follow(&Went::nothing(), said);
        joining
    }

    /// What to wait on for the kernel saying a network changed, when it can.
    #[must_use]
    pub fn waiting_on(&self) -> Option<BorrowedFd<'_>> {
        self.told.as_ref().map(AsFd::as_fd)
    }

    /// The link-local networks the group is joined on now, in the kernel's
    /// order.
    #[must_use]
    pub fn joined(&self) -> Vec<Network> {
        self.joined
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The kernel said a network changed: read what it said, ask the
    /// interfaces again, and join every network not yet joined — and hand back
    /// the interfaces it said went, which the responders follow too.
    ///
    /// What could not be read is [`Went::any_of_them`]: which interfaces went
    /// is then not known, and nothing is counted joined on that guess.
    pub fn changed(&self, said: &mut dyn FnMut(&str)) -> Went {
        let mut went = Went::nothing();
        if let Some(told) = &self.told {
            let emptied = crate::unix::emptied(told.as_fd(), &mut |datagram| match datagram {
                Some(datagram) => went.heard(datagram),
                None => went.lost(),
            });
            if let Err(why) = emptied {
                said(&format!(
                    "what the kernel said about a network change could not be read ({why}); the interfaces are asked again anyway, and every membership is taken afresh"
                ));
                went.lost();
            }
        }
        self.follow(&went, said);
        went
    }

    /// Ask the interfaces, leave networks that went, and join the ones that
    /// are new.
    fn follow(&self, went: &Went, said: &mut dyn FnMut(&str)) {
        let reported = match reported_by_the_kernel() {
            Ok(reported) => reported,
            Err(why) => {
                said(&format!(
                    "this machine's networks could not be read ({why}); discovery stays joined where it was"
                ));
                return;
            }
        };
        let Some(discovery) = &self.discovery else {
            return;
        };
        let now = link_local_networks(&reported);
        let mut joined = self.joined.lock().unwrap_or_else(PoisonError::into_inner);
        let (kept, gone) = kept_and_gone(joined.drain(..), &now, went);
        *joined = kept;
        for network in &gone {
            left(discovery, network);
        }
        let new = now
            .into_iter()
            .filter(|network| !joined.contains(network))
            .collect();
        let newly = joined_on(new, |network| join(discovery, network), said);
        joined.extend(newly);
    }
}

/// Which of the networks `joined` stay joined, and which are left: a network
/// stays only where it is reported `now` and the kernel did not say its
/// interface `went`.
fn kept_and_gone(
    joined: impl Iterator<Item = Network>,
    now: &[Network],
    went: &Went,
) -> (Vec<Network>, Vec<Network>) {
    joined.partition(|network| now.contains(network) && !went.includes(network.index()))
}

/// Join `discovery` at `ff02::fb` on `network`'s interface.
///
/// A membership the socket already holds there is **taken afresh** — left, and
/// joined again — because this is called only for a network this file does not
/// hold, and such a membership may be one a deleted interface with the same
/// number left behind, with no interface in the group behind it. Where the
/// interface did keep it, leaving and joining again costs one report on the
/// link.
///
/// # Errors
/// What the kernel answered the join with — the second one, where there were
/// two: an interface that went since the networks were read, and everything a
/// join can fail with.
fn join(discovery: &UdpSocket, network: &Network) -> Result<(), std::io::Error> {
    match discovery.join_multicast_v6(&THE_IPV6_ADDRESS, network.index()) {
        Err(why) if why.kind() == std::io::ErrorKind::AddrInUse => {
            left(discovery, network);
            discovery.join_multicast_v6(&THE_IPV6_ADDRESS, network.index())
        }
        joined => joined,
    }
}

/// Leave `ff02::fb` on `network`'s interface, whether or not the interface is
/// still there.
///
/// Nothing is said when the kernel refuses: its one refusal is that the socket
/// held no membership there, and then there was nothing to leave.
fn left(discovery: &UdpSocket, network: &Network) {
    drop(discovery.leave_multicast_v6(&THE_IPV6_ADDRESS, network.index()));
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr, UdpSocket};

    use alo_nearby::THE_IPV6_ADDRESS;

    use super::{join, kept_and_gone, left};
    use crate::interfaces_that_went::Went;
    use crate::networks::{
        IFF_MULTICAST, IFF_RUNNING, IFF_UP, Interface, Network, link_local_networks,
    };
    use crate::route_messages::reported_by_the_kernel;

    /// A network numbered `index`, as `crate::networks` reads one.
    fn numbered(index: u32) -> Network {
        link_local_networks(&[Interface {
            index,
            name: format!("if{index}"),
            flags: IFF_UP | IFF_RUNNING | IFF_MULTICAST,
            addresses: Vec::new(),
            ipv6: vec!["fe80::1".parse().unwrap()],
        }])
        .into_iter()
        .next()
        .unwrap()
    }

    /// This machine's loopback interface as a network a socket can be joined
    /// on — the one interface every test host has.
    fn loopback() -> Network {
        numbered(
            reported_by_the_kernel()
                .unwrap()
                .into_iter()
                .find(|interface| interface.addresses.contains(&Ipv4Addr::LOCALHOST))
                .unwrap()
                .index,
        )
    }

    /// Whether the interface numbered `index` is in `ff02::fb`, as
    /// `/proc/net/igmp6` lists it.
    fn in_the_group(index: u32) -> bool {
        std::fs::read_to_string("/proc/net/igmp6")
            .unwrap()
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .any(|words| {
                words.first() == Some(&index.to_string().as_str())
                    && words.get(2) == Some(&"ff0200000000000000000000000000fb")
            })
    }

    /// **A membership the socket already holds is still held after joining again**:
    /// the join succeeds, and the socket and the interface are both still in
    /// the group afterwards.
    #[test]
    fn a_membership_the_socket_already_holds_is_still_held_after_joining_again() {
        let network = loopback();
        let socket = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)).unwrap();
        socket
            .join_multicast_v6(&THE_IPV6_ADDRESS, network.index())
            .unwrap();

        join(&socket, &network).unwrap();
        assert!(in_the_group(network.index()));
        assert_eq!(
            socket
                .join_multicast_v6(&THE_IPV6_ADDRESS, network.index())
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::AddrInUse,
            "a membership taken afresh is not held"
        );
    }

    /// **A network that went is left**: the socket holds no membership there
    /// afterwards, so an interface given that number again is joined for real.
    #[test]
    fn a_network_that_went_is_left() {
        let network = loopback();
        let socket = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)).unwrap();
        join(&socket, &network).unwrap();

        left(&socket, &network);
        socket
            .join_multicast_v6(&THE_IPV6_ADDRESS, network.index())
            .unwrap();
    }

    /// **Leaving a network never joined is nothing**, and joining it afterwards
    /// is an ordinary join.
    #[test]
    fn leaving_a_network_never_joined_is_nothing() {
        let network = loopback();
        let socket = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)).unwrap();
        left(&socket, &network);
        join(&socket, &network).unwrap();
    }

    /// **A network whose interface the kernel said went is left, even where the
    /// same network is reported again** — the interface at that number is a new
    /// one, with nobody in the group — and a network that is not reported is
    /// left as before, while one that neither went nor moved stays joined.
    #[test]
    fn a_network_whose_interface_went_is_left_even_where_it_is_reported_again() {
        let (forty, forty_one, forty_two) = (numbered(40), numbered(41), numbered(42));
        let joined = [forty.clone(), forty_one.clone(), forty_two.clone()];
        let now = [forty.clone(), forty_one.clone()];

        let mut went = Went::nothing();
        let (kept, gone) = kept_and_gone(joined.clone().into_iter(), &now, &went);
        assert_eq!(kept, vec![forty.clone(), forty_one.clone()]);
        assert_eq!(gone, vec![forty_two.clone()]);

        went.lost();
        let (kept, gone) = kept_and_gone(joined.into_iter(), &now, &went);
        assert!(kept.is_empty(), "a round that lost messages kept {kept:?}");
        assert_eq!(gone.len(), 3);
    }

    /// **A join at a number no interface has is refused**, never counted as
    /// joined — the refusal `crate::networks::joined_on` turns into a line in
    /// the service log.
    #[test]
    fn a_join_where_no_interface_is_is_refused() {
        let socket = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)).unwrap();
        assert!(join(&socket, &numbered(999_999)).is_err());
    }
}
