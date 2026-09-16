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
//! A network that **goes** needs nothing done: the kernel drops a membership
//! with the interface it was on, and forgetting it here means the same interface
//! coming back — a cable pulled and plugged in again, an interface recreated with
//! a new index — is joined again rather than assumed joined.
//!
//! A routing socket that will not open at start is a line in the service log and
//! a machine joined on the networks it had at start — found where it was, and
//! told so, rather than a stopped service.
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
        joining.follow(said);
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
    /// interfaces again, and join every network not yet joined.
    pub fn changed(&self, said: &mut dyn FnMut(&str)) {
        if let Some(told) = &self.told
            && let Err(why) = crate::unix::emptied(told.as_fd())
        {
            said(&format!(
                "what the kernel said about a network change could not be read ({why}); the interfaces are asked again anyway"
            ));
        }
        self.follow(said);
    }

    /// Ask the interfaces, forget networks that went, and join the ones that
    /// are new.
    fn follow(&self, said: &mut dyn FnMut(&str)) {
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
        joined.retain(|network| now.contains(network));
        let new = now
            .into_iter()
            .filter(|network| !joined.contains(network))
            .collect();
        let newly = joined_on(new, |network| join(discovery, network), said);
        joined.extend(newly);
    }
}

/// Join `discovery` at `ff02::fb` on `network`'s interface.
///
/// Already joined is joined: an interface the kernel kept the membership on
/// across a change this file did not see is not a network that failed.
fn join(discovery: &UdpSocket, network: &Network) -> Result<(), std::io::Error> {
    match discovery.join_multicast_v6(&THE_IPV6_ADDRESS, network.index()) {
        Err(why) if why.kind() == std::io::ErrorKind::AddrInUse => Ok(()),
        joined => joined,
    }
}
