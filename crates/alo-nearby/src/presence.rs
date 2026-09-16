//! What a machine advertises, which is a closed list of two things.
//!
//! # The list, and why it is closed
//!
//! [`Presence`] holds a [`MachineId`] and a port. There is no field for the
//! person, the organisation, the models on the machine, what it has granted,
//! what it is doing, or what it is called — and there is no field for whether
//! it has paired with anything, so **what a machine advertises is the same
//! whether it has paired with nothing or with everything**. Presence that
//! changed shape when a pairing was made would tell a network watching it that
//! a pairing had been made.
//!
//! The list is closed in the type rather than by convention. A later change
//! that wanted to say more would have to add a field here, in a file whose
//! entire subject is that it does not, rather than append a key somewhere in a
//! packet builder.
//!
//! # And what is found is no more than that
//!
//! [`Found`] is the other side: one machine, seen. It carries what was
//! advertised and [`Standing`], which today has one arm — nobody is paired with
//! anybody, because pairing does not exist yet. Being on the network is not
//! standing of any kind (ADR 0003), and the enum is the shape of that sentence:
//! finding a machine moves nothing.

use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU32;

use crate::heard_from::HeardFrom;
use crate::machine::MachineId;

/// The service this machine answers to on a local network.
///
/// `_alo-os._tcp.local` in DNS-SD's own spelling: an application protocol named
/// `alo-os`, over TCP, on the link-local name space every machine on the
/// network shares.
pub const SERVICE: &str = "_alo-os._tcp.local";

/// The one key an advertisement may carry beyond the service and the machine,
/// being which version of this protocol the machine speaks.
pub const VERSION_KEY: &str = "v";

/// The version this crate speaks.
pub const VERSION: &str = "1";

/// What this machine says about itself on a local network.
///
/// Made from an identity and a port, and nothing else is reachable from here —
/// see this module's own documentation for why the list is closed in the type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Presence {
    /// Which machine.
    machine: MachineId,
    /// Where it would answer, if anything were ever permitted to ask it.
    port: u16,
}

impl Presence {
    /// This machine, at a port.
    ///
    /// The port is advertised; what listens on it is whoever runs alo OS,
    /// with [`crate::Receiving`] for proposals and the corridor for questions.
    /// Being reachable is not being usable, and what it takes to use a machine
    /// found this way is [ADR 0003]'s mutual pairing, which is what a proposal
    /// on that port asks for.
    ///
    /// [ADR 0003]: https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0003-the-network-is-not-authority.md
    #[must_use]
    pub const fn of(machine: MachineId, port: u16) -> Self {
        Self { machine, port }
    }

    /// Which machine this is.
    #[must_use]
    pub const fn machine(&self) -> &MachineId {
        &self.machine
    }

    /// The port it advertises.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// The name this machine answers to in the service, which is its identity
    /// and the service's own name.
    ///
    /// A DNS-SD instance name is ordinarily something a person reads — *Disan's
    /// printer* — and that is exactly the leak this crate is organised against,
    /// so the instance is the identity.
    #[must_use]
    pub fn instance(&self) -> String {
        format!("{}.{SERVICE}", self.machine)
    }

    /// The host name it answers to, which is again its identity and nothing a
    /// person chose.
    #[must_use]
    pub fn host(&self) -> String {
        format!("{}.local", self.machine)
    }
}

/// One machine, seen on the local network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// Which machine it said it was.
    pub machine: MachineId,
    /// The port it advertised.
    pub port: u16,
    /// The address its answer came from.
    ///
    /// Measured, not advertised: an advertisement carries no `A` record
    /// (`advertising.rs` says why), and the one true address a machine can be
    /// reached at is the one it was heard from. What dials the machine later
    /// dials this, so the next hop is what discovery measured rather than what
    /// somebody typed. A link-local IPv6 address carries the interface it was
    /// heard on ([`HeardFrom`]), because without it the address names no network.
    pub address: HeardFrom,
    /// Where else the same machine answered from in the same look: one address
    /// for each further network it was heard on, in the order the networks
    /// were asked, and empty for a machine heard on one.
    ///
    /// Measured as [`address`](Self::address) is. A machine on two networks is
    /// one machine with an address on each ([`crate::Around::heard_on_each`]),
    /// and [`address`](Self::address) stays the one heard first, which is what
    /// dials it.
    pub also_at: Vec<HeardFrom>,
    /// What that means for this machine, which is nothing.
    pub standing: Standing,
}

/// What a machine found on the network is to this one.
///
/// # One arm
///
/// ADR 0003: *being on the same network is not authority.* Finding a machine
/// establishes that it exists and nothing else, so today every machine found is
/// [`Standing::NotPaired`] — there is no way to be anything else, because
/// pairing is not built. The enum exists at one arm rather than being a `bool`
/// or absent because the next task adds the other, and because a reader looking
/// for *what does finding a machine give it* should find the answer written
/// down rather than have to notice that nothing gives it anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Standing {
    /// Seen, and nothing more. Nothing on this machine is open to it.
    NotPaired,
}

impl Found {
    /// A machine seen on the network, heard from `address`, which is not
    /// paired with this one, because seeing it pairs nothing.
    ///
    /// For an address that names its own network — IPv4, or IPv6 that is not
    /// link-local. A machine heard over a link-local address is [`Found::heard`],
    /// with the interface it was heard on.
    #[must_use]
    pub const fn seen(machine: MachineId, port: u16, address: IpAddr) -> Self {
        Self::heard(machine, port, HeardFrom::named(address))
    }

    /// A machine heard from `address`, measured with the interface it arrived
    /// on where that address is link-local — not paired with this one, because
    /// hearing it pairs nothing.
    #[must_use]
    pub const fn heard(machine: MachineId, port: u16, address: HeardFrom) -> Self {
        Self {
            machine,
            port,
            address,
            also_at: Vec::new(),
            standing: Standing::NotPaired,
        }
    }

    /// Every address the machine answered from in the look it was found in:
    /// the one heard first, then one for each further network or family.
    pub fn addresses(&self) -> impl Iterator<Item = HeardFrom> + '_ {
        std::iter::once(self.address).chain(self.also_at.iter().copied())
    }

    /// Where the machine answers: the address it was heard from and the port
    /// it advertised — with the interface, for a link-local address.
    ///
    /// The one thing a `Found` says about reaching a machine, and the thing
    /// nothing in this crate dials — what dials it is the corridor, under a
    /// pairing two people made.
    #[must_use]
    pub const fn where_it_answers(&self) -> SocketAddr {
        self.address.at(self.port)
    }

    /// The interface of the network the machine was heard on, where discovery
    /// measured one — which is what a connection to it is held to.
    ///
    /// [ADR 0041](../../../docs/decisions/0041-a-link-local-departure-names-its-interface.md)
    /// for a link-local address and
    /// [ADR 0044](../../../docs/decisions/0044-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md)
    /// for a private IPv4 one: `192.168.1.20` on the wired network and
    /// `192.168.1.20` on the Wi-Fi are two machines, and a socket held to
    /// nothing reaches whichever the route says at the moment it connects.
    /// `None` where nobody said — a machine heard at an address that names its
    /// own network, and every measurement made before there were networks to
    /// tell apart.
    #[must_use]
    pub const fn on_the_network(&self) -> Option<NonZeroU32> {
        match self.address.interface() {
            Some(interface) => NonZeroU32::new(interface),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Found, Presence, SERVICE, Standing};
    use crate::machine::MachineId;

    #[expect(
        clippy::unwrap_used,
        reason = "in a test, a panic on an unexpected Err is the failure being reported"
    )]
    /// An identity for a test to build a presence from.
    fn a_machine() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// The instance a machine answers to is its identity, not a name anybody
    /// chose — which is the field DNS-SD ordinarily fills with a person's own
    /// words.
    #[test]
    fn the_name_on_the_network_is_the_identity_and_the_service() {
        let presence = Presence::of(a_machine(), 7_610);
        assert_eq!(
            presence.instance(),
            format!("0f1e2d3c4b5a69788796a5b4c3d2e1f0.{SERVICE}")
        );
        assert_eq!(presence.host(), "0f1e2d3c4b5a69788796a5b4c3d2e1f0.local");
    }

    /// **Presence is made of two things**, and this is the test that fails if
    /// somebody adds a third: two presences with the same identity and port are
    /// the same presence, whatever else is true of either machine.
    #[test]
    fn two_machines_agreeing_on_identity_and_port_advertise_the_same_thing() {
        assert_eq!(
            Presence::of(a_machine(), 7_610),
            Presence::of(a_machine(), 7_610)
        );
    }

    /// Finding a machine gives this one nothing, which is ADR 0003 in the one
    /// place the type could have said otherwise.
    #[test]
    fn a_machine_that_has_been_found_is_not_paired_with() {
        assert_eq!(
            Found::seen(a_machine(), 7_610, std::net::Ipv4Addr::LOCALHOST.into()).standing,
            Standing::NotPaired
        );
    }
}
