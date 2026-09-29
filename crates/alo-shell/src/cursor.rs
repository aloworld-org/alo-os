//! Authorized cursor state and backend presentation snapshots.

use crate::Server;
use smithay::{
    input::pointer::{CursorImageAttributes, CursorImageStatus},
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
    utils::{Logical, Point},
    wayland::compositor::with_states,
};
use std::sync::Mutex;

/// Authorized client cursor or compositor-owned fallback, for trusted backends.
#[derive(Debug, Clone)]
pub enum Cursor {
    /// Unpositioned legacy snapshot (no pointer seat); nested uses its host arrow.
    /// Offscreen/direct rendering adds no pixels for this variant.
    Default,
    /// Compositor-owned scale-one arrow, including after focus loss/destruction.
    Arrow {
        /// Logical tip position; the arrow's hotspot is (0, 0).
        /// Rendering floors fractional coordinates and refuses non-finite values.
        location: Point<f64, Logical>,
    },
    /// The pointer is on a frame's own resize band, and says which way it drags.
    ///
    /// ADR 0071: *the pointer is the affordance.* A frame shows no furniture, so
    /// this is the only thing that tells a person the border under them resizes
    /// rather than moves.
    Resize {
        /// Which edge or corner, which is what picks the arrow.
        edge: crate::FrameEdge,
        /// Where the pointer is. The arrow's hotspot is its middle, not a tip,
        /// so what is drawn is offset from this rather than starting at it.
        location: Point<f64, Logical>,
    },
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
        let default = Cursor::Arrow {
            location: pointer.location,
        };
        if pointer.handle.current_focus().is_none() {
            // Nothing of a client's is under the pointer, so the shell's own
            // bands may answer. A frame shows no furniture (ADR 0065), which
            // makes this arrow the only thing that says the border under a
            // person resizes rather than moves — ADR 0071's *the pointer is the
            // affordance*. Asked here rather than earlier so that anywhere a
            // client actually drew, including outside its declared geometry, the
            // cursor stays that client's business.
            if let Some((_, edge)) = self.the_edge_under(pointer.location) {
                return Cursor::Resize {
                    edge,
                    location: pointer.location,
                };
            }
            return default;
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
            _ => default,
        }
    }
}
