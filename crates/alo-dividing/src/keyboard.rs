//! Dividing the screen from the keyboard: the focused share, with the next
//! window.
//!
//! **The chord is `alo-shortcuts`' value, never this crate's.** A person already
//! has two shortcuts that say what a keyboard split is for —
//! `alo_shortcuts::Action::SnapLeft`, *Put the window on the left half*, and
//! `SnapRight` — and whatever chords they bound those to are the ones that
//! divide. [`side_bound_to`] asks their shortcuts what a pressed chord does and
//! answers with a side only when it is one of those two, so a person who moved
//! *left half* to another chord moved the split with it, and one who cleared it
//! has no keyboard split at all.
//!
//! **Why those two and not a new action.** This crate's plan reads
//! `alo-shortcuts` and never edits it, and those two actions already mean what a
//! keyboard split does: the focused window goes to that half. What v0.5 adds is
//! what happens to the other half — it is given to the next window rather than
//! left showing whatever was behind — and that is this crate's decision, not a
//! second binding.
//!
//! # What *the focused share, with the next window* means
//!
//! - **Nothing divided yet:** the focused window and the next one take the two
//!   halves of the display.
//! - **The focused window has a share:** that share is halved, the focused
//!   window on the side pressed and the next window on the other — which is how
//!   a half becomes two quarters from the keyboard.
//! - **The next window already has a share elsewhere:** it leaves that share to
//!   its neighbour first, so pressing the chord twice evens a pair out rather
//!   than putting one window in two places.
//! - **The focused window is floating over a divided display:** refused with
//!   [`Refused::NotDivided`], which says to drag it to an edge — dividing
//!   somebody's existing arrangement because a floating window had focus would
//!   rearrange windows they were not looking at.

use alo_shortcuts::{Action, Chord, Shortcuts};

use crate::division::Division;
use crate::node::Node;
use crate::refusing::Refused;
use crate::side::{Axis, Side};
use crate::window::Window;

/// The side of its share a keyboard split puts the focused window on, when
/// `action` is a keyboard split.
#[must_use]
pub const fn side_for(action: Action) -> Option<Side> {
    match action {
        Action::SnapLeft => Some(Side::Left),
        Action::SnapRight => Some(Side::Right),
        Action::TheAgent
        | Action::Launcher
        | Action::CloseWindow
        | Action::MinimiseWindow
        | Action::MaximiseWindow
        | Action::NextWindow
        | Action::PreviousWindow
        | Action::NextApplication
        | Action::PreviousApplication => None,
    }
}

/// Whether this chord, pressed on this person's shortcuts, is a keyboard split
/// — and the side it puts the focused window on.
#[must_use]
pub fn side_bound_to(shortcuts: &Shortcuts, chord: Chord) -> Option<Side> {
    shortcuts.action_for(chord).and_then(side_for)
}

impl Division {
    /// Divide the focused window's share with the next window, the focused one
    /// on `side`.
    ///
    /// `next` is the window the shell would move to next, or `None` when no
    /// other window is open.
    ///
    /// # Errors
    /// - [`Refused::NothingToShareWith`] when there is no next window, or it is
    ///   the focused one;
    /// - [`Refused::NotDivided`] when the display is divided and the focused
    ///   window is not part of it;
    /// - [`Refused::TooNarrow`] or [`Refused::TooShort`] when either window
    ///   would be squeezed below its minimum.
    pub fn divide_with_next(
        &mut self,
        focused: Window,
        next: Option<Window>,
        side: Side,
    ) -> Result<(), Refused> {
        let next = next
            .filter(|next| next.id() != focused.id())
            .ok_or(Refused::NothingToShareWith)?;
        let tree = match self.tree() {
            None => {
                let whole = Node::Share(focused);
                for axis in [Axis::SideBySide, Axis::OneAboveTheOther] {
                    whole.fits(axis, self.display().size().along(axis))?;
                }
                whole
            }
            Some(tree) if tree.holds(focused.id()) => self
                .without(next.id())
                .ok_or(Refused::NotDivided(focused.id()))?,
            Some(_) => return Err(Refused::NotDivided(focused.id())),
        };
        let divided = Self::halved(tree, self.display(), focused.id(), next, side.opposite())?;
        self.replace(divided);
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{any, area, at_least, display, divided_in_halves};
    use crate::window::WindowId;

    /// Only the two halves are keyboard splits; nothing else a shortcut does
    /// divides the screen.
    #[test]
    fn only_the_two_halves_are_keyboard_splits() {
        for action in Action::ALL {
            let expected = match action {
                Action::SnapLeft => Some(Side::Left),
                Action::SnapRight => Some(Side::Right),
                _ => None,
            };
            assert_eq!(side_for(*action), expected, "{action:?}");
        }
    }

    /// With nothing divided, the focused window and the next take the halves;
    /// pressed again on a half, the half becomes two.
    #[test]
    fn a_keyboard_split_divides_the_focused_share_with_the_next_window() {
        let mut division = Division::of(display());
        division
            .divide_with_next(any(1), Some(any(2)), Side::Right)
            .unwrap();
        assert_eq!(
            division.share_of(WindowId::from_compositor(1)),
            Some(area(960, 0, 960, 1080))
        );
        assert_eq!(
            division.share_of(WindowId::from_compositor(2)),
            Some(area(0, 0, 960, 1080))
        );

        division
            .divide_with_next(any(1), Some(any(3)), Side::Left)
            .unwrap();
        assert_eq!(
            division.share_of(WindowId::from_compositor(1)),
            Some(area(960, 0, 480, 1080))
        );
        assert_eq!(
            division.share_of(WindowId::from_compositor(3)),
            Some(area(1440, 0, 480, 1080))
        );

        // The next window already divided elsewhere leaves that share first.
        division
            .divide_with_next(any(1), Some(any(2)), Side::Left)
            .unwrap();
        assert_eq!(division.shares().len(), 3);
        assert_eq!(
            division.share_of(WindowId::from_compositor(1)),
            Some(area(0, 0, 480, 1080))
        );
        assert_eq!(
            division.share_of(WindowId::from_compositor(2)),
            Some(area(480, 0, 480, 1080))
        );
        assert_eq!(
            division.share_of(WindowId::from_compositor(3)),
            Some(area(960, 0, 960, 1080))
        );
    }

    /// Every keyboard split that cannot divide says why, and nothing moves.
    #[test]
    fn a_keyboard_split_that_cannot_divide_says_why() {
        let mut division = divided_in_halves(1, 2);
        let before = division.shares();
        assert_eq!(
            division.divide_with_next(any(1), None, Side::Left),
            Err(Refused::NothingToShareWith)
        );
        assert_eq!(
            division.divide_with_next(any(1), Some(any(1)), Side::Left),
            Err(Refused::NothingToShareWith)
        );
        assert_eq!(
            division.divide_with_next(any(7), Some(any(2)), Side::Left),
            Err(Refused::NotDivided(WindowId::from_compositor(7)))
        );
        assert_eq!(
            division.divide_with_next(any(1), Some(at_least(3, 481, 1)), Side::Left),
            Err(Refused::TooNarrow(WindowId::from_compositor(3)))
        );
        assert_eq!(division.shares(), before);

        let mut empty = Division::of(display());
        assert_eq!(
            empty.divide_with_next(at_least(1, 2000, 1), Some(any(2)), Side::Left),
            Err(Refused::TooNarrow(WindowId::from_compositor(1)))
        );
        assert!(empty.is_empty());
    }
}
