//! The applications that would not close, named — and the one road past them.
//!
//! *Names any that refused to close, and never kills one silently* is two
//! clauses, and this file is both of them. [`WouldNotClose`] carries every
//! application that stayed, each said as its own sentence, and it carries what
//! already went, because a person who cancels a log-out at this point has had
//! three applications close and deserves to be told so rather than to find out.
//!
//! # The only road past it is the person's, and it is not a kill
//!
//! [`WouldNotClose::even_so`] takes this value **by value**: a
//! [`crate::MayEnd`] cannot be made without one, and this one cannot exist
//! without the names being in hand. So there is no *log out anyway* a surface
//! could offer before it had something to show, and no forced end that happened
//! without the list existing.
//!
//! And it is still not a kill. Nothing in this crate signals, terminates or
//! kills anything: what happens is that the session ends over an application
//! that said no, which is what the person asked for after reading its name.
//! `tests/nothing_here_reaches_into_an_application.rs` reads the crate for it.

use alo_applications::Application;
use alo_strings::{Filling, Said, Strings};

use crate::ending::MayEnd;
use crate::words;

/// A log-out that stopped: these applications would not close.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WouldNotClose {
    /// The ones that stayed, in the order they were asked.
    would_not: Vec<Application>,
    /// The ones that had already closed by then, in the order they were asked.
    closed: Vec<Application>,
}

impl WouldNotClose {
    /// What the walk found. `pub(crate)`: only a log-out can say this happened.
    pub(crate) const fn of(would_not: Vec<Application>, closed: Vec<Application>) -> Self {
        Self { would_not, closed }
    }

    /// The applications that would not close, in the order they were asked.
    #[must_use]
    pub fn would_not_close(&self) -> &[Application] {
        &self.would_not
    }

    /// The applications that had already closed, in the order they were asked.
    ///
    /// They are gone whatever the person decides next: a log-out that stops is
    /// not a log-out that puts them back, and no system can put back an
    /// application that has saved its work and exited.
    #[must_use]
    pub fn already_closed(&self) -> &[Application] {
        &self.closed
    }

    /// One sentence per application that would not close, in the language the
    /// person reads.
    ///
    /// One each rather than one list: a sentence naming *three applications*
    /// has a plural in it that not every language forms the way English does,
    /// and a surface that has to join names with commas has invented a sentence
    /// nobody translated. Each is the application's identifier inside a sentence
    /// that says what it means.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Vec<Said> {
        self.would_not
            .iter()
            .map(|what| {
                strings.say(
                    &words::WOULD_NOT_CLOSE.key(),
                    &Filling::of("application", what.identifier().to_owned()),
                )
            })
            .collect()
    }

    /// What to say about the ones that went, when any did.
    ///
    /// [`None`] when nothing had closed yet, because *some have already closed*
    /// is not a sentence to show a person for whom nothing has.
    #[must_use]
    pub fn said_about_what_closed(&self, strings: &Strings) -> Option<Said> {
        (!self.closed.is_empty())
            .then(|| strings.say(&words::SOME_HAVE_ALREADY_CLOSED.key(), &Filling::nothing()))
    }

    /// What the road past this is called, so that what a person clicks says what
    /// it costs.
    #[must_use]
    pub fn what_insisting_is_called(strings: &Strings) -> Said {
        strings.say(&words::LOG_OUT_ANYWAY.key(), &Filling::nothing())
    }

    /// The person read the names and said *log out anyway*.
    ///
    /// Consumes the refusal, which is the whole point: the only way to make a
    /// [`MayEnd`] over an application that said no is to be holding the list of
    /// which ones did.
    #[must_use]
    pub fn even_so(self) -> MayEnd {
        MayEnd::the_person_insisted()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::ending::AfterWhat;
    use crate::testing::{an_application, in_english};

    /// **Every application that would not close is said, by identifier**, and
    /// the sentence has nothing else in it to fill.
    #[test]
    fn every_application_that_would_not_close_is_named() {
        let strings = in_english();
        let refused = WouldNotClose::of(
            vec![an_application("org.example.Editor")],
            vec![an_application("org.example.Mail")],
        );
        let said = refused.said(&strings);
        assert_eq!(said.len(), 1);
        let one = said.first().unwrap();
        assert!(one.unfilled().is_empty(), "{one}");
        assert!(one.text().contains("org.example.Editor"), "{one}");
        assert!(!one.text().contains("org.example.Mail"), "{one}");
    }

    /// **What already closed is said only when something had**, because a
    /// person nothing happened to should not be told that something did.
    #[test]
    fn what_already_closed_is_said_only_when_something_did() {
        let strings = in_english();
        let nothing_went =
            WouldNotClose::of(vec![an_application("org.example.Editor")], Vec::new());
        assert_eq!(nothing_went.said_about_what_closed(&strings), None);
        assert!(nothing_went.already_closed().is_empty());

        let some_went = WouldNotClose::of(
            vec![an_application("org.example.Editor")],
            vec![an_application("org.example.Mail")],
        );
        let said = some_went.said_about_what_closed(&strings).unwrap();
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.is_a_bug(), "{said}");
    }

    /// **Insisting is a sentence a person can read before they choose it**, and
    /// it is the only road to a session ending over an application that stayed.
    #[test]
    fn insisting_is_named_and_is_the_only_road_past() {
        let strings = in_english();
        let named = WouldNotClose::what_insisting_is_called(&strings);
        assert!(!named.is_a_bug(), "{named}");
        assert!(named.text().to_lowercase().contains("log out"), "{named}");

        let refused = WouldNotClose::of(vec![an_application("org.example.Editor")], Vec::new());
        assert_eq!(refused.even_so().after(), AfterWhat::ThePersonInsisted);
    }
}
