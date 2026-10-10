//! Whether a window's edge is revealed, and what a pointer does to that.
//!
//! §5 of `docs/design/the-external-window-edge.md`, the section #603 marked **not
//! built — the host owns pointer, focus, menu and drag**. The geometry and the
//! painter landed there; what no file did was decide *revealed or not*. Measured
//! before this was written: the only caller of [`crate::edge_of`] outside a test
//! was `examples/a_real_application.rs`, and it passed a literal `true`.
//!
//! # Nothing new is decided here, and that is why this file is short
//!
//! [`alo_dock::revealing::Revealing`] already **is** §5. Its four keep-revealed
//! conditions are the specification's four, one for one:
//!
//! | the specification | the machine |
//! |---|---|
//! | the pointer is in the region or on its controls | `the_pointer_is` |
//! | keyboard focus is inside it | `the_keyboard` |
//! | its window menu is open | `a_menu` |
//! | a drag or related interaction is in progress | `a_drag` |
//!
//! And it carries *no timed grace period* as a property of its type rather than
//! as a promise a caller keeps: there is no instant, no duration and no timer
//! anywhere in it, so *a person must never have to move quickly to reach a
//! control* cannot be broken by this file. The same reason
//! [`crate::the_panel_reveals`] is short — this is that file's shape for a
//! second surface, and the two share the machine rather than each implementing
//! one.
//!
//! # The order is the correctness
//!
//! Controls and strip are asked **before** the region, because the region
//! contains them by construction — [`crate::edge_of`] builds every control's
//! target at the region's own top, inside its width — and *on the surface* is
//! the more specific answer. The other order would report a pointer resting on
//! Close as merely asking for the edge, and the machine would then let it
//! conceal from under the pointer.
//!
//! **At rest there is nothing specific to find, which is what makes that order
//! safe rather than lucky.** `edge_of` gives a concealed edge `strip: None` and
//! no controls at all, so a concealed edge is **all region** and the first
//! answer a pointer can get is [`ThePointer::AtTheEdge`] — the one that reveals.
//! Were controls laid out at rest, a pointer arriving directly on one would be
//! classified `OnTheSurface`, and `Revealing` sets *on the surface* to its own
//! current answer, so an edge nothing had revealed yet would refuse to reveal.
//! `crate::the_panel_reveals` records the same dependency for the same reason: a
//! rail of no height contains nothing, which is what makes a concealed panel all
//! strip.
//!
//! # One opinion about one window's edge
//!
//! This asks the `WindowEdge` it is given and nothing else, so it has no opinion about
//! the Dock's edge, the put-aside panel's, or which of them wins where they
//! overlap. `crate::the_panel_reveals` names the arbiter that will go in front of
//! all of them when it exists; the window edge is a fourth claimant on it and
//! changes nothing here when it arrives.

use alo_dock::revealing::{Revealing, ThePointer};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle};

use crate::window_edge::WindowEdge;

/// What this pointer is to one window edge's reveal machine.
///
/// **Asked of the laid-out `WindowEdge`, because that is what a person is looking at.**
/// `crate::edge_of` is the one authoritative calculation the specification asks
/// for — *one authoritative geometry calculation serves rendering, pointer
/// targets and reachability* — so this answers about the edge as it actually
/// appears rather than about a second calculation that would have to agree.
///
/// See this file's header for why the three questions are asked in this order
/// and why that is safe at rest.
pub(crate) fn what_the_pointer_is(edge: &WindowEdge, at: Point<i32, Logical>) -> ThePointer {
    // **`target`, never `highlight`.** The 44 × 44 is what answers; the 32 × 28
    // is only what is filled behind it while hovered. Hit-testing the highlight
    // would shrink every control by six pixels each side and four top and
    // bottom, which is the specification's one hard floor — *44 × 44,
    // non-overlapping* — quietly undone by reading the wrong field of the right
    // struct. The compiler cannot tell these apart: both are
    // `Rectangle<i32, Logical>` on the same value.
    if edge
        .controls
        .iter()
        .any(|control| holds(control.target, at))
    {
        return ThePointer::OnTheSurface;
    }
    if edge.strip.is_some_and(|strip| holds(strip, at)) {
        return ThePointer::OnTheSurface;
    }
    if holds(edge.region, at) {
        return ThePointer::AtTheEdge;
    }
    ThePointer::Elsewhere
}

/// Whether a rectangle contains a point, half-open, with no area containing
/// nothing.
///
/// The rule `crate::the_panel_reveals` and `crate::which_preview_the_pointer_is_on`
/// both apply, and for their reason: two rectangles that share an edge must not
/// both contain the points along it. Here it also carries the rest case — a
/// concealed edge has no strip and no controls, and a `WindowEdge` of a window with
/// no width has a region of no width that contains nothing rather than
/// everything.
fn holds(rectangle: Rectangle<i32, Logical>, at: Point<i32, Logical>) -> bool {
    if rectangle.size.w <= 0 || rectangle.size.h <= 0 {
        return false;
    }
    let right = rectangle.loc.x.saturating_add(rectangle.size.w);
    let bottom = rectangle.loc.y.saturating_add(rectangle.size.h);
    at.x >= rectangle.loc.x && at.x < right && at.y >= rectangle.loc.y && at.y < bottom
}

impl crate::Server {
    /// Read the pointer once and tell every mapped window's edge about it.
    ///
    /// **Called once a frame, by the draw**, which is the only place that knows
    /// a frame is being built and the same cadence `crate::the_panel_reveals`
    /// answers at. A push from `crate::pointer::pointer_motion` would be the
    /// same information arriving at a worse moment: the edge is laid out from
    /// the reveal state, so sampling it where the layout happens keeps one
    /// answer per frame instead of a state that changes under a half-drawn one.
    ///
    /// **Each window is laid out from its own current answer, not from
    /// `true`.** That is the loop §5 describes: a concealed edge is all region,
    /// the region is what reveals it, and a revealed edge's controls and strip
    /// are what hold it. Laying every edge out as revealed in order to ask about
    /// it would mean a pointer in the strip's rows kept an edge that nothing had
    /// revealed.
    ///
    /// **Three of §5's four conditions are not fed here, and that is deliberate
    /// rather than missing.** Keyboard focus inside the edge, an open window
    /// menu and a drag in progress are all things §6's interactions *create* —
    /// there is no keyboard navigation into the edge, no menu and no drag to
    /// report yet. `Revealing` already carries all three
    /// (`the_keyboard`, `a_menu`, `a_drag`), tested in `alo-dock`, so §6 calls
    /// them and nothing here changes. Nothing unfed was added to this crate for
    /// them.
    pub fn the_edges_were_told_where_the_pointer_is(&mut self) {
        let Some(at) = self.where_the_pointer_is_in_logical_pixels() else {
            // **No pointer is not the origin**, for the reason the sibling
            // accessor gives: a machine whose seat has never had a pointing
            // device would otherwise reveal whichever window's edge happens to
            // contain `(0, 0)`.
            return;
        };
        let mapped: Vec<WlSurface> = self.mapped_surfaces().cloned().collect();
        let decorations = crate::window_edge_who_draws::who_draws_a_frame();

        // **Pruned against what is mapped**, so a window that closed takes its
        // reveal state with it. A `WlSurface` is a protocol object at an id the
        // server reuses; a stale entry would hand the next window at that id an
        // edge somebody else revealed.
        self.edges_revealed
            .retain(|(surface, _)| mapped.contains(surface));

        for surface in mapped {
            let was = self
                .edges_revealed
                .iter()
                .find(|(it, _)| *it == surface)
                .map_or_else(Revealing::covered, |(_, machine)| *machine);
            let edge = crate::window_edge::edge_of(
                crate::where_a_window_is(&surface),
                decorations,
                was.is_revealed(),
            );
            let now = was.the_pointer_is(what_the_pointer_is(&edge, at));
            match self
                .edges_revealed
                .iter_mut()
                .find(|(it, _)| *it == surface)
            {
                Some((_, machine)) => *machine = now,
                None => self.edges_revealed.push((surface, now)),
            }
        }
    }

    /// Whether this window's edge is revealed.
    ///
    /// **`false` for a window nothing has told about a pointer yet**, which is
    /// the same answer as concealed and is the honest one: an edge is covered
    /// until something reveals it, and `Revealing::covered` is that state.
    #[must_use]
    pub fn is_this_windows_edge_revealed(&self, surface: &WlSurface) -> bool {
        self.edges_revealed
            .iter()
            .find(|(it, _)| it == surface)
            .is_some_and(|(_, machine)| machine.is_revealed())
    }
}

#[cfg(test)]
#[path = "the_window_edge_reveals_tests.rs"]
mod tests;
