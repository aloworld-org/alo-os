//! Export a completed CPU image directly into owned, reusable readback bytes.

use crate::{ReadbackError, RenderError, ScanoutPixels};
use drm::buffer::DrmFourcc;
use pixman::{Image, Operation};

pub(super) fn capture(
    source: &Image<'_, '_>,
    retained: Option<Vec<u8>>,
) -> Result<ScanoutPixels, RenderError> {
    let width = u32::try_from(source.width()).map_err(|_| ReadbackError::Layout)?;
    let height = u32::try_from(source.height()).map_err(|_| ReadbackError::Layout)?;
    let (_, length) = crate::readback::layout((width, height))?;
    let mut pixels = match retained {
        Some(pixels) if pixels.len() == length => pixels,
        Some(_) => return Err(ReadbackError::Layout.into()),
        None => {
            let mut pixels = Vec::new();
            pixels
                .try_reserve_exact(length)
                .map_err(ReadbackError::from)?;
            pixels.resize(length, 0);
            pixels
        }
    };
    copy_into(source, &mut pixels)?;
    Ok(crate::readback::owned_xrgb((width, height), pixels)?)
}

fn copy_into(source: &Image<'_, '_>, pixels: &mut [u8]) -> Result<(), RenderError> {
    let width = u32::try_from(source.width()).map_err(|_| ReadbackError::Layout)?;
    let height = u32::try_from(source.height()).map_err(|_| ReadbackError::Layout)?;
    let (stride, length) = crate::readback::layout((width, height))?;
    if pixels.len() != length
        || source.stride() != stride
        || DrmFourcc::try_from(source.format()).ok() != Some(super::READBACK)
    {
        return Err(ReadbackError::Layout.into());
    }
    // The cast checks alignment and length before the engine receives its slice.
    let words =
        bytemuck::try_cast_slice_mut::<u8, u32>(pixels).map_err(|_| ReadbackError::Layout)?;
    let format =
        pixman::FormatCode::try_from(super::READBACK).map_err(|_| ReadbackError::Layout)?;
    let mut destination = Image::from_slice_mut(
        format,
        width as usize,
        height as usize,
        words,
        stride,
        false,
    )
    .map_err(|error| RenderError::Submission(format!("export software frame: {error}")))?;
    // This is a Pixman copy, not a renderer binding. Its temporary borrow ends
    // before bytes are normalized, published or returned to the reuse pool.
    destination.composite32(
        Operation::Src,
        source,
        None,
        (0, 0),
        (0, 0),
        (0, 0),
        (width as i32, height as i32),
    );
    Ok(())
}

#[cfg(test)]
#[path = "software_readback_tests.rs"]
mod tests;
