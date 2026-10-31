//! **Vector artwork inked into a raster**, at whatever size its box is in the
//! display's pixels.
//!
//! One road for every surface that draws the design's own SVG: the reply box's
//! icons (`crate::the_reply_boxs_icons`) and the egress indicator's arrow
//! (`crate::egress_status_raster`). It was the reply box's alone until the
//! indicator needed it on 2026-10-31, and two copies of the blend would be two
//! answers to how an edge pixel is coloured.

use alo_vector_shapes::AShape;
use smithay::utils::{Physical, Rectangle};

use crate::painted::Washed;

/// Ink `shape` into `panel`, stretched to `at` (in the display's pixels), in
/// `ink`.
///
/// The shape's coverage is blended over what is already there, so its edges
/// are smooth against whatever ground it lands on. Pixels outside the raster
/// are left alone: artwork on a surface running off the screen is cut rather
/// than refused.
pub(crate) fn ink_a_shape(
    panel: &mut Washed,
    shape: &AShape,
    at: Rectangle<i32, Physical>,
    ink: [u8; 3],
) {
    let (Ok(wide_is), Ok(tall_is)) = (u32::try_from(at.size.w), u32::try_from(at.size.h)) else {
        return;
    };
    let covered = shape.coverage(wide_is, tall_is);
    let (wide, side) = (panel.area.size.w, at.size.w.max(1));
    for (index, coverage) in covered.into_iter().enumerate() {
        if coverage == 0 {
            continue;
        }
        let Ok(index) = i32::try_from(index) else {
            return;
        };
        let (x, y) = (
            at.loc.x + index % side - panel.area.loc.x,
            at.loc.y + index / side - panel.area.loc.y,
        );
        if x < 0 || y < 0 || x >= wide || y >= panel.area.size.h {
            continue;
        }
        let Ok(byte) = usize::try_from((y * wide + x) * 4) else {
            continue;
        };
        if let Some(pixel) = panel.pixels.get_mut(byte..byte + 3) {
            for (channel, inked) in pixel.iter_mut().zip(ink) {
                *channel = blended(*channel, inked, coverage);
            }
        }
    }
}

/// `under` with `over` painted on it at this coverage.
fn blended(under: u8, over: u8, coverage: u8) -> u8 {
    let (under, over, coverage) = (u32::from(under), u32::from(over), u32::from(coverage));
    let mixed = (over * coverage + under * (255 - coverage) + 127) / 255;
    u8::try_from(mixed).unwrap_or(u8::MAX)
}
