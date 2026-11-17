//! The words on the compositor's door for **alo taking one picture of the
//! window in front** — the screen road's picture
//! ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md)
//! §1) and `docs/contracts/application-door.md`, *`picture-of`*.
//!
//! ```text
//! → picture-of net.supertuxkart.SuperTuxKart
//! ← picture 48213 1280 800
//! ← <exactly 48213 bytes: the PNG>
//!   (or: not-in-front, not-alos-copy, could-not-draw)
//! ```
//!
//! **One application and nothing else.** The line has no place for a position,
//! a region or a second window: the compositor draws the window that holds the
//! keyboard, if it is that application's, alone. **The length is said, not the
//! end**, because a PNG holds every byte value; the reader reads exactly that
//! many, and never more than [`Pictured::THE_MOST`].

use alo_applications::Application;

use crate::request::Request;

/// The word that asks for a picture.
pub const PICTURE_OF: &str = "picture-of";

/// alo asking for one picture of the window in front.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToPicture {
    /// The application whose window must have the keyboard.
    pub application: Application,
}

impl ToPicture {
    /// The line, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        String::new()
    }

    /// One line, read — [`None`] when it is not about a picture at all, so the
    /// door can read it as one of the other acts.
    #[must_use]
    pub fn read(line: &str) -> Option<Result<Self, NotToPicture>> {
        let _ = line;
        None
    }
}

/// Why a line that began as a picture is not one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotToPicture {
    /// Longer than any line on this door may be.
    #[error("a line on this door is at most {} bytes", Request::THE_MOST)]
    TooLong,
    /// It named no application, more than one, one no verb could name, or
    /// something after it.
    #[error("a picture names exactly one application, by its identifier, and nothing else")]
    NotOneApplication,
}

/// What the door answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pictured {
    /// A PNG of this many bytes follows, of a window this wide and tall in
    /// pixels.
    Picture {
        /// How many bytes of PNG follow the line.
        bytes: usize,
        /// How wide the picture is, in pixels.
        wide: u32,
        /// How tall the picture is, in pixels.
        tall: u32,
    },
    /// The window that has the keyboard is not one of that application's, or
    /// no window has it. No picture was taken.
    NotInFront,
    /// It is the person's own window of that application, and at their setting
    /// alo works only in its own copy. No picture was taken.
    NotAlosCopy,
    /// The window was that application's and could not be drawn. No picture
    /// was taken.
    CouldNotDraw,
}

impl Pictured {
    /// The most bytes one picture may be: a window larger than any display
    /// alo OS lays out is still under it as a PNG, and a reader never holds
    /// more.
    pub const THE_MOST: usize = 32 * 1024 * 1024;

    /// The words on the wire: for a picture, only its head.
    #[must_use]
    pub fn written(self) -> String {
        String::new()
    }

    /// One answer's line, read — [`None`] for a line that is not one.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        let _ = line;
        None
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "picturing_tests.rs"]
mod tests;
