//! The egress indicator laid out and rasterised for one output: a row for each
//! line `alo-egress` wrote, at the far end of the dock, and nothing at all
//! while nothing is leaving.
//!
//! # What is drawn
//!
//! While the machine's indicator is quiet — every question answered here, no
//! errand under way — **nothing**: no row, no mark, no terracotta, not one
//! pixel of this surface. While something is leaving, one row per line, each a
//! mark (`crate::egress_status_mark`) and the line itself, in the order the
//! indicator holds them, stacked from the status area's corner
//! (`crate::egress_status_place`).
//!
//! # What it reads
//!
//! `alo_indicator::Drawn` and nothing else about what is leaving. Each row's
//! words are `alo_indicator::Drawn::lines_said` — `alo-egress`'s sentence,
//! answered through the person's own vocabulary — and are drawn as they came:
//! never shortened, re-worded, or assembled here. An agent's egress and alo
//! OS's own errand are one list and are drawn the same way, because *no
//! telemetry* is a promise only somebody who can see the machine's unasked
//! traffic can check.
//!
//! # When the lines do not all fit
//!
//! A line is never dropped silently. When the output cannot hold every row, as
//! many as fit are drawn and the last row that fits carries the indicator's
//! own count instead — `alo_indicator::Drawn::lamp_said`, *3 things are
//! leaving this machine right now* — with its mark, so the screen still says
//! how much is leaving even where it cannot say all of what.

use alo_appearance::{Scheme, TextScale};
use alo_dock::{Dock, Screen};
use alo_indicator::Drawn;
use alo_strings::{Direction, Strings};

use crate::egress_status_mark::mark;
use crate::egress_status_place::{Across, Place, Stacked};
use crate::painted::{Inked, Solid};
use crate::status_row::{Laid, Measure, Palette, Row};
use crate::{Contrast, RenderError, WindowControlLabels};

/// How the indicator looks, as the person's appearance and language decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EgressStatusLook {
    /// Light or dark, as `alo-appearance` decides it.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure.
    pub scale: TextScale,
    /// Which way the person reads, which decides the far end of a row and
    /// which side of its words the mark is on.
    pub reading: Direction,
    /// The design's palette, or the one high contrast decides
    /// (`crate::access_contrast`).
    pub contrast: Contrast,
}

/// The largest output side, in pixels, the indicator is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// The indicator for one output size, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EgressStatusPicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// Every row, nearest the dock first.
    pub(crate) rows: Vec<Row>,
    /// Flat shapes, painted first.
    pub(crate) solids: Vec<Solid>,
    /// Words, painted after the shapes.
    pub(crate) inked: Vec<Inked>,
}

impl EgressStatusPicture {
    /// Whether nothing at all is drawn.
    pub(crate) fn is_empty(&self) -> bool {
        self.rows.is_empty() && self.solids.is_empty() && self.inked.is_empty()
    }
}

/// A row's ground and its ink, from the one palette door. Terracotta is the
/// mark's and not here.
pub(crate) fn palette(scheme: Scheme, contrast: Contrast) -> Palette {
    Palette {
        ground: contrast.ground(scheme),
        ink: contrast.ink(scheme),
    }
}

/// Lay the indicator out for an output of `size` and rasterise it.
///
/// A quiet indicator is an empty picture on any output. A lit one is laid out
/// at the far end of `dock` as `alo-dock` places it on this output, `beyond`
/// pixels past the corner — which is where the in-use indicator sits when
/// anything is in use, and zero when nothing is.
///
/// # Errors
/// [`RenderError::EgressStatusScene`] when something is leaving and the output
/// is too small for `alo-dock` to lay a dock out on, or larger than any output
/// this surface is laid out for. A frame that cannot carry the indicator is
/// refused rather than drawn without it.
pub(crate) fn picture(
    drawn: &Drawn,
    strings: &Strings,
    dock: &Dock,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: EgressStatusLook,
    beyond: i32,
) -> Result<EgressStatusPicture, RenderError> {
    let mut picture = EgressStatusPicture {
        size,
        rows: Vec::new(),
        solids: Vec::new(),
        inked: Vec::new(),
    };
    if drawn.lamp().is_dark() {
        return Ok(picture);
    }
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::EgressStatusScene);
    }
    let screen = Screen::of(
        u32::try_from(width).map_err(|_| RenderError::EgressStatusScene)?,
        u32::try_from(height).map_err(|_| RenderError::EgressStatusScene)?,
    )
    .map_err(|_| RenderError::EgressStatusScene)?;
    let measure = Measure::of(look.scale);
    let place = Place::of(
        dock.layout_on(screen, look.scale, look.reading),
        size,
        measure.px(8),
    )
    .beyond(beyond);
    let fonts = &mut labels.fonts;
    let palette = palette(look.scheme, look.contrast);

    let sentences: Vec<String> = drawn
        .lines_said(strings)
        .into_iter()
        .map(|said| said.into_text())
        .collect();
    let gap = measure.px(4);
    let mut shaped: Vec<Laid> = sentences
        .iter()
        .map(|text| Laid::of(fonts, text, place, measure, palette))
        .collect();
    let whole = |rows: &[Laid]| -> i32 {
        rows.iter().map(|row| row.height).sum::<i32>() + gap * (rows.len() as i32 - 1).max(0)
    };
    if whole(&shaped) > place.room_down {
        let count = Laid::of(
            fonts,
            drawn.lamp_said(strings).text(),
            place,
            measure,
            palette,
        );
        while !shaped.is_empty() && whole(&shaped) + gap + count.height > place.room_down {
            shaped.pop();
        }
        shaped.push(count);
    }

    let mut at_y = place.stacked;
    for laid in shaped {
        let left = match place.across {
            Across::FromLeft(x) => x,
            Across::FromRight(x) => x - laid.width,
        };
        let top = match at_y {
            Stacked::Downwards(y) => {
                at_y = Stacked::Downwards(y + laid.height + gap);
                y
            }
            Stacked::Upwards(y) => {
                at_y = Stacked::Upwards(y - laid.height - gap);
                y - laid.height
            }
        };
        let drawn = laid.place(
            (left.max(0), top.clamp(0, (height - 1).max(0))),
            size,
            measure,
            palette,
            look.reading,
            |at| mark(at.loc.x, at.loc.y, at.size.w, look.scheme, look.contrast),
        );
        picture.solids.extend(drawn.solids);
        picture.inked.extend(drawn.inked);
        picture.rows.push(drawn.row);
    }
    Ok(picture)
}

/// One row shaped and measured, not yet placed.
#[cfg(test)]
#[path = "egress_status_raster_tests.rs"]
mod tests;
