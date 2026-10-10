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
    shaped
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
