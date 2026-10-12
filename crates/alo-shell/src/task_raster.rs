//! A task window's furniture, rasterised: the panel, the chips, the reply field
//! and the controls.
//!
//! `alo_tasks::layout` says where the five rows are and `alo_tasks::measures`
//! holds every figure. This turns that into one rounded panel with its shapes
//! drawn into the same raster, which is the only way they get edges that are
//! not staircases — `crate::dock_surface` says why at length, and this goes
//! through its `a_panel` rather than a second rasteriser so a change to the
//! radius or the shadow cannot reach the Dock and miss this.
//!
//! # What is here and what is not
//!
//! **The furniture**: the window's own panel, the shared-file chips, the reply
//! field, the grounds of the four composer controls, and the circle the one
//! Send-or-Stop button sits in. Everything whose place comes from the layout
//! alone.
//!
//! **Not the words, and not the conversation's blocks.** How tall a message is
//! depends on how many lines its text wraps to, so `alo_tasks::TheBlocks` takes
//! heights measured from a font — and nothing here holds one yet. A window
//! drawn by this is a window with its furniture and no text in it, which is
//! **not** a window a person could use, and the next change is the one that
//! shapes text into it.
//!
//! # No display scale
//!
//! Every figure is a logical measure used directly as a pixel, which is
//! `crate::dock_raster`'s documented decision and the reason given there: a
//! scale applied to one measure of an otherwise unscaled geometry is worse than
//! one applied to none. This joins that pending change rather than growing its
//! own conversion.

use alo_tasks::layout::{TheRows, TooSmall};
use alo_tasks::measures as m;
use smithay::utils::{Physical, Rectangle};

use crate::desktop_look::DesktopLook;
use crate::dock_surface::{APanelsShapes, AWell, a_panel};
use crate::painted::Washed;
use crate::task_look::TaskPalette;

/// One task window on a display: where it is, and how much it holds.
///
/// **Where it goes is not decided here.** `docs/design/where-a-new-window-opens.md`
/// is the owner's specification for placement and it is the compositor's to
/// apply — *explicit placement wins*, and *restoring is different from
/// opening*. This takes the rectangle somebody already decided on, so that a
/// change to placement cannot reach the rasteriser and a change to the
/// rasteriser cannot move anybody's window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ATaskOnTheDisplay {
    /// Where the window is, in this display's pixels.
    pub at: smithay::utils::Rectangle<i32, smithay::utils::Physical>,
    /// How many files the person shared with this task, for the row of chips.
    pub how_many_files: usize,
}

/// Why a task window could not be rasterised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoTaskPicture {
    /// The window is too small to lay out, and this is what it would need.
    TooSmall(TooSmall),
    /// The person's accent is not one they may choose.
    AccentRefused,
}

/// A task window's furniture, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskPicture {
    /// The panel and every rounded shape on it, in one raster.
    pub(crate) panel: Washed,
    /// Where the window itself sits.
    pub(crate) at: Rectangle<i32, Physical>,
}

/// A logical measure as a pixel count.
fn px(of: u32) -> i32 {
    i32::try_from(of).unwrap_or(i32::MAX)
}

/// Rasterise the furniture of a task window of this size, at this corner.
///
/// `how_many_files` is how many shared-file chips to draw — the row above the
/// conversation. Taken as a count rather than as names because this draws their
/// grounds and not their labels.
///
/// # Errors
/// [`NoTaskPicture::TooSmall`] for a window the layout refuses, carrying the
/// least it would need; [`NoTaskPicture::AccentRefused`] for an accent nobody
/// may choose.
pub(crate) fn furniture(
    at: Rectangle<i32, Physical>,
    look: DesktopLook,
    how_many_files: usize,
) -> Result<TaskPicture, NoTaskPicture> {
    let (wide, tall) = (
        u32::try_from(at.size.w.max(0)).unwrap_or(0),
        u32::try_from(at.size.h.max(0)).unwrap_or(0),
    );
    let rows = TheRows::of(wide, tall).map_err(NoTaskPicture::TooSmall)?;
    let palette = TaskPalette::of(look).map_err(|_| NoTaskPicture::AccentRefused)?;
    let controls = rows.controls();

    // Everything inside the window is placed from its inside corner, which is
    // one border in on each side.
    let inside = |x: u32, y: u32| {
        (
            at.loc.x + px(m::BORDER) + px(x),
            at.loc.y + px(m::BORDER) + px(y),
        )
    };

    let mut wells: Vec<AWell> = Vec::new();

    // **The shared-file chips**, at `radius/sm`. A row of them, left to right,
    // and none at all when nothing has been shared — which is the honest state
    // of a task somebody asked in plain words with no file attached.
    for which in 0..how_many_files {
        let along = u32::try_from(which).unwrap_or(0) * (m::A_CHIP_IS_WIDE + m::GAP);
        let x = m::BESIDE_A_ROW + along;
        // A chip that would reach past the row is not drawn: the design shows a
        // count beside the chips, and running them off the edge is worse than
        // showing fewer.
        if x + m::A_CHIP_IS_WIDE > rows.shared_files.wide {
            break;
        }
        let down = rows.shared_files.from_the_top
            + (m::THE_SHARED_FILES_ARE_TALL.saturating_sub(m::A_CHIP_IS_TALL)) / 2;
        let (cx, cy) = inside(x, down);
        wells.push(AWell {
            at: Rectangle::new(
                (cx, cy).into(),
                (px(m::A_CHIP_IS_WIDE), px(m::A_CHIP_IS_TALL)).into(),
            ),
            corner: m::A_CHIPS_CORNER,
        });
    }

    // **The reply field**, at 18 — the one well in the window that is not 12,
    // and half the reason a well carries its own corner.
    let field_is_wide = rows.composer.wide.saturating_sub(m::BESIDE_A_ROW * 2);
    let (fx, fy) = inside(
        m::BESIDE_A_ROW,
        rows.composer.from_the_top + m::ABOVE_THE_FIELD,
    );
    wells.push(AWell {
        at: Rectangle::new(
            (fx, fy).into(),
            (px(field_is_wide), px(m::THE_FIELD_IS_TALL)).into(),
        ),
        corner: m::THE_FIELDS_CORNER,
    });

    // **The three square controls**, at `radius/md`. Attach, the model and
    // permissions have a cool ground; *where the processing happens* is text
    // with no ground at all, which is why it is not here.
    for (x, width) in [
        (controls.attach, m::A_CONTROL_IS),
        (controls.model, m::THE_MODEL_CONTROL_IS_WIDE),
        (controls.permissions, m::A_CONTROL_IS),
    ] {
        let (cx, cy) = inside(x, controls.from_the_top);
        wells.push(AWell {
            at: Rectangle::new((cx, cy).into(), (px(width), px(m::A_CONTROL_IS)).into()),
            corner: m::A_CONTROLS_CORNER,
        });
    }

    // **The one button is a circle**, not a well: its radius is half its side,
    // and `circles` is the shape that draws one with an edge rather than a
    // staircase.
    let (bx, by) = inside(controls.the_one_button, controls.from_the_top);
    let button = [Rectangle::new(
        (bx, by).into(),
        (px(m::A_CONTROL_IS), px(m::A_CONTROL_IS)).into(),
    )];

    let panel = a_panel(
        at,
        palette.surface,
        palette.edge,
        (at.loc.x + at.size.w, at.loc.y + at.size.h),
        APanelsShapes {
            circles: &button,
            circle: palette.the_one_button,
            marks: &[],
            mark: palette.ink,
            wells: &wells,
            well_is: palette.cool,
            rings: &[],
            ring_is: palette.ink,
        },
    )
    .ok_or(NoTaskPicture::TooSmall(TooSmall::NotTallEnough {
        the_least: alo_tasks::layout::the_least_height(),
    }))?;

    Ok(TaskPicture { panel, at })
}

#[cfg(test)]
#[path = "task_raster_tests.rs"]
mod tests;
