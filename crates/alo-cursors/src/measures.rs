//! **Every measure the pointer is drawn to**, in one file.
//!
//! `CLAUDE.md`, *nothing is built to one screen size*: these are **logical**
//! measures and artwork units, never pixels on a particular display. A caller
//! turns [`LOGICAL_SIZE`] into pixels through its display's scale and asks
//! [`crate::draw`] for that many; everything below is in the artwork's own
//! units and grows with it, so a dense screen draws the same pointer sharper
//! rather than smaller.

/// How many units across the artwork's box is. Every path in this crate is
/// written on it, as both of the owner's candidate arrows were.
pub const BOX_UNITS: f32 = 32.0;

/// How large one box is drawn at 100 per cent, in logical pixels.
///
/// **24, because that is the size every other desktop's pointer is** — it is
/// what `XCURSOR_SIZE` means when nobody has set it — so alo's pointer is the
/// size a person's hand already expects. The arrow inside it is about 17 of
/// those pixels tall, as the 18-row mask it replaces was.
pub const LOGICAL_SIZE: u32 = 24;

/// How far the white keyline reaches outside each shape, in units.
///
/// What keeps a navy pointer visible over a navy window: the person always sees
/// a light edge around it, on any background.
pub const KEYLINE: f32 = 1.5;

/// How far the shadow falls below the shape, in units.
pub const SHADOW_FALLS: f32 = 1.0;

/// How far the shadow spreads softly, in units.
pub const SHADOW_SOFTNESS: f32 = 1.5;

/// How much of the shadow's colour shows where the shape is solid, out of 255.
///
/// About 30 per cent: enough to lift the pointer off a white page, little
/// enough that it never reads as a second outline.
pub const SHADOW_OPACITY: u8 = 77;

/// The colour a shadow is cast in: `navy/950`, `#07131F`, as every raised
/// surface in the shell has it (`crates/alo-shell/src/a_window_raised.rs`).
pub const SHADOW_COLOUR: [u8; 3] = [0x07, 0x13, 0x1F];

/// How many pixels a box drawn at `per_cent` is: [`LOGICAL_SIZE`] through the
/// display's scale, rounded to the nearest pixel and never below one.
///
/// Nought per cent is read as one hundred, as the shell reads a display with
/// no answer for its scale.
#[must_use]
pub fn pixels_at(per_cent: u16) -> u32 {
    let per_cent = if per_cent == 0 { 100 } else { u32::from(per_cent) };
    ((LOGICAL_SIZE * per_cent + 50) / 100).max(1)
}
