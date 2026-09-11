//! Whether a record is what it says it is.
//!
//! Every rule `believing.rs` asks of a record file is about its *place*: not a
//! link, root's or the person's, nobody else able to write it. All three are
//! satisfied by a record written whole by whoever already owns the file — and
//! on a managed machine (ADR 0004) *no administrator can act as a person* is a
//! promise a record nobody can check makes thinner than it reads.
//!
//! This is not a signature. Signing needs a key this repository has nowhere to
//! keep yet, and inventing one here would be the kind of decision ADR 0004
//! exists to have made deliberately. It is the one agreement a plausible copy
//! cannot forge cheaply: **what the entries say happened has to agree with what
//! the file's own beginning says about itself.** Two ways it can fail to:
//!
//! - **An entry from before the record begins.** [`crate::Head`] says the
//!   moment the record now starts at, and a shortening keeps only what a rule
//!   still keeps — every entry it leaves is from that moment or after it. An
//!   entry from before it is one no shortening could have left behind.
//! - **Moments that run backwards.** The record is appended to as things
//!   happen, so a later line is a later moment — or the same one, because two
//!   things in one instant are ordinary. A line from *earlier* than the line
//!   before it is an order this machine never writes.
//!
//! # Reported beside the answer, never instead of it
//!
//! A disagreement is not a refusal of the file, and it must not be one: a
//! reader that threw the record away would let whoever replaced it choose
//! between being believed and being unreadable, and unreadable is the better
//! deal. So this is [`crate::Damage`]'s shape — everything that could be read
//! comes back, and this comes back beside it, as whole sentences a person is
//! shown rather than a flag on a struct a surface may forget to draw.
//!
//! # What is deliberately silent
//!
//! A record this crate wrote and shortened itself. [`crate::Writing`] appends
//! moments as they happen and [`crate::Writing::prune`] keeps only entries at
//! or after the moment it writes into the head — so the daemon's own record,
//! shortened however many times, has nothing here to say about it. That is the
//! case easiest to get wrong, and `a_record_this_crate_wrote_and_shortened_is_silent`
//! below is the test that keeps it right.

use std::time::SystemTime;

use alo_strings::{Filling, Said, Strings};

use crate::head::Head;
use crate::words;

/// Where a record and its own beginning disagree.
///
/// Read back beside [`crate::Damage`] by [`crate::Reading`], and empty for
/// every record this crate wrote and shortened itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Disagreement {
    /// Lines holding an entry from before the moment the head says the record
    /// starts at, counting the first line of the file as one.
    before_it_begins: Vec<u64>,
    /// Lines holding an entry from earlier than the entry before it, counted
    /// the same way.
    runs_backwards: Vec<u64>,
}

impl Disagreement {
    /// Whether the record and its beginning agree — which is every record this
    /// machine wrote itself.
    #[must_use]
    pub fn agrees(&self) -> bool {
        self.before_it_begins.is_empty() && self.runs_backwards.is_empty()
    }

    /// Which lines hold an entry from before the record says it begins,
    /// counting the first line of the file as one.
    #[must_use]
    pub fn before_it_begins(&self) -> &[u64] {
        &self.before_it_begins
    }

    /// Which lines hold an entry from earlier than the entry before it,
    /// counted the same way.
    #[must_use]
    pub fn runs_backwards(&self) -> &[u64] {
        &self.runs_backwards
    }

    /// What this says, in the language the person reads: nought, one or two
    /// whole sentences, to be drawn one under another.
    ///
    /// Which lines, are numbers — [`Disagreement::before_it_begins`] and
    /// [`Disagreement::runs_backwards`] — shown beside the sentences rather
    /// than inside them, as [`crate::Damage`]'s are.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Vec<Said> {
        let mut said = Vec::new();
        if !self.before_it_begins.is_empty() {
            said.push(strings.say(&words::BEFORE_IT_BEGINS.key(), &Filling::nothing()));
        }
        if !self.runs_backwards.is_empty() {
            said.push(strings.say(&words::RUNS_BACKWARDS.key(), &Filling::nothing()));
        }
        said
    }
}

/// The reader's side of the check: one entry at a time, as the file is walked.
///
/// Held by [`crate::Reading`] while it reads, so the moments are compared in
/// the one place that knows which line each entry came from — and so the check
/// is a rule about *reading* a record rather than a field in its format, which
/// is what `docs/contracts/record-file.md` gained additively: a reader's rule,
/// and not a byte of the shape.
#[derive(Debug)]
pub(crate) struct Noticing {
    /// The moment the head says the record starts at, where anything has been
    /// removed.
    since: Option<SystemTime>,
    /// The moment of the entry before this one, where there was one.
    previous: Option<SystemTime>,
    /// What has disagreed so far.
    disagreement: Disagreement,
}

impl Noticing {
    /// Ready to read a record that begins the way this head says.
    pub(crate) fn under(head: &Head) -> Self {
        Self {
            since: head.since(),
            previous: None,
            disagreement: Disagreement::default(),
        }
    }

    /// One entry, read whole from this line.
    ///
    /// An entry **at** the moment the record begins is not before it — a
    /// shortening keeps the entry at its own boundary — and an entry at the
    /// same moment as the one before it is ordinary. Only an earlier moment,
    /// in either comparison, is a disagreement.
    pub(crate) fn entry(&mut self, at: SystemTime, line: u64) {
        if self.since.is_some_and(|begins| at < begins) {
            self.disagreement.before_it_begins.push(line);
        }
        if self.previous.is_some_and(|before| at < before) {
            self.disagreement.runs_backwards.push(line);
        }
        self.previous = Some(at);
    }

    /// What disagreed, once the whole file has been walked.
    pub(crate) fn done(self) -> Disagreement {
        self.disagreement
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::keeping::Keeping;
    use crate::reading::Reading;
    use crate::testing::{a_folder_of_our_own, day, in_english, noon, said, turned_away_at};
    use crate::writing::Writing;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// A record on a real disk holding entries at these moments, written the
    /// way the daemon writes one.
    fn a_record_of(what: &str, moments: &[SystemTime]) -> PathBuf {
        let path = a_folder_of_our_own(what).join("record.jsonl");
        let mut writing = Writing::opening(&path).unwrap();
        for at in moments {
            writing.keep(&turned_away_at(*at)).unwrap();
        }
        path
    }

    /// The same record with its first line replaced by a head claiming it was
    /// shortened to this moment — which is the forgery this module is about: a
    /// believable file whose beginning says something its entries deny.
    fn claiming_it_starts_at(path: &Path, since: SystemTime) {
        let text = fs::read_to_string(path).unwrap();
        let head = Head::new().shortened_to(since, Keeping::for_days(30).unwrap());
        let mut forged = head.line().unwrap();
        forged.push_str(text.split_once('\n').unwrap().1);
        fs::write(path, forged).unwrap();
    }

    /// **What this crate writes and shortens, this check is silent about** —
    /// the case easiest to get wrong, and the one that makes the sentences
    /// worth showing when they do appear. A fortnight is written, shortened,
    /// and added to afterwards, and at every step the record is what it says
    /// it is.
    #[test]
    fn a_record_this_crate_wrote_and_shortened_is_silent() {
        let path = a_record_of("its-own", &[]);
        let mut writing = Writing::opening(&path).unwrap();
        for days_ago in (0..14_u32).rev() {
            writing
                .keep(&turned_away_at(noon() - day() * days_ago))
                .unwrap();
        }
        assert!(Reading::at(&path).unwrap().disagreement().agrees());

        writing
            .prune(Keeping::for_days(7).unwrap(), noon())
            .unwrap();
        let shortened = Reading::at(&path).unwrap();
        assert!(!shortened.head().is_whole(), "it really was shortened");
        assert!(shortened.disagreement().agrees());
        assert!(shortened.disagreement().said(&in_english()).is_empty());

        writing.keep(&turned_away_at(noon())).unwrap();
        assert!(Reading::at(&path).unwrap().disagreement().agrees());
    }

    /// **An entry from before the record says it begins is a disagreement**,
    /// said in words — and everything that could be read still comes back,
    /// because a reader that refused the file would make being unreadable the
    /// better forgery.
    #[test]
    fn an_entry_from_before_the_beginning_is_reported_and_everything_still_answers() {
        let path = a_record_of("forged", &[noon() - day() * 2, noon()]);
        claiming_it_starts_at(&path, noon() - day());

        let reading = Reading::at(&path).unwrap();
        assert_eq!(reading.record().len(), 2, "everything readable came back");
        let disagreement = reading.disagreement();
        assert!(!disagreement.agrees());
        assert_eq!(disagreement.before_it_begins(), [2]);
        assert_eq!(disagreement.runs_backwards(), &[] as &[u64]);

        let sentences = disagreement.said(&in_english());
        assert_eq!(sentences.len(), 1);
        let message = said(sentences.first().unwrap());
        assert!(message.contains("not what it says it is"), "{message}");
        assert!(message.contains("before the moment"), "{message}");
    }

    /// **An entry at the moment the record begins is not before it.** A
    /// shortening keeps the entry at its own boundary, so the boundary itself
    /// must read as legitimate — this is the off-by-one that would report
    /// every honestly shortened record on this machine.
    #[test]
    fn an_entry_at_the_moment_the_record_begins_is_not_before_it() {
        let path = a_record_of("boundary", &[noon() - day(), noon()]);
        claiming_it_starts_at(&path, noon() - day());
        assert!(Reading::at(&path).unwrap().disagreement().agrees());
    }

    /// **Moments that run backwards are a disagreement**, and two entries in
    /// one moment are not: two things in one instant are ordinary, and a rule
    /// that reported them would report the machine's own record on a busy
    /// second.
    #[test]
    fn moments_that_run_backwards_are_reported_and_a_shared_moment_is_not() {
        let ordinary = a_record_of("shared", &[noon(), noon(), noon() + day()]);
        assert!(Reading::at(&ordinary).unwrap().disagreement().agrees());

        let backwards = a_record_of("backwards", &[noon(), noon() + day(), noon()]);
        let reading = Reading::at(&backwards).unwrap();
        assert_eq!(reading.disagreement().runs_backwards(), [4]);
        assert_eq!(reading.disagreement().before_it_begins(), &[] as &[u64]);

        let sentences = reading.disagreement().said(&in_english());
        assert_eq!(sentences.len(), 1);
        let message = said(sentences.first().unwrap());
        assert!(message.contains("not what it says it is"), "{message}");
        assert!(message.contains("order"), "{message}");
    }

    /// **A line that cannot be read does not hide a disagreement around it.**
    /// The order is asked of the entries that could be read, across the gap —
    /// or a forger could put one torn line between two reordered ones and be
    /// believed.
    #[test]
    fn a_disagreement_is_noticed_across_a_line_that_could_not_be_read() {
        let path = a_record_of("across", &[noon() + day(), noon(), noon() - day()]);
        let text = fs::read_to_string(&path).unwrap();
        let mut lines = text.lines();
        let (head, first, last) = (
            lines.next().unwrap(),
            lines.next().unwrap(),
            lines.nth(1).unwrap(),
        );
        // The middle entry torn: line 3, of head, entry, entry, entry.
        fs::write(&path, format!("{head}\n{first}\n{{\"at\":\"torn\n{last}\n")).unwrap();

        let reading = Reading::at(&path).unwrap();
        assert!(reading.damage().must_be_looked_at());
        assert_eq!(reading.disagreement().runs_backwards(), [4]);
    }

    /// **Two disagreements are two whole sentences**, drawn one under another —
    /// the join is not punctuation a program can pick for a language it does
    /// not know, exactly as [`crate::Damage`] settled for its two.
    #[test]
    fn two_disagreements_are_two_whole_sentences() {
        let path = a_record_of("both", &[noon(), noon() - day()]);
        claiming_it_starts_at(&path, noon() + day());

        let disagreement = Reading::at(&path).unwrap().disagreement().clone();
        assert_eq!(disagreement.before_it_begins(), [2, 3]);
        assert_eq!(disagreement.runs_backwards(), [3]);

        let sentences = disagreement.said(&in_english());
        assert_eq!(sentences.len(), 2);
        let mut sentences = sentences.iter().map(said);
        let first = sentences.next().unwrap();
        let second = sentences.next().unwrap();
        assert!(first.contains("before the moment") && !first.contains("order"));
        assert!(second.contains("order") && !second.contains("before the moment"));
    }

    /// A shell that never declared this crate's words shows the key and says it
    /// is a bug, rather than being handed English nobody offered to translate.
    #[test]
    fn a_disagreement_nobody_declared_the_words_for_says_so() {
        let path = a_record_of("undeclared", &[noon(), noon() - day()]);
        let disagreement = Reading::at(&path).unwrap().disagreement().clone();
        let strings = Strings::of(alo_strings::Vocabulary::empty());
        let sentences = disagreement.said(&strings);
        assert_eq!(sentences.len(), 1);
        assert!(sentences.first().unwrap().is_a_bug());
    }

    /// A record with nothing wrong says nothing, and there is nothing to draw.
    #[test]
    fn a_record_that_agrees_with_itself_says_nothing() {
        let disagreement = Disagreement::default();
        assert!(disagreement.agrees());
        assert!(disagreement.said(&in_english()).is_empty());
    }
}
