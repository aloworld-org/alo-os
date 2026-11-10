//! **Left and Right in the window panel**: this one window on one half of its
//! display's work area, and nothing else moved.
//!
//! `docs/design/the-window-controls.md`, frame `11:84`: *Left and Right preview
//! where the window would go when pointed at or reached by keyboard. The
//! preview never moves or resizes the real window. Click or Enter commits;
//! Escape or dismissing cancels. Other windows are not touched. The destination
//! is the display's available work area less what the shell reserves, and the
//! window's minimum size is respected; if it cannot fit, the person is told
//! rather than given a smaller half.*
//!
//! # Not the division's operation, and still the division's half
//!
//! The `Super+Left` and `Super+Right` chords ask `alo-dividing` to divide the
//! focused window's share **with the next window** (`crate::window_dividing`),
//! which moves that second window too. The panel's choice is promised to touch
//! no other window, so it is not that operation. It puts this window into a
//! share of its own — the same mode a division's share uses, so a client is
//! told it is tiled — on half the work area of **the display the window is
//! on** (`crate::the_room_a_window_fills`), which is also what lets it work on
//! a machine with more than one display, where the division refuses.
//!
//! **The half is not worked out here.** `tests/one_layout_decider.rs` holds
//! that one file builds a window's share and that it asks `alo-dividing`
//! rather than dividing a display itself, so this asks
//! `crate::window_dividing::alone_on_one_side`, which asks
//! `alo_dividing::alone_on` — cut by the same rule the chords' halving uses,
//! the odd unit on the right. A window put on the left by the panel and by the
//! chord gets the same half.
//!
//! # The half
//!
//! The work area cut down the middle, with the window's band at the top of
//! it, as a maximised window's is. A window whose own minimum is wider or
//! taller than that is refused with [`NoRoom`], and the refusal is what the
//! preview says before anything is chosen, so the person is told at the moment
//! they look.

use alo_dividing::{Refused, Side, Window, WindowId};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Rectangle};

use crate::the_room_a_window_fills::ADisplaysRoom;
use crate::window_mode::Share;

/// Which half.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OneSide {
    /// The left half.
    Left,
    /// The right half.
    Right,
}

/// Why a window cannot go on a half.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoRoom {
    /// It says it cannot be made as narrow as the half.
    TooNarrow,
    /// It says it cannot be made as short as the half under its band.
    TooShort,
}

/// Where a window on this side would go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheHalf {
    /// The half on the plane, band included — what the preview outlines.
    pub(crate) outline: Rectangle<i32, Logical>,
    /// The window's own share of it, under its band, or why it cannot have
    /// one.
    pub(crate) share: Result<Share, NoRoom>,
}

impl OneSide {
    /// The side `alo-dividing` calls this.
    const fn in_the_division(self) -> Side {
        match self {
            Self::Left => Side::Left,
            Self::Right => Side::Right,
        }
    }
}

/// This side of this room, for this window.
///
/// [`None`] where the room has no half to give — a work area too small to
/// hold a band and a window under it on either side.
pub(crate) fn the_half(room: ADisplaysRoom, side: OneSide, window: Window) -> Option<TheHalf> {
    let (outline, share) = crate::window_dividing::alone_on_one_side(
        room.work,
        room.band,
        side.in_the_division(),
        window,
    )?;
    let share = share.map_err(|refused| match refused {
        Refused::TooShort(_) => NoRoom::TooShort,
        // `alone_on` answers only the two squeezes; the rest are a
        // division's and are named so a new one is a compile error here.
        Refused::TooNarrow(_)
        | Refused::NotDivided(_)
        | Refused::NothingToShareWith
        | Refused::NoNeighbour { .. }
        | Refused::Changed => NoRoom::TooNarrow,
    });
    Some(TheHalf { outline, share })
}

impl crate::Server {
    /// Where this window would go on this side, as the preview shows it.
    pub(crate) fn where_this_side_would_put(
        &self,
        surface: &WlSurface,
        side: OneSide,
    ) -> Option<TheHalf> {
        let room = self.surfaces.the_room_for(surface)?;
        // The number is only carried back in a refusal, which is told as this
        // window's; a window not yet numbered is asked as window 0 rather than
        // given a number by being looked at.
        let id = WindowId::from_compositor(
            crate::window_number::Numbers::given_to(surface).unwrap_or_default(),
        );
        let window = match crate::window_dividing::smallest(surface) {
            Some(minimum) => Window::at_least(id, minimum),
            None => Window::any_size(id),
        };
        the_half(room, side, window)
    }

    /// The preview of the choice in hand, where it is Left or Right: the half
    /// as this display draws it, in the band's units at `scale`, and what its
    /// corner says.
    pub(crate) fn the_preview_of(
        &self,
        surface: &WlSurface,
        in_hand: Option<crate::window_menu::WhatARowDoes>,
        scale: u32,
    ) -> Option<crate::window_menu_paint::ThePreview> {
        let side = in_hand?.the_side()?;
        let half = self.where_this_side_would_put(surface, side)?;
        let drawn =
            crate::the_terminal_on_the_plane::on_the_screen(half.outline, self.the_camera());
        let outline =
            crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(drawn, scale);
        Some(crate::window_menu_paint::ThePreview {
            outline,
            says: the_words_of(side, half.share),
        })
    }

    /// Put this window on this side, moving no other.
    ///
    /// # Errors
    /// [`crate::WindowModeError::OutputUnavailable`] where no display has a
    /// half to give or the window cannot fit in it, and every refusal the
    /// window modes give a move, a resize or a popup grab.
    pub(crate) fn put_this_window_on(
        &mut self,
        surface: &WlSurface,
        side: OneSide,
    ) -> Result<(), crate::WindowModeError> {
        let share = self
            .where_this_side_would_put(surface, side)
            .and_then(|half| half.share.ok())
            .ok_or(crate::WindowModeError::OutputUnavailable)?;
        self.surfaces
            .set_window_mode(surface, crate::window_mode::Mode::InAShare(share))?;
        Ok(())
    }
}

/// What a preview's corner says, and a reader is told: which half and that
/// nothing has moved, or that the window cannot fit there.
pub(crate) const fn the_words_of(side: OneSide, share: Result<Share, NoRoom>) -> alo_strings::Word {
    match (share, side) {
        (Err(_), _) => alo_access::words::NO_ROOM_IN_HALF,
        (Ok(_), OneSide::Left) => alo_access::words::LEFT_HALF_PREVIEW,
        (Ok(_), OneSide::Right) => alo_access::words::RIGHT_HALF_PREVIEW,
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "a_window_on_one_side_tests.rs"]
mod tests;
