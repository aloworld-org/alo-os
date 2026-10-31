//! **Small vector artwork, read from the design file's own SVG and drawn sharp
//! at whatever size a display asks for.**
//!
//! The task window's reply box drew three grey boxes and an empty circle where
//! the design has an Attach paperclip, a chevron, a Permissions shield and a
//! Send arrow or Stop square. The artwork had never been committed, and nothing
//! in the shell could draw a curve that was not a letter. The owner's decision
//! of 2026-10-30: take the icons from the design file as SVG, keep them in the
//! repository, and draw them as vectors so they stay sharp at every scale.
//!
//! # What it reads, and what it refuses
//!
//! **A small part of SVG, read strictly**: one `<svg>` with a `viewBox` from
//! zero, any number of `<g>` groups, and `<path>` elements filled or stroked in
//! one colour, with SVG's caps and joins. That is everything the design file
//! exports for these icons. Anything else — a transform, a gradient, text, a
//! second kind of element, an attribute this crate does not read — is
//! **refused with its name** rather than skipped, because a picture drawn with
//! part of it silently missing is a picture that looks measured and is not.
//! [`reading`] has the rules.
//!
//! # What it draws
//!
//! **Coverage, not colour.** [`AShape::coverage`] answers how much of each
//! pixel the artwork covers, for a box of any size in pixels. The caller inks
//! it in the colour its own palette says, so the artwork follows the person's
//! appearance and contrast settings rather than the hex the file was exported
//! with. The colours in the file are kept ([`AShape::colours`]) so a test can
//! hold them to the palette the design names.

pub mod drawing;
pub mod reading;

pub use reading::{AShape, NotAShape};
