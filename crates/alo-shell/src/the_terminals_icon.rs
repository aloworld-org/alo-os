//! **The terminal's icon on its window edge**: a navy tile with the prompt mark
//! on it, as the approved frame draws it.
//!
//! # What the design draws
//!
//! `557:148`, *Window · Terminal · not focused*, carries the same edge as every
//! application (`554:56`), and in its icon's place (`I557:150;554:42`, 16 × 16
//! at (14, 20)) a tile. Read off the frame's render on 2026-11-01, pixel by
//! pixel, because the instance's own nodes cannot be asked for their fills:
//!
//! | | figure |
//! |---|---|
//! | the tile | 16 × 16, `#102a43` — `text/primary` — with corners of about 4 |
//! | the mark | white, the prompt-and-line of *Terminal* in Settings, about 10 across, centred |
//!
//! The corner is `554:26`'s *its own tile at corner 4*, which
//! `docs/design/the-external-window-edge.md` already records for every icon
//! tile. The mark is `icons/settings/terminal.svg`, the design's own artwork
//! for the terminal, drawn into a 14 frame: that file's group is 13.3 of a
//! 20 frame, and 14 puts the stroke's outer edge at the 10 the render shows.
//!
//! # The same tile focused and not
//!
//! The render is of the window **not** focused and the tile is navy there; the
//! focused frames draw an application's own colours unchanged. An icon is the
//! application's, and the look changes only what alo draws round a window.
//!
//! # Drawn at the size it is shown
//!
//! At the display's own scale, so the mark is drawn sharp at 200 % rather than
//! a 16-pixel tile stretched. It costs one tile of a few hundred pixels a
//! frame, while a terminal is open.

use alo_dock::artwork::APicture;
use smithay::utils::Rectangle;

use crate::settings_canvas::Canvas;
use crate::the_settings_icons::ASettingsIcon;

/// The tile: `554:26`'s 16.
const THE_TILE: f64 = 16.0;
/// Its corner, from the same node.
const ITS_CORNER: f64 = 4.0;
/// The frame the mark is drawn into, centred on the tile.
const THE_MARKS_FRAME: f64 = 14.0;
/// The mark's colour: white, as the render shows it.
const THE_MARK: [u8; 3] = [0xFF, 0xFF, 0xFF];

/// How many pixels the tile is across at a display drawn at `scale` per cent.
#[must_use]
pub(crate) fn across_at(scale: u32) -> u32 {
    let across = THE_TILE * f64::from(scale) / 100.0;
    // A display's scale is a few hundred per cent at most, so this is a few
    // dozen pixels, far inside `u32`.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a sixteen-unit tile through a display's scale is a small positive count"
    )]
    let across = across.round().max(1.0) as u32;
    across
}

/// The terminal's icon, drawn `across` pixels square.
///
/// [`None`] only for a size no picture can have.
#[must_use]
pub(crate) fn the_terminals_icon(across: u32) -> Option<APicture> {
    let side = i32::try_from(across).ok()?;
    if side <= 0 {
        return None;
    }
    let to_pixels = f64::from(across) / THE_TILE;
    let mut canvas = Canvas::over(Rectangle::from_size((side, side).into()));
    let tile = alo_appearance::Role::TextPrimary.colour();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a four-unit corner through a display's scale is a small number"
    )]
    let corner = (ITS_CORNER * to_pixels) as f32;
    canvas.fill(
        Rectangle::from_size((side, side).into()),
        corner,
        [tile.red(), tile.green(), tile.blue()],
        1.0,
    );
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a fourteen-unit frame through a display's scale is a small number"
    )]
    let frame = (THE_MARKS_FRAME * to_pixels).round() as i32;
    let inset = (side - frame) / 2;
    ASettingsIcon::Terminal.drawn(
        &mut canvas,
        Rectangle::new((inset, inset).into(), (frame, frame).into()),
        THE_MARK,
    );
    APicture::of(across, across, canvas.washed().pixels).ok()
}

#[cfg(test)]
#[path = "the_terminals_icon_tests.rs"]
mod tests;
