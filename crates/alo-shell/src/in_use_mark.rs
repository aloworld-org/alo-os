//! The three shapes beside the in-use indicator's lines: a screen, a lens and
//! a microphone, drawn so silhouette alone tells them apart.
//!
//! `alo_in_use::Mark` says *which* shape and nothing about size, place or
//! stroke — its own words: *a person distinguishing them at the size of a
//! status-area glyph is reading silhouette, not strokes*. This file is the
//! drawing, and it keeps that promise the only way a rasteriser can: the three
//! are unalike in outline, at whatever size the person's text scale gives them,
//! and each stays recognisable with every hue taken off the screen.
//!
//! # Why a shape at all
//!
//! ADR 0010: terracotta on cream is 2.87:1, under what WCAG 2.1 §1.4.11 asks of
//! a shape carrying meaning. So the agent never appears without a mark and a
//! word beside its colour — and this indicator has three things to tell apart
//! rather than one, so each gets a mark of its own.

use alo_appearance::{Colour, Scheme, Token};
use smithay::utils::{Physical, Rectangle};

use crate::Contrast;
use crate::painted::Solid;

/// A mark's colours for one line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MarkColours {
    /// Its edge, against the row's ground.
    pub(crate) edge: [u8; 3],
    /// Its fill: the line's own colour, which is terracotta exactly when the
    /// agent is the one using the thing, and navy otherwise.
    pub(crate) fill: [u8; 3],
    /// The shape cut out of the fill, against it.
    pub(crate) cut: [u8; 3],
}

impl MarkColours {
    /// The colours for a line of `colour`, in this scheme and palette.
    ///
    /// The fill is the token `alo-in-use` chose, taken through the same palette
    /// door every other surface uses — so in high contrast it is that palette's
    /// own terracotta rather than the design's, and still means the agent.
    pub(crate) fn of(colour: Token, scheme: Scheme, contrast: Contrast) -> Self {
        Self {
            edge: contrast.ink(scheme),
            fill: contrast.accent(scheme, colour.colour()),
            cut: match contrast {
                Contrast::AsDesigned => rgb(match scheme {
                    Scheme::Light => Token::Cream.colour(),
                    Scheme::Dark => Token::Charcoal.colour(),
                }),
                Contrast::High => contrast.ground(scheme),
            },
        }
    }
}

/// A colour as the painter takes it.
const fn rgb(colour: Colour) -> [u8; 3] {
    [colour.red(), colour.green(), colour.blue()]
}

/// Draw one mark inside `area`, in the shape `alo-in-use` named.
///
/// The whole square is the mark's; each shape is drawn inside it with its own
/// outline, and no two share a silhouette:
///
/// - **the screen** is a wide rectangle standing on a foot,
/// - **the lens** is a filled circle inside a ring, which is the only round
///   one,
/// - **the microphone** is an upright capsule on a stem, which is the only one
///   taller than it is wide.
pub(crate) fn mark(
    area: Rectangle<i32, Physical>,
    which: alo_in_use::Mark,
    colours: MarkColours,
) -> Vec<Solid> {
    let mut solids = vec![Solid {
        area,
        colour: colours.edge,
    }];
    let inset = (area.size.w / 8).max(1);
    let inner = shrink(area, inset);
    solids.push(Solid {
        area: inner,
        colour: colours.fill,
    });
    match which {
        alo_in_use::Mark::Rectangle => screen(inner, colours, &mut solids),
        alo_in_use::Mark::Lens => lens(inner, colours, &mut solids),
        alo_in_use::Mark::Capsule => microphone(inner, colours, &mut solids),
    }
    solids.retain(|solid| solid.area.size.w > 0 && solid.area.size.h > 0);
    solids
}

/// A wide rectangle standing on a foot.
///
/// **Wider than it is tall, including the foot**, which is what makes it read
/// as a screen rather than as a page: the panel takes most of the width and
/// only half the height, and the foot sits straight under it.
fn screen(inside: Rectangle<i32, Physical>, colours: MarkColours, solids: &mut Vec<Solid>) {
    let step = (inside.size.w / 5).max(1);
    let panel = Rectangle::new(
        (inside.loc.x + step / 2, inside.loc.y + inside.size.h / 4).into(),
        ((inside.size.w - step).max(1), (inside.size.h / 2).max(1)).into(),
    );
    solids.push(Solid {
        area: panel,
        colour: colours.cut,
    });
    solids.push(Solid {
        area: Rectangle::new(
            (inside.loc.x + 2 * step, panel.loc.y + panel.size.h).into(),
            ((inside.size.w - 4 * step).max(1), step).into(),
        ),
        colour: colours.cut,
    });
}

/// A circle inside a ring: the only round mark.
fn lens(inside: Rectangle<i32, Physical>, colours: MarkColours, solids: &mut Vec<Solid>) {
    let radius = (inside.size.w.min(inside.size.h) / 2).max(1);
    let centre = (
        inside.loc.x + inside.size.w / 2,
        inside.loc.y + inside.size.h / 2,
    );
    // A disc drawn as rows, which is how a rasteriser without a path makes a
    // circle: each row is as wide as the chord at that height.
    for row in 0..radius {
        let half = chord(radius, row);
        for (y, height) in [(centre.1 - row - 1, 1), (centre.1 + row, 1)] {
            solids.push(Solid {
                area: Rectangle::new(
                    (centre.0 - half, y).into(),
                    ((half * 2).max(1), height).into(),
                ),
                colour: colours.cut,
            });
        }
    }
}

/// Half the width of a disc of `radius` at `row` rows from its middle.
fn chord(radius: i32, row: i32) -> i32 {
    let inner = radius.saturating_mul(radius) - row.saturating_mul(row);
    let mut half: i32 = 0;
    while (half + 1).saturating_mul(half + 1) <= inner {
        half += 1;
    }
    (half * 2 / 3).max(1)
}

/// An upright capsule on a stem: the only mark taller than it is wide.
fn microphone(inside: Rectangle<i32, Physical>, colours: MarkColours, solids: &mut Vec<Solid>) {
    let step = (inside.size.w / 5).max(1);
    let body = Rectangle::new(
        (inside.loc.x + 2 * step, inside.loc.y + step).into(),
        (step, (inside.size.h - 3 * step).max(1)).into(),
    );
    solids.push(Solid {
        area: body,
        colour: colours.cut,
    });
    solids.push(Solid {
        area: Rectangle::new(
            (inside.loc.x + step, body.loc.y + body.size.h).into(),
            ((3 * step).max(1), step.max(1)).into(),
        ),
        colour: colours.cut,
    });
}

/// The same rectangle, `by` pixels smaller on every side.
fn shrink(area: Rectangle<i32, Physical>, by: i32) -> Rectangle<i32, Physical> {
    Rectangle::new(
        (area.loc.x + by, area.loc.y + by).into(),
        ((area.size.w - 2 * by).max(1), (area.size.h - 2 * by).max(1)).into(),
    )
}

#[cfg(test)]
#[path = "in_use_mark_tests.rs"]
mod tests;
