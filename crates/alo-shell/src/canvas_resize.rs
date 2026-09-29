//! **The shell's own resize**, begun from a press on an edge or a corner.
//!
//! Task 4 of `docs/autonomy/the-smallest-canvas-worth-showing.md`, and ADR 0071's
//! half of it: the edges and the corners resize, the name moves, and the cursor
//! says which.
//!
//! # One resize, reached from the other end
//!
//! `crate::resize_transaction` already resizes a frame properly — it configures
//! the client **while the drag is happening**, refreshes the client's limits on
//! every motion so a minimum that arrives mid-drag is honoured, and anchors the
//! opposite edge so a corner drag does not slide the window. All of that was
//! written for `xdg_toplevel.resize`, the road ADR 0071 refuses.
//!
//! So this begins the same transaction from a press on the shell's own band
//! instead. **Not a second implementation**: a compositor with two ways to resize
//! is a compositor where one of them is subtly wrong, and the plan's constraint
//! against two layout deciders is the same argument one level down.
//!
//! # Why the client road has to be refused in the same change
//!
//! ADR 0071 is explicit and the ordering is the whole of it: *the resize refusal
//! ships welded to the edge gesture and the cursor, never separated from them.*
//! Nothing in this shell could resize before this file, so refusing the client's
//! request first would have left a person unable to resize anything at all. Until
//! both exist a client can still move itself by resizing from the left edge and
//! then the right — which composes into a translation — so the refusal is not
//! optional once the gesture lands.

use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
};

use crate::{FrameEdge, window_resize::ResizeEdge};

impl FrameEdge {
    /// The same edge, as the resize arithmetic names it.
    ///
    /// Two vocabularies for one idea, joined here rather than by making
    /// `crate::frame_edges` speak the resize crate's language: what a *pointer is
    /// over* and what a *resize anchors against* are different questions that
    /// happen to have the same eight answers, and a band that was named after the
    /// arithmetic would be harder to read where it is used.
    pub(crate) const fn as_resize_edge(self) -> ResizeEdge {
        match self {
            Self::Top => ResizeEdge::Top,
            Self::Bottom => ResizeEdge::Bottom,
            Self::Left => ResizeEdge::Left,
            Self::Right => ResizeEdge::Right,
            Self::TopLeft => ResizeEdge::TopLeft,
            Self::TopRight => ResizeEdge::TopRight,
            Self::BottomLeft => ResizeEdge::BottomLeft,
            Self::BottomRight => ResizeEdge::BottomRight,
        }
    }
}

impl crate::Server {
    /// Begin a resize because a press landed on a frame's edge or corner.
    ///
    /// `button` is the one being pressed, which the seat has not recorded yet:
    /// `crate::pointer` asks the bands before it delivers or records anything, so a
    /// resize that waited for the seat would begin on the second click.
    ///
    /// Whether one began. `false` where the press was not on a band, where the
    /// frame has no geometry to resize from, and where a resize or a move is
    /// already under way — a second grab would be two gestures arguing about one
    /// window.
    ///
    /// **The press is not passed on.** A band is the shell's own, outside the
    /// frame, so no client was under it; the alternative would be telling an
    /// application about a press that was never inside it.
    pub fn resize_from_the_edge_under(&mut self, at: Point<f64, Logical>, button: u32) -> bool {
        if self.surfaces.window_resize.is_some() || self.surfaces.window_move.is_some() {
            return false;
        }
        // **Where the application drew, the press is the application's** — ADR
        // 0065, and the band yields to it rather than the other way round.
        //
        // A band is measured from the frame's *window geometry*, which is what a
        // person sees as the window; a client may have surfaces outside it, and a
        // subsurface hanging past the bottom-right corner is real content a person
        // clicked on, not a place to start a resize. Asked as *is anything of a
        // client's under this point* rather than by comparing rectangles, so the
        // answer is the same one `pointer_motion` and the renderer already give.
        if self.pointer_target(at).is_some() {
            return false;
        }
        let Some((frame, edge)) = self.the_edge_under(at) else {
            return false;
        };
        let Some(role) = self.surfaces.mapped_toplevel(&frame).cloned() else {
            return false;
        };
        self.surfaces
            .begin_resize_on_the_shells_own_band(role, edge.as_resize_edge(), Some(button))
    }

    /// Begin one on a named frame and edge, however the caller decided.
    ///
    /// Separate from the hit test above so that what this compositor *does* with
    /// an edge can be asked without a pointer, which is the same separation
    /// `crate::desktop_swipes` keeps between recognising a gesture and carrying
    /// one out — and so the transaction's own tests can drive it, which they did
    /// through `xdg_toplevel.resize` until ADR 0071 refused that road.
    #[must_use]
    pub fn begin_a_resize(&mut self, frame: &WlSurface, edge: FrameEdge) -> bool {
        let Some(role) = self.surfaces.mapped_toplevel(frame).cloned() else {
            return false;
        };
        self.surfaces
            .begin_resize_on_the_shells_own_band(role, edge.as_resize_edge(), None)
    }
}

// **`resize_button` was here, and it was a second road to one door.**
//
// It filtered for BTN_LEFT and forwarded to `Surfaces::window_resize_button` —
// which `Server::pointer_button` has called on its own first line of button
// handling since long before this file existed, for every button rather than one.
// So the drag already ended correctly and this ended it a second, narrower way,
// which is the shape of the *two ways to resize* this file's own header argues
// against. Nothing called it; deleting it is the same judgement one level down.
