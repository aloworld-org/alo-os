//! **alo OS's pointer, drawn by alo**: every cursor shape a desktop asks for,
//! from one vector source, in the palette's colours, at any display's scale.
//!
//! Until this crate the pointer was an 18-row mask of black and white pixels
//! whose own comment said it was *independent of the pending shell palette*,
//! drawn at one size on every display; and an application that wanted a text
//! cursor or a hand drew its own, so the pointer changed look at every window's
//! edge. `docs/features.md`, *The alo Cursor*, first line: *one cursor, drawn
//! by alo, in every application*.
//!
//! # What is here
//!
//! - [`Silhouette`] — which arrow the person's pointer is, in one place. The
//!   owner chose the precision kite on 2026-11-06; the familiar arrow is kept
//!   beside it as *Classic*.
//! - [`artwork`] — the one table from each name the cursor-shape protocol
//!   defines ([`EVERY_SHAPE`]) to the drawing alo answers with, and the
//!   drawings themselves as SVG path data on a 32-unit box.
//! - [`draw`] — one shape into a square of pixels: a `text/primary` navy shape,
//!   a white keyline and a soft shadow, premultiplied.
//! - [`theme`] and [`xcursor_file`] — the same drawings as an XCursor theme,
//!   for applications that draw their own pointer; written at build time by
//!   `alo-cursor-theme` and installed by the image.
//! - [`measures`] — the logical size and the artwork's own measures, which a
//!   caller passes through its display's scale ([`measures::pixels_at`]).
//!
//! Nothing here reads a file, opens a socket or knows about Wayland: the
//! compositor asks for pixels, and so can a build step that writes a theme.

pub mod artwork;
mod artwork_arrows;
mod artwork_hands;
mod artwork_marks;
pub mod drawing;
pub mod measures;
pub mod silhouette;
pub mod theme;
pub mod xcursor_file;

pub use artwork::{Drawing, EVERY_SHAPE, drawing_for, has_its_own_drawing};
pub use cursor_icon::CursorIcon;
pub use drawing::{LARGEST, NotDrawn, Picture, draw};
pub use silhouette::Silhouette;
