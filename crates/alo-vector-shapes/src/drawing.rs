//! Drawing artwork into a box of pixels, as coverage.
//!
//! **The file's `viewBox` is stretched to the box it is drawn in**, and every
//! stroke widens with it: a 28-unit icon drawn into 56 pixels on a display at
//! 200 per cent has strokes 4 pixels wide, which is the same icon drawn sharper
//! rather than a thinner one. The pixels come from `zeno`, the rasteriser that
//! already draws every letter the shell shows, anti-aliased at 256 levels.
//!
//! Where two paths overlap they are combined the way one colour painted over
//! itself is — `a + b × (1 − a)` — so an overlap is never darker than either.

use zeno::{Fill, Mask, Stroke, Transform};

use crate::reading::{APart, AShape, Cap, Join, Paint};

impl AShape {
    /// How much of each pixel the artwork covers, drawn into a box `wide` by
    /// `tall` pixels: `wide × tall` bytes, row by row, from 0 (nothing) to 255
    /// (all of it).
    ///
    /// A box with no pixels answers an empty list.
    #[must_use]
    pub fn coverage(&self, wide: u32, tall: u32) -> Vec<u8> {
        let mut covered = vec![0_u8; pixels_in(wide, tall)];
        for part in &self.parts {
            let this = coverage_of_one(self, part, wide, tall);
            for (all, one) in covered.iter_mut().zip(this) {
                *all = together(*all, one);
            }
        }
        covered
    }
}

/// How many pixels a box holds, as a length.
fn pixels_in(wide: u32, tall: u32) -> usize {
    usize::try_from(u64::from(wide) * u64::from(tall)).unwrap_or(0)
}

/// One path's coverage, drawn into the box.
pub(crate) fn coverage_of_one(shape: &AShape, part: &APart, wide: u32, tall: u32) -> Vec<u8> {
    let mut covered = vec![0_u8; pixels_in(wide, tall)];
    if covered.is_empty() {
        return covered;
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "a box of pixels is far below the size where f32 loses whole pixels"
    )]
    let (across, down) = (wide as f32 / shape.wide, tall as f32 / shape.tall);
    let mut mask = Mask::new(part.d.as_str());
    mask.size(wide, tall)
        .transform(Some(Transform::scale(across, down)));
    match part.paint {
        Paint::Filled => {
            mask.style(Fill::NonZero);
        }
        Paint::Stroked { width, cap, join } => {
            let mut stroke = Stroke::new(width);
            stroke.cap(cap_of(cap)).join(join_of(join));
            mask.style(stroke);
        }
    }
    mask.render_into(&mut covered, None);
    covered
}

/// The rasteriser's own name for a cap.
const fn cap_of(cap: Cap) -> zeno::Cap {
    match cap {
        Cap::Butt => zeno::Cap::Butt,
        Cap::Round => zeno::Cap::Round,
        Cap::Square => zeno::Cap::Square,
    }
}

/// The rasteriser's own name for a join.
const fn join_of(join: Join) -> zeno::Join {
    match join {
        Join::Miter => zeno::Join::Miter,
        Join::Round => zeno::Join::Round,
        Join::Bevel => zeno::Join::Bevel,
    }
}

/// Two coverages of one colour, one painted over the other.
fn together(under: u8, over: u8) -> u8 {
    let (under, over) = (u16::from(under), u16::from(over));
    let both = over + under * (255 - over) / 255;
    u8::try_from(both).unwrap_or(u8::MAX)
}

#[cfg(test)]
#[path = "drawing_tests.rs"]
mod tests;
