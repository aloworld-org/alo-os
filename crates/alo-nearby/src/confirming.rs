//! What one machine sends the other to say *the person here confirmed* — made
//! so that only the machine whose offer was in the proposal could have sent
//! it.
//!
//! # Why a confirmation is proved rather than merely said
//!
//! A pairing is kept on each machine only after both people have confirmed on
//! their own, and each machine learns of the *other* person's confirmation
//! over the wire. A confirmation anybody on the network could send would let
//! a stranger tell reception that the studio's person said yes — and
//! reception, whose own person had said yes, would keep a pairing the studio
//! never made. What stops that is the key `deliberating.rs` agreed the moment
//! both offers were known: it is held by exactly the two machines whose
//! offers crossed, so a tag made with it over the transcript is something
//! only they can make. Bluetooth's key check at the end of its pairing is the
//! same step under another name, and ADR 0031 is why the key is there to use.
//!
//! # The wire spelling
//!
//! One line: `alo-os/1 confirmed <from> <to> <tag>`, and [`Confirmation::read`]
//! refuses every other shape. The tag is over which machine is confirming,
//! which it is confirming to, and the whole transcript — both identities,
//! both offers, the list and the duration — so a confirmation for one
//! proposal says nothing about any other, and one machine's confirmation
//! cannot be turned round and presented as the other's.

use ring::hmac::{HMAC_SHA256, Key};

use crate::hexing;
use crate::keying::{PairingKey, Transcript};
use crate::machine::MachineId;
use crate::proof::A_TAG;
use crate::refusing::NotNearby;

/// What every confirmation begins with on the wire, and which version it
/// speaks.
const ON_THE_WIRE: &str = "alo-os/1";

/// The word that says what kind of line this is.
const CONFIRMED: &str = "confirmed";

/// What the signed bytes begin with, so a tag over some other structure —
/// a proof, say — could never be read as one over this.
const A_CONFIRMATION: &[u8] = b"alo-os pairing confirmed v1\0";

/// *The person here confirmed*, from one machine to the other.
///
/// Made only inside this crate, from the agreed key — and the key is held
/// only by a deliberation that has both offers. There is no way to make one
/// without it, and [`read`](Self::read) checks only the shape of one that
/// arrived.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Confirmation {
    /// The machine whose person confirmed.
    from: MachineId,
    /// The machine being told.
    to: MachineId,
    /// The tag over all of it and the transcript.
    tag: [u8; A_TAG],
}

impl Confirmation {
    /// A confirmation from `from` to `to`, made with the key both agreed over
    /// `transcript`.
    pub(crate) fn made(
        key: &PairingKey,
        transcript: &Transcript,
        from: &MachineId,
        to: &MachineId,
    ) -> Self {
        let mut confirmation = Self {
            from: from.clone(),
            to: to.clone(),
            tag: [0; A_TAG],
        };
        let key = Key::new(HMAC_SHA256, key.bytes());
        let tag = ring::hmac::sign(&key, &confirmation.signed(transcript));
        confirmation.tag.copy_from_slice(tag.as_ref());
        confirmation
    }

    /// The bytes the tag is over.
    fn signed(&self, transcript: &Transcript) -> Vec<u8> {
        let mut over = Vec::with_capacity(A_CONFIRMATION.len() + 72 + transcript.bytes().len());
        over.extend_from_slice(A_CONFIRMATION);
        over.extend_from_slice(self.from.as_str().as_bytes());
        over.push(0);
        over.extend_from_slice(self.to.as_str().as_bytes());
        over.push(0);
        over.extend_from_slice(transcript.bytes());
        over
    }

    /// Whether the tag is the one `key` makes over this and `transcript`.
    ///
    /// Constant time, which is `ring`'s and not this file's.
    pub(crate) fn is_by(&self, key: &PairingKey, transcript: &Transcript) -> bool {
        let key = Key::new(HMAC_SHA256, key.bytes());
        ring::hmac::verify(&key, &self.signed(transcript), &self.tag).is_ok()
    }

    /// The confirmation as it travels: one line.
    #[must_use]
    pub fn said(&self) -> String {
        format!(
            "{ON_THE_WIRE} {CONFIRMED} {} {} {}",
            self.from,
            self.to,
            hexing::said(&self.tag)
        )
    }

    /// A confirmation somebody sent, read back and checked for shape — and for
    /// nothing else: whether it was made by the machine being paired with is
    /// [`crate::Proposals::confirmation_arrived`]'s to answer.
    ///
    /// # Errors
    ///
    /// [`NotNearby::NotAConfirmation`] for anything that is not exactly the
    /// line [`said`](Self::said) writes, and [`NotNearby::NotAMachineIdentity`]
    /// for a machine in it that is not one.
    pub fn read(said: &str) -> Result<Self, NotNearby> {
        let fields: Vec<&str> = said.trim().split(' ').collect();
        let &[version, kind, from, to, tag] = fields.as_slice() else {
            return Err(NotNearby::NotAConfirmation(format!(
                "{} fields rather than five",
                fields.len()
            )));
        };
        if version != ON_THE_WIRE {
            return Err(NotNearby::NotAConfirmation(format!(
                "`{version}` is not a version spoken here"
            )));
        }
        if kind != CONFIRMED {
            return Err(NotNearby::NotAConfirmation(format!(
                "`{kind}` is not a confirmation"
            )));
        }
        let mut bytes = [0_u8; A_TAG];
        if !hexing::read(tag, &mut bytes) {
            return Err(NotNearby::NotAConfirmation(
                "the tag is not sixty-four hexadecimal characters".to_owned(),
            ));
        }
        Ok(Self {
            from: MachineId::read(from)?,
            to: MachineId::read(to)?,
            tag: bytes,
        })
    }

    /// The machine whose person says they confirmed.
    #[must_use]
    pub const fn from(&self) -> &MachineId {
        &self.from
    }

    /// The machine being told.
    #[must_use]
    pub const fn to(&self) -> &MachineId {
        &self.to
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::Confirmation;
    use crate::keying::Keying;
    use crate::refusing::NotNearby;
    use crate::testing::{both_sides, reception, studio};

    /// **A confirmation made on one machine verifies on the other**, and
    /// survives the wire.
    #[test]
    fn a_confirmation_made_on_one_side_verifies_on_the_other_and_survives_the_wire() {
        let (at_reception, at_studio) = both_sides();
        let from_reception = Confirmation::made(
            at_reception.key().unwrap(),
            &at_reception.transcript().unwrap(),
            &reception(),
            &studio(),
        );
        assert!(from_reception.is_by(at_studio.key().unwrap(), &at_studio.transcript().unwrap()));

        let travelled = Confirmation::read(&from_reception.said()).unwrap();
        assert_eq!(travelled, from_reception);
        assert_eq!(travelled.from(), &reception());
        assert_eq!(travelled.to(), &studio());
        assert!(travelled.is_by(at_studio.key().unwrap(), &at_studio.transcript().unwrap()));
    }

    /// **A confirmation from a machine without the key does not verify** —
    /// one made with another key, over another transcript, or the studio's
    /// own turned round and presented as reception's.
    #[test]
    fn a_confirmation_without_the_key_or_for_another_proposal_does_not_verify() {
        let (at_reception, at_studio) = both_sides();
        let key = at_studio.key().unwrap();
        let transcript = at_studio.transcript().unwrap();

        // A stranger who agreed a key of its own with somebody.
        let (elsewhere, _) = both_sides();
        let forged = Confirmation::made(
            elsewhere.key().unwrap(),
            &elsewhere.transcript().unwrap(),
            &reception(),
            &studio(),
        );
        assert!(!forged.is_by(key, &transcript));

        // The right key, but over the terms of another proposal.
        let (other_terms, _) = both_sides();
        let on_other_terms = Confirmation::made(
            at_reception.key().unwrap(),
            &other_terms.transcript().unwrap(),
            &reception(),
            &studio(),
        );
        assert!(!on_other_terms.is_by(key, &transcript));

        // The studio's own confirmation, turned round.
        let studios_own = Confirmation::made(key, &transcript, &studio(), &reception());
        let turned_round = Confirmation::read(&format!(
            "alo-os/1 confirmed {} {} {}",
            reception(),
            studio(),
            studios_own.said().rsplit(' ').next().unwrap()
        ))
        .unwrap();
        assert!(!turned_round.is_by(key, &transcript));
    }

    /// Everything that is not the line this file writes is refused by shape.
    #[test]
    fn what_is_not_a_confirmation_is_refused_rather_than_read() {
        let (at_reception, _) = both_sides();
        let good = Confirmation::made(
            at_reception.key().unwrap(),
            &at_reception.transcript().unwrap(),
            &reception(),
            &studio(),
        )
        .said();
        let fields: Vec<&str> = good.split(' ').collect();
        assert_eq!(fields.len(), 5, "{good}");
        let with = |at: usize, replaced: &str| -> String {
            fields
                .iter()
                .enumerate()
                .map(|(n, field)| if n == at { replaced } else { field })
                .collect::<Vec<_>>()
                .join(" ")
        };
        let mut with_a_field_missing = good.clone();
        with_a_field_missing.truncate(good.rfind(' ').unwrap());
        for not_one in [
            String::new(),
            with(0, "alo-os/2"),
            with(1, "proposal"),
            with_a_field_missing,
            format!("{good} extra"),
            with(2, "disan-laptop"),
            with(4, "abc"),
            good.to_uppercase(),
            Keying::fresh().unwrap().offer().said(),
        ] {
            assert!(
                matches!(
                    Confirmation::read(&not_one).unwrap_err(),
                    NotNearby::NotAConfirmation(_) | NotNearby::NotAMachineIdentity(_)
                ),
                "`{not_one}` was read as a confirmation"
            );
        }
    }
}
