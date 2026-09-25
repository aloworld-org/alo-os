//! A chord that puts a window on a side, answered by the division.
//!
//! **This is what replaces `crate::window_tiling`'s half.** That file computes
//! half of an output and sets a window mode; a display holding a division has a
//! tree of shares, and a window's place must come from one of them and not
//! both. The shell plan's constraint says so in as many words, and task 10
//! wrote down that the moment a `Division` became `Server` state there would be
//! two layout deciders unless one went.
//!
//! So a chord now asks `alo_dividing::Division::divide_with_next`: *divide the
//! focused window's share with the next window, the focused one on this side*.
//! The side is `alo_dividing::keyboard::side_for`'s answer, as it has been
//! since 2026-09-22, and what is new is that the **layout** is the division's
//! too.
//!
//! # What a window says it needs is carried across
//!
//! A window's minimum size is read from its own XDG state and handed over, so
//! `alo-dividing` can refuse a split that would squeeze either window below
//! what it asked for. Handing over *any size* would have made that refusal
//! impossible to reach, which is a protection lost rather than a detail
//! skipped.

use alo_dividing::{Refused, Side, Window, area::Size};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::compositor::with_states;

/// Why a chord could not divide the display.
#[derive(Debug, thiserror::Error)]
pub enum NotDivided {
    /// The division refused. `alo-dividing`'s own answer, carried whole: what
    /// a person is told about it is that crate's sentence and never one
    /// assembled here.
    #[error("the division refused: {0:?}")]
    Dividing(Refused),
    /// There is no display to divide — nothing has arrived yet.
    #[error("this session has no display to divide")]
    NoDisplay,
    /// There is more than one, and which display a window is on is
    /// `alo-displays`' answer, which nothing here asks yet.
    #[error(
        "this machine has more than one display, and nothing here decides which a window is on"
    )]
    MoreThanOneDisplay,
}

impl crate::Server {
    /// Put the focused window on `side`, sharing with the window next in order.
    ///
    /// The layout is `alo-dividing`'s from end to end: which side a chord
    /// means, whether there is anything to share with, whether either window
    /// would be squeezed, and where the boundary falls. What this does is find
    /// the two windows and hand them over.
    ///
    /// # Errors
    /// [`NotDivided`]. A refusal leaves the division exactly as it was — that
    /// is `alo-dividing`'s guarantee, not this file's.
    pub fn divide_focused_with_next(
        &mut self,
        focused: &WlSurface,
        side: Side,
    ) -> Result<(), NotDivided> {
        let display = self.the_only_display()?;
        let next = self.next_in_order(focused);
        let focused_window = self.window_for(focused);
        let next_window = next.as_ref().map(|next| self.window_for(next));
        // A division is changed where it lives: it is not `Clone`, because it
        // carries an identity and a count of its changes that two copies would
        // both claim.
        if self.desk.dividing(display).is_none() {
            let area = self.desk.area_of(display).ok_or(NotDivided::NoDisplay)?;
            self.desk.divide(display, alo_dividing::Division::of(area));
        }
        self.desk
            .dividing_mut(display)
            .ok_or(NotDivided::NoDisplay)?
            .divide_with_next(focused_window, next_window, side)
            .map_err(NotDivided::Dividing)
    }

    /// The one display this session has.
    ///
    /// A machine with two screens needs `alo-displays` to say which one a
    /// window is on, and nothing here asks it yet — so two is refused by name
    /// rather than answered with a guess about the first one.
    fn the_only_display(&self) -> Result<alo_desktops::DisplayId, NotDivided> {
        let mut displays = self.desk.displays();
        let first = displays.next().ok_or(NotDivided::NoDisplay)?;
        if displays.next().is_some() {
            return Err(NotDivided::MoreThanOneDisplay);
        }
        Ok(first)
    }

    /// The window after this one in the order a person switches through.
    fn next_in_order(&mut self, focused: &WlSurface) -> Option<WlSurface> {
        self.switch_order.refresh(self.surfaces.buffered());
        self.switch_order
            .in_order()
            .filter(|root| self.surfaces.mapped_toplevel(root).is_some())
            .find(|root| *root != focused)
            .cloned()
    }

    /// This surface as `alo-dividing` holds a window: its number and the
    /// smallest size it says it can be drawn at.
    fn window_for(&mut self, surface: &WlSurface) -> Window {
        let id = self.desk.number_of(surface);
        match smallest(surface) {
            Some(minimum) => Window::at_least(id, minimum),
            None => Window::any_size(id),
        }
    }
}

/// The smallest size a window says it can be drawn at, in logical units.
///
/// [`None`] where it has said nothing, which is a window with no minimum rather
/// than one of no size.
fn smallest(surface: &WlSurface) -> Option<Size> {
    with_states(surface, |states| {
        let min = states
            .cached_state
            .get::<smithay::wayland::shell::xdg::SurfaceCachedState>()
            .current()
            .min_size;
        (min.w > 0 && min.h > 0).then(|| {
            Size::of(
                u32::try_from(min.w).unwrap_or(u32::MAX),
                u32::try_from(min.h).unwrap_or(u32::MAX),
            )
        })
    })
}

#[cfg(test)]
#[path = "window_dividing_tests.rs"]
mod tests;
