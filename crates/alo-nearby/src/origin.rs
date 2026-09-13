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
//! # A pairing is standing, not a permission to act
//!
//! [`Origin::paired`] asks one question of this machine's pairings — is there
//! one with that machine, standing now — and asks nothing about what it
//! permits. [`crate::MayAskIts`] has no arm for *run a verb on that machine*
//! and says why it never will: pairing lets a machine **ask**, and a verb it
//! asks for is decided by the grants **this** machine's person made, on this
//! machine, to that machine. The pairing is what makes the asker somebody this
//! machine can name; the grant is what decides the answer. Neither stands in
//! for the other, and a machine with a pairing and no grant is refused at every
//! door exactly as an agent with no grant is.
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
//! only the second is what the pairings are asked about.
//!
//! # What is refused here, and what is not written down
//!
//! A machine that was merely discovered, one whose pairing has ended, and one
//! whose pairing was revoked all reach [`Origin::paired`] and are refused with
//! one arm, [`NotPaired::NotWithThatMachine`], for the reason `alo-asking`'s
//! corridor gives: telling a caller *which* kind of not-paired it is would be
//! telling it how to become paired. Nothing on this machine was consulted for
//! that refusal — no grant, no verb list, no person — so nothing is written in
//! the record about it, exactly as nothing is written when this machine
//! declines to *ask* an unpaired one. Whatever carries verbs between machines
//! is where a knock from a stranger is counted, the way `alo-agentd` counts a
//! knock from a caller it does not know.

use std::time::SystemTime;

use crate::machine::MachineId;
use crate::pairing::{NotPaired, Pairings};

/// What a grantee naming a paired machine begins with, so that it can never be
/// the name of an agent on this machine.
///
/// One spelling, in one place: whatever makes a grant to a machine and whatever
/// asks the grants about one have to agree on it, and a daemon that refuses a
/// local agent this prefix has one string to refuse.
const A_MACHINE: &str = "machine:";

/// A paired machine, as a place a verb may arrive from.
///
/// Made only by [`paired`](Self::paired), which asks this machine's own
/// pairings. There is no other constructor, so holding one is evidence that two
/// people agreed and that their agreement had not ended at the moment it was
/// made — and every door a verb then goes through asks the pairings again for
/// its own moment, because an agreement can end during a turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    /// The other machine, as the pairing knows it.
    machine: MachineId,
    /// What this machine's person called it when they paired. Its identity,
    /// spelt out, if they called it nothing.
    called: String,
}

impl Origin {
    /// The machine a verb arrived from, if this machine is paired with it now.
    ///
    /// `now` is passed rather than read, as it is everywhere a pairing is
    /// asked about, so that *at the moment of arriving* is a moment the caller
    /// names. `called` is the name this machine's person gave the other
    /// machine; it decides nothing and is kept for what a person reads.
    ///
    /// # Errors
    ///
    /// [`NotPaired::NotWithThatMachine`] when no pairing with that machine
    /// stands at `now` — a machine that was merely discovered, one whose
    /// pairing has ended, and one whose pairing was revoked, with no arm that
    /// distinguishes them.
    pub fn paired(
        pairings: &Pairings,
        machine: &MachineId,
        called: &str,
        now: SystemTime,
    ) -> Result<Self, NotPaired> {
        if !pairings.paired_with(machine, now) {
            return Err(NotPaired::NotWithThatMachine);
        }
        let called = called.trim();
        Ok(Self {
            machine: machine.clone(),
            called: if called.is_empty() {
                machine.as_str().to_owned()
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
    use std::time::{Duration, SystemTime};

    use super::Origin;
    use crate::deliberating::{Deliberating, Proposal, Side};
    use crate::machine::MachineId;
    use crate::pairing::{NotPaired, Pairings};
    use crate::permitting::MayAskIts;

    /// This machine — the one with the GPU in it, being asked.
    fn here() -> MachineId {
        MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
    }

    /// The machine down the corridor, asking.
    fn the_reception() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// A machine nobody paired with.
    fn a_stranger() -> MachineId {
        MachineId::read("99998888777766665555444433332222").unwrap()
    }

    /// A moment to reason from.
    fn a_moment() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// A pairing two people made, for a day, permitting the reception machine
    /// to ask this one's models.
    fn paired_for_a_day() -> Pairings {
        let mut pairings = Pairings::none();
        pairings.keep(
            Deliberating::of(
                Proposal::checked(
                    the_reception(),
                    here(),
                    &[MayAskIts::Models],
                    Duration::from_secs(86_400),
                )
                .unwrap(),
            )
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(Side::TheOneAsked, a_moment())
            .unwrap(),
        );
        pairings
    }

    /// **A paired machine is an origin**, named as the person named it, and
    /// its principal is its identity and not that name.
    #[test]
    fn a_paired_machine_is_an_origin_named_as_the_person_named_it() {
        let origin = Origin::paired(
            &paired_for_a_day(),
            &the_reception(),
            "  the reception machine ",
            a_moment(),
        )
        .unwrap();
        assert_eq!(origin.machine(), &the_reception());
        assert_eq!(origin.called(), "the reception machine");
        assert_eq!(
            origin.principal(),
            "machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0"
        );
        assert!(Origin::names_a_machine(&origin.principal()));
        assert!(!origin.principal().contains("reception"));
    }

    /// **A machine nobody paired with is no origin**, however present it is on
    /// the network. There is no constructor that skips the pairing.
    #[test]
    fn a_machine_nobody_paired_with_is_no_origin() {
        let refused = Origin::paired(
            &Pairings::none(),
            &the_reception(),
            "the reception machine",
            a_moment(),
        )
        .unwrap_err();
        assert_eq!(refused, NotPaired::NotWithThatMachine);

        let refused = Origin::paired(
            &paired_for_a_day(),
            &a_stranger(),
            "somebody's laptop",
            a_moment(),
        )
        .unwrap_err();
        assert_eq!(refused, NotPaired::NotWithThatMachine);
    }

    /// **A pairing that has ended makes no origin, and neither does one that was
    /// revoked** — and it is the same refusal as never having paired, because
    /// telling a caller which kind of not-paired it is would be telling it how
    /// to become paired.
    #[test]
    fn a_pairing_that_has_ended_or_was_revoked_makes_no_origin() {
        let a_week_later = a_moment() + Duration::from_secs(7 * 86_400);
        let ended = Origin::paired(
            &paired_for_a_day(),
            &the_reception(),
            "the reception machine",
            a_week_later,
        )
        .unwrap_err();
        assert_eq!(ended, NotPaired::NotWithThatMachine);

        let mut pairings = paired_for_a_day();
        assert!(pairings.revoke(&the_reception()));
        let revoked = Origin::paired(
            &pairings,
            &the_reception(),
            "the reception machine",
            a_moment(),
        )
        .unwrap_err();
        assert_eq!(revoked, NotPaired::NotWithThatMachine);
    }

    /// **What a pairing permits is not asked here.** A pairing for the workspace
    /// alone still names the machine: the pairing is what makes the asker
    /// somebody this machine can name, and the grants on this machine are what
    /// decide any verb it asks for.
    #[test]
    fn a_pairing_for_anything_names_the_machine_and_permits_no_verb_by_itself() {
        let mut pairings = Pairings::none();
        pairings.keep(
            Deliberating::of(
                Proposal::checked(
                    the_reception(),
                    here(),
                    &[MayAskIts::Workspace],
                    Duration::from_secs(86_400),
                )
                .unwrap(),
            )
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(Side::TheOneAsked, a_moment())
            .unwrap(),
        );
        let origin = Origin::paired(&pairings, &the_reception(), "reception", a_moment()).unwrap();
        assert_eq!(origin.called(), "reception");
        // And nothing on the origin is, or answers, a permission: what it
        // offers is a name to make a grant out to.
        assert!(!pairings.permits(&the_reception(), MayAskIts::Models, a_moment()));
    }

    /// A machine the person called nothing is named by its identity, because a
    /// sentence with an empty name in it says nothing about who asked.
    #[test]
    fn a_machine_called_nothing_is_named_by_its_identity() {
        let origin =
            Origin::paired(&paired_for_a_day(), &the_reception(), "   ", a_moment()).unwrap();
        assert_eq!(origin.called(), the_reception().as_str());
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
