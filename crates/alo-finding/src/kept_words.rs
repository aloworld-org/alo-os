//! The kept words of one file, **held in one piece**.
//!
//! Task 13 bounded what one file's words may hold and measured what they do
//! hold; the measurement said where the memory actually goes. A word kept as
//! its own `String` costs **twenty-four bytes of place in the list** and an
//! allocation of its own that the allocator rounds up — about thirty-two bytes
//! on a machine measured here, for a word of seven or eight letters. For a
//! hundred and fifty thousand words that is some eight megabytes of bookkeeping
//! around one megabyte of letters.
//!
//! So the words are not a hundred and fifty thousand allocations. They are
//! **two**: every word joined end to end in one piece, and where each one
//! begins beside it. What a word costs is its own bytes and one place.
//!
//! # Why the order is kept, and what that buys
//!
//! The words arrive sorted and are joined in that order, so a search is still a
//! binary search over the words — [`KeptWords::says`] — and not a scan. Nothing
//! about how a search answers changes; only what it reads.
//!
//! # Why eight bytes a word and not four
//!
//! A place could be a `u32`: a file is read up to `alo_files::MOST_READ`, a
//! megabyte, three orders of magnitude below what a `u32` can say. That would
//! save four bytes a word — some six hundred kilobytes on the largest folder
//! task 13 measured, against the eight megabytes this file is here to save.
//!
//! It is not worth what it costs. A `u32` needs a bound on the total length and
//! a branch for a file that exceeds it, and **that branch cannot be reached, so
//! it cannot be tested** — which means it is a branch that would be written
//! wrong and stay wrong. The honest ways out of it are all worse than eight
//! bytes: dropping the words silently, keeping some of them, or cutting a word
//! in half, and each of those is a search that quietly stops finding things.
//! A place is a `usize`, there is no bound, and there is nothing to get wrong.
//!
//! # No `unsafe`
//!
//! Every byte range is taken with `get`, checked like any other slice. The one
//! piece is an ordinary `str` and the places are ordinary numbers.

use serde::{Serialize, Serializer, ser::SerializeSeq};

/// The kept words of one file, joined in one piece with where each begins.
///
/// Built from words already sorted and each held once — which is what
/// `crate::wording` hands over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeptWords {
    /// Every word, in order, joined end to end.
    joined: Box<str>,
    /// Where each word begins in [`Self::joined`], and where the last one
    /// ends — so there is always one more of these than there are words, and
    /// a word is the piece between two of them.
    starts: Box<[usize]>,
}

impl Default for KeptWords {
    /// No words — the same shape an empty list is joined into, so there is
    /// exactly one way to hold nothing and two of them cannot fail to be equal.
    fn default() -> Self {
        Self::of(&[])
    }
}

impl KeptWords {
    /// No words at all.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// These words, joined in one piece.
    ///
    /// They are expected sorted and each once; nothing here sorts them, because
    /// the caller has already done it and doing it twice would hide which one
    /// was responsible for the order a search depends on.
    pub(crate) fn of(words: &[String]) -> Self {
        let whole: usize = words.iter().map(String::len).sum();
        let mut joined = String::with_capacity(whole);
        let mut starts = Vec::with_capacity(words.len() + 1);
        for word in words {
            starts.push(joined.len());
            joined.push_str(word);
        }
        starts.push(joined.len());
        Self {
            joined: joined.into_boxed_str(),
            starts: starts.into_boxed_slice(),
        }
    }

    /// How many words are kept.
    #[must_use]
    pub fn len(&self) -> usize {
        self.starts.len().saturating_sub(1)
    }

    /// Whether none are.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The word in this place, or nothing where there is none.
    #[must_use]
    pub fn get(&self, which: usize) -> Option<&str> {
        let from = *self.starts.get(which)?;
        let to = *self.starts.get(which + 1)?;
        self.joined.get(from..to)
    }

    /// Every word, in the order they are kept — which is sorted.
    pub fn iter(&self) -> impl Iterator<Item = &str> + '_ {
        (0..self.len()).filter_map(|which| self.get(which))
    }

    /// Whether one of the kept words is this one, which is already lower case.
    ///
    /// A binary search, as it was when each word was its own `String`.
    #[must_use]
    pub fn says(&self, word: &str) -> bool {
        let mut low = 0;
        let mut high = self.len();
        while low < high {
            let middle = low + (high - low) / 2;
            match self.get(middle).map(|kept| kept.cmp(word)) {
                Some(std::cmp::Ordering::Less) => low = middle + 1,
                Some(std::cmp::Ordering::Greater) => high = middle,
                Some(std::cmp::Ordering::Equal) => return true,
                None => return false,
            }
        }
        false
    }

    /// What these words hold in hand: the one piece and the places, each
    /// counted by what it asked the machine for.
    ///
    /// This is the index's own count, as task 13's is. It cannot see what the
    /// allocator rounded each of these two requests up to — but there are two
    /// of them rather than one per word, which is the whole point.
    #[must_use]
    pub fn bytes_in_hand(&self) -> usize {
        self.joined.len() + self.starts.len() * size_of::<usize>()
    }

    /// The words as they were held before, each its own `String`.
    ///
    /// For the deprecated way of asking, and for writing the index file, where
    /// they are written as they always were.
    #[must_use]
    pub fn to_vec(&self) -> Vec<String> {
        self.iter().map(str::to_owned).collect()
    }
}

impl From<Vec<String>> for KeptWords {
    /// Words already sorted and each once, taken into one piece.
    ///
    /// The way a fixture and a reader of an index file both say it, so neither
    /// spells out the joining.
    fn from(words: Vec<String>) -> Self {
        Self::of(&words)
    }
}

impl<const N: usize> From<[&str; N]> for KeptWords {
    /// A fixture's words, in the order they are written.
    fn from(words: [&str; N]) -> Self {
        let owned: Vec<String> = words.iter().map(|word| (*word).to_owned()).collect();
        Self::of(&owned)
    }
}

impl Serialize for KeptWords {
    /// Written as the list of words it always was, so an index file is byte
    /// for byte what it was before this type existed.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut writing = serializer.serialize_seq(Some(self.len()))?;
        for word in self.iter() {
            writing.serialize_element(word)?;
        }
        writing.end()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;

    /// Words, as the crate hands them over: sorted, each once.
    fn some(words: &[&str]) -> KeptWords {
        let owned: Vec<String> = words.iter().map(|word| (*word).to_owned()).collect();
        KeptWords::of(&owned)
    }

    /// **Every word comes back exactly as it went in, in order.**
    #[test]
    fn the_words_come_back_as_they_went_in() {
        let kept = some(&["alpha", "beta", "gamma"]);
        assert_eq!(kept.len(), 3);
        assert_eq!(kept.get(0), Some("alpha"));
        assert_eq!(kept.get(1), Some("beta"));
        assert_eq!(kept.get(2), Some("gamma"));
        assert_eq!(kept.get(3), None);
        assert_eq!(kept.iter().collect::<Vec<_>>(), ["alpha", "beta", "gamma"]);
        assert_eq!(kept.to_vec(), ["alpha", "beta", "gamma"]);
    }

    /// **A word that is a piece of its neighbour is still its own word.**
    ///
    /// Joining end to end with nothing between them is the one thing that
    /// could run two words together, and the places are what stop it.
    #[test]
    fn words_that_run_into_each_other_are_still_separate() {
        let kept = some(&["can", "candle", "candlelight"]);
        assert_eq!(kept.get(0), Some("can"));
        assert_eq!(kept.get(1), Some("candle"));
        assert_eq!(kept.get(2), Some("candlelight"));
        assert!(kept.says("can"));
        assert!(kept.says("candle"));
        assert!(!kept.says("cand"));
        assert!(!kept.says("candlelights"));
    }

    /// **A search finds every kept word and nothing else.**
    #[test]
    fn a_search_finds_what_is_there_and_refuses_what_is_not() {
        let words = ["apple", "banana", "cherry", "date", "elderberry"];
        let kept = some(&words);
        for word in words {
            assert!(kept.says(word), "{word} is kept and was not found");
        }
        for word in ["", "a", "apples", "zebra", "bananb", "banan"] {
            assert!(!kept.says(word), "{word} is not kept and was found");
        }
    }

    /// **No words at all is a thing, and says nothing.**
    #[test]
    fn nothing_kept_says_nothing() {
        let kept = KeptWords::none();
        assert!(kept.is_empty());
        assert_eq!(kept.len(), 0);
        assert_eq!(kept.get(0), None);
        assert!(!kept.says("anything"));
        assert_eq!(kept.iter().count(), 0);
        assert_eq!(some(&[]), kept);
    }

    /// **Words in any script survive being joined**, because the pieces are
    /// taken at the boundaries the letters were written at.
    #[test]
    fn words_in_every_script_come_back_whole() {
        let kept = some(&["ελλάδα", "café", "日本語", "мир"]);
        assert_eq!(
            kept.iter().collect::<Vec<_>>(),
            ["ελλάδα", "café", "日本語", "мир"]
        );
        assert!(kept.says("日本語"));
        assert!(kept.says("café"));
    }

    /// **What it holds is its words' own bytes and one place each.**
    ///
    /// The number this task exists to move, asserted rather than described.
    #[test]
    fn what_it_holds_is_the_words_and_one_place_each() {
        let words = ["alpha", "beta", "gamma"];
        let kept = some(&words);
        let letters: usize = words.iter().map(|word| word.len()).sum();
        assert_eq!(
            kept.bytes_in_hand(),
            letters + size_of::<usize>() * (words.len() + 1)
        );
    }

    /// **It is written as the list of words it always was.**
    ///
    /// The index file is byte for byte what it was, and this is why.
    #[test]
    fn it_is_written_as_a_plain_list_of_words() {
        let kept = some(&["alpha", "beta"]);
        let written = serde_json::to_string(&kept).expect("words are written");
        assert_eq!(written, r#"["alpha","beta"]"#);
    }
}
