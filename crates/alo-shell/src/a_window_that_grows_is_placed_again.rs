//! **A window alo placed is placed again if it grows over another before the
//! person has touched it.**
//!
//! `docs/features.md`: *a new window never lands on top of another — the view
//! glides to empty space*. `crate::a_new_window_is_placed` places a window
//! once, on the first frame after its first buffer, at **the size that buffer
//! had**. Some applications draw a small first frame and only then settle at
//! their real size, and a window placed in a gap that fitted the first size can
//! cover its neighbours at the second.
//!
//! Measured, not supposed: the canvas layout kept on the walk machine (an image
//! of `main` at 57b98c15) recorded VSCodium (Electron, `com.vscodium.codium`)
//! at 457 × 157 and, at the same corner later, at 1200 × 800. A space found for
//! the first is no promise about the second.
//!
//! # The rule
//!
//! A window alo placed keeps a note of the size it was placed at, and of
//! whether the person opened it. On a later frame, if its size has changed
//! **and** it now covers another window on its Place, it is placed again
//! exactly as a new window of that size would be — beside the active window
//! where there is room, and where there is none and the person opened it and
//! kept *Glide to empty space*, in the nearest empty space with the view gliding
//! there. A window that grew and still covers nothing stays where it is.
//!
//! # Never a window the person placed
//!
//! The note is dropped the moment anything else positions the window — a drag,
//! a resize from its edges, a maximise, a restore, *bring back* — because every
//! one of those goes through `crate::window_placement::set`, which forgets it.
//! It is dropped too when the person presses on the window
//! (`crate::pointer`): a window somebody has started working in does not jump
//! from under their pointer because its application resized it. And it is
//! dropped when the window unmaps, with its placement.
//!
//! **This makes an existing promise true rather than adding one**: no setting
//! is offered and no rule is new. It is rule 6 of
//! `docs/design/where-a-new-window-opens.md` — *use the application's requested
//! initial size* — read as the size the application settled at rather than the
//! first one it drew.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Size};
use smithay::wayland::compositor::with_states;
use std::sync::Mutex;

/// What alo placed a window at, kept until the person or anything else moves it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AsAloPlacedIt {
    /// The window's size when it was placed, without its strip.
    pub(crate) size: Size<i32, Logical>,
    /// Whether the person opened it — so placing it again may move the view,
    /// as the first placing could (rule 8).
    pub(crate) theirs: bool,
}

/// Private compositor data on the window's surface; clients cannot write it.
#[derive(Default)]
struct Settling(Mutex<Option<AsAloPlacedIt>>);

/// **alo placed this window**, at this size: note it, so a later growth over
/// another window can be answered.
pub(crate) fn alo_placed_it(surface: &WlSurface, size: Size<i32, Logical>, theirs: bool) {
    with_states(surface, |states| {
        states.data_map.insert_if_missing(Settling::default);
        if let Some(data) = states.data_map.get::<Settling>() {
            *data.0.lock().unwrap_or_else(|_| std::process::abort()) =
                Some(AsAloPlacedIt { size, theirs });
        }
    });
}

/// **Something other than alo's first placing positioned it**, or the person
/// pressed on it: it is never placed again on its own.
pub(crate) fn it_is_settled(surface: &WlSurface) {
    with_states(surface, |states| {
        if let Some(data) = states.data_map.get::<Settling>() {
            *data.0.lock().unwrap_or_else(|_| std::process::abort()) = None;
        }
    });
}

/// What alo placed this window at, while that is still the last word on it.
pub(crate) fn as_alo_placed_it(surface: &WlSurface) -> Option<AsAloPlacedIt> {
    with_states(surface, |states| {
        states
            .data_map
            .get::<Settling>()
            .and_then(|data| *data.0.lock().unwrap_or_else(|_| std::process::abort()))
    })
}

impl crate::Server {
    /// **Place again every window alo placed that has since grown over
    /// another**, against this frame's view. Returns how many moved.
    pub(crate) fn place_again_every_window_that_grew(
        &mut self,
        frame: &crate::a_new_window_is_placed::ThisFramesView,
    ) -> usize {
        let grown: Vec<(WlSurface, AsAloPlacedIt, Size<i32, Logical>)> = self
            .mapped_surfaces()
            .filter_map(|surface| {
                let was = as_alo_placed_it(surface)?;
                let now = crate::a_new_window_is_placed::the_size_it_wants(surface)?;
                (now != was.size).then(|| (surface.clone(), was, now))
            })
            .collect();
        let mut moved = 0;
        for (surface, was, now) in grown {
            // A maximised or tiled window is where its mode put it, and the
            // mode is the person's.
            if self.surfaces.has_window_mode(&surface) {
                it_is_settled(&surface);
                continue;
            }
            let (open, _) = self.the_windows_a_new_one_opens_among(&surface);
            let where_it_is = crate::where_a_window_is(&surface);
            let here = frame.strip.round(where_it_is);
            let covers = frame
                .strip
                .round_each(&open)
                .iter()
                .any(|other| other.overlaps(here));
            // **Or has grown partly out of the room the Dock leaves** — under
            // the Dock, or off the screen's edge — which Firefox did on
            // 2026-11-12 (`crate::the_room_alo_places_a_window_in`). One wholly
            // out of the view is one the person panned away from, and is left.
            let partly_out = crate::the_room_alo_places_a_window_in::is_partly_out_of(
                where_it_is.loc,
                now,
                frame.room,
                frame.strip,
            );
            if covers || partly_out {
                let _asked = self.ask_it_to_fit(&surface, now, frame.room, frame.strip);
                let point = self.where_this_window_goes(&surface, now, was.theirs, frame);
                crate::window_placement::set(&surface, Some(point));
                moved += 1;
            }
            // Still alo's, at its new size: a window that grows twice is
            // answered twice.
            alo_placed_it(&surface, now, was.theirs);
        }
        moved
    }
}
