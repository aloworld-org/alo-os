//! Rasterize a complete group of shaped lines without losing any ink.

use crate::WindowControlPageError;
use alo_appearance::{Scheme, Token};
use cosmic_text::{Color, FontSystem, LayoutRun, SwashCache};

/// Input is a nonempty group of whole visual lines selected by the page planner.
pub(crate) fn rasterize(
    fonts: &mut FontSystem,
    runs: &[LayoutRun<'_>],
    size: (i32, i32),
    scheme: Scheme,
) -> Result<Vec<[u8; 4]>, WindowControlPageError> {
    let (ground, ink) = match scheme {
        Scheme::Light => (Token::Cream.colour(), Token::Navy.colour()),
        Scheme::Dark => (Token::Charcoal.colour(), Token::Cream.colour()),
    };
    let background = [ground.red(), ground.green(), ground.blue(), 255];
    let ink = Color::rgb(ink.red(), ink.green(), ink.blue());
    let mut cache = SwashCache::new();
    let mut pixels = vec![background; (size.0 * size.1) as usize];
    let top = runs
        .first()
        .ok_or(crate::WindowControlLabelError::Text)?
        .line_top;
    let mut clipped = false;
    for run in runs {
        for glyph in run.glyphs {
            let physical = glyph.physical((0., 0.), 1.0);
            cache.with_pixels(
                fonts,
                physical.cache_key,
                glyph.color_opt.unwrap_or(ink),
                |x, y, color| {
                    let x = physical.x + x + 4;
                    let y = (run.line_y - top) as i32 + physical.y + y + 4;
                    if x < 0 || y < 0 || x >= size.0 || y >= size.1 {
                        clipped |= color.a() != 0;
                        return;
                    }
                    let Some(pixel) = pixels.get_mut((y * size.0 + x) as usize) else {
                        clipped = true;
                        return;
                    };
                    for (channel, value) in
                        pixel[..3].iter_mut().zip([color.r(), color.g(), color.b()])
                    {
                        let alpha = u32::from(color.a());
                        *channel =
                            ((u32::from(value) * alpha + u32::from(*channel) * (255 - alpha) + 127)
                                / 255) as u8;
                    }
                },
            );
        }
    }
    if clipped {
        return Err(WindowControlPageError::LineTooLarge);
    }
    Ok(pixels)
}
