//! The mark beside every line of the egress indicator: terracotta, and never
//! only terracotta.
//!
//! `docs/features.md`, ★: *terracotta always arrives with a mark and a word. A
//! signal carried by hue fails for anybody who cannot distinguish that hue,
//! and EN 301 549 does not allow colour to be the only means of conveying
//! anything.* The word is the line `alo-egress` wrote; this is the mark — an
//! arrow leaving a square, drawn in a colour that stands apart from terracotta
//! by lightness and not by hue, inside an edge that stands apart from the
//! ground. Take every hue off the screen and the arrow is still there.

use alo_appearance::{Colour, Scheme, Token};

use crate::Contrast;
use smithay::utils::Rectangle;

use crate::painted::Solid;

/// The three colours of the mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MarkColours {
    /// Its edge, against the line's ground.
    pub(crate) edge: Colour,
    /// Its fill: terracotta, which means the agent and nothing else.
    pub(crate) fill: Colour,
    /// The arrow, against the fill.
    pub(crate) arrow: Colour,
}

impl MarkColours {
    /// The mark's three colours in this scheme, in whichever palette the
    /// surface around it is drawn in.
    ///
    /// The fill is terracotta in the design and `alo_access::HighContrast`'s
    /// own terracotta — the same hue, taken deep enough to read — where high
    /// contrast is on, because the mark means the agent in both (ADR 0010).
    /// The arrow is drawn against that fill: the design's navy or charcoal,
    /// and in high contrast the ground, which is the far end of a palette
    /// whose accent was chosen to sit at the other one.
    pub(crate) fn of(scheme: Scheme, contrast: Contrast) -> Self {
        Self {
            edge: colour(contrast.ink(scheme)),
            fill: colour(contrast.accent(scheme, Token::Terracotta.colour())),
            arrow: match contrast {
                Contrast::AsDesigned => match scheme {
                    Scheme::Light => Token::Navy.colour(),
                    Scheme::Dark => Token::Charcoal.colour(),
                },
                Contrast::High => colour(contrast.ground(scheme)),
            },
        }
    }
}

/// A colour as the palette door hands it over.
const fn colour(rgb: [u8; 3]) -> Colour {
    Colour::of(rgb[0], rgb[1], rgb[2])
}

/// A colour as the painter takes it.
pub(crate) fn rgb(colour: Colour) -> [u8; 3] {
    [colour.red(), colour.green(), colour.blue()]
}

/// The mark, `side` pixels square with its top-left corner at (`x`, `y`).
///
/// The edge first, the terracotta inside it, and then the arrow: a stem and a
/// head that widens one row at a time, pointing up and out of the square.
pub(crate) fn mark(x: i32, y: i32, side: i32, scheme: Scheme, contrast: Contrast) -> Vec<Solid> {
    let colours = MarkColours::of(scheme, contrast);
    let side = side.max(8);
    let edge = (side / 12).max(1);
    let inner = side - 2 * edge;
    let mut solids = vec![
        Solid {
            area: Rectangle::new((x, y).into(), (side, side).into()),
            colour: rgb(colours.edge),
        },
        Solid {
            area: Rectangle::new((x + edge, y + edge).into(), (inner, inner).into()),
            colour: rgb(colours.fill),
        },
    ];
    let (left, top) = (x + edge, y + edge);
    let stem = (inner / 6).max(1);
    let head_top = inner / 6;
    let head_rows = (inner * 3 / 10).max(1);
    let widest = (inner * 3 / 5).max(stem);
    let centre = left + inner / 2;
    for row in 0..head_rows {
        let wide = stem + (widest - stem) * (row + 1) / head_rows;
        solids.push(Solid {
            area: Rectangle::new(
                (centre - wide / 2, top + head_top + row).into(),
                (wide, 1).into(),
            ),
            colour: rgb(colours.arrow),
        });
    }
    let stem_top = top + head_top + head_rows;
    let stem_bottom = top + inner - inner / 6;
    solids.push(Solid {
        area: Rectangle::new(
            (centre - stem / 2, stem_top).into(),
            (stem, (stem_bottom - stem_top).max(1)).into(),
        ),
        colour: rgb(colours.arrow),
    });
    solids
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_appearance::{ENOUGH_FOR_A_SHAPE, ENOUGH_FOR_TEXT};

    /// **The arrow is told from the terracotta by lightness, not by hue**, in
    /// both schemes — WCAG 2.1 §1.4.11's contrast for a shape that is the only
    /// thing saying something.
    #[test]
    fn the_arrow_stands_apart_from_terracotta_without_its_hue() {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let colours = MarkColours::of(scheme, Contrast::AsDesigned);
            assert_eq!(colours.fill, Token::Terracotta.colour());
            let contrast = colours.arrow.contrast_with(colours.fill);
            assert!(
                contrast >= ENOUGH_FOR_A_SHAPE,
                "{scheme:?}: the arrow is {contrast:.2} from terracotta"
            );
        }
    }

    /// **The arrow stands apart from the fill in high contrast too**, where
    /// the fill is the other palette's terracotta rather than the design's —
    /// the same clause, measured on the palette a person who needs it sees.
    #[test]
    fn the_arrow_stands_apart_from_the_fill_in_high_contrast() {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let colours = MarkColours::of(scheme, Contrast::High);
            assert_eq!(
                colours.fill,
                alo_access::HighContrast::of(scheme).accent,
                "{scheme:?}: the mark is no longer the agent's colour in high contrast"
            );
            let contrast = colours.arrow.contrast_with(colours.fill);
            assert!(
                contrast >= ENOUGH_FOR_A_SHAPE,
                "{scheme:?}: the arrow is {contrast:.2} from the fill in high contrast"
            );
        }
    }

    /// **The mark's edge stands apart from the ground in high contrast**, so
    /// the square is a shape there as well as in the design.
    #[test]
    fn the_edge_stands_apart_from_the_ground_in_high_contrast() {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let colours = MarkColours::of(scheme, Contrast::High);
            let ground = colour(Contrast::High.ground(scheme));
            let contrast = colours.edge.contrast_with(ground);
            assert!(
                contrast >= ENOUGH_FOR_A_SHAPE,
                "{scheme:?}: the edge is {contrast:.2} from the ground in high contrast"
            );
        }
    }

    /// **The mark's edge stands apart from the ground it is drawn on**, so the
    /// square is a shape even where terracotta and the ground are close.
    #[test]
    fn the_edge_stands_apart_from_the_ground() {
        for (scheme, ground) in [
            (Scheme::Light, Token::Cream.colour()),
            (Scheme::Dark, Token::Charcoal.colour()),
        ] {
            let contrast = MarkColours::of(scheme, Contrast::AsDesigned)
                .edge
                .contrast_with(ground);
            assert!(
                contrast >= ENOUGH_FOR_TEXT,
                "{scheme:?}: the edge is {contrast:.2} from the ground"
            );
        }
    }

    /// **The mark is a shape with an arrow in it at every size**, and the arrow
    /// is inside the terracotta rather than beside it.
    #[test]
    fn the_arrow_is_inside_the_square_at_every_size() {
        for side in [1, 8, 18, 36, 54] {
            let solids = mark(10, 20, side, Scheme::Light, Contrast::AsDesigned);
            let square = solids.first().unwrap().area;
            let fill = solids.get(1).unwrap().area;
            let arrow = solids.get(2..).unwrap();
            assert!(arrow.len() >= 2, "{side}: no arrow");
            for part in arrow {
                assert_eq!(part.colour, rgb(Token::Navy.colour()));
                assert_eq!(
                    part.area.intersection(fill),
                    Some(part.area),
                    "{side}: {part:?} leaves the terracotta"
                );
            }
            assert_eq!(fill.intersection(square), Some(fill));
        }
    }
}
