//! What could not be read back out of a record.
//!
//! A record is evidence, so a line in it that cannot be read is not a line to
//! step over. `alo-record` has no way to lose an entry — there is no `remove`,
//! no `edit` and no `forget` — and a reader that silently skipped what it could
//! not parse would hand that guarantee back: anything able to write a broken
//! line into the file would be able to make an entry disappear without leaving
//! a mark.
//!
//! So reading answers with what it read **and** with this, and nothing that
//! could not be read is dropped quietly.
//!
//! # Two different things, and only one of them is alarming
//!
//! A line in the middle of a record that does not parse was written whole and
//! is not whole now. That is disk trouble or somebody's hand, and it is why
//! [`crate::Writing::prune`] refuses to shorten a record that has one: rewriting
//! the file would tidy the evidence away, and the tidying would be the last
//! thing anybody could have looked at.
//!
//! The **last** line, cut off partway with no newline after it, is what a
//! machine losing power in the middle of a write looks like. Nothing complete
//! was lost, because a line is written and flushed to the disk before the next
//! one starts. That one is ordinary, it is tolerated, and pruning drops it.
//!
//! # Not a sentence with the other stuck on the end
//!
//! [`Damage::said`] answers with nought, one or two whole sentences to be drawn
//! one under another. `alo-shortcuts` settled this in item 9c and `alo-models`
//! kept it in 9f: the join between two sentences is not punctuation a program
//! can pick for a language it does not know.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// What could not be read back.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Damage {
    /// Which lines could not be read, counting the first line of the file as
    /// one, in the order they appear.
    unreadable: Vec<u64>,
    /// Whether the last line stops partway through.
    unfinished: bool,
}

impl Damage {
    /// Nothing wrong.
    pub(crate) fn none() -> Self {
        Self::default()
    }

    /// Note a line that is there and cannot be read.
    pub(crate) fn unreadable_at(&mut self, line: u64) {
        self.unreadable.push(line);
    }

    /// Note that the last line stops partway through.
    pub(crate) fn ends_partway(&mut self) {
        self.unfinished = true;
    }

    /// Whether the whole record read back.
    #[must_use]
    pub fn nothing_wrong(&self) -> bool {
        self.unreadable.is_empty() && !self.unfinished
    }

    /// Which lines are there and cannot be read, counting the first line of the
    /// file as one.
    #[must_use]
    pub fn unreadable(&self) -> &[u64] {
        &self.unreadable
    }

    /// How many lines are there and cannot be read.
    #[must_use]
    pub fn how_many_unreadable(&self) -> usize {
        self.unreadable.len()
    }

    /// Whether the last line stops partway through, which is what an
    /// interrupted machine leaves behind.
    #[must_use]
    pub fn last_line_is_unfinished(&self) -> bool {
        self.unfinished
    }

    /// Whether this is the kind of damage a record is looked at for rather than
    /// pruned through.
    #[must_use]
    pub fn must_be_looked_at(&self) -> bool {
        !self.unreadable.is_empty()
    }

    /// What this says, in the language the person reads: nought, one or two
    /// whole sentences, to be drawn one under another.
    ///
    /// How many lines, and which, are numbers — [`Damage::unreadable`] — shown
    /// beside the sentence rather than inside it.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Vec<Said> {
        let mut said = Vec::new();
        if self.must_be_looked_at() {
            said.push(strings.say(&words::UNREADABLE.key(), &Filling::nothing()));
        }
        if self.unfinished {
            said.push(strings.say(&words::UNFINISHED.key(), &Filling::nothing()));
        }
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
    use crate::testing::{in_english, said};

    /// A record that read back whole says nothing, and there is nothing to
    /// draw.
    #[test]
    fn a_record_that_read_back_whole_says_nothing() {
        let damage = Damage::none();
        assert!(damage.nothing_wrong());
        assert!(!damage.must_be_looked_at());
        assert!(damage.said(&in_english()).is_empty());
        assert_eq!(damage.how_many_unreadable(), 0);
    }

    /// **A line that cannot be read is a line somebody has to look at**, and
    /// the sentence says so rather than reporting a count of skipped lines.
    #[test]
    fn a_line_that_cannot_be_read_asks_to_be_looked_at() {
        let mut damage = Damage::none();
        damage.unreadable_at(4);
        damage.unreadable_at(9);
        assert!(!damage.nothing_wrong());
        assert!(damage.must_be_looked_at());
        assert_eq!(damage.unreadable(), [4, 9]);
        assert_eq!(damage.how_many_unreadable(), 2);

        let sentences = damage.said(&in_english());
        assert_eq!(sentences.len(), 1);
        let message = said(sentences.first().unwrap());
        assert!(message.contains("not all of what happened"), "{message}");
        assert!(!message.contains('4'), "how many is not in the sentence");
    }

    /// An interrupted write is ordinary and reads that way — and it is not the
    /// same thing as a line that was written whole and is not whole now.
    #[test]
    fn a_write_the_machine_interrupted_reads_as_what_it_is() {
        let mut damage = Damage::none();
        damage.ends_partway();
        assert!(!damage.nothing_wrong());
        assert!(
            !damage.must_be_looked_at(),
            "an interrupted write is not a record to hold back from pruning"
        );
        let sentences = damage.said(&in_english());
        assert_eq!(sentences.len(), 1);
        assert!(said(sentences.first().unwrap()).contains("losing power"));
    }

    /// **Two things wrong are two sentences**, never one with the other stuck
    /// on the end: the join is not punctuation a program can pick.
    #[test]
    fn two_things_wrong_are_two_whole_sentences() {
        let mut damage = Damage::none();
        damage.unreadable_at(2);
        damage.ends_partway();
        let sentences = damage.said(&in_english());
        assert_eq!(sentences.len(), 2);
        // Each says one thing and nothing of the other's: a reader draws them
        // one under another, and whoever translates them never has to guess
        // what joins them.
        let mut sentences = sentences.iter().map(said);
        let first = sentences.next().unwrap();
        let second = sentences.next().unwrap();
        assert!(first.contains("cannot be read") && !first.contains("power"));
        assert!(second.contains("power") && !second.contains("cannot be read"));
    }

    /// A shell that never declared this crate's words shows the key and says it
    /// is a bug, rather than being handed English nobody offered to translate.
    #[test]
    fn damage_nobody_declared_the_words_for_says_so() {
        let strings = Strings::of(alo_strings::Vocabulary::empty());
        let mut damage = Damage::none();
        damage.unreadable_at(1);
        let sentences = damage.said(&strings);
        assert_eq!(sentences.len(), 1);
        assert!(sentences.first().is_some_and(Said::is_a_bug));
    }
}
