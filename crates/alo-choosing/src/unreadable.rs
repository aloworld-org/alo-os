//! What a parser said about a settings file, with everything the file said
//! taken out of it.
//!
//! # The leak this closes
//!
//! `NotSet::NotUnderstood` used to carry a `toml::de::Error` whole, and a TOML
//! error quotes the line it failed on:
//!
//! ```text
//! TOML parse error at line 6, column 1
//!   |
//! 6 | key = "sk-live-0123456789"
//!   | ^^^
//! unknown field `key`
//! ```
//!
//! The sentence a person reads never contained that — it is filled with the
//! path and nothing else — but the value did, so a `Debug` of the refusal, a
//! log line that formatted one, or a support bundle that captured one would
//! have carried somebody's API key. `CLAUDE.md`: *credentials never appear in
//! logs, errors, or commits.* This is that rule made structural rather than
//! remembered.
//!
//! It was found by a test written for the settings file's own protection —
//! there is no `key` field, so a pasted credential is refused — which then
//! showed the refusal repeating what it had just refused to store.
//!
//! # What survives, and the rule that decides
//!
//! **A quoted run in a TOML error is kept only if it is one of this file
//! format's own words.** Everything else is `…`.
//!
//! That is the whole rule, and it is a list rather than a heuristic because a
//! heuristic is what fails on the one input nobody tried. `format`, `answers`,
//! `provider` and the rest are ours: they are in
//! `docs/contracts/person-settings.md`, they are the same on every machine, and
//! knowing which one was wrong is most of what a person needs. A credential is
//! not on the list and cannot be added to it by anything a person types.
//!
//! Both delimiters are treated the same. TOML quotes a *value* with `"` and a
//! *name* with a backtick, so keeping backticked runs would have been nearly
//! right — and nearly right here means a key pasted as a bare name, which TOML
//! accepts, arriving in a log as ``unknown field `sk-live-…` ``.
//!
//! **And the line and column survive whole**, because they are arithmetic about
//! the file rather than anything in it, and because *which line* is what sends
//! somebody to the right place.

use std::fmt;

/// Every word this file format uses for itself.
///
/// Kept beside the shapes in [`crate::written`] rather than derived from them:
/// serde has no way to hand back the names it knows, and a list that has to be
/// updated by hand when a key is added is one a reviewer can see. A word
/// missing from here costs a diagnostic; a word wrongly in here would have to
/// be one somebody could type a credential into, and none of these is.
const OUR_OWN_WORDS: &[&str] = &[
    "format",
    "answers",
    "catalogue",
    "brought",
    "provider",
    "reading",
    "languages",
    "name",
    "endpoint",
    "region",
    "needs-a-key",
    "model",
    "id",
    "bytes-on-disk",
    "quantisation",
    "drives-verbs",
];

/// Where in the file, counted from one because that is how an editor counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct At {
    /// Which line.
    pub line: usize,

    /// Which column.
    pub column: usize,
}

/// What a TOML parser said, with nothing of the file's own in it.
///
/// **No field holds anything the person typed**, which is what makes the
/// `Debug` below safe to derive rather than something a reader has to trust.
#[derive(Clone, PartialEq, Eq)]
pub struct NotToml {
    /// The parser's own words, with every quoted run that is not one of
    /// [`OUR_OWN_WORDS`] replaced.
    said: String,

    /// Where it gave up, when it said.
    at: Option<At>,
}

impl NotToml {
    /// What the parser said about this text, redacted.
    ///
    /// `said` is the file, and it is read **only** to turn a byte offset into a
    /// line and a column. Nothing from it is kept.
    #[must_use]
    pub fn of(why: &toml::de::Error, said: &str) -> Self {
        Self {
            said: without_what_the_file_said(why.message()),
            at: why
                .span()
                .and_then(|span| line_and_column(said, span.start)),
        }
    }

    /// The parser's words, with the file's own taken out.
    #[must_use]
    pub fn said(&self) -> &str {
        &self.said
    }

    /// Where it gave up, when the parser said where.
    #[must_use]
    pub const fn at(&self) -> Option<At> {
        self.at
    }
}

impl fmt::Display for NotToml {
    fn fmt(&self, saying: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.at {
            Some(At { line, column }) => {
                write!(saying, "{} (line {line}, column {column})", self.said)
            }
            None => write!(saying, "{}", self.said),
        }
    }
}

/// The same as [`Display`](fmt::Display), and written by hand so that it stays
/// that way.
///
/// A derived one would print the fields, which is the same text — today. The
/// reason to write it out is that the next field somebody adds here would be
/// printed too, and the whole point of this type is that what it prints was
/// decided rather than defaulted.
impl fmt::Debug for NotToml {
    fn fmt(&self, saying: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(saying, "NotToml({self})")
    }
}

/// The parser's message, with every quoted run that is not one of this
/// format's own words replaced by an ellipsis.
///
/// Both delimiters, and the quotes are kept so that the sentence still reads as
/// one: `unknown field `…`, expected one of `format`, `answers`` says what kind
/// of mistake it was and which words were expected, and says nothing about what
/// was written instead.
fn without_what_the_file_said(message: &str) -> String {
    let mut kept = String::with_capacity(message.len());
    let mut rest = message;
    while let Some(opened) = rest.find(['`', '"']) {
        let delimiter = rest[opened..].chars().next().unwrap_or('`');
        kept.push_str(&rest[..opened]);
        let after = &rest[opened + delimiter.len_utf8()..];
        let Some(closed) = after.find(delimiter) else {
            // An unbalanced quote is the parser saying something this does not
            // understand, and what it might be quoting is the file. Nothing
            // after it survives.
            kept.push(delimiter);
            kept.push('…');
            return kept;
        };
        let inside = &after[..closed];
        kept.push(delimiter);
        if OUR_OWN_WORDS.contains(&inside) {
            kept.push_str(inside);
        } else {
            kept.push('…');
        }
        kept.push(delimiter);
        rest = &after[closed + delimiter.len_utf8()..];
    }
    kept.push_str(rest);
    kept
}

/// A byte offset in this text as a line and a column, both from one.
fn line_and_column(said: &str, offset: usize) -> Option<At> {
    let upto = said.get(..offset.min(said.len()))?;
    let line = upto.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = upto
        .rfind('\n')
        .map_or(upto.len(), |newline| upto.len().saturating_sub(newline + 1))
        + 1;
    Some(At { line, column })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Ok is the failure being reported"
)]
mod tests {
    use super::*;

    /// A key nobody should ever have typed into a settings file, and which this
    /// module exists so that nobody ever reads back out of one.
    const A_SYNTHETIC_SECRET: &str = "sk-live-DO-NOT-LOG-9f3c1a";

    /// **This format's own words survive**, because knowing which key was wrong
    /// is most of what somebody needs to fix their file.
    #[test]
    fn the_words_this_format_uses_for_itself_are_kept() {
        assert_eq!(
            without_what_the_file_said("unknown field `key`, expected one of `format`, `answers`"),
            "unknown field `…`, expected one of `format`, `answers`"
        );
        assert_eq!(
            without_what_the_file_said("missing field `endpoint`"),
            "missing field `endpoint`"
        );
    }

    /// **And anything else does not**, whichever way the parser quoted it.
    ///
    /// A value is quoted with `"` and a name with a backtick, and a credential
    /// pasted as a bare name is a name — so both are treated alike rather than
    /// nearly alike.
    #[test]
    fn nothing_the_file_said_survives_either_kind_of_quote() {
        for message in [
            format!("invalid type: string \"{A_SYNTHETIC_SECRET}\", expected a boolean"),
            format!("unknown field `{A_SYNTHETIC_SECRET}`, expected one of `format`"),
            format!("duplicate key `{A_SYNTHETIC_SECRET}`"),
            format!("something nobody has seen \"{A_SYNTHETIC_SECRET}"),
        ] {
            let redacted = without_what_the_file_said(&message);
            assert!(
                !redacted.contains(A_SYNTHETIC_SECRET),
                "the credential survived: {redacted}"
            );
        }
    }

    /// **The line and the column are arithmetic about the file rather than
    /// anything in it**, so they survive whole — and they are what sends
    /// somebody to the right place.
    #[test]
    fn where_it_gave_up_is_counted_from_one() {
        let said = "format = 1\n\n[answers]\nkey = \"x\"\n";
        assert_eq!(line_and_column(said, 0), Some(At { line: 1, column: 1 }));
        assert_eq!(line_and_column(said, 12), Some(At { line: 3, column: 1 }));
        // A newline itself belongs to the line it ends.
        assert_eq!(
            line_and_column(said, 10),
            Some(At {
                line: 1,
                column: 11
            })
        );
    }

    /// **Neither rendering of this type can carry a credential**, which is the
    /// whole claim: a `Debug` is what a log line reaches for.
    #[test]
    fn neither_display_nor_debug_can_carry_a_credential() {
        // The real road: the text a person wrote, through the door this crate
        // reads settings with, so what is redacted is what a parser really
        // said rather than a message this test invented.
        let said = format!(
            "format = 2\n\n[[provider]]\nname = \"Mistral\"\n\
             endpoint = \"https://api.mistral.ai\"\nkey = \"{A_SYNTHETIC_SECRET}\"\n"
        );
        let refused = crate::written::read(&said, std::path::Path::new("/tmp/settings.toml"))
            .expect_err("a key nobody declared is refused");
        let crate::refusing::NotSet::NotUnderstood { why, .. } = &refused else {
            unreachable!("a key nobody declared is text that is not settings: {refused:?}")
        };

        assert!(!format!("{why}").contains(A_SYNTHETIC_SECRET), "{why}");
        assert!(!format!("{why:?}").contains(A_SYNTHETIC_SECRET), "{why:?}");
        // And the whole refusal, which is what a log line would reach for.
        assert!(!format!("{refused:?}").contains(A_SYNTHETIC_SECRET));

        // What survives is worth having: which line, and that a key nobody
        // declared is what went wrong.
        assert_eq!(why.at(), Some(At { line: 6, column: 1 }));
        assert!(why.said().contains("unknown field"), "{why}");
    }
}
