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
    /// **The output a narrow window's edge is clamped inside.**
    ///
    /// The same extent the draw uses, so the edge a pointer is classified
    /// against is the edge that was painted — two different clamps would mean
    /// a press landing where no control was drawn.
    ///
    /// No extent yet means nothing has drawn, and an edge clamped into a zero
    /// output would collapse to `x = 0`. So the window's own width is used as
    /// the room, which is what the pre-compact layout did for every window and
    /// leaves the overhang unclamped until the first frame.
    ///
    /// `pub(crate)` because `crate::the_edge_takes_the_keyboard` needs the
    /// same answer, and two callers of one calculation is the arrangement this
    /// file's own comment asks for.
    pub(crate) fn the_room_an_edge_is_clamped_in(&self) -> smithay::utils::Size<i32, Logical> {
        self.surfaces.popups.output_size.map_or_else(
            || smithay::utils::Size::from((i32::MAX, i32::MAX)),
            // **At the scale of the display it is the size of**, so the room is
            // in the edge's own units (`crate::an_edge_in_its_displays_units`).
            |pixels| {
                crate::window_edge::the_usable_output(
                    pixels,
                    self.the_scale_an_edge_is_measured_at(),
                )
            },
        )
    }

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
    /// **Three of §5's four conditions are fed here, and the fourth is not.**
    /// This paragraph said that three were *not* fed, which was true while
    /// nothing could focus a control or open a menu. §6 built both, so two of
    /// them arrive from `crate::the_edge_takes_the_keyboard`, which holds one
    /// keyboard state per window beside `edges_revealed` and is pruned on the
    /// same walk:
    ///
    /// | condition | fed from |
    /// |---|---|
    /// | the pointer | this file, once a frame |
    /// | keyboard focus | `TheKeyboardOnTheEdge::whether_the_keyboard_holds_it` |
    /// | an open menu | `TheKeyboardOnTheEdge::whether_a_menu_holds_it` |
    /// | a drag | **nothing** — no drag of a window begins from the edge yet |
    ///
    /// The fourth is left unfed rather than reported as `IsNot` by something
    /// that cannot know: `Revealing::a_drag` defaults to not dragging, so an
    /// absent reporter and a reporter saying no are the same answer today, and
    /// only one of them is a claim.
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
        let room = self.the_room_an_edge_is_clamped_in();
        // **The pointer and each window in the edge's units**, at the display's
        // scale, so what reveals an edge is where the edge is drawn.
        let scale = self.the_scale_an_edge_is_measured_at();
        let at = crate::an_edge_in_its_displays_units::a_point_in_the_edges_units(at, scale);

        // **Pruned against what is mapped**, so a window that closed takes its
        // reveal state with it. A `WlSurface` is a protocol object at an id the
        // server reuses; a stale entry would hand the next window at that id an
        // edge somebody else revealed.
        self.edges_revealed
            .retain(|(surface, _)| mapped.contains(surface));
        // And the keyboard's own state, on the same walk and for the same
        // reason: an id the server reuses must not hand the next window at it
        // a focus ring somebody else put there.
        self.only_the_edges_still_open(&mapped);

        for surface in mapped {
            let was = self
                .edges_revealed
                .iter()
                .find(|(it, _)| *it == surface)
                .map_or_else(Revealing::covered, |(_, machine)| *machine);
            let edge = crate::window_edge::edge_of(
                crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(
                    crate::where_a_window_is(&surface),
                    scale,
                ),
                decorations,
                // **Laid out as it is drawn**: shown, where titles are always
                // shown, so a pointer arriving on Close is on Close.
                was.is_revealed() || self.window_titles.are_always_shown(),
                room,
            );
            // **Two of §5's four conditions, fed from §6.** Read before the
            // mutable borrow below, which is why it is a local rather than a
            // call inside the expression.
            //
            // The keyboard and the menu are asked of
            // `crate::the_edge_takes_the_keyboard`, which holds one state per
            // window beside this one and is pruned on the same walk.
            let keyboard = self.where_the_keyboard_is_on_this_windows_edge(&surface);
            let now = was
                .the_pointer_is(what_the_pointer_is(&edge, at))
                .the_keyboard(keyboard.whether_the_keyboard_holds_it())
                .a_menu(keyboard.whether_a_menu_holds_it());
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
    ///
    /// **Always, while the person's titles are always shown** — the owner's
    /// answer of 2026-11-01, and what this release ships. The machine below
    /// still runs either way, so a person who chooses *show the title only
    /// when I point at it* gets the edge §5 describes from the next frame.
    #[must_use]
    pub fn is_this_windows_edge_revealed(&self, surface: &WlSurface) -> bool {
        self.window_titles.are_always_shown()
            || self
                .edges_revealed
                .iter()
                .find(|(it, _)| it == surface)
                .is_some_and(|(_, machine)| machine.is_revealed())
    }
}

#[cfg(test)]
#[path = "the_window_edge_reveals_tests.rs"]
mod tests;
