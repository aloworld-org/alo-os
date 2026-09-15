//! The discovery group, joined on every network this machine is on — and kept
//! joined as networks come and go.
//!
//! `crate::networks` decides which interfaces discovery is joined on out of what
//! the kernel reports; this is the socket actually joined on them, at start and
//! afterwards.
//!
//! # A network that appears after start is followed, not waited for
//!
//! A laptop is started on the Wi-Fi at home and docked at the office an hour
//! later. A daemon that read its interfaces once at start would be found on the
//! Wi-Fi all day and silently absent from the wired network it was plugged into —
//! the very failure a machine on two networks has — until somebody restarted a
//! service they have never heard of. So the kernel's own notification is
//! followed: [`Joining`] holds a routing socket subscribed to links and IPv4
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

use std::net::{Ipv4Addr, UdpSocket};
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::sync::{Mutex, PoisonError};

use alo_nearby::THE_ADDRESS;

use crate::networks::{Network, discovery_networks, joined_on};
use crate::route_messages::reported_by_the_kernel;

/// The discovery group, joined on every network this machine is on.
#[derive(Debug)]
pub struct Joining {
    /// The routing socket the kernel says a network changed on, when it opened.
    told: Option<OwnedFd>,
    /// The networks the group is joined on now.
    joined: Mutex<Vec<Network>>,
}

impl Joining {
    /// Join `discovery` to the group on every network this machine is on now,
    /// and listen for the kernel saying that changed.
    ///
    /// Every failure on the way — the kernel's interfaces unreadable, one
    /// network refusing the join, the routing socket refusing to open — is a
    /// line handed to `said` and never a refusal to start.
    pub fn on_every_network(discovery: &UdpSocket, said: &mut dyn FnMut(&str)) -> Self {
        let told = match crate::unix::told_when_networks_change() {
            Ok(told) => Some(told),
            Err(why) => {
                said(&format!(
                    "the kernel will not say when a network changes ({why}); discovery is joined on the networks this machine is on now, and a network that appears later is not joined until the service starts again"
                ));
                None
            }
        };
        let joining = Self {
            told,
            joined: Mutex::new(Vec::new()),
        };
        joining.follow(discovery, said);
        joining
    }

    /// What to wait on for the kernel saying a network changed, when it can.
    #[must_use]
    pub fn waiting_on(&self) -> Option<BorrowedFd<'_>> {
        self.told.as_ref().map(AsFd::as_fd)
    }

    /// The networks the group is joined on now, in the kernel's order.
    #[must_use]
    pub fn joined(&self) -> Vec<Network> {
        self.joined
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The kernel said a network changed: read what it said, ask the
    /// interfaces again, and join every network not yet joined.
    pub fn changed(&self, discovery: &UdpSocket, said: &mut dyn FnMut(&str)) {
        if let Some(told) = &self.told
            && let Err(why) = crate::unix::emptied(told.as_fd())
        {
            said(&format!(
                "what the kernel said about a network change could not be read ({why}); the interfaces are asked again anyway"
            ));
        }
        self.follow(discovery, said);
    }

    /// Ask the interfaces, forget networks that went, and join the ones that
    /// are new.
    fn follow(&self, discovery: &UdpSocket, said: &mut dyn FnMut(&str)) {
        let reported = match reported_by_the_kernel() {
            Ok(reported) => reported,
            Err(why) => {
                said(&format!(
                    "this machine's networks could not be read ({why}); discovery stays joined where it was"
                ));
                return;
            }
        };
        let now = discovery_networks(&reported);
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

/// Join `discovery` to the group on `network`, at its address.
///
/// Already joined is joined: an interface the kernel kept the membership on
/// across a change this file did not see is not a network that failed.
fn join(discovery: &UdpSocket, network: &Network) -> Result<(), std::io::Error> {
    let address: Ipv4Addr = network.address();
    match discovery.join_multicast_v4(&THE_ADDRESS, &address) {
        Err(why) if why.kind() == std::io::ErrorKind::AddrInUse => Ok(()),
        joined => joined,
    }
}
