//! The division on one display: the share each window is in, the boundary
//! between two, and the outline a drop would take.
//!
//! **Whose decisions these are.** `alo-dividing` decides all of them — where a
//! share is (`Division::shares`), what a drop would do
//! (`Division::propose_drop`, which answers with an `Offer`), and where a
//! boundary moves to (`Division::move_boundary`). This file turns those answers
//! into pixels and decides none of them. There is no branch here on how near an
//! edge a pointer is, no half, no quarter, and nowhere one could be added.
//!
//! # The outline is drawn before the drop, because that is the point of it
//!
//! A person dragging a window is deciding whether to let go. The outline is the
//! answer to *what happens if I do*, and it is drawn from
//! `alo_dividing::Proposal::area` — the same rectangle
//! `alo_dividing::Division::commit` will use, rather than a second guess at it.
//! An outline that disagreed with the commit would be the machine lying at the
//! one moment somebody could still change their mind.
//!
//! # One layout decision, and it is not this crate's
//!
//! There was a second one until 2026-09-26: `crate::window_tiling` computed
//! half of an output and set a window mode from it. It has gone, and a chord
//! now reaches `crate::window_dividing`, which hands the two windows to
//! `alo_dividing::Division::divide_with_next` and lays out what comes back.
//! The plan's constraint is that there are never two layout deciders in one
//! compositor; removing the half is how that is kept, rather than keeping a
//! half that agreed with the division by inspection.
//!
//! # Logical units in, pixels out
//!
//! `alo-dividing` works in logical units and knows nothing about scale.
//! Everything here multiplies by the display's scale exactly once, at the
//! boundary, so a division is never stored in pixels and a share never arrives
//! already scaled.

use alo_dividing::{Area, Division, Offer};
use smithay::utils::{Physical, Point, Rectangle};

use crate::painted::Solid;

/// How thick the outline a drop would take is drawn, in logical units.
const OUTLINE: i32 = 4;

/// How thick the rule between two shares is, in logical units.
const BOUNDARY: i32 = 2;

/// The division on one display, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct DivisionPicture {
    /// Each window's share, in painting order.
    pub(crate) shares: Vec<Rectangle<i32, Physical>>,
    /// The rule between two shares.
    pub(crate) boundaries: Vec<Rectangle<i32, Physical>>,
    /// The outline a drop would take, where a drop is being offered.
    pub(crate) offered: Option<Rectangle<i32, Physical>>,
    /// Flat shapes, in painting order.
    pub(crate) solids: Vec<Solid>,
}

/// One logical area as pixels on a display of this scale.
fn in_pixels(area: Area, scale: i32) -> Rectangle<i32, Physical> {
    let at: Point<i32, Physical> = (
        i32::try_from(area.x())
            .unwrap_or(i32::MAX)
            .saturating_mul(scale),
        i32::try_from(area.y())
            .unwrap_or(i32::MAX)
            .saturating_mul(scale),
    )
        .into();
    let size = (
        i32::try_from(area.width())
            .unwrap_or(i32::MAX)
            .saturating_mul(scale),
        i32::try_from(area.height())
            .unwrap_or(i32::MAX)
            .saturating_mul(scale),
    );
    Rectangle::new(at, size.into())
}

/// Draw `division` on a display of `scale`, with `offer` outlined where one is
/// being made.
pub(crate) fn picture(
    division: &Division,
    offer: &Offer,
    scale: i32,
    ink: [u8; 3],
    accent: [u8; 3],
) -> DivisionPicture {
    let scale = scale.max(1);
    let shares: Vec<Rectangle<i32, Physical>> = division
        .shares()
        .into_iter()
        .map(|share| in_pixels(share.area(), scale))
        .collect();
    let boundaries = between(&shares, BOUNDARY * scale);

    let mut solids: Vec<Solid> = boundaries
        .iter()
        .map(|rule| Solid {
            area: *rule,
            colour: ink,
        })
        .collect();

    // The outline a drop would take. Drawn in the person's accent, because it
    // is the one thing on the screen that is about to happen rather than
    // already true.
    let offered = match offer {
        Offer::Proposed(proposal) => Some(in_pixels(proposal.area(), scale)),
        // Nothing offered and a refusal both draw nothing. A refusal is said in
        // words by `alo-dividing`'s own vocabulary, and an outline that meant
        // *no* would be a shape a person has to learn.
        Offer::Nothing | Offer::Refused(_) => None,
    };
    if let Some(outline) = offered {
        solids.extend(edges_of(outline, OUTLINE * scale, accent));
    }

    DivisionPicture {
        shares,
        boundaries,
        offered,
        solids,
    }
}

/// The rules where two shares meet.
///
/// Found from the shares themselves rather than from the tree, so a boundary
/// cannot come to be drawn somewhere no two shares actually meet.
fn between(shares: &[Rectangle<i32, Physical>], thick: i32) -> Vec<Rectangle<i32, Physical>> {
    let mut rules = Vec::new();
    for (at, one) in shares.iter().enumerate() {
        for other in shares.iter().skip(at + 1) {
            let right = one.loc.x + one.size.w;
            let bottom = one.loc.y + one.size.h;
            // Side by side: one's right edge is the other's left.
            if right == other.loc.x {
                let top = one.loc.y.max(other.loc.y);
                let low = bottom.min(other.loc.y + other.size.h);
                if low > top {
                    rules.push(Rectangle::new(
                        (right - thick / 2, top).into(),
                        (thick, low - top).into(),
                    ));
                }
            }
            // One above the other.
            if bottom == other.loc.y {
                let left = one.loc.x.max(other.loc.x);
                let right_edge = right.min(other.loc.x + other.size.w);
                if right_edge > left {
                    rules.push(Rectangle::new(
                        (left, bottom - thick / 2).into(),
                        (right_edge - left, thick).into(),
                    ));
                }
            }
        }
    }
    rules
}

/// The four edges of a rectangle, as an outline rather than a fill.
///
/// A filled rectangle would hide the window underneath it, and what a person is
/// deciding about is where the window is going rather than what colour it is.
fn edges_of(area: Rectangle<i32, Physical>, thick: i32, colour: [u8; 3]) -> Vec<Solid> {
    let thick = thick.max(1).min(area.size.w.min(area.size.h).max(1));
    let (x, y, w, h) = (area.loc.x, area.loc.y, area.size.w, area.size.h);
    vec![
        Solid {
            area: Rectangle::new((x, y).into(), (w, thick).into()),
            colour,
        },
        Solid {
            area: Rectangle::new((x, y + h - thick).into(), (w, thick).into()),
            colour,
        },
        Solid {
            area: Rectangle::new((x, y).into(), (thick, h).into()),
            colour,
        },
        Solid {
            area: Rectangle::new((x + w - thick, y).into(), (thick, h).into()),
            colour,
        },
    ]
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_dividing::area::{Point as LogicalPoint, Size};
    use alo_dividing::window::{Window, WindowId};
    use alo_dividing::{Area, Side};

    use super::*;

    /// The ink and the accent, named rather than resolved: which colours a
    /// person chose is the look's business and not this file's.
    const INK: [u8; 3] = [17, 17, 17];
    const ACCENT: [u8; 3] = [0, 90, 200];

    /// A display of a usual size, in logical units.
    fn a_display() -> Area {
        Area::of(LogicalPoint::at(0, 0), Size::of(1920, 1080)).expect("a display")
    }

    /// A window the compositor is calling this.
    fn a_window(id: u64) -> Window {
        Window::any_size(WindowId::from_compositor(id))
    }

    /// Mail beside an editor, divided side by side.
    fn side_by_side() -> Division {
        let mut division = Division::of(a_display());
        division
            .divide_with_next(a_window(1), Some(a_window(2)), Side::Left)
            .expect("two windows share a display");
        division
    }

    /// **Every share the division decided is drawn, and no other.**
    ///
    /// Counted against `Division::shares`, so a picture cannot come to hold a
    /// rectangle the division does not know about.
    #[test]
    fn the_shares_drawn_are_the_shares_the_division_decided() {
        let division = side_by_side();
        let drawn = picture(&division, &Offer::Nothing, 1, INK, ACCENT);

        let decided: Vec<Rectangle<i32, Physical>> = division
            .shares()
            .into_iter()
            .map(|share| in_pixels(share.area(), 1))
            .collect();
        assert_eq!(drawn.shares, decided);
        assert_eq!(drawn.shares.len(), 2);
    }

    /// **A boundary is drawn where two shares meet, and only there.**
    #[test]
    fn a_boundary_is_drawn_between_two_shares_and_nowhere_else() {
        let alone = Division::of(a_display());
        let undivided = picture(&alone, &Offer::Nothing, 1, INK, ACCENT);
        assert!(
            undivided.boundaries.is_empty(),
            "an undivided display was given a boundary"
        );

        let drawn = picture(&side_by_side(), &Offer::Nothing, 1, INK, ACCENT);
        assert_eq!(drawn.boundaries.len(), 1);
        let rule = drawn.boundaries.first().expect("one rule");
        // It runs down the screen between the two, not across it.
        assert!(rule.size.h > rule.size.w, "{rule:?}");
        for share in &drawn.shares {
            assert!(
                rule.loc.x >= share.loc.x - 2 || rule.loc.x <= share.loc.x + share.size.w + 2,
                "the rule is not where the two shares meet"
            );
        }
    }

    /// **A boundary moved by the division is drawn where it moved to**, so the
    /// rule and the shares cannot disagree about where the edge is.
    #[test]
    fn moving_a_boundary_moves_the_rule_with_it() {
        let mut division = side_by_side();
        let before = picture(&division, &Offer::Nothing, 1, INK, ACCENT);
        let was = before.boundaries.first().expect("one rule").loc.x;

        division
            .move_boundary(a_window(1).id(), Side::Right, 1200)
            .expect("a boundary a person may drag");
        let after = picture(&division, &Offer::Nothing, 1, INK, ACCENT);
        let now = after.boundaries.first().expect("one rule").loc.x;

        assert_ne!(was, now, "the rule did not move with the boundary");
        // And it is still between the two shares it separates.
        let left = after.shares.first().expect("a share");
        assert!(
            (now - (left.loc.x + left.size.w)).abs() <= BOUNDARY,
            "the rule left the edge it marks: {now} against {left:?}"
        );
    }

    /// **What a drop would do is outlined before it is done**, with exactly the
    /// rectangle the division proposed.
    ///
    /// The outline is the answer to *what happens if I let go*. One that
    /// disagreed with what `commit` then does would be the machine lying at the
    /// one moment somebody could still change their mind — so it is taken from
    /// `Proposal::area` rather than worked out again here.
    #[test]
    fn a_drop_is_outlined_with_the_area_the_division_proposed() {
        let division = side_by_side();
        let offer = division.propose_drop(LogicalPoint::at(4, 540), a_window(3), None);
        let Offer::Proposed(proposal) = &offer else {
            unreachable!("a pointer at the left edge proposes a place: {offer:?}")
        };
        let expected = in_pixels(proposal.area(), 1);

        let drawn = picture(&division, &offer, 1, INK, ACCENT);
        assert_eq!(drawn.offered, Some(expected));

        // It is an outline rather than a fill: four edges, and the middle of
        // the proposed area is not painted.
        let middle = (
            expected.loc.x + expected.size.w / 2,
            expected.loc.y + expected.size.h / 2,
        );
        assert!(
            !drawn.solids.iter().any(|solid| {
                solid.colour == ACCENT
                    && solid.area.loc.x < middle.0
                    && middle.0 < solid.area.loc.x + solid.area.size.w
                    && solid.area.loc.y < middle.1
                    && middle.1 < solid.area.loc.y + solid.area.size.h
            }),
            "the proposal was filled rather than outlined, hiding the window under it"
        );
    }

    /// **Nothing offered and a refusal both draw nothing**, and neither is a
    /// shape a person has to learn.
    #[test]
    fn nothing_offered_and_a_refusal_are_both_drawn_as_nothing() {
        let division = side_by_side();
        let middle = division.propose_drop(LogicalPoint::at(960, 540), a_window(3), None);
        assert_eq!(middle, Offer::Nothing, "the middle of a screen offered one");

        let drawn = picture(&division, &middle, 1, INK, ACCENT);
        assert_eq!(drawn.offered, None);
        assert!(
            !drawn.solids.iter().any(|solid| solid.colour == ACCENT),
            "something was drawn in the accent with nothing offered"
        );
    }

    /// **A division is in logical units and the display is in pixels**, and
    /// the scale is applied exactly once.
    ///
    /// A share that arrived already scaled, or was scaled twice, would put a
    /// window at half or four times the size of the place it was given.
    #[test]
    fn the_scale_is_applied_once_at_the_boundary() {
        let division = side_by_side();
        let at_one = picture(&division, &Offer::Nothing, 1, INK, ACCENT);
        let at_two = picture(&division, &Offer::Nothing, 2, INK, ACCENT);

        for (one, two) in at_one.shares.iter().zip(at_two.shares.iter()) {
            assert_eq!(two.loc.x, one.loc.x * 2);
            assert_eq!(two.loc.y, one.loc.y * 2);
            assert_eq!(two.size.w, one.size.w * 2);
            assert_eq!(two.size.h, one.size.h * 2);
        }
    }
}
