//! Decode bounded ordinary PNG/JPEG files into opaque pixels.
use crate::RenderError;
use image::{ImageReader, Limits, RgbImage};
use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
};
/// Maximum compressed input, including metadata.
const MAX_BYTES: u64 = 32 * 1024 * 1024;
/// Maximum decoded or output pixel count before allocating.
pub(crate) const MAX_PIXELS: u64 = 16_777_216;
/// Read a bounded ordinary file and refuse unsupported dimensions before decoding.
pub(crate) fn decode(path: &Path) -> Result<RgbImage, RenderError> {
    use rustix::fs::{Mode, OFlags};
    // Read-only at the OS boundary; never create or modify a selected image.
    // NONBLOCK prevents a selected FIFO from hanging before its type is refused.
    let file = File::from(
        rustix::fs::open(
            path,
            OFlags::RDONLY | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(image_error)?,
    );
    let metadata = file.metadata().map_err(image_error)?;
    if !metadata.is_file() || metadata.len() > MAX_BYTES {
        return Err(RenderError::LockScene);
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(image_error)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(RenderError::LockScene);
    }
    let reader = ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(image_error)?;
    let (w, h) = reader.into_dimensions().map_err(image_error)?;
    if w == 0 || h == 0 || u64::from(w) * u64::from(h) > MAX_PIXELS {
        return Err(RenderError::LockScene);
    }
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(image_error)?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    // Composite transparency against opaque black so no client can show through.
    let rgba = reader.decode().map_err(image_error)?.to_rgba8();
    Ok(RgbImage::from_fn(w, h, |x, y| {
        let [red, green, blue, alpha] = rgba.get_pixel(x, y).0;
        image::Rgb([
            ((u16::from(red) * u16::from(alpha)) / 255) as u8,
            ((u16::from(green) * u16::from(alpha)) / 255) as u8,
            ((u16::from(blue) * u16::from(alpha)) / 255) as u8,
        ])
    }))
}
/// Keep diagnostic image errors out of the translated UI.
fn image_error(error: impl std::fmt::Display) -> RenderError {
    RenderError::Submission(format!("lock background: {error}"))
}
