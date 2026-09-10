//! What the record answers, as a person reads it.
//!
//! One question, one reading of the file, one account: the lines that answer
//! it, in the order they happened, and what the record says about its own
//! completeness.
//!
//! # The caveats are part of the answer, not a footnote under it
//!
//! An account of what a machine did is worth exactly what the record behind it
//! is worth, and there are four ways for an account to be worth less than it
//! looks: the record can hold nothing that answers the question, it can no
//! longer go all the way back, part of it can be unreadable, and the answer can
//! have been longer than the bound the account was asked for.
//! [`Account::said`] answers with a sentence for each of those that is true,
//! and the first two of them
//! are read together on purpose — *nothing in this machine's record answers
//! that question* beside *this record does not go all the way back* is the
//! difference between **the agent did nothing in March** and **this record does
//! not reach March**, which is the difference somebody is about to draw a
//! conclusion from.
//!
//! **The record's own sentences, not this crate's.** Whether a record is whole
//! is `alo_keeping::Head`'s to say and what could not be read is
//! `alo_keeping::Damage`'s, and both already have the words for it. Only
//! *nothing here answers that* is this crate's, because only this crate knows
//! that a question was asked and matched nothing.
//!
//! # An account is a moment, and it does not refresh itself
//!
//! It holds no path, no file and no way back to one. Asking again is asking
//! [`crate::Recounting`] again, which reads the disk again — so an account
//! cannot quietly become a different account while somebody is reading it, and
//! a stale one is stale in a way its holder chose.

use std::time::SystemTime;

use alo_keeping::{Damage, Head, Keeping, Reading};
use alo_record::Asking;
use alo_strings::{Filling, Said, Strings};

use crate::bounding::AtMost;
use crate::told::Told;
use crate::words;

/// What the record answers, as a person reads it.
///
/// Made from a record that was read off a disk, and from nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// What answered the question, in the order it happened — at most as many
    /// as were asked for, and the most recent of them.
    told: Vec<Told>,
    /// How many entries answered the question, before the bound narrowed it.
    answered: usize,
    /// How many entries the record holds, before the question narrowed it.
    everything: usize,
    /// What the record says about where it begins.
    head: Head,
    /// What could not be read out of it.
    damage: Damage,
}

impl Account {
    /// What this record answers to this question.
    ///
    /// Crate-private: the only public road to an account is
    /// [`crate::Recounting`], which reads the file. That is what stops an
    /// account being assembled out of a record somebody had in memory, which is
    /// the one thing the plan's first acceptance rules out.
    ///
    /// **Bounded, and the bound keeps the newest.** A record is a year long on
    /// a machine that has been working for a year; [`AtMost`] is why there is no
    /// door here that answers with all of it, and why what is kept is the last
    /// of what answered rather than the first.
    pub(crate) fn of(reading: &Reading, asking: &Asking, most: AtMost) -> Self {
        let answered = reading.record().answering(asking).count();
        Self {
            told: reading
                .record()
                .answering(asking)
                .skip(answered.saturating_sub(most.how_many()))
                .map(Told::of)
                .collect(),
            answered,
            everything: reading.record().len(),
            head: reading.head().clone(),
            damage: reading.damage().clone(),
        }
    }

    /// What answered the question, oldest first — the order it happened in,
    /// which is the order somebody reads it in.
    #[must_use]
    pub fn told(&self) -> &[Told] {
        &self.told
    }

    /// How many lines are in front of the person.
    ///
    /// Never more than the [`AtMost`] the account was made under. How many
    /// answered the question altogether is [`Account::how_many_answered`], and
    /// the two are read together wherever the difference matters.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.told.len()
    }

    /// How many entries answered the question, before the bound narrowed it.
    ///
    /// A number beside the account rather than inside a sentence, for
    /// [`Account::how_many_in_the_record`]'s reason: how many is counted
    /// differently in different languages.
    #[must_use]
    pub fn how_many_answered(&self) -> usize {
        self.answered
    }

    /// Whether everything that answered the question is here.
    ///
    /// False where the bound kept the account short. **A surface must not draw
    /// the lines without this**: an account bounded to a screenful and an
    /// account of everything that happened look identical, and only one of them
    /// is the whole answer. [`Account::said`] already puts it in words.
    #[must_use]
    pub fn is_all_that_answered(&self) -> bool {
        self.told.len() == self.answered
    }

    /// Whether nothing in the record answers it.
    ///
    /// **Never on its own an answer of *nothing happened*.** Read
    /// [`Account::goes_all_the_way_back`] beside it, or show
    /// [`Account::said`], which already puts the two together.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.told.is_empty()
    }

    /// How many entries the record holds altogether, before the question
    /// narrowed it.
    ///
    /// A number, beside the account rather than inside a sentence: how many is
    /// counted differently in different languages, and a plural written by hand
    /// here would be one this machine got wrong in most of them.
    #[must_use]
    pub fn how_many_in_the_record(&self) -> usize {
        self.everything
    }

    /// Whether this is still everything that ever happened on this machine.
    #[must_use]
    pub fn goes_all_the_way_back(&self) -> bool {
        self.head.is_whole()
    }

    /// The moment the record begins at, where anything has been removed.
    ///
    /// A moment, not a sentence: how a date is written belongs to the reader's
    /// region, which is not the same thing as their language.
    #[must_use]
    pub fn begins(&self) -> Option<SystemTime> {
        self.head.since()
    }

    /// The rule that removed what came before, where anything was removed.
    #[must_use]
    pub fn under(&self) -> Option<Keeping> {
        self.head.under()
    }

    /// What could not be read out of the record.
    ///
    /// Never stepped over: a line that cannot be read is reported, because
    /// anything able to write a broken line into the file would otherwise be
    /// able to make an entry disappear without leaving a mark.
    #[must_use]
    pub fn damage(&self) -> &Damage {
        &self.damage
    }

    /// Everything a person must read beside the lines: nought to four whole
    /// sentences, to be drawn one under another.
    ///
    /// Not one sentence with the others stuck on the end. `alo-shortcuts`
    /// settled that in item 9c and `alo-keeping` keeps it: the join between two
    /// sentences is not punctuation a program can pick for a language it does
    /// not know.
    ///
    /// The order is the order they are read in: what this record answered,
    /// then what the record is, then what could not be read out of it.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Vec<Said> {
        let mut said = Vec::new();
        if self.is_empty() {
            said.push(strings.say(&words::NOTHING_TO_TELL.key(), &Filling::nothing()));
        }
        if !self.is_all_that_answered() {
            said.push(strings.say(&words::ONLY_THE_MOST_RECENT.key(), &Filling::nothing()));
        }
        said.push(self.head.said(strings));
        said.extend(self.damage.said(strings));
        said
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{
        a_long_afternoon, a_record_at, an_afternoon, answered_here, archived, in_english, noon,
        somewhere_of_our_own,
    };
    use crate::told::Outcome;
    use alo_record::Only;

    /// An account of an afternoon, read back off a real disk.
    fn an_account_of(what: &str, asking: &Asking) -> Account {
        let kept_at = somewhere_of_our_own(what);
        a_record_at(&kept_at, &an_afternoon());
        Account::of(&Reading::at(&kept_at).unwrap(), asking, AtMost::ONE_SITTING)
    }

    /// Everything that happened, in the order it happened — which is the order
    /// somebody reads it in when they ask what the agent did.
    #[test]
    fn an_account_answers_in_the_order_it_happened() {
        let account = an_account_of("in-order", &Asking::anything());
        assert_eq!(account.how_many(), an_afternoon().len());
        assert_eq!(account.how_many_in_the_record(), an_afternoon().len());
        assert!(!account.is_empty());

        let moments: Vec<SystemTime> = account.told().iter().map(Told::at).collect();
        let mut sorted = moments.clone();
        sorted.sort_unstable();
        assert_eq!(moments, sorted);
    }

    /// **A question narrows the account and nothing else does.** What the
    /// record was asked is `alo-record`'s question, put to the record itself —
    /// so an account of one agent's afternoon is that agent's, and how many
    /// entries there are altogether is still answerable beside it.
    #[test]
    fn a_question_narrows_what_is_told_and_not_what_the_record_holds() {
        let account = an_account_of("narrowed", &Asking::anything().by("@files"));
        assert!(account.how_many() < account.how_many_in_the_record());
        assert!(
            account
                .told()
                .iter()
                .all(|told| told.agent().is_some_and(|agent| agent.is("@files")))
        );
        assert_eq!(account.how_many_in_the_record(), an_afternoon().len());
    }

    /// **What was refused reads back as refused**, as a question put to the
    /// record rather than by reading every line — the plan's second acceptance,
    /// asked of a whole account.
    #[test]
    fn what_was_refused_is_a_question_the_account_answers() {
        let account = an_account_of("refused", &Asking::anything().only(Only::Refusals));
        assert!(!account.is_empty());
        assert!(
            account
                .told()
                .iter()
                .all(|told| told.outcome() != Outcome::Ran),
            "something that ran was answered to a question about refusals"
        );
        assert!(
            account
                .told()
                .iter()
                .any(|told| told.outcome() == Outcome::TheGrantsSaidNo)
        );
    }

    /// **Nothing in the record is not nothing happened.** An account with no
    /// lines says so, and says it beside the record's own account of whether it
    /// goes all the way back — because somebody reading the first sentence is
    /// about to conclude something from it.
    #[test]
    fn an_account_with_nothing_in_it_says_so_beside_what_the_record_is() {
        let kept_at = somewhere_of_our_own("nothing");
        a_record_at(&kept_at, &[answered_here()]);
        let reading = Reading::at(&kept_at).unwrap();
        let account = Account::of(
            &reading,
            &Asking::anything().by("@nobody"),
            AtMost::ONE_SITTING,
        );

        assert!(account.is_empty());
        assert_eq!(account.how_many_in_the_record(), 1);
        assert!(account.goes_all_the_way_back());
        assert_eq!(account.begins(), None);
        assert_eq!(account.under(), None);

        let strings = in_english();
        let said = account.said(&strings);
        let (nothing, whole) = (said.first().unwrap(), said.get(1).unwrap());
        assert_eq!(said.len(), 2, "{said:?}");
        assert!(nothing.text().contains("nothing in this machine's record"));
        assert!(!nothing.is_a_bug(), "{nothing}");
        assert!(whole.text().contains("nothing has been removed"));
    }

    /// An account that answers something says nothing about having nothing to
    /// say, and still says what the record is.
    #[test]
    fn an_account_that_answers_something_still_says_what_the_record_is() {
        let kept_at = somewhere_of_our_own("whole");
        a_record_at(&kept_at, &[archived()]);
        let account = Account::of(
            &Reading::at(&kept_at).unwrap(),
            &Asking::anything(),
            AtMost::ONE_SITTING,
        );
        let said = account.said(&in_english());
        assert_eq!(said.len(), 1);
        assert!(
            said.first()
                .is_some_and(|whole| whole.text().contains("nothing has been removed"))
        );
        assert_eq!(account.told().first().map(Told::at), Some(noon()));
    }

    /// **An account is bounded, and it keeps the most recent.** A machine that
    /// has been working for a year holds a year of entries, and an account of
    /// all of them is not an answer anybody reads — so the bound keeps the end
    /// of the record, still oldest first, and says how many answered
    /// altogether.
    #[test]
    fn an_account_holds_the_most_recent_of_what_answered_and_no_more() {
        let kept_at = somewhere_of_our_own("bounded");
        let afternoon = a_long_afternoon(50);
        a_record_at(&kept_at, &afternoon);
        let reading = Reading::at(&kept_at).unwrap();
        let account = Account::of(&reading, &Asking::anything(), AtMost::entries(10).unwrap());

        assert_eq!(account.how_many(), 10);
        assert_eq!(account.how_many_answered(), 50);
        assert_eq!(account.how_many_in_the_record(), 50);
        assert!(!account.is_all_that_answered());
        assert!(!account.is_empty());

        // The last ten, in the order they happened: the newest is last, and the
        // oldest of the ten is the forty-first thing that happened.
        let moments: Vec<SystemTime> = account.told().iter().map(Told::at).collect();
        let mut sorted = moments.clone();
        sorted.sort_unstable();
        assert_eq!(moments, sorted);
        assert_eq!(
            account.told().last().map(Told::at),
            afternoon.last().map(alo_record::Entry::at)
        );
        assert_eq!(
            account.told().first().map(Told::at),
            afternoon.get(40).map(alo_record::Entry::at)
        );
    }

    /// **A bound nobody was told about is a machine that looks idle.** An
    /// account that left something out says so, above what the record says
    /// about itself; one that did not, does not.
    #[test]
    fn an_account_that_left_something_out_says_so() {
        let kept_at = somewhere_of_our_own("bounded-said");
        a_record_at(&kept_at, &a_long_afternoon(4));
        let reading = Reading::at(&kept_at).unwrap();
        let strings = in_english();

        let bounded = Account::of(&reading, &Asking::anything(), AtMost::entries(2).unwrap());
        let said = bounded.said(&strings);
        assert_eq!(said.len(), 2, "{said:?}");
        assert!(
            said.first()
                .is_some_and(|most| most.text().contains("most recent part")),
            "{said:?}"
        );
        assert!(said.first().is_some_and(|most| !most.is_a_bug()));

        // The same record, asked for more than it holds, says nothing about it.
        let whole = Account::of(&reading, &Asking::anything(), AtMost::entries(4).unwrap());
        assert!(whole.is_all_that_answered());
        assert_eq!(whole.said(&strings).len(), 1);
        assert_eq!(whole.how_many(), whole.how_many_answered());
    }

    /// **A line that could not be read is part of the answer.** An account that
    /// quietly dropped it would let anything able to corrupt one line make an
    /// entry disappear without leaving a mark.
    #[test]
    fn a_line_that_could_not_be_read_is_said_rather_than_stepped_over() {
        let kept_at = somewhere_of_our_own("damaged");
        a_record_at(&kept_at, &an_afternoon());
        let text = std::fs::read_to_string(&kept_at).unwrap();
        let lines: Vec<&str> = text
            .lines()
            .enumerate()
            .map(|(at, line)| {
                if at == 2 {
                    "{\"at\":\"halfway through a sentence"
                } else {
                    line
                }
            })
            .collect();
        std::fs::write(&kept_at, format!("{}\n", lines.join("\n"))).unwrap();

        let account = Account::of(
            &Reading::at(&kept_at).unwrap(),
            &Asking::anything(),
            AtMost::ONE_SITTING,
        );
        assert_eq!(account.how_many(), an_afternoon().len() - 1);
        assert!(account.damage().must_be_looked_at());

        let said = account.said(&in_english());
        assert_eq!(said.len(), 2, "{said:?}");
        assert!(
            said.get(1)
                .is_some_and(|wrong| wrong.text().contains("cannot be read")),
            "{said:?}"
        );
    }

    /// **A record that does not go all the way back says so**, in the words
    /// `alo-keeping` already wrote for it — so an absence in the account is not
    /// read as an innocence.
    #[test]
    fn a_record_that_was_shortened_says_where_it_begins() {
        let kept_at = somewhere_of_our_own("shortened");
        std::fs::write(
            &kept_at,
            "{\"format\":1,\"since\":{\"secs_since_epoch\":1760000000,\
             \"nanos_since_epoch\":0},\"under\":{\"for-days\":30}}\n",
        )
        .unwrap();

        let account = Account::of(
            &Reading::at(&kept_at).unwrap(),
            &Asking::anything(),
            AtMost::ONE_SITTING,
        );
        assert!(!account.goes_all_the_way_back());
        assert_eq!(account.begins(), Some(noon()));
        assert_eq!(account.under(), Keeping::for_days(30).ok());

        let said = account.said(&in_english());
        assert_eq!(said.len(), 2, "{said:?}");
        assert!(
            said.first()
                .is_some_and(|nothing| nothing.text().contains("nothing in this machine's record"))
        );
        assert!(
            said.get(1)
                .is_some_and(|since| since.text().contains("does not go all the way back")),
            "{said:?}"
        );
    }

    /// A shell that never declared this crate's words shows the key and says it
    /// is a bug, rather than being handed English nobody offered to translate.
    #[test]
    fn an_account_nobody_declared_the_words_for_says_so() {
        let kept_at = somewhere_of_our_own("undeclared");
        a_record_at(&kept_at, &[archived()]);
        let account = Account::of(
            &Reading::at(&kept_at).unwrap(),
            &Asking::anything().by("@no"),
            AtMost::ONE_SITTING,
        );
        let strings = Strings::of(alo_strings::Vocabulary::empty());
        let said = account.said(&strings);
        assert!(said.iter().all(Said::is_a_bug), "{said:?}");
        assert_eq!(
            said.first().map(Said::text),
            Some("«recounting.nothing-to-tell»")
        );
    }
}
