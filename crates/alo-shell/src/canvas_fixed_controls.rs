//! The bounds of the fixed controls, held where a drag can ask about them.
//!
//! **This is the piece whose absence meant *a frame is never lost* was never in
//! force.** `crate::canvas_never_lost` has held the rule since it was written —
//! a frame keeps a usable part of its name outside every fixed control — with
//! unit tests, integration tests, and **no caller on any path a person can
//! take.** `window_move.rs` and `canvas_space_drag.rs` mentioned it zero times.
//! A person could drag a frame under the Dock and the rule forbidding it was
//! never asked.
//!
//! The reason was not an oversight about wiring. It was that **the Dock's bounds
//! exist only inside the raster path, at draw time**, and the drag happens
//! between draws with no way to reach them. So the rule took the rectangle as an
//! argument, every caller that could supply one was a test, and the tests
//! invented it.
//!
//! # Why the shell is entitled to hold this
//!
//! The owner's status-area ruling of 2026-09-30 says it outright: the status area
//! is fixed at the top-right, **its contents may change, but the shell always
//! knows its current bounds.** A place on the server where the fixed controls'
//! current bounds live is that sentence and nothing more. It is not a new
//! promise to anybody — a frame that could already be dragged anywhere can now
//! be dragged everywhere except out of reach.
//!
//! # What is in the set tonight, and what joins it
//!
//! **The Dock only.** `crate::canvas_never_lost::enough_of_the_name_is_reachable`
//! already takes the controls as a **slice** rather than three arguments, with the
//! reason in its own header: *the promise is outside every fixed control, and a
//! fourth control added later must join it by construction.* So this grows by
//! pushing, not by changing a signature.
//!
//! The **status area** is not in it yet because it has no bounds to read:
//! `EgressStatusPicture` carries rows and solids and no rectangle, and where it
//! sits is a design decision the owner has scheduled rather than settled. The
//! **minimized-window panel** is not in it because its region belongs to another
//! lane. Both join here when they can answer where they are, and neither is
//! guessed at in the meantime.
//!
//! # The units, traced rather than argued
//!
//! **No conversion happens here, and that is deliberate.** Two lanes spent an
//! hour on 2026-09-30 diagnosing a unit mismatch that does not exist, because
//! these rectangles are typed `Physical` and the band they are compared against
//! is `Logical`. The values come from the same space. The trace:
//!
//! ```text
//! alo-displays/src/scale.rs:144   laid_out(pixels) = pixels * 100 / per_cent   — it divides
//! alo-shell/src/screens.rs:246    room: room_at(drawn_at, reported.pixels())
//! alo-shell/src/screens.rs:256    room_at -> Scale::laid_out                   — laid out from here
//! alo-shell/src/screens_raster.rs:100  dock_raster::picture(dock, look, place.room(), 0)
//! alo-shell/src/dock_raster.rs:77      pub(crate) band: Rectangle<i32, Physical>
//! ```
//!
//! `Scale::laid_out` is used in **exactly one** production place in this crate,
//! `screens.rs:256`, and everything downstream of it carries laid-out values. So
//! a 44-logical floor compared against these rectangles is already the same
//! apparent size at every scale, which is what logical units are for and the unit
//! WCAG specifies. **Adding the display's conversion here would divide a second
//! time and halve the protection on a dense screen.**
//!
//! **What this trace does not establish**, and the laptop lane was right to stop
//! it being claimed: that the `Physical` label is *wrong*. A compositor that
//! renders into a framebuffer of laid-out size and scales at scanout would make
//! that label correct for its own frame, with two notions of physical living one
//! layer apart. Nobody has established which this is. What is established is
//! narrower and sufficient: **every quantity in the comparison comes from one
//! space, so the arithmetic is consistent and no conversion belongs in it.**

use smithay::utils::{Physical, Rectangle};

/// Where the fixed controls are, as the last draw laid them out.
///
/// Empty until something has been drawn, which is the honest state: before the
/// first frame the shell does not know where the Dock is, and a drag then has
/// nothing to be protected from. **Empty means unknown, not *no controls*** —
/// and the two agree here only because a Dock nobody has drawn cannot cover
/// anything.
#[derive(Debug, Default)]
pub(crate) struct FixedControls {
    /// One rectangle per control, in the laid-out space this module's note
    /// describes.
    bounds: Vec<Rectangle<i32, Physical>>,
    /// How much of a name must stay clear, at the size this person draws text.
    ///
    /// **Held beside the bounds because the draw knows both and a drag knows
    /// neither.** `Server::a_usable_handle_at` needs an
    /// `alo_appearance::TextScale`, and nothing on the server carries one — the
    /// person's appearance reaches the shell through the draw. So the floor is
    /// worked out where the scale is known and kept here with the rectangles it
    /// will be compared against.
    ///
    /// [`None`] before the first draw, which is the same unknown as an empty set
    /// of bounds and is refused the same way: with nothing drawn there is nothing
    /// to be protected from.
    handle: Option<(f64, f64)>,
}

impl FixedControls {
    /// What a drag must keep a frame's name clear of.
    pub(crate) fn bounds(&self) -> &[Rectangle<i32, Physical>] {
        &self.bounds
    }

    /// How much of a name must stay clear, or [`None`] before the first draw.
    pub(crate) const fn handle(&self) -> Option<(f64, f64)> {
        self.handle
    }

    /// Replace the set with what this draw laid out.
    ///
    /// Replaced rather than merged: a control that stopped being drawn — a Dock
    /// that gave way to a window needing its room — must leave the set, or a
    /// frame would be held away from a rectangle nothing occupies.
    pub(crate) fn drawn(&mut self, bounds: Vec<Rectangle<i32, Physical>>, handle: (f64, f64)) {
        self.bounds = bounds;
        self.handle = Some(handle);
    }
}

impl crate::Server {
    /// Where the fixed controls are, for a drag to keep a name clear of.
    #[must_use]
    pub fn the_fixed_controls(&self) -> &[Rectangle<i32, Physical>] {
        self.fixed_controls.bounds()
    }

    /// Record where this draw put the fixed controls — **all of them, as a
    /// set.**
    ///
    /// Called from the draw, because the draw is the only place that knows.
    ///
    /// # A set, not a list to extend by hand
    ///
    /// The promise is *outside every fixed control*, and the owner's wording of
    /// 2026-09-30 is explicit that **the controls are a set rather than a list**:
    /// *a fourth fixed control added later must join it, because outside every
    /// fixed control is the promise and outside the three we thought of is
    /// not.*
    ///
    /// So this takes whatever the draw laid out, rather than one named control
    /// per argument. **It was `the_dock_was_drawn` and took the Dock's band
    /// alone until 2026-10-02**, which meant the rule was in force against one
    /// of the three: a frame could keep its name clear of the Dock and sit
    /// entirely under the put-aside panel, and nothing could tell, because the
    /// shell had never been given the panel's bounds to check against.
    ///
    /// A new control joins by **being drawn** and handed over here, which is
    /// the one place that cannot forget — a caller that adds a surface to the
    /// picture and not to this call has a compiler error rather than a silent
    /// hole, because the picture it passes is the thing being read.
    ///
    /// # What contributes, and what does not
    ///
    /// The Dock's **band** rather than the picture's presence:
    /// `crate::dock_room`'s note says the band is laid out whether or not the
    /// Dock is showing, and a Dock a person asked to give way is not covering
    /// anything — so a Dock that is not drawn contributes nothing.
    ///
    /// The panel's **reserved column** rather than its rail. They are not the
    /// same rectangle: the rail is what is drawn and the reserved column is the
    /// full-height area the panel owns, by the owner's ruling, and it is the
    /// reserved one a name has to stay clear of. **An empty panel has a rail of
    /// no height, and a rectangle with no height contributes nothing** — which
    /// is why the reserved column is taken unconditionally and the rail is not
    /// taken at all.
    pub fn the_fixed_controls_were_drawn(
        &mut self,
        drawn: FixedControlsDrawn,
        text: alo_appearance::TextScale,
    ) {
        let handle = Self::a_usable_handle_at(text);
        let bounds = [drawn.dock_band, Some(drawn.panel_reserved)]
            .into_iter()
            .flatten()
            .filter(|area| area.size.w > 0 && area.size.h > 0)
            .collect();
        self.fixed_controls.drawn(bounds, handle);
    }
}

/// Where this draw put each fixed control.
///
/// A value rather than one argument per control, because they are **one thing**:
/// what this frame put in front of the canvas. An argument list grows a fourth
/// entry when somebody remembers; a struct grows one when the compiler says so.
///
/// *The status area is absent, and that is a fact about the status area rather
/// than an omission here.* [ADR
/// 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
/// took it off the Dock, the owner fixed its position at the top-right of the
/// viewport on 2026-09-30, and **nothing draws it yet** —
/// `crate::desktop_raster`'s own note says the clock, battery, network and
/// volume have no location in the picture. A field holding a rectangle nobody
/// lays out would be a hole with a name on it. It joins when it is drawn.
#[derive(Debug, Clone, Copy)]
pub struct FixedControlsDrawn {
    /// The Dock's band, or [`None`] where the Dock gave way.
    pub dock_band: Option<Rectangle<i32, Physical>>,
    /// The full-height column the put-aside panel owns.
    ///
    /// Not the rail. An empty panel's is a rectangle of no width or no height,
    /// which is filtered out rather than special-cased — *a panel nobody has
    /// put a window into covers nothing* is the true answer and not a
    /// placeholder.
    pub panel_reserved: Rectangle<i32, Physical>,
}

impl crate::Server {
    /// Move the frame this drag holds, as far towards the pointer as it may go.
    ///
    /// **The caller `crate::canvas_never_lost` never had.** The rule it holds —
    /// *a frame keeps a usable part of its name outside every fixed control* — is
    /// a v0.01 promise that was not in force on any path a person could take
    /// until this function existed. Everything before it was a public API, unit
    /// tests, and integration tests calling it directly.
    ///
    /// Answers whether a drag handled this motion, which is what the pointer
    /// dispatch needs and all it needs.
    ///
    /// # The frame stops; it is never taken back
    ///
    /// `crate::window_move::where_this_drag_would_put_it` proposes and this
    /// commits, so a placement the rule refuses is **never written**. The frame
    /// keeps the last position that passed while the pointer carries on moving,
    /// which is the owner's ruling of 2026-09-30 word for word: *never accept an
    /// invalid placement and then pull the frame back.* A version that set the
    /// position and corrected it afterwards would draw one frame of the window
    /// somewhere it may not be, and that frame is the snap-back.
    ///
    /// # Errors
    /// [`InputError`](crate::InputError) where the pointer is out of the
    /// placement range, unchanged from before, or where refreshing input fails.
    pub(crate) fn a_drag_moves_the_frame(
        &mut self,
        location: smithay::utils::Point<f64, smithay::utils::Logical>,
    ) -> Result<bool, crate::InputError> {
        let Some(crate::window_move::Proposed { root, at: wanted }) =
            self.surfaces.where_this_drag_would_put_it(location)?
        else {
            return Ok(false);
        };
        // Where it is now, so a refusal can leave it exactly there. `At::checked`
        // refuses a point off the plane; a frame already off it is not one this
        // rule can reason about, so the drag is allowed and the plane's own
        // bounds in `crate::window_placement` remain the backstop.
        let origin = crate::window_buffer_origin(&root) + crate::scene::geometry_origin(&root);
        let (Some(from), Some(asked)) = (
            alo_canvas::At::checked(origin.x.floor() as i32, origin.y.floor() as i32),
            alo_canvas::At::checked(wanted.x, wanted.y),
        ) else {
            crate::window_placement::set(&root, Some(wanted));
            return Ok(true);
        };
        // Nothing drawn yet, so nothing known to be protected from.
        let Some(handle) = self.fixed_controls.handle() else {
            crate::window_placement::set(&root, Some(wanted));
            return Ok(true);
        };
        let controls: Vec<Rectangle<i32, Physical>> = self.the_fixed_controls().to_vec();
        let allowed = self.show_all_would_still_reach(&root, asked)
            && self.enough_of_the_name_is_reachable(&root, from, asked, &controls, handle);
        let may = if allowed { asked } else { from };
        crate::window_placement::set(&root, Some((may.x, may.y).into()));
        Ok(true)
    }
}
