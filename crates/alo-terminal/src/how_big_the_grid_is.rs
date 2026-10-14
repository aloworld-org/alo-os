//! How many columns and lines a terminal has, and how much it remembers.
//!
//! # Why this is a type and not three numbers
//!
//! The engine asks for its size through a trait with three questions —
//! *how many columns*, *how many lines on screen*, and *how many lines in
//! total including scrollback* — and the third is the first two's business
//! rather than a caller's. A caller handing over three loose numbers can hand
//! over a total smaller than the screen, which is not a terminal that is
//! wrong, it is a terminal that cannot exist.
//!
//! So the type is constructed from what a person chooses — a width, a height,
//! and how much history to keep — and works the third out itself.
//!
//! # Zero is refused rather than clamped
//!
//! A terminal with no columns has no cell to put a character in, and a
//! terminal with no lines has nowhere for a line to be. Both are refused by
//! name. **Clamping them to one would be worse**: it would turn a caller's
//! arithmetic bug into a terminal that draws, so nobody would find the bug
//! until somebody wondered why a window was one character wide.

/// How big a terminal is, and how much it remembers.
///
/// Columns and lines are what a person sees. Scrollback is what they can
/// reach by scrolling up, and it is **in addition** to the screen rather than
/// including it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HowBig {
    /// How many cells across. Never zero.
    columns: usize,
    /// How many lines are on the screen. Never zero.
    lines: usize,
    /// How many lines are kept above the screen. May be zero — a terminal that
    /// remembers nothing is a choice somebody may make, unlike one with no
    /// screen.
    remembered: usize,
}

/// Why something is not a size a terminal can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotASize {
    /// No columns, so no cell for a character to be in.
    #[error("a terminal with no columns has nowhere to put a character")]
    NoColumns,
    /// No lines, so nowhere for a line to be.
    #[error("a terminal with no lines has nowhere for a line to be")]
    NoLines,
}

impl HowBig {
    /// A terminal this wide, this tall, remembering this many lines above the
    /// screen.
    ///
    /// # Errors
    /// [`NotASize::NoColumns`] or [`NotASize::NoLines`] for a zero in either,
    /// refused rather than clamped for the reason this module's header gives.
    pub const fn of(columns: usize, lines: usize, remembered: usize) -> Result<Self, NotASize> {
        if columns == 0 {
            return Err(NotASize::NoColumns);
        }
        if lines == 0 {
            return Err(NotASize::NoLines);
        }
        Ok(Self {
            columns,
            lines,
            remembered,
        })
    }

    /// How many lines are kept above the screen.
    #[must_use]
    pub const fn remembered(self) -> usize {
        self.remembered
    }
}

/// The engine asks its three questions here.
///
/// **`total_lines` is derived and never stored**, which is the whole reason
/// this type exists: the engine's own documentation says the total *includes
/// scrollback and visible lines*, so a total that did not equal the sum would
/// be a grid whose two halves disagreed about how big it was.
impl alacritty_terminal::grid::Dimensions for HowBig {
    fn total_lines(&self) -> usize {
        self.lines + self.remembered
    }

    fn screen_lines(&self) -> usize {
        self.lines
    }

    fn columns(&self) -> usize {
        self.columns
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alacritty_terminal::grid::Dimensions as _;

    use super::{HowBig, NotASize};

    #[test]
    fn a_size_answers_the_three_questions_the_engine_asks() {
        let size = HowBig::of(80, 24, 1_000).expect("eighty by twenty-four is a terminal");
        assert_eq!(size.columns(), 80);
        assert_eq!(size.screen_lines(), 24);
        assert_eq!(size.remembered(), 1_000);
    }

    /// **The total is the sum, and this is the assertion that keeps it one.**
    ///
    /// The engine's own words are that the total *includes scrollback and
    /// visible lines*. A total stored separately could be set to anything,
    /// and a grid whose two halves disagree about its height is the kind of
    /// fault that shows up as a line drawn in the wrong place much later.
    #[test]
    fn the_total_is_the_screen_plus_what_is_remembered() {
        for (lines, remembered) in [(24, 1_000), (1, 0), (50, 1), (2, 100_000)] {
            let size = HowBig::of(80, lines, remembered).expect("a terminal");
            assert_eq!(
                size.total_lines(),
                lines + remembered,
                "{lines} on screen and {remembered} remembered"
            );
            assert!(
                size.total_lines() >= size.screen_lines(),
                "a terminal cannot remember less than it shows"
            );
        }
    }

    /// **Remembering nothing is allowed**, unlike having no screen. A person
    /// may ask for a terminal that keeps no history; nobody can ask for one
    /// with nowhere to put a character.
    #[test]
    fn a_terminal_may_remember_nothing() {
        let size = HowBig::of(80, 24, 0).expect("remembering nothing is a choice");
        assert_eq!(size.total_lines(), 24);
    }

    #[test]
    fn zero_is_refused_by_name_rather_than_clamped() {
        assert_eq!(HowBig::of(0, 24, 100), Err(NotASize::NoColumns));
        assert_eq!(HowBig::of(80, 0, 100), Err(NotASize::NoLines));
        // Both wrong answers the first question asked, which is the one a
        // caller fixes first.
        assert_eq!(HowBig::of(0, 0, 100), Err(NotASize::NoColumns));
    }

    /// **Each refusal says what is wrong in words**, because this is the one
    /// error a caller meets while getting their arithmetic right and a
    /// message naming the wrong field costs them the time the message was
    /// meant to save.
    #[test]
    fn each_refusal_names_what_is_missing() {
        assert!(NotASize::NoColumns.to_string().contains("columns"));
        assert!(NotASize::NoLines.to_string().contains("lines"));
        assert_ne!(
            NotASize::NoColumns.to_string(),
            NotASize::NoLines.to_string()
        );
    }
}
