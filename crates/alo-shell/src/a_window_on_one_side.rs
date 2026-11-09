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
//! # Not the division, and why
//!
//! The `Super+Left` and `Super+Right` chords ask `alo-dividing` to divide the
//! focused window's share **with the next window** (`crate::window_dividing`),
//! which moves that second window too. The panel's choice is promised to touch
//! no other window, so it is not that operation. It puts this window into a
//! share of its own — the same mode a division's share uses, so a client is
//! told it is tiled — whose rectangle is half the work area of **the display
//! the window is on** (`crate::the_room_a_window_fills`), which is also what
//! lets it work on a machine with more than one display, where the division
//! refuses.
//!
//! # The half
//!
//! The work area split down the middle — the left half takes the odd unit —
//! with the window's band at the top of it, as a maximised window's is. A
//! window whose own minimum is wider or taller than that is refused with
//! [`NoRoom`], and the refusal is what the preview says before anything is
//! chosen, so the person is told at the moment they look.

use alo_dividing::Axis;
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

/// This side of this room, for a window whose minimum is `smallest`.
///
/// [`None`] where the room has no half to give — a work area too small to
/// hold a band and a window under it on either side.
pub(crate) fn the_half(
    room: ADisplaysRoom,
    side: OneSide,
    smallest: Option<(i32, i32)>,
) -> Option<TheHalf> {
    let work = room.work;
    let left_wide = work.size.w - work.size.w / 2;
    let (x, wide) = match side {
        OneSide::Left => (work.loc.x, left_wide),
        OneSide::Right => (work.loc.x + left_wide, work.size.w / 2),
    };
    let tall = work.size.h - room.band;
    if wide <= 0 || tall <= 0 {
        return None;
    }
    let outline = Rectangle::new((x, work.loc.y).into(), (wide, work.size.h).into());
    let share = match smallest {
        Some((w, _)) if w > wide => Err(NoRoom::TooNarrow),
        Some((_, h)) if h > tall => Err(NoRoom::TooShort),
        _ => Ok(Share {
            at: (x, work.loc.y + room.band),
            size: (wide, tall),
        }),
    };
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
        let smallest = crate::window_dividing::smallest(surface).map(|size| {
            let it = |along: u32| i32::try_from(along).unwrap_or(i32::MAX);
            (
                it(size.along(Axis::SideBySide)),
                it(size.along(Axis::OneAboveTheOther)),
            )
        });
        the_half(room, side, smallest)
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
        let drawn = crate::the_terminal_on_the_plane::on_the_screen(half.outline, self.the_camera());
        let outline = crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(
            Rectangle::new(
                (drawn.loc.x, drawn.loc.y).into(),
                (drawn.size.w, drawn.size.h).into(),
            ),
            scale,
        );
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
