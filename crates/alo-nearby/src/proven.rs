//! Whether a message that says it came from a paired machine did — decided in
//! one place, before anything else on this machine is asked.
//!
//! ADR 0031. [`Proven::checked`] is the receiving side's whole judgement of a
//! [`Proof`], in this order:
//!
//! 1. **Is a pairing with the machine it names standing now?** Asked of this
//!    machine's own pairings at the moment — so a pairing revoked or expired
//!    stops proofs at that moment, not at the next restart, because the key is
//!    on the row and the row is gone.
//! 2. **Is it for this machine?** A proof made for another machine is refused
//!    even though its tag would not verify here anyway, so that the refusal
//!    says the true thing.
//! 3. **Does the tag verify?** With the pairing's key, over what arrived, in
//!    constant time. This is the check that refuses a stranger presenting a
//!    paired machine's identity: an identity is read off the wire by design,
//!    and confers nothing without the key.
//! 4. **Is it from now?** Within [`WHILE_A_PROOF_STANDS`] of this machine's
//!    clock, either way.
//! 5. **Has it been seen?** A tag accepted within that window is not accepted
//!    again.
//!
//! # What a refusal here is, and what it is not
//!
//! Nothing on this machine was consulted for any of the five — no grant, no
//! verb list, no person — so nothing is written in the record about a proof
//! that failed, exactly as nothing is written when an unpaired machine offers
//! inference. A record that named a machine on the strength of a proof that
//! did not verify would be the thing this file exists to prevent.

use std::time::SystemTime;

use crate::machine::MachineId;
use crate::pairing::Pairings;
use crate::proof::Proof;
use crate::replaying::{Seen, WHILE_A_PROOF_STANDS};
use crate::words;

/// Why a proof was not accepted.
///
/// **No `Display`**, for the reason [`crate::NotPaired`] has none: the road to
/// words is [`said`](NotProven::said), in the language of the person reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NotProven {
    /// No pairing with the machine the proof names stands now.
    ///
    /// One arm for a machine merely discovered, one whose pairing ended, and
    /// one whose pairing was revoked — the same arm and the same sentence as
    /// [`crate::NotPaired::NotWithThatMachine`], for the same reason.
    NotWithThatMachine,
    /// The proof was made for a different machine than this one.
    NotForThisMachine,
    /// The tag does not verify with the key the pairing holds.
    ///
    /// The refusal this file exists for: whoever sent this had the identity and
    /// not the key.
    NotFromThatMachine,
    /// The moment in the proof is further from this machine's clock than a
    /// proof may be.
    NotNow,
    /// This exact proof has been accepted already.
    AlreadySeen,
}

impl NotProven {
    /// The sentence a person reads for this.
    #[must_use]
    pub const fn word(self) -> alo_strings::Word {
        match self {
            Self::NotWithThatMachine => words::NOT_PAIRED_WITH_THE_ONE_THAT_ASKED,
            Self::NotForThisMachine | Self::NotFromThatMachine => {
                words::NOT_FROM_THE_MACHINE_IT_NAMES
            }
            Self::NotNow => words::A_PROOF_FROM_ANOTHER_MOMENT,
            Self::AlreadySeen => words::A_PROOF_ALREADY_USED,
        }
    }

    /// This refusal, in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &alo_strings::Strings) -> alo_strings::Said {
        strings.say(&self.word().key(), &alo_strings::Filling::nothing())
    }
}

/// A message that proved it came from a paired machine.
///
/// Made only by [`checked`](Self::checked). Holding one is evidence that, at
/// `at`, this machine held a pairing with `machine` and the message carried a
/// proof only that pairing's key could make.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proven {
    /// The machine the message came from.
    machine: MachineId,
    /// The moment it was checked, on this machine's clock.
    at: SystemTime,
}

impl Proven {
    /// Judge a proof, as `here`, against this machine's own pairings, at `now`.
    ///
    /// `about` is exactly the bytes the message carried, and `seen` is what
    /// this machine remembers having accepted.
    ///
    /// # Errors
    ///
    /// [`NotProven`], in the order the module's own documentation gives, and
    /// nothing is remembered in `seen` unless the proof was accepted.
    pub fn checked(
        pairings: &Pairings,
        here: &MachineId,
        proof: &Proof,
        about: &[u8],
        now: SystemTime,
        seen: &mut Seen,
    ) -> Result<Self, NotProven> {
        let Some(pairing) = pairings.with(proof.from(), now) else {
            return Err(NotProven::NotWithThatMachine);
        };
        if proof.to() != here {
            return Err(NotProven::NotForThisMachine);
        }
        if !proof.is_by(pairing, about) {
            return Err(NotProven::NotFromThatMachine);
        }
        let apart = match now.duration_since(proof.at()) {
            Ok(ahead) => ahead,
            Err(behind) => behind.duration(),
        };
        if apart > WHILE_A_PROOF_STANDS {
            return Err(NotProven::NotNow);
        }
        if !seen.remember(proof.tag(), proof.at(), now) {
            return Err(NotProven::AlreadySeen);
        }
        Ok(Self {
            machine: proof.from().clone(),
            at: now,
        })
    }

    /// The machine the message came from.
    #[must_use]
    pub const fn machine(&self) -> &MachineId {
        &self.machine
    }

    /// The moment it was checked.
    #[must_use]
    pub const fn at(&self) -> SystemTime {
        self.at
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use super::{NotProven, Proven};
    use crate::pairing::Pairings;
    use crate::proof::Proof;
    use crate::replaying::{Seen, WHILE_A_PROOF_STANDS};
    use crate::testing::{a_moment, a_stranger, paired, paired_between, reception, studio};

    /// The studio's pairings, holding its row about reception, and reception's
    /// own row.
    fn at_the_studio() -> (Pairings, crate::pairing::Pairing) {
        let (on_reception, on_studio) = paired();
        let mut pairings = Pairings::none();
        pairings.keep(on_studio);
        (pairings, on_reception)
    }

    /// **A proof from the paired machine is accepted**, and says which machine.
    #[test]
    fn a_proof_from_the_paired_machine_is_accepted() {
        let (pairings, on_reception) = at_the_studio();
        let proof = Proof::made(&on_reception, &reception(), b"a question", a_moment());
        let proven = Proven::checked(
            &pairings,
            &studio(),
            &proof,
            b"a question",
            a_moment(),
            &mut Seen::nothing(),
        )
        .unwrap();
        assert_eq!(proven.machine(), &reception());
        assert_eq!(proven.at(), a_moment());
    }

    /// **A stranger presenting reception's identity is refused**, with a key
    /// of its own or with none, and the refusal is about the key.
    #[test]
    fn a_stranger_presenting_a_paired_machines_identity_is_refused() {
        let (pairings, _) = at_the_studio();
        // A stranger holding a row naming the studio — a pairing the studio's
        // person never made, so the studio holds no key for it — who read
        // reception's identity off the wire and presents it.
        let (strangers_row, _) = paired_between(a_stranger(), studio());
        let forged = Proof::made(&strangers_row, &reception(), b"a question", a_moment());
        assert_eq!(forged.from(), &reception());
        assert_eq!(forged.to(), &studio());
        let refused = Proven::checked(
            &pairings,
            &studio(),
            &forged,
            b"a question",
            a_moment(),
            &mut Seen::nothing(),
        )
        .unwrap_err();
        assert_eq!(refused, NotProven::NotFromThatMachine);
    }

    /// A proof over one message does not stand for another: the bytes were
    /// altered, and the tag says so.
    #[test]
    fn a_message_altered_in_transit_is_refused() {
        let (pairings, on_reception) = at_the_studio();
        let proof = Proof::made(
            &on_reception,
            &reception(),
            b"may the tenant sublet?",
            a_moment(),
        );
        let refused = Proven::checked(
            &pairings,
            &studio(),
            &proof,
            b"may the tenant keep a dog?",
            a_moment(),
            &mut Seen::nothing(),
        )
        .unwrap_err();
        assert_eq!(refused, NotProven::NotFromThatMachine);
    }

    /// **A replayed proof is refused**, and nothing about it is remembered
    /// twice.
    #[test]
    fn a_proof_replayed_from_an_earlier_exchange_is_refused() {
        let (pairings, on_reception) = at_the_studio();
        let mut seen = Seen::nothing();
        let proof = Proof::made(&on_reception, &reception(), b"a question", a_moment());
        assert!(
            Proven::checked(
                &pairings,
                &studio(),
                &proof,
                b"a question",
                a_moment(),
                &mut seen
            )
            .is_ok()
        );
        let a_little_later = a_moment() + Duration::from_secs(30);
        let refused = Proven::checked(
            &pairings,
            &studio(),
            &proof,
            b"a question",
            a_little_later,
            &mut seen,
        )
        .unwrap_err();
        assert_eq!(refused, NotProven::AlreadySeen);
        assert_eq!(seen.how_many(), 1);
    }

    /// **A proof from a pairing since revoked is refused at the moment**, and
    /// so is one from a pairing that has ended.
    #[test]
    fn a_proof_from_a_pairing_since_revoked_or_expired_is_refused_at_the_moment() {
        let (mut pairings, on_reception) = at_the_studio();
        let proof = Proof::made(&on_reception, &reception(), b"a question", a_moment());
        assert!(
            Proven::checked(
                &pairings,
                &studio(),
                &proof,
                b"a question",
                a_moment(),
                &mut Seen::nothing()
            )
            .is_ok()
        );

        let ended = a_moment() + Duration::from_secs(86_400);
        let still_fresh = Proof::made(&on_reception, &reception(), b"a question", ended);
        assert_eq!(
            Proven::checked(
                &pairings,
                &studio(),
                &still_fresh,
                b"a question",
                ended,
                &mut Seen::nothing()
            )
            .unwrap_err(),
            NotProven::NotWithThatMachine
        );

        assert!(pairings.revoke(&reception()));
        let again = Proof::made(
            &on_reception,
            &reception(),
            b"a question",
            a_moment() + Duration::from_secs(1),
        );
        assert_eq!(
            Proven::checked(
                &pairings,
                &studio(),
                &again,
                b"a question",
                a_moment() + Duration::from_secs(1),
                &mut Seen::nothing()
            )
            .unwrap_err(),
            NotProven::NotWithThatMachine
        );
    }

    /// A proof for another machine is refused as such, even from a paired one.
    #[test]
    fn a_proof_made_for_another_machine_is_refused() {
        let (pairings, on_reception) = at_the_studio();
        let proof = Proof::made(&on_reception, &reception(), b"a question", a_moment());
        let refused = Proven::checked(
            &pairings,
            &a_stranger(),
            &proof,
            b"a question",
            a_moment(),
            &mut Seen::nothing(),
        )
        .unwrap_err();
        assert_eq!(refused, NotProven::NotForThisMachine);
    }

    /// A proof from too far from this machine's clock is refused, in either
    /// direction, and one just inside the window stands.
    #[test]
    fn a_proof_from_another_moment_is_refused() {
        let (pairings, on_reception) = at_the_studio();
        let proof = Proof::made(&on_reception, &reception(), b"a question", a_moment());
        let too_late = a_moment() + WHILE_A_PROOF_STANDS + Duration::from_secs(1);
        let too_early = a_moment() - WHILE_A_PROOF_STANDS - Duration::from_secs(1);
        for elsewhere in [too_late, too_early] {
            assert_eq!(
                Proven::checked(
                    &pairings,
                    &studio(),
                    &proof,
                    b"a question",
                    elsewhere,
                    &mut Seen::nothing()
                )
                .unwrap_err(),
                NotProven::NotNow
            );
        }
        assert!(
            Proven::checked(
                &pairings,
                &studio(),
                &proof,
                b"a question",
                a_moment() + WHILE_A_PROOF_STANDS,
                &mut Seen::nothing()
            )
            .is_ok()
        );
    }

    /// Every refusal has a sentence.
    #[test]
    fn every_refusal_can_be_said_to_a_person() {
        for refused in [
            NotProven::NotWithThatMachine,
            NotProven::NotForThisMachine,
            NotProven::NotFromThatMachine,
            NotProven::NotNow,
            NotProven::AlreadySeen,
        ] {
            assert!(!refused.word().says().is_empty(), "{refused:?}");
        }
    }
}
