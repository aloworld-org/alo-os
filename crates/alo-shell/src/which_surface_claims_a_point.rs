//! Which fixed surface claims a point, answered from the bounds the draw recorded.
//!
//! **This gives `alo_put_aside::the_region_the_panel_claims` its first caller.** The
//! panel lane wrote `WhatEachSurfaceSaid` and `at_most_one_surface_claims_it` — a
//! statement that no point belongs to two surfaces — and nothing outside that crate
//! consumed either, so the invariant held in its own tests and was never asked about
//! a real screen. That is the fault this module's neighbours exist because of:
//! `canvas_never_lost`'s rule, the recheck and its mover, and the panel's own
//! bring-back were each correct, tested and unreachable.
//!
//! # Why the labels had to be kept
//!
//! `crate::canvas_fixed_controls` flattens the controls into one slice, correctly:
//! the rule is *outside **every** fixed control*, so which rectangle a frame is clear
//! of does not matter to it. A classifier is the opposite question — *which surface
//! claims this point* — and a flat set cannot answer it, because flattening is
//! exactly what discards the labels. So `FixedControls` now keeps what the draw
//! handed over beside the set it folds, and this reads the labelled one.
//!
//! # The top controls are the one surface that cannot answer yet
//!
//! `WhatEachSurfaceSaid` names three: the top controls, the Dock, the panel. **Two of
//! them exist.** The top controls are canvas task 6a and nothing draws them, so this
//! answers `false` for them and says so rather than guessing — a band for a surface
//! nobody paints is the hole with a name on it that `FixedControlsDrawn`'s own note
//! refuses.
//!
//! When they land, they join by being drawn and handed over like the status area was,
//! and this file changes one line. What it must not do in the meantime is report
//! *not claimed* as though it were measured: a reader seeing `the_top_controls:
//! false` at a point inside the band's designed region would be reading an absence,
//! not an answer, so this returns the flag with that caveat recorded here rather than
//! in a comment at the call site.
//!
//! # What this is not
//!
//! Not a resolution. The panel lane's note is explicit that a version answering
//! *which surface wins* would be the priority order that file refuses to contain, and
//! that the owner already settled priority elsewhere. This reports what each surface
//! says about one point, and the invariant checks the answers do not conflict.

use smithay::utils::{Physical, Point, Rectangle};

impl crate::Server {
    /// What each fixed surface says about this point, or [`None`] before a draw.
    ///
    /// [`None`] is *nothing has been drawn*, which is a real state at session start:
    /// a window maps before the first frame, so a pointer can arrive before any
    /// surface knows where it is. Answering *nobody claims it* then would be a
    /// measurement nobody took.
    #[must_use]
    pub fn what_each_surface_said_at(
        &self,
        at: Point<i32, Physical>,
    ) -> Option<alo_put_aside::the_region_the_panel_claims::WhatEachSurfaceSaid> {
        let labelled = self.fixed_controls.labelled()?;
        Some(
            alo_put_aside::the_region_the_panel_claims::WhatEachSurfaceSaid {
                // **No longer an absence.** This read `false` with a note saying so
                // until the band was reserved: now the draw records where the top
                // controls are, so the answer is measured like the other two. A
                // hardcoded `false` left beside two real answers is the kind of
                // third value that reads as agreement.
                the_top_controls: labelled.top_controls.is_some_and(|band| holds(band, at)),
                the_dock: labelled.dock_band.is_some_and(|band| holds(band, at)),
                // Taken unconditionally because an empty panel's reserved column is
                // a rectangle of no extent, which holds no point — the same reason
                // `the_fixed_controls_were_drawn` takes it without asking whether
                // the panel is showing.
                the_panel: holds(labelled.panel_reserved, at),
            },
        )
    }

    /// Whether the controls the last draw laid out leave every point to one surface.
    ///
    /// **Asked of the rectangles themselves rather than of a sampled grid.** Two
    /// rectangles conflict exactly when they intersect, so an overlap test answers
    /// for every point at once and a sweep of sampled points would answer for the
    /// points somebody thought of. The panel lane's invariant is per-point by
    /// design, because a classifier hands it one point at a time; this is the same
    /// statement made about the whole screen, and it is the form a draw can check.
    #[must_use]
    pub fn at_most_one_surface_claims_any_point(&self) -> Option<bool> {
        let labelled = self.fixed_controls.labelled()?;
        let dock = labelled.dock_band;
        let panel = labelled.panel_reserved;
        let top = labelled.top_controls;
        // Every pair, rather than the two that happened to exist when this was
        // written. An empty rectangle intersects nothing, so a panel holding no
        // windows, a Dock that was not drawn and a band that gave way to a
        // full-screen window all drop out without a special case.
        //
        // **Three surfaces means three pairs, and naming them is what stops a fourth
        // joining unnoticed.** The top controls were added to the set in #442 and this
        // check compared two of them until then — which would have passed while the
        // new band overlapped either one.
        let pairs = [(dock, Some(panel)), (dock, top), (top, Some(panel))];
        Some(pairs.into_iter().all(|(one, other)| match (one, other) {
            (Some(one), Some(other)) => !overlap(one, other),
            _ => true,
        }))
    }
}

/// Whether this rectangle holds this point.
///
/// Written out rather than taken from smithay's `contains`, because the edge rule is
/// the subject: a rectangle's far edge is **not** inside it, so two rectangles that
/// meet along a line do not both claim the line. `contains` is inclusive on both
/// edges and would make every shared edge a conflict.
fn holds(area: Rectangle<i32, Physical>, at: Point<i32, Physical>) -> bool {
    at.x >= area.loc.x
        && at.y >= area.loc.y
        && at.x < area.loc.x.saturating_add(area.size.w)
        && at.y < area.loc.y.saturating_add(area.size.h)
}

/// Whether two rectangles share any point, on the same half-open edge rule.
fn overlap(one: Rectangle<i32, Physical>, other: Rectangle<i32, Physical>) -> bool {
    let apart = |a: Rectangle<i32, Physical>, b: Rectangle<i32, Physical>| {
        a.loc.x.saturating_add(a.size.w) <= b.loc.x || a.loc.y.saturating_add(a.size.h) <= b.loc.y
    };
    one.size.w > 0
        && one.size.h > 0
        && other.size.w > 0
        && other.size.h > 0
        && !apart(one, other)
        && !apart(other, one)
}
