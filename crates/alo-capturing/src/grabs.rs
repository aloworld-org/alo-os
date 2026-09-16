//! The rented screen-capture mechanism, as one question.
//!
//! The plan's acceptance says where the pixels come from, and it is a sentence
//! about provenance: a screenshot is taken **through the rented screen-capture
//! mechanism**. Nothing in this crate reads a display, opens a framebuffer,
//! talks a display protocol or encodes an image — ADR 0011 is why, and
//! `docs/autonomy/v0-5-capture-and-the-room-plan.md` says it again in the
//! plan's own words: no screen grabber, no encoder and no media server of our
//! own.
//!
//! [`Grabs`] is that question, and it is a trait for the reason
//! `alo_in_use::Streams` is one: the machine reaches its mechanism by running a
//! program, and a test cannot. The only implementation alo OS ships is
//! [`crate::TheScreenCast`].
//!
//! # What the mechanism is told, and what it is not
//!
//! It is told a rectangle of one screen, and the screen that rectangle is on.
//! It is **not** told what is being captured — whether the rectangle is a
//! window, a selection or the whole screen is a decision this crate made
//! already ([`crate::What::across`]), and a mechanism that had to make it again
//! would be a second answer to *which pixels* living inside something we rent.
//!
//! It is also not told where the picture is going. A mechanism that knew about
//! a folder or a clipboard could put a picture somewhere nobody asked; this one
//! hands back bytes and has nowhere to put them.
//!
//! # Refusals it can answer with, and one it cannot
//!
//! Three, and they are the three `alo_in_use::NotHeard` has, for the same
//! reason: a part that is not running, one that did not answer, and one that
//! answered something unusable are three different things for whoever is fixing
//! the machine, and a person told the same sentence for all three goes and does
//! the wrong one.
//!
//! What it cannot answer is *not allowed*. Whether this capture may happen at
//! all was decided before the mechanism was reached — the lock screen and
//! another person's session in [`crate::Screenshot::of`], an application's
//! grant in `alo-portals` — and a mechanism that could refuse on policy would
//! be a second place where policy lived.

use alo_strings::{Filling, Said, Strings};

use crate::picture::Picture;
use crate::region::Region;
use crate::screen::Screen;
use crate::words::{self, Word};

/// Whatever can take one picture of a rectangle of the screen.
///
/// Implemented by the rented screen-capture mechanism, and by nothing else.
pub trait Grabs {
    /// One picture of that rectangle of that screen.
    ///
    /// Takes `&mut self` because reaching a screen-capture mechanism is not a
    /// pure question: the one alo OS ships starts a program and waits for it.
    ///
    /// # Errors
    /// [`NotGrabbed`], when the mechanism is not there, does not answer, or
    /// answers something that is not a picture. Never an empty picture standing
    /// in for one of those.
    fn grab(&mut self, across: Region, on: Screen) -> Result<Picture, NotGrabbed>;
}

/// Why no picture came back from the rented mechanism.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotGrabbed {
    /// There is nothing on this machine that reads the screen.
    #[error("nothing on this machine reads the screen: {said}")]
    NothingReadsTheScreen {
        /// What the machine said when it was looked for. English, for whoever
        /// is fixing it, and never shown to a person.
        said: String,
    },
    /// It was asked and did not answer.
    #[error("what reads the screen did not answer: {said}")]
    NoPicture {
        /// What it said instead. English, for whoever is fixing it.
        said: String,
    },
    /// It answered, and what came back was not a picture.
    #[error("what reads the screen answered with nothing usable: {said}")]
    NothingCameBack {
        /// What was wrong with the answer. English, for whoever is fixing it.
        said: String,
    },
}

impl NotGrabbed {
    /// The string this crate declares for it: the key a translator's file is
    /// sorted by, and the English beside it.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::NothingReadsTheScreen { .. } => words::NOTHING_READS_THE_SCREEN,
            Self::NoPicture { .. } => words::NO_PICTURE,
            Self::NothingCameBack { .. } => words::NOTHING_CAME_BACK,
        }
    }

    /// What was said about it in English, for whoever is fixing it.
    ///
    /// Never put in front of a person: what a person reads is [`NotGrabbed::said`]
    /// and it is answered in their own language.
    #[must_use]
    pub fn diagnosis(&self) -> &str {
        match self {
            Self::NothingReadsTheScreen { said }
            | Self::NoPicture { said }
            | Self::NothingCameBack { said } => said,
        }
    }

    /// What to tell the person, in the language they read.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Every way of not getting a picture, so no test below quietly skips one.
    fn every_refusal() -> [NotGrabbed; 3] {
        [
            NotGrabbed::NothingReadsTheScreen {
                said: "the mechanism is not on this machine".to_owned(),
            },
            NotGrabbed::NoPicture {
                said: "it failed: no such node".to_owned(),
            },
            NotGrabbed::NothingCameBack {
                said: "the picture came back with no bytes in it".to_owned(),
            },
        ]
    }

    /// **Every refusal says something a person could read**, and no two read
    /// the same — somebody told the same sentence for a component that is not
    /// running and one that answered nothing would go and do the wrong thing.
    #[test]
    fn every_refusal_reads_and_no_two_read_the_same() {
        let strings = in_english();
        let mut seen: Vec<String> = Vec::new();
        for refusal in every_refusal() {
            let said = refusal.said(&strings);
            assert!(!said.text().is_empty(), "{refusal:?} says nothing");
            assert!(!said.is_a_bug(), "{refusal:?} is not declared");
            assert_eq!(said.text(), refusal.word().says());
            assert!(
                !seen.contains(&said.text().to_owned()),
                "two refusals both say {said}"
            );
            seen.push(said.text().to_owned());
        }
    }

    /// **The diagnosis is kept and is not the sentence.** Whoever is fixing the
    /// machine needs what it actually said; the person who asked for a picture
    /// never sees it.
    #[test]
    fn what_the_machine_said_is_kept_for_whoever_is_fixing_it() {
        let strings = in_english();
        for refusal in every_refusal() {
            assert!(!refusal.diagnosis().is_empty(), "{refusal:?}");
            assert!(
                !refusal.said(&strings).text().contains(refusal.diagnosis()),
                "{refusal:?} puts a diagnostic in front of a person"
            );
        }
    }

    /// **A refusal arrives in the language the person reads** when somebody has
    /// translated it, and says so.
    #[test]
    fn a_refusal_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::NO_PICTURE,
            "Es konnte kein Bild des Bildschirms aufgenommen werden: Der Teil von alo OS, der den \
             Bildschirm liest, hat nicht geantwortet. Melden Sie sich ab und wieder an",
        )]);
        let said = NotGrabbed::NoPicture {
            said: "no such node".to_owned(),
        }
        .said(&strings);
        assert!(said.is_translated());
        assert!(said.text().starts_with("Es konnte kein Bild"));

        // The one nobody translated is still English, and says it is.
        let untranslated = NotGrabbed::NothingCameBack {
            said: "empty".to_owned(),
        }
        .said(&strings);
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug());
    }
}
