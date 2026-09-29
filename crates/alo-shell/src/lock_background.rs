//! The lock screen's surface, as reusable pixels.
//!
//! **alo OS ships no wallpaper** (ADR 0075), so this is a colour fill and the
//! bound on how large a surface it will allocate for. It used to resolve a
//! shipped name or a personal path, decode a PNG or a JPEG, and fit it five
//! ways; all of that went with the picture.
//!
//! What the record replaced that rule with is the part that mattered and is
//! still true here: **the lock screen shows the surface and no client's
//! pixels**. Nothing in this file can reach a client, because nothing in it
//! reaches anything at all.

use alo_appearance::Background;

use crate::RenderError;

/// The largest surface this will allocate pixels for.
///
/// About the screen rather than about any image — it was
/// `lock_image_decode::MAX_PIXELS` until ADR 0075 removed that file, and the
/// question it answers outlived the pictures it was written for.
pub(crate) const MAX_PIXELS: u64 = 16_777_216;

/// The largest side of a surface this will fit.
pub(crate) const LARGEST_SIDE: i32 = 8192;

/// Whether an output is one this will allocate for.
pub(crate) fn a_size_worth_painting(size: (i32, i32)) -> bool {
    let (width, height) = size;
    width > 0
        && height > 0
        && width <= LARGEST_SIDE
        && height <= LARGEST_SIDE
        && u64::from(width.unsigned_abs()) * u64::from(height.unsigned_abs()) <= MAX_PIXELS
}

/// Prepared surface pixels, reusable across clock ticks at the same size.
///
/// No file path and no client content is retained in them, which is now true by
/// construction rather than by care.
#[derive(Clone)]
pub struct LockBackground {
    /// Validated output extent.
    pub(crate) size: (i32, i32),
    /// Opaque RGBA bytes in top-to-bottom row order.
    pub(crate) pixels: Vec<u8>,
    /// The appearance choice whose pixels these are.
    pub(crate) chosen: Background,
}

impl LockBackground {
    /// This surface, at this size.
    ///
    /// # Errors
    /// [`RenderError::LockScene`] for an output with no pixels, or more of them
    /// than this will allocate.
    pub fn prepare(chosen: &Background, size: (i32, i32)) -> Result<Self, RenderError> {
        if !a_size_worth_painting(size) {
            return Err(RenderError::LockScene);
        }
        let (width, height) = size;
        let colour = chosen.colour();
        let mut pixels = vec![0u8; width as usize * height as usize * 4];
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel.copy_from_slice(&[colour.red(), colour.green(), colour.blue(), 255]);
        }
        Ok(Self {
            size,
            pixels,
            chosen: *chosen,
        })
    }
}
