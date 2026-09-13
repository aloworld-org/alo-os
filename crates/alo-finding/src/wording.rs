//! What *contents* means: the words in a file.
//!
//! The plan says it in one line — *no model is asked what a file is about, and
//! contents means the words in the file* — and this is that line as code. A
//! word is a run of letters or digits, in any script, and it is kept in lower
//! case so that *Contract* and *contract* are one word. What is kept is the
//! set of words, each once, sorted, and not the text: an index that held every
//! document whole would be a second copy of every document, and the promise is
//! an index.

/// The words in this text: each once, in lower case, sorted.
pub(crate) fn words_of(text: &str) -> Vec<String> {
    let mut words: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect();
    words.sort_unstable();
    words.dedup();
    words
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
}
