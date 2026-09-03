//! Where else this machine may put a question, and what it may not.
//!
//! Worked out once, at the moment a question failed, from the places a person
//! has set up and the rule their organisation set. What comes out is two lists
//! and one fact: the [offers](Offer) a person may approve, the
//! [refusals](alo_models::NotAllowed) explaining the doors that are closed, and
//! whether there was ever another door at all.
//!
//! # The policy is asked here, not when an offer is answered
//!
//! An offer that existed and then turned out to be forbidden would be a dialogue
//! that punishes somebody for pressing the button it drew. So a source the
//! policy refuses never becomes an [`Offer`]; it becomes a line saying which
//! rule closed it, in that rule's own words rather than in a summary of them —
//! `alo_models::NotTried::Forbidden`'s decision from item 9f, one crate on.
//!
//! # And a person is told about the closed doors
//!
//! It would be tidier to leave them out. ADR 0008 says the policy *is stated in
//! words a person on that machine can read*, and ADR 0004 says a person on a
//! managed machine is told so — a machine that quietly showed nothing would
//! leave somebody believing they had no provider when what they have is an
//! organisation with a rule.
//!
//! # What is never here
//!
//! **The place that just failed.** Offering it back would be the retry a person
//! can already ask for themselves, dressed as a decision — and it would put a
//! sentence about leaving the building in front of somebody for a place nothing
//! is leaving to.
//!
//! **The same place twice.** A settings file that names one provider twice is a
//! dialogue with two identical buttons, and whichever a person presses they
//! will wonder what the other one was.

use alo_models::{InferenceSource, NotAllowed, SourcePolicy};
use alo_strings::{Filling, Said, Strings};

use crate::offer::Offer;
use crate::words;

/// The places this machine could ask instead, and the ones it may not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elsewhere {
    /// What a person may approve, in the order the places were given.
    offers: Vec<Offer>,
    /// Why each of the others is not on offer, in that rule's own words.
    refused: Vec<NotAllowed>,
    /// Whether there was no other place at all — as against places that were
    /// all refused.
    nothing_else: bool,
}

impl Elsewhere {
    /// Work out what is left, given the place that failed, the places this
    /// machine has, and the rule it is under.
    ///
    /// `pub(crate)`: the only caller is [`crate::Answering::did_not_answer`],
    /// because an `Elsewhere` that existed without a failure would be a list of
    /// approvals waiting for a question that never went wrong.
    pub(crate) fn of(
        failed: &InferenceSource,
        others: &[InferenceSource],
        policy: &SourcePolicy,
    ) -> Self {
        let mut offers = Vec::new();
        let mut refused = Vec::new();
        let mut already: Vec<&InferenceSource> = Vec::new();
        for other in others {
            if other == failed || already.contains(&other) {
                continue;
            }
            already.push(other);
            match policy.refusal(other) {
                Some(rule) => refused.push(rule),
                None => offers.push(Offer::of(other.clone())),
            }
        }
        Self {
            offers,
            refused,
            nothing_else: already.is_empty(),
        }
    }

    /// What a person may approve.
    ///
    /// Empty is an ordinary answer and not an error: most machines have one
    /// place to ask, and a machine that is set to answer its own questions and
    /// nothing else has one on purpose.
    #[must_use]
    pub fn offers(&self) -> &[Offer] {
        &self.offers
    }

    /// Why the other places are not on offer, in the words of the rule that
    /// closed them.
    #[must_use]
    pub fn refused(&self) -> &[NotAllowed] {
        &self.refused
    }

    /// Whether there is nothing a person can approve.
    #[must_use]
    pub fn is_nowhere(&self) -> bool {
        self.offers.is_empty()
    }

    /// Whether there was no other place at all.
    ///
    /// Different from [`is_nowhere`](Self::is_nowhere), and the difference is
    /// the whole of what a person needs to know: *you have not set up anywhere
    /// else* and *your organisation does not permit the places you have set up*
    /// send somebody to two different screens.
    #[must_use]
    pub fn nothing_else(&self) -> bool {
        self.nothing_else
    }

    /// What to draw beneath the offers, one line each, in the order they are
    /// worth reading.
    ///
    /// **Lines rather than clauses**, which is `alo-shortcuts`' rule from item
    /// 9c: the separator between two sentences is not punctuation a program can
    /// pick, and the conjunction would be a word placed by a machine that does
    /// not know the sentence.
    #[must_use]
    pub fn lines(&self, strings: &Strings) -> Vec<Said> {
        let mut lines: Vec<Said> = self.refused.iter().map(|rule| rule.said(strings)).collect();
        if self.nothing_else {
            lines.push(strings.say(&words::NOWHERE_ELSE.key(), &Filling::nothing()));
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{here, hosted, in_english, paired, somewhere};

    /// The ordinary case: a machine with a provider set up, a rule that permits
    /// it, and a local model that has just failed.
    #[test]
    fn a_permitted_place_becomes_something_a_person_can_approve() {
        let elsewhere = Elsewhere::of(&here(), &[hosted()], &SourcePolicy::Anywhere);
        assert_eq!(elsewhere.offers().len(), 1);
        assert_eq!(
            elsewhere.offers().first().map(Offer::source),
            Some(&hosted())
        );
        assert!(!elsewhere.is_nowhere());
        assert!(!elsewhere.nothing_else());
        assert!(elsewhere.lines(&in_english()).is_empty());
    }

    /// **The place that just failed is never offered back.** A person who wants
    /// to try the same place again can ask again; putting it in a list of
    /// places to *leave to* would be a sentence about egress for a place
    /// nothing leaves to.
    #[test]
    fn the_place_that_failed_is_not_among_the_places_to_try_instead() {
        let elsewhere = Elsewhere::of(&here(), &[here(), hosted()], &SourcePolicy::Anywhere);
        assert_eq!(elsewhere.offers().len(), 1);
        assert_eq!(
            elsewhere.offers().first().map(Offer::source),
            Some(&hosted())
        );

        // And a machine whose only other entry is the one that failed has
        // nowhere else at all.
        let alone = Elsewhere::of(&here(), &[here()], &SourcePolicy::Anywhere);
        assert!(alone.is_nowhere());
        assert!(alone.nothing_else());
    }

    /// One place named twice is one offer. Two identical buttons is a dialogue
    /// where whichever a person presses, they wonder what the other one was.
    #[test]
    fn one_place_named_twice_is_offered_once() {
        let elsewhere = Elsewhere::of(
            &here(),
            &[hosted(), hosted(), paired()],
            &SourcePolicy::Anywhere,
        );
        assert_eq!(elsewhere.offers().len(), 2);
        assert_eq!(
            elsewhere.offers().first().map(Offer::source),
            Some(&hosted())
        );
        assert_eq!(
            elsewhere.offers().get(1).map(Offer::source),
            Some(&paired())
        );
    }

    /// **The order is the person's.** They set these up; a list this crate
    /// sorted would put a provider somebody added last week above the machine
    /// they use every day, for a reason nobody could see.
    #[test]
    fn the_places_are_offered_in_the_order_they_were_given() {
        let elsewhere = Elsewhere::of(
            &here(),
            &[somewhere(), paired(), hosted()],
            &SourcePolicy::Anywhere,
        );
        let sources: Vec<&InferenceSource> = elsewhere.offers().iter().map(Offer::source).collect();
        assert_eq!(sources, [&somewhere(), &paired(), &hosted()]);
    }

    /// **A forbidden place is not an offer, and it is not silence either.** The
    /// person is told which rule closed the door, in that rule's own words, so
    /// the machine cannot explain one rule two ways.
    #[test]
    fn a_place_the_rule_forbids_is_explained_rather_than_offered() {
        let policy = SourcePolicy::InTheBuilding;
        let elsewhere = Elsewhere::of(&here(), &[paired(), hosted()], &policy);

        assert_eq!(elsewhere.offers().len(), 1);
        assert_eq!(
            elsewhere.offers().first().map(Offer::source),
            Some(&paired())
        );
        assert_eq!(elsewhere.refused().len(), 1);
        assert_eq!(
            elsewhere.refused().first(),
            policy.refusal(&hosted()).as_ref(),
            "the rule's own refusal, not a second account of it"
        );

        let strings = in_english();
        let lines = elsewhere.lines(&strings);
        assert_eq!(lines.len(), 1);
        assert!(
            lines
                .first()
                .is_some_and(|line| line.text().contains("keep questions in the building")),
            "{lines:?}"
        );
    }

    /// **Nowhere at all and nowhere permitted are different things to be
    /// told**, because they send a person to two different screens: one to
    /// Settings to add a provider, the other to whoever manages the machine.
    #[test]
    fn having_nowhere_and_being_allowed_nowhere_say_different_things() {
        let strings = in_english();

        let none = Elsewhere::of(&here(), &[], &SourcePolicy::Anywhere);
        assert!(none.is_nowhere());
        assert!(none.nothing_else());
        let lines = none.lines(&strings);
        assert_eq!(lines.len(), 1);
        assert!(
            lines
                .first()
                .is_some_and(|line| line.text().contains("nowhere else set up")),
            "{lines:?}"
        );

        let forbidden = Elsewhere::of(&here(), &[hosted()], &SourcePolicy::ThisMachineOnly);
        assert!(forbidden.is_nowhere());
        assert!(!forbidden.nothing_else());
        let lines = forbidden.lines(&strings);
        assert_eq!(lines.len(), 1);
        assert!(
            lines
                .first()
                .is_some_and(|line| line.text().contains("answer only on itself")),
            "{lines:?}"
        );
    }

    /// Every door closed by a rule gets a line of its own, rather than one line
    /// standing in for however many there were.
    #[test]
    fn every_closed_door_is_a_line_of_its_own() {
        let elsewhere = Elsewhere::of(
            &here(),
            &[hosted(), somewhere(), paired()],
            &SourcePolicy::ThisMachineOnly,
        );
        assert!(elsewhere.is_nowhere());
        assert_eq!(elsewhere.refused().len(), 3);
        assert_eq!(elsewhere.lines(&in_english()).len(), 3);
    }
}
