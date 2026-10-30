//! One network interface, as the kernel reports it.
//!
//! The flags are the kernel's own `IFF_*` values, named rather than written as
//! numbers wherever they are tested, so a reader can check them against
//! `<linux/if.h>`.

use std::net::{Ipv4Addr, Ipv6Addr};

/// `IFF_UP`: the interface is administratively up.
pub const IFF_UP: u32 = 0x1;
/// `IFF_LOOPBACK`: the interface is loopback.
pub const IFF_LOOPBACK: u32 = 0x8;
/// `IFF_RUNNING`: the interface is operationally up.
pub const IFF_RUNNING: u32 = 0x40;
/// `IFF_MULTICAST`: the interface carries multicast.
pub const IFF_MULTICAST: u32 = 0x1000;

/// One interface, as the kernel reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    /// The kernel's index for it, which is how an address names its interface.
    pub index: u32,
    /// Its name, for a line in the service log.
    pub name: String,
    /// The kernel's `IFF_*` flags for it.
    pub flags: u32,
    /// Its IPv4 addresses, in the order the kernel listed them.
    pub addresses: Vec<Ipv4Addr>,
    /// Its IPv6 addresses that can be used now — not ones the kernel is still
    /// checking nobody else holds, nor ones that check failed — in the order
    /// the kernel listed them.
    pub ipv6: Vec<Ipv6Addr>,
}
