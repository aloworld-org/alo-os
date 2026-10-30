//! **This machine's own network addresses**, read from the kernel at the moment
//! somebody opens the form that adds a provider.
//!
//! The owner decided on 2026-10-30 that the caution Settings shows for a
//! provider at this machine's address — *alo cannot see where this service
//! sends your questions* — covers the machine's own network addresses as well
//! as loopback: a model server a person runs here and reaches as
//! `http://192.168.1.20:8000` is just as much a program on this machine, and
//! what it does with a question is just as invisible (ADR 0021).
//!
//! # Read once, when asked, never watched
//!
//! The shell asks when a form opens and this answers then. Nothing here holds a
//! socket open or listens for a change: a reading taken while nobody is filling
//! a form in would be a watcher, and an address that changes while somebody
//! types is read again the next time a form opens.
//!
//! # A machine that cannot say is a machine with only loopback
//!
//! If the kernel will not answer, the reason goes to the service log and the
//! answer is empty — so the caution still covers loopback, as it did before,
//! and nothing is invented.

use std::net::IpAddr;

/// Every address this machine's interfaces hold now, loopback left out because
/// the form already knows it.
#[must_use]
pub(crate) fn read_now() -> Vec<IpAddr> {
    match alo_agentd::route_messages::reported_by_the_kernel() {
        Ok(interfaces) => of(&interfaces),
        Err(why) => {
            eprintln!("alo-desktop: this machine's own addresses did not read — {why}");
            Vec::new()
        }
    }
}

/// The addresses `interfaces` hold, in both families, without loopback.
fn of(interfaces: &[alo_agentd::networks::Interface]) -> Vec<IpAddr> {
    interfaces
        .iter()
        .flat_map(|interface| {
            let v4 = interface.addresses.iter().copied().map(IpAddr::V4);
            let v6 = interface.ipv6.iter().copied().map(IpAddr::V6);
            v4.chain(v6)
        })
        .filter(|address| !address.is_loopback())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::*;

    /// One interface holding these addresses.
    fn holding(name: &str, v4: &[Ipv4Addr], v6: &[Ipv6Addr]) -> alo_agentd::networks::Interface {
        alo_agentd::networks::Interface {
            index: 2,
            name: name.to_owned(),
            flags: alo_agentd::networks::IFF_UP,
            addresses: v4.to_vec(),
            ipv6: v6.to_vec(),
        }
    }

    /// **Every address on every interface, in both families**, and none of
    /// loopback's.
    #[test]
    fn every_address_but_loopback_is_this_machines() {
        let lan = Ipv4Addr::new(192, 168, 1, 20);
        let ula = Ipv6Addr::new(0xfd00, 0, 0, 0, 0, 0, 0, 0x20);
        let read = of(&[
            holding("lo", &[Ipv4Addr::LOCALHOST], &[Ipv6Addr::LOCALHOST]),
            holding("wlan0", &[lan], &[ula]),
        ]);
        assert_eq!(read, [IpAddr::V4(lan), IpAddr::V6(ula)]);
    }

    /// **A machine with nothing but loopback has nothing to add.**
    #[test]
    fn a_machine_with_only_loopback_has_nothing_to_add() {
        assert!(of(&[holding("lo", &[Ipv4Addr::LOCALHOST], &[])]).is_empty());
        assert!(of(&[]).is_empty());
    }

    /// **The kernel on this machine answers**, and what is read from it holds
    /// no loopback address. Measured against the machine the test runs on,
    /// which has loopback at least, so an answer that kept it would fail here.
    #[test]
    fn the_kernel_here_answers_without_loopback() {
        let reported = alo_agentd::route_messages::reported_by_the_kernel();
        assert!(reported.is_ok(), "{reported:?}");
        assert!(read_now().iter().all(|address| !address.is_loopback()));
    }
}
