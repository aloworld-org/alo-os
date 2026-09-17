//! One request, as it crosses the door: one line of five words.
//!
//! ```text
//! printers.add 3f6c…(64 hex) 7 1760000000 9a01…(64 hex)
//! ^verb        ^argument     ^approval    ^proof
//!                              ^issued
//! ```
//!
//! A verb from the closed list, its one argument in the one shape it takes, and
//! the token a turn issued for exactly that. Single spaces, nothing before and
//! nothing after, at most [`LONGEST`] bytes of UTF-8. Everything else is refused
//! before any of it is believed — and the verb a refusal names is only ever
//! what the line said, kept through `alo_record::Line`, never something
//! assembled from it.

use crate::approving::Token;
use crate::verbs::SystemVerb;

/// The most bytes a request may be, newline excluded.
///
/// A request is at most a verb name, sixty-four hexadecimal characters, two
/// numbers of twenty digits and sixty-four more characters, and four spaces:
/// well under two hundred. Anything longer is not a request.
pub const LONGEST: usize = 256;

/// A request the door can act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Request {
    /// What is asked for.
    verb: SystemVerb,
    /// The approval it is asked under.
    token: Token,
}

/// Why a line is not a request the door can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotRead {
    /// It is not five words, or not UTF-8, or too long, or its token is not
    /// written as one.
    NotARequest,
    /// It is shaped like a request, and its verb and argument are not one of the
    /// broker's verbs exactly. `asked_for` is the first word, as it arrived.
    NotOneOfItsVerbs {
        /// What stood where the verb goes.
        asked_for: String,
    },
}

impl Request {
    /// A request for this verb under this token.
    #[must_use]
    pub const fn of(verb: SystemVerb, token: Token) -> Self {
        Self { verb, token }
    }

    /// What is asked for.
    #[must_use]
    pub const fn verb(&self) -> &SystemVerb {
        &self.verb
    }

    /// The approval it is asked under.
    #[must_use]
    pub const fn token(&self) -> &Token {
        &self.token
    }

    /// Read one line, without its newline.
    ///
    /// # Errors
    /// [`NotRead`].
    pub fn read(line: &[u8]) -> Result<Self, NotRead> {
        if line.len() > LONGEST {
            return Err(NotRead::NotARequest);
        }
        let line = std::str::from_utf8(line).map_err(|_| NotRead::NotARequest)?;
        let words: Vec<&str> = line.split(' ').collect();
        let [verb, argument, approval, issued, proof] = words.as_slice() else {
            return Err(NotRead::NotARequest);
        };
        let token = Token::read(approval, issued, proof).ok_or(NotRead::NotARequest)?;
        let verb = SystemVerb::read(verb, argument).ok_or_else(|| NotRead::NotOneOfItsVerbs {
            asked_for: (*verb).to_owned(),
        })?;
        Ok(Self { verb, token })
    }

    /// The line, as the side that asks writes it, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        format!(
            "{} {} {}",
            self.verb.name(),
            self.verb.argument().written(),
            self.token.written()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approving::ApprovingKey;
    use crate::arguments::{Identity, Switch};
    use std::time::{Duration, SystemTime};

    /// A request for every verb there is.
    fn every_request() -> Vec<Request> {
        let key = ApprovingKey::of(&[3; 32]);
        let at = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
        let drive = Identity::of_what_was_reported(b"a drive");
        SystemVerb::one_of_each(drive, Switch::On)
            .into_iter()
            .zip(0..)
            .map(|(verb, approval)| Request::of(verb, key.issue(&verb, approval, at)))
            .collect()
    }

    /// **Every verb's request reads back from what the asking side writes**, and
    /// fits well within the longest a request may be.
    #[test]
    fn every_request_reads_back_as_written() {
        for request in every_request() {
            let written = request.written();
            assert!(written.len() <= LONGEST, "{written}");
            assert_eq!(Request::read(written.as_bytes()), Ok(request), "{written}");
        }
    }

    /// **Anything shaped differently is not a request**: a word missing or
    /// extra, a doubled space, a trailing one, a token misspelt, bytes that are
    /// not text, a line too long.
    #[test]
    fn a_line_shaped_differently_is_not_a_request() {
        let Some(request) = every_request().into_iter().next() else {
            return assert!(!every_request().is_empty());
        };
        let written = request.written();
        let words: Vec<&str> = written.split(' ').collect();
        let [verb, argument, approval, issued, proof] = words.as_slice() else {
            return assert_eq!(words.len(), 5);
        };
        for line in [
            format!("{verb} {argument} {approval} {issued}"),
            format!("{written} more"),
            written.replacen(' ', "  ", 1),
            format!("{written} "),
            format!(" {written}"),
            format!("{verb} {argument} {approval} 0{issued} {proof}"),
            format!(
                "{verb} {argument} {approval} {issued} {}",
                proof.to_uppercase()
            ),
            String::new(),
            "x".repeat(LONGEST + 1),
        ] {
            assert_eq!(
                Request::read(line.as_bytes()),
                Err(NotRead::NotARequest),
                "{line}"
            );
        }
        assert_eq!(
            Request::read(&[0xff, 0xfe, b' ', b'a']),
            Err(NotRead::NotARequest)
        );
    }

    /// **A well-shaped line for a verb that is not exactly one of the broker's**
    /// names what it asked for and nothing more.
    #[test]
    fn a_verb_not_on_the_list_is_named_as_it_was_asked_for() {
        let Some(request) = every_request().into_iter().next() else {
            return assert!(!every_request().is_empty());
        };
        let written = request.written();
        for (swapped, asked_for) in [
            (written.replacen("printers.add", "exec", 1), "exec"),
            (
                written.replacen("printers.add", "storage.format", 1),
                "storage.format",
            ),
            (
                written.replacen("printers.add", "network.radio", 1),
                "network.radio",
            ),
        ] {
            assert_eq!(
                Request::read(swapped.as_bytes()),
                Err(NotRead::NotOneOfItsVerbs {
                    asked_for: asked_for.to_owned()
                }),
                "{swapped}"
            );
        }
    }
}
