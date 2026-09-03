//! Why a turn has no answer to a question, and what left the machine while it
//! found out.
//!
//! [`crate::NotDone`] is the same idea about a verb, and the two are kept apart
//! because a caller has nothing in common to do about them: a verb that was
//! refused is the capability model working, and a question that was not
//! answered is a place that did not reply. One enum holding twelve variants
//! would be twelve arms in every match and the wrong half read carefully.
//!
//! | What came back | What happened | What is written down |
//! |---|---|---|
//! | [`NoAnswer::NotAQuestion`] | Nothing was asked: no text, or no model named | nothing — see below |
//! | [`NoAnswer::CannotBeShown`] | Nothing left. Law 1 could not put the place on the indicator | nothing — see below |
//! | [`NoAnswer::HeldBack`] | Nothing left. The rule in force refused it | `held back`, in the rule's own words |
//! | [`NoAnswer::DidNotAnswer`] | The place did not reply. It left first, or it did not, depending where it was put | `left`, when it left |
//! | [`NoAnswer::Miswired`] | Nothing was asked. The permission and the place are not the same place | nothing — see below |
//! | [`NoAnswer::NotRecorded`] | What happened could not be written down | nothing, which is the problem |
//! | [`NoAnswer::TurnClosed`] | Something earlier in this turn could not be written down | nothing |
//!
//! # What the record keeps of a question, and what it does not
//!
//! Everything that **left this machine** is written down before the caller
//! hears about it, answered or not: `alo_record::Entry::left` is made from the
//! departure the door hands back, and the door hands one back on both roads.
//! Everything a **rule stopped from leaving** is written down too, as
//! `alo_record::Entry::held_back`. A question answered **here** is
//! `alo_record::Entry::answered_here`, which names who asked and nothing else.
//!
//! Four things are deliberately not written, and they are the four where
//! nothing left the machine and nothing on the machine answered:
//!
//! - **A question that never became one.** There was no place, no destination
//!   and no attempt — and what a person typed is the one thing ADR 0001 §7 says
//!   is never kept, so an entry would say only that somebody had pressed a key.
//! - **A place that could not be shown.** `alo_asking::NotAsked::CannotBeShown`
//!   is a provider whose *name* cannot be drawn onto the indicator; it is fixed
//!   by whoever added the provider, nothing was addressed and nothing left.
//! - **A permission and a place that disagree.** Nothing was sent, and it is a
//!   fault in the wiring rather than a thing that happened on the machine.
//! - **A question this machine could not answer.** `alo-answering` settled this
//!   one and it is followed rather than decided again: an entry per failure
//!   would build a log of somebody's questions failing, one honest entry at a
//!   time, and `alo_record::Happened::AnsweredHere` would be a lie about a
//!   question nothing answered. What the record keeps is what happens *next* —
//!   if the person takes an offer, the egress it causes is written down like
//!   any other.
//!
//! # Six of the seven are worded by whoever refused
//!
//! [`NoAnswer::said`] hands the question straight on, as [`crate::NotDone`]
//! does, so this crate still says exactly one sentence of its own. The seventh
//! is [`NoAnswer::Miswired`] and it answers [`None`]: it keeps its English and
//! its `Display` in `alo-asking` because its reader is whoever wired a question
//! to somewhere, and inventing a sentence for it here would be this crate
//! saying something to a person about a state a person cannot cause.

use alo_answering::Failed;
use alo_asking::{Miswired, NotAQuestion};
use alo_egress::{DestinationError, NotPermitted};
use alo_keeping::NotKept;
use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Why a question a turn put somewhere has no answer.
///
/// **Not `PartialEq`**, like `alo_asking::NotAsked`: one failure is not
/// another, and two of these carry things that are not values.
#[derive(Debug)]
pub enum NoAnswer {
    /// Nothing was asked — there was no question, or no model was named to
    /// answer it.
    NotAQuestion(NotAQuestion),
    /// **Nothing left.** The place could not be put on the indicator, and law 1
    /// does not permit an egress nobody can be shown.
    CannotBeShown(DestinationError),
    /// **Nothing left.** The rule in force when the question was asked refused
    /// it, in the rule's own words.
    HeldBack(NotPermitted),
    /// The place did not answer: what the person may be told, and what they may
    /// be asked.
    ///
    /// **The turn does nothing with it.** Taking one of its offers is the
    /// person's act and `alo_answering::Failed::take` is the only door, so this
    /// comes back whole and belongs to whoever is showing it — which is also
    /// what lets a person think about it for longer than the turn lasts.
    /// [`crate::Turning::asking`] is where the answer to a taken offer comes
    /// back in.
    ///
    /// **Boxed** for `alo_asking::NotAsked::DidNotAnswer`'s reason: a failure
    /// carries every offer this machine could make, which would otherwise be
    /// the size of every answer this door returns.
    DidNotAnswer(Box<Failed>),
    /// **Nothing was asked.** The permission named one place and another was
    /// put the question, or a failure was reported where it could not have
    /// happened.
    Miswired(Miswired),
    /// What happened could not be written down, and the turn is over.
    NotRecorded {
        /// Why the record could not be written.
        why: NotKept,
        /// Whether the question had already left this machine when it happened.
        ///
        /// The difference a daemon acts on. `false` is a machine that has
        /// stopped keeping evidence and has nothing to be missing evidence
        /// *of*; `true` is somebody's question having gone to a provider with
        /// no record that it did, which is law 1's second half failing on the
        /// largest egress this product causes.
        after_it_left: bool,
    },
    /// Something earlier in this turn could not be written down, so nothing
    /// more will be done under it.
    TurnClosed,
}

impl NoAnswer {
    /// What this says, in the language the person reads — and nothing for the
    /// one that is not said to anybody.
    ///
    /// Six of the seven hand the question straight to whoever made the refusal,
    /// so a person reads one sentence about a rule wherever in alo OS that rule
    /// refused something. [`NoAnswer::Miswired`] answers [`None`], and this
    /// module's documentation has the argument.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Said> {
        match self {
            Self::NotAQuestion(why) => Some(why.said(strings)),
            Self::CannotBeShown(why) => Some(why.said(strings)),
            Self::HeldBack(refused) => Some(refused.said(strings)),
            Self::DidNotAnswer(failed) => Some(failed.said(strings)),
            Self::Miswired(_) => None,
            Self::NotRecorded { why, .. } => Some(why.said(strings)),
            Self::TurnClosed => Some(strings.say(&words::TURN_CLOSED.key(), &Filling::nothing())),
        }
    }

    /// Whether this attempt sent anything off the machine at all.
    ///
    /// The question a person asks first, answered once rather than by every
    /// caller matching seven variants. It is a fact about **this attempt**: a
    /// closed turn answers `true` because this door did nothing, whatever an
    /// earlier one did.
    #[must_use]
    pub fn nothing_left(&self) -> bool {
        match self {
            Self::NotAQuestion(_)
            | Self::CannotBeShown(_)
            | Self::HeldBack(_)
            | Self::Miswired(_)
            | Self::TurnClosed => true,
            // Where it was put decides it, and the place itself is the answer:
            // a provider that did not reply was still reached, and a model on
            // this machine that did not reply went nowhere.
            Self::DidNotAnswer(failed) => !failed.source().causes_egress(),
            Self::NotRecorded { after_it_left, .. } => !after_it_left,
        }
    }

    /// Whether this turn is over because the record could not be written.
    ///
    /// True of both halves of that one fact, as [`crate::NotDone`] answers it,
    /// because what a caller does about them is the same thing and it is not
    /// *try again*.
    #[must_use]
    pub fn is_the_end_of_the_turn(&self) -> bool {
        matches!(self, Self::NotRecorded { .. } | Self::TurnClosed)
    }
}

impl From<NotAQuestion> for NoAnswer {
    fn from(why: NotAQuestion) -> Self {
        Self::NotAQuestion(why)
    }
}

impl From<Miswired> for NoAnswer {
    fn from(why: Miswired) -> Self {
        Self::Miswired(why)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{every_way_a_question_can_fail, in_english, translated};
    use alo_strings::Strings;

    /// **Every one of them that is said to a person says something**, in the
    /// language they read, and none of them reaches somebody as a key.
    #[test]
    fn every_way_a_question_can_fail_says_something() {
        let strings = in_english();
        for no_answer in every_way_a_question_can_fail() {
            let Some(said) = no_answer.said(&strings) else {
                assert!(
                    matches!(no_answer, NoAnswer::Miswired(_)),
                    "{no_answer:?} says nothing, and it is not the wiring refusal"
                );
                continue;
            };
            assert!(!said.is_a_bug(), "{no_answer:?}: {said}");
            assert!(!said.text().is_empty(), "{no_answer:?}");
        }
    }

    /// **Five of the six are somebody else's words.** With only this crate's
    /// own vocabulary loaded, the closed turn reads and everything else is a
    /// key nothing declares — which is the test that catches this crate
    /// starting to say something one of its five dependencies already says.
    #[test]
    fn the_only_sentence_this_crate_says_about_a_question_is_the_closed_turn() {
        let ours = Strings::of(words::turn_words().unwrap());
        let mut said_by_us = 0;
        for no_answer in every_way_a_question_can_fail() {
            if no_answer.said(&ours).is_some_and(|said| !said.is_a_bug()) {
                said_by_us += 1;
            }
        }
        assert_eq!(
            said_by_us, 1,
            "this crate has started saying something somebody else already says"
        );
    }

    /// **The question a person asks first**, and the one road where the answer
    /// is `false` is the one where something really went to a provider.
    #[test]
    fn what_left_the_machine_is_answered_by_where_the_question_was_put() {
        let mut left = 0;
        for no_answer in every_way_a_question_can_fail() {
            if !no_answer.nothing_left() {
                left += 1;
            }
            // A refusal and the end of the turn are never the same answer.
            if no_answer.is_the_end_of_the_turn() {
                assert!(
                    matches!(
                        no_answer,
                        NoAnswer::NotRecorded { .. } | NoAnswer::TurnClosed
                    ),
                    "{no_answer:?}"
                );
            }
        }
        assert_eq!(
            left, 2,
            "the question that reached a provider and the record that broke after it left"
        );
    }

    /// And the one sentence this crate says about a question is translated like
    /// everything else — it is read by the person whose machine has stopped
    /// keeping evidence.
    #[test]
    fn the_sentence_this_crate_says_is_one_a_translator_can_move() {
        let german = translated(&[(
            words::TURN_CLOSED,
            "dieser Vorgang wurde beendet, weil nicht aufgezeichnet werden konnte, was geschehen \
             ist",
        )]);
        let said = NoAnswer::TurnClosed.said(&german).unwrap();
        assert!(said.is_translated(), "{said}");
    }
}
