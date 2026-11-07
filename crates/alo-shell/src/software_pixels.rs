//! Owned, initialized pixels borrowed by Pixman only while a frame is drawn.

use crate::{ReadbackError, RenderError, ScanoutPixels};

/// One detached CPU allocation; no device or renderer owns its storage.
pub(super) struct Scratch {
    size: (u32, u32),
    stride: usize,
    words: Vec<u32>,
}

impl Scratch {
    pub(super) fn new(size: (i32, i32)) -> Result<Self, ReadbackError> {
        let width = u32::try_from(size.0).map_err(|_| ReadbackError::Layout)?;
        let height = u32::try_from(size.1).map_err(|_| ReadbackError::Layout)?;
        let (stride, bytes) = crate::readback::layout((width, height))?;
        let mut words = Vec::new();
        words.try_reserve_exact(bytes / 4)?;
        words.resize(bytes / 4, 0);
        Ok(Self {
            size: (width, height),
            stride,
            words,
        })
    }

    pub(super) fn width(&self) -> usize {
        self.size.0 as usize
    }
    pub(super) fn height(&self) -> usize {
        self.size.1 as usize
    }
    pub(super) fn bytes(&self) -> usize {
        self.words.capacity() * 4
    }

    /// The exclusive slice borrow ends before readback or reuse is possible.
    pub(super) fn image(&mut self) -> Result<pixman::Image<'_, 'static>, RenderError> {
        let format =
            pixman::FormatCode::try_from(super::READBACK).map_err(|_| ReadbackError::Layout)?;
        let image = pixman::Image::from_slice_mut(
            format,
            self.width(),
            self.height(),
            &mut self.words,
            self.stride,
            false,
        )
        .map_err(|error| RenderError::Submission(format!("bind owned software pixels: {error}")))?;
        if image.width() != self.size.0 as usize
            || image.height() != self.size.1 as usize
            || image.stride() != self.stride
            || image.format() != format
        {
            return Err(ReadbackError::Layout.into());
        }
        Ok(image)
    }

    pub(super) fn readback(&self) -> Result<ScanoutPixels, ReadbackError> {
        crate::readback::copy_words(self.size, &self.words)
    }

    #[cfg(test)]
    pub(super) fn as_ptr(&self) -> *const u32 {
        self.words.as_ptr()
    }
}

#[cfg(test)]
#[path = "software_pixels_tests.rs"]
mod tests;
