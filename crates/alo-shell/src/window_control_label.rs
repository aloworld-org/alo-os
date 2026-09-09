//! Externalized control names shaped without consulting host fonts or clients.

use alo_appearance::{Scheme, TextScale, Token};
use alo_strings::{Said, Strings};
use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache, Wrap};
use smithay::utils::{Physical, Rectangle};

use crate::WindowControl;

/// Refusal before a prepared label can be painted. Diagnostic, not UI wording.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WindowControlLabelError {
    /// Invalid viewport, origin, box dimensions or excessive pixel allocation.
    #[error("unsupported native label geometry")]
    Geometry,
    /// Empty or excessive text (maximum 4096 UTF-8 bytes).
    #[error("unsupported native label text length")]
    Text,
    /// The host did not register the action vocabulary or supplied unfilled text.
    #[error("native control label vocabulary is incomplete")]
    Vocabulary,
    /// No valid font face was supplied.
    #[error("invalid native label font")]
    Font,
    /// The supplied fonts cannot represent the label; no silent missing boxes.
    #[error("native label has an unsupported glyph")]
    MissingGlyph,
}

/// A reusable native shaper with an explicit, private font database.
/// No filesystem scan, network access, application context or input authority.
pub struct WindowControlLabels {
    /// Loaded once, with only explicitly supplied fonts.
    pub(crate) fonts: FontSystem,
}

/// Immutable label pixels and the full externalized text, including provenance.
/// It does not own a window, change hit geometry or authorize an action.
pub struct WindowControlLabel {
    /// Unabridged words and translation provenance.
    pub(crate) said: Said,
    /// Validated clipping output.
    pub(crate) viewport: Rectangle<i32, Physical>,
    /// Validated label box, including padding.
    pub(crate) bounds: Rectangle<i32, Physical>,
    /// Bounded row-major opaque RGBA image.
    pub(crate) pixels: Vec<[u8; 4]>,
    /// Text or output loss requires alternate full-text presentation.
    pub(crate) clipped: bool,
}

impl WindowControlLabels {
    /// Use the bundled, OFL-licensed Inter face specified by the design brief.
    pub fn new() -> Result<Self, WindowControlLabelError> {
        Self::from_fonts([include_bytes!("../fonts/Inter.ttf").to_vec()])
    }

    /// Supply an explicit primary font followed by fallback fonts. Every entry
    /// must parse into at least one face. The first face's family is the default.
    /// This permits additional scripts without reading unrelated host fonts.
    pub fn from_fonts(
        fonts: impl IntoIterator<Item = Vec<u8>>,
    ) -> Result<Self, WindowControlLabelError> {
        let mut db = cosmic_text::fontdb::Database::new();
        for data in fonts {
            let before = db.faces().count();
            db.load_font_data(data);
            if db.faces().count() == before {
                return Err(WindowControlLabelError::Font);
            }
        }
        let family = db
            .faces()
            .next()
            .and_then(|face| face.families.first())
            .map(|(name, _)| name.clone())
            .ok_or(WindowControlLabelError::Font)?;
        db.set_sans_serif_family(family);
        Ok(Self {
            fonts: FontSystem::new_with_locale_and_db("en-US".into(), db),
        })
    }

    /// Shape the entire bounded wording once, including off-page glyph checks.
    pub(crate) fn shape(
        &mut self,
        control: &WindowControl,
        strings: &Strings,
        width: i32,
        scale: TextScale,
    ) -> Result<(Said, Buffer), WindowControlLabelError> {
        let said = control.action().said(strings);
        let buffer = self.shape_said(&said, width, scale)?;
        Ok((said, buffer))
    }

    /// Shape explicit externalized wording with the same font and scale policy.
    pub(crate) fn shape_said(
        &mut self,
        said: &Said,
        width: i32,
        scale: TextScale,
    ) -> Result<Buffer, WindowControlLabelError> {
        if said.is_a_bug() {
            return Err(WindowControlLabelError::Vocabulary);
        }
        if said.text().trim().is_empty() || said.text().len() > 4096 {
            return Err(WindowControlLabelError::Text);
        }
        let factor = f32::from(scale.as_percent()) / 100.0;
        let mut buffer = Buffer::new(&mut self.fonts, Metrics::new(14.0 * factor, 20.0 * factor));
        buffer.set_wrap(&mut self.fonts, Wrap::WordOrGlyph);
        // Shape the complete bounded label so clipping cannot hide missing glyphs.
        buffer.set_size(&mut self.fonts, Some((width - 8) as f32), None);
        buffer.set_text(
            &mut self.fonts,
            said.text(),
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.fonts, false);
        if buffer
            .layout_runs()
            .any(|run| run.glyphs.iter().any(|glyph| glyph.glyph_id == 0))
        {
            return Err(WindowControlLabelError::MissingGlyph);
        }
        Ok(buffer)
    }

    /// Prepare one control's full `Action::said` label, including disabled ones.
    /// At scale one, 14px text/20px line height follow the person's TextScale.
    /// The explicit box has 4px padding and wraps at word/glyph boundaries.
    /// Size is 9..=2048 by 9..=512; viewport and origin use the strip's limits.
    /// Text and output clipping are reported, while `said()` retains every word.
    /// Hosts must provide a way to read the full text when `clipped()` is true.
    pub fn prepare(
        &mut self,
        control: &WindowControl,
        strings: &Strings,
        geometry: LabelGeometry,
        scheme: Scheme,
        scale: TextScale,
    ) -> Result<WindowControlLabel, WindowControlLabelError> {
        self.prepare_said(control.action().said(strings), geometry, scheme, scale)
    }

    /// Raster explicit wording using the ordinary complete-label policy.
    pub(crate) fn prepare_said(
        &mut self,
        said: Said,
        geometry: LabelGeometry,
        scheme: Scheme,
        scale: TextScale,
    ) -> Result<WindowControlLabel, WindowControlLabelError> {
        let LabelGeometry {
            viewport,
            origin,
            size,
        } = geometry;
        crate::WindowControlLayout::new(viewport, origin, [false; 3], false)
            .map_err(|_| WindowControlLabelError::Geometry)?;
        if !(9..=2048).contains(&size.0) || !(9..=512).contains(&size.1) {
            return Err(WindowControlLabelError::Geometry);
        }
        let buffer = self.shape_said(&said, size.0, scale)?;
        let viewport = Rectangle::from_size(viewport.into());
        let bounds = Rectangle::new(origin.into(), size.into());
        let mut clipped = bounds.intersection(viewport) != Some(bounds)
            || buffer
                .layout_runs()
                .any(|run| run.line_top + run.line_height > (size.1 - 8) as f32);
        let (ground, ink) = match scheme {
            Scheme::Light => (Token::Cream.colour(), Token::Navy.colour()),
            Scheme::Dark => (Token::Charcoal.colour(), Token::Cream.colour()),
        };
        let background = [ground.red(), ground.green(), ground.blue(), 255];
        let mut pixels = vec![background; (size.0 * size.1) as usize];
        let mut invalid_offset = false;
        buffer.draw(
            &mut self.fonts,
            &mut SwashCache::new(),
            Color::rgb(ink.red(), ink.green(), ink.blue()),
            |x, y, w, h, color| {
                for dy in 0..h {
                    for dx in 0..w {
                        let x = i64::from(x) + i64::from(dx) + 4;
                        let y = i64::from(y) + i64::from(dy) + 4;
                        // Bearings and accents may extend into nominal padding.
                        // Clip actual ink only at the box, not the text advance.
                        if x < 0 || y < 0 || x >= i64::from(size.0) || y >= i64::from(size.1) {
                            clipped |= color.a() != 0;
                            continue;
                        }
                        let Some(pixel) = pixels.get_mut((y * i64::from(size.0) + x) as usize)
                        else {
                            invalid_offset = true;
                            continue;
                        };
                        for (channel, value) in
                            pixel[..3].iter_mut().zip([color.r(), color.g(), color.b()])
                        {
                            let alpha = u32::from(color.a());
                            *channel = ((u32::from(value) * alpha
                                + u32::from(*channel) * (255 - alpha)
                                + 127)
                                / 255) as u8;
                        }
                    }
                }
            },
        );
        if invalid_offset {
            return Err(WindowControlLabelError::Geometry);
        }
        Ok(WindowControlLabel {
            said,
            viewport,
            bounds,
            pixels,
            clipped,
        })
    }
}

/// Explicit output-local, scale-one placement; preparation validates all fields.
#[derive(Clone, Copy, Debug)]
pub struct LabelGeometry {
    /// Positive viewport, at most 1,000,000 per dimension.
    pub viewport: (i32, i32),
    /// Each coordinate between -1,000,000 and 1,000,000.
    pub origin: (i32, i32),
    /// Width 9..=2048 and height 9..=512, including padding.
    pub size: (i32, i32),
}

impl WindowControlLabel {
    /// Full externalized text and its translated/source-fallback provenance.
    pub fn said(&self) -> &Said {
        &self.said
    }

    /// True when any text or label bounds do not fit the box/viewport.
    pub fn clipped(&self) -> bool {
        self.clipped
    }

    /// Full label rectangle before viewport clipping.
    pub fn bounds(&self) -> Rectangle<i32, Physical> {
        self.bounds
    }

    /// Opaque row-major RGBA pixels for this label's full box, before output clipping.
    pub fn pixels(&self) -> &[[u8; 4]] {
        &self.pixels
    }
}

#[cfg(test)]
#[path = "window_control_label_tests.rs"]
mod tests;
