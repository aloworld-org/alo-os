//! One line out: it was done, or it was not and this is which.
//!
//! Three answers, as `alo-broker`'s door has three, and for its reason: *one
//! line in and one line out* is a door a caller can be written against without
//! reading this crate.
//!
//! # Why a refusal is a kind and not a sentence
//!
//! Nothing that crosses this door is a sentence a person reads. What a person
//! reads about an application verb is `alo-applications`' own, rendered in
//! their language before the verb ever reached a window — so a sentence here
//! would be a second wording of something already said, in whatever language
//! the compositor happened to think in.
//!
//! So what crosses is which kind it was, and the daemon turns that into the
//! words it already has. One vocabulary, one place, as everywhere else.

use crate::acting::Acting;

/// What the compositor did, or did not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// It was done.
    Acted,
    /// It was not, and this is which way.
    NotActed(NotActed),
}

/// Why an act this door understood did nothing.
///
/// Each of these is a fact about the machine rather than a refusal by the
/// capability model: by the time a line reaches this door the grants have been
/// asked and the person has approved. So **none of them means *not
/// permitted*** — that answer is given long before, at the daemon, and a door
/// that could give it would be a second place deciding it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotActed {
    /// There is no window of that application to act on.
    ///
    /// For `focus` and `close`, which need one. Never for `open`, which is
    /// allowed to start it.
    #[error("that application has no window here")]
    NoWindow,
    /// The application is not installed, so there is nothing to start.
    #[error("that application is not installed here")]
    NotInstalled,
    /// It was asked and did not do it.
    ///
    /// An application entitled to say *are you sure* to the person is an
    /// application that has not closed, and that is not a failure — which is
    /// why this is told apart from the compositor being unable to ask.
    #[error("the application was asked and has not done it")]
    ItDidNot,
    /// The compositor could not ask.
    ///
    /// The bus, the protocol, or the compositor's own state. The machine rather
    /// than the application, and the one of these that is worth somebody
    /// looking at a log for.
    #[error("this machine could not ask the application")]
    CouldNotAsk,
}

impl Answer {
    /// The line this crosses as.
    #[must_use]
    pub const fn written(&self) -> &'static str {
        match self {
            Self::Acted => "acted",
            Self::NotActed(why) => why.on_the_wire(),
        }
    }

    /// One line, read.
    ///
    /// # Returns
    /// [`None`] for a word this crate does not know, which a caller reads as a
    /// compositor newer or older than itself — never as *it worked*. That
    /// asymmetry is deliberate: an unknown answer must not be optimistic.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        let line = line.trim();
        if line == "acted" {
            return Some(Self::Acted);
        }
        NotActed::off_the_wire(line).map(Self::NotActed)
    }
}

impl NotActed {
    /// Every reason, in the order this file declares them.
    pub const EVERY_ONE: [Self; 4] = [
        Self::NoWindow,
        Self::NotInstalled,
        Self::ItDidNot,
        Self::CouldNotAsk,
    ];

    /// The word this crosses as.
    #[must_use]
    pub const fn on_the_wire(self) -> &'static str {
        match self {
            Self::NoWindow => "no-window",
            Self::NotInstalled => "not-installed",
            Self::ItDidNot => "it-did-not",
            Self::CouldNotAsk => "could-not-ask",
        }
    }

    /// The reason a word off the wire is, if this crate knows it.
    #[must_use]
    pub fn off_the_wire(word: &str) -> Option<Self> {
        Self::EVERY_ONE
            .into_iter()
            .find(|why| why.on_the_wire() == word)
    }

    /// Whether this is worth somebody reading a log about.
    ///
    /// True for one of the four. The other three are ordinary answers about an
    /// application — it has no window, it is not installed, it declined — and a
    /// machine that logged those as faults would bury the one that is.
    #[must_use]
    pub const fn is_this_machines_fault(self) -> bool {
        matches!(self, Self::CouldNotAsk)
    }
}

/// Which reasons an act can honestly meet.
///
/// `open` may start an application, so it never answers [`NotActed::NoWindow`];
/// `focus` and `close` need a window and can. Held here rather than remembered,
/// because a door that answered *no window* to an `open` would be telling a
/// person something that is not a reason.
#[must_use]
pub fn could_answer(acting: &Acting, why: NotActed) -> bool {
    match acting {
        Acting::Open(_) => why != NotActed::NoWindow,
        Acting::Focus(_) | Acting::Close(_) => true,
    }
}

#[cfg(test)]
// Counted: one `expect` and no `unwrap` or `panic!`, so only the one it uses —
// an unfulfilled expectation is an error in this workspace.
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::{Answer, NotActed, could_answer};
    use crate::acting::Acting;
    use alo_applications::Application;

    fn mail() -> Application {
        Application::identified("org.gnome.Geary").expect("an identifier")
    }

    /// **Every answer crosses as one word and comes back as itself**, and no
    /// two share a word.
    #[test]
    fn every_answer_crosses_as_one_word_and_comes_back_as_itself() {
        let mut words = vec![Answer::Acted.written()];
        assert_eq!(Answer::read("acted"), Some(Answer::Acted));
        for why in NotActed::EVERY_ONE {
            let answer = Answer::NotActed(why);
            let word = answer.written();
            assert!(!word.contains(' '), "{word}");
            assert_eq!(Answer::read(word), Some(answer));
            words.push(word);
        }
        words.sort_unstable();
        words.dedup();
        assert_eq!(words.len(), NotActed::EVERY_ONE.len() + 1);
    }

    /// **A word this crate does not know is never read as *it worked*.**
    ///
    /// The asymmetry that matters: a compositor newer than its caller must not
    /// be able to say something that reads as success.
    #[test]
    fn an_unknown_answer_is_never_optimistic() {
        for word in ["", "ok", "done", "acted-ish", "yes", "no-windows"] {
            assert_ne!(Answer::read(word), Some(Answer::Acted), "`{word}`");
        }
    }

    /// **One of the four is this machine's fault and three are not**, so the
    /// one worth a log is not buried under the three that are ordinary.
    #[test]
    fn only_one_reason_is_worth_reading_a_log_about() {
        let ours: Vec<NotActed> = NotActed::EVERY_ONE
            .into_iter()
            .filter(|why| why.is_this_machines_fault())
            .collect();
        assert_eq!(ours, vec![NotActed::CouldNotAsk]);
    }

    /// **`open` can never answer *no window***, because it is allowed to start
    /// one. Held rather than remembered.
    #[test]
    fn open_never_answers_that_there_is_no_window() {
        assert!(!could_answer(&Acting::Open(mail()), NotActed::NoWindow));
        assert!(could_answer(&Acting::Focus(mail()), NotActed::NoWindow));
        assert!(could_answer(&Acting::Close(mail()), NotActed::NoWindow));
        // And every other reason is honest for every act.
        for act in [
            Acting::Open(mail()),
            Acting::Focus(mail()),
            Acting::Close(mail()),
        ] {
            for why in NotActed::EVERY_ONE {
                if why != NotActed::NoWindow {
                    assert!(could_answer(&act, why), "{act:?} {why:?}");
                }
            }
        }
    }
}
