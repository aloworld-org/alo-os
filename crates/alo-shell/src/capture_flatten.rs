//! Burning what a person hid into the picture they save.
//!
//! `alo_capturing::Marks::saving` decides **which** areas are hidden and
//! demands they be burnt into the pixels; it holds no pixels of its own and
//! hands the work here. This file is that renderer, and what comes out of it is
//! what a person's colleague will open.
//!
//! # It is not a blur, and that is the point
//!
//! A blur is a filter, and a filter can be undone: the detail is still in the
//! picture, spread out, and enough of it comes back with the right kernel that
//! people have read credit-card numbers off blurred screenshots. A *pixelation*
//! is worse still, because each block is the average of a small square and the
//! original can often be guessed from the blocks alone.
//!
//! So a hidden area is replaced by **one flat colour — the average of what was
//! there** — and the bytes underneath it are gone from the file. What a person
//! asked to hide is hidden, not obscured, and nothing about the strength of a
//! kernel stands between them and that.
//!
//! The average rather than black, because a redaction that reads as part of the
//! picture is one a person can see they made: a flat block the colour of the
//! thing it covers is unmistakably deliberate, and tells a reader something was
//! removed without shouting.
//!
//! # A hidden area that hid nothing is a refusal
//!
//! If an area falls entirely outside the picture, nothing would be burnt in and
//! the file would be saved with the thing still in it. That is refused rather
//! than saved: a person who marked something to hide and got a picture with it
//! showing has been failed in the one way this feature exists to prevent.

use std::io::Cursor;

use alo_capturing::{Picture, Region};
use image::{ImageFormat, ImageReader, RgbaImage};

/// Why a picture could not be saved with what was hidden burnt into it.
#[derive(Debug, thiserror::Error)]
pub enum NotFlattened {
    /// The bytes that came back from the screen are not a picture this machine
    /// can read.
    #[error("the picture taken could not be read: {0}")]
    Unreadable(String),
    /// The picture could not be written back out.
    #[error("the picture could not be written: {0}")]
    Unwritable(String),
    /// An area a person marked to hide is not on the picture at all, so
    /// nothing would have been hidden.
    #[error(
        "an area marked to hide is not on this picture: {width}×{height} at \
         {left},{top} is outside {across}×{down}"
    )]
    HidNothing {
        /// How far in the area starts.
        left: u32,
        /// How far down it starts.
        top: u32,
        /// How wide it is.
        width: u32,
        /// How tall it is.
        height: u32,
        /// The picture's width.
        across: u32,
        /// The picture's height.
        down: u32,
    },
}

/// The picture to save, with every hidden area replaced by one flat colour.
///
/// Hand this to [`alo_capturing::Marks::saving`], which supplies the areas.
///
/// # Errors
/// [`NotFlattened`]: a picture that cannot be read or written, and an area that
/// would have hidden nothing.
pub fn burnt_in(as_taken: &Picture, hidden: &[Region]) -> Result<Picture, NotFlattened> {
    let mut picture = ImageReader::new(Cursor::new(as_taken.bytes()))
        .with_guessed_format()
        .map_err(|why| NotFlattened::Unreadable(why.to_string()))?
        .decode()
        .map_err(|why| NotFlattened::Unreadable(why.to_string()))?
        .into_rgba8();
    for region in hidden {
        hide(&mut picture, *region)?;
    }
    let mut bytes = Vec::new();
    picture
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|why| NotFlattened::Unwritable(why.to_string()))?;
    Picture::of(bytes).map_err(|why| NotFlattened::Unwritable(why.to_string()))
}

/// Replace one area with the average of what was in it.
fn hide(picture: &mut RgbaImage, region: Region) -> Result<(), NotFlattened> {
    let (across, down) = (picture.width(), picture.height());
    let left = region.from_the_left();
    let top = region.from_the_top();
    let right = left.saturating_add(region.width()).min(across);
    let bottom = top.saturating_add(region.height()).min(down);
    if left >= across || top >= down || right <= left || bottom <= top {
        return Err(NotFlattened::HidNothing {
            left,
            top,
            width: region.width(),
            height: region.height(),
            across,
            down,
        });
    }
    let mut total = [0u64; 4];
    let mut counted = 0u64;
    for y in top..bottom {
        for x in left..right {
            let pixel = picture.get_pixel(x, y).0;
            for (sum, channel) in total.iter_mut().zip(pixel) {
                *sum += u64::from(channel);
            }
            counted += 1;
        }
    }
    let counted = counted.max(1);
    let average = image::Rgba(total.map(|sum| u8::try_from(sum / counted).unwrap_or(u8::MAX)));
    for y in top..bottom {
        for x in left..right {
            picture.put_pixel(x, y, average);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "capture_flatten_tests.rs"]
mod tests;
