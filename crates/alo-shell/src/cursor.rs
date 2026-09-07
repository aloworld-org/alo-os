//! Authorized cursor state and backend presentation snapshots.

use crate::Server;
use smithay::{
    input::pointer::{CursorImageAttributes, CursorImageStatus},
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
    utils::{Logical, Point},
    wayland::compositor::with_states,
};
use std::sync::Mutex;

/// Cursor selected by the focused client, for trusted display backends only.
#[derive(Debug, Clone)]
pub enum Cursor {
    /// Use the backend's ordinary arrow (also after focus loss or destruction).
    Default,
    /// The focused client explicitly requested no cursor.
    Hidden,
    /// Draw this tree above windows at its hotspot-adjusted logical origin.
    Surface {
        /// Authorized cursor-role surface; it may currently have no buffer.
        surface: WlSurface,
        /// Pointer position minus the requested cursor hotspot, at output scale one.
        location: Point<f64, Logical>,
    },
}

impl Server {
    /// Snapshot cursor state without reading application context or dispatching.
    pub fn cursor(&self) -> Cursor {
        let Some(pointer) = &self.surfaces.pointer else {
            return Cursor::Default;
        };
        if pointer.handle.current_focus().is_none() {
            return Cursor::Default;
        }
        match &self.surfaces.cursor {
            CursorImageStatus::Hidden => Cursor::Hidden,
            CursorImageStatus::Surface(surface) if surface.is_alive() => {
                let hotspot = with_states(surface, |states| {
                    states
                        .data_map
                        .get::<Mutex<CursorImageAttributes>>()
                        .and_then(|attributes| attributes.lock().ok().map(|a| a.hotspot))
                        .unwrap_or_default()
                });
                // Keep full-range client hotspots out of integer arithmetic.
                let location = pointer.location - hotspot.to_f64();
                Cursor::Surface {
                    surface: surface.clone(),
                    location,
                }
            }
            _ => Cursor::Default,
        }
    }
}
