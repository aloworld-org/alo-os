//! The question as a person is shown it: one sentence, and the two answers
//! under it.
//!
//! The plan's acceptance for this task is a sentence about provenance — *a
//! proposed change is shown as the sentence `alo-turn` renders* — so provenance
//! is what this type is shaped around rather than what its documentation
//! promises.
//!
//! # There is one door, and it is a change that is really waiting
//!
//! [`Asked::of`] takes an `alo_capability::Waiting` and reads it. There is no
//! constructor here that takes a string, a verb, a path or anything else
//! somebody assembled, and no public field, no `From` and no deserialiser
//! either. What that buys is the whole guarantee of this surface:
//!
//! - a `Waiting` holds an `alo_capability::Proposal`, and the only way to one
//!   is `Proposal::checked` — which refuses a read, refuses a change the grants
//!   already say no to, and refuses a question that stands for no time;
//! - the proposal holds the `alo_capability::Call` that was validated against
//!   the closed list of verbs, and its sentence is filled from **those**
//!   arguments;
//! - so the words on this surface are the machine's own account of what it is
//!   about to do, and there is no shape in which a description a model wrote
//!   could arrive here instead.
//!
//! The compile-fail examples on [`Asked`] are that argument as tests: a second
//! door added later stops being a design discussion and starts being a failing
//! build.
//!
//! # A question that has lapsed is never put in front of anybody
//!
//! [`Asked::of`] answers [`None`] once the question has stopped standing. A
//! surface that showed one would be offering a person an answer that cannot be
//! given — `alo_capability::Approvals::approve` refuses it — and the click that
//! follows teaches them that approving is something that sometimes just fails.
//!
//! # And how long is left is a duration, not a sentence
//!
//! [`Asked::lapses_in`] hands back the time left. It is deliberately not
//! worded: alo OS has no way to say *four minutes* in twenty-four languages
//! yet, and a sentence written here would be this crate inventing a time format
//! for the whole system. What a shell draws from it — a countdown, a bar,
//! nothing at all — is the shell's, and when there is one way to word a
//! duration it will be `alo-strings`' and every surface will use it.

use std::time::{Duration, SystemTime};

use alo_capability::{Grantee, ProposalId, Waiting};
use alo_strings::{Filling, Said, Strings};

use crate::words;

/// One change, worded as the person reading it will read it.
///
/// Made from a change that is really waiting, and from nothing else. Cheap to
/// clone and compared by value, so a shell can tell *this is already on the
/// screen* from *this is a different question* without asking the turn again.
///
/// ```
/// use alo_approving::Asked;
/// use alo_capability::Waiting;
/// use alo_strings::{Said, Strings};
/// use std::time::SystemTime;
///
/// fn on_the_screen(waiting: &Waiting, strings: &Strings, now: SystemTime) -> Option<Said> {
///     let asked = Asked::of(waiting, strings, now)?;
///
///     // The sentence is the proposal's own, rendered in the person's language,
///     // and the two answers under it are this crate's only English.
///     assert_eq!(asked.sentence(), &waiting.proposal.sentence(strings));
///     assert!(!asked.approve_said(strings).text().is_empty());
///     assert!(!asked.no_said(strings).text().is_empty());
///     Some(asked.sentence().clone())
/// }
/// ```
///
/// A question cannot be made out of a sentence somebody wrote, and this is what
/// says so — there is no such constructor to call:
///
/// ```compile_fail
/// let asked = alo_approving::Asked::saying("move march.pdf into Archive");
/// ```
///
/// nor can what is on the surface be reached past, because the sentence is not
/// a field anybody outside this crate can touch:
///
/// ```compile_fail
/// fn reworded(asked: alo_approving::Asked) -> alo_strings::Said {
///     asked.sentence
/// }
/// ```
///
/// Both are checked by unmarking them: the first fails with **E0599, no
/// function or associated item named `saying`**, and the second with **E0616,
/// field `sentence` of struct `Asked` is private** — neither on a typo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asked {
    /// The number this question is answered by.
    id: ProposalId,
    /// What the person is being asked to approve, in the language they read.
    ///
    /// Filled by [`Asked::of`] out of the proposal and by nothing else, which
    /// is the whole guarantee this type makes.
    sentence: Said,
    /// Whose question it is: the agent the change would run under.
    agent: Grantee,
    /// How long is left to answer, at the moment it was shown.
    lapses_in: Duration,
}

impl Asked {
    /// A change that is waiting, as the person in front of the machine sees it.
    ///
    /// The only way to an [`Asked`]. Answers [`None`] when the question has
    /// stopped standing, because a question that cannot be answered is not one
    /// to put in front of anybody.
    #[must_use]
    pub fn of(waiting: &Waiting, strings: &Strings, now: SystemTime) -> Option<Self> {
        Some(Self {
            id: waiting.id,
            sentence: waiting.proposal.sentence(strings),
            agent: waiting.proposal.grantee().clone(),
            lapses_in: waiting.proposal.lapses_in(now)?,
        })
    }

    /// The number this question is answered by.
    ///
    /// What a shell carries back with the answer, and what the record keeps
    /// beside what ran — so one approval and one execution are the same thing
    /// in the evidence as well as in the code.
    #[must_use]
    pub fn id(&self) -> ProposalId {
        self.id
    }

    /// The sentence the person is being asked about.
    ///
    /// The proposal's own, filled from the arguments the call was validated
    /// with. Nothing in this crate wrote it and nothing in this crate can
    /// change it.
    #[must_use]
    pub fn sentence(&self) -> &Said {
        &self.sentence
    }

    /// The agent whose change this is.
    ///
    /// Shown because a person approving something is entitled to know which
    /// agent asked: on a machine with several, *approve* means *this one may do
    /// this once*, and a surface that hid whose question it was would be asking
    /// people to answer half of it.
    #[must_use]
    pub fn agent(&self) -> &Grantee {
        &self.agent
    }

    /// How long was left to answer at the moment this was shown.
    ///
    /// Never zero: a question with nothing left does not become an [`Asked`] at
    /// all. Not a sentence, for the reason this module's documentation gives.
    #[must_use]
    pub fn lapses_in(&self) -> Duration {
        self.lapses_in
    }

    /// The answer that carries the change out, in the language the person
    /// reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::approving_words`] answers with
    /// the key, marked `Said::is_a_bug` — the honest answer to *the shell
    /// forgot to declare what this crate can say*.
    #[must_use]
    pub fn approve_said(&self, strings: &Strings) -> Said {
        strings.say(&words::APPROVE.key(), &Filling::nothing())
    }

    /// The answer that stops it, in the language the person reads.
    ///
    /// *No* is a whole answer and nothing asks why, so there is one string here
    /// and no second surface behind it.
    #[must_use]
    pub fn no_said(&self, strings: &Strings) -> Said {
        strings.say(&words::NO.key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{archiving_march, files, hour, in_english, noon, one_waiting, translated};

    /// **The sentence on the surface is the proposal's own.** Not a copy of it,
    /// not a rendering of the verb's name, and not anything this crate wrote:
    /// the same call, asked for the same sentence, gives the same value.
    #[test]
    fn the_sentence_shown_is_the_one_the_proposal_renders() {
        let strings = in_english();
        let (approvals, id) = one_waiting(archiving_march());
        let waiting = approvals.of(id).unwrap();
        let asked = Asked::of(waiting, &strings, noon()).unwrap();

        assert_eq!(asked.sentence(), &waiting.proposal.sentence(&strings));
        assert_eq!(
            asked.sentence().text(),
            "move /home/anna/Invoices/march.pdf into /home/anna/Archive"
        );
        assert_eq!(asked.id(), waiting.id);
        assert_eq!(asked.agent(), &files());
        assert_eq!(asked.lapses_in(), hour());
    }

    /// **The sentence arrives in the language the person reads**, because it is
    /// looked up wherever it is shown rather than kept as text — which is what
    /// makes one proposal one value and not one per language.
    #[test]
    fn the_sentence_is_read_in_the_language_the_person_reads() {
        let (approvals, id) = one_waiting(archiving_march());
        let german = translated();
        let asked = Asked::of(approvals.of(id).unwrap(), &german, noon()).unwrap();
        assert!(asked.sentence().is_translated(), "{}", asked.sentence());
        assert_eq!(
            asked.sentence().text(),
            "/home/anna/Invoices/march.pdf nach /home/anna/Archive verschieben"
        );
        assert!(asked.approve_said(&german).is_translated());
    }

    /// **A question that has lapsed is never put in front of anybody.** There
    /// is nothing to answer, so there is nothing to show — and a surface that
    /// showed it would be offering an answer the machine will refuse.
    #[test]
    fn a_question_that_has_lapsed_is_never_shown() {
        let strings = in_english();
        let (approvals, id) = one_waiting(archiving_march());
        let waiting = approvals.of(id).unwrap();
        assert!(Asked::of(waiting, &strings, noon()).is_some());
        assert!(Asked::of(waiting, &strings, noon() + hour()).is_none());
        assert!(
            Asked::of(waiting, &strings, noon() + hour() + hour()).is_none(),
            "a question that lapsed an hour ago came back"
        );
    }

    /// **How long is left comes off the question**, and shrinks as the question
    /// stands. It is a duration and not a sentence, so nothing here can be read
    /// in a language nobody chose.
    #[test]
    fn how_long_is_left_comes_off_the_question() {
        let strings = in_english();
        let (approvals, id) = one_waiting(archiving_march());
        let half = Duration::from_secs(30 * 60);
        let later = Asked::of(approvals.of(id).unwrap(), &strings, noon() + half).unwrap();
        assert_eq!(later.lapses_in(), half);
        assert!(!later.lapses_in().is_zero());
    }

    /// **The two answers are this crate's only English**, and both of them read
    /// on a machine that declared its list.
    #[test]
    fn the_two_answers_read_and_do_not_read_the_same() {
        let strings = in_english();
        let (approvals, id) = one_waiting(archiving_march());
        let asked = Asked::of(approvals.of(id).unwrap(), &strings, noon()).unwrap();

        let approve = asked.approve_said(&strings);
        let no = asked.no_said(&strings);
        assert!(!approve.is_a_bug(), "{approve}");
        assert!(!no.is_a_bug(), "{no}");
        assert_ne!(approve.text(), no.text());
    }

    /// A shell can tell one question from another without asking the turn
    /// again, which is what a surface that redraws needs and what stops a
    /// second question quietly replacing the one somebody is reading.
    #[test]
    fn two_questions_are_two_values() {
        let strings = in_english();
        let (march, marchs) = one_waiting(archiving_march());
        let (april, aprils) = one_waiting(crate::testing::archiving_april());
        let one = Asked::of(march.of(marchs).unwrap(), &strings, noon()).unwrap();
        let other = Asked::of(april.of(aprils).unwrap(), &strings, noon()).unwrap();
        assert_ne!(one, other);
        assert_eq!(one, one.clone());
    }
}
