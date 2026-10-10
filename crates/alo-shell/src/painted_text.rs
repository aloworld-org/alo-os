//! Text shaped with the bundled font and inked into a box of owned pixels.
//!
//! Shared by the native surfaces that draw sentences — the sign-in screen and
//! the egress indicator — so a sentence is wrapped, shaped and inked one way on
//! this machine. What the sentence says is never decided here: it arrives as
//! text a crate that owns the words already answered.

use cosmic_text::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache, Weight, Wrap,
};

use crate::painted::Inked;

/// What alo's own surfaces are drawn in.
///
/// **Manrope, because it is what the design is drawn in.** Every `Label/*` in
/// the design file is Manrope, and until 2026-10-09 this crate bundled only
/// Inter and asked for `Family::SansSerif` — so nothing in this product drew
/// the typeface its own design uses. `fonts/README.md` carries the licence,
/// the upstream revision and the hash.
///
/// **This is the shell's text and never an application's.** An application
/// keeps its own typography; what this governs is what alo says in its own
/// voice — a window's title on its edge, a sentence on the sign-in screen.
///
/// A script Manrope does not cover falls through to the rest of the font
/// database, which is why Inter stays loaded. Narrowing that would mean a
/// person's language stops rendering.
pub(crate) const ALOS_OWN_FACE: &str = "Manrope";

/// The weight the design draws its labels at: `Label/Medium` is SemiBold.
pub(crate) const ALOS_OWN_WEIGHT: Weight = Weight(600);

/// How alo asks for its own text, in one place so a measurement and a drawing
/// cannot disagree about the face they meant.
pub(crate) fn alos_own_text<'a>() -> Attrs<'a> {
    Attrs::new()
        .family(Family::Name(ALOS_OWN_FACE))
        .weight(ALOS_OWN_WEIGHT)
}

/// Text shaped and inked into its own box, not yet placed.
pub(crate) struct Shaped {
    /// The box's width.
    pub(crate) width_of_box: i32,
    /// The ink's own width, for a caret after it.
    pub(crate) width: i32,
    /// The box's height.
    pub(crate) height: i32,
    /// Row-major pixels of the box.
    pub(crate) pixels: Vec<[u8; 3]>,
}

impl Shaped {
    /// The box at (`x`, `y`), cut at the bottom of an output `limit` high.
    pub(crate) fn placed(self, x: i32, y: i32, limit: i32) -> Option<Inked> {
        let rows = self.height.min(limit - y);
        if rows <= 0 || self.width_of_box <= 0 {
            return None;
        }
        let kept = (rows * self.width_of_box) as usize;
        let mut pixels = self.pixels;
        pixels.truncate(kept);
        Some(Inked {
            area: smithay::utils::Rectangle::new((x, y).into(), (self.width_of_box, rows).into()),
            pixels,
        })
    }
}

/// A sentence, wrapped to `width`, in a box exactly as tall as its lines.
pub(crate) fn sentence(
    fonts: &mut FontSystem,
    text: &str,
    width: i32,
    metrics: Metrics,
    ground: [u8; 3],
    ink: [u8; 3],
) -> Shaped {
    let mut buffer = Buffer::new(fonts, metrics);
    buffer.set_wrap(fonts, Wrap::WordOrGlyph);
    buffer.set_size(fonts, Some(width as f32), None);
    buffer.set_text(fonts, text, &alos_own_text(), Shaping::Advanced);
    buffer.shape_until_scroll(fonts, false);
    let height = buffer
        .layout_runs()
        .map(|run| run.line_top + run.line_height)
        .fold(0.0, f32::max)
        .ceil() as i32;
    let mut shaped = inked(fonts, &buffer, (width, height.max(1)), 0, ground, ink);
    shaped.width = buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0, f32::max)
        .ceil() as i32;
    shaped.width = shaped.width.clamp(0, width);
    shaped
}

/// How wide this text wants to be, shaped and unwrapped.
///
/// **The question [`sentence`] cannot answer.** That one wraps, and clamps
/// its reported width to the box it was given, so text too wide to fit comes
/// back the same width as text that just fits. Anything deciding whether to
/// truncate needs the width the text actually wants.
///
/// Shaped with the same attributes and the same `Shaping::Advanced` as every
/// other sentence on this machine, so a measurement and a drawing cannot
/// disagree about the same string.
pub(crate) fn how_wide(fonts: &mut FontSystem, text: &str, metrics: Metrics) -> i32 {
    let mut buffer = Buffer::new(fonts, metrics);
    // No wrapping and no width: the natural run, however long.
    buffer.set_wrap(fonts, Wrap::None);
    buffer.set_size(fonts, None, None);
    buffer.set_text(fonts, text, &alos_own_text(), Shaping::Advanced);
    buffer.shape_until_scroll(fonts, false);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a shaped run's width in whole pixels is what a layout measures in"
    )]
    let widest = buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0, f32::max)
        .ceil() as i32;
    widest
}

/// One short string, inked **centred** in a box `width` across.
///
/// For the letter a Dock icon shows when an application has no artwork — the
/// owner's ruling of 2026-10-10, and today that is every application, because
/// `alo-applications` names an icon in none of its files.
///
/// **It exists rather than a caller centring for itself**, because centring is
/// the one thing a caller cannot do after the fact: [`sentence`] inks into a box
/// of the width it was given, and a caller holding that box can only move the
/// whole box, ground and all, which drags the Dock's own colour over whatever is
/// beside it. The shift belongs inside the ink, which is what [`inked`]'s
/// `shift` is for and what this passes.
///
/// A single grapheme does not wrap, so the box is one line tall and the caller
/// centres it down its own axis from [`Shaped::height`].
pub(crate) fn centred(
    fonts: &mut FontSystem,
    text: &str,
    width: i32,
    metrics: Metrics,
    ground: [u8; 3],
    ink: [u8; 3],
) -> Shaped {
    let mut buffer = Buffer::new(fonts, metrics);
    // **No wrapping.** One letter cannot be broken across lines, and allowing it
    // would let a wide glyph — a flag emoji is two cells — become two rows half
    // an icon each.
    buffer.set_wrap(fonts, Wrap::None);
    buffer.set_size(fonts, Some(width as f32), None);
    // **alo's own face**, the same one every other surface on this machine is
    // drawn in, rather than whatever the system calls sans-serif.
    buffer.set_text(fonts, text, &alos_own_text(), Shaping::Advanced);
    buffer.shape_until_scroll(fonts, false);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a shaped run's width and height in whole pixels is what a layout measures in"
    )]
    let (height, drawn) = {
        let height = buffer
            .layout_runs()
            .map(|run| run.line_top + run.line_height)
            .fold(0.0, f32::max)
            .ceil() as i32;
        let drawn = buffer
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0, f32::max)
            .ceil() as i32;
        (height, drawn)
    };
    // Half the room left over, and never negative: a glyph wider than its box
    // starts at the box's own edge rather than before it, which keeps it inside
    // the icon even when it does not fit.
    let shift = ((width - drawn) / 2).max(0);
    let mut shaped = inked(fonts, &buffer, (width, height.max(1)), shift, ground, ink);
    shaped.width = drawn.clamp(0, width);
    trimmed_to_its_ink(shaped, ground)
}

/// The same box with its blank rows cut off the top and the bottom.
///
/// **So that centring the box centres the letter.** A line box is not a letter:
/// it holds the ascent, the descent and the leading of whatever the font would
/// need for `Áyg`, and a capital occupies the upper part of it. Centre the box
/// and the letter rides high — visibly so, which is what the Dock's first
/// drawing showed.
///
/// Trimming is better than asking the font for its cap height because it is
/// **exact for the glyph actually drawn**: a Devanagari letter with a matra
/// above it, an emoji that fills the line, and a Latin capital all have
/// different ink, and this centres what each of them really is rather than what
/// a Latin capital would be.
///
/// A box with no ink in it comes back unchanged. There is nothing to centre and
/// a height of zero would place nothing at all.
fn trimmed_to_its_ink(shaped: Shaped, ground: [u8; 3]) -> Shaped {
    let across = shaped.width_of_box;
    if across <= 0 {
        return shaped;
    }
    let inked_rows: Vec<usize> = shaped
        .pixels
        .chunks(across as usize)
        .enumerate()
        .filter(|(_, row)| row.iter().any(|pixel| *pixel != ground))
        .map(|(which, _)| which)
        .collect();
    let (Some(first), Some(last)) = (inked_rows.first(), inked_rows.last()) else {
        return shaped;
    };
    let from = first.saturating_mul(across as usize);
    let to = last.saturating_add(1).saturating_mul(across as usize);
    let Some(kept) = shaped.pixels.get(from..to.min(shaped.pixels.len())) else {
        return shaped;
    };
    let height =
        i32::try_from(last.saturating_sub(*first).saturating_add(1)).unwrap_or(shaped.height);
    Shaped {
        width_of_box: across,
        width: shaped.width,
        height,
        pixels: kept.to_vec(),
    }
}

/// Ink a shaped buffer into a box of `size`, moved `shift` pixels across.
pub(crate) fn inked(
    fonts: &mut FontSystem,
    buffer: &Buffer,
    size: (i32, i32),
    shift: i32,
    ground: [u8; 3],
    ink: [u8; 3],
) -> Shaped {
    let (width, height) = size;
    let mut pixels = vec![ground; (width.max(0) * height.max(0)) as usize];
    buffer.draw(
        fonts,
        &mut SwashCache::new(),
        Color::rgb(ink[0], ink[1], ink[2]),
        |x, y, w, h, colour| {
            for dy in 0..h as i32 {
                for dx in 0..w as i32 {
                    let (px, py) = (x + dx + shift, y + dy);
                    if px < 0 || py < 0 || px >= width || py >= height {
                        continue;
                    }
                    let Some(pixel) = pixels.get_mut((py * width + px) as usize) else {
                        continue;
                    };
                    let alpha = u32::from(colour.a());
                    for (channel, value) in
                        pixel.iter_mut().zip([colour.r(), colour.g(), colour.b()])
                    {
                        *channel =
                            ((u32::from(value) * alpha + u32::from(*channel) * (255 - alpha) + 127)
                                / 255) as u8;
                    }
                }
            }
        },
    );
    Shaped {
        width_of_box: width,
        width,
        height,
        pixels,
    }
}

#[cfg(test)]
mod the_letter_is_centred_on_its_ink {
    use super::{Metrics, centred};

    /// The bundled face, as the shell loads it.
    fn fonts() -> cosmic_text::FontSystem {
        let mut fonts = cosmic_text::FontSystem::new();
        fonts
            .db_mut()
            .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
        fonts
    }

    /// Which rows of a shaped box have any ink in them.
    fn inked_rows(shaped: &super::Shaped, ground: [u8; 3]) -> Vec<usize> {
        shaped
            .pixels
            .chunks(shaped.width_of_box as usize)
            .enumerate()
            .filter(|(_, row)| row.iter().any(|pixel| *pixel != ground))
            .map(|(which, _)| which)
            .collect()
    }

    /// **The box is the letter, with no blank rows above or below it.**
    ///
    /// A line box holds the ascent, descent and leading a font would need for
    /// `Áyg`, and a capital sits in the upper part of it — so centring the box
    /// puts the letter high, which is exactly what the Dock's first drawing
    /// showed. Trimming makes *centre the box* and *centre the letter* the same
    /// act.
    #[test]
    fn a_letters_box_begins_and_ends_with_its_ink() {
        let ground = [255, 255, 255];
        let ink = [0, 0, 0];
        for letter in ["F", "M", "B", "g", "कि"] {
            let shaped = centred(
                &mut fonts(),
                letter,
                48,
                Metrics::new(32.0, 32.0),
                ground,
                ink,
            );
            let rows = inked_rows(&shaped, ground);
            assert!(!rows.is_empty(), "{letter}: nothing was inked at all");
            assert_eq!(
                rows.first().copied(),
                Some(0),
                "{letter}: the box starts with {} blank rows above the letter, so centring it \
                 would ride high",
                rows.first().copied().unwrap_or_default()
            );
            assert_eq!(
                rows.last().copied(),
                Some(shaped.pixels.len() / shaped.width_of_box as usize - 1),
                "{letter}: the box ends with blank rows below the letter"
            );
        }
    }

    /// **A letter with a descender is taller than one without**, which is the
    /// check that the trim follows each glyph rather than cutting to a constant.
    #[test]
    fn the_box_follows_the_glyph_rather_than_a_fixed_height() {
        let ground = [255, 255, 255];
        let ink = [0, 0, 0];
        let mut fonts = fonts();
        let capital = centred(&mut fonts, "F", 48, Metrics::new(32.0, 32.0), ground, ink);
        let full_stop = centred(&mut fonts, ".", 48, Metrics::new(32.0, 32.0), ground, ink);
        // **`F` against `.`, not `F` against `g`.** The first pair tried here
        // was `F` and `g`, and they trim to the **same** height — a capital's
        // cap height and a descender's x-height-plus-tail are close enough at
        // 32 pixels to coincide. That was this test's assumption being wrong
        // rather than the trim, and a full stop cannot coincide with anything.
        assert!(
            full_stop.height < capital.height,
            "a full stop ({}) trimmed no shorter than a capital ({}), so the trim is cutting to \
             a fixed height rather than reading each glyph's ink",
            full_stop.height,
            capital.height
        );
    }

    /// **A box with no ink comes back whole**, because there is nothing to
    /// centre and a height of zero would place nothing at all.
    #[test]
    fn a_blank_box_is_left_alone() {
        let ground = [12, 34, 56];
        let shaped = centred(
            &mut fonts(),
            " ",
            48,
            Metrics::new(32.0, 32.0),
            ground,
            ground,
        );
        assert!(
            shaped.height >= 1,
            "a blank box was trimmed out of existence"
        );
        assert!(!shaped.pixels.is_empty());
    }
}
