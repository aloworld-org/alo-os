//! Full-frame GLES export for unbound scanout upload, never an agent capture API.

use drm::buffer::DrmFourcc;
use smithay::{
    backend::renderer::{
        ExportMem, Texture, TextureMapping,
        gles::{GlesError, GlesRenderer, GlesTarget},
    },
    utils::Rectangle,
};

/// Location of the first row returned by GL readback in the intended output.
/// The caller knows the rendering transform; mapping metadata alone does not.
#[derive(Clone, Copy, Debug)]
pub enum RowOrder {
    /// First GL row is the output's top row (Smithay `Transform::Normal`).
    TopToBottom,
    /// First GL row is the output's bottom row (`Transform::Flipped180`).
    BottomToTop,
}

/// Readback refuses malformed layouts and retains upstream graphics failures.
#[derive(Debug, thiserror::Error)]
pub enum ReadbackError {
    /// Empty, oversized, unsupported or inconsistent mapping metadata/bytes.
    #[error("invalid scanout readback layout")]
    Layout,
    /// Copy or CPU mapping failed; no scanout candidate has been touched.
    #[error("GLES scanout readback: {0}")]
    Graphics(#[from] GlesError),
    /// The owned CPU frame could not be allocated.
    #[error("scanout readback allocation: {0}")]
    Allocation(#[from] std::collections::TryReserveError),
}

/// Owned, tightly packed top-to-bottom B,G,R,0 bytes ready for unbound upload.
/// Alpha is discarded after rendering; no compositing or colour conversion occurs.
pub struct ScanoutPixels {
    /// Validated full-target extent.
    size: (u32, u32),
    /// Tightly packed byte pitch.
    stride: usize,
    /// Immutable converted frame storage.
    pixels: Vec<u8>,
}

impl ScanoutPixels {
    /// Full physical extent of this immutable scene.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Borrow the validated source accepted by `DisplayResources::with_frame`.
    pub fn frame(&self) -> std::io::Result<crate::XrgbFrame<'_>> {
        crate::XrgbFrame::new(self.size, self.stride, &self.pixels)
    }

    /// Immutable CPU bytes; destination stride padding is added only at upload.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

/// Copy the entire bound GLES target into an owned scanout source.
///
/// Call after finishing drawing and before reusing/swapping the target. GL's
/// read-only mapping synchronizes the copy before bytes are accessed. The target
/// and renderer must belong to the same graphics context. `order` describes the
/// rendered output, not the mapping's inversion flag; no rotation is inferred.
/// This synchronous CPU path supplies unbound uploads, not presentation or flips.
/// It exposes no server request, agent verb, screenshot or background capture.
pub fn readback_xrgb(
    renderer: &mut GlesRenderer,
    target: &GlesTarget<'_>,
    order: RowOrder,
) -> Result<ScanoutPixels, ReadbackError> {
    let size = target.size();
    let dimensions = (
        u32::try_from(size.w).map_err(|_| ReadbackError::Layout)?,
        u32::try_from(size.h).map_err(|_| ReadbackError::Layout)?,
    );
    layout(dimensions)?;
    // RGBA/UNSIGNED_BYTE is the GLES readback baseline. Its DRM spelling is ABGR.
    let mapping =
        renderer.copy_framebuffer(target, Rectangle::from_size(size), DrmFourcc::Abgr8888)?;
    // Pinned GLES exports tightly packed rows and marks GL mappings inverted.
    // Refuse changed metadata rather than guessing a layout or silently flipping.
    validate_mapping(
        dimensions,
        (mapping.width(), mapping.height()),
        Texture::format(&mapping),
        mapping.flipped(),
    )?;
    convert(dimensions, order, renderer.map_texture(&mapping)?)
}

/// Check pinned export metadata before mapping its storage.
fn validate_mapping(
    expected: (u32, u32),
    actual: (u32, u32),
    format: Option<DrmFourcc>,
    flipped: bool,
) -> Result<(), ReadbackError> {
    if actual != expected
        || !flipped
        || !matches!(format, Some(DrmFourcc::Abgr8888 | DrmFourcc::Xbgr8888))
    {
        return Err(ReadbackError::Layout);
    }
    Ok(())
}

/// Protect upstream's `width * height * 4` signed i32 arithmetic before I/O.
pub(crate) fn layout(size: (u32, u32)) -> Result<(usize, usize), ReadbackError> {
    let bytes = size.0.checked_mul(size.1).and_then(|n| n.checked_mul(4));
    let Some(bytes) = bytes.filter(|n| *n > 0 && *n <= i32::MAX as u32) else {
        return Err(ReadbackError::Layout);
    };
    Ok((size.0 as usize * 4, bytes as usize))
}

/// Convert only complete, tightly packed four-channel mappings.
pub(crate) fn convert(
    size: (u32, u32),
    order: RowOrder,
    rgba: &[u8],
) -> Result<ScanoutPixels, ReadbackError> {
    let (stride, length) = layout(size)?;
    if rgba.len() != length {
        return Err(ReadbackError::Layout);
    }
    let mut pixels = Vec::new();
    pixels.try_reserve_exact(length)?;
    pixels.resize(length, 0);
    for (row, destination) in pixels.chunks_exact_mut(stride).enumerate() {
        let source_row = match order {
            RowOrder::TopToBottom => row,
            RowOrder::BottomToTop => size.1 as usize - 1 - row,
        };
        let source = rgba
            .get(source_row * stride..(source_row + 1) * stride)
            .ok_or(ReadbackError::Layout)?;
        for (source, destination) in source
            .as_chunks::<4>()
            .0
            .iter()
            .zip(destination.as_chunks_mut::<4>().0)
        {
            *destination = [source[2], source[1], source[0], 0];
        }
    }
    Ok(ScanoutPixels {
        size,
        stride,
        pixels,
    })
}

#[cfg(test)]
#[path = "readback_tests.rs"]
mod tests;
