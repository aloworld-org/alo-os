//! **One cursor as an XCursor file**: the format every Linux toolkit loads a
//! cursor theme from when it draws its own pointer.
//!
//! An application that does not ask the compositor for a shape by name — an
//! older GTK or Qt, anything under XWayland — reads `cursors/<name>` from the
//! theme `XCURSOR_THEME` names and draws those pixels itself. Writing alo's own
//! drawings in this format is what makes those applications show alo's pointer
//! too (`docs/features.md`, *The alo Cursor*: *an application that brings its own
//! images is given alo's cursor theme to draw from*).
//!
//! # The format, as libXcursor reads it
//!
//! Every number is a little-endian `u32`:
//!
//! - a header: `Xcur`, its own length (16), the file version (`0x1_0000`) and
//!   how many entries follow;
//! - one table entry per image: its type (`0xfffd_0002`, an image), its nominal
//!   size, and where in the file it starts;
//! - each image: its header length (36), the type again, the nominal size, the
//!   image version (1), width, height, the hotspot's x and y, a delay (nought,
//!   because none of these is animated), then `width × height` pixels, each
//!   `A R G B` from the high byte down, **premultiplied** — which is what
//!   [`crate::Picture`] already is.
//!
//! A toolkit picks the image whose nominal size is nearest what it wants, so one
//! file carries the cursor at every size a display might ask for.

use crate::Picture;

/// The file's first four bytes.
const MAGIC: [u8; 4] = *b"Xcur";
/// The length of the file header.
const FILE_HEADER: u32 = 16;
/// The file format's version.
const FILE_VERSION: u32 = 0x1_0000;
/// The table type of an image.
const AN_IMAGE: u32 = 0xfffd_0002;
/// The length of an image's header.
const IMAGE_HEADER: u32 = 36;
/// The image format's version.
const IMAGE_VERSION: u32 = 1;
/// The length of one table entry.
const TABLE_ENTRY: u32 = 12;

/// Why an XCursor file was not written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotWritten {
    /// No images were given: a cursor file with nothing in it draws nothing.
    NoImages,
    /// Two images claim the same nominal size, and a toolkit would pick either.
    TheSameSizeTwice(u32),
    /// A picture's pixels are not its size's worth.
    NotItsSize(u32),
    /// The file would be larger than its own offsets can address.
    TooLarge,
}

impl core::fmt::Display for NotWritten {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoImages => write!(f, "a cursor file was asked for with no images in it"),
            Self::TheSameSizeTwice(size) => {
                write!(f, "two images in one cursor file are both size {size}")
            }
            Self::NotItsSize(size) => {
                write!(f, "a picture of size {size} does not hold that many pixels")
            }
            Self::TooLarge => write!(f, "a cursor file would be larger than 4 GiB"),
        }
    }
}

impl std::error::Error for NotWritten {}

/// `pictures` as one XCursor file, each at its own size, smallest first.
///
/// # Errors
/// [`NotWritten`] for no pictures, two of one size, a picture whose pixels do
/// not match its size, or a file too large to address.
pub fn xcursor_file(pictures: &[Picture]) -> Result<Vec<u8>, NotWritten> {
    if pictures.is_empty() {
        return Err(NotWritten::NoImages);
    }
    let mut ordered: Vec<&Picture> = pictures.iter().collect();
    ordered.sort_by_key(|picture| picture.size);
    for pair in ordered.windows(2) {
        if let [one, other] = pair
            && one.size == other.size
        {
            return Err(NotWritten::TheSameSizeTwice(one.size));
        }
    }
    let count = u32::try_from(ordered.len()).map_err(|_| NotWritten::TooLarge)?;
    let mut file = Vec::new();
    file.extend_from_slice(&MAGIC);
    for number in [FILE_HEADER, FILE_VERSION, count] {
        file.extend_from_slice(&number.to_le_bytes());
    }
    let mut at = FILE_HEADER
        .checked_add(count.checked_mul(TABLE_ENTRY).ok_or(NotWritten::TooLarge)?)
        .ok_or(NotWritten::TooLarge)?;
    for picture in &ordered {
        for number in [AN_IMAGE, picture.size, at] {
            file.extend_from_slice(&number.to_le_bytes());
        }
        let pixels = picture
            .size
            .checked_mul(picture.size)
            .and_then(|area| area.checked_mul(4))
            .ok_or(NotWritten::TooLarge)?;
        at = at
            .checked_add(IMAGE_HEADER)
            .and_then(|end| end.checked_add(pixels))
            .ok_or(NotWritten::TooLarge)?;
    }
    for picture in ordered {
        let expected = usize::try_from(picture.size)
            .ok()
            .and_then(|side| side.checked_mul(side))
            .and_then(|area| area.checked_mul(4));
        if expected != Some(picture.pixels.len()) {
            return Err(NotWritten::NotItsSize(picture.size));
        }
        let (x, y) = picture.hotspot;
        for number in [
            IMAGE_HEADER,
            AN_IMAGE,
            picture.size,
            IMAGE_VERSION,
            picture.size,
            picture.size,
            x,
            y,
            0,
        ] {
            file.extend_from_slice(&number.to_le_bytes());
        }
        for pixel in picture.pixels.chunks_exact(4) {
            if let [red, green, blue, alpha] = pixel {
                let argb = u32::from_be_bytes([*alpha, *red, *green, *blue]);
                file.extend_from_slice(&argb.to_le_bytes());
            }
        }
    }
    Ok(file)
}

#[cfg(test)]
#[path = "xcursor_file_tests.rs"]
mod tests;
