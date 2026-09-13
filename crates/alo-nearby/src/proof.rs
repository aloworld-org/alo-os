//! What a message carries to show it came from the machine it says it did.
//!
//! ADR 0031. A [`Proof`] is made on the sending machine with the key its
//! pairing holds, over four things: which machine is sending, which is being
//! sent to, the moment, and a digest of what the message carries. The receiving
//! machine recomputes the same tag with the same key — `proven.rs` — and
//! anything that lacks the key, names another machine, was altered, or is from
//! another moment computes to something else.
//!
//! # No randomness in a proof
//!
//! The tag is HMAC over a moment and a body, so it is already unique per
//! message, and it is what the receiver remembers to refuse a replay. A nonce
//! would add randomness to every message and a failure path for a machine that
//! cannot draw any; ADR 0031 says why it is not here.
//!
//! # The wire spelling
//!
//! One line: `alo-os/1 <from> <to> <seconds> <nanoseconds> <tag>`, every field
//! lowercase hexadecimal or decimal, so that a header, a file and a test all
//! spell a proof one way and [`Proof::read`] refuses every other.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ring::hmac::{HMAC_SHA256, Key};

use crate::hexing;
use crate::machine::MachineId;
use crate::pairing::Pairing;
use crate::refusing::NotNearby;

/// How many bytes a tag is, being SHA-256's width.
pub(crate) const A_TAG: usize = 32;

/// What every proof begins with on the wire, and which version it speaks.
const ON_THE_WIRE: &str = "alo-os/1";

/// What the signed bytes begin with, so a tag over some other structure could
/// never be read as one over this.
const A_PROOF: &[u8] = b"alo-os proof v1\0";

/// A proof that a message came from a paired machine.
///
/// Made only by [`made`](Self::made), which takes a [`Pairing`] — and a
/// pairing in hand is evidence that two people agreed and that this machine
/// holds the key. There is no way to make one without the key, which is the
/// whole of what a stranger presenting a paired machine's identity lacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    /// The machine sending.
    from: MachineId,
    /// The machine being sent to.
    to: MachineId,
    /// When it was made, as the sender's clock had it.
    at: SystemTime,
    /// The tag over all of it.
    tag: [u8; A_TAG],
}

impl Proof {
    /// A proof, made on `here` for the machine `pairing` is with, over
    /// `about` — the exact bytes the message will carry — at `now`.
    ///
    /// `now` is passed rather than read, as everywhere in this crate, so a
    /// proof can be made about a moment a test names.
    #[must_use]
    pub fn made(pairing: &Pairing, here: &MachineId, about: &[u8], now: SystemTime) -> Self {
        let mut proof = Self {
            from: here.clone(),
            to: pairing.with().clone(),
            at: now,
            tag: [0; A_TAG],
        };
        let key = Key::new(HMAC_SHA256, pairing.key().bytes());
        let tag = ring::hmac::sign(&key, &proof.signed(about));
        proof.tag.copy_from_slice(tag.as_ref());
        proof
    }

    /// The bytes the tag is over, for the one file that makes tags and the one
    /// that checks them.
    pub(crate) fn signed(&self, about: &[u8]) -> Vec<u8> {
        let (seconds, nanoseconds) = split(self.at);
        let digest = ring::digest::digest(&ring::digest::SHA256, about);
        let mut over = Vec::with_capacity(128);
        over.extend_from_slice(A_PROOF);
        over.extend_from_slice(self.from.as_str().as_bytes());
        over.push(0);
        over.extend_from_slice(self.to.as_str().as_bytes());
        over.push(0);
        over.extend_from_slice(&seconds.to_be_bytes());
        over.extend_from_slice(&nanoseconds.to_be_bytes());
        over.extend_from_slice(digest.as_ref());
        over
    }

    /// The proof as it travels: one line.
    #[must_use]
    pub fn said(&self) -> String {
        let (seconds, nanoseconds) = split(self.at);
        format!(
            "{ON_THE_WIRE} {} {} {seconds} {nanoseconds} {}",
            self.from,
            self.to,
            hexing::said(&self.tag)
        )
    }

    /// A proof somebody sent, read back and checked for shape — and for nothing
    /// else: whether it is *true* is [`crate::Proven::checked`]'s.
    ///
    /// # Errors
    ///
    /// [`NotNearby::NotAProof`] for anything that is not exactly the line
    /// [`said`](Self::said) writes, and [`NotNearby::NotAMachineIdentity`] for
    /// a machine in it that is not one.
    pub fn read(said: &str) -> Result<Self, NotNearby> {
        let mut fields = said.trim().split(' ');
        let (Some(version), Some(from), Some(to), Some(seconds), Some(nanoseconds), Some(tag)) = (
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
        ) else {
            return Err(NotNearby::NotAProof("too few fields".to_owned()));
        };
        if fields.next().is_some() {
            return Err(NotNearby::NotAProof("too many fields".to_owned()));
        }
        if version != ON_THE_WIRE {
            return Err(NotNearby::NotAProof(format!(
                "`{version}` is not a version spoken here"
            )));
        }
        let seconds: u64 = seconds
            .parse()
            .map_err(|_| NotNearby::NotAProof("the seconds are not a number".to_owned()))?;
        let nanoseconds: u32 = nanoseconds
            .parse()
            .map_err(|_| NotNearby::NotAProof("the nanoseconds are not a number".to_owned()))?;
        if nanoseconds >= 1_000_000_000 {
            return Err(NotNearby::NotAProof(
                "more nanoseconds than a second holds".to_owned(),
            ));
        }
        let at = UNIX_EPOCH
            .checked_add(Duration::new(seconds, nanoseconds))
            .ok_or_else(|| NotNearby::NotAProof("a moment past the end of time".to_owned()))?;
        let mut bytes = [0_u8; A_TAG];
        if !hexing::read(tag, &mut bytes) {
            return Err(NotNearby::NotAProof(
                "the tag is not sixty-four hexadecimal characters".to_owned(),
            ));
        }
        Ok(Self {
            from: MachineId::read(from)?,
            to: MachineId::read(to)?,
            at,
            tag: bytes,
        })
    }

    /// The machine that says it is sending.
    #[must_use]
    pub const fn from(&self) -> &MachineId {
        &self.from
    }

    /// The machine it is for.
    #[must_use]
    pub const fn to(&self) -> &MachineId {
        &self.to
    }

    /// The moment it claims to be from.
    #[must_use]
    pub const fn at(&self) -> SystemTime {
        self.at
    }

    /// The tag, for the file that checks it and the one that remembers it.
    pub(crate) const fn tag(&self) -> &[u8; A_TAG] {
        &self.tag
    }

    /// Whether the tag is the one `pairing`'s key makes over `about`.
    ///
    /// Constant time, which is `ring`'s and not this file's. A `false` is a tag
    /// that does not verify and nothing more; what to say about it is the
    /// caller's.
    pub(crate) fn is_by(&self, pairing: &Pairing, about: &[u8]) -> bool {
        let key = Key::new(HMAC_SHA256, pairing.key().bytes());
        ring::hmac::verify(&key, &self.signed(about), &self.tag).is_ok()
    }
}

/// A moment as seconds and nanoseconds since 1970, which is how it travels.
///
/// A moment before 1970 is written as the epoch itself: a clock that far wrong
/// is refused by the receiver as being from another moment, which is true.
fn split(at: SystemTime) -> (u64, u32) {
    let since = at.duration_since(UNIX_EPOCH).unwrap_or_default();
    (since.as_secs(), since.subsec_nanos())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use super::Proof;
    use crate::refusing::NotNearby;
    use crate::testing::{a_moment, paired, reception, studio};

    /// A proof made at reception is one the studio's row verifies, and it is
    /// read back off the wire as it was written.
    #[test]
    fn a_proof_made_on_one_side_verifies_on_the_other_and_survives_the_wire() {
        let (on_reception, on_studio) = paired();
        let proof = Proof::made(&on_reception, &reception(), b"a question", a_moment());
        assert_eq!(proof.from(), &reception());
        assert_eq!(proof.to(), &studio());
        assert_eq!(proof.at(), a_moment());
        assert!(proof.is_by(&on_studio, b"a question"));

        let travelled = Proof::read(&proof.said()).unwrap();
        assert_eq!(travelled, proof);
        assert!(travelled.is_by(&on_studio, b"a question"));
    }

    /// A proof over one body does not verify over another: the message it is
    /// about is part of it.
    #[test]
    fn a_proof_is_about_the_bytes_it_was_made_over() {
        let (on_reception, on_studio) = paired();
        let proof = Proof::made(&on_reception, &reception(), b"a question", a_moment());
        assert!(!proof.is_by(&on_studio, b"a different question"));
        assert!(!proof.is_by(&on_studio, b""));
    }

    /// Two proofs over the same bytes at different moments differ, which is
    /// what lets the receiver remember one without refusing the other.
    #[test]
    fn two_proofs_at_different_moments_have_different_tags() {
        let (on_reception, _) = paired();
        let one = Proof::made(&on_reception, &reception(), b"a question", a_moment());
        // A microsecond rather than a nanosecond: Windows keeps a moment to a
        // hundred nanoseconds, and a nanosecond later is the same moment there.
        let later = Proof::made(
            &on_reception,
            &reception(),
            b"a question",
            a_moment() + Duration::from_micros(1),
        );
        assert_ne!(one.tag(), later.tag());
    }

    /// Everything that is not the line this file writes is refused by shape.
    #[test]
    fn what_is_not_a_proof_is_refused_rather_than_read() {
        let (on_reception, _) = paired();
        let good = Proof::made(&on_reception, &reception(), b"a question", a_moment()).said();
        let fields: Vec<&str> = good.split(' ').collect();
        assert_eq!(fields.len(), 6, "{good}");
        // The same line with one field replaced, by position.
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
            with(0, "Bearer"),
            with_a_field_missing,
            format!("{good} extra"),
            with(1, "disan-laptop"),
            with(3, "soon"),
            with(4, "1000000000"),
            with(4, "-1"),
            with(5, "abc"),
            good.to_uppercase(),
        ] {
            assert!(
                matches!(
                    Proof::read(&not_one).unwrap_err(),
                    NotNearby::NotAProof(_) | NotNearby::NotAMachineIdentity(_)
                ),
                "`{not_one}` was read as a proof"
            );
        }
    }
}
