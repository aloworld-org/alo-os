//! Text shaped with the bundled font and inked into a box of owned pixels.
//!
//! Shared by the native surfaces that draw sentences — the sign-in screen and
//! the egress indicator — so a sentence is wrapped, shaped and inked one way on
//! this machine. What the sentence says is never decided here: it arrives as
//! text a crate that owns the words already answered.

use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache, Wrap};

use crate::painted::Inked;

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
    buffer.set_text(
        fonts,
        text,
        &Attrs::new().family(Family::SansSerif),
        Shaping::Advanced,
    );
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
