//! The other end of the corridor: a paired machine, as a place a verb may
//! arrive from.
//!
//! [`crate::pairing`] is this machine's side of ADR 0003 when this machine
//! **asks**. This is its side when this machine is **asked** — when an agent on
//! machine A causes something to happen on machine B, and this is B:
//!
//! > B evaluates it against B's own grants and verb list, not A's; B's person
//! > approves any change, on B, from B's own approval surface; B records it,
//! > with A named as the origin.
//!
//! What this file decides is the first word of that: *who* is asking, in terms
//! this machine's grants can be made out to. Everything after it is `alo-turn`'s
//! and `alo-capability`'s, unchanged.
//!
//! # An origin is made from a proof, and from nothing else
//!
//! ADR 0031. [`Origin::proven`] takes a [`Proof`] and the bytes it is about,
//! and asks [`Proven::checked`] — the one judgement of whether a message came
//! from the machine it names — before it asks anything else. There is no
//! constructor that takes an identity alone: an identity is read off the wire
//! by design, and a stranger on the office WiFi presenting a paired machine's
//! identity reaches this door and is refused before any grant is asked, which
//! is the lateral movement ADR 0003 exists to refuse.
//!
//! # A pairing is standing, not a permission to act
//!
//! The proof establishes that a pairing with that machine stands now and that
//! the message is its; it establishes nothing about what the machine may do.
//! [`crate::MayAskIts`] has no arm for *run a verb on that machine* and says
//! why it never will: pairing lets a machine **ask**, and a verb it asks for is
//! decided by the grants **this** machine's person made, on this machine, to
//! that machine. The pairing is what makes the asker somebody this machine can
//! name; the grant is what decides the answer. Neither stands in for the other,
//! and a machine with a pairing and no grant is refused at every door exactly
//! as an agent with no grant is.
//!
//! # The principal is the identity, and the name is for people
//!
//! An [`Origin`] carries two things about the other machine and keeps them
//! apart. Its **identity** is what the pairing was made with, and it is what a
//! grant on this machine is made *to* — [`Origin::principal`] is that identity
//! spelt as a grantee, so a grant made for one machine can never permit
//! another, however a person later names either of them. Its **name** is what
//! this machine's person called it when they paired, and it is what that
//! person reads: in a refusal, on the approval surface once a shell resolves
//! it, and in the record's `origin`.
//!
//! The name arrives with the identity and is checked against nothing, which is
//! why it decides nothing: a caller that handed over the wrong name would
//! mislabel a record, and a caller that could hand over the wrong *identity*
//! would be choosing whose grants to ask. Only the second is an authority, and
//! only the second is what the proof is about.
//!
//! # What is refused here, and what is not written down
//!
//! Every refusal is [`NotProven`]'s: a machine merely discovered, one whose
//! pairing has ended, and one whose pairing was revoked are one arm, for the
//! reason `alo-asking`'s corridor gives — telling a caller *which* kind of
//! not-paired it is would be telling it how to become paired — and a proof
//! that does not verify, one for another machine, one from another moment and
//! one seen before are the rest. Nothing on this machine was consulted for any
//! of them — no grant, no verb list, no person — so nothing is written in the
//! record about it. Whatever carries verbs between machines is where a knock
//! from a stranger is counted, the way `alo-agentd` counts a knock from a
//! caller it does not know.

use std::time::SystemTime;

use crate::machine::MachineId;
use crate::pairing::Pairings;
use crate::proof::Proof;
use crate::proven::{NotProven, Proven};
use crate::replaying::Seen;

/// What a grantee naming a paired machine begins with, so that it can never be
/// the name of an agent on this machine.
///
/// One spelling, in one place: whatever makes a grant to a machine and whatever
/// asks the grants about one have to agree on it, and a daemon that refuses a
/// local agent this prefix has one string to refuse.
const A_MACHINE: &str = "machine:";

/// A paired machine, as a place a verb may arrive from.
///
/// Made only by [`proven`](Self::proven), which asks this machine's own
/// pairings and checks the proof the message carried. There is no other
/// constructor, so holding one is evidence that two people agreed, that their
/// agreement had not ended at the moment it was made, and that what arrived
/// was made with the key only that agreement holds — and every door a verb
/// then goes through asks the pairings again for its own moment, because an
/// agreement can end during a turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    /// The other machine, as the pairing knows it.
    machine: MachineId,
    /// What this machine's person called it when they paired. Its identity,
    /// spelt out, if they called it nothing.
    called: String,
}

impl Origin {
    /// The machine a verb arrived from, if its proof holds against this
    /// machine's pairings now.
    ///
    /// `here` is this machine; `about` is exactly the bytes that arrived with
    /// the proof; `seen` is what this machine remembers having accepted.
    /// `now` is passed rather than read, as it is everywhere a pairing is
    /// asked about, so that *at the moment of arriving* is a moment the caller
    /// names. `called` is the name this machine's person gave the other
    /// machine; it decides nothing and is kept for what a person reads.
    ///
    /// # Errors
    ///
    /// [`NotProven`], as [`Proven::checked`] answers it — including
    /// [`NotProven::NotWithThatMachine`] when no pairing with the machine the
    /// proof names stands at `now`.
    pub fn proven(
        pairings: &Pairings,
        here: &MachineId,
        proof: &Proof,
        about: &[u8],
        called: &str,
        now: SystemTime,
        seen: &mut Seen,
    ) -> Result<Self, NotProven> {
        let proven = Proven::checked(pairings, here, proof, about, now, seen)?;
        let called = called.trim();
        Ok(Self {
            machine: proven.machine().clone(),
            called: if called.is_empty() {
                proven.machine().as_str().to_owned()
            } else {
                called.to_owned()
            },
        })
    }

    /// The other machine, as the pairing knows it.
    #[must_use]
    pub const fn machine(&self) -> &MachineId {
        &self.machine
    }

    /// What this machine's person called the other machine when they paired.
    ///
    /// What a person reads, wherever this machine says something about a verb
    /// from there: never a machine's identity in the middle of a sentence.
    #[must_use]
    pub fn called(&self) -> &str {
        &self.called
    }

    /// The name a grant on this machine is made to, for this machine.
    ///
    /// Spelt from the identity and not from the name, so that a grant made to
    /// one machine permits that machine and no other — the name is a person's
    /// label, and two machines can be given the same one in succession. It
    /// begins with a prefix no agent on this machine is called by, so a grant
    /// to a machine and a grant to an agent cannot be the same row in a
    /// person's list.
    #[must_use]
    pub fn principal(&self) -> String {
        format!("{A_MACHINE}{}", self.machine.as_str())
    }

    /// Whether a grantee's name is one this machine makes out to a paired
    /// machine.
    ///
    /// For whatever names agents on this machine: an agent that arrived
    /// calling itself by a machine's name would be asking to be answered by
    /// that machine's grants, and this is the one check that refuses it.
    #[must_use]
    pub fn names_a_machine(grantee: &str) -> bool {
        grantee.trim().starts_with(A_MACHINE)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use super::Origin;
    use crate::pairing::{Pairing, Pairings};
    use crate::proof::Proof;
    use crate::proven::NotProven;
    use crate::replaying::Seen;
    use crate::testing::{a_moment, a_stranger, paired, paired_between, reception, studio};

    /// This machine's pairings — the studio's, holding its row about the
    /// reception machine — and reception's own row, to make proofs with.
    fn paired_for_a_day() -> (Pairings, Pairing) {
        let (on_reception, on_studio) = paired();
        let mut pairings = Pairings::none();
        pairings.keep(on_studio);
        (pairings, on_reception)
    }

    /// A proof from reception over a verb, at the moment.
    fn a_proof_from_reception(on_reception: &Pairing) -> Proof {
        Proof::made(on_reception, &reception(), b"list_folder", a_moment())
    }

    /// **A paired machine that proves itself is an origin**, named as the
    /// person named it, and its principal is its identity and not that name.
    #[test]
    fn a_paired_machine_that_proves_itself_is_an_origin_named_as_the_person_named_it() {
        let (pairings, on_reception) = paired_for_a_day();
        let origin = Origin::proven(
            &pairings,
            &studio(),
            &a_proof_from_reception(&on_reception),
            b"list_folder",
            "  the reception machine ",
            a_moment(),
            &mut Seen::nothing(),
        )
        .unwrap();
        assert_eq!(origin.machine(), &reception());
        assert_eq!(origin.called(), "the reception machine");
        assert_eq!(
            origin.principal(),
            "machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0"
        );
        assert!(Origin::names_a_machine(&origin.principal()));
        assert!(!origin.principal().contains("reception"));
    }

    /// **A stranger presenting reception's identity is no origin**, and is
    /// refused before any grant is asked: there is no constructor that takes
    /// an identity without a proof, and the proof does not verify.
    #[test]
    fn a_stranger_presenting_a_paired_machines_identity_is_no_origin() {
        let (pairings, _) = paired_for_a_day();
        let (strangers_row, _) = paired_between(a_stranger(), studio());
        let forged = Proof::made(&strangers_row, &reception(), b"list_folder", a_moment());
        let refused = Origin::proven(
            &pairings,
            &studio(),
            &forged,
            b"list_folder",
            "the reception machine",
            a_moment(),
            &mut Seen::nothing(),
        )
        .unwrap_err();
        assert_eq!(refused, NotProven::NotFromThatMachine);
    }

    /// **A machine nobody paired with is no origin**, however present it is on
    /// the network and whatever it holds.
    #[test]
    fn a_machine_nobody_paired_with_is_no_origin() {
        let (_, on_reception) = paired_for_a_day();
        let refused = Origin::proven(
            &Pairings::none(),
            &studio(),
            &a_proof_from_reception(&on_reception),
            b"list_folder",
            "the reception machine",
            a_moment(),
            &mut Seen::nothing(),
        )
        .unwrap_err();
        assert_eq!(refused, NotProven::NotWithThatMachine);
    }

    /// **A pairing that has ended makes no origin, and neither does one that was
    /// revoked** — and it is the same refusal as never having paired, because
    /// telling a caller which kind of not-paired it is would be telling it how
    /// to become paired.
    #[test]
    fn a_pairing_that_has_ended_or_was_revoked_makes_no_origin() {
        let (pairings, on_reception) = paired_for_a_day();
        let a_week_later = a_moment() + Duration::from_secs(7 * 86_400);
        let ended = Origin::proven(
            &pairings,
            &studio(),
            &Proof::made(&on_reception, &reception(), b"list_folder", a_week_later),
            b"list_folder",
            "the reception machine",
            a_week_later,
            &mut Seen::nothing(),
        )
        .unwrap_err();
        assert_eq!(ended, NotProven::NotWithThatMachine);

        let mut pairings = pairings;
        assert!(pairings.revoke(&reception()));
        let revoked = Origin::proven(
            &pairings,
            &studio(),
            &a_proof_from_reception(&on_reception),
            b"list_folder",
            "the reception machine",
            a_moment(),
            &mut Seen::nothing(),
        )
        .unwrap_err();
        assert_eq!(revoked, NotProven::NotWithThatMachine);
    }

    /// **A proof replayed makes no second origin.** The same bytes arriving
    /// again are refused, which is what stops somebody who recorded a verb
    /// off the wire from having it run twice.
    #[test]
    fn a_proof_replayed_makes_no_second_origin() {
        let (pairings, on_reception) = paired_for_a_day();
        let mut seen = Seen::nothing();
        let proof = a_proof_from_reception(&on_reception);
        assert!(
            Origin::proven(
                &pairings,
                &studio(),
                &proof,
                b"list_folder",
                "the reception machine",
                a_moment(),
                &mut seen
            )
            .is_ok()
        );
        let again = Origin::proven(
            &pairings,
            &studio(),
            &proof,
            b"list_folder",
            "the reception machine",
            a_moment() + Duration::from_secs(5),
            &mut seen,
        )
        .unwrap_err();
        assert_eq!(again, NotProven::AlreadySeen);
    }

    /// A machine the person called nothing is named by its identity, because a
    /// sentence with an empty name in it says nothing about who asked.
    #[test]
    fn a_machine_called_nothing_is_named_by_its_identity() {
        let (pairings, on_reception) = paired_for_a_day();
        let origin = Origin::proven(
            &pairings,
            &studio(),
            &a_proof_from_reception(&on_reception),
            b"list_folder",
            "   ",
            a_moment(),
            &mut Seen::nothing(),
        )
        .unwrap();
        assert_eq!(origin.called(), reception().as_str());
    }

    /// An agent on this machine cannot be called by a machine's name, and this
    /// is the one check that says which names those are.
    #[test]
    fn an_agents_name_is_never_a_machines() {
        assert!(!Origin::names_a_machine("@files"));
        assert!(!Origin::names_a_machine("the reception machine"));
        assert!(Origin::names_a_machine(
            "machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0"
        ));
        assert!(Origin::names_a_machine("  machine:anything"));
    }
}
