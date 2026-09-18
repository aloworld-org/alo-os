//! The lock policy's five permitted facts, rasterized with no access to a desktop.
use crate::{LockBackground, RenderError, SignInLook, SignInShows, WindowControlLabels};
use alo_appearance::{Scheme, Token};
use alo_formats::{Regionally, Timezone};
use alo_locking::LockScreen;
use alo_strings::Strings;
use cosmic_text::Metrics;

/// Formatting selected by the person; no timezone is guessed from the network.
pub struct LockLook<'a> {
    /// Existing appearance tokens and text scaling.
    pub appearance: SignInLook,
    /// Regional clock formatting.
    pub region: &'a Regionally,
    /// The explicitly selected civil timezone.
    pub timezone: &'a Timezone,
    /// The person's vocabulary.
    pub strings: &'a Strings,
}

/// An opaque lock frame, containing pixels of permitted facts only.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LockPicture {
    /// The output extent used for layout.
    pub(crate) size: (i32, i32),
    /// Opaque RGBA image and native text, ready for one texture.
    pub(crate) pixels: Vec<u8>,
}

/// Draw the policy view and optionally the existing credential-entry form.
pub(crate) fn picture(
    screen: &LockScreen,
    background: &LockBackground,
    fields: Option<SignInShows<'_>>,
    labels: &mut WindowControlLabels,
    look: &LockLook<'_>,
) -> Result<LockPicture, RenderError> {
    let size = background.size;
    if size.0 < 320 || size.1 < 480 || &background.chosen != screen.image() {
        return Err(RenderError::LockScene);
    }
    let mut pixels = background.pixels.clone();
    if let Some(fields) = fields {
        // Reserve the top and bottom thirds for lock facts. Reuse the complete
        // sign-in raster in the middle third, refusing any clipped refusal text.
        let height = size.1 / 3;
        let mut form =
            crate::sign_in_raster::picture(fields, labels, (size.0, height), look.appearance)?;
        if form
            .inked
            .iter()
            .any(|ink| ink.area.loc.y + ink.area.size.h >= height)
        {
            return Err(RenderError::LockScene);
        }
        for solid in &mut form.solids {
            solid.area.loc.y += height;
        }
        for ink in &mut form.inked {
            ink.area.loc.y += height;
        }
        crate::lock_pixels::overlay(&mut pixels, size, &form.solids, &form.inked);
    }
    let dark = look.appearance.scheme == Scheme::Dark;
    let rgb = crate::egress_status_mark::rgb;
    let ground = rgb(if dark {
        Token::Charcoal
    } else {
        Token::Porcelain
    }
    .colour());
    let ink = rgb(if dark { Token::Cream } else { Token::Navy }.colour());
    let factor = f32::from(look.appearance.scale.as_percent()) / 100.0;
    let margin = (16.0 * factor).ceil() as i32;
    let width = (size.0 - margin * 2).min(960);
    let left = (size.0 - width) / 2;
    let mut solids = Vec::new();
    let mut inked = Vec::new();
    let time = crate::lock_clock::written(screen.at(), look.timezone, look.region)?;
    let locked = screen.locked_said(look.strings);
    let mut y = margin;
    for (text, font) in [(time.as_str(), 32.0), (locked.text(), 18.0)] {
        let drawn = text_box(text, labels, width, font * factor, ground, ink)?;
        if y + drawn.height > size.1 / 3 {
            return Err(RenderError::LockScene);
        }
        let height = drawn.height;
        inked.push(
            drawn
                .placed(left, y, size.1)
                .ok_or(RenderError::LockScene)?,
        );
        y += height;
    }
    if let Some(battery) = screen.battery() {
        solids.extend(crate::lock_battery::draw(
            battery, size, margin, ground, ink,
        ));
    }
    if screen.lamp().is_lit() {
        let said = screen.lamp().said(look.strings);
        let drawn = text_box(said.text(), labels, width - 44, 16.0 * factor, ground, ink)?;
        let y = size.1 - margin - 36 - drawn.height;
        if y < size.1 * 2 / 3 {
            return Err(RenderError::LockScene);
        }
        solids.extend(crate::egress_status_mark::mark(
            left,
            y,
            28,
            look.appearance.scheme,
        ));
        inked.push(
            drawn
                .placed(left + 44, y, size.1)
                .ok_or(RenderError::LockScene)?,
        );
    }
    crate::lock_pixels::overlay(&mut pixels, size, &solids, &inked);
    Ok(LockPicture { size, pixels })
}

/// Bound translated text and shape it on an opaque contrasting ground.
fn text_box(
    text: &str,
    labels: &mut WindowControlLabels,
    width: i32,
    font: f32,
    ground: [u8; 3],
    ink: [u8; 3],
) -> Result<crate::painted_text::Shaped, RenderError> {
    if text.len() > 4096 || width < 32 {
        return Err(RenderError::LockScene);
    }
    let shaped = crate::painted_text::sentence(
        &mut labels.fonts,
        text,
        width,
        Metrics::new(font, font * 1.3),
        ground,
        ink,
    );
    if shaped.height > 200 {
        return Err(RenderError::LockScene);
    }
    Ok(shaped)
}
