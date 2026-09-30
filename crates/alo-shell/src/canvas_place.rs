//! Which Place each window is on, and which one the person is looking at.
//!
//! `docs/autonomy/the-canvas-and-its-places.md` task 1, the shell's half:
//! *every mapped window answers which Place it is on.* The identity itself is
//! [`alo_canvas::Place`]'s — this crate does not define one, and the plan says
//! why: the canvas owns what a Place is, and a second `PlaceId` invented here
//! would be the `one-plane-two-vocabularies` fault committed deliberately by
//! people who had just finished writing that fault down.
//!
//! # Where a window's Place is kept, and why not beside its position
//!
//! In the surface's own `data_map`, exactly as `crate::window_placement` keeps a
//! placement — private compositor state a client cannot write through the
//! protocol. **What is different is the lifetime.**
//!
//! A placement is reset on unmap, because it belongs to one mapping lifetime: a
//! position from a previous mapping is a position the window no longer has. A
//! Place is not reset, because it is the opposite kind of fact. A window that
//! unmaps and maps again — the XDG fresh-handshake path in
//! `crate::surfaces::commit` — is the same window on the same surface, and
//! putting it back on whichever Place the person happens to be looking at would
//! move somebody's window while their back was turned. So a Place is written
//! once, when the toplevel is created, and lives as long as the surface does.
//!
//! # Why it is assigned at creation rather than at mapping
//!
//! So that *no toplevel this compositor knows about is ever without one*. The
//! alternative — assign it the moment a window maps — leaves a window of time
//! where a surface exists with no Place, and every reader of that state then
//! needs an answer for a case that should not arise. [`alo_canvas::Place`]
//! deliberately has no `Default` for that reason; this module is what makes the
//! absence affordable, by leaving nothing for a default to have to cover.

use alo_canvas::Place;
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface, wayland::compositor::with_states,
};
use std::sync::Mutex;

/// Private compositor data; clients cannot write this through protocol state.
#[derive(Default)]
struct OnAPlace(Mutex<Option<Place>>);

/// Which Place this surface is on, or [`None`] if it is not a toplevel this
/// compositor created.
#[must_use]
pub(crate) fn the_place_of(surface: &WlSurface) -> Option<Place> {
    with_states(surface, |states| {
        states
            .data_map
            .get::<OnAPlace>()
            .and_then(|data| *data.0.lock().unwrap_or_else(|_| std::process::abort()))
    })
}

/// Put this surface on this Place.
///
/// Called once when a toplevel is created, and again by *Move to Place* when a
/// person moves a window between surfaces. Never called on unmap — see this
/// module's note on lifetime.
pub(crate) fn put_on(surface: &WlSurface, place: Place) {
    with_states(surface, |states| {
        states.data_map.insert_if_missing(OnAPlace::default);
        if let Some(data) = states.data_map.get::<OnAPlace>() {
            *data.0.lock().unwrap_or_else(|_| std::process::abort()) = Some(place);
        }
    });
}

impl crate::Server {
    /// Which Place the person is looking at.
    #[must_use]
    pub fn the_place_now(&self) -> Place {
        self.surfaces.place
    }

    /// Which Place this window is on, or [`None`] for a surface that is not one
    /// of this compositor's toplevels.
    #[must_use]
    pub fn the_place_of_the_window(&self, surface: &WlSurface) -> Option<Place> {
        the_place_of(surface)
    }
}

impl crate::Server {
    /// Put this window on this Place.
    ///
    /// **The primitive, not the road.** Task 3 of the canvas plan is *a frame
    /// moves between Places, by pointer and by keyboard* — a drag out through the
    /// World, and a *Move to Place* a keyboard can reach. Neither exists yet, and
    /// this is not either of them: it is the one line underneath both, which task
    /// 1 needs in order to have a Place that can be observed to change at all.
    ///
    /// Nothing about the window's rectangle changes, which is task 3's *the work
    /// goes with it* stated where it is enforced rather than where it is promised.
    pub fn move_the_window_to(&mut self, surface: &WlSurface, place: Place) {
        put_on(surface, place);
    }
}
