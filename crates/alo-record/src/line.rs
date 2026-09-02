//! Text as the record keeps it: one line, printable, and bounded.
//!
//! Everything that enters the record as words comes through here, and the
//! reason is that a record is read. It is read in a terminal by somebody
//! answering "what did the agent do this afternoon", and in a security review
//! by somebody who was not there. Text that can move a cursor, clear a line or
//! run off the end of a screen makes a record that shows one thing and says
//! another, and evidence that can be made to lie is not evidence.
//!
//! Most of what arrives is already safe — [`alo_capability::Value`] refuses
//! control characters at the boundary, so a sentence a person approved cannot
//! contain one. The refusals are where it matters: a call that never validated
//! is refused *with the text that arrived in it*, and that text was written by
//! whatever the model was persuaded to send. Applying the same rule to
//! everything is cheaper than remembering which half is trusted.
//!
//! Bounded for the same reason. A refusal naming a path of ten thousand
//! characters is a refusal nobody reads, and a record nobody reads is one
//! somebody turns off.

use std::fmt;

use serde::{Deserialize, Serialize};

/// One line of text, as the record keeps it.
///
/// Made only by [`Line::of`], which is what makes the guarantee worth having:
/// there is no other way to put words into a record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Line(String);

impl Line {
    /// The most characters a line keeps, before it is cut short.
    ///
    /// Long enough for a sentence naming two full paths, which is the longest
    /// thing a person is realistically asked to approve, and short enough that
    /// one entry cannot fill a screen on its own.
    pub const LONGEST: usize = 512;

    /// What is shown when a line was cut short, so that a truncated record
    /// never reads as a complete one.
    const CUT: char = '…';

    /// Take text into the record: control characters out, runs of blank space
    /// collapsed, and cut short if it runs on.
    ///
    /// Control characters become spaces rather than disappearing, so that two
    /// words separated by a newline stay two words instead of silently becoming
    /// one that was never written.
    #[must_use]
    pub fn of(text: &str) -> Self {
        let flattened: String = text
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect();
        let mut line = flattened.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.chars().count() > Self::LONGEST {
            line = line
                .chars()
                .take(Self::LONGEST.saturating_sub(1))
                .chain(std::iter::once(Self::CUT))
                .collect();
        }
        Self(line)
    }

    /// The words, for showing.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether this line says exactly that.
    ///
    /// Exact, like every other identity in the capability model: a query for
    /// what `@files` did must not also answer for `@Files`.
    #[must_use]
    pub fn is(&self, text: &str) -> bool {
        self.0 == text
    }

    /// Whether there are no words at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The refusal path that matters: a call that never validated is recorded
    /// with the text that arrived in it, and that text was written by whatever
    /// the model was persuaded to send. It cannot rewrite what the record
    /// appears to say.
    #[test]
    fn text_that_could_rewrite_the_record_never_gets_into_it() {
        for attempt in [
            "march.pdf\nran: deleted everything",
            "\u{1b}[2Kmoved nothing",
            "march\u{7}.pdf",
            "march\r\ndeleted",
        ] {
            let line = Line::of(attempt);
            assert!(
                !line.as_str().chars().any(char::is_control),
                "{attempt:?} kept a control character: {line:?}"
            );
        }
    }

    /// Two words separated by a newline stay two words. Dropping the character
    /// would join them into one that nobody wrote, which is a different kind of
    /// lie from the one being prevented.
    #[test]
    fn words_split_by_a_control_character_stay_split() {
        assert!(Line::of("march\ndeleted").is("march deleted"));
        assert!(Line::of("  march   deleted  ").is("march deleted"));
        assert!(Line::of("\t\n ").is_empty());
    }

    /// A record nobody can read is a record nobody keeps, so a line that runs
    /// on is cut — and says that it was.
    #[test]
    fn a_line_that_runs_on_is_cut_short_and_says_so() {
        let long = "x".repeat(Line::LONGEST * 2);
        let line = Line::of(&long);
        assert_eq!(line.as_str().chars().count(), Line::LONGEST);
        assert!(line.as_str().ends_with(Line::CUT));

        let exact = "y".repeat(Line::LONGEST);
        assert!(Line::of(&exact).is(&exact));
    }

    /// A line survives being written down and read back, because that is the
    /// only way a record outlives the session that made it.
    #[test]
    fn a_line_survives_being_written_down_and_read_back() {
        let line = Line::of("move /home/anna/Invoices/march.pdf into /home/anna/Archive");
        let written = serde_json::to_string(&line).unwrap_or_default();
        assert!(written.starts_with('"'), "{written}");
        assert_eq!(serde_json::from_str::<Line>(&written).ok(), Some(line));
    }

    /// Identities are matched exactly here too — a question about one agent
    /// must not be answered about another.
    #[test]
    fn a_line_is_matched_exactly() {
        let line = Line::of("@files");
        assert!(line.is("@files"));
        assert!(!line.is("@Files"));
        assert!(!line.is("@file"));
        assert_eq!(line.to_string(), "@files");
    }
}
