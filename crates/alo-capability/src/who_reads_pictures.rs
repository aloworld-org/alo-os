//! **Who reads the pictures alo takes of a window**, for the applications that
//! describe nothing about their windows: the person's choice, kept beside the
//! switches that let alo use pictures and keys at all
//! ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md)
//! §4, and the owner's answer of 2026-11-17: *"It will always depend on the
//! user, if he wants the third party to read the screens pictures then he
//! causes"*).
//!
//! # Three answers today, and the fourth is not built
//!
//! ADR 0147 §4 offers three readers and names alo's own built-in recognition
//! as the one used before anybody has chosen. **That recogniser is not built**,
//! so it is not offered here — never offering is not taking away — and until
//! the person chooses, nothing reads a picture: [`WhoReadsPictures::NoOne`],
//! and alo says it cannot see inside the application rather than taking a
//! picture nobody will read.
//!
//! - [`WhoReadsPictures::NoOne`]: how a machine arrives, and what a file that
//!   will not read answers ([`WhoReadsPictures::as_read`]).
//! - [`WhoReadsPictures::ThisComputer`]: the person's own model, on this
//!   computer; nothing is sent.
//! - [`WhoReadsPictures::AProvider`]: an outside service the person added, by
//!   its name; **every picture leaves for it**, and what it costs is ADR 0147
//!   §4's own sentence ([`WhoReadsPictures::what_it_costs`]).
//!
//! # Not a grant, and not the switch
//!
//! It answers *what reads a picture*, and nothing else. Whether alo may take a
//! picture of one application's window is [`crate::PicturesAndKeys`]; whether
//! alo may use that application at all is a grant. A reader chosen here reaches
//! no application on its own.
//!
//! Kept in the same file as those switches
//! (`docs/contracts/how-code-runs-file.md`, `pictures-read-by`), so **an agent
//! cannot choose a reader for itself** — least of all one a picture would
//! leave for.

use alo_strings::Word;

use crate::words;

/// The name [`words::PICTURES_SENT_TO`] fills with the service's name, given
/// once here so the sentence and whoever fills it cannot come to disagree.
pub const THE_PROVIDER: &str = "provider";

/// What reads the pictures alo takes of a window.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum WhoReadsPictures {
    /// Nothing does, so alo takes no picture — how a machine arrives.
    #[default]
    NoOne,
    /// The person's own model, on this computer. Nothing is sent.
    ThisComputer,
    /// An outside service the person added, by the name they gave it. Every
    /// picture is sent to it.
    AProvider(String),
}

impl WhoReadsPictures {
    /// The two answers Settings always offers, in the order it draws them;
    /// one more is offered for each service the person added.
    pub const ALWAYS_OFFERED: [Self; 2] = [Self::NoOne, Self::ThisComputer];

    /// An outside service, by a name one could be: not empty, and with no
    /// space around it. Matched as written.
    #[must_use]
    pub fn a_provider(name: &str) -> Option<Self> {
        (!name.is_empty() && name.trim() == name).then(|| Self::AProvider(name.to_owned()))
    }

    /// The service, where one reads them.
    #[must_use]
    pub fn the_provider(&self) -> Option<&str> {
        match self {
            Self::AProvider(name) => Some(name),
            Self::NoOne | Self::ThisComputer => None,
        }
    }

    /// Whether anything reads a picture, so that one may be taken at all.
    #[must_use]
    pub const fn may_take_pictures(&self) -> bool {
        !matches!(self, Self::NoOne)
    }

    /// Whether each picture leaves this computer.
    #[must_use]
    pub const fn sends_pictures_away(&self) -> bool {
        matches!(self, Self::AProvider(_))
    }

    /// The choice's name, where it has one of the product's own; a service is
    /// drawn by its own name, which is data and is never translated.
    #[must_use]
    pub const fn the_word(&self) -> Option<Word> {
        match self {
            Self::NoOne => Some(words::READ_BY_NO_ONE),
            Self::ThisComputer => Some(words::READ_BY_THIS_COMPUTER),
            Self::AProvider(_) => None,
        }
    }

    /// The line under the choice.
    #[must_use]
    pub const fn what_it_is(&self) -> Word {
        match self {
            Self::NoOne => words::READ_BY_NO_ONE_IS,
            Self::ThisComputer => words::READ_BY_THIS_COMPUTER_IS,
            Self::AProvider(_) => words::READ_BY_A_PROVIDER_IS,
        }
    }

    /// What choosing it costs, where it costs anything: ADR 0147 §4's own
    /// sentence for a service, filled with [`THE_PROVIDER`].
    #[must_use]
    pub const fn what_it_costs(&self) -> Option<Word> {
        match self {
            Self::AProvider(_) => Some(words::PICTURES_SENT_TO),
            Self::NoOne | Self::ThisComputer => None,
        }
    }

    /// The answer a reader of the person's file goes by: what it says, or,
    /// for a file that is there and will not read, **no one** — never a
    /// service a picture would leave for.
    #[must_use]
    pub fn as_read<E>(read: Result<Self, E>) -> Self {
        read.unwrap_or_default()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "who_reads_pictures_tests.rs"]
mod tests;
