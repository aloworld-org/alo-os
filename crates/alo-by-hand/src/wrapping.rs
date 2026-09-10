//! Reading a quotation that markdown wrapped.
//!
//! `docs/features.md` writes one promise per line, however long. `docs/by-hand.md`
//! is prose a person reads and its lines wrap at eighty characters like every
//! other document here, so a quotation of forty words arrives with a line break
//! in the middle of it that was never in the promise.
//!
//! Comparing them verbatim would therefore refuse the longest and most specific
//! quotations — the ones least likely to fit two promises — and reward short
//! vague ones. Which would be a check teaching whoever uses it to quote less.
//!
//! So both sides are read as one line first. Nothing else is normalised: the
//! words, the punctuation and the markdown are compared exactly as written,
//! because a quotation that has drifted from what it quotes is the thing this
//! check exists to catch.

/// A piece of text as one line: every run of whitespace becomes one space.
#[must_use]
pub fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A quotation that wrapped reads as the line it was taken from.
    #[test]
    fn a_wrapped_quotation_reads_as_one_line() {
        assert_eq!(
            one_line("A text editor and an image viewer, so a fresh\nmachine is not helpless"),
            "A text editor and an image viewer, so a fresh machine is not helpless"
        );
        assert_eq!(
            one_line("  spaced   out \n\n and indented  "),
            "spaced out and indented"
        );
        assert_eq!(one_line(""), "");
    }

    /// And nothing else is normalised. A quotation that changed a word, a comma
    /// or a piece of markdown is a quotation that no longer matches.
    #[test]
    fn nothing_but_the_whitespace_is_normalised() {
        assert_ne!(one_line("a **terminal**"), one_line("a terminal"));
        assert_ne!(one_line("open, focus, close"), one_line("open focus close"));
        assert_ne!(one_line("A file manager"), one_line("a file manager"));
    }
}
