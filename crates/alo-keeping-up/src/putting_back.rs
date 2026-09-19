//! Putting back what an agent changed: what the machine has to have kept for
//! that to be possible, and the sentence the person approves when it is.
//!
//! [`crate::undoing`] answers *could this ever be undone*. This answers *can it
//! be undone here, now* — which is a different question with a different set of
//! honest refusals, because it depends on the machine rather than on the verb.
//!
//! # The road, and why it is not on this machine yet
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) chose
//! option A: the base's own read-only copies of the person's home, one taken
//! before a changing turn and one after, compared to learn exactly what the
//! turn changed and read to put it back. Making them is not this crate's and
//! not this plan's — the filesystem is the installer's, the home is the
//! accounts lane's, the bracket is `alo-turn`'s and the privilege is the
//! broker's, each named in the decision. **Until all four exist on a machine,
//! [`WhatWasKept::NothingOnThisMachine`] is the true answer there**, and every
//! change an agent made says *not yet on this machine* rather than pretending.
//!
//! That is a sentence rather than a stub: the machine is not claiming an undo
//! it does not have, and the day the bracket lands the only thing that changes
//! is which [`WhatWasKept`] is handed in.
//!
//! # What is decided here and what is measured elsewhere
//!
//! A [`Bracket`] is the **answer** to comparing the two sides of a turn: what
//! the turn changed, and which of those is no longer as the turn left it.
//! Comparing them is the base's own tools' work, in the privileged path, and
//! nothing here does it — *never a second implementation guessing at inverses*
//! is the plan's constraint and ADR 0045's point. What is decided here is what
//! that answer means: an undo, or a refusal, and which one.
//!
//! # An undo is the person's
//!
//! There is **no agent verb** here and none anywhere, which
//! `tests/undo_what_the_agent_did.rs` holds against every verb this machine
//! ships. An agent choosing which of a person's approvals to reverse is the
//! thing ADR 0045 point 2 forbids by name. [`AnUndo::said`] is the sentence a
//! person approves, and it waits for one approval exactly as a change does —
//! because an undo **is** a change.

use alo_strings::{Filling, Said, Strings};

use crate::undoing::{Change, NotUndoable, WhatWasDone};
use crate::words;

/// What separates one name from the next where several are put back at once.
const AND_THE_NEXT: &str = ", ";

/// What the machine kept of a person's files either side of the turn that
/// changed them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatWasKept {
    /// Nothing on this machine keeps what a person's files were before a turn.
    ///
    /// The answer on every machine alo OS installs today, and the reason every
    /// change verb says *not yet on this machine*: the disk is formatted
    /// without anything that can hold an earlier state of a folder, so there is
    /// nothing to put back from. It stops being the answer when the installer,
    /// the accounts lane, `alo-turn` and the broker have each landed their part
    /// of ADR 0045 — and not one moment earlier.
    NothingOnThisMachine,
    /// This machine can keep it, and for this one turn there was no room.
    ///
    /// ADR 0045's third accepted term: a machine too full still runs the turn,
    /// and says plainly that this one cannot be undone. A verb that refused to
    /// run for want of an undo would be a machine that stopped working to
    /// protect a feature.
    NotThisTurn,
    /// It was kept and has been let go, for one of the three reasons there are:
    /// [`crate::HowFarBack`] no longer reaches it, the disk needed the room, or
    /// the person forgot it ([`WhatWasKept::forgetting`]).
    ///
    /// One member for three causes on purpose. What a person can do about it is
    /// the same in all three — nothing — and a record that says *which* turns
    /// lost their undo is where the difference belongs (ADR 0045, second term).
    LetGo,
    /// Both sides of the turn are still kept, and this is what they say about
    /// it.
    EitherSideOfTheTurn(Bracket),
}

/// What comparing the two kept sides of a changing turn says about it.
///
/// Made by whoever compared them, which is not this crate. Two lists, and the
/// second is drawn from the first: what the turn changed, and which of those
/// is no longer as the turn left it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Bracket {
    /// What the turn changed, by name.
    changed: Vec<String>,
    /// Which of those is no longer as the turn left it.
    since: Vec<String>,
}

/// The moment a change was made, worded where the person reads it.
///
/// A moment is a date and a time, and how one is written is a language's
/// business rather than this crate's — which names no time at all
/// ([`crate::before`] has the reason). So it arrives already worded, from
/// whoever read the record, and the one thing held here is that it is **not
/// nothing**: an offer with a blank where the moment belongs is an offer a
/// person cannot check, and the answer is not to make it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhenItWasDone(String);

/// An undo, decided and ready to offer to the person whose files it puts back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnUndo {
    /// The change it puts back.
    change: Change,
    /// What it puts back, by name.
    puts_back: Vec<String>,
}

impl Bracket {
    /// What the two sides say: `changed` is what the turn changed, and `since`
    /// is which of those is no longer as it left it.
    ///
    /// Both are as the comparison found them. Nothing here trims or reconciles
    /// the lists, because a disagreement between them is whoever compared
    /// them having a bug, and quietly tidying it away is how that bug reaches a
    /// person's files.
    #[must_use]
    pub fn comparing(changed: Vec<String>, since: Vec<String>) -> Self {
        Self { changed, since }
    }

    /// What the turn changed, by name.
    #[must_use]
    pub fn changed(&self) -> &[String] {
        &self.changed
    }

    /// What of it is no longer as the turn left it.
    #[must_use]
    pub fn not_as_it_was_left(&self) -> &[String] {
        &self.since
    }
}

impl WhatWasKept {
    /// The sentence the person approves to forget everything an undo could put
    /// back.
    ///
    /// **One act, and there is only one** (ADR 0045 point 5). Not one per turn
    /// and not one per folder: a person who wants the space back wants it back,
    /// and a machine that made them reclaim it a turn at a time would be
    /// keeping it by attrition. What they approve says both halves — that
    /// nothing an agent has changed can be put back afterwards, and that the
    /// space it holds is freed.
    #[must_use]
    pub fn forgetting(strings: &Strings) -> Said {
        strings.say(
            &words::FORGETTING_WHAT_CAN_BE_PUT_BACK.key(),
            &Filling::nothing(),
        )
    }
}

impl WhenItWasDone {
    /// A moment, as somebody has already worded it — [`None`] for text with
    /// nothing in it.
    #[must_use]
    pub fn worded(text: &str) -> Option<Self> {
        let text = text.trim();
        (!text.is_empty()).then(|| Self(text.to_owned()))
    }

    /// The moment, as it will read in the sentence.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AnUndo {
    /// Whether what the record says happened can be put back on a machine that
    /// kept `kept`, and what it would put back.
    ///
    /// Every refusal is decided **before** anything is offered, which is the
    /// same rule [`crate::GoingBack::offered`] keeps and for a sharper reason:
    /// an undo offered and then failing halfway is a person's files left
    /// between two states nobody chose.
    ///
    /// # Errors
    /// [`NotUndoable`], the reason, in a sentence the person reads instead of
    /// an offer.
    pub fn offered(what: WhatWasDone, kept: &WhatWasKept) -> Result<Self, NotUndoable> {
        let change = what.could_be_put_back()?;
        let bracket = match kept {
            WhatWasKept::NothingOnThisMachine => {
                return Err(NotUndoable::NothingKeepsWhatWasThere);
            }
            WhatWasKept::NotThisTurn => return Err(NotUndoable::TheTurnWasNotKept),
            WhatWasKept::LetGo => return Err(NotUndoable::NoLongerKept),
            WhatWasKept::EitherSideOfTheTurn(bracket) => bracket,
        };
        if !bracket.not_as_it_was_left().is_empty() {
            return Err(NotUndoable::ChangedSinceItWasDone);
        }
        if bracket.changed().is_empty() {
            return Err(NotUndoable::NothingLeftToPutBack);
        }
        Ok(Self {
            change,
            puts_back: bracket.changed().to_vec(),
        })
    }

    /// The change this puts back.
    #[must_use]
    pub fn change(&self) -> Change {
        self.change
    }

    /// What it puts back, by name — never empty.
    #[must_use]
    pub fn puts_back(&self) -> &[String] {
        &self.puts_back
    }

    /// The sentence the person approves.
    ///
    /// It names both of the things ADR 0045 point 2 asks of it: **what** will
    /// be put back, and **when** it was changed. What is named is every name
    /// the comparison found, one after another — a surface with room to list
    /// them properly has [`AnUndo::puts_back`] and should use it, and this is
    /// what a sentence can carry.
    #[must_use]
    pub fn said(&self, when: &WhenItWasDone, strings: &Strings) -> Said {
        strings.say(
            &words::UNDO_OFFERED.key(),
            &Filling::of("what", self.puts_back.join(AND_THE_NEXT)).and("when", when.as_str()),
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// A turn that renamed one file and is still exactly as it left it.
    fn a_rename_still_as_it_was() -> WhatWasKept {
        WhatWasKept::EitherSideOfTheTurn(Bracket::comparing(
            vec!["/home/ana/Invoices/March.pdf".to_owned()],
            Vec::new(),
        ))
    }

    /// The moment the person is shown, already worded.
    fn on_tuesday() -> WhenItWasDone {
        WhenItWasDone::worded("Tuesday at 14:05").unwrap()
    }

    /// **A rename the machine kept both sides of is offered, in a sentence
    /// naming what comes back and when it changed.**
    #[test]
    fn a_rename_the_machine_kept_is_offered_with_what_and_when() {
        let undo = AnUndo::offered(
            WhatWasDone::ChangedFiles(Change::Renamed),
            &a_rename_still_as_it_was(),
        )
        .unwrap();
        assert_eq!(undo.change(), Change::Renamed);
        assert_eq!(undo.puts_back(), ["/home/ana/Invoices/March.pdf"]);

        let said = undo.said(&on_tuesday(), &in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(
            said.text().contains("/home/ana/Invoices/March.pdf"),
            "{said}"
        );
        assert!(said.text().contains("Tuesday at 14:05"), "{said}");
    }

    /// **Several names are all named**, because approving *some of them* is not
    /// something a person can be asked to do.
    #[test]
    fn everything_that_comes_back_is_named_in_the_sentence() {
        let kept = WhatWasKept::EitherSideOfTheTurn(Bracket::comparing(
            vec!["March.pdf".to_owned(), "April.pdf".to_owned()],
            Vec::new(),
        ));
        let undo = AnUndo::offered(WhatWasDone::ChangedFiles(Change::Moved), &kept).unwrap();
        let said = undo.said(&on_tuesday(), &in_english());
        assert!(said.text().contains("March.pdf"), "{said}");
        assert!(said.text().contains("April.pdf"), "{said}");
    }

    /// **On the machine alo OS installs today, nothing kept what was there, and
    /// that is what a person is told** — the honest *not yet on this machine*
    /// rather than an offer nothing could carry out.
    #[test]
    fn a_machine_that_keeps_nothing_says_so_instead_of_offering() {
        for change in Change::EVERY {
            let refused = AnUndo::offered(
                WhatWasDone::ChangedFiles(change),
                &WhatWasKept::NothingOnThisMachine,
            )
            .unwrap_err();
            assert_eq!(refused, NotUndoable::NothingKeepsWhatWasThere, "{change:?}");
            let said = refused.said(&in_english());
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.text().contains("does not keep"), "{said}");
        }
    }

    /// **A turn the machine had no room to keep is refused in its own words**,
    /// and the turn itself ran — the machine never stops working to protect an
    /// undo.
    #[test]
    fn a_turn_there_was_no_room_to_keep_is_refused_in_its_own_words() {
        let refused = AnUndo::offered(
            WhatWasDone::ChangedFiles(Change::Archived),
            &WhatWasKept::NotThisTurn,
        )
        .unwrap_err();
        assert_eq!(refused, NotUndoable::TheTurnWasNotKept);
        assert!(!refused.said(&in_english()).is_a_bug());
    }

    /// **What has been let go is refused**, whether the window passed, the disk
    /// needed the room or the person forgot it.
    #[test]
    fn what_has_been_let_go_is_refused() {
        let refused = AnUndo::offered(
            WhatWasDone::ChangedFiles(Change::Moved),
            &WhatWasKept::LetGo,
        )
        .unwrap_err();
        assert_eq!(refused, NotUndoable::NoLongerKept);
        assert!(!refused.said(&in_english()).is_a_bug());
    }

    /// **An undo never overwrites a later change** (ADR 0045 point 3): one name
    /// no longer as the turn left it refuses the whole undo, before it is
    /// offered, rather than putting some of it back over the person's own work.
    #[test]
    fn anything_changed_since_refuses_the_undo_before_it_is_offered() {
        let kept = WhatWasKept::EitherSideOfTheTurn(Bracket::comparing(
            vec!["March.pdf".to_owned(), "April.pdf".to_owned()],
            vec!["April.pdf".to_owned()],
        ));
        let refused =
            AnUndo::offered(WhatWasDone::ChangedFiles(Change::Renamed), &kept).unwrap_err();
        assert_eq!(refused, NotUndoable::ChangedSinceItWasDone);
        let said = refused.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("changed since"), "{said}");
    }

    /// **A bracket that found nothing offers nothing.** Two identical sides
    /// mean the turn left the person's files as it found them, and an offer
    /// would be a sentence with nothing in it.
    #[test]
    fn a_turn_that_changed_nothing_offers_nothing() {
        let kept = WhatWasKept::EitherSideOfTheTurn(Bracket::default());
        let refused = AnUndo::offered(WhatWasDone::ChangedFiles(Change::Moved), &kept).unwrap_err();
        assert_eq!(refused, NotUndoable::NothingLeftToPutBack);
        assert!(!refused.said(&in_english()).is_a_bug());
    }

    /// **What was never undoable is refused for what it was, not for what the
    /// machine kept.** A printed page on a machine with the whole road built is
    /// still a printed page.
    #[test]
    fn what_was_never_undoable_is_refused_for_what_it_was() {
        let refused =
            AnUndo::offered(WhatWasDone::ItWasPrinted, &a_rename_still_as_it_was()).unwrap_err();
        assert_eq!(refused, NotUndoable::ItWasPrintedOnPaper);
    }

    /// **A moment with nothing in it is not a moment**, so an offer cannot be
    /// worded with a blank where the person would look for when it happened.
    #[test]
    fn a_blank_moment_is_not_a_moment() {
        assert_eq!(WhenItWasDone::worded(""), None);
        assert_eq!(WhenItWasDone::worded("   "), None);
        assert_eq!(
            WhenItWasDone::worded("  Tuesday at 14:05 ")
                .unwrap()
                .as_str(),
            "Tuesday at 14:05"
        );
    }

    /// **Forgetting is one act, and what it costs is in the sentence.**
    #[test]
    fn forgetting_is_one_act_that_says_what_it_costs() {
        let said = WhatWasKept::forgetting(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(
            said.text().contains("none of them can be put back"),
            "{said}"
        );
        assert!(said.text().contains("space"), "{said}");
    }
}
