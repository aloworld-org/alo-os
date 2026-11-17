//! What a field in one of this crate's decided lists has to be.
//!
//! `shipped.toml` and `catalogue.toml` both pin an upstream release and record
//! the licence it may be shipped under, and both are refused when a field is
//! not one. These live here rather than in either reader for the reason both
//! those files give about themselves: **a rule written twice is two rules that
//! can disagree**, and a version the shipped list accepted and the catalogue
//! refused would be this repository contradicting itself in a way no test of
//! either file alone could see.
//!
//! # A release is written the way its source writes it
//!
//! `is_a_release` does not decide what 5.6.2 *means*, and nothing here orders
//! two versions. Until 2026-11-17 it took digits and dots only, which refused
//! three releases Flathub lists exactly as they are: `v2.22.4` (a games
//! launcher), `1.2.95.453.g0eeebbed` (a music player) and `3.4.13-linux1`. The
//! owner decided that day that the market offers every kind of application, and
//! a rule that made us rewrite an upstream's release before we could record it
//! would make the pin say something its source never said.
//!
//! So the rule is now **the shape of a release as sources write them**: it
//! starts with a digit (or `v` and a digit), it is one short run of letters,
//! digits and the separators releases use, and no two separators touch. What it
//! still refuses is everything that is not a release at all — a word like
//! `latest`, a space, a path, something that starts like an option, nothing.
//!
//! # A licence is read, not just looked at
//!
//! [`Licence::read`] parses an SPDX expression — identifiers joined by `AND`,
//! `OR` and `WITH`, in brackets where they nest — and notes one thing about it:
//! whether any part of it is `LicenseRef-proprietary`, which is how Flathub
//! records an application whose source is not published. That flag is the
//! reason for reading rather than checking characters: the market labels such
//! an application *not open source* (the owner, 2026-11-17), and a label is
//! only as true as the reading behind it.
//!
//! Flathub writes proprietary terms as `LicenseRef-proprietary=https://…`, the
//! address of the terms after the `=`. That address is kept, and it is the only
//! place an `=` or an address is accepted. **Any other `LicenseRef-…` is
//! refused**, because a reference nobody here has read is a licence nobody
//! here can label.

/// The longest release this reads, in bytes. The longest Flathub listed for
/// the catalogue on 2026-11-17 was twenty; anything near this is not a release.
const THE_LONGEST_RELEASE: usize = 64;

/// The one reference to terms that are not an open licence.
const PROPRIETARY: &str = "LicenseRef-proprietary";

/// Whether this is a release, written as its source writes it.
///
/// At most [`THE_LONGEST_RELEASE`] bytes of ASCII letters, digits and the
/// separators `.`, `-`, `+`, `_` and `~`; starting with a digit, or with `v` and
/// a digit; ending with a letter or a digit; and never two separators side by
/// side.
#[must_use]
pub(crate) fn is_a_release(version: &str) -> bool {
    let separator = |c: char| matches!(c, '.' | '-' | '+' | '_' | '~');
    let digits_from = version
        .strip_prefix('v')
        .or_else(|| version.strip_prefix('V'))
        .unwrap_or(version);
    let starts_with_a_digit = digits_from.starts_with(|c: char| c.is_ascii_digit());
    let ends_well = version.ends_with(|c: char| c.is_ascii_alphanumeric());
    let only_these = version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || separator(c));
    let no_two_together = !version
        .as_bytes()
        .windows(2)
        .any(|two| two.iter().all(|byte| separator(char::from(*byte))));
    version.len() <= THE_LONGEST_RELEASE
        && starts_with_a_digit
        && ends_well
        && only_these
        && no_two_together
}

/// A licence, read as an SPDX expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Licence {
    /// The expression exactly as the list wrote it.
    written: String,
    /// Whether any part of it is `LicenseRef-proprietary`.
    proprietary: bool,
    /// Where the proprietary terms are published, where the list says.
    terms: Option<String>,
}

impl Licence {
    /// Read a licence expression, or [`None`] for anything that is not one.
    ///
    /// Takes identifiers made of ASCII letters, digits, `.` and `-` with an
    /// optional trailing `+`; the operators `AND`, `OR` and `WITH`, in capitals
    /// or in lower case as Flathub writes some of them; brackets; and
    /// `LicenseRef-proprietary`, optionally followed by `=` and an `https://`
    /// address with no space, bracket or quotation mark in it.
    #[must_use]
    pub fn read(text: &str) -> Option<Self> {
        let tokens = tokens(text)?;
        let mut reading = Reading {
            tokens: &tokens,
            at: 0,
            proprietary: false,
            terms: None,
        };
        reading.expression()?;
        if reading.at != tokens.len() {
            return None;
        }
        Some(Self {
            written: text.to_owned(),
            proprietary: reading.proprietary,
            terms: reading.terms,
        })
    }

    /// The expression as the list wrote it.
    #[must_use]
    pub fn written(&self) -> &str {
        &self.written
    }

    /// Whether every part of it is an open licence.
    ///
    /// **False when any part is proprietary**, so an expression joining an open
    /// licence to proprietary terms is labelled as what a person cannot read
    /// all the source of, never as what they can.
    #[must_use]
    pub const fn is_open_source(&self) -> bool {
        !self.proprietary
    }

    /// Where the proprietary terms are published, where the list says.
    #[must_use]
    pub fn terms(&self) -> Option<&str> {
        self.terms.as_deref()
    }
}

/// Whether this reads as a licence expression — [`Licence::read`], answered.
#[must_use]
pub(crate) fn is_a_licence(licence: &str) -> bool {
    Licence::read(licence).is_some()
}

/// One piece of an expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token<'a> {
    /// `(`
    Open,
    /// `)`
    Close,
    /// `AND` or `OR`.
    Joins,
    /// `WITH`.
    With,
    /// A licence or exception identifier.
    Named,
    /// `LicenseRef-proprietary`, and the address of its terms if written.
    Proprietary(Option<&'a str>),
}

/// The expression, cut into its pieces, or [`None`] for a piece that is not
/// one. Only a plain space separates pieces: a newline or a tab is refused,
/// because a licence is one line.
fn tokens(text: &str) -> Option<Vec<Token<'_>>> {
    let mut out = Vec::new();
    let mut rest = text;
    loop {
        rest = rest.trim_start_matches(' ');
        if let Some(after) = rest.strip_prefix('(') {
            out.push(Token::Open);
            rest = after;
            continue;
        }
        if let Some(after) = rest.strip_prefix(')') {
            out.push(Token::Close);
            rest = after;
            continue;
        }
        if rest.is_empty() {
            break;
        }
        let end = rest.find([' ', '(', ')']).unwrap_or(rest.len());
        let (word, after) = rest.split_at(end);
        out.push(token(word)?);
        rest = after;
    }
    Some(out)
}

/// One word of an expression.
fn token(word: &str) -> Option<Token<'_>> {
    match word {
        "AND" | "and" | "OR" | "or" => return Some(Token::Joins),
        "WITH" | "with" => return Some(Token::With),
        _ => {}
    }
    if word == PROPRIETARY {
        return Some(Token::Proprietary(None));
    }
    if let Some(terms) = word
        .strip_prefix(PROPRIETARY)
        .and_then(|rest| rest.strip_prefix('='))
    {
        let address = terms.strip_prefix("https://")?;
        let fits = !address.is_empty()
            && address
                .chars()
                .all(|c| c.is_ascii_graphic() && !matches!(c, '"' | '\'' | '<' | '>' | '\\'));
        return fits.then_some(Token::Proprietary(Some(terms)));
    }
    if word.starts_with("LicenseRef-") || word.starts_with("DocumentRef-") {
        return None;
    }
    let bare = word.strip_suffix('+').unwrap_or(word);
    let fits = bare.starts_with(|c: char| c.is_ascii_alphanumeric())
        && bare
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'));
    fits.then_some(Token::Named)
}

/// Where a reading of the pieces has got to.
struct Reading<'a, 'b> {
    /// Every piece.
    tokens: &'b [Token<'a>],
    /// The next one to read.
    at: usize,
    /// Whether a proprietary piece has been read.
    proprietary: bool,
    /// The address of its terms, if one was written.
    terms: Option<String>,
}

impl<'a> Reading<'a, '_> {
    /// The piece at the reading's place, and the place moved past it.
    fn next(&mut self) -> Option<Token<'a>> {
        let one = *self.tokens.get(self.at)?;
        self.at += 1;
        Some(one)
    }

    /// Whether the next piece is this one, moving past it if it is.
    fn took(&mut self, wanted: Token<'a>) -> bool {
        let is = self.tokens.get(self.at) == Some(&wanted);
        if is {
            self.at += 1;
        }
        is
    }

    /// `term ((AND | OR) term)*`
    fn expression(&mut self) -> Option<()> {
        self.term()?;
        while self.took(Token::Joins) {
            self.term()?;
        }
        Some(())
    }

    /// `( expression ) | licence [WITH exception] | proprietary`
    fn term(&mut self) -> Option<()> {
        match self.next()? {
            Token::Open => {
                self.expression()?;
                self.took(Token::Close).then_some(())
            }
            Token::Named => {
                if self.took(Token::With) {
                    return (self.next()? == Token::Named).then_some(());
                }
                Some(())
            }
            Token::Proprietary(terms) => {
                self.proprietary = true;
                if let Some(terms) = terms {
                    self.terms = Some(terms.to_owned());
                }
                Some(())
            }
            Token::Close | Token::Joins | Token::With => None,
        }
    }
}

#[cfg(test)]
#[path = "fields_tests.rs"]
mod tests;
