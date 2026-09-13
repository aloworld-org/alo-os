//! The packet a machine sends about itself, and the one it sends to ask.
//!
//! # What is in an answer, and what is deliberately not
//!
//! Three records: a `PTR` saying the service has this instance, an `SRV` saying
//! where the instance is, and a `TXT` carrying one key. Everything in all three
//! is the machine's own identity, the service's name, the port, and the version
//! of this protocol — which is the closed list [`Presence`] holds, spelled on
//! the wire.
//!
//! **There is no `A` record**, and that is a decision rather than an omission.
//! An `A` record is where a responder lists the addresses it can be reached at,
//! and a machine with a wired connection, a wireless one and a virtual machine
//! bridge would list three — telling a network watching it how this machine is
//! attached to the world, which is more than presence. The address a machine is
//! reachable at is the source address of its own answer, which the packet has
//! already revealed by arriving; nothing further is owed and nothing further is
//! given. A resolver that insists on an `A` record gets no answer from us,
//! which is the correct trade in the direction of saying less.
//!
//! # And what an answer weighs
//!
//! It is under two hundred bytes and holds no pointers. Pointers would save
//! perhaps forty of them in a packet that already fits inside anything, at the
//! cost of an answer that can only be read with the whole packet in hand.

use crate::presence::{Presence, SERVICE, VERSION, VERSION_KEY};
use crate::refusing::NotNearby;
use crate::wire::{
    IN, IN_AND_THE_ONLY_ONE, kind, write_data, write_name, write_sixteen, write_thirty_two,
};

/// How long another machine may hold what it heard, in seconds.
///
/// Two minutes, which is what mDNS uses for the records that say where a
/// service is. A machine that goes away is forgotten within it, and a machine
/// that is there re-announces long before.
pub const HELD_FOR: u32 = 120;

/// The header bits on an answer: this is a reply, and it is authoritative.
const AN_ANSWER: u16 = 0x8400;

/// The header bits on a question, which are none of them.
const A_QUESTION: u16 = 0;

/// What a machine sends when it wants to know who else is here.
///
/// One question, for the service's instances. It asks for the service and not
/// for any machine, because asking for a machine by name would say which
/// machine this one is looking for — and *who I am looking for* is not presence
/// either.
///
/// # Errors
///
/// [`NotNearby::NotAName`] if the service name cannot be written, which is not
/// reachable while [`SERVICE`] is a constant in this workspace.
pub fn a_question() -> Result<Vec<u8>, NotNearby> {
    let mut packet = Vec::new();
    write_sixteen(&mut packet, 0); // No transaction id: mDNS matches on content.
    write_sixteen(&mut packet, A_QUESTION);
    write_sixteen(&mut packet, 1); // One question,
    write_sixteen(&mut packet, 0); // no answers,
    write_sixteen(&mut packet, 0); // nothing official,
    write_sixteen(&mut packet, 0); // and nothing extra.
    write_name(&mut packet, SERVICE)?;
    write_sixteen(&mut packet, kind::PTR);
    write_sixteen(&mut packet, IN);
    Ok(packet)
}

/// What a machine sends about itself.
///
/// # Errors
///
/// [`NotNearby::NotAName`] if the identity or the service cannot be written as
/// a name. Not reachable from a [`MachineId`](crate::MachineId) this crate
/// made, which is thirty-two characters, and checked anyway because the type
/// could one day be made from somewhere else.
pub fn about(presence: &Presence) -> Result<Vec<u8>, NotNearby> {
    let instance = presence.instance();
    let host = presence.host();

    let mut packet = Vec::new();
    write_sixteen(&mut packet, 0);
    write_sixteen(&mut packet, AN_ANSWER);
    write_sixteen(&mut packet, 0); // Nothing asked,
    write_sixteen(&mut packet, 1); // one answer,
    write_sixteen(&mut packet, 0); // nothing official,
    write_sixteen(&mut packet, 2); // and two records the answer needs.

    // The service has this instance.
    write_name(&mut packet, SERVICE)?;
    write_sixteen(&mut packet, kind::PTR);
    write_sixteen(&mut packet, IN);
    write_thirty_two(&mut packet, HELD_FOR);
    write_data(&mut packet, &instance, |data| write_name(data, &instance))?;

    // The instance is at this host and port. Priority and weight are zero:
    // there is one of this machine, and nothing to choose between.
    write_name(&mut packet, &instance)?;
    write_sixteen(&mut packet, kind::SRV);
    write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
    write_thirty_two(&mut packet, HELD_FOR);
    let port = presence.port();
    write_data(&mut packet, &instance, |data| {
        write_sixteen(data, 0);
        write_sixteen(data, 0);
        write_sixteen(data, port);
        write_name(data, &host)
    })?;

    // And it speaks this version of the protocol, which is the only key.
    write_name(&mut packet, &instance)?;
    write_sixteen(&mut packet, kind::TXT);
    write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
    write_thirty_two(&mut packet, HELD_FOR);
    write_data(&mut packet, &instance, |data| {
        let said = format!("{VERSION_KEY}={VERSION}");
        let Ok(length) = u8::try_from(said.len()) else {
            return Err(NotNearby::NotAName(said));
        };
        data.push(length);
        data.extend_from_slice(said.as_bytes());
        Ok(())
    })?;

    Ok(packet)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::{a_question, about};
    use crate::machine::MachineId;
    use crate::presence::Presence;
    use crate::wire::kind;

    /// A presence for a test to advertise.
    fn a_presence() -> Presence {
        Presence::of(
            MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap(),
            7_610,
        )
    }

    /// **No `A` record.** A machine does not list the addresses it can be
    /// reached at, because a machine with a wired connection, a wireless one
    /// and a bridge would be listing how it is attached to the world.
    #[test]
    fn an_advertisement_lists_no_address_of_this_machines_own() {
        let packet = about(&a_presence()).unwrap();
        // Every record's type is in the packet as two bytes; an `A` record's
        // type beside the internet class is the pair that must not be there.
        let an_address = [0, u8::try_from(kind::A).unwrap(), 0x80, 1];
        assert!(
            !packet.windows(4).any(|four| four == an_address),
            "the advertisement lists an address of this machine's own"
        );
    }

    /// A question asks for the service, not for a machine — *who I am looking
    /// for* is not presence either.
    #[test]
    fn a_question_names_the_service_and_no_machine() {
        let packet = a_question().unwrap();
        assert!(packet.windows(7).any(|of| of == b"_alo-os"));
        assert!(
            !packet.windows(4).any(|of| of == b"0f1e"),
            "the question names a machine"
        );
    }

    /// An advertisement is small enough that nothing on any network has to
    /// think about it.
    #[test]
    fn an_advertisement_fits_in_a_packet_nothing_has_to_think_about() {
        assert!(about(&a_presence()).unwrap().len() < 512);
    }
}
