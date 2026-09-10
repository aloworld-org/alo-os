//! The refusal a person reads when what is leaving cannot be put on a screen.
//!
//! A machine that cannot show what is leaving it, and says nothing about that,
//! is a machine on which law 1 has quietly stopped being true. So there is no
//! silent failure anywhere in this crate: every way the indicator can fail to
//! reach a screen is one of these, and every one of them has a sentence.
//!
//! Like everything else this repository says to a person, the sentences are
//! declared in [`crate::words`] and answered through [`NotShown::said`]; there
//! is no `Display` that would put English on a screen by accident.
//!
//! # Where these are read, since the screen is the thing that is missing
//!
//! Not on the indicator's own surface — there is not one. They are read in a
//! service log by whoever is standing the machine up, or on a text console
//! while a session starts. They are still declared and still translated,
//! because a person reading a log on their own machine reads their own
//! language, and because the alternative is English written into a source file,
//! which `CLAUDE.md` calls a bug wherever it appears.

use alo_strings::{Filling, Said, Strings};

use crate::surface::SurfaceRefused;
use crate::words::{self, Word};

/// Why what is leaving this machine was not shown.
///
/// Two shapes of nowhere: no compositor at all, and a compositor that refused.
/// They are kept apart because the person is told different things — one is
/// *the desktop is not running*, the other is a fact the compositor knows, like
/// a machine with no screen — and because they are fixed by different actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotShown {
    /// There is no compositor to ask: nothing on this machine is drawing a
    /// screen at all.
    NoCompositor,
    /// The compositor was asked and refused, for the reason carried.
    Surface(SurfaceRefused),
}

impl NotShown {
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
    /// [`crate::indicator_words`] answers with the key, marked `Said::is_a_bug`
    /// — the honest answer to *the shell forgot to declare what this crate can
    /// say*.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Every way of not being shown, so no test below quietly skips one.
    const EVERY_REFUSAL: [NotShown; 2] = [
        NotShown::NoCompositor,
        NotShown::Surface(SurfaceRefused::NothingToShowOn),
    ];

    /// **Every refusal says something a person could read**, and no two ways of
    /// failing read the same — a person told the same sentence for a missing
    /// desktop and a missing screen would go and fix the wrong one.
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
    /// has translated it, and says so.
    #[test]
    fn a_refusal_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::NO_COMPOSITOR,
            "Was dieses Gerät verlässt, kann nicht angezeigt werden: der Schreibtisch läuft \
             nicht. Melden Sie sich am Schreibtisch an, dann erscheint die Anzeige mit ihm",
        )]);
        let said = NotShown::NoCompositor.said(&strings);
        assert!(said.is_translated());
        assert!(said.text().starts_with("Was dieses Gerät"));

        // The one nobody translated is still English, and says it is.
        let untranslated = NotShown::Surface(SurfaceRefused::NothingToShowOn).said(&strings);
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug());
    }
}
