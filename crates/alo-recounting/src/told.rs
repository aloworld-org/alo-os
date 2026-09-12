//! One thing that happened, as a person reads it back.
//!
//! The plan's acceptance for this task ends on a sentence about provenance —
//! *nothing in the answer is a sentence a model wrote* — so provenance is what
//! this type is shaped around rather than what its documentation promises.
//!
//! # There is one door, and it is an entry that was written down
//!
//! [`Told::of`] takes an `alo_record::Entry` and reads it. There is no
//! constructor here that takes a string, a verb, a moment or anything else
//! somebody assembled, no public field, no `From` and no deserialiser. What
//! that buys is the whole guarantee of this surface:
//!
//! - an `Entry` is made by one of `alo-record`'s named constructors, each of
//!   which is a point in the journey ADR 0001 §5 and §7 describe;
//! - the sentence it carries is `alo_capability::Call::sentence`, filled from
//!   the arguments that survived validation and rendered in the language the
//!   person was reading at the moment it happened;
//! - so what appears on this surface is the machine's own account of what it
//!   did, and there is no shape in which a description a model wrote could
//!   arrive here instead.
//!
//! # This crate does not word anything a person reads about their own machine
//!
//! [`Told::of`] takes no `alo_strings::Strings`, and that absence is the
//! argument. Every line of text on this type is already text: the sentence was
//! worded when it happened, the reason something was refused is the refusal's
//! own words as the person was shown them, and neither can be rendered again
//! here. A surface that re-worded either would be a machine with two accounts
//! of one moment, and the one read afterwards would be the one nothing checked.
//!
//! The only exception is [`Outcome`], which is the clause saying *what became
//! of this* — and it is a value, worded where it is read, out of the closed
//! list in [`crate::words`].
//!
//! # What the model wrote, kept apart from what the machine said
//!
//! One entry does carry text from outside: a verb name that never became a call
//! is whatever the model was persuaded to send, and `alo-record` keeps it —
//! *what did it try* is the question a security review actually asks. It is
//! never the sentence here. It comes back from [`Told::asked_for`] alone, whose
//! name says what it is, while [`Told::sentence`] answers [`None`] — because
//! there is no sentence: nothing was ever validated to generate one from.

use std::time::SystemTime;

use alo_egress::Destination;
use alo_record::{Entry, Happened, Line, Stopped};
use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// What became of one entry.
///
/// Ten, and they are `alo_record::Happened`'s seven with its three ways of
/// being stopped told apart — because *nobody was asked*, *the person said no*
/// and *the grants said no at the last moment* are three different facts about
/// a machine, and an account that flattened them into *refused* would answer
/// none of the questions each one raises.
///
/// Derived in one place, from an exhaustive match. A kind of entry added to the
/// record and not given a clause here is a build that fails rather than a line
/// somebody reads as blank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// A verb ran.
    Ran,
    /// A change the grants already refused, so nobody was interrupted.
    NobodyWasAsked,
    /// A person was shown the sentence and said no.
    ThePersonSaidNo,
    /// The grants were asked at the moment it would have run, and said no.
    TheGrantsSaidNo,
    /// A verb that is not on the list, or an argument that did not validate.
    NeverBecameACall,
    /// A question answered on this machine (ADR 0008).
    AnsweredHere,
    /// A question a rule refused before it was put anywhere.
    NeverPutAnywhere,
    /// The person's grants were not read again, so the service went on under
    /// the list it already had.
    GrantsNotReadAgain,
    /// A turn the machine would not run, because there was no boundary to run
    /// it inside (ADR 0015).
    NotBounded,
    /// Something left this machine (law 1).
    Left,
    /// Something the egress policy refused to let leave.
    HeldBack,
    /// alo OS reached the network with nobody having asked (★ *no telemetry*).
    LeftOnItsOwn,
}

impl Outcome {
    /// What became of this entry, read off the entry itself.
    fn of(happened: &Happened) -> Self {
        match happened {
            Happened::Ran { .. } => Self::Ran,
            Happened::Stopped { how, .. } => match how {
                Stopped::BeforeAnybodyWasAsked(_) => Self::NobodyWasAsked,
                Stopped::ByThePerson => Self::ThePersonSaidNo,
                Stopped::AtTheMoment(_) => Self::TheGrantsSaidNo,
            },
            Happened::TurnedAway { .. } => Self::NeverBecameACall,
            Happened::AnsweredHere { .. } => Self::AnsweredHere,
            Happened::NeverPutAnywhere { .. } => Self::NeverPutAnywhere,
            Happened::GrantsNotReadAgain { .. } => Self::GrantsNotReadAgain,
            Happened::NotBounded { .. } => Self::NotBounded,
            Happened::Left { .. } => Self::Left,
            Happened::HeldBack { .. } => Self::HeldBack,
            Happened::LeftOnItsOwn { .. } => Self::LeftOnItsOwn,
        }
    }

    /// The string this crate declares for it: the key a translator's file is
    /// sorted by, and the English beside it.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::Ran => words::RAN,
            Self::NobodyWasAsked => words::NOBODY_WAS_ASKED,
            Self::ThePersonSaidNo => words::THE_PERSON_SAID_NO,
            Self::TheGrantsSaidNo => words::THE_GRANTS_SAID_NO,
            Self::NeverBecameACall => words::NEVER_BECAME_A_CALL,
            Self::AnsweredHere => words::ANSWERED_HERE,
            Self::NeverPutAnywhere => words::NEVER_PUT_ANYWHERE,
            Self::GrantsNotReadAgain => words::GRANTS_NOT_READ_AGAIN,
            Self::NotBounded => words::NOT_BOUNDED,
            Self::Left => words::LEFT,
            Self::HeldBack => words::HELD_BACK,
            Self::LeftOnItsOwn => words::LEFT_ON_ITS_OWN,
        }
    }

    /// The clause a person reads, in the language they read.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::recounting_words`] answers with
    /// the key, marked `Said::is_a_bug` — the honest answer to *the shell
    /// forgot to declare what this crate can say*.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// One entry, as a person reads it back.
///
/// Made from an entry the record holds, and from nothing else.
///
/// ```
/// use alo_recounting::{Outcome, Told};
/// use alo_record::Entry;
/// use alo_strings::Strings;
///
/// fn on_the_screen(entry: &Entry, strings: &Strings) -> String {
///     let told = Told::of(entry);
///
///     // The clause is this crate's, in the person's language. The sentence
///     // after it is the record's own and is not worded here at all.
///     let clause = told.outcome().said(strings);
///     match told.sentence() {
///         Some(sentence) => format!("{clause} — {sentence}"),
///         None => clause.into_text(),
///     }
/// }
/// ```
///
/// An account cannot be made out of a sentence somebody wrote, and this is what
/// says so — there is no such constructor to call:
///
/// ```compile_fail
/// let told = alo_recounting::Told::saying("the agent tidied your invoices");
/// ```
///
/// nor can what is on the surface be reached past, because the sentence is not
/// a field anybody outside this crate can touch:
///
/// ```compile_fail
/// fn reworded(told: alo_recounting::Told) -> Option<alo_record::Line> {
///     told.sentence
/// }
/// ```
///
/// Both are checked by unmarking them: the first fails with **E0599, no
/// function or associated item named `saying`**, and the second with **E0616,
/// field `sentence` of struct `Told` is private** — neither on a typo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Told {
    /// When it happened.
    at: SystemTime,
    /// Whose authority it was under — absent for the machine's own errand.
    agent: Option<Line>,
    /// What became of it.
    outcome: Outcome,
    /// The verb, by name — absent when nothing became a call.
    verb: Option<Line>,
    /// The sentence the machine generated when it happened.
    ///
    /// Filled by [`Told::of`] out of the entry and by nothing else, which is
    /// the whole guarantee this type makes.
    sentence: Option<Line>,
    /// Why it was refused, in the words the person was shown.
    because: Option<Line>,
    /// The verb name that arrived from outside and never became a call.
    asked_for: Option<Line>,
    /// Where something went, or would have gone.
    went_to: Option<Destination>,
    /// The approval it ran from.
    from_approval: Option<u64>,
    /// The grants it ran against.
    against: Vec<u64>,
}

impl Told {
    /// One entry, as a person reads it back.
    ///
    /// The only way to a [`Told`]. Takes no `alo_strings::Strings`, because
    /// nothing here is worded: see this module's documentation.
    #[must_use]
    pub fn of(entry: &Entry) -> Self {
        let happened = entry.happened();
        Self {
            at: entry.at(),
            agent: entry.agent().cloned(),
            outcome: Outcome::of(happened),
            verb: entry.what().map(|what| what.verb().clone()),
            sentence: entry.what().map(|what| what.sentence().clone()),
            because: happened.why_stopped().cloned(),
            asked_for: match happened {
                Happened::TurnedAway { verb, .. } => Some(verb.clone()),
                _ => None,
            },
            went_to: happened.destination().cloned(),
            from_approval: happened.from_approval(),
            against: happened.against().to_vec(),
        }
    }

    /// When it happened.
    ///
    /// A moment, not a sentence: how a date and a time are written belongs to
    /// the reader's region, which is not the same thing as their language, and
    /// a format written here would be this crate deciding one for the whole
    /// system.
    #[must_use]
    pub fn at(&self) -> SystemTime {
        self.at
    }

    /// Whose authority it was under — [`None`] when nobody's was.
    ///
    /// The one entry with no answer here is the machine's own errand, and
    /// [`None`] is that answer rather than a gap in it: nobody granted alo OS
    /// permission to fetch a model, so a name in this position would be an
    /// authority the record invented. `alo_record::happened` has the whole of
    /// why, and this surface does not get to disagree with it.
    #[must_use]
    pub fn agent(&self) -> Option<&Line> {
        self.agent.as_ref()
    }

    /// What became of it.
    #[must_use]
    pub fn outcome(&self) -> Outcome {
        self.outcome
    }

    /// The verb, by name — [`None`] when nothing ever became a call.
    #[must_use]
    pub fn verb(&self) -> Option<&Line> {
        self.verb.as_ref()
    }

    /// The sentence the machine generated when it happened.
    ///
    /// The words a person read and approved, or would have read for something
    /// nobody is asked about. Nothing in this crate wrote it and nothing in
    /// this crate can change it.
    ///
    /// [`None`] where there was never a call to generate one from — a question
    /// answered here, something that left, and above all a verb that was turned
    /// away, whose text is [`Told::asked_for`] and is deliberately not this.
    #[must_use]
    pub fn sentence(&self) -> Option<&Line> {
        self.sentence.as_ref()
    }

    /// Why it was refused, in the words the person was shown at the time.
    ///
    /// The refusal's own, kept by the record as it was worded then. [`None`]
    /// when nothing was refused, and when a person simply said no: *no* is the
    /// whole answer, and a system that recorded a reason would be a system that
    /// asked for one.
    ///
    /// **It may quote what the model asked for** — *there is no verb called
    /// `…`* is `alo-capability`'s sentence about a verb name that arrived from
    /// outside, and quoting it is what makes the refusal answer *what did it
    /// try*. The sentence around it is the machine's, generated before anything
    /// was asked of it, and what is quoted inside it has been through
    /// `alo_record::Line`, so it can neither be mistaken for a description of
    /// what happened nor rewrite the record it is drawn into.
    #[must_use]
    pub fn because(&self) -> Option<&Line> {
        self.because.as_ref()
    }

    /// The verb name that arrived from outside and never became a call.
    ///
    /// **This is the one thing on this surface that a model wrote**, and it is
    /// here because *what did it try* is the question a security review asks.
    /// It is not a sentence, it is not worded by this machine, and a surface
    /// drawing it must not draw it as though it were: it is what was typed at
    /// alo OS, quoted.
    ///
    /// [`None`] for everything else, which is every entry that became a call.
    #[must_use]
    pub fn asked_for(&self) -> Option<&Line> {
        self.asked_for.as_ref()
    }

    /// Where something went, or would have gone had a rule permitted it.
    ///
    /// `alo-egress`' own value, worded where it is read, so what a person reads
    /// afterwards and what the indicator showed them at the time are one
    /// rendering of one departure. [`None`] for everything that never left.
    #[must_use]
    pub fn went_to(&self) -> Option<&Destination> {
        self.went_to.as_ref()
    }

    /// Which approval it ran from, when it ran from one.
    ///
    /// One of ADR 0001 §7's four answers, and [`None`] for a read is an answer
    /// rather than a gap: a read needs no approval.
    #[must_use]
    pub fn from_approval(&self) -> Option<u64> {
        self.from_approval
    }

    /// Which grants it ran against.
    ///
    /// The last of ADR 0001 §7's four answers, kept as numbers rather than as
    /// handles: a record holds facts about the past, not references into a list
    /// that has moved on.
    #[must_use]
    pub fn against(&self) -> &[u64] {
        &self.against
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{
        an_afternoon, answered_here, archived, declined, fetched_a_model, held_back, hour,
        in_english, left, never_asked, never_put_anywhere, noon, not_bounded, ran_a_read,
        refused_at_the_moment, translated, turned_away,
    };

    /// **The sentence read back is the one the machine wrote down.** Not a
    /// summary, not a rendering of the verb's name, and above all not something
    /// composed here: the same entry, asked twice, says the same thing.
    #[test]
    fn the_sentence_read_back_is_the_one_the_record_kept() {
        let entry = archived();
        let told = Told::of(&entry);
        assert_eq!(told.outcome(), Outcome::Ran);
        assert_eq!(told.at(), noon());
        assert!(told.agent().is_some_and(|agent| agent.is("@files")));
        assert!(told.verb().is_some_and(|verb| verb.is("move_file")));
        assert_eq!(
            told.sentence(),
            entry.what().map(alo_record::What::sentence),
            "the account and the record worded one moment differently"
        );
        assert!(told.sentence().is_some_and(|sentence| {
            sentence.is("move /home/anna/Invoices/march.pdf into /home/anna/Archive")
        }));
        assert!(told.from_approval().is_some());
        assert_eq!(told.against().len(), 2);
        assert_eq!(told.because(), None);
        assert_eq!(told.asked_for(), None);
        assert_eq!(told.went_to(), None);
    }

    /// A read ran under nobody's approval, and the account says so rather than
    /// leaving the question open.
    #[test]
    fn a_read_reads_back_as_having_needed_no_approval() {
        let told = Told::of(&ran_a_read());
        assert_eq!(told.outcome(), Outcome::Ran);
        assert_eq!(told.from_approval(), None);
        assert_eq!(told.against().len(), 1);
        assert!(told.sentence().is_some());
    }

    /// **A turn that was refused reads back as refused**, and which of the three
    /// refusals it was survives the journey — the plan's second acceptance,
    /// asked of one entry.
    #[test]
    fn a_refusal_reads_back_as_a_refusal_and_says_which_one() {
        let never = Told::of(&never_asked());
        assert_eq!(never.outcome(), Outcome::NobodyWasAsked);
        assert!(never.because().is_some());

        let said_no = Told::of(&declined());
        assert_eq!(said_no.outcome(), Outcome::ThePersonSaidNo);
        assert_eq!(
            said_no.because(),
            None,
            "a person is not asked to justify saying no"
        );
        assert!(said_no.sentence().is_some(), "what they declined is kept");

        let last_moment = Told::of(&refused_at_the_moment());
        assert_eq!(last_moment.outcome(), Outcome::TheGrantsSaidNo);
        assert!(
            last_moment
                .because()
                .is_some_and(|why| why.as_str().contains("has not been granted")),
            "{:?}",
            last_moment.because()
        );

        // And no two of them are the same clause, or a person reading their own
        // record could not tell the machine refusing from themselves refusing.
        let strings = in_english();
        let clauses = [never, said_no, last_moment].map(|told| told.outcome().said(&strings));
        assert_ne!(clauses[0].text(), clauses[1].text());
        assert_ne!(clauses[1].text(), clauses[2].text());
        assert_ne!(clauses[0].text(), clauses[2].text());
    }

    /// **What a model wrote is never a sentence in the account.** A verb that
    /// never validated has no sentence — there was nothing to generate one from
    /// — and the text that arrived comes back only from the accessor that says
    /// what it is.
    #[test]
    fn what_a_model_wrote_is_quoted_and_is_never_the_sentence() {
        let told = Told::of(&turned_away());
        assert_eq!(told.outcome(), Outcome::NeverBecameACall);
        assert_eq!(
            told.sentence(),
            None,
            "text from outside became the machine's own sentence"
        );
        assert_eq!(told.verb(), None);
        assert!(
            told.asked_for()
                .is_some_and(|asked| asked.is("tidy_everything")),
            "{:?}",
            told.asked_for()
        );
        assert!(told.because().is_some());
    }

    /// A question answered here kept nothing about the question, and there is
    /// nowhere in the account for one to appear.
    #[test]
    fn a_question_answered_here_reads_back_with_no_question_in_it() {
        let told = Told::of(&answered_here());
        assert_eq!(told.outcome(), Outcome::AnsweredHere);
        assert_eq!(told.sentence(), None);
        assert_eq!(told.went_to(), None);
        assert!(told.agent().is_some_and(|agent| agent.is("@mail")));

        // And one a rule refused before it was put anywhere reads as that, with
        // the rule's own words after it.
        let nowhere = Told::of(&never_put_anywhere());
        assert_eq!(nowhere.outcome(), Outcome::NeverPutAnywhere);
        assert!(nowhere.because().is_some());
        assert_eq!(nowhere.went_to(), None);
    }

    /// **A turn the machine would not run reads back as the machine's own
    /// refusal**: it names the agent, it carries the sentence the person was
    /// shown, and it has no verb, no destination and no approval — because
    /// nothing became any of those. The machine's own account of its boundary
    /// stays on the entry and is not drawn onto this surface, which shows
    /// people what they were shown.
    #[test]
    fn a_turn_with_no_boundary_reads_back_as_the_machine_refusing() {
        let told = Told::of(&not_bounded());
        assert_eq!(told.outcome(), Outcome::NotBounded);
        assert!(told.agent().is_some_and(|agent| agent.is("@files")));
        assert!(
            told.because()
                .is_some_and(|why| why.as_str().starts_with("nothing was done"))
        );
        assert_eq!(told.verb(), None);
        assert_eq!(told.sentence(), None);
        assert_eq!(told.went_to(), None);
        assert_eq!(told.from_approval(), None);
        assert!(told.against().is_empty());
    }

    /// **Law 1, read back.** Something that left says where it went; something
    /// a rule stopped says where it would have gone and that it did not — and
    /// the two are different clauses, because an account that read them the same
    /// would answer *what left this machine* with more than what left it.
    #[test]
    fn what_left_and_what_was_stopped_from_leaving_read_differently() {
        let departed = Told::of(&left());
        assert_eq!(departed.outcome(), Outcome::Left);
        assert!(departed.went_to().is_some());

        let stopped = Told::of(&held_back());
        assert_eq!(stopped.outcome(), Outcome::HeldBack);
        assert!(stopped.went_to().is_some());
        assert!(stopped.because().is_some());

        let strings = in_english();
        assert_ne!(
            departed.outcome().said(&strings).text(),
            stopped.outcome().said(&strings).text()
        );
        assert!(
            stopped
                .outcome()
                .said(&strings)
                .text()
                .contains("nothing left")
        );
    }

    /// **★ No telemetry, read back.** What the machine did on its own is under
    /// nobody's authority, and the account has no name to put there rather than
    /// one it invented.
    #[test]
    fn what_the_machine_did_on_its_own_names_nobody() {
        let told = Told::of(&fetched_a_model());
        assert_eq!(told.outcome(), Outcome::LeftOnItsOwn);
        assert_eq!(told.agent(), None);
        assert!(told.went_to().is_some());
        assert_eq!(told.at(), noon() + hour());
    }

    /// **Every entry the record can hold has a clause**, and no two of them read
    /// the same. A kind of entry with no clause would reach a person as a key.
    #[test]
    fn every_kind_of_entry_reads_as_something_and_no_two_read_alike() {
        let strings = in_english();
        let mut seen: Vec<String> = Vec::new();
        let mut outcomes: Vec<Outcome> = Vec::new();
        for entry in an_afternoon() {
            let told = Told::of(&entry);
            let said = told.outcome().said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(!said.text().is_empty());
            if !outcomes.contains(&told.outcome()) {
                assert!(!seen.contains(&said.text().to_owned()), "two say {said}");
                outcomes.push(told.outcome());
                seen.push(said.text().to_owned());
            }
        }
        assert_eq!(
            outcomes.len(),
            10,
            "an afternoon that does not hold one of every kind of entry proves less than it looks"
        );
    }

    /// **The clause arrives in the language the person reads.** It is this
    /// crate's only English, and the sentence beside it is not translated here
    /// at all: it was worded when it happened.
    #[test]
    fn the_clause_is_read_in_the_language_the_person_reads() {
        let german = translated(&[(words::RAN, "der Agent hat dies getan")]);
        let told = Told::of(&archived());
        let clause = told.outcome().said(&german);
        assert!(clause.is_translated(), "{clause}");
        assert_eq!(clause.text(), "der Agent hat dies getan");

        // The sentence is what it was when it was written down, whichever
        // language the account is read in afterwards.
        assert!(
            told.sentence()
                .is_some_and(|sentence| sentence.as_str().starts_with("move ")),
            "the record was re-worded by whoever read it back"
        );
    }

    /// A shell that never declared this crate's words shows the key and says it
    /// is a bug, rather than being handed English nobody offered to translate.
    #[test]
    fn a_clause_nobody_declared_the_words_for_says_so() {
        let strings = Strings::of(alo_strings::Vocabulary::empty());
        let said = Outcome::Ran.said(&strings);
        assert!(said.is_a_bug());
        assert_eq!(said.text(), "«recounting.outcome.ran»");
    }
}
