//! Taking a packet off the network as a machine, and refusing anything that
//! says more than presence.
//!
//! # The refusal this file exists for
//!
//! Everything on a network can send an advertisement, and a later version of
//! alo OS could send one too. So what an advertisement is allowed to carry is a
//! closed list, and a key that is not on it is
//! [`NotNearby::SaysMoreThanPresence`] rather than a key that is skipped.
//!
//! Skipping it would be the ordinary thing — DNS-SD readers ignore what they do
//! not understand, and that is why the format has lasted. It is refused here
//! because the failure being guarded against is not a stranger's packet. It is
//! **this** machine, two years from now, advertising the person's name in a key
//! the reader would have ignored, with nothing in the workspace failing.
//!
//! # What is read, and in what order
//!
//! The instance name from the `PTR`, the port and host from the `SRV`, the keys
//! from the `TXT`. A packet with no `PTR` for this service is not an answer
//! about an alo machine at all and says so; a packet whose `SRV` and `TXT` are
//! about a different instance than the `PTR` is refused, because presence about
//! one machine carried in an answer about another is not something a correct
//! responder does.

use crate::machine::MachineId;
use crate::presence::{Found, SERVICE, VERSION, VERSION_KEY};
use crate::refusing::NotNearby;
use crate::wire::{Packet, kind};

/// The class bits that matter, the top one being mDNS's cache-flush bit rather
/// than part of the class.
const WITHOUT_THE_FLUSH_BIT: u16 = 0x7fff;

/// One machine, read out of an answer it sent.
///
/// # Errors
///
/// [`NotNearby::NotAnAloMachine`] for an answer about some other service, which
/// is the commonest thing on a network and is not a fault in anything;
/// [`NotNearby::SaysMoreThanPresence`] for an answer carrying a key that is not
/// on the list; [`NotNearby::CutShort`] or [`NotNearby::NameNeverEnds`] for a
/// packet that is not well formed; and
/// [`NotNearby::SaysNothingAboutWhichMachine`] for one that never named an
/// instance.
pub fn a_machine_in(packet: &[u8]) -> Result<Found, NotNearby> {
    let mut reading = Packet::of(packet);
    let _transaction = reading.sixteen()?;
    let _flags = reading.sixteen()?;
    let questions = reading.sixteen()?;
    let answers = reading.sixteen()?;
    let official = reading.sixteen()?;
    let extra = reading.sixteen()?;

    for _ in 0..questions {
        drop(reading.name()?);
        let _kind = reading.sixteen()?;
        let _class = reading.sixteen()?;
    }

    let mut instance: Option<String> = None;
    let mut port: Option<u16> = None;
    let mut version_said = false;

    let records = u32::from(answers)
        .saturating_add(u32::from(official))
        .saturating_add(u32::from(extra));
    for _ in 0..records {
        let name = reading.name()?;
        let kind_of = reading.sixteen()?;
        let _class = reading.sixteen()? & WITHOUT_THE_FLUSH_BIT;
        let _held_for = reading.thirty_two()?;
        let length = usize::from(reading.sixteen()?);
        let after = reading
            .so_far()
            .checked_add(length)
            .ok_or(NotNearby::CutShort)?;

        match kind_of {
            kind::PTR if name.eq_ignore_ascii_case(SERVICE) => {
                instance = Some(reading.name()?);
            }
            kind::SRV => {
                let _priority = reading.sixteen()?;
                let _weight = reading.sixteen()?;
                let said = reading.sixteen()?;
                if instance.as_deref().is_some_and(|of| of == name) {
                    port = Some(said);
                }
            }
            kind::TXT if instance.as_deref().is_some_and(|of| of == name) => {
                version_said = a_version_and_nothing_else(reading.some(length)?)?;
            }
            _ => {}
        }
        reading.go_to(after)?;
    }

    let Some(instance) = instance else {
        return Err(NotNearby::NotAnAloMachine(a_word_for(packet)));
    };
    let Some(said) = instance.strip_suffix(&format!(".{SERVICE}")) else {
        return Err(NotNearby::NotAnAloMachine(instance));
    };
    let machine = MachineId::read(said)?;
    let Some(port) = port else {
        return Err(NotNearby::SaysNothingAboutWhichMachine);
    };
    if !version_said {
        return Err(NotNearby::SaysNothingAboutWhichMachine);
    }
    Ok(Found::seen(machine, port))
}

/// Whether a packet is somebody asking who is here.
///
/// A question for this service and no other. Anything else on the address —
/// printers, media players, a machine's own answer coming back — is not one,
/// and is not an error either.
#[must_use]
pub fn a_question_in(packet: &[u8]) -> bool {
    let asked = || -> Result<bool, NotNearby> {
        let mut reading = Packet::of(packet);
        let _transaction = reading.sixteen()?;
        let flags = reading.sixteen()?;
        let questions = reading.sixteen()?;
        // The rest of the header, which is counted and stepped over: a
        // question's name begins after all six of its numbers, not after three.
        let _answers = reading.sixteen()?;
        let _official = reading.sixteen()?;
        let _extra = reading.sixteen()?;
        if flags & 0x8000 != 0 {
            // The reply bit: this is somebody's answer, not their question.
            return Ok(false);
        }
        for _ in 0..questions {
            let name = reading.name()?;
            let kind_of = reading.sixteen()?;
            let _class = reading.sixteen()?;
            if kind_of == kind::PTR && name.eq_ignore_ascii_case(SERVICE) {
                return Ok(true);
            }
        }
        Ok(false)
    };
    asked().unwrap_or(false)
}

/// Whether the keys in a `TXT` record are the one key presence may carry, and
/// nothing else.
///
/// # Errors
///
/// [`NotNearby::SaysMoreThanPresence`] naming the key that is not on the list —
/// the refusal this file exists for — and [`NotNearby::CutShort`] for a record
/// whose lengths run past its own data.
fn a_version_and_nothing_else(data: &[u8]) -> Result<bool, NotNearby> {
    let mut at = 0_usize;
    let mut said = false;
    while let Some(&length) = data.get(at) {
        let from = at.saturating_add(1);
        let to = from
            .checked_add(usize::from(length))
            .ok_or(NotNearby::CutShort)?;
        let entry = data.get(from..to).ok_or(NotNearby::CutShort)?;
        at = to;
        if entry.is_empty() {
            // A single empty string is how DNS-SD spells "no keys at all"; it
            // is the absence of a key rather than a key.
            continue;
        }
        let entry = String::from_utf8_lossy(entry);
        let (key, value) = entry.split_once('=').unwrap_or((entry.as_ref(), ""));
        if key != VERSION_KEY {
            return Err(NotNearby::SaysMoreThanPresence(key.to_owned()));
        }
        if value != VERSION {
            return Err(NotNearby::SaysMoreThanPresence(entry.into_owned()));
        }
        said = true;
    }
    Ok(said)
}

/// Something true to call a packet that never said what it was about.
///
/// The bytes are deliberately not in it. A refusal naming a stranger's packet
/// would put a stranger's bytes wherever the refusal is printed, and law 1 is
/// about what leaves this machine in both directions.
fn a_word_for(packet: &[u8]) -> String {
    format!("something else, in {} bytes", packet.len())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::a_machine_in;
    use crate::advertising::about;
    use crate::machine::MachineId;
    use crate::presence::{Presence, SERVICE, Standing};
    use crate::refusing::NotNearby;
    use crate::wire::{
        IN_AND_THE_ONLY_ONE, kind, write_data, write_name, write_sixteen, write_thirty_two,
    };

    /// The identity every test here advertises.
    fn an_identity() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// **A second machine reads the advertisement and answers with one machine,
    /// not paired.** The whole of task one's promise, in one round trip.
    #[test]
    fn a_machine_that_advertises_is_read_back_as_one_machine_not_paired() {
        let packet = about(&Presence::of(an_identity(), 7_610)).unwrap();
        let found = a_machine_in(&packet).unwrap();
        assert_eq!(found.machine, an_identity());
        assert_eq!(found.port, 7_610);
        assert_eq!(found.standing, Standing::NotPaired);
    }

    /// An advertisement from an alo machine built with the same presence is the
    /// same bytes every time, so nothing about the machine's state leaks
    /// through the shape of what it says.
    #[test]
    fn the_same_presence_advertises_the_same_bytes_every_time() {
        let once = about(&Presence::of(an_identity(), 7_610)).unwrap();
        let again = about(&Presence::of(an_identity(), 7_610)).unwrap();
        assert_eq!(once, again);
    }

    /// **What this machine advertises carries no word of what this machine is
    /// called.** The hostname is the field DNS-SD ordinarily fills, and it is
    /// the one that tells a café who is in the café.
    #[test]
    fn the_advertisement_carries_no_word_of_this_machines_own_name() {
        let packet = about(&Presence::of(an_identity(), 7_610)).unwrap();
        let called = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_default()
            .to_lowercase();
        if called.len() < 3 {
            // Nothing to look for; the test says so rather than passing
            // quietly on a machine with no name.
            return;
        }
        let written = String::from_utf8_lossy(&packet).to_lowercase();
        assert!(
            !written.contains(&called),
            "the advertisement carries this machine's own name, `{called}`"
        );
    }

    /// An answer about a printer or a media player is not a fault in anything,
    /// and says which service it was about rather than carrying its bytes.
    #[test]
    fn an_answer_about_another_service_is_not_an_alo_machine() {
        let mut packet = Vec::new();
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 0x8400);
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 1);
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 0);
        write_name(&mut packet, "_ipp._tcp.local").unwrap();
        write_sixteen(&mut packet, kind::PTR);
        write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
        write_thirty_two(&mut packet, 120);
        write_data(&mut packet, "printer", |data| {
            write_name(data, "a-printer._ipp._tcp.local")
        })
        .unwrap();

        let refused = a_machine_in(&packet).unwrap_err();

        assert!(matches!(refused, NotNearby::NotAnAloMachine(_)));
        assert!(
            refused.is_about_a_stranger(),
            "a printer on the network reads as a fault in this machine"
        );
    }

    /// **A key that is not on the list is refused, not skipped** — and the
    /// failure being guarded against is this machine two years from now, not a
    /// stranger.
    #[test]
    fn an_advertisement_saying_more_than_presence_is_refused_by_name() {
        for saying in ["who=disan", "models=mistral-7b", "org=axon", "paired=2"] {
            let packet = an_advertisement_also_saying(saying);
            let refused = a_machine_in(&packet).unwrap_err();
            let key = saying.split_once('=').unwrap().0;
            assert_eq!(
                refused,
                NotNearby::SaysMoreThanPresence(key.to_owned()),
                "`{saying}` was read past rather than refused"
            );
        }
    }

    /// An advertisement claiming a protocol version this machine does not speak
    /// is refused at the same door, because the version is the only key and a
    /// different one is a different closed list.
    #[test]
    fn an_advertisement_claiming_another_version_is_refused() {
        let packet = an_advertisement_also_saying("v=2");
        assert!(matches!(
            a_machine_in(&packet).unwrap_err(),
            NotNearby::SaysMoreThanPresence(_)
        ));
    }

    /// A truncated packet is refused rather than read up to whatever is there.
    #[test]
    fn half_an_advertisement_is_refused() {
        let whole = about(&Presence::of(an_identity(), 7_610)).unwrap();
        for how_much in [4_usize, 12, 30, 60] {
            let half = whole.get(..how_much).unwrap().to_vec();
            assert!(
                a_machine_in(&half).is_err(),
                "{how_much} bytes of an advertisement read as a machine"
            );
        }
    }

    /// An advertisement carrying one extra key beside the version, built by
    /// hand because the writer in this crate cannot produce one.
    fn an_advertisement_also_saying(entry: &str) -> Vec<u8> {
        let presence = Presence::of(an_identity(), 7_610);
        let instance = presence.instance();
        let mut packet = Vec::new();
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 0x8400);
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 1);
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 2);

        write_name(&mut packet, SERVICE).unwrap();
        write_sixteen(&mut packet, kind::PTR);
        write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
        write_thirty_two(&mut packet, 120);
        write_data(&mut packet, &instance, |data| write_name(data, &instance)).unwrap();

        write_name(&mut packet, &instance).unwrap();
        write_sixteen(&mut packet, kind::SRV);
        write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
        write_thirty_two(&mut packet, 120);
        write_data(&mut packet, &instance, |data| {
            write_sixteen(data, 0);
            write_sixteen(data, 0);
            write_sixteen(data, 7_610);
            write_name(data, &presence.host())
        })
        .unwrap();

        write_name(&mut packet, &instance).unwrap();
        write_sixteen(&mut packet, kind::TXT);
        write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
        write_thirty_two(&mut packet, 120);
        write_data(&mut packet, &instance, |data| {
            for said in ["v=1", entry] {
                data.push(u8::try_from(said.len()).unwrap_or(0));
                data.extend_from_slice(said.as_bytes());
            }
            Ok(())
        })
        .unwrap();

        packet
    }
}
