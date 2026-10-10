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

use crate::window_edge::{OnTheEdge, WindowEdge};

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
    match where_on_the_edge(edge, at) {
        WhereOnTheEdge::Away => ThePointer::Elsewhere,
        // The approach: in the region, on nothing in particular. This is the
        // answer that reveals, and at rest it is the only one available.
        WhereOnTheEdge::TheRegion => ThePointer::AtTheEdge,
        // **A concealed edge's drag band is still the approach**, and this arm
        // is the one the restructure was wrong about until a test caught it.
        // At rest `edge_of` gives the drag region a shallow band at `y = 28`
        // spanning the edge — where the grip is drawn and where a press at rest
        // lands — and that band is *inside* the interaction region. Reading it
        // as *on the surface* made a pointer arriving directly on it set
        // `on_the_surface` from `Revealing`'s own current answer, which on a
        // concealed edge is `false`: the edge refused to reveal from the one
        // band a person aims at. A concealed edge draws no strip, so
        // `strip.is_none()` is exactly *not revealed* and needs no argument of
        // its own.
        WhereOnTheEdge::TheDrag if edge.strip.is_none() => ThePointer::AtTheEdge,
        // On a revealed edge's furniture: the strip, the part of it that drags,
        // or a control. `Revealing` reads *on the surface* as *keep it*, which
        // each of these is — including the path between them, which that enum's
        // own note says the caller must classify as one continuous region.
        WhereOnTheEdge::TheStrip | WhereOnTheEdge::TheDrag | WhereOnTheEdge::AControl(_) => {
            ThePointer::OnTheSurface
        }
    }
}

/// Where on one window's edge a point is, in as much detail as anything needs.
///
/// **One calculation with two readers.** [`crate::window_edge`]'s header sets
/// that rule for the geometry — *a second calculation anywhere* is a second
/// answer — and it applies as much to asking about it. §5 needs only *is this
/// the edge*; §6 needs *which control*. So the reveal and the press read one
/// function rather than each hit-testing the same rectangles and drifting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhereOnTheEdge {
    /// Not on this window's edge at all.
    Away,
    /// Inside the interaction region, on nothing more specific. The approach.
    TheRegion,
    /// On the strip that is drawn, away from the part that drags.
    TheStrip,
    /// On the movement region, where a press drags the window.
    TheDrag,
    /// On one control's target, which is what a press there reaches.
    AControl(OnTheEdge),
}

/// Where a point is on this edge, most specific answer first.
///
/// **The order is the correctness.** These rectangles nest: a control's target
/// and the drag region both lie inside the region, and the revealed drag takes
/// the region's whole height. So the most specific has to win, or a press on
/// Close would read as a drag and a pointer on a control would read as merely
/// approaching.
///
/// Controls before the drag is belt and braces rather than load-bearing:
/// `edge_of` ends the revealed drag region one gap **before** the first
/// control, so the two do not overlap today. Asking in this order means they
/// could without this answering wrongly — and it is the clause *buttons and
/// menus do not start a drag* held by the shape rather than by a caller
/// remembering.
pub(crate) fn where_on_the_edge(edge: &WindowEdge, at: Point<i32, Logical>) -> WhereOnTheEdge {
    // **`target`, never `highlight`.** The 44 × 44 is what answers; the 32 × 28
    // is only what is filled behind it while hovered. Hit-testing the highlight
    // would shrink every control by six pixels each side and fourteen at the
    // top, which is the specification's one hard floor — *44 × 44,
    // non-overlapping* — quietly undone by reading the wrong field of the right
    // struct. The compiler cannot tell these apart: both are
    // `Rectangle<i32, Logical>` on the same value.
    if let Some(control) = edge
        .controls
        .iter()
        .find(|control| holds(control.target, at))
    {
        return WhereOnTheEdge::AControl(control.does);
    }
    if holds(edge.drag, at) {
        return WhereOnTheEdge::TheDrag;
    }
    if edge.strip.is_some_and(|strip| holds(strip, at)) {
        return WhereOnTheEdge::TheStrip;
    }
    if holds(edge.region, at) {
        return WhereOnTheEdge::TheRegion;
    }
    WhereOnTheEdge::Away
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

/// What a press on one window's edge does.
///
/// §6 of `docs/design/the-external-window-edge.md`. Read off the same
/// classification the reveal reads, so a control that highlights under the
/// pointer is the control a press there reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhatAPressDoes {
    /// Nothing. The press is not this edge's, or it landed on edge that has no
    /// action — the rows above the strip, or the sliver of strip past the last
    /// control.
    Nothing,
    /// Moves the window, which is what the movement region is for.
    MovesTheWindow,
    /// One of the edge's own actions, through the mechanism that already does
    /// it.
    This(OnTheEdge),
}

/// What a press at this point on this edge does.
///
/// **The clause this function exists to make unbreakable is *buttons and menus
/// do not start a drag*.** It is not enforced by remembering to check the
/// controls first; it is enforced by there being one classification, whose most
/// specific answer is the control. A press cannot be both, because
/// [`where_on_the_edge`] returns one value.
///
/// **A concealed edge is still movable**, which is the clause *borderless
/// applications stay movable and recoverable*: at rest the drag region is the
/// shallow band the grip is drawn in, and a press there moves the window
/// without the edge having to be revealed first. Nothing here asks whether it
/// is revealed.
pub(crate) fn what_a_press_does(edge: &WindowEdge, at: Point<i32, Logical>) -> WhatAPressDoes {
    match where_on_the_edge(edge, at) {
        WhereOnTheEdge::AControl(does) => WhatAPressDoes::This(does),
        WhereOnTheEdge::TheDrag => WhatAPressDoes::MovesTheWindow,
        // **Not a drag.** The movement region is `edge.drag` and nothing else:
        // the rows above the strip are the approach, and the strip past the
        // last control is the trailing margin. A press on either is the
        // window's own business, not the edge's.
        WhereOnTheEdge::TheStrip | WhereOnTheEdge::TheRegion | WhereOnTheEdge::Away => {
            WhatAPressDoes::Nothing
        }
    }
}

#[cfg(test)]
#[path = "the_window_edge_reveals_tests.rs"]
mod tests;
