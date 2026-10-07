//! What the strip at the top of a first-start screen says, and where it appears.
//!
//! **It says `alo OS`, and it says nothing else.** The owner settled this on
//! 2026-10-07 in the words *our name is alo OS, not 2-of-5*, replacing a label
//! that had carried a step count on some screens and a stage name on others.
//! `docs/design/the-first-start.md` holds the measurements; this file holds the
//! two rules a build can get wrong.
//!
//! # The strip is identity, so it carries no state
//!
//! It is on screen through the whole of setup — **before any window, wallpaper
//! or account exists** — which makes it the most persistent thing a person sees
//! of what they are installing. That is the argument for a name and against
//! everything else: a counter spends the most valuable line on the screen on
//! the least valuable information.
//!
//! **There is therefore nothing to read out of it.** It is the same six
//! characters on all sixty-nine screens that have one. Three separate readings
//! of the design file tried to infer behaviour from this label — that it
//! tracked the step a person came from, that it counted five steps — and
//! `docs/design/the-first-start.md` keeps the record of all three being wrong.
//! A build that derives navigation, progress or ordering from the strip is
//! repeating that mistake against a label that now cannot support it.
//!
//! # And the count could not have been honest
//!
//! A person who takes the network-machine branch, turns the screen reader on,
//! or adds both a PIN and a fingerprint walks a different number of screens
//! from one who does not. So *N of 5* was either wrong or so coarse it told
//! nobody anything, and **no denominator may be derived from the frame
//! inventory**: seventy-seven states are not five steps.
//!
//! `nothing_the_strip_says_counts_anything`, in this file's own tests, is that
//! rule as a check rather than as a paragraph, and it is deliberately stricter
//! than the decision — it refuses a digit anywhere, so a later `alo OS 2`
//! would have to argue with it rather than slip past. *Named in prose rather
//! than linked: `cargo doc` does not resolve a link to a `#[cfg(test)]` item,
//! and the gate runs without `--document-private-items`.*
//!
//! # Why this is not an [`alo_strings::Word`]
//!
//! Everything else a person reads in this crate is declared in [`crate::words`]
//! and answered in the language they read. **This is not, because it is a
//! product name and a translated product name is a different product.**
//! Another crate's own notes already say so where they have to explain
//! themselves to a translator: *"alo OS" is the product's name and is never
//! translated.*
//!
//! So there is no key, nothing for a translator to fill in, and nothing a
//! locale can change. **That is the one exception in this crate**, and it is
//! narrow on purpose: the word *Optional* above a heading, the name of every
//! action, and every sentence on every screen all stay translated.
//!
//! # What this file does not claim to own
//!
//! **The product's name is spelled in several crates** — `alo_sleeping`'s own
//! `US`, `alo_notifying::sender`, `alo_brokerd`'s boot entry, and others. This
//! is the setup strip, not a canonical home for the name, and unifying those
//! is a separate change that would touch other lanes' files for no gain to
//! this decision.

/// What the strip says, wherever it appears.
///
/// Not a translated string: see this module's header.
pub const THE_STRIP_SAYS: &str = "alo OS";

/// The two kinds of first-start screen, which differ in whether they have a
/// strip at all.
///
/// **This is the distinction a build gets wrong**, and it is structural rather
/// than a list of frames to remember. Measured across all seventy-seven frames
/// of the design file on 2026-10-07: sixty-nine carry the strip and eight do
/// not, and the eight are exactly the ones that show the running machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Screen {
    /// A setup sheet: a column of eyebrow, title, introduction, choices and
    /// actions, drawn over the wallpaper while setup is still asking.
    SetupSheet,
    /// The running canvas, with the Dock and a place name — the machine itself,
    /// reached when setup has finished or been declined.
    RunningCanvas,
}

/// Every kind of screen, so a caller can walk them without writing the list
/// out and getting it wrong.
pub const EVERY_SCREEN: [Screen; 2] = [Screen::SetupSheet, Screen::RunningCanvas];

impl Screen {
    /// What this screen's strip says, or [`None`] where it has no strip.
    ///
    /// **A canvas has none, and must not be given one.** The canvas is the
    /// machine a person now owns; a setup label stamped on it would say setup
    /// is still running after it has finished, which is the opposite of what
    /// the last screen of the flow just told them.
    ///
    /// ```
    /// use alo_setting_up::the_strip::{Screen, THE_STRIP_SAYS};
    ///
    /// assert_eq!(Screen::SetupSheet.strip(), Some(THE_STRIP_SAYS));
    /// assert_eq!(Screen::RunningCanvas.strip(), None);
    /// ```
    #[must_use]
    pub const fn strip(self) -> Option<&'static str> {
        match self {
            Self::SetupSheet => Some(THE_STRIP_SAYS),
            Self::RunningCanvas => None,
        }
    }

    /// Whether this screen has a strip.
    #[must_use]
    pub const fn has_a_strip(self) -> bool {
        self.strip().is_some()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The decision itself, written where it fails if somebody edits it.
    #[test]
    fn the_strip_says_the_product_name() {
        assert_eq!(THE_STRIP_SAYS, "alo OS");
    }

    /// **The rule the old label broke.** Stricter than the decision on purpose:
    /// a digit anywhere is refused, so neither `2 OF 5` nor a version number
    /// can arrive without arguing with this test first.
    #[test]
    fn nothing_the_strip_says_counts_anything() {
        for screen in EVERY_SCREEN {
            let Some(said) = screen.strip() else {
                continue;
            };
            assert!(
                !said.chars().any(|character| character.is_ascii_digit()),
                "{screen:?} counts something: {said}"
            );
        }
    }

    /// **And it is not a stage name either.** The label used to read `SET UP ·
    /// AI CHOICE`, `AI CHOICE · DETAILS` and `OPTIONAL · ALO ACCESS` on
    /// different screens; all three are gone, and the separator they used is
    /// the cheap way to notice one coming back.
    #[test]
    fn the_strip_is_one_name_rather_than_two_halves() {
        let said = Screen::SetupSheet.strip().unwrap();
        assert!(!said.contains('·'), "{said} is two halves");
        assert!(!said.to_lowercase().contains("set up"), "{said}");
        assert!(!said.to_lowercase().contains("optional"), "{said}");
    }

    /// **Identity, so every screen that has one says the same thing.** If a
    /// second sheet-like screen is ever added, it says this too — which is the
    /// property that makes the strip useless to infer from, and that is the
    /// point.
    #[test]
    fn every_screen_with_a_strip_says_exactly_the_same_thing() {
        let said: Vec<&str> = EVERY_SCREEN
            .iter()
            .filter_map(|screen| screen.strip())
            .collect();
        assert!(!said.is_empty(), "no screen has a strip at all");
        for one in &said {
            assert_eq!(*one, THE_STRIP_SAYS);
        }
    }

    /// The canvas has none. Asserted rather than left to the match arm,
    /// because this is the half a build is likely to get wrong.
    #[test]
    fn the_running_canvas_has_no_strip() {
        assert_eq!(Screen::RunningCanvas.strip(), None);
        assert!(!Screen::RunningCanvas.has_a_strip());
        assert!(Screen::SetupSheet.has_a_strip());
    }

    /// `EVERY_SCREEN` is every screen, which is what lets the tests above walk
    /// the list instead of repeating it.
    #[test]
    fn every_screen_is_in_every_screen() {
        assert_eq!(EVERY_SCREEN.len(), 2);
        for screen in EVERY_SCREEN {
            // A match the compiler checks: a new variant fails to build here
            // rather than being silently left out of the list.
            match screen {
                Screen::SetupSheet | Screen::RunningCanvas => {}
            }
        }
    }
}
