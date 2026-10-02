//! A frame dragged out to the World and dropped into another Place.
//!
//! `docs/autonomy/the-canvas-and-its-places.md` task 3's **pointer road**. The
//! keyboard road is `Move to Place`; this is the one a pointer takes, and
//! together they are the task's *neither road is the only road*.
//!
//! # It adds no gesture, which is the canvas's standing constraint
//!
//! There is no drag-to-the-edge, no hot corner and no drop target. A person
//! **takes hold of a frame by its name** — `crate::window_move`, ADR 0071's road —
//! and then **zooms out** the way they always do, which since task 2's gesture
//! reaches the World. The frame is still held. Dropping it on a tile puts it on
//! that Place. Three things a person already knows, in an order nobody had to be
//! told.
//!
//! # What a drag means changes with the level, and that is the whole of this file
//!
//! On a Place, a drag moves a frame **to a point**: `crate::canvas_fixed_controls`
//! proposes a position, the never-lost rule judges it, and the placement is
//! written. In the World there is no point to move it to — the World's
//! coordinates are the tiles' and a frame's are its own Place's, and writing one
//! into the other would move a window by the distance between two unrelated
//! planes.
//!
//! So **in the World a drag writes no placement at all.** It is a question about
//! *which Place*, answered on release. A frame being dragged in the World is
//! exactly where it was and keeps its point and its size, which is also the
//! task's constraint: *a frame belongs to exactly one Place at every moment —
//! there is no between, and a frame in flight has the Place it started on until
//! it has the one it ends on.*
//!
//! # And a drop on nothing is a drop on nothing
//!
//! Released over the gap between two tiles, the frame stays where it was, on the
//! Place it was already on. Choosing the nearest Place for somebody who aimed
//! between two is the canvas's standing refusal to move a person's things without
//! their saying so, and here it would move a window rather than a view.

use alo_canvas::{At, Place, Showing};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

impl crate::Server {
    /// Whether a drag in progress is choosing a Place rather than a point.
    ///
    /// **Asked before a placement is written**, not after. The alternative —
    /// write the position and correct it on release — is the snap-back the owner
    /// forbade for the never-lost rule, and here it would be worse: the frame
    /// would visibly move by the distance between two unrelated coordinate
    /// spaces before being put back.
    #[must_use]
    pub fn a_drag_is_choosing_a_place(&self) -> bool {
        self.showing() == Showing::TheWorld && self.surfaces.window_move.is_some()
    }

    /// Drop the frame this drag holds onto the Place under this point.
    ///
    /// Answers the Place it was put on, or [`None`] where the drag was not
    /// choosing a Place, or the point is between tiles, or the frame is already
    /// on the Place it was dropped on — which is a drop that changed nothing and
    /// should not read as a move.
    ///
    /// **A move and never a copy**, by the same mechanism the keyboard road uses:
    /// `crate::canvas_place::put_on` replaces, so the frame is on the new Place
    /// and is not on the old one, with no moment at which it is on both.
    ///
    /// **The work goes with it**, which here means nothing is done to it: the
    /// frame keeps its point and its size, and only the surface those are on
    /// changes.
    pub fn drop_the_frame_into_the_place_at(&mut self, point: At) -> Option<Place> {
        if !self.a_drag_is_choosing_a_place() {
            return None;
        }
        let held = self
            .surfaces
            .window_move
            .as_ref()
            .map(|m| m.root().clone())?;
        let onto = self.the_world().the_place_at(point)?;
        if self.the_place_of_the_window(&held) == Some(onto) {
            return None;
        }
        crate::canvas_place::put_on(&held, onto);
        Some(onto)
    }

    /// The frame a drag is holding, if one is held.
    ///
    /// Here so a test can name the frame it is about rather than inferring it
    /// from whichever surface happens to be front-most.
    #[must_use]
    pub fn the_frame_a_drag_holds(&self) -> Option<WlSurface> {
        self.surfaces.window_move.as_ref().map(|m| m.root().clone())
    }
}
