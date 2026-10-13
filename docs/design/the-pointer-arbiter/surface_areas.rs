//! Which surface a point on the screen belongs to, when more than one could
//! reach it.
//!
//! Three surfaces reveal from three edges — the top controls, the Dock along
//! the bottom, and the panel of put-aside windows at its own edge. The design
//! draws their regions overlapping: the top controls span the full width and
//! the panel's region runs the full height, so both cover the corner between
//! them.
//!
//! **The owner settled it on 2026-09-30:**
//!
//! > Make the top controls region stop before the right panel. Top controls
//! > span the screen up to the reserved right-panel area. Right panel: owns
//! > that area, including the top-right corner. Bottom Dock: stops before the
//! > same area. The reserved width follows the panel's current expanded or
//! > collapsed width. Activation strips and pointer paths must follow those
//! > bounds too, **so one pointer position cannot reveal two surfaces.**
//!
//! # That last clause is why this is one function and not three predicates
//!
//! *One pointer position cannot reveal two surfaces* can be held two ways. It
//! can be asserted over a grid of points against three independent predicates —
//! which tests it where the grid happens to land, at whichever panel width the
//! test chose. Or the question can be asked once and answered with **one
//! value**, which makes two surfaces claiming a point not a bug to be caught
//! but a sentence that cannot be written.
//!
//! This is the second. [`whose_area`] returns one [`WhoseArea`], so there is no
//! arrangement of rectangles, no panel width and no screen size at which two
//! surfaces both claim a point. The rule is not tested here because there is
//! nothing to test: it is the return type.
//!
//! **The order of the branches is the correctness, and this paragraph said the
//! opposite until 2026-10-01.** It read: *the panel's reserved area is
//! subtracted from the other two rather than merely tried first, so moving the
//! arms of the match cannot change an answer.* There is no subtraction in this
//! file. [`whose_area`] is three early returns with the panel asked **first**,
//! and that order decides every contested point — measured by the lane that
//! read this header against the code below it: **378 points of the grid fixture
//! change answer if the arms are reordered**, the first at reserved width 112,
//! point (1328, 848), `ThePanel` as written and `TheDock` swapped.
//!
//! The sentence was worse than merely untrue. A maintainer who reorders these
//! branches for readability had been **told by the file that it was safe**. The
//! grid test stops them, so the cost is a confusing red rather than a shipped
//! bug — but the prose was an invitation to do the thing the test forbids. The
//! test's own comment already knew, one screen below: *which a version that
//! merely tried the panel first would also pass.* That is this file describing
//! its own implementation correctly in one place and wrongly in another.
//!
//! **And the claim was defending something the return type already gives.**
//! `whose_area` answers one [`WhoseArea`], so two surfaces cannot both claim a
//! point whatever the rectangles are — that is above, it is true, and it needs
//! no subtraction. What the order decides is not *whether* a point has one
//! owner but *which* owner it has where the reserved column overlaps the Dock's
//! band. The panel is asked first because the reserved column is the panel's
//! whatever else reaches it, which is the owner's rule of 2026-09-30, and
//! `the_regions_a_pointer_can_be_in` is the argument. **The grid test is what
//! holds it**, not a subtraction: swap the arms and it fails on the first
//! contested point.
//!
//! *The same shape as `alo_put_aside::restoring::travel_for` asking the Place
//! before the geometry: the order is the property, so it is stated as the
//! property rather than described as an incidental.*
//!
//! # Told, never fetched, and therefore no screen size
//!
//! Every rectangle is handed in. This file does not ask a compositor where
//! anything is, because only the caller knows this display's scale and origin —
//! the same seam `crate::dock_room` takes, and for the same reason.
//!
//! It also satisfies the standing rule that nothing is built to one screen
//! size the way that rule is best satisfied: **there is no figure here to
//! convert.** Not a strip width, not a reserved width, not a margin. A reader
//! looking for 1440 will not find it, because the caller that knows the display
//! is the one that computes them.
//!
//! `docs/design/the-regions-a-pointer-can-be-in.md` holds the measurements and
//! which kind of number each one is.

use smithay::utils::{Physical, Point, Rectangle};

/// Whose area a point falls in.
///
/// **One value, which is the whole design of this module.** Two surfaces
/// cannot both claim a point because a function cannot return two answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhoseArea {
    /// The panel of put-aside windows, including the corner it shares with the
    /// top controls.
    ThePanel,
    /// The Dock along the bottom edge, up to where the panel's area begins.
    TheDock,
    /// The controls along the top edge, up to where the panel's area begins.
    TheTopControls,
    /// No revealing surface. The window underneath has it.
    Nobody,
}

/// Where the three surfaces are on one display, in that display's physical
/// pixels.
///
/// The caller computes these from the display and the panel's current width;
/// nothing here derives them, and nothing here remembers them between frames.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TheSurfaces {
    /// What the panel owns outright: its own extent along its edge, plus the
    /// reserved width the other two stop at. It reaches the full height of the
    /// display, which is what makes the shared corner the panel's.
    pub(crate) reserved_for_the_panel: Rectangle<i32, Physical>,
    /// Where the Dock asks and keeps, **as drawn and not reduced**.
    ///
    /// This said *before the panel's area is taken out of it* and *the
    /// subtraction happens here so that one rule is applied in one place*. It
    /// does not happen here or anywhere: this is the region the caller drew,
    /// and the reserved column is kept off it by [`whose_area`] asking the
    /// panel first rather than by any rectangle being made smaller. Corrected
    /// 2026-10-01; the header says what holds the rule instead.
    pub(crate) the_docks_area: Rectangle<i32, Physical>,
    /// The same for the controls along the top edge.
    pub(crate) the_top_controls_area: Rectangle<i32, Physical>,
}

/// Whose area this point is in.
///
/// **The panel is asked first and the other two are asked about what is left**,
/// rather than the three being tried in an order that happens to favour one. A
/// point inside the panel's reserved area is the panel's even if it also lies
/// inside the rectangle the Dock was drawn with, which is the owner's rule
/// stated once.
///
/// A rectangle with no area contains nothing, checked rather than assumed —
/// `crate::dock_room::overlaps` learned that the strict comparisons do not give
/// it for free, and the same arithmetic is here.
pub(crate) fn whose_area(at: Point<i32, Physical>, surfaces: &TheSurfaces) -> WhoseArea {
    if holds(surfaces.reserved_for_the_panel, at) {
        return WhoseArea::ThePanel;
    }
    if holds(surfaces.the_docks_area, at) {
        return WhoseArea::TheDock;
    }
    if holds(surfaces.the_top_controls_area, at) {
        return WhoseArea::TheTopControls;
    }
    WhoseArea::Nobody
}

/// Whether a rectangle contains a point.
///
/// Half-open: the top-left edge belongs to the rectangle and the bottom-right
/// does not, so two rectangles that share an edge do not both contain the
/// points along it. Without that, *stops before the reserved area* and *owns
/// the reserved area* would both be true on the seam between them — which is
/// exactly the pointer position the owner's rule is about.
///
/// A rectangle with no width or no height contains nothing.
fn holds(rectangle: Rectangle<i32, Physical>, at: Point<i32, Physical>) -> bool {
    if rectangle.size.w <= 0 || rectangle.size.h <= 0 {
        return false;
    }
    let right = rectangle.loc.x.saturating_add(rectangle.size.w);
    let bottom = rectangle.loc.y.saturating_add(rectangle.size.h);
    at.x >= rectangle.loc.x && at.x < right && at.y >= rectangle.loc.y && at.y < bottom
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::utils::Size;

    /// A display 1440 across and 960 down, with the three areas the design
    /// draws — but with the panel's reserved column running the full height,
    /// as the owner's rule requires, and the other two passed as drawn.
    ///
    /// The numbers are the design's, used here as *a* screen rather than *the*
    /// screen: nothing in the module reads them, and the tests below that care
    /// about size use a different display on purpose.
    fn as_the_design_draws_them() -> TheSurfaces {
        TheSurfaces {
            reserved_for_the_panel: rect(1328, 0, 112, 960),
            the_docks_area: rect(304, 838, 832, 122),
            the_top_controls_area: rect(0, 0, 1440, 84),
        }
    }

    fn rect(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Physical> {
        Rectangle::new(Point::from((x, y)), Size::from((w, h)))
    }

    fn at(x: i32, y: i32) -> Point<i32, Physical> {
        Point::from((x, y))
    }

    /// **The shared corner is the panel's**, which is the decision this module
    /// exists to carry. The top controls are drawn across the whole width, so
    /// this point is inside their rectangle too.
    #[test]
    fn the_corner_they_share_belongs_to_the_panel() {
        let surfaces = as_the_design_draws_them();
        let corner = at(1400, 40);

        assert!(
            holds(surfaces.the_top_controls_area, corner),
            "the test is not about the corner unless the top controls are drawn over it"
        );
        assert_eq!(whose_area(corner, &surfaces), WhoseArea::ThePanel);
    }

    /// **The top controls keep everything left of the reserved area.**
    #[test]
    fn the_top_controls_keep_what_the_panel_does_not_reserve() {
        let surfaces = as_the_design_draws_them();
        assert_eq!(
            whose_area(at(700, 40), &surfaces),
            WhoseArea::TheTopControls
        );
        assert_eq!(
            whose_area(at(1327, 40), &surfaces),
            WhoseArea::TheTopControls,
            "one pixel short of the reserved area is still theirs"
        );
        assert_eq!(
            whose_area(at(1328, 40), &surfaces),
            WhoseArea::ThePanel,
            "the first column of the reserved area is the panel's"
        );
    }

    /// **The Dock keeps the bottom edge up to the same bound.**
    #[test]
    fn the_dock_keeps_the_bottom_up_to_the_same_bound() {
        let surfaces = as_the_design_draws_them();
        assert_eq!(whose_area(at(700, 900), &surfaces), WhoseArea::TheDock);
        assert_eq!(whose_area(at(1400, 900), &surfaces), WhoseArea::ThePanel);
    }

    /// **The window underneath keeps everything else.** A surface nobody is
    /// reaching for does not get a pointer resting in the middle of the work.
    #[test]
    fn the_middle_of_the_screen_belongs_to_nobody() {
        let surfaces = as_the_design_draws_them();
        assert_eq!(whose_area(at(700, 400), &surfaces), WhoseArea::Nobody);
    }

    /// **Two surfaces cannot both claim a point, at any width, and this test
    /// cannot fail while the signature is what it is.**
    ///
    /// It is written anyway, over a grid and over three panel widths including
    /// the widest, because that is where the top region is smallest and the
    /// corner nearest.
    ///
    /// **What it actually holds is the branch order**: that the answer for a
    /// point inside the reserved area does not depend on what the other two
    /// rectangles are. A version that tried the panel **last** fails it; the
    /// version written here, which tries the panel first, passes it — and that
    /// is not a weakness of the test, it is the point of it, because trying the
    /// panel first *is* the implementation.
    ///
    /// *This read `what it actually holds is the subtraction` until 2026-10-01,
    /// naming an implementation this file never had, and said in the same breath
    /// that a version which merely tried the panel first would also pass —
    /// which was the file reporting its own code correctly while the header
    /// denied it. The lane that read the header against the code measured 378
    /// points of this grid changing answer when the arms are reordered.*
    #[test]
    fn no_point_is_claimed_by_two_surfaces_at_any_panel_width() {
        for reserved_width in [112, 240, 512] {
            let left = 1440 - reserved_width;
            let surfaces = TheSurfaces {
                reserved_for_the_panel: rect(left, 0, reserved_width, 960),
                // Both drawn across the whole display on purpose: the reserved
                // column overlaps them completely, so every point in it is
                // contested and only the branch order decides it. Ask the panel
                // last and this fails on the first such point.
                the_docks_area: rect(0, 838, 1440, 122),
                the_top_controls_area: rect(0, 0, 1440, 84),
            };

            for x in (0..1440).step_by(16) {
                for y in (0..960).step_by(16) {
                    let point = at(x, y);
                    let whose = whose_area(point, &surfaces);
                    if holds(surfaces.reserved_for_the_panel, point) {
                        assert_eq!(
                            whose,
                            WhoseArea::ThePanel,
                            "({x}, {y}) is in the reserved column at width \
                             {reserved_width} and went to {whose:?}"
                        );
                    } else {
                        assert_ne!(
                            whose,
                            WhoseArea::ThePanel,
                            "({x}, {y}) is outside the reserved column at width \
                             {reserved_width} and went to the panel"
                        );
                    }
                }
            }
        }
    }

    /// **A panel at the other edge works without this file naming an edge.**
    ///
    /// The panel's edge is data, not a constant —
    /// `alo_put_aside::the_region_the_panel_claims::WhichEdge` is why. Nothing
    /// here reads it: a reserved column on the left is decided by the same
    /// branch order as one on the right, because that order names no edge
    /// either.
    #[test]
    fn a_panel_mirrored_to_the_left_needs_no_change_here() {
        let surfaces = TheSurfaces {
            reserved_for_the_panel: rect(0, 0, 112, 960),
            the_docks_area: rect(304, 838, 832, 122),
            the_top_controls_area: rect(0, 0, 1440, 84),
        };
        assert_eq!(whose_area(at(40, 40), &surfaces), WhoseArea::ThePanel);
        assert_eq!(
            whose_area(at(700, 40), &surfaces),
            WhoseArea::TheTopControls
        );
    }

    /// **A display that is not the one the design was drawn on.** The module
    /// holds no figure, so a small panel screen and a wide desk screen are the
    /// same arithmetic.
    #[test]
    fn a_screen_that_is_not_the_frame_the_design_was_drawn_on() {
        let small = TheSurfaces {
            reserved_for_the_panel: rect(1278, 0, 88, 768),
            the_docks_area: rect(200, 690, 600, 78),
            the_top_controls_area: rect(0, 0, 1366, 64),
        };
        assert_eq!(whose_area(at(1300, 20), &small), WhoseArea::ThePanel);
        assert_eq!(whose_area(at(400, 20), &small), WhoseArea::TheTopControls);
        assert_eq!(whose_area(at(400, 700), &small), WhoseArea::TheDock);

        let wide = TheSurfaces {
            reserved_for_the_panel: rect(3616, 0, 224, 2160),
            the_docks_area: rect(1000, 1900, 1800, 244),
            the_top_controls_area: rect(0, 0, 3840, 168),
        };
        assert_eq!(whose_area(at(3700, 80), &wide), WhoseArea::ThePanel);
        assert_eq!(whose_area(at(1500, 80), &wide), WhoseArea::TheTopControls);
    }

    /// **A surface with no area claims nothing**, which had to be written
    /// rather than inherited — `crate::dock_room` found that the strict
    /// comparisons do not give it for free, and a display with no panel yet is
    /// a real state rather than a tidiness one.
    #[test]
    fn a_surface_with_no_area_claims_nothing() {
        let surfaces = TheSurfaces {
            reserved_for_the_panel: rect(1328, 0, 0, 960),
            the_docks_area: rect(304, 838, 832, 0),
            the_top_controls_area: rect(0, 0, 1440, 84),
        };
        assert_eq!(
            whose_area(at(1400, 40), &surfaces),
            WhoseArea::TheTopControls,
            "a panel of no width kept the corner it does not occupy"
        );
        assert_eq!(whose_area(at(700, 900), &surfaces), WhoseArea::Nobody);
    }

    /// **The seam belongs to exactly one of them.** Half-open containment is
    /// what makes *stops before the reserved area* and *owns the reserved area*
    /// describe the same column boundary rather than disagreeing on it.
    #[test]
    fn the_seam_between_two_areas_belongs_to_one_of_them() {
        let surfaces = TheSurfaces {
            reserved_for_the_panel: rect(1328, 0, 112, 960),
            the_docks_area: rect(0, 838, 1328, 122),
            the_top_controls_area: rect(0, 0, 1328, 84),
        };
        assert_eq!(whose_area(at(1327, 900), &surfaces), WhoseArea::TheDock);
        assert_eq!(whose_area(at(1328, 900), &surfaces), WhoseArea::ThePanel);
        assert_eq!(
            whose_area(at(1439, 959), &surfaces),
            WhoseArea::ThePanel,
            "the last pixel of the display is inside the panel's column"
        );
        assert_eq!(
            whose_area(at(1440, 959), &surfaces),
            WhoseArea::Nobody,
            "one past the display's edge is nobody's"
        );
    }

    /// **No measurement and no edge reaches the code in this file.** Held by
    /// reading the source above the tests, the way this repository's other
    /// source checks are — the standing rule is that a figure from a frame
    /// becomes a rule, and the way this module keeps it is by having no figure
    /// to convert.
    ///
    /// **The prose is excluded, and its first version was not.** That version
    /// failed on the module header's own sentence *a reader looking for 1440
    /// will not find it*, which contains the digits it denies. The claim worth
    /// holding is about what the code does, and a check that cannot quote the
    /// owner's ruling — which says *Right panel* — is a check that forbids
    /// explaining itself.
    #[test]
    fn no_measurement_reaches_the_code_in_this_file() {
        let source = include_str!("surface_areas.rs");
        let above = source
            .split_once("#[cfg(test)]")
            .map_or(source, |(above, _)| above);
        let code: String = above
            .lines()
            .filter(|line| {
                let line = line.trim_start();
                !line.starts_with("//!") && !line.starts_with("///") && !line.starts_with("//")
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            code.contains("pub(crate) fn whose_area"),
            "the filter took the code away with the prose"
        );
        assert!(
            !code.contains("reader looking"),
            "the filter left prose behind, so the rest of this proves nothing"
        );

        for forbidden in ["1440", "960", "1366", "112", "Right", "Left"] {
            assert!(
                !code.contains(forbidden),
                "a measurement or an edge reached the code: {forbidden}"
            );
        }
    }
}
