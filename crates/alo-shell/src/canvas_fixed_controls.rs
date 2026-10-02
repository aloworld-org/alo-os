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

    /// The handle floor the last draw was measured with, or [`None`] where
    /// nothing has been drawn.
    ///
    /// **Already converted into the controls' own pixels** by
    /// [`Self::the_fixed_controls_were_drawn`], so a caller must not scale it
    /// again — and must not reach for
    /// [`A_USABLE_HANDLE`](crate::A_USABLE_HANDLE) instead, which is the logical
    /// figure and would be too small by this display's scale.
    ///
    /// [`None`] means *nothing has been drawn*, which is a real state at
    /// session start because a window maps before the first frame. It does not
    /// mean *no protection wanted*, and a caller that treats it as the second
    /// owes a recheck once a frame exists.
    #[must_use]
    pub fn the_handle_the_controls_were_drawn_with(&self) -> Option<(f64, f64)> {
        self.fixed_controls.handle()
    }

    /// Every frame whose name the controls, **as they are now**, leave
    /// unreachable.
    ///
    /// # Nothing asked this until 2026-10-02, and that is the gap rather than
    /// the feature
    ///
    /// The rule is in force on the two roads a frame *moves* by: a drag, and a
    /// recovery putting a window back where it was left. Both ask before
    /// placing. **Neither asks again afterwards**, and the controls move
    /// underneath a frame that is not moving at all:
    ///
    /// - the Dock gives way to a window needing its room, and comes back;
    /// - a window is put aside, so the panel's reserved column appears where
    ///   there was none;
    /// - the display changes, or its scale does, and every rectangle is
    ///   somewhere else;
    /// - the person moves the Dock to another edge.
    ///
    /// After any of those, a frame that was reachable is not, and **nothing
    /// notices**. Task 8's third acceptance clause is exactly this:
    /// *reachability is rechecked when those bounds change.*
    ///
    /// # This answers, and does not move anything
    ///
    /// The clause's other half — *where recovery needs a frame moved, the move
    /// is shown and its previous position recorded* — is **not** here, and the
    /// split is deliberate rather than partial delivery. A frame moved by the
    /// machine is a person's arrangement edited without them, so it owes a
    /// showing and a record; answering *which frames* owes neither, and is what
    /// a mover would have to ask first.
    ///
    /// **Making the loss detectable comes before making it recoverable**,
    /// because a silent loss cannot be tested for and this repository has spent
    /// two days on checks that could not fail.
    ///
    /// Each answer is the frame's id and where it is, so a caller has what it
    /// needs to record a previous position without asking a second time.
    #[must_use]
    pub fn frames_the_controls_now_hide(&self) -> Vec<(u64, alo_canvas::At)> {
        // Nothing drawn is not everything hidden. Before the first frame there
        // are no bounds, and answering *all of them* would be the most alarming
        // possible way to say *I do not know yet*.
        let Some(handle) = self.fixed_controls.handle() else {
            return Vec::new();
        };
        let controls: Vec<Rectangle<i32, Physical>> = self.the_fixed_controls().to_vec();
        if controls.is_empty() {
            return Vec::new();
        }
        self.mapped_surfaces()
            .filter_map(|surface| {
                let at = crate::canvas_place::the_place_of(surface)
                    .and(Some(()))
                    .and_then(|()| {
                        let origin = crate::window_buffer_origin(surface)
                            + crate::scene::geometry_origin(surface);
                        alo_canvas::At::checked(origin.x.floor() as i32, origin.y.floor() as i32)
                    })?;
                // Asked of where it **is**, so `from` and `wanted` are the same
                // point: this is not a proposed move, it is the position the
                // frame already holds being re-examined against controls that
                // have moved under it.
                (!self.enough_of_the_name_is_reachable(surface, at, at, &controls, handle))
                    .then(|| (crate::window_number::Numbers::of(surface), at))
            })
            .collect()
    }

    /// Bring back every frame the controls now hide, and say what was done.
    ///
    /// # A frame moved by the machine is a person's arrangement edited without them
    ///
    /// So this reports what it did rather than doing it quietly, and
    /// [`Recovery::BroughtBack`] carries **where the frame was** as well as where
    /// it now is. That is the owner's clause: *where recovery needs a frame
    /// moved, the move is shown and its previous position recorded.* The record
    /// is here. **The showing is owed by whatever draws it** — this crate can
    /// move a window and cannot tell a person anything.
    ///
    /// # The rule is the oracle, and nothing here computes geometry
    ///
    /// [`Self::enough_of_the_name_is_reachable`] already answers *could this
    /// frame be at that point*, for any point, because it translates the name
    /// band by the difference it is asked about. So a place to move to is
    /// **searched for by asking it**, never worked out from the controls' edges.
    ///
    /// That matters beyond tidiness. A closed form would have to assume what a
    /// control looks like — along an edge, rectangular, one at a time — and the
    /// promise is *outside **every** fixed control*, whatever they turn out to
    /// be. A fourth control of an awkward shape would silently break arithmetic
    /// and cannot break a search.
    ///
    /// # Nearest first, up and left before down and right
    ///
    /// Candidates are tried in increasing distance and the first the rule
    /// accepts wins. **Up and left first is a choice rather than an accident:**
    /// the Dock lies along the bottom and the panel down a side, so away from
    /// them is where the room is, and a frame pushed further into a control in
    /// order to escape it would be a strange thing to watch happen.
    ///
    /// A frame no candidate rescues is [`Recovery::CouldNotBeBroughtBack`] and
    /// is **left exactly where it is**. Moving it somewhere arbitrary because
    /// the search ran out would be worse than leaving it: where the person put
    /// it is at least a place they know.
    #[must_use = "a recovery nobody shows is a frame that moved silently"]
    pub fn bring_back_frames_the_controls_hide(&mut self) -> Vec<Recovery> {
        // Planned while nothing is borrowed mutably, then applied. The rule
        // needs `&self` and placing needs `&mut self`, and interleaving them
        // would be asking the rule about a tree being changed underneath it.
        let plans: Vec<Recovery> = self
            .frames_the_controls_now_hide()
            .into_iter()
            .map(|(id, was)| {
                self.somewhere_this_frame_can_be_reached(id, was)
                    .map_or(Recovery::CouldNotBeBroughtBack { id, at: was }, |now| {
                        Recovery::BroughtBack { id, was, now }
                    })
            })
            .collect();

        for plan in &plans {
            if let Recovery::BroughtBack { id, now, .. } = plan {
                let surface = self
                    .mapped_surfaces()
                    .find(|surface| crate::window_number::Numbers::of(surface) == *id)
                    .cloned();
                if let Some(surface) = surface {
                    let _ = self.place_window(&surface, (now.x, now.y));
                }
            }
        }
        plans
    }

    /// The nearest point the rule accepts for this frame, or [`None`].
    fn somewhere_this_frame_can_be_reached(
        &self,
        id: u64,
        was: alo_canvas::At,
    ) -> Option<alo_canvas::At> {
        let handle = self.fixed_controls.handle()?;
        let controls: Vec<Rectangle<i32, Physical>> = self.the_fixed_controls().to_vec();
        let surface = self
            .mapped_surfaces()
            .find(|surface| crate::window_number::Numbers::of(surface) == id)?
            .clone();

        // Eight units a step, out to a thousand. The step is smaller than a name
        // band, so no reachable gap is stepped over; the bound is there because
        // **a search that cannot fail is a hang** — controls covering everything
        // is a real state, and the answer to it is *could not*, not a loop.
        (1..=125).find_map(|step| {
            let away = step * 8;
            [(0, -away), (-away, 0), (0, away), (away, 0)]
                .into_iter()
                .find_map(|(dx, dy)| {
                    let candidate = alo_canvas::At::checked(
                        was.x.saturating_add(dx),
                        was.y.saturating_add(dy),
                    )?;
                    self.enough_of_the_name_is_reachable(
                        &surface, was, candidate, &controls, handle,
                    )
                    .then_some(candidate)
                })
        })
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
    /// # The handle floor is logical and these rectangles are not
    ///
    /// [`A_USABLE_HANDLE`](crate::A_USABLE_HANDLE) is **44 × 24 logical
    /// pixels**, by the owner's ruling, and `a_usable_handle_at` scales it by
    /// the person's text size and nothing else. The rectangles arriving here are
    /// laid out from `target.size()` — the framebuffer — so they are in **the
    /// display's own pixels.**
    ///
    /// Comparing the two without converting makes the protected area **too small
    /// by the display's scale**: on a screen drawing two pixels per logical one,
    /// a floor of 44 is 44 framebuffer pixels where the promise is 88. A person
    /// on a dense display was promised a handle and given half of one, and every
    /// test passed because **every test ran at one to one.**
    ///
    /// So the conversion happens here, **once**, which is the owner's words
    /// exactly: *preserve at least 44 × 24 logical pixels of unobstructed drag
    /// area; convert to physical coordinates once at the established rendering
    /// boundary.* This is that boundary — the one place that holds both the
    /// logical floor and the rectangles it is compared against.
    ///
    /// The two scales are **not** the same thing and both apply: the text scale
    /// is the person asking for larger targets, and the display scale is how
    /// many pixels this screen draws for one logical unit. A fixed 44 × 24 would
    /// shrink against everything around it for the person who most needs it not
    /// to; an unconverted one shrinks against the screen.
    pub fn the_fixed_controls_were_drawn(
        &mut self,
        drawn: FixedControlsDrawn,
        text: alo_appearance::TextScale,
    ) {
        // **The handle is not converted, and the display's scale is not read
        // here.** The owner's ruling of 2026-10-01 says the floor is 44 × 24
        // *logical*, scaled by the person's text size and nothing else, and
        // explicitly asks that any implication of a second conversion be
        // removed. Between 2026-10-02 and this change it was converted anyway,
        // by me, in the two commits that followed that ruling.
        //
        // What makes it wrong is measurable rather than a matter of reading:
        // `desktop_raster_tests::the_dock_band_and_the_panel_column_do_not_move_with_the_displays_scale`
        // draws the same display at 100, 125, 150 and 200 per cent and finds
        // **both rectangles identical**. They are laid out from the room, which
        // arrives already divided by the scale, and from measures scaled by text
        // size alone — so they are logical, and a handle multiplied by the scale
        // protected 88 × 48 at 200 per cent where the promise is 44 × 24.
        //
        // The `Physical` marker on them proves nothing: it is satisfied by
        // construction, which is the trap this module's header was written about
        // after two lanes lost an hour to it. **It cost a third hour here, and
        // the test that was supposed to protect the conversion asserted it
        // instead** — a denser display restricting the drag further reads as the
        // floor arriving, and is equally the floor being too large.
        let handle = Self::a_usable_handle_at(text);
        let bounds = [drawn.dock_band, Some(drawn.panel_reserved)]
            .into_iter()
            .flatten()
            .filter(|area| area.size.w > 0 && area.size.h > 0)
            .collect();
        self.fixed_controls.drawn(bounds, handle);
    }
}

/// What was done about one frame the controls had hidden.
///
/// **Returned rather than logged**, because the owner's clause is that a move is
/// *shown* and its previous position *recorded*. A surface cannot show what it
/// is not told, and `was` is what makes the move undoable rather than merely
/// visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recovery {
    /// It was hidden, and there was somewhere to put it.
    BroughtBack {
        /// Which frame, as the compositor numbers its windows.
        id: u64,
        /// Where the person had left it.
        was: alo_canvas::At,
        /// Where it is now.
        now: alo_canvas::At,
    },
    /// It was hidden and nothing the search tried was reachable, so it has
    /// **not** been moved.
    CouldNotBeBroughtBack {
        /// Which frame.
        id: u64,
        /// Where it still is.
        at: alo_canvas::At,
    },
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
        // **In the World a drag writes no placement.** It is choosing a Place,
        // not a point, and the World's coordinates are the tiles' rather than any
        // frame's — writing one into the other would move a window by the
        // distance between two unrelated planes. `crate::canvas_dragged_into_a_place`
        // carries the reasoning; the drop happens on release.
        if self.a_drag_is_choosing_a_place() {
            return Ok(true);
        }
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
