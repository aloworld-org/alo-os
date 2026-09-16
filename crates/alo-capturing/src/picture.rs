//! The picture itself: bytes, and what form they are in.
//!
//! What came back from the rented screen-capture mechanism, on its way to a
//! file or to the clipboard. Nothing in this crate looks inside it, decodes it,
//! resizes it, or asks it anything — which is why the file's name can promise
//! to say nothing about what was on the screen, and why nothing here can
//! accidentally become an image library.
//!
//! # One form, and it is the clipboard's own spelling
//!
//! A picture of a screen is a lossless image, and [`Kind::image_png`] is how
//! `alo-clipboard` already spells one. Using that spelling rather than a second
//! one of this crate's own is what makes a picture handed to the clipboard the
//! same value as a picture handed to a file: there is no conversion anywhere
//! between the two roads, because there is nothing to convert.
//!
//! # A picture with no bytes in it is not a picture
//!
//! [`Picture::of`] refuses one. The rented mechanism answering with nothing is
//! a failure of the mechanism rather than a very small picture, and a person
//! who was told *saved* and later opened an empty file would have been told
//! something untrue at the one moment they were paying attention.

use alo_clipboard::Kind;

use crate::grabs::NotGrabbed;

/// A picture of the screen, as the rented mechanism handed it over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    /// The bytes themselves.
    bytes: Vec<u8>,
}

impl Picture {
    /// The picture these bytes are.
    ///
    /// # Errors
    /// [`NotGrabbed::NothingCameBack`] for no bytes at all. A mechanism that
    /// answered with nothing has failed, and calling that a picture would put
    /// an empty file in somebody's folder under a message saying it was saved.
    pub fn of(bytes: Vec<u8>) -> Result<Self, NotGrabbed> {
        if bytes.is_empty() {
            return Err(NotGrabbed::NothingCameBack {
                said: "the picture came back with no bytes in it".to_owned(),
            });
        }
        Ok(Self { bytes })
    }

    /// The form it is in, as the clipboard spells one.
    #[must_use]
    pub fn form(&self) -> Kind {
        Kind::image_png()
    }

    /// The bytes themselves.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// How many bytes it is.
    #[must_use]
    pub fn how_many_bytes(&self) -> usize {
        self.bytes.len()
    }

    /// The bytes, taken.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A picture is its bytes**, handed on exactly as they arrived: nothing
    /// here re-encodes, and nothing here looks inside.
    #[test]
    fn a_picture_is_the_bytes_that_came_back() {
        let picture = Picture::of(b"\x89PNG\r\n\x1a\nand the rest".to_vec()).unwrap();
        assert!(picture.bytes().starts_with(b"\x89PNG"));
        assert_eq!(picture.how_many_bytes(), 20);
        assert_eq!(picture.clone().into_bytes(), picture.bytes());
    }

    /// **A picture is in the form the clipboard already spells**, so a picture
    /// going to a file and a picture going to the clipboard are one value with
    /// nothing converting between them.
    #[test]
    fn a_picture_is_in_the_form_the_clipboard_already_spells() {
        let picture = Picture::of(b"\x89PNG".to_vec()).unwrap();
        assert_eq!(picture.form(), Kind::image_png());
        assert_eq!(picture.form().media(), "image/png");
    }

    /// **Nothing came back is a failure, not an empty picture.** Somebody told
    /// their picture was saved, who then opened an empty file, was told
    /// something untrue at the moment they were paying attention.
    #[test]
    fn a_picture_with_no_bytes_is_a_failure_rather_than_a_small_picture() {
        let refused = Picture::of(Vec::new()).unwrap_err();
        assert!(
            matches!(refused, NotGrabbed::NothingCameBack { .. }),
            "{refused:?}"
        );
        assert!(refused.diagnosis().contains("no bytes"));
    }
}
