//! Prepare reusable pixels for the appearance-selected lock background.
use crate::{
    RenderError,
    lock_background_path::selected,
    lock_image_decode::{MAX_PIXELS, decode},
};
use alo_appearance::Background;
use std::{path::Path, time::Duration};

/// Prepared background pixels, reusable across clock ticks at the same output size.
/// No file path or client content is retained in the pixels.
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
    /// Resolve shipped names in the installed image and personal paths explicitly.
    /// `running` is elapsed rotation time; the appearance model chooses the index.
    /// # Errors
    /// Refuses missing/malformed images, oversized images/outputs, special files,
    /// and directories with more than 4096 entries. PNG and JPEG are supported.
    pub fn prepare(
        chosen: &Background,
        size: (i32, i32),
        running: Duration,
    ) -> Result<Self, RenderError> {
        Self::within(
            chosen,
            size,
            running,
            Path::new("/usr/share/alo/wallpapers"),
        )
    }

    /// Resolve names beneath an explicit installed-image root.
    pub(crate) fn within(
        chosen: &Background,
        size: (i32, i32),
        running: Duration,
        shipped: &Path,
    ) -> Result<Self, RenderError> {
        let (w, h) = size;
        if w <= 0 || h <= 0 || w > 8192 || h > 8192 || w as u64 * h as u64 > MAX_PIXELS {
            return Err(RenderError::LockScene);
        }
        let mut pixels = vec![0u8; w as usize * h as usize * 4];
        match chosen {
            Background::Colour(c) => {
                for pixel in pixels.as_chunks_mut::<4>().0 {
                    pixel.copy_from_slice(&[c.red(), c.green(), c.blue(), 255]);
                }
            }
            _ => {
                let (path, fitting) = selected(chosen, running, shipped)?;
                let picture = decode(&path)?;
                crate::lock_image_fit::paint(&picture, fitting, size, &mut pixels);
            }
        }
        Ok(Self {
            size,
            pixels,
            chosen: chosen.clone(),
        })
    }
}
