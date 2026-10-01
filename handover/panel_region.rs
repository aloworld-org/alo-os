//! Where a pointer is, as far as the panel of put-aside windows is concerned.
//!
//! **This makes an existing promise true rather than adding one.**
//! `alo_put_aside::the_region_the_panel_claims` states what *in the strip*, *on
//! the panel* and *elsewhere* mean for this surface and deliberately holds no
//! coordinate; `alo_dock::Revealing` consumes those three and deliberately holds
//! no geometry. Neither can turn a pointer position into an answer, and nothing
//! else did. This is the caller both of them were written for.
//!
//! # Whose area first, then which part
//!
//! Two questions, asked in that order, because they are different questions.
//!
//! [`crate::surface_areas::whose_area`] answers *which surface may claim this
//! point at all* and returns **one** value, so the owner's rule — *one pointer
//! position cannot reveal two surfaces* — is unrepresentable rather than merely
//! forbidden. Only once the answer is the panel does this file ask *which part
//! of the panel*, and a point the arbitration gave to the Dock is
//! [`WhereThePointerIs::Elsewhere`] here however plainly it lies inside a
//! rectangle the panel drew.
//!
//! **That ordering is the whole design.** A classifier that asked *is it on the
//! panel* first and arbitrated afterwards would decide the shared corner by the
//! order its branches were written, which is what the panel's own contract
//! refuses to commit and what the owner settled by ruling rather than by
//! letting geometry fall out.
//!
//! # The strip reveals; the parts keep
//!
//! `WhereThePointerIs` distinguishes them and the machine needs the
//! distinction: the strip **reveals** a concealed panel, the panel's own area
//! only **keeps** one already revealed. So a pointer resting where the panel
//! *would be* if it were showing must not reveal it, and the parts are only
//! asked about when there is something drawn to be on.
//!
//! # Told, never fetched
//!
//! Every rectangle is handed in, as in `crate::dock_room` and
//! `crate::surface_areas`. This file holds no measurement, no edge and no screen
//! size — the caller that knows the display computes them, which is the same
//! seam the panel's contract keeps by holding no number at all.

use alo_put_aside::the_region_the_panel_claims::{PartOfThePanel, WhereThePointerIs};
use smithay::utils::{Physical, Point, Rectangle};

use crate::surface_areas::{TheSurfaces, WhoseArea, whose_area};

/// The panel as it is on one display now, for classifying a pointer against.
///
/// `parts` is in painting order and **must not overlap**: two parts containing
/// one point is a fault in whoever laid them out, not a case to resolve here,
/// and resolving it by order is the thing this module refuses to do between
/// surfaces. It is held by a test rather than by this sentence.
#[derive(Debug, Clone)]
pub(crate) struct ThePanelOnScreen<'a> {
    /// The strip along the panel's edge that asks for it.
    pub(crate) strip: Rectangle<i32, Physical>,
    /// The panel's own drawn parts, each with what it is.
    ///
    /// Empty when the panel is concealed: there is nothing to be on.
    pub(crate) parts: &'a [(Rectangle<i32, Physical>, PartOfThePanel)],
}

/// Where this pointer is, as far as the panel is concerned.
///
/// Arbitration first: a point that is not the panel's area is `Elsewhere` here,
/// whatever the panel has drawn there.
pub(crate) fn where_the_pointer_is(
    at: Point<i32, Physical>,
    panel: &ThePanelOnScreen<'_>,
    surfaces: &TheSurfaces,
) -> WhereThePointerIs {
    if whose_area(at, surfaces) != WhoseArea::ThePanel {
        return WhereThePointerIs::Elsewhere;
    }
    // **Parts before the strip.** They overlap by construction — the panel is
    // drawn inside the column whose edge the strip runs along — and *on the
    // panel* is the more specific answer. The other order would report a
    // pointer resting on a preview as merely asking, and the machine would
    // then let the panel conceal under it.
    for (area, part) in panel.parts {
        if holds(*area, at) {
            return WhereThePointerIs::OnThePanel(*part);
        }
    }
    if holds(panel.strip, at) {
        return WhereThePointerIs::InTheStrip;
    }
    WhereThePointerIs::Elsewhere
}

/// Whether a rectangle contains a point, half-open, as `crate::surface_areas`
/// does — so two rectangles sharing an edge do not both contain the points
/// along it, and a rectangle with no area contains nothing.
fn holds(rectangle: Rectangle<i32, Physical>, at: Point<i32, Physical>) -> bool {
    if rectangle.size.w <= 0 || rectangle.size.h <= 0 {
        return false;
    }
    let right = rectangle.loc.x.saturating_add(rectangle.size.w);
    let bottom = rectangle.loc.y.saturating_add(rectangle.size.h);
    at.x >= rectangle.loc.x && at.x < right && at.y >= rectangle.loc.y && at.y < bottom
}
