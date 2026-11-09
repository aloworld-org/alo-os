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
    pixels: Pixels,
}

/// The complete immutable allocation, retained for every borrowed byte view.
#[derive(Clone)]
enum Pixels {
    /// Existing capture backends and GLES keep their owned byte vectors.
    Owned(std::sync::Arc<Vec<u8>>),
    /// Software rendering can publish its initialized image allocation directly.
    Mapped(std::sync::Arc<smithay::backend::renderer::pixman::PixmanMapping>),
}

/// Immutable native pixels whose allocation stays alive through every clone.
/// This additive capture owner supports both existing vectors and CPU mappings.
#[derive(Clone)]
pub struct SharedPixels(Pixels);

impl AsRef<[u8]> for SharedPixels {
    fn as_ref(&self) -> &[u8] {
        self.0.bytes()
    }
}

impl std::ops::Deref for SharedPixels {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl From<std::sync::Arc<Vec<u8>>> for SharedPixels {
    fn from(pixels: std::sync::Arc<Vec<u8>>) -> Self {
        Self(Pixels::Owned(pixels))
    }
}

impl SharedPixels {
    /// Preserve the older Vec-backed capture interface when a backend needs it.
    pub fn into_vec(self) -> std::sync::Arc<Vec<u8>> {
        match self.0 {
            Pixels::Owned(pixels) => pixels,
            Pixels::Mapped(pixels) => std::sync::Arc::new(pixels.as_ref().as_ref().to_vec()),
        }
    }
}

impl Pixels {
    /// Both owners expose the same validated native byte layout.
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Owned(pixels) => pixels,
            Self::Mapped(pixels) => pixels.as_ref().as_ref(),
        }
    }
}

impl ScanoutPixels {
    /// Full physical extent of this immutable scene.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Borrow the validated source accepted by `DisplayResources::with_frame`.
    pub fn frame(&self) -> std::io::Result<crate::XrgbFrame<'_>> {
        crate::XrgbFrame::new(self.size, self.stride, self.pixels.bytes())
    }

    /// Immutable CPU bytes; destination stride padding is added only at upload.
    pub fn pixels(&self) -> &[u8] {
        self.pixels.bytes()
    }

    /// Share the owned allocation without copying the frame's bytes.
    /// A consumer cannot mutate storage while scanout or another reader holds it.
    pub(crate) fn shared(&self) -> std::sync::Arc<Vec<u8>> {
        match &self.pixels {
            Pixels::Owned(pixels) => pixels.clone(),
            Pixels::Mapped(_) => std::sync::Arc::new(self.pixels.bytes().to_vec()),
        }
    }

    /// Keep the complete immutable storage without converting its owner type.
    pub(crate) fn retained(&self) -> SharedPixels {
        SharedPixels(self.pixels.clone())
    }

    /// The same extent and stride, carrying pixels drawn from these.
    ///
    /// The one door for a frame made out of another — the magnifier
    /// (`crate::access_magnifier`) is what asks — so the extent, the stride and
    /// the length stay one invariant held in this file. [`None`] for a length
    /// that is not this frame's, rather than a frame a display would read past
    /// the end of.
    pub(crate) fn redrawn(&self, pixels: Vec<u8>) -> Option<Self> {
        (pixels.len() == self.pixels.bytes().len()).then_some(Self {
            size: self.size,
            stride: self.stride,
            pixels: Pixels::Owned(std::sync::Arc::new(pixels)),
        })
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
    // Extend initialized four-byte pixels directly. Zeroing a byte vector first
    // writes the full frame twice; this exact-size iterator needs no such pass.
    let mut pixels: Vec<[u8; 4]> = Vec::new();
    pixels.try_reserve_exact(length / 4)?;
    for row in 0..size.1 as usize {
        let source_row = match order {
            RowOrder::TopToBottom => row,
            RowOrder::BottomToTop => size.1 as usize - 1 - row,
        };
        let source = rgba
            .get(source_row * stride..(source_row + 1) * stride)
            .ok_or(ReadbackError::Layout)?;
        pixels.extend(
            source
                .as_chunks::<4>()
                .0
                .iter()
                .map(|source| [source[2], source[1], source[0], 0]),
        );
    }
    Ok(ScanoutPixels {
        size,
        stride,
        pixels: Pixels::Owned(std::sync::Arc::new(pixels.into_flattened())),
    })
}

/// Normalize an owned native frame without allocating or copying its storage.
pub(crate) fn owned_xrgb(
    size: (u32, u32),
    mut pixels: Vec<u8>,
) -> Result<ScanoutPixels, ReadbackError> {
    let (stride, length) = layout(size)?;
    if pixels.len() != length {
        return Err(ReadbackError::Layout);
    }
    normalize_x(&mut pixels);
    Ok(ScanoutPixels {
        size,
        stride,
        pixels: Pixels::Owned(std::sync::Arc::new(pixels)),
    })
}

/// Normalize initialized image storage before sharing it with scanout readers.
pub(crate) fn mapped_xrgb(
    mut mapping: smithay::backend::renderer::pixman::PixmanMapping,
) -> Result<
    (
        ScanoutPixels,
        std::sync::Arc<smithay::backend::renderer::pixman::PixmanMapping>,
    ),
    ReadbackError,
> {
    let size = (Texture::width(&mapping), Texture::height(&mapping));
    let (stride, length) = layout(size)?;
    if Texture::format(&mapping) != Some(DrmFourcc::Xrgb8888) || mapping.as_ref().len() != length {
        return Err(ReadbackError::Layout);
    }
    normalize_x(mapping.as_mut());
    let mapping = std::sync::Arc::new(mapping);
    Ok((
        ScanoutPixels {
            size,
            stride,
            pixels: Pixels::Mapped(mapping.clone()),
        },
        mapping,
    ))
}

/// Clear only the unused channel, preserving the completed native colors.
fn normalize_x(pixels: &mut [u8]) {
    let (blocks, tail) = pixels.as_chunks_mut::<16>();
    for block in blocks {
        *block =
            (u128::from_le_bytes(*block) & 0x00ff_ffff_00ff_ffff_00ff_ffff_00ff_ffff).to_le_bytes();
    }
    for [_, _, _, unused] in tail.as_chunks_mut::<4>().0 {
        *unused = 0;
    }
}

#[cfg(test)]
#[path = "readback_tests.rs"]
mod tests;
