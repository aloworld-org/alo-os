//! The sign-in screen laid out and rasterised: solid shapes and inked text,
//! owned, for one output size.
//!
//! # What is drawn
//!
//! The ground, and then either one sentence alone — *make an account*, or
//! accounts that would not be read — or two fields with the last sentence the
//! greeting answered beneath them. The name field shows the name as typed,
//! scrolled so its end stays visible. The password field shows **whether**
//! something is typed, as one band of the same width however much is typed,
//! and never a mark per letter: a count of dots is the password's length on a
//! screen anybody behind the person can read.
//!
//! The waiting field is told apart by a thicker edge **and** a caret, never by
//! colour alone, and neither is terracotta, which means the agent and is not
//! on a screen nobody has signed in to yet.
//!
//! # What it reads
//!
//! `crate::SignInShows` and nothing else about the screen, so this file has no
//! way to reach the password even by mistake. Colours are `alo-appearance`'s
//! tokens and the scale is the person's `TextScale`; the font is the bundled
//! face `crate::WindowControlLabels` already loads, never a host font.

use alo_appearance::{Scheme, TextScale, Token};
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Wrap};
use smithay::utils::{Physical, Rectangle};

use crate::painted::{Inked, Solid};
use crate::painted_text::{Shaped, inked, sentence};
use crate::{RenderError, SignInField, SignInShows, WindowControlLabels};

/// How the screen looks: the scheme and the text scale the person chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignInLook {
    /// Light or dark.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure on the screen.
    pub scale: TextScale,
}

/// The largest output side, in pixels, the screen is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// The whole screen for one output size, ready to paint.
///
/// Holds pixels of the name as typed and of sentences, and nothing about the
/// password but whether one is typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SignInPicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// Flat shapes, painted first, in order.
    pub(crate) solids: Vec<Solid>,
    /// Text, painted after the shapes, in order.
    pub(crate) inked: Vec<Inked>,
}

/// The colours of one scheme.
#[derive(Clone, Copy)]
struct Palette {
    /// The screen's ground.
    ground: [u8; 3],
    /// Inside a field.
    field: [u8; 3],
    /// Text, edges and the caret.
    ink: [u8; 3],
}

impl Palette {
    /// `alo-appearance`'s tokens for this scheme. Terracotta is not among them.
    fn of(scheme: Scheme) -> Self {
        let rgb = |token: Token| {
            let colour = token.colour();
            [colour.red(), colour.green(), colour.blue()]
        };
        match scheme {
            Scheme::Light => Self {
                ground: rgb(Token::Porcelain),
                field: rgb(Token::Cream),
                ink: rgb(Token::Navy),
            },
            Scheme::Dark => Self {
                ground: rgb(Token::Charcoal),
                field: rgb(Token::Charcoal),
                ink: rgb(Token::Cream),
            },
        }
    }
}

/// Lay the screen out for an output of `size` and rasterise it.
///
/// # Errors
/// [`RenderError::SignInScene`] when the output is too small to hold two
/// fields, or larger than any output this screen is laid out for.
pub(crate) fn picture(
    shows: SignInShows<'_>,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: SignInLook,
) -> Result<SignInPicture, RenderError> {
    let measure = Measure::of(look.scale);
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::SignInScene);
    }
    let column = (width - 2 * measure.px(16)).min(measure.px(440));
    let field_height = measure.px(48);
    let gap = measure.px(12);
    if column < measure.px(120) || height < 2 * field_height + gap + 2 * measure.px(16) {
        return Err(RenderError::SignInScene);
    }
    let palette = Palette::of(look.scheme);
    let left = (width - column) / 2;
    let mut drawn = SignInPicture {
        size,
        solids: vec![Solid {
            area: Rectangle::from_size((width, height).into()),
            colour: palette.ground,
        }],
        inked: Vec::new(),
    };
    let fonts = &mut labels.fonts;

    match shows {
        SignInShows::Sentence(said) => {
            let text = sentence(
                fonts,
                said.text(),
                column,
                measure.metrics(),
                palette.ground,
                palette.ink,
            );
            let top = ((height - text.height) / 2).max(measure.px(16));
            drawn.inked.extend(text.placed(left, top, height));
        }
        SignInShows::Fields {
            name,
            password_typed,
            field,
            said,
        } => {
            let under = said.map(|said| {
                sentence(
                    fonts,
                    said.text(),
                    column,
                    measure.metrics(),
                    palette.ground,
                    palette.ink,
                )
            });
            let under_height = under
                .as_ref()
                .map_or(0, |text| measure.px(24) + text.height);
            let whole = 2 * field_height + gap + under_height;
            // A refusal must fit whole; clipping can otherwise omit it entirely.
            if whole + 2 * measure.px(16) > height {
                return Err(RenderError::SignInScene);
            }
            let top = ((height - whole) / 2).max(measure.px(16));

            let name_box = Rectangle::new((left, top).into(), (column, field_height).into());
            let password_box = Rectangle::new(
                (left, top + field_height + gap).into(),
                (column, field_height).into(),
            );
            let name_waits = field == SignInField::Name;
            framed(&mut drawn.solids, name_box, name_waits, &measure, palette);
            framed(
                &mut drawn.solids,
                password_box,
                !name_waits,
                &measure,
                palette,
            );

            let inside = column - 2 * measure.px(14);
            let line = measure.px(24);
            let name_top = top + (field_height - line) / 2;
            let typed = typed_name(fonts, name, inside, &measure, palette.field, palette.ink);
            let caret_at = left + measure.px(14) + typed.width.min(inside);
            drawn
                .inked
                .extend(typed.placed(left + measure.px(14), name_top, height));
            if name_waits {
                drawn
                    .solids
                    .push(caret(caret_at, name_top, line, &measure, palette));
            }

            let password_top = password_box.loc.y + (field_height - line) / 2;
            let band_width = inside - measure.px(6);
            if password_typed {
                drawn.solids.push(Solid {
                    area: Rectangle::new(
                        (
                            left + measure.px(14),
                            password_top + (line - measure.px(8)) / 2,
                        )
                            .into(),
                        (band_width, measure.px(8)).into(),
                    ),
                    colour: palette.ink,
                });
            }
            if !name_waits {
                let after = if password_typed {
                    band_width + measure.px(3)
                } else {
                    0
                };
                drawn.solids.push(caret(
                    left + measure.px(14) + after,
                    password_top,
                    line,
                    &measure,
                    palette,
                ));
            }

            if let Some(text) = under {
                let below = password_box.loc.y + field_height + measure.px(24);
                drawn.inked.extend(text.placed(left, below, height));
            }
        }
    }
    Ok(drawn)
}

/// Measures scaled by the person's text scale, never below one pixel.
struct Measure {
    /// The scale, in percent.
    percent: i32,
}

impl Measure {
    /// Measures for this scale.
    fn of(scale: TextScale) -> Self {
        Self {
            percent: i32::from(scale.as_percent()),
        }
    }

    /// `at_one` pixels at the ordinary scale, scaled.
    fn px(&self, at_one: i32) -> i32 {
        (at_one * self.percent / 100).max(1)
    }

    /// The text metrics every line on this screen uses.
    fn metrics(&self) -> Metrics {
        let factor = self.percent as f32 / 100.0;
        Metrics::new(16.0 * factor, 24.0 * factor)
    }
}

/// A field's edge and inside; the waiting one has the thicker edge.
fn framed(
    solids: &mut Vec<Solid>,
    area: Rectangle<i32, Physical>,
    waiting: bool,
    measure: &Measure,
    palette: Palette,
) {
    let edge = if waiting {
        measure.px(3)
    } else {
        measure.px(1)
    };
    solids.push(Solid {
        area,
        colour: palette.ink,
    });
    solids.push(Solid {
        area: Rectangle::new(
            (area.loc.x + edge, area.loc.y + edge).into(),
            (area.size.w - 2 * edge, area.size.h - 2 * edge).into(),
        ),
        colour: palette.field,
    });
}

/// The caret at the end of what is typed.
fn caret(x: i32, top: i32, line: i32, measure: &Measure, palette: Palette) -> Solid {
    Solid {
        area: Rectangle::new(
            (x, top + measure.px(2)).into(),
            (measure.px(2), line - 2 * measure.px(4)).into(),
        ),
        colour: palette.ink,
    }
}

/// The name as typed, on one line, scrolled so that its end is in view.
fn typed_name(
    fonts: &mut FontSystem,
    name: &str,
    width: i32,
    measure: &Measure,
    ground: [u8; 3],
    ink: [u8; 3],
) -> Shaped {
    let mut buffer = Buffer::new(fonts, measure.metrics());
    buffer.set_wrap(fonts, Wrap::None);
    buffer.set_size(fonts, None, None);
    buffer.set_text(
        fonts,
        name,
        &Attrs::new().family(Family::SansSerif),
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(fonts, false);
    let wide = buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0, f32::max)
        .ceil() as i32;
    let shift = (wide - width).max(0);
    let mut shaped = inked(fonts, &buffer, (width, measure.px(24)), -shift, ground, ink);
    shaped.width = wide.min(width);
    shaped
}

#[cfg(test)]
#[path = "sign_in_raster_tests.rs"]
mod tests;
