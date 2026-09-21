//! The seam between what the record says and whether it can be put back.
//!
//! `alo-keeping-up` decides ★ *undo what the agent did* and names no record;
//! `alo-record` keeps what happened and decides nothing about undoing. One of
//! them has to read the other, and this is the one place it is done — so a
//! reviewer checking that the table in
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) point 1
//! is the table the machine actually applies has one file to read rather than
//! every surface that ever shows a record.
//!
//! # Nothing here puts anything back
//!
//! There is no rename, no copy and no removal in this file, and there is not
//! going to be one: *what can be undone is undone through the mechanism that
//! made it undoable, and never by a second implementation guessing at
//! inverses* is the plan's constraint and the decision's point. What is here
//! is the **answer** — this can be put back, or it cannot and here is why —
//! and on every machine alo OS installs today the answer is the second one.
//!
//! # Why this machine keeps nothing, and what would change it
//!
//! ADR 0045 chose the base's own read-only copies of a person's home, one
//! taken before a changing turn and one after. Four things have to land before
//! a machine has them, and none of them is this plan's:
//!
//! - the installer formats with a filesystem that has them at all — the
//!   installer plan's, and it has to land **before the certified laptop is
//!   installed**, because the filesystem is chosen at install and cannot be
//!   converted;
//! - each person's home is a subvolume of its own — the accounts work's;
//! - a turn that runs a change verb is bracketed by two of them — lane A's
//!   `alo-turn`;
//! - and taking one needs a privilege `alo-agentd` does not hold — the broker
//!   plan's.
//!
//! Until all four exist here, [`what_this_machine_kept`] answers
//! [`WhatWasKept::NothingOnThisMachine`], every change an agent made says *not
//! yet on this machine*, and that is a true sentence rather than a stub. The
//! day the bracket lands, this is the one function that changes.

use alo_capability::Effect;
use alo_keeping_up::{AnUndo, NotUndoable, WhatWasDone, WhatWasKept};
use alo_record::{Entry, Happened};

/// What one entry of the record says happened, as far as undoing cares.
///
/// ADR 0045 point 1's table, as an exhaustive match: a kind of entry added to
/// the record and not given an answer here is a build that fails rather than a
/// change an agent made that quietly answers the wrong thing.
///
/// The verbs are read by **name and effect together**. A verb this does not
/// know still lands somewhere honest — a new read verb only looked, a new
/// change verb is not one of the changes there is a road back for — which is
/// what stops the table starting to lie the day a verb arrives.
#[must_use]
pub fn what_was_done(entry: &Entry) -> WhatWasDone {
    match entry.happened() {
        // What ran is the only entry a verb's name can be read off, and the
        // effect beside it is the record's own answer rather than a guess.
        Happened::Ran { what, .. } => {
            WhatWasDone::of_a_verb(what.verb().as_str(), what.effect() == Effect::Change)
        }
        // Nothing happened: a call that was refused, one that never formed, a
        // question never put anywhere, an egress the policy held back, a turn
        // with no boundary to run inside, a request the broker refused, and a
        // turn the machine slept through — whose work, if it did any, has its
        // own entries and is answered by them.
        Happened::Stopped { .. }
        | Happened::TurnedAway { .. }
        | Happened::NeverPutAnywhere { .. }
        | Happened::HeldBack { .. }
        | Happened::NotBounded { .. }
        | Happened::SleptThrough { .. }
        | Happened::Brokered {
            refused: Some(_), ..
        } => WhatWasDone::NothingHappened,
        // The broker handed one of its own verbs on: a printer, a network, a
        // drive. Not a file of the person's, so not one of the three.
        Happened::Brokered { refused: None, .. } => WhatWasDone::ChangedSomethingElse,
        // It left this machine, whatever carried it.
        Happened::Left { .. } => WhatWasDone::ItLeft,
        // A model was told it, here or for a machine this one is paired with.
        Happened::AnsweredHere { .. } | Happened::AnsweredForAnotherMachine { .. } => {
            WhatWasDone::AModelWasTold
        }
        // Nobody's agent did it: the machine starting on another build, going
        // back to the build before, a pairing kept, a workspace opened, grants
        // it could not read again, an errand of its own, an undo already
        // done — and the machine letting go of what it was keeping, which is
        // housekeeping on a timer and not anything an agent may ask for
        // (ADR 0045's seventh term).
        Happened::Updated { .. }
        | Happened::RolledBack { .. }
        | Happened::Paired { .. }
        | Happened::WorkspaceOpened { .. }
        | Happened::GrantsNotReadAgain { .. }
        | Happened::LeftOnItsOwn { .. }
        | Happened::LetGo { .. }
        | Happened::Undone { .. } => WhatWasDone::NotAnAgents,
    }
}

/// What this machine kept of a person's files either side of a changing turn.
///
/// [`WhatWasKept::NothingOnThisMachine`] on every machine alo OS installs
/// today, for the four reasons this file's own documentation sets out. It is
/// the plain truth rather than a fault or a stub: the machine says what it
/// cannot do instead of offering something that would fail halfway through a
/// person's own folder.
#[must_use]
pub fn what_this_machine_kept() -> WhatWasKept {
    WhatWasKept::NothingOnThisMachine
}

/// Whether what one entry of the record says happened can be put back on this
/// machine, and what putting it back would put back.
///
/// The two questions in the order they have to be asked: *could this ever be
/// undone*, which is about what was done, and then *can it be undone here*,
/// which is about what this machine kept. Both are answered **before** an
/// offer is made, because an undo offered and then failing halfway is a
/// person's files left between two states nobody chose.
///
/// # Errors
/// [`NotUndoable`], the reason, which is the deliverable rather than a
/// consolation for one. `alo_keeping_up::NotUndoable::said` is the sentence a
/// person reads instead of an offer.
pub fn putting_back(entry: &Entry) -> Result<AnUndo, NotUndoable> {
    AnUndo::offered(what_was_done(entry), &what_this_machine_kept())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::{Duration, SystemTime};

    use alo_egress::{Destination, Errand, Indicator, OnItsOwn};
    use alo_record::brokered::AtTheBroker;

    use super::*;

    /// A fixed moment, so that everything about time here is arithmetic.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// **What no agent did is not an agent's to undo.**
    ///
    /// The machine starting on another build, going back to the build before, a
    /// pairing kept, a workspace opened and grants it could not read again are
    /// each the machine's own or the person's own. Going back to the build
    /// before is a person's act with a sentence of its own
    /// (`crate::yesterday`), and answering it as an undo would put two
    /// different acts behind one word.
    #[test]
    fn what_no_agent_did_is_not_an_agents_to_undo() {
        for not_an_agents in [
            Entry::updated("sha256:a", "sha256:b", noon()),
            Entry::rolled_back("sha256:b", "sha256:a", noon()),
            Entry::paired("the studio", noon()),
            Entry::a_workspace_was_opened(
                "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
                "10.0.0.2:8443",
                noon(),
            ),
            Entry::the_grants_were_not_read_again("the grants could not be read", noon()),
        ] {
            assert_eq!(what_was_done(&not_an_agents), WhatWasDone::NotAnAgents);
            assert_eq!(
                putting_back(&not_an_agents).unwrap_err(),
                NotUndoable::NotAnAgents
            );
        }
    }

    /// **An errand of the machine's own is nobody's agent's either**, though it
    /// left the machine: ADR 0045's table puts it with what no agent did, and
    /// *what left this machine* is about what an agent sent.
    #[test]
    fn an_errand_of_the_machines_own_is_not_an_agents_either() {
        let underway = Indicator::default().beginning_on_its_own(
            OnItsOwn::for_(
                Errand::CheckingForAnUpdate,
                Destination::at("updates.alo.example").unwrap(),
            ),
            noon(),
        );
        let errand = Entry::left_on_its_own(&underway);
        assert_eq!(what_was_done(&errand), WhatWasDone::NotAnAgents);
        assert_eq!(putting_back(&errand).unwrap_err(), NotUndoable::NotAnAgents);
    }

    /// **What the broker refused changed nothing, and what it handed on changed
    /// something that is not a file of the person's** — two answers, because a
    /// refusal and a printer that was really set up are not one fact.
    #[test]
    fn the_broker_is_answered_by_whether_it_handed_anything_on() {
        let refused = Entry::refused_by_the_broker(
            Some("add_printer"),
            None,
            AtTheBroker::NotApproved,
            noon(),
        );
        assert_eq!(what_was_done(&refused), WhatWasDone::NothingHappened);
        assert_eq!(
            putting_back(&refused).unwrap_err(),
            NotUndoable::NothingHappened
        );

        let handed_on = Entry::handed_on_by_the_broker("add_printer", 7, noon());
        assert_eq!(what_was_done(&handed_on), WhatWasDone::ChangedSomethingElse);
        assert_eq!(
            putting_back(&handed_on).unwrap_err(),
            NotUndoable::NotOneOfTheChangesPutBack
        );
    }

    /// **This machine keeps nothing of what a person's files were**, and there
    /// is one place that says so rather than every surface deciding it for
    /// itself.
    #[test]
    fn this_machine_keeps_nothing_of_what_the_files_were() {
        assert_eq!(what_this_machine_kept(), WhatWasKept::NothingOnThisMachine);
    }
}
