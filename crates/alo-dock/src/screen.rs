//! The screen a dock is laid out on, and the side it takes its thickness from.
//!
//! **A dock takes from the side it sits on, not from the screen's short side.**
//! A dock along the bottom takes from the height; one down the left takes from
//! the width. Measuring both against the same side would let a dock down the
//! left of a wide screen grow to a quarter of it while a dock along the bottom
//! of the same screen was squeezed — one number, two very different results,
//! which is exactly the *horizontal bar somebody turned sideways* that
//! `docs/features.md` refuses.
//!
//! **What this crate lays out for, and why it is 1366 by 768.** The thresholds
//! in [`crate::measures`] are chosen against a screen rather than in the
//! abstract, and the screen they are chosen against is
//! [`Screen::the_smallest`] — the size that fills the Windows 10 fleet
//! `docs/hardware.md` says this product exists to catch. Nothing is certified
//! yet, so this is not a claim about a machine we have tested; it is the floor
//! the arithmetic is held to, and if a certified machine turns out to be
//! smaller, the tests in [`crate::layout`] fail rather than the layout quietly
//! getting worse.
//!
//! **Which screen this is, is not asked here.** `alo_appearance::DisplayId`
//! names a display, and *one dock per display* is v0.5. At v0.01 there is one
//! dock and it is laid out against whichever screen it is being drawn on.

use alo_strings::{Filling, Said, Strings};

use crate::along::Along;
use crate::room::Room;
use crate::words::{self, Word};

/// The width of the screen the thresholds in this crate are measured against.
const SMALLEST_WIDTH: u32 = 1366;

/// The height of the screen the thresholds in this crate are measured against.
const SMALLEST_HEIGHT: u32 = 768;

/// Why something is not a screen a dock can sit on.
///
/// There is no `Display`: the only road to words is [`ScreenError::said`].
/// Nothing deserialises a screen — it is what a compositor reports, not what a
/// settings file holds — so there is no key-writing refusal beside this one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenError {
    /// A width or a height of nothing: what was reported, both numbers.
    NotAScreen(u32, u32),
    /// Smaller than a dock could sit on: what was reported, then the least a
    /// side can be.
    TooSmall(u32, u32, u32),
}

impl ScreenError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::NotAScreen(_, _) => words::NOT_A_SCREEN,
            Self::TooSmall(_, _, _) => words::SCREEN_TOO_SMALL,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NotAScreen(width, height) => {
                Filling::of("width", width.to_string()).and("height", height.to_string())
            }
            Self::TooSmall(width, height, least) => Filling::of("width", width.to_string())
                .and("height", height.to_string())
                .and("least", least.to_string()),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// One screen, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Screen {
    /// How wide it is.
    width: Room,
    /// How tall it is.
    height: Room,
}

impl Screen {
    /// A screen this wide and this tall.
    ///
    /// # Errors
    /// [`ScreenError`], which names both numbers and, when the screen is merely
    /// too small, the least a side can be.
    pub fn of(width: u32, height: u32) -> Result<Self, ScreenError> {
        if width == 0 || height == 0 {
            return Err(ScreenError::NotAScreen(width, height));
        }
        let least = Room::the_least_a_side_can_be().as_pixels();
        if width < least || height < least {
            return Err(ScreenError::TooSmall(width, height, least));
        }
        Ok(Self {
            width: Room::pixels(width),
            height: Room::pixels(height),
        })
    }

    /// The smallest screen alo OS lays its dock out for, which is the screen
    /// every threshold in this crate is measured against.
    ///
    /// Built rather than checked, because it is written in this file and cannot
    /// arrive from anywhere; [`Screen::of`] would accept it, and there is a test
    /// that says so.
    #[must_use]
    pub const fn the_smallest() -> Self {
        Self {
            width: Room::pixels(SMALLEST_WIDTH),
            height: Room::pixels(SMALLEST_HEIGHT),
        }
    }

    /// How wide it is.
    #[must_use]
    pub const fn width(self) -> Room {
        self.width
    }

    /// How tall it is.
    #[must_use]
    pub const fn height(self) -> Room {
        self.height
    }

    /// The side a dock running this way takes its thickness out of: the height
    /// for a dock across the screen, the width for one down it.
    #[must_use]
    pub const fn the_side_a_dock_takes_from(self, along: Along) -> Room {
        match along {
            Along::Across => self.height,
            Along::Down => self.width,
        }
    }

    /// The side a dock running this way spans: the width for a dock across the
    /// screen, the height for one down it.
    #[must_use]
    pub const fn the_side_a_dock_runs_along(self, along: Along) -> Room {
        match along {
            Along::Across => self.width,
            Along::Down => self.height,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// **The screen this crate is measured against is held to the rule a
    /// reported screen is held to.** It is built by the compiler, so this is what
    /// puts it back through the check — or the check is advice.
    #[test]
    fn the_smallest_screen_would_be_accepted_from_a_compositor() {
        let smallest = Screen::the_smallest();
        assert_eq!(
            Screen::of(smallest.width().as_pixels(), smallest.height().as_pixels()),
            Ok(smallest)
        );
        assert_eq!(smallest.width().as_pixels(), 1366);
        assert_eq!(smallest.height().as_pixels(), 768);
    }

    /// **A dock takes from the side it sits on.** A wide screen gives a dock down
    /// its left more room than a dock along its bottom, and that is the point
    /// rather than an accident: the two orientations are laid out against
    /// different measurements.
    #[test]
    fn a_dock_takes_from_the_side_it_sits_on_and_spans_the_other() {
        let screen = Screen::the_smallest();
        assert_eq!(
            screen.the_side_a_dock_takes_from(Along::Across),
            screen.height()
        );
        assert_eq!(
            screen.the_side_a_dock_takes_from(Along::Down),
            screen.width()
        );
        assert_eq!(
            screen.the_side_a_dock_runs_along(Along::Across),
            screen.width()
        );
        assert_eq!(
            screen.the_side_a_dock_runs_along(Along::Down),
            screen.height()
        );
        assert!(
            screen.the_side_a_dock_takes_from(Along::Down)
                > screen.the_side_a_dock_takes_from(Along::Across),
            "a landscape screen has more width than height to give"
        );
    }

    /// A screen with nothing in one direction is not a screen, and the refusal
    /// names both numbers so that whoever reads it can see which one was zero.
    #[test]
    fn a_screen_with_no_width_or_no_height_is_refused() {
        assert_eq!(Screen::of(0, 768), Err(ScreenError::NotAScreen(0, 768)));
        assert_eq!(Screen::of(1366, 0), Err(ScreenError::NotAScreen(1366, 0)));
        assert_eq!(Screen::of(0, 0), Err(ScreenError::NotAScreen(0, 0)));

        let strings = in_english();
        assert_eq!(
            Screen::of(0, 768).unwrap_err().said(&strings).text(),
            "a screen has a width and a height — 0 by 768 is not one"
        );
    }

    /// **A screen too small for a dock is refused rather than laid out badly.**
    /// It is what lets [`crate::Layout`] answer without a `Result`: below this
    /// floor there is no honest answer, and inventing one would put a dock over
    /// the whole of somebody's screen.
    #[test]
    fn a_screen_too_small_to_hold_a_dock_is_refused_and_says_the_floor() {
        let least = Room::the_least_a_side_can_be().as_pixels();
        assert_eq!(
            Screen::of(least - 1, 768),
            Err(ScreenError::TooSmall(least - 1, 768, least))
        );
        assert_eq!(
            Screen::of(1366, least - 1),
            Err(ScreenError::TooSmall(1366, least - 1, least))
        );
        assert!(Screen::of(least, least).is_ok(), "the floor itself is fine");

        let strings = in_english();
        let said = Screen::of(320, 240).unwrap_err().said(&strings);
        assert!(said.text().contains("320 by 240"), "{said}");
        assert!(said.text().contains(&least.to_string()), "{said}");
        assert!(said.unfilled().is_empty());
    }

    /// A refusal is read in the language of the person reading it, and the
    /// numbers inside it come off their own machine rather than out of a
    /// translation.
    #[test]
    fn a_refusal_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::NOT_A_SCREEN,
            "ein Bildschirm hat eine Breite und eine Höhe — {width} × {height} ist keiner",
        )]);
        let said = Screen::of(1366, 0).unwrap_err().said(&strings);
        assert_eq!(
            said.text(),
            "ein Bildschirm hat eine Breite und eine Höhe — 1366 × 0 ist keiner"
        );
        assert!(said.is_translated());
        assert!(said.unfilled().is_empty());
    }
}
