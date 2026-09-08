//! Native root placement shared by every scene consumer.

use crate::{InputError, Server};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
    wayland::compositor::with_states,
};
use std::sync::Mutex;

/// A refused placement, or a failure refreshing input after placement changed.
#[derive(Debug, thiserror::Error)]
pub enum WindowPlacementError {
    /// Only this display's live mapped roots may be placed.
    #[error("placement target is not a mapped toplevel in this display")]
    Unmapped,
    /// Coordinates must be within one million logical pixels of the origin.
    #[error("window placement exceeds the supported coordinate range")]
    OutOfRange,
    /// Placement changed, but refreshing the existing pointer route failed.
    #[error("window placed but pointer refresh failed: {0}")]
    Pointer(#[from] InputError),
}

/// Private compositor data; clients cannot write this through protocol state.
#[derive(Default)]
struct Placement(Mutex<Option<Point<i32, Logical>>>);

/// Buffer origin for scene consumers, subtracting only committed XDG geometry.
/// Unplaced windows retain the initial buffer origin of zero.
/// Read on the display's dispatch/render thread, like Smithay renderer state.
pub fn window_buffer_origin(surface: &WlSurface) -> Point<f64, Logical> {
    let placement = with_states(surface, |states| {
        states
            .data_map
            .get::<Placement>()
            .and_then(|data| *data.0.lock().unwrap_or_else(|_| std::process::abort()))
    });
    placement.map_or_else(Point::default, |point| {
        point.to_f64() - crate::scene::geometry_origin(surface)
    })
}

/// Placement belongs to one mapping lifetime, not the reusable protocol object.
pub(crate) fn reset(surface: &WlSurface) {
    set(surface, None);
}

/// Replace compositor-owned placement on the display thread.
fn set(surface: &WlSurface, point: Option<Point<i32, Logical>>) {
    with_states(surface, |states| {
        states.data_map.insert_if_missing(Placement::default);
        if let Some(data) = states.data_map.get::<Placement>() {
            *data.0.lock().unwrap_or_else(|_| std::process::abort()) = point;
        }
    });
}

impl Server {
    /// Place a mapped window's committed geometry origin in logical output space.
    ///
    /// This trusted shell primitive is not an agent verb or interactive drag.
    /// Negative/offscreen positions are permitted within +/-1,000,000 per axis;
    /// this bounds upstream popup constraint arithmetic without clamping intent.
    /// Placement changes immediately, with no client configure, buffer resize,
    /// activation or stacking change. All scene consumers use the same origin.
    /// Committed geometry changes keep the requested geometry origin anchored;
    /// unmapping resets placement. Pointer refresh respects existing grabs and
    /// reactive popup constraints update through normal display dispatch.
    pub fn place_window(
        &mut self,
        surface: &WlSurface,
        position: (i32, i32),
    ) -> Result<(), WindowPlacementError> {
        if self.surfaces.mapped_toplevel(surface).is_none() {
            return Err(WindowPlacementError::Unmapped);
        }
        if ![position.0, position.1]
            .into_iter()
            .all(|value| (-1_000_000..=1_000_000).contains(&value))
        {
            return Err(WindowPlacementError::OutOfRange);
        }
        set(surface, Some(position.into()));
        if let Some((location, time)) = self.surfaces.pointer_position() {
            self.pointer_motion(location.x, location.y, time)?;
        }
        Ok(())
    }
}
