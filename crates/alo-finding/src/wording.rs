//! What *contents* means: the words in a file, and how many of them one
//! entry keeps.
//!
//! The plan says it in one line — *no model is asked what a file is about, and
//! contents means the words in the file* — and this is that line as code. A
//! word is a run of letters or digits, in any script, and it is kept in lower
//! case so that *Contract* and *contract* are one word. What is kept is the
//! set of words, each once, sorted, and not the text: an index that held every
//! document whole would be a second copy of every document, and the promise is
//! an index.
//!
//! # Once per file, and at most fifty thousand
//!
//! The set is gathered as the text is read, so a word said a thousand times
//! is held once while gathering as well as after, and the list handed back
//! has no room to spare. At most [`MOST_WORDS`] different words are kept for
//! one file, and they are the **first** ones the file says: a later word is
//! counted and not kept. The first, rather than the alphabetically first,
//! because a bound that always dropped the words beginning with *z* would be
//! a search that could never find some words in any large file, whereas the
//! beginning of a file is a thing a person can be told about.

use std::borrow::Cow;
use std::collections::HashMap;

/// The most different words an index keeps for one file: fifty thousand.
///
/// Prose is not expected to reach it at the megabyte an index reads — a whole
/// English novel of that size says some seventeen thousand different words,
/// and a language that builds its words out of many endings, as Finnish and
/// Hungarian do, is estimated at two to three times as many — so a person's
/// letters, notes and books are kept whole in every language this machine is
/// written for. What reaches it is text a program wrote: logs, tables of
/// numbers, lists of identifiers. The bound keeps what one entry holds in
/// hand to about the words of a megabyte and fifty thousand places, measured
/// in `tests/an_index_that_fits_in_hand.rs`.
pub const MOST_WORDS: usize = 50_000;

/// The words of a text, as an index keeps them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Kept {
    /// The first words the text says, each once, lower case, sorted.
    pub(crate) words: Vec<String>,
    /// How many more different words it says that were not kept.
    pub(crate) unkept: usize,
}

/// The words in this text: each once, in lower case, sorted.
pub(crate) fn words_of(text: &str) -> Vec<String> {
    kept_of(text, usize::MAX).words
}

/// The words in this text, at most `most` of them — the first it says —
/// each once, in lower case, sorted, and how many more there were.
pub(crate) fn kept_of(text: &str, most: usize) -> Kept {
    // Every different word seen, and whether it was one of the first `most`.
    // A word already in lower case is borrowed from the text, so gathering
    // allocates once per different word, never once per occurrence.
    let mut seen: HashMap<Cow<'_, str>, bool> = HashMap::new();
    let mut kept = 0;
    for word in text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
    {
        let lowered = lowered(word);
        if seen.contains_key(lowered.as_ref()) {
            continue;
        }
        let keeping = kept < most;
        if keeping {
            kept += 1;
        }
        seen.insert(lowered, keeping);
    }
    let unkept = seen.len() - kept;
    let mut words = Vec::with_capacity(kept);
    for (word, keeping) in seen {
        if keeping {
            let mut word = word.into_owned();
            word.shrink_to_fit();
            words.push(word);
        }
    }
    words.sort_unstable();
    Kept { words, unkept }
}

/// This word in lower case: borrowed when it already is.
fn lowered(word: &str) -> Cow<'_, str> {
    let already = word.chars().all(|c| {
        let mut lower = c.to_lowercase();
        lower.next() == Some(c) && lower.next().is_none()
    });
    if already {
        Cow::Borrowed(word)
    } else {
        Cow::Owned(word.to_lowercase())
    }
}

/// Whether a sorted list of words holds this one.
pub(crate) fn says(words: &[String], word: &str) -> bool {
    words
        .binary_search_by(|kept| kept.as_str().cmp(word))
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Words are letters and digits in any script, once each, in lower
    /// case, in order.
    #[test]
    fn the_words_of_a_text_are_each_once_in_lower_case_and_in_order() {
        let words = words_of(
            "The contract Anna sent \u{2014} the CONTRACT, in 2026! Za\u{17c}\u{f3}\u{142}\u{107}.",
        );
        assert_eq!(
            words,
            [
                "2026",
                "anna",
                "contract",
                "in",
                "sent",
                "the",
                "za\u{17c}\u{f3}\u{142}\u{107}"
            ]
        );
        assert!(says(&words, "contract"));
        assert!(says(&words, "2026"));
        assert!(
            !says(&words, "Contract"),
            "a query is lowered before asking"
        );
        assert!(!says(&words, "summer"));
        assert!(words_of("... \u{2014} !!!").is_empty());
    }

    /// **The first words said are the ones kept**, a word said again is not
    /// a second word, and every different word past the bound is counted —
    /// whatever case it was said in.
    #[test]
    fn the_first_words_are_kept_and_the_rest_are_counted() {
        let kept = kept_of("zebra apple Zebra ZEBRA mango apple kiwi Mango fig", 3);
        assert_eq!(kept.words, ["apple", "mango", "zebra"]);
        assert_eq!(kept.unkept, 2, "kiwi and fig");

        let all = kept_of("zebra apple Zebra", 3);
        assert_eq!(all.words, ["apple", "zebra"]);
        assert_eq!(all.unkept, 0);

        let none = kept_of("zebra apple", 0);
        assert!(none.words.is_empty());
        assert_eq!(none.unkept, 2);
        assert_eq!(kept_of("", 3).unkept, 0);
    }

    /// **What is kept is held with no room to spare**: the list and every
    /// word in it, whether the word was borrowed or lowered.
    #[test]
    fn what_is_kept_has_no_room_to_spare() {
        let text = "Alpha beta GAMMA delta \u{130}stanbul ".repeat(1000);
        let kept = kept_of(&text, MOST_WORDS);
        assert_eq!(kept.words.len(), 5);
        assert_eq!(kept.words.capacity(), kept.words.len());
        for word in &kept.words {
            assert_eq!(word.capacity(), word.len(), "{word}");
        }
    }

    /// A word is lowered the way `str::to_lowercase` lowers it, including a
    /// Greek capital sigma at the end of a word, and borrowed only when that
    /// would change nothing.
    #[test]
    fn a_word_is_lowered_as_the_standard_library_lowers_it() {
        for word in [
            "already",
            "Mixed",
            "\u{39f}\u{394}\u{39f}\u{3a3}",
            "\u{1c5}x",
            "\u{130}",
        ] {
            assert_eq!(lowered(word), word.to_lowercase(), "{word}");
        }
        assert!(matches!(lowered("already"), Cow::Borrowed(_)));
    }
}
