//! Upload bounded opaque native text as one sparse raster, preserving its order.

use crate::{
    RenderError,
    painted::{Inked, Washed},
};
use smithay::{
    backend::renderer::{ImportMem, Renderer},
    utils::Rectangle,
};

/// Larger or malformed collections retain the existing pixel-run painter.
const MAX_BYTES: usize = 8 * 1024 * 1024;

/// One imported raster and its original destination in output pixels.
pub(crate) type Uploaded<T> = (T, Rectangle<i32, smithay::utils::Physical>);

/// Import before opening the frame; no texture outlives its owning renderer.
pub(crate) fn upload<R: Renderer + ImportMem>(
    renderer: &mut R,
    inked: &[Inked],
) -> Result<Option<Uploaded<R::TextureId>>, RenderError> {
    let Some(raster) = raster(inked)? else {
        return Ok(None);
    };
    renderer
        .import_memory(
            &raster.pixels,
            smithay::backend::allocator::Fourcc::Abgr8888,
            (raster.area.size.w, raster.area.size.h).into(),
            false,
        )
        .map(|texture| Some((texture, raster.area)))
        .map_err(|error| RenderError::Submission(format!("upload native ink: {error}")))
}

/// All gaps are transparent; each original opaque image replaces earlier ink.
fn raster(inked: &[Inked]) -> Result<Option<Washed>, RenderError> {
    let Some(first) = inked.first() else {
        return Ok(None);
    };
    let mut area = first.area;
    for ink in inked {
        let (Ok(w), Ok(h)) = (
            usize::try_from(ink.area.size.w),
            usize::try_from(ink.area.size.h),
        ) else {
            return Ok(None);
        };
        if w == 0 || h == 0 || w.checked_mul(h) != Some(ink.pixels.len()) {
            return Ok(None);
        }
        let Some(right) = ink.area.loc.x.checked_add(ink.area.size.w) else {
            return Ok(None);
        };
        let Some(bottom) = ink.area.loc.y.checked_add(ink.area.size.h) else {
            return Ok(None);
        };
        let Some(old_right) = area.loc.x.checked_add(area.size.w) else {
            return Ok(None);
        };
        let Some(old_bottom) = area.loc.y.checked_add(area.size.h) else {
            return Ok(None);
        };
        let left = area.loc.x.min(ink.area.loc.x);
        let top = area.loc.y.min(ink.area.loc.y);
        let Some(width) = old_right.max(right).checked_sub(left) else {
            return Ok(None);
        };
        let Some(height) = old_bottom.max(bottom).checked_sub(top) else {
            return Ok(None);
        };
        area = Rectangle::new((left, top).into(), (width, height).into());
    }
    let (Ok(width), Ok(height)) = (usize::try_from(area.size.w), usize::try_from(area.size.h))
    else {
        return Ok(None);
    };
    let Some(bytes) = width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(4))
        .filter(|n| *n <= MAX_BYTES)
    else {
        return Ok(None);
    };
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(bytes)
        .map_err(|error| RenderError::Submission(format!("allocate native ink: {error}")))?;
    pixels.resize(bytes, 0);
    for ink in inked {
        let (Ok(x), Ok(y), Ok(w)) = (
            usize::try_from(ink.area.loc.x - area.loc.x),
            usize::try_from(ink.area.loc.y - area.loc.y),
            usize::try_from(ink.area.size.w),
        ) else {
            return Ok(None);
        };
        for (row, source) in ink.pixels.chunks(w).enumerate() {
            let start = ((y + row) * width + x) * 4;
            let Some(target) = pixels.get_mut(start..start + w * 4) else {
                return Ok(None);
            };
            for ([red, green, blue], target) in source.iter().zip(target.as_chunks_mut::<4>().0) {
                target.copy_from_slice(&[*red, *green, *blue, 255]);
            }
        }
    }
    Ok(Some(Washed { area, pixels }))
}

#[cfg(test)]
#[path = "native_inked_tests.rs"]
mod tests;
