//! What a person reads when this machine could not reach a model — the whole
//! of it, once.
//!
//! Four lines, in one value, in the order they are read. Two of them are
//! `alo-answering`'s and two are this crate's, and the reason they are gathered
//! here rather than assembled by whoever is drawing them is the guarantee this
//! whole crate exists for: **a telling that was suppressed produces no sentence
//! anywhere.** A surface that built its own four lines out of the failure would
//! be a surface that could draw them on the turn [`crate::Telling`] said
//! nothing about, and the suppression would be advice rather than a mechanism.
//!
//! # The order is not the surface's to choose
//!
//! [`ToldOnce::lines`] answers them in reading order, because the order is part
//! of the sentence ADR 0009 describes — *says so once, where it happened, and
//! continues*:
//!
//! 1. **which part of the machine has stopped**, which nothing else says;
//! 2. **what went wrong and where**, which is `alo-answering`'s;
//! 3. **that nothing was sent anywhere**, which is `alo-answering`'s and is
//!    shown whether or not there was anywhere to send it;
//! 4. **that the machine carries on**, which is the *and continues*.
//!
//! A surface that put the reassurance first and the failure last would be
//! telling somebody not to worry before saying what about.
//!
//! # The door onwards is still open, and it is still a person's
//!
//! [`ToldOnce`] holds the failure, so [`ToldOnce::elsewhere`] and
//! [`ToldOnce::take`] are `alo_answering::Failed`'s own two, unchanged. That is
//! deliberate and it is the constraint the plan sets on this task: *it never
//! chooses another source.* Nothing here takes an offer, ranks one, sorts them
//! or marks one as recommended — ADR 0008's *never a silent fallback* runs in
//! both directions, and *we spent your money elsewhere because the first place
//! was empty* is the worst available version of it.
//!
//! What a suppressed telling does with those offers is nothing, because it does
//! not exist: [`crate::Telling`] consumes the failure and drops it. Nothing is
//! shown, so nobody can answer an offer, so there is nothing to answer with.

use alo_answering::{Answering, Elsewhere, Failed, NotOffered, Offer};
use alo_strings::{Filling, Said, Strings};

use crate::unavailable::Unavailable;
use crate::words;

/// How many lines one telling is.
///
/// Written down so that [`ToldOnce::lines`] and whoever reads its answer cannot
/// disagree about how many there are.
pub const HOW_MANY_LINES: usize = 4;

/// What a person is told, once, about this machine not being able to reach a
/// model.
///
/// Made by [`crate::Telling::about`] and by nothing else. There is no
/// constructor from a sentence, no public field, no `From`, no `Default` and no
/// deserialiser — so what reaches a screen came from a question that was really
/// put somewhere and really was not answered, and it came through the one place
/// that knows whether it has been said before.
///
/// **Deliberately not `Clone`**, which is `alo_answering::Failed`'s rule and
/// for its reason turned one step further: a clone would be a second copy of a
/// telling that was permitted once, and *said once* would hold only for callers
/// who did not think of it.
///
/// ```
/// use alo_answering::{Answering, WentWrong};
/// use alo_models::{InferenceSource, SourcePolicy};
/// use alo_strings::Strings;
/// use alo_telling::{Telling, WhoAsked, telling_words};
///
/// let mut vocabulary = telling_words().expect("this crate's own words");
/// alo_answering::declare_into(&mut vocabulary).expect("and the failure's");
/// alo_models::declare_into(&mut vocabulary).expect("and the place's");
/// let strings = Strings::of(vocabulary);
///
/// let failed = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere)
///     .expect("nothing forbids answering here")
///     .did_not_answer(WentWrong::NothingAnswered, &[], &SourcePolicy::Anywhere)
///     .expect("a runtime that is not running can happen here");
///
/// let mut telling = Telling::nothing_said_yet();
/// let tell = telling.about(failed, WhoAsked::ThePerson);
/// let told = tell.to_say().expect("nothing has said this yet");
///
/// let lines: Vec<String> = told
///     .lines(&strings)
///     .into_iter()
///     .map(alo_strings::Said::into_text)
///     .collect();
/// assert_eq!(lines[0], "The agent cannot answer right now");
/// assert_eq!(lines[1], "nothing answered on this machine");
/// assert!(lines[3].starts_with("You can carry on."));
/// ```
///
/// A telling cannot be made out of a sentence somebody wrote, and this is what
/// says so — there is no such constructor to call:
///
/// ```compile_fail
/// let _ = alo_telling::ToldOnce::saying("your credit has run out");
/// ```
///
/// nor out of an unavailability that was remembered earlier, which would be a
/// telling put back on a screen away from where it happened:
///
/// ```compile_fail
/// fn again(unavailable: alo_telling::Unavailable) -> alo_telling::ToldOnce {
///     alo_telling::ToldOnce::of(unavailable)
/// }
/// ```
///
/// Both are checked by unmarking them: each fails with **E0599, no function or
/// associated item named `saying`** and **`of`** respectively — neither on a
/// typo.
#[derive(Debug, PartialEq, Eq)]
pub struct ToldOnce {
    /// The failure this is about.
    ///
    /// Held whole rather than read apart, so that the sentence about what went
    /// wrong is the one `alo-answering` words and this crate cannot grow a
    /// second account of one moment. It also keeps the offers with the telling,
    /// which is what stops adopting this crate from quietly closing the one
    /// door ADR 0008 leaves open.
    failed: Failed,
}

impl ToldOnce {
    /// Made by [`crate::Telling::about`] and by nothing else.
    pub(crate) fn about(failed: Failed) -> Self {
        Self { failed }
    }

    /// Which unavailability this is: where the question was put, and what went
    /// wrong there.
    #[must_use]
    pub fn unavailable(&self) -> Unavailable {
        Unavailable::of(&self.failed)
    }

    /// The heading: which part of this machine has stopped.
    ///
    /// This crate's own, because every sentence `alo-answering` says is about a
    /// *question* and none of them names the agent — so a person who pressed
    /// the key expecting an assistant is told what has gone quiet.
    #[must_use]
    pub fn the_agent_cannot_answer(&self, strings: &Strings) -> Said {
        strings.say(&words::THE_AGENT_CANNOT_ANSWER.key(), &Filling::nothing())
    }

    /// What went wrong, and where — `alo_answering::Failed::said`, unchanged.
    #[must_use]
    pub fn what_happened(&self, strings: &Strings) -> Said {
        self.failed.said(strings)
    }

    /// That nothing was sent anywhere, and nothing will be unless the person
    /// says so — `alo_answering::Failed::nothing_was_sent`, unchanged.
    #[must_use]
    pub fn nothing_was_sent(&self, strings: &Strings) -> Said {
        self.failed.nothing_was_sent(strings)
    }

    /// That the rest of the machine is unaffected and nothing is waiting.
    ///
    /// This crate's own, and the *and continues* half of ADR 0009's sentence.
    /// It says nothing about buying anything and offers nothing to buy.
    #[must_use]
    pub fn carry_on(&self, strings: &Strings) -> Said {
        strings.say(&words::CARRY_ON.key(), &Filling::nothing())
    }

    /// The whole telling, in the order a person reads it.
    ///
    /// See this module's documentation for why the order is here rather than in
    /// whatever is drawing it.
    #[must_use]
    pub fn lines(&self, strings: &Strings) -> [Said; HOW_MANY_LINES] {
        [
            self.the_agent_cannot_answer(strings),
            self.what_happened(strings),
            self.nothing_was_sent(strings),
            self.carry_on(strings),
        ]
    }

    /// Where else this machine could ask, and where it may not —
    /// `alo_answering::Failed::elsewhere`, unchanged.
    ///
    /// Offered, never chosen. Nothing here ranks them, sorts them or marks one.
    #[must_use]
    pub fn elsewhere(&self) -> &Elsewhere {
        self.failed.elsewhere()
    }

    /// Take one of the offers: the person said yes to this place, this once.
    ///
    /// `alo_answering::Failed::take`, unchanged and still taking `self`, so one
    /// telling is worth at most one attempt somewhere else.
    ///
    /// # Errors
    /// `alo_answering::NotOffered`, carrying the failure back, when the offer
    /// was not one this telling was about.
    pub fn take(self, offer: &Offer) -> Result<Answering, NotOffered> {
        self.failed.take(offer)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_telling_about, here, hosted, in_english, paired, somewhere};
    use alo_answering::WentWrong;

    /// **The whole telling, in the order it is read**, with the two lines this
    /// crate owns around the two it does not.
    #[test]
    fn a_telling_is_four_lines_in_the_order_a_person_reads_them() {
        let strings = in_english();
        let told = a_telling_about(here(), WentWrong::NothingAnswered, &[]);
        let lines = told.lines(&strings);

        assert_eq!(lines[0].text(), "The agent cannot answer right now");
        assert_eq!(lines[1].text(), "nothing answered on this machine");
        assert_eq!(
            lines[2].text(),
            "nothing was sent anywhere, and nothing will be unless you say so"
        );
        assert_eq!(
            lines[3].text(),
            "You can carry on. Nothing else on this machine depends on the agent, and nothing \
             here is waiting for you to do anything about this"
        );
    }

    /// **Every line is a sentence rather than a key.** A telling that reached
    /// somebody as `telling.carry-on` would be this crate's own bug arriving at
    /// the moment their machine already failed them.
    #[test]
    fn every_line_of_every_telling_says_something() {
        let strings = in_english();
        for why in [
            WentWrong::NothingAnswered,
            WentWrong::TookTooLong,
            WentWrong::NothingUsable,
            WentWrong::NoModelThere,
            WentWrong::KeyNotAccepted,
            WentWrong::SentSomewhereElse,
            WentWrong::RanOut,
            WentWrong::HavingTrouble(503),
        ] {
            let told = a_telling_about(hosted(), why, &[]);
            for line in told.lines(&strings) {
                assert!(!line.is_a_bug(), "{why:?}: {line}");
                assert!(!line.text().is_empty(), "{why:?}");
            }
        }
    }

    /// **This crate says two of the four and no more.** With only its own
    /// vocabulary loaded the two headings read and the two in the middle are
    /// keys nothing declares — which is the test that catches this crate
    /// starting to say something `alo-answering` already says.
    #[test]
    fn only_two_of_the_four_lines_are_this_crates_own() {
        let ours = Strings::of(words::telling_words().unwrap());
        let told = a_telling_about(here(), WentWrong::NothingAnswered, &[]);
        let said_by_us = told
            .lines(&ours)
            .into_iter()
            .filter(|line| !line.is_a_bug())
            .count();
        assert_eq!(
            said_by_us, 2,
            "this crate has started saying something alo-answering already says"
        );
    }

    /// **Running out is told exactly as anything else is.** The line about the
    /// account is `alo-answering`'s and says what is so; the lines around it do
    /// not change, offer nothing and sell nothing. A telling that grew a
    /// sentence for this one reason would be the advertisement ADR 0009
    /// rejected, arriving at the moment somebody is least able to ignore it.
    #[test]
    fn running_out_is_told_in_the_same_words_as_anything_else() {
        let strings = in_english();
        let ran_out = a_telling_about(hosted(), WentWrong::RanOut, &[]);
        let broken = a_telling_about(hosted(), WentWrong::NothingAnswered, &[]);

        assert_eq!(
            ran_out.lines(&strings)[0].text(),
            broken.lines(&strings)[0].text()
        );
        assert_eq!(
            ran_out.lines(&strings)[2].text(),
            broken.lines(&strings)[2].text()
        );
        assert_eq!(
            ran_out.lines(&strings)[3].text(),
            broken.lines(&strings)[3].text()
        );
        // And the one line that does differ is the one that says what happened.
        assert_ne!(
            ran_out.lines(&strings)[1].text(),
            broken.lines(&strings)[1].text()
        );
    }

    /// **The door onwards is unchanged and it is still a person's.** The offers
    /// a failure carried are the offers the telling carries: none is chosen,
    /// none is ranked, and running out opens no door that a broken runtime
    /// would not.
    #[test]
    fn a_telling_offers_what_the_failure_offered_and_chooses_nothing() {
        let places = [hosted(), paired()];
        let ran_out = a_telling_about(here(), WentWrong::RanOut, &places);
        let broken = a_telling_about(here(), WentWrong::NothingAnswered, &places);
        assert_eq!(ran_out.elsewhere(), broken.elsewhere());
        assert_eq!(ran_out.elsewhere().offers().len(), 2);

        // Taking one is still an act: it is the person's answer that reaches
        // an `Answering`, and nothing here has one until they give it.
        let offer = ran_out.elsewhere().offers().first().cloned().unwrap();
        let answering = ran_out.take(&offer).unwrap();
        assert_eq!(answering.source(), &hosted());
    }

    /// **An offer from another failure is refused**, and the failure comes back
    /// rather than the person being left looking at a dialogue that has
    /// forgotten what it was about.
    #[test]
    fn an_offer_this_telling_never_made_is_refused_and_nothing_is_lost() {
        let told = a_telling_about(here(), WentWrong::NothingAnswered, &[hosted()]);
        let elsewhere = a_telling_about(here(), WentWrong::NothingAnswered, &[somewhere()]);
        let other = elsewhere.elsewhere().offers().first().cloned().unwrap();

        let refused = told.take(&other).unwrap_err();
        assert_eq!(refused.back().source(), &here());
    }

    /// The identity a telling carries is the identity of the failure it is
    /// about, so what is remembered and what was shown cannot drift apart.
    #[test]
    fn a_telling_knows_which_unavailability_it_is() {
        let told = a_telling_about(hosted(), WentWrong::RanOut, &[]);
        assert_eq!(told.unavailable().source(), &hosted());
        assert_eq!(told.unavailable().why(), WentWrong::RanOut);
    }

    /// **A half-translated telling says so.** The line about what went wrong
    /// names the place inside it, so a German sentence naming an English
    /// provider is not reported as translated — and the two lines this crate
    /// owns have no gaps at all, so each is translated or it is not.
    #[test]
    fn a_telling_is_only_as_translated_as_its_lines() {
        let strings = in_english();
        for line in a_telling_about(here(), WentWrong::NothingAnswered, &[]).lines(&strings) {
            assert!(!line.is_translated(), "{line}");
        }
    }
}
