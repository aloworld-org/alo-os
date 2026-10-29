//! The server's answer about what a pointer is on, for one window's edge.
//!
//! `crate::window_edge_pointing` is the geometry — a position and a
//! `WindowEdge` in, a control out — and knows nothing about a server. This is
//! the one method that asks it on a host's behalf, and it exists for exactly
//! one reason: **a host cannot read the pointer.**
//!
//! # What only the server can supply
//!
//! `crate::window_edge::edge_of` is public and so is
//! [`what_is_under`](crate::what_is_under), so a host builds its own edge and
//! could do its own geometry. `where_the_pointer_is_in_logical_pixels` is
//! `pub(crate)`, and that file gives the reason: a pointer that is absent must
//! stay distinguishable from a pointer at the origin. A host made to supply a
//! position would have to invent one for a machine whose seat has never had a
//! pointing device, and the invented answer highlights a control on whichever
//! window happens to sit at `(0, 0)`.
//!
//! So this method takes the edge and adds the pointer. It deliberately does
//! **not** build the edge: the host already has one — it needs it to draw —
//! and a second `edge_of` here would be a second layout that could disagree
//! with the one being painted.
//!
//! # Why not in the file that reveals
//!
//! `crate::the_window_edge_reveals` owns a state machine: one `Revealing` per
//! mapped window, advanced once a frame, pruned when a window closes. Hover
//! holds no state at all — it is this frame's pointer against this frame's
//! layout, and the answer is discarded with the frame. A design change to the
//! reveal delay and a design change to what a press lands on are two
//! different changes, so they are two files (law 4).

use crate::window_edge::{OnTheEdge, WindowEdge};
use crate::window_edge_paint::Pointing;

impl crate::Server {
    /// How this edge should be drawn, for wherever the pointer is.
    ///
    /// The value [`crate::EdgePicture::of`] takes. `edge` is the same
    /// `WindowEdge` the frame is about to be painted from — pass the one you
    /// built, not a fresh one.
    ///
    /// `focused` is the host's to supply and is passed straight through.
    ///
    /// **This said nothing in this repository could focus a control, and that
    /// is no longer true.** §6 built the keyboard road:
    /// `crate::Server::where_the_keyboard_is_on_this_windows_edge` holds one
    /// state per window and
    /// `crate::TheKeyboardOnTheEdge::what_the_painter_shows` is the value to
    /// pass. `examples/a_real_application` passes it, which is what the old
    /// wording promised would happen — the call site changed and this method
    /// did not.
    ///
    /// Answers nothing hovered in three cases, and they are one picture
    /// deliberately — the design draws a highlight on the hovered control and
    /// on nothing else: there is no pointer on this machine, the pointer is
    /// not on this edge, or it is on the edge and not on a control.
    ///
    /// **An edge laid out as concealed has no control to be over**, so this
    /// answers nothing hovered for one however near the pointer is. Call it
    /// after
    /// [`the_edges_were_told_where_the_pointer_is`](crate::Server::the_edges_were_told_where_the_pointer_is),
    /// which is what advances the reveal that `edge_of` then lays out.
    #[must_use]
    ///
    /// The pointer is read at the scale of the display drawn last
    /// (`crate::an_edge_in_its_displays_units`); a draw for one particular
    /// display asks `what_this_edge_is_pointing_at_on` with its own.
    pub fn what_this_edge_is_pointing_at(
        &self,
        edge: &WindowEdge,
        focused: Option<OnTheEdge>,
    ) -> Pointing {
        self.what_this_edge_is_pointing_at_on(
            edge,
            focused,
            self.the_scale_an_edge_is_measured_at(),
        )
    }

    /// The same answer for an edge laid out on a display drawn at `scale` per
    /// cent: the pointer is taken into that edge's units first, so on a dense
    /// display the highlight is on the control under the pointer.
    pub(crate) fn what_this_edge_is_pointing_at_on(
        &self,
        edge: &WindowEdge,
        focused: Option<OnTheEdge>,
        scale: u32,
    ) -> Pointing {
        crate::window_edge_pointing::pointing_at(
            edge,
            self.where_the_pointer_is_in_logical_pixels().map(|at| {
                crate::an_edge_in_its_displays_units::a_point_in_the_edges_units(at, scale)
            }),
            focused,
        )
    }
}

#[cfg(test)]
#[path = "the_window_edge_is_pointed_at_tests.rs"]
mod tests;
