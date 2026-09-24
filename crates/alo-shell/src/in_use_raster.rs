//! The in-use indicator laid out and rasterised for one output: one row for
//! every line `alo-in-use` wrote, in the status area, and nothing at all while
//! nothing is in use.
//!
//! # What is drawn
//!
//! While nothing on the machine is watching or listening — no screen shared, no
//! camera, no microphone — **nothing**: no row, no mark, not one pixel. While
//! something is, one row per line: its mark (`crate::in_use_mark`), its
//! sentence as `alo-in-use` worded it, and the agent's dot beside it when the
//! agent is the one using the thing.
//!
//! # Where it sits, and why it and not the other one has the corner
//!
//! **The in-use indicator takes the status area's corner and the egress
//! indicator stacks beyond it.** ADR 0010 makes this indicator's *position* one
//! of the three things a person is given besides a colour, and says why:
//! position is the one that costs nothing to learn, so the camera is in the
//! same place every glance. That only holds if this stack's origin does not
//! move — so it gets the fixed corner, and what is leaving is stacked past it.
//!
//! The egress indicator's own rule is untouched by that: its first line stays
//! nearest *its* origin and a new one is added beyond, so nothing already on
//! the screen moves when something else starts leaving.
//!
//! # The order is the crate's, and this file cannot change it
//!
//! The screen comes before the camera and the camera before the microphone,
//! because `alo_in_use::Position` says so; two things using the camera are two
//! rows at the same position, one under the other. Nothing here sorts by who is
//! using something — if position depended on who, the microphone would move
//! when an agent picked it up, and the glance a person had learned would stop
//! working at the moment it mattered most.
//!
//! # What it reads
//!
//! `alo_in_use::Line` and nothing else. Every row's words are `Line::said` —
//! that crate's sentence, answered through the person's own vocabulary — and
//! are drawn as they came: never shortened, re-worded or assembled here.

use alo_appearance::{Scheme, TextScale};
use alo_dock::{Dock, Screen};
use alo_in_use::Line;
use alo_strings::{Direction, Strings};
use smithay::utils::{Physical, Rectangle};

use crate::egress_status_place::{Across, Place, Stacked};
use crate::in_use_mark::{MarkColours, mark};
use crate::painted::{Inked, Solid};
use crate::status_row::{Laid, Measure, Palette, Row};
use crate::{Contrast, RenderError, WindowControlLabels};

/// How the indicator looks, as the person's appearance and language decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InUseLook {
    /// Light or dark, as `alo-appearance` decides it.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure.
    pub scale: TextScale,
    /// Which way the person reads, which decides the far end of a row and
    /// which side of its words the mark is on.
    pub reading: Direction,
    /// The design's palette, or the one high contrast decides.
    pub contrast: Contrast,
}

/// The largest output side, in pixels, the indicator is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// The indicator for one output size, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InUsePicture {
    /// The output size it was laid out for.
    pub(crate) size: (i32, i32),
    /// Every row, nearest the dock first.
    pub(crate) rows: Vec<Row>,
    /// How tall the whole stack is, so what is leaving can be put beyond it.
    pub(crate) height: i32,
    /// Flat shapes, painted first.
    pub(crate) solids: Vec<Solid>,
    /// Words, painted after the shapes.
    pub(crate) inked: Vec<Inked>,
}

impl InUsePicture {
    /// Whether nothing at all is drawn.
    pub(crate) fn is_empty(&self) -> bool {
        self.rows.is_empty() && self.solids.is_empty() && self.inked.is_empty()
    }
}

/// A row's ground and its ink, from the one palette door.
fn palette(scheme: Scheme, contrast: Contrast) -> Palette {
    Palette {
        ground: contrast.ground(scheme),
        ink: contrast.ink(scheme),
    }
}

/// Lay the indicator out for an output of `size` and rasterise it.
///
/// Nothing in use is an empty picture on any output. Anything in use is laid
/// out at the status area's corner as `alo-dock` places it.
///
/// # Errors
/// [`RenderError::InUseScene`] when something is in use and the output is too
/// small for `alo-dock` to lay a dock out on, or larger than any output this
/// surface is laid out for. **A frame that cannot carry this indicator is
/// refused rather than drawn without it** — a machine that could not say its
/// camera was on may not put a desktop up instead.
pub(crate) fn picture(
    lines: &[Line],
    strings: &Strings,
    dock: &Dock,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: InUseLook,
) -> Result<InUsePicture, RenderError> {
    let mut picture = InUsePicture {
        size,
        rows: Vec::new(),
        height: 0,
        solids: Vec::new(),
        inked: Vec::new(),
    };
    if lines.is_empty() {
        return Ok(picture);
    }
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::InUseScene);
    }
    let screen = Screen::of(
        u32::try_from(width).map_err(|_| RenderError::InUseScene)?,
        u32::try_from(height).map_err(|_| RenderError::InUseScene)?,
    )
    .map_err(|_| RenderError::InUseScene)?;
    let measure = Measure::of(look.scale);
    let place = Place::of(
        dock.layout_on(screen, look.scale, look.reading),
        size,
        measure.px(8),
    );
    let palette = palette(look.scheme, look.contrast);

    // The crate's order, and only the crate's: by what is in use, keeping the
    // order it gave for two lines about the same thing.
    let mut ordered: Vec<&Line> = lines.iter().collect();
    ordered.sort_by_key(|line| line.position());

    let gap = measure.px(4);
    let fonts = &mut labels.fonts;
    let shaped: Vec<(Laid, &Line)> = ordered
        .into_iter()
        .map(|line| {
            (
                Laid::of(fonts, line.said(strings).text(), place, measure, palette),
                line,
            )
        })
        .collect();

    let mut at_y = place.stacked;
    let mut used = 0;
    for (laid, line) in shaped {
        // A row that would not fit is not drawn, and neither is any after it.
        // **Nothing is elided into a count here**, unlike what is leaving: a
        // line saying the camera is on has no shorter form that still says
        // which thing is on, and *2 things are in use* is not that sentence.
        if used + laid.height > place.room_down {
            break;
        }
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
        used += laid.height + gap;
        let colours = MarkColours::of(line.colour(), look.scheme, look.contrast);
        let dot = line.the_agents_dot();
        let which = line.mark();
        let drawn = laid.place(
            (left.max(0), top.clamp(0, (height - 1).max(0))),
            size,
            measure,
            palette,
            look.reading,
            |at| {
                let mut solids = mark(at, which, colours);
                if dot {
                    solids.extend(the_agents_dot(at, colours, measure));
                }
                solids
            },
        );
        picture.solids.extend(drawn.solids);
        picture.inked.extend(drawn.inked);
        picture.rows.push(drawn.row);
    }
    picture.height = used.max(0);
    Ok(picture)
}

/// ADR 0010's small dot, drawn beside a mark that means the agent.
///
/// It arrives with terracotta or not at all, which is `alo-in-use`'s own rule
/// (`Line::the_agents_dot` is true exactly when the line is terracotta), so
/// this file never decides whether to draw it.
fn the_agents_dot(
    at: Rectangle<i32, Physical>,
    colours: MarkColours,
    measure: Measure,
) -> Vec<Solid> {
    let side = measure.px(4);
    vec![Solid {
        area: Rectangle::new(
            (at.loc.x + at.size.w - side, at.loc.y - side / 2).into(),
            (side, side).into(),
        ),
        colour: colours.fill,
    }]
}

#[cfg(test)]
#[path = "in_use_raster_tests.rs"]
mod tests;
