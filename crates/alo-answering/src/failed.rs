//! The place a question was put did not answer it: what a person reads, and
//! the one door out.
//!
//! This is the type ADR 0008's *never a silent fallback* exists as. A machine
//! that fell back would have no type here at all — the second attempt would be
//! a line of code inside whatever asked the first time, and the person would
//! learn about it from their bill.
//!
//! # The door is one, and it is spent by going through it
//!
//! [`Failed::take`] takes `self`. One failure yields at most one attempt
//! somewhere else, and a second is not a program that compiles — the shape
//! `alo_capability::Approved::redeem` has, for the same reason: *one approval,
//! one action* is worth more as a thing the compiler knows than as a rule a
//! daemon remembers.
//!
//! # And the failure alone is not the door
//!
//! Holding a `Failed` permits nothing. It is a thing to show somebody, and
//! until they answer one of its offers there is no `Answering` anywhere in the
//! program — which is what
//! makes *nothing was sent, and nothing will be unless you say so* a fact about
//! the types rather than a sentence in a dialogue.
//!
//! # What is not written down
//!
//! Nothing here reaches `alo-record`. A question that failed is not something
//! an agent did and not something that left, and an entry per failure would
//! build a log of somebody's questions failing, one honest entry at a time —
//! `alo-context`'s reasoning about what an entry per invocation would build.
//! What the record keeps is what *happens next*: if an offer is taken, the
//! egress it causes goes through `alo-egress` and is written down there, like
//! every other departure. `alo-files`' rule is the other half of it — a refusal
//! by the machine and a refusal by a rule are different types, and only one of
//! them is evidence about a policy.

use alo_models::InferenceSource;
use alo_strings::{Filling, Said, Strings};

use crate::answering::Answering;
use crate::elsewhere::Elsewhere;
use crate::offer::Offer;
use crate::refusing::NotOffered;
use crate::words;
use crate::wrong::WentWrong;

/// A question that was not answered where it was put.
///
/// **Deliberately not `Clone`**, which is `alo_capability::Approved`'s rule and
/// for exactly its reason: a clone would be a second way to take an offer from
/// one failure, and *one failure, at most one attempt elsewhere* would hold
/// only for callers who did not think of it. Not deserialisable either, like
/// everything in this crate.
#[derive(Debug, PartialEq, Eq)]
pub struct Failed {
    /// Where it was put.
    source: InferenceSource,
    /// What went wrong there.
    why: WentWrong,
    /// Where else it could go, and where it may not.
    elsewhere: Elsewhere,
}

impl Failed {
    /// Made by [`crate::Answering::did_not_answer`] and by nothing else: a
    /// failure that did not come from an attempt would be a dialogue offering
    /// to send somebody's question somewhere, about a question nobody asked.
    pub(crate) fn of(source: InferenceSource, why: WentWrong, elsewhere: Elsewhere) -> Self {
        Self {
            source,
            why,
            elsewhere,
        }
    }

    /// Where the question was put.
    #[must_use]
    pub fn source(&self) -> &InferenceSource {
        &self.source
    }

    /// What went wrong there.
    #[must_use]
    pub fn why(&self) -> WentWrong {
        self.why
    }

    /// Where else this machine could ask, and where it may not.
    #[must_use]
    pub fn elsewhere(&self) -> &Elsewhere {
        &self.elsewhere
    }

    /// What went wrong, in the language the person reads.
    ///
    /// Names the place, because *the model did not answer* is a different thing
    /// to be told about this machine than about somebody's API — and the place
    /// goes in as a [`Said`] rather than as text, so a half-translated line
    /// says so.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::answering_words`] answers with the key, marked.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = Filling::nothing().and_said("source", &self.source.said(strings));
        let filling = match self.why {
            WentWrong::HavingTrouble(status) => filling.and("status", status.to_string()),
            WentWrong::NothingAnswered
            | WentWrong::TookTooLong
            | WentWrong::NothingUsable
            | WentWrong::NoModelThere
            | WentWrong::KeyNotAccepted
            | WentWrong::SentSomewhereElse
            | WentWrong::RanOut => filling,
        };
        strings.say(&self.why.word().key(), &filling)
    }

    /// The line that says nothing happened instead, and nothing will.
    ///
    /// **Always shown, whether or not there is anywhere else to ask.** A person
    /// who has just watched a question fail has no way of knowing their records
    /// did not go somewhere else to be answered, and ADR 0008 rejects falling
    /// back precisely because that is invisible when it happens. A promise
    /// nobody is told about is not a feature.
    #[must_use]
    pub fn nothing_was_sent(&self, strings: &Strings) -> Said {
        strings.say(&words::NOTHING_WAS_SENT.key(), &Filling::nothing())
    }

    /// Take one of the offers: the person said yes to this place, this once.
    ///
    /// Takes `self`, so one failure is worth at most one attempt elsewhere:
    ///
    /// ```
    /// use alo_answering::{Answering, WentWrong, answering_words};
    /// use alo_models::{InferenceSource, Region, SourcePolicy};
    /// use alo_strings::Strings;
    ///
    /// # fn main() {
    /// let alo = InferenceSource::Hosted {
    ///     provider: "alo".to_owned(),
    ///     region: Region::Declared("the EU".to_owned()),
    /// };
    /// let strings = Strings::of(answering_words().expect("this crate's own words"));
    /// let here = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere)
    ///     .expect("nothing forbids answering here");
    /// let failed = here
    ///     .did_not_answer(WentWrong::NothingAnswered, &[alo.clone()], &SourcePolicy::Anywhere)
    ///     .expect("a runtime that is not running can happen here");
    ///
    /// // Nothing has been sent, and the person is told so.
    /// assert_eq!(
    ///     failed.nothing_was_sent(&strings).text(),
    ///     "nothing was sent anywhere, and nothing will be unless you say so",
    /// );
    ///
    /// let offer = failed.elsewhere().offers().first().cloned().expect("alo is permitted");
    /// let elsewhere = failed.take(&offer).expect("that offer was this failure's");
    /// assert_eq!(elsewhere.source(), &alo);
    /// # }
    /// ```
    ///
    /// The same failure taken twice is not a program:
    ///
    /// ```compile_fail
    /// use alo_answering::{Answering, WentWrong};
    /// use alo_models::{InferenceSource, Region, SourcePolicy};
    ///
    /// # fn main() {
    /// let alo = InferenceSource::Hosted {
    ///     provider: "alo".to_owned(),
    ///     region: Region::Declared("the EU".to_owned()),
    /// };
    /// let failed = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere)
    ///     .expect("nothing forbids answering here")
    ///     .did_not_answer(WentWrong::NothingAnswered, &[alo], &SourcePolicy::Anywhere)
    ///     .expect("a runtime that is not running can happen here");
    /// let offer = failed.elsewhere().offers().first().cloned().expect("alo is permitted");
    /// let _once = failed.take(&offer);
    /// let _twice = failed.take(&offer); // the failure was spent above
    /// # }
    /// ```
    ///
    /// # Errors
    /// [`NotOffered`], carrying this failure back, when the offer was not one
    /// of this failure's. Nothing is sent, and the person can be shown the
    /// failure again rather than a dialogue that has lost what it was about.
    pub fn take(self, offer: &Offer) -> Result<Answering, NotOffered> {
        if self.elsewhere.offers().contains(offer) {
            Ok(Answering::new(offer.source().clone()))
        } else {
            Err(NotOffered::of(self))
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{failing, here, hosted, in_english, paired, somewhere, translated};
    use alo_models::SourcePolicy;

    /// The line names what went wrong **and** where, because the second is what
    /// a person needs in order to know whether their question went anywhere.
    #[test]
    fn what_went_wrong_is_said_with_the_place_it_went_wrong_in_it() {
        let strings = in_english();
        assert_eq!(
            failing(here(), WentWrong::NothingAnswered, &[])
                .said(&strings)
                .text(),
            "nothing answered on this machine"
        );
        assert_eq!(
            failing(here(), WentWrong::TookTooLong, &[])
                .said(&strings)
                .text(),
            "nothing answered on this machine within the time this machine waits"
        );
        assert_eq!(
            failing(here(), WentWrong::NoModelThere, &[])
                .said(&strings)
                .text(),
            "the model this question needed was not there to answer on this machine"
        );
    }

    /// A status is a number the service answered with, not a word and not a
    /// count — so it goes into the sentence as it arrived.
    #[test]
    fn a_service_in_trouble_says_what_it_answered_without_repeating_its_own_words() {
        assert_eq!(
            failing(hosted(), WentWrong::HavingTrouble(503), &[])
                .said(&in_english())
                .text(),
            "nothing was answered by alo, in the EU — it answered 503, which is a problem at that \
             end rather than yours"
        );
    }

    /// **The reassurance is shown whether or not there is anywhere to go.** A
    /// person whose machine has one model and no provider still needs telling
    /// that nothing was sent, and they are the person least likely to be shown
    /// anything else.
    #[test]
    fn nothing_was_sent_is_said_even_when_there_was_nowhere_to_send_it() {
        let strings = in_english();
        let failed = failing(here(), WentWrong::NothingAnswered, &[]);
        assert!(failed.elsewhere().is_nowhere());
        assert_eq!(
            failed.nothing_was_sent(&strings).text(),
            "nothing was sent anywhere, and nothing will be unless you say so"
        );
    }

    /// **The whole of the promise, as a test.** Holding a failure permits
    /// nothing: the only way to an [`Answering`] somewhere else is through an
    /// offer a person answered, and the offers are what a person is shown.
    #[test]
    fn a_failure_alone_is_permission_to_ask_nowhere() {
        let failed = failing(here(), WentWrong::NothingAnswered, &[hosted(), paired()]);
        // Two places a person may be asked about, and nothing decided.
        assert_eq!(failed.elsewhere().offers().len(), 2);
        assert_eq!(failed.source(), &here());
        assert_eq!(failed.why(), WentWrong::NothingAnswered);
    }

    /// **An offer from another failure authorises nothing**, and the refusal
    /// hands the failure back so the person is not left looking at a dialogue
    /// that has forgotten what it was about.
    #[test]
    fn an_offer_this_failure_never_made_is_refused_and_nothing_is_lost() {
        let failed = failing(here(), WentWrong::NothingAnswered, &[hosted()]);
        // Built by another failure entirely, over a place this one never
        // offered.
        let other = failing(here(), WentWrong::NothingAnswered, &[somewhere()])
            .elsewhere()
            .offers()
            .first()
            .cloned()
            .unwrap();

        let refused = failed.take(&other).unwrap_err();
        assert!(
            refused
                .said(&in_english())
                .text()
                .contains("nothing was sent")
        );
        // And the failure comes back, so the same person can be shown the same
        // dialogue rather than an empty one.
        assert_eq!(refused.back().source(), &here());
    }

    /// **A failure line is only as translated as the place inside it.** A
    /// German sentence naming an English provider is a line half its reader
    /// cannot read, and it is not reported as translated.
    #[test]
    fn a_failure_naming_an_untranslated_place_does_not_claim_to_be_translated() {
        let strings = translated(&[(words::NOTHING_ANSWERED, "{source} hat nicht geantwortet")]);
        let said = failing(hosted(), WentWrong::NothingAnswered, &[]).said(&strings);
        assert!(!said.is_translated(), "{said}");
        assert!(said.text().ends_with("hat nicht geantwortet"), "{said}");
    }

    /// **Running out reads as what it is**, and the sentence carries all three
    /// of the things ADR 0009 says a person needs: what happened, what it takes
    /// to fix, and that nothing else about their machine has changed.
    #[test]
    fn running_out_of_money_is_reported_as_a_state_of_an_account() {
        assert_eq!(
            failing(hosted(), WentWrong::RanOut, &[])
                .said(&in_english())
                .text(),
            "nothing was answered by alo, in the EU — the account there has run out, so nothing \
             will be answered until it is paid for, and nothing else about this machine has changed"
        );
    }

    /// **Running out is not a reason to spend somebody's money somewhere else.**
    /// ADR 0008's *never a silent fallback* runs in both directions, and this is
    /// the direction it would be worst in — so what a failed question may do
    /// next is identical whether the place ran out of money or was simply not
    /// running, and the only door onwards is still an offer a person answered.
    #[test]
    fn an_account_that_ran_out_opens_no_door_that_a_broken_one_would_not() {
        let places = [hosted(), somewhere()];
        let ran_out = failing(here(), WentWrong::RanOut, &places);
        let nothing_answered = failing(here(), WentWrong::NothingAnswered, &places);

        assert_eq!(ran_out.elsewhere(), nothing_answered.elsewhere());
        assert_eq!(
            ran_out.nothing_was_sent(&in_english()).text(),
            nothing_answered.nothing_was_sent(&in_english()).text()
        );
        // And holding it permits nothing: the two offers are things to ask
        // somebody, and no `Answering` exists anywhere in this program yet.
        assert_eq!(ran_out.elsewhere().offers().len(), 2);
    }

    /// A refusal the machine could not have made is refused where it is
    /// reported: the door into this crate is the only place that can catch it.
    ///
    /// A machine on this network is the place that cannot have refused a key,
    /// because nothing in this repository reaches one. This machine was on that
    /// list until item 18b gave a person a way to run a service here that has
    /// a key of their own — `wrong.rs` has the reasoning.
    #[test]
    fn a_key_refused_where_nothing_can_send_one_never_becomes_a_failure_at_all() {
        let refused = Answering::chosen(paired(), &SourcePolicy::Anywhere)
            .unwrap()
            .did_not_answer(WentWrong::KeyNotAccepted, &[], &SourcePolicy::Anywhere)
            .unwrap_err();
        assert_eq!(refused, crate::NotWhatFailed::NoKeyThere);
    }
}
