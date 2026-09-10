//! The refusal a person reads when the agent cannot be summoned.
//!
//! The plan is blunt about why this type exists: *a key that silently does
//! nothing is the worst outcome available*. When the chord is pressed and the
//! agent cannot appear, what happens instead is a sentence — in the person's
//! own language, through `alo-strings` — and this file is where each way of
//! failing meets the sentence that says so.
//!
//! Like everything else this repository says to a person, the sentences are
//! declared in [`crate::words`] and answered through [`NotSummoned::said`];
//! there is no `Display` that would put English on a screen by accident.

use alo_strings::{Filling, Said, Strings};

use crate::surface::SurfaceRefused;
use crate::words::{self, Word};

/// Why the agent was not summoned when the key was pressed.
///
/// Two shapes of nowhere: no compositor at all, and a compositor that
/// refused. They are kept apart because the person is told different things —
/// one is *the desktop is not running*, the other is a fact the compositor
/// knows, like a machine with no screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotSummoned {
    /// There is no compositor to ask: nothing on this machine can show the
    /// overlay, because nothing is drawing a screen at all.
    NoCompositor,
    /// The compositor was asked — once — and refused, for the reason carried.
    Surface(SurfaceRefused),
}

impl NotSummoned {
    /// The string this crate declares for it: the key a translator's file is
    /// sorted by, and the English beside it.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::NoCompositor => words::NO_COMPOSITOR,
            Self::Surface(SurfaceRefused::NothingToShowOn) => words::NOTHING_TO_SHOW_ON,
        }
    }

    /// What to tell the person, in the language they read.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not:
    /// there is always something to put in front of the person, and where it
    /// came from is on the [`Said`]. A `Strings` that was never given
    /// [`crate::overlay_words`] answers with the key, marked `Said::is_a_bug`
    /// — the honest answer to *the shell forgot to declare what this crate
    /// can say*.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Every way of not being summoned, so no test below quietly skips one.
    const EVERY_REFUSAL: [NotSummoned; 2] = [
        NotSummoned::NoCompositor,
        NotSummoned::Surface(SurfaceRefused::NothingToShowOn),
    ];

    /// **Every refusal says something a person could read**, and no two ways
    /// of failing read the same — a person told the same sentence for a
    /// missing desktop and a missing screen would fix the wrong one.
    #[test]
    fn every_refusal_reads_and_no_two_read_the_same() {
        let strings = in_english();
        let mut seen = Vec::new();
        for refusal in EVERY_REFUSAL {
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

    /// **The refusal arrives in the language the person reads** when somebody
    /// has translated it, and says so — the whole of what declaring these
    /// through `alo-strings` buys.
    #[test]
    fn a_refusal_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::NO_COMPOSITOR,
            "Der Agent kann nirgends erscheinen: der Schreibtisch läuft nicht. Melden Sie sich \
             am Schreibtisch an und drücken Sie die Taste noch einmal",
        )]);
        let said = NotSummoned::NoCompositor.said(&strings);
        assert!(said.is_translated());
        assert!(said.text().starts_with("Der Agent"));

        // The one nobody translated is still English, and says it is.
        let untranslated = NotSummoned::Surface(SurfaceRefused::NothingToShowOn).said(&strings);
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug());
    }
}
