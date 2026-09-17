//! Why a division was left as it was, and what a person is told.
//!
//! **Every refusal leaves the division exactly as it was.** Nothing in this crate
//! changes a share and then discovers it should not have: each operation is
//! decided against the tree first and applied only when all of it holds, so a
//! person told *the screen has been left as it was* is told the truth.
//!
//! The one refusal the plan names is the one this crate is most careful about: a
//! window whose minimum size is larger than the share it would be given is **not
//! squeezed below it**. Every system that snaps windows has met an application
//! that cannot be drawn narrower than 800 units; the common answer is to give it
//! the half anyway and let it overflow its neighbour, which is the overlap this
//! crate exists to rule out.
//!
//! Like `alo-shortcuts`' refusals, a [`Refused`] has no `Display`: the only road
//! to words is [`Refused::said`], in the language the person reads.

use alo_strings::{Filling, Said, Strings};

use crate::side::{Axis, Side};
use crate::window::WindowId;
use crate::words::{self, Word};

/// Why a division did not change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// This window has no share in the division.
    NotDivided(WindowId),
    /// There is no second window to divide the screen with.
    NothingToShareWith,
    /// This edge of the window's share is an edge of the display, so there is no
    /// boundary there to move.
    NoNeighbour {
        /// The window whose edge was dragged.
        window: WindowId,
        /// Which edge.
        edge: Side,
    },
    /// This window cannot be made as narrow as its share would be.
    TooNarrow(WindowId),
    /// This window cannot be made as short as its share would be.
    TooShort(WindowId),
    /// The division changed after a drop was proposed, so the proposal no longer
    /// describes what dropping would do.
    Changed,
}

impl Refused {
    /// The window squeezed along an axis: too narrow for a side-by-side cut, too
    /// short for one above the other.
    pub(crate) const fn squeezed(window: WindowId, axis: Axis) -> Self {
        match axis {
            Axis::SideBySide => Self::TooNarrow(window),
            Axis::OneAboveTheOther => Self::TooShort(window),
        }
    }

    /// The window this is about, when it is about one — so the shell can mark it
    /// beside the sentence, which never names a window itself.
    #[must_use]
    pub const fn window(self) -> Option<WindowId> {
        match self {
            Self::NotDivided(window)
            | Self::TooNarrow(window)
            | Self::TooShort(window)
            | Self::NoNeighbour { window, .. } => Some(window),
            Self::NothingToShareWith | Self::Changed => None,
        }
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::NotDivided(_) => words::NOT_DIVIDED,
            Self::NothingToShareWith => words::NOTHING_TO_SHARE_WITH,
            Self::NoNeighbour { .. } => words::NO_NEIGHBOUR,
            Self::TooNarrow(_) => words::TOO_NARROW,
            Self::TooShort(_) => words::TOO_SHORT,
            Self::Changed => words::CHANGED,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use std::collections::BTreeSet;

    /// Every refusal says something of its own, declared, with nothing left to
    /// fill in — a person told the wrong reason would try the same thing again.
    #[test]
    fn every_refusal_says_something_of_its_own() {
        let strings = in_english();
        let window = WindowId::from_compositor(1);
        let mut seen = BTreeSet::new();
        for refused in [
            Refused::NotDivided(window),
            Refused::NothingToShareWith,
            Refused::NoNeighbour {
                window,
                edge: Side::Left,
            },
            Refused::TooNarrow(window),
            Refused::TooShort(window),
            Refused::Changed,
        ] {
            let said = refused.said(&strings);
            assert!(!said.is_a_bug(), "{refused:?}");
            assert!(said.unfilled().is_empty(), "{refused:?}: {said}");
            assert!(seen.insert(said.text().to_owned()), "{refused:?}");
        }
    }

    /// Too narrow and too short are two sentences, chosen by the axis of the
    /// cut, and each names the window so the shell can mark it.
    #[test]
    fn a_squeezed_window_is_named_and_the_sentence_says_which_way() {
        let window = WindowId::from_compositor(9);
        let narrow = Refused::squeezed(window, Axis::SideBySide);
        let short = Refused::squeezed(window, Axis::OneAboveTheOther);
        assert_eq!(narrow, Refused::TooNarrow(window));
        assert_eq!(short, Refused::TooShort(window));
        assert_eq!(narrow.window(), Some(window));
        let strings = in_english();
        assert!(narrow.said(&strings).text().contains("narrow"));
        assert!(short.said(&strings).text().contains("short"));
        assert_eq!(Refused::Changed.window(), None);
    }
}
