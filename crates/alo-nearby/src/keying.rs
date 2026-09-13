//! What a pairing leaves each machine holding, and how the two of them come to
//! hold the same thing without either sending it.
//!
//! ADR 0031. Each side makes a [`Keying`] for one proposal: a private half it
//! keeps and an [`Offer`] it sends. Two offers cross the wire, no private half
//! does, and each machine computes the same [`PairingKey`] from its own private
//! half and the other's offer. Everything on a network can read both offers and
//! is no nearer the key for it; what an eavesdropper at the moment of pairing
//! cannot do is *substitute* an offer without the two people noticing, and
//! [`Code`] is what they notice it by.
//!
//! # Nothing here is written by this crate
//!
//! X25519, HKDF and SHA-256 are `ring`'s. This file names which of them, in
//! what order, over which bytes — the transcript — and refuses to be cleverer
//! than that. A key derivation this team wrote would be the security bug the
//! rest of the workspace is organised to avoid.

use std::fmt::{self, Debug, Formatter};
use std::time::Duration;

use ring::agreement::{EphemeralPrivateKey, UnparsedPublicKey, X25519, agree_ephemeral};
use ring::hkdf::{HKDF_SHA256, Salt};

use crate::hexing;
use crate::machine::MachineId;
use crate::pairing::NotPaired;
use crate::permitting::MayAskIts;
use crate::refusing::NotNearby;

/// How many bytes a public half is, which for X25519 is thirty-two.
const AN_OFFER: usize = 32;

/// How many bytes a pairing key is.
const A_KEY: usize = 32;

/// How many digits the code two people compare has.
const DIGITS: u32 = 6;

/// What the key is derived for, so a key for something else derived from the
/// same secret could never be this one.
const FOR_A_PAIRING: &[&[u8]] = &[b"alo-os pairing key v1"];

/// What the transcript begins with, so bytes that happen to look like one for a
/// different purpose are not one.
const A_TRANSCRIPT: &[u8] = b"alo-os pairing v1\0";

/// What the code is derived from, apart from the transcript.
const A_CODE: &[u8] = b"alo-os pairing code v1\0";

/// The public half of one side's part in one pairing.
///
/// Thirty-two bytes, made once per proposal and never reused: two pairings of
/// one machine carry two different offers, so nothing watching the network
/// can link them through it.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Offer([u8; AN_OFFER]);

impl Offer {
    /// An offer somebody else said, checked before it is believed.
    ///
    /// # Errors
    ///
    /// [`NotNearby::NotAnOffer`] for anything that is not sixty-four lowercase
    /// hexadecimal characters.
    pub fn read(said: &str) -> Result<Self, NotNearby> {
        let mut bytes = [0_u8; AN_OFFER];
        if hexing::read(said, &mut bytes) {
            Ok(Self(bytes))
        } else {
            Err(NotNearby::NotAnOffer(said.len()))
        }
    }

    /// The offer as it is written on the wire.
    #[must_use]
    pub fn said(&self) -> String {
        hexing::said(&self.0)
    }
}

impl Debug for Offer {
    fn fmt(&self, into: &mut Formatter<'_>) -> fmt::Result {
        write!(into, "Offer({})", self.said())
    }
}

/// One side's part in one pairing: the private half it keeps, and the offer it
/// sends.
///
/// Deliberately not `Clone`: there is one private half, it is consumed by the
/// agreement, and a copy would be a second chance to agree with somebody else.
#[derive(Debug)]
pub struct Keying {
    /// The half that stays on this machine.
    private: EphemeralPrivateKey,
    /// The half that goes to the other one.
    offer: Offer,
}

impl Keying {
    /// A fresh part, from the kernel's randomness.
    ///
    /// # Errors
    ///
    /// [`NotNearby::NoRandomness`] if the kernel will not give any, which is the
    /// same refusal [`MachineId::made`] makes and for the same reason: there is
    /// no fallback generator.
    pub fn fresh() -> Result<Self, NotNearby> {
        let random = ring::rand::SystemRandom::new();
        let private = EphemeralPrivateKey::generate(&X25519, &random)
            .map_err(|_| NotNearby::NoRandomness("the system generator refused".to_owned()))?;
        let public = private
            .compute_public_key()
            .map_err(|_| NotNearby::NoRandomness("no public half could be made".to_owned()))?;
        let mut offer = [0_u8; AN_OFFER];
        let bytes = public.as_ref();
        if bytes.len() != AN_OFFER {
            return Err(NotNearby::NoRandomness(format!(
                "a public half of {} bytes",
                bytes.len()
            )));
        }
        offer.copy_from_slice(bytes);
        Ok(Self {
            private,
            offer: Offer(offer),
        })
    }

    /// The half that goes to the other machine.
    #[must_use]
    pub const fn offer(&self) -> &Offer {
        &self.offer
    }

    /// The key both machines will hold, from this side's private half and the
    /// other side's offer, bound to what was agreed.
    ///
    /// Consumes the private half: one agreement per keying.
    ///
    /// # Errors
    ///
    /// [`NotPaired::NoKey`] if the agreement fails, which an offer of the right
    /// length does not cause and is answered rather than unwrapped.
    pub(crate) fn agreed(
        self,
        theirs: &Offer,
        transcript: &Transcript,
    ) -> Result<PairingKey, NotPaired> {
        let salt = Salt::new(HKDF_SHA256, &transcript.bytes);
        let peer = UnparsedPublicKey::new(&X25519, theirs.0);
        agree_ephemeral(self.private, &peer, |shared| {
            let mut key = [0_u8; A_KEY];
            salt.extract(shared)
                .expand(FOR_A_PAIRING, HKDF_SHA256)
                .and_then(|okm| okm.fill(&mut key))
                .map(|()| PairingKey(key))
        })
        .map_err(|_| NotPaired::NoKey)?
        .map_err(|_| NotPaired::NoKey)
    }
}

/// The key one pairing is made with, held on each machine's row for it.
///
/// Printed as nothing: it is the one value in this crate that must never reach
/// a log, a record or a screen. Its equality is the ordinary kind and is owed
/// nothing better — two rows on one machine are compared for the list a person
/// sees, and nothing that arrives from a network is ever compared with a key.
/// The one comparison an attacker can influence is a tag against a tag, and
/// that is `ring`'s constant-time verify in `proof.rs`.
#[derive(Clone, PartialEq, Eq)]
pub struct PairingKey([u8; A_KEY]);

impl PairingKey {
    /// The key, for the two files that sign and verify with it.
    pub(crate) const fn bytes(&self) -> &[u8; A_KEY] {
        &self.0
    }

    /// A key a test names, for the tests about pairings that are not about
    /// keys. Not compiled into anything a machine runs.
    #[cfg(test)]
    pub(crate) const fn of(bytes: [u8; A_KEY]) -> Self {
        Self(bytes)
    }
}

impl Debug for PairingKey {
    fn fmt(&self, into: &mut Formatter<'_>) -> fmt::Result {
        into.write_str("PairingKey(..)")
    }
}

/// Everything the two people agreed, as bytes both sides spell the same way.
///
/// Both identities, both offers, the enumerated list and the duration: the key
/// is derived *over* this, so a pairing whose terms were altered in transit
/// leaves the two sides holding different keys — and every proof failing —
/// rather than a pairing on terms one of them never saw.
pub(crate) struct Transcript {
    /// The bytes, in one order.
    bytes: Vec<u8>,
}

impl Transcript {
    /// The transcript of one proposal, once both offers are known.
    pub(crate) fn of(
        asking: &MachineId,
        asked: &MachineId,
        asking_offer: &Offer,
        asked_offer: &Offer,
        may: &[MayAskIts],
        lasting: Duration,
    ) -> Self {
        let mut bytes = Vec::with_capacity(192);
        bytes.extend_from_slice(A_TRANSCRIPT);
        bytes.extend_from_slice(asking.as_str().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(asked.as_str().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&asking_offer.0);
        bytes.extend_from_slice(&asked_offer.0);
        for arm in may {
            bytes.extend_from_slice(arm.word().named().as_bytes());
            bytes.push(0);
        }
        bytes.extend_from_slice(&lasting.as_secs().to_be_bytes());
        Self { bytes }
    }

    /// The six digits two people compare, derived from this transcript.
    pub(crate) fn code(&self) -> Code {
        let mut over = Vec::with_capacity(A_CODE.len().saturating_add(self.bytes.len()));
        over.extend_from_slice(A_CODE);
        over.extend_from_slice(&self.bytes);
        let digest = ring::digest::digest(&ring::digest::SHA256, &over);
        let first_four = digest
            .as_ref()
            .get(..4)
            .and_then(|four| <[u8; 4]>::try_from(four).ok())
            .unwrap_or([0; 4]);
        let number = u32::from_be_bytes(first_four) % 10_u32.pow(DIGITS);
        Code(format!("{number:06}"))
    }
}

/// Six digits shown on both machines beside the confirmation.
///
/// **What a surface owes.** Each person sees this on their own machine before
/// they say yes, and says yes only if the other person reads the same six
/// digits. Two matching codes mean the two offers that reached each machine
/// are the two that were made, and nobody on the network stood in between.
/// A surface that hides the code, or shows it only on one side, has removed the
/// only check ADR 0003 allows against an attacker present at the moment of
/// pairing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code(String);

impl Code {
    /// The digits, as a person reads them out.
    #[must_use]
    pub fn digits(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use super::{Keying, Offer, PairingKey, Transcript};
    use crate::machine::MachineId;
    use crate::permitting::MayAskIts;
    use crate::refusing::NotNearby;

    /// The two machines in every test here.
    fn one() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// The other.
    fn another() -> MachineId {
        MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
    }

    /// The transcript of one ordinary proposal.
    fn transcript(a: &Offer, b: &Offer) -> Transcript {
        Transcript::of(
            &one(),
            &another(),
            a,
            b,
            &[MayAskIts::Models],
            Duration::from_secs(86_400),
        )
    }

    /// **Two machines that sent each other only their offers hold the same
    /// key**, which is the whole of what X25519 is for.
    #[test]
    fn two_sides_that_exchanged_offers_hold_the_same_key() {
        let a = Keying::fresh().unwrap();
        let b = Keying::fresh().unwrap();
        let (a_offer, b_offer) = (a.offer().clone(), b.offer().clone());
        let on_a = a.agreed(&b_offer, &transcript(&a_offer, &b_offer)).unwrap();
        let on_b = b.agreed(&a_offer, &transcript(&a_offer, &b_offer)).unwrap();
        assert_eq!(on_a, on_b);
    }

    /// **A third machine that read both offers off the wire holds a different
    /// key**, however it combines what it read.
    #[test]
    fn a_machine_that_read_both_offers_holds_a_different_key() {
        let a = Keying::fresh().unwrap();
        let b = Keying::fresh().unwrap();
        let stranger = Keying::fresh().unwrap();
        let (a_offer, b_offer) = (a.offer().clone(), b.offer().clone());
        let on_a = a.agreed(&b_offer, &transcript(&a_offer, &b_offer)).unwrap();
        let guessed = stranger
            .agreed(&b_offer, &transcript(&a_offer, &b_offer))
            .unwrap();
        assert_ne!(on_a, guessed);
    }

    /// **The terms are part of the key.** The same two offers on different
    /// terms — a longer duration, a wider list — derive a different key, so a
    /// proposal altered in transit pairs nothing that works.
    #[test]
    fn the_same_offers_on_different_terms_derive_a_different_key() {
        let a = Keying::fresh().unwrap();
        let b = Keying::fresh().unwrap();
        let (a_offer, b_offer) = (a.offer().clone(), b.offer().clone());
        let as_agreed = a.agreed(&b_offer, &transcript(&a_offer, &b_offer)).unwrap();
        let altered = Transcript::of(
            &one(),
            &another(),
            &a_offer,
            &b_offer,
            &[MayAskIts::Models, MayAskIts::Workspace],
            Duration::from_secs(86_400),
        );
        let on_other_terms = b.agreed(&a_offer, &altered).unwrap();
        assert_ne!(as_agreed, on_other_terms);
    }

    /// Two keyings made one after the other differ, which is what makes an
    /// offer unlinkable across pairings.
    #[test]
    fn two_keyings_made_on_this_machine_are_not_the_same() {
        assert_ne!(
            Keying::fresh().unwrap().offer(),
            Keying::fresh().unwrap().offer()
        );
    }

    /// An offer is read back as it was written, and anything else is refused.
    #[test]
    fn an_offer_written_here_is_read_back_and_anything_else_is_refused() {
        let offer = Keying::fresh().unwrap().offer().clone();
        assert_eq!(Offer::read(&offer.said()).unwrap(), offer);
        for not_one in ["", "disan-laptop", &offer.said().to_uppercase(), "00"] {
            assert!(
                matches!(Offer::read(not_one).unwrap_err(), NotNearby::NotAnOffer(_)),
                "`{not_one}` was read as an offer"
            );
        }
    }

    /// **The code is the same on both sides and different for a substituted
    /// offer**, which is what two people comparing it can tell.
    #[test]
    fn the_code_matches_on_both_sides_and_changes_when_an_offer_is_substituted() {
        let a = Keying::fresh().unwrap().offer().clone();
        let b = Keying::fresh().unwrap().offer().clone();
        let in_between = Keying::fresh().unwrap().offer().clone();
        let on_a = transcript(&a, &b).code();
        let on_b = transcript(&a, &b).code();
        assert_eq!(on_a, on_b);
        assert_eq!(on_a.digits().len(), 6);
        assert!(on_a.digits().chars().all(|digit| digit.is_ascii_digit()));
        assert_ne!(on_a, transcript(&a, &in_between).code());
        assert_ne!(on_a, transcript(&in_between, &b).code());
    }

    /// The key prints as nothing, so it cannot reach a log by way of `{:?}`.
    #[test]
    fn the_key_prints_as_nothing() {
        let key = PairingKey([7; 32]);
        assert_eq!(format!("{key:?}"), "PairingKey(..)");
    }
}
