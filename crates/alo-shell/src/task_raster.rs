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
use crate::task_text::AStyle;

/// One task window on a display: where it is, and how much it holds.
///
/// **Where it goes is not decided here.** `docs/design/where-a-new-window-opens.md`
/// is the owner's specification for placement and it is the compositor's to
/// apply — *explicit placement wins*, and *restoring is different from
/// opening*. This takes the rectangle somebody already decided on, so that a
/// change to placement cannot reach the rasteriser and a change to the
/// rasteriser cannot move anybody's window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ATaskOnTheDisplay<'a> {
    /// Where the window is, in this display's pixels.
    pub at: smithay::utils::Rectangle<i32, smithay::utils::Physical>,
    /// How many files the person shared with this task, for the row of chips.
    pub how_many_files: usize,
    /// Everything a person reads in it that is not the conversation.
    pub words: crate::task_text::TheWordsInATaskWindow<'a>,
    /// What was said in it, oldest first — the order a person reads them in.
    ///
    /// **An empty slice is a window that has just opened**, which is a real
    /// state and not a placeholder: a request opens its window before anything
    /// has answered, so the first thing a person sees is proof the machine
    /// heard them.
    pub said: &'a [crate::task_text::WhatWasSaid<'a>],
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
    /// Its words, painted after its panel.
    ///
    /// **Separate from the panel's raster.** The panel is one texture with its
    /// rounded shapes drawn into it; text is inked onto the ground it sits on
    /// so the edges of the letters blend into that ground rather than into
    /// whatever was behind the window. Two lists, painted in order.
    pub(crate) inked: Vec<crate::painted::Inked>,
}

/// Which style one kind of block is written in.
///
/// A person's words and alo's are both Body/Large — **the same size on purpose**,
/// because one speaker's words drawn smaller than the other's says whose words
/// matter. An account of what was done is Body/Medium, which is the design's own
/// distinction between what was *said* and what was *done*.
const fn style_of(whose: alo_tasks::WhoseBlock) -> AStyle {
    match whose {
        alo_tasks::WhoseBlock::ThePersons | alo_tasks::WhoseBlock::Alos => AStyle::Body,
        alo_tasks::WhoseBlock::AnAccount => AStyle::SmallBody,
    }
}

/// The ground one kind of block is inked onto.
///
/// **Alo's reply has no bubble**, so its ground is the window's own surface;
/// the other two sit on `bg/cool`. Getting this wrong does not misplace
/// anything — it leaves a halo round every letter where the text was blended
/// into a colour that is not behind it.
const fn ground_of(whose: alo_tasks::WhoseBlock, palette: &TaskPalette) -> [u8; 3] {
    match whose {
        alo_tasks::WhoseBlock::Alos => palette.surface,
        alo_tasks::WhoseBlock::ThePersons | alo_tasks::WhoseBlock::AnAccount => palette.cool,
    }
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
    words: &crate::task_text::TheWordsInATaskWindow<'_>,
    said: &[crate::task_text::WhatWasSaid<'_>],
    fonts: &mut cosmic_text::FontSystem,
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

    // **The words, inked onto the ground each sits on.** Each is placed from the
    // window's own inside corner, cut at the window's bottom edge so a long
    // title cannot write past the frame.
    let measure = look.measure();
    let limit = at.loc.y + at.size.h;
    let mut inked = Vec::new();
    // **Measured here, before `write` exists, and that order is forced.** Both
    // the measuring and the writing need the font system mutably, and two
    // closures holding it at once is what the compiler refused — so the
    // measuring finishes and gives the borrow up before the writer takes it.
    //
    // **The conversation, in two passes.** Every block has to be measured
    // before any can be stacked — a message's height depends on how many lines
    // it wrapped to — and only then can each be placed. One pass cannot do it:
    // the second block's position needs the first block's height.
    //
    // `alo_tasks::TheBlocks` owns the stacking, the sides and the widths, and
    // takes the heights it is told. This measures them.
    let row_is_wide = rows.conversation.wide;
    let heights: Vec<(alo_tasks::WhoseBlock, u32)> = said
        .iter()
        .map(|one| {
            let inside = row_is_wide.saturating_sub(m::INSIDE_THE_CONVERSATION * 2);
            let wide = one.whose.how_wide().min(inside);
            // A bubble's padding comes off the width its words get, so a
            // request and a reply of the same length wrap differently — which
            // is what the design draws.
            let for_words = if one.whose.is_in_a_bubble() {
                wide.saturating_sub(m::INSIDE_A_BLOCK * 2)
            } else {
                wide
            };
            let tall = crate::task_text::how_tall(
                fonts,
                crate::task_text::ToWrite {
                    words: one.words,
                    style: style_of(one.whose),
                    ground: ground_of(one.whose, &palette),
                    ink: palette.ink,
                },
                px(for_words),
                measure,
            );
            // A bubble is taller than its words by its padding.
            let tall = if one.whose.is_in_a_bubble() {
                tall + m::INSIDE_A_BLOCK * 2
            } else {
                tall
            };
            (one.whose, tall)
        })
        .collect();

    let blocks = alo_tasks::TheBlocks::stacked(&heights, row_is_wide);
    // **Opened at the newest thing said**, which is where a conversation has to
    // start: opening at the top would show somebody the request they typed an
    // hour ago and hide what alo just answered.
    let view = rows
        .conversation
        .tall
        .saturating_sub(m::INSIDE_THE_CONVERSATION * 2);
    let scrolled = blocks.scrolled_to_the_newest(view);
    let top_of_the_column = rows.conversation.from_the_top + m::INSIDE_THE_CONVERSATION;

    // **Centred in the band it is given, from its own measured height.** Not
    // from a half-of-a-leading constant: the height depends on the font, the
    // person's text size and how many lines it wrapped to, so a constant would
    // be right only at one setting and would drift at the one a person turns on
    // in order to read.
    let mut write = |words: &str,
                     style: AStyle,
                     ground: [u8; 3],
                     ink: [u8; 3],
                     x: u32,
                     band: (u32, u32),
                     width: u32| {
        if words.is_empty() {
            return;
        }
        let shaped = crate::task_text::shaped(
            fonts,
            crate::task_text::ToWrite {
                words,
                style,
                ground,
                ink,
            },
            px(width),
            measure,
        );
        let (top, tall) = band;
        let spare = px(tall) - shaped.height;
        let down = px(top) + if spare > 0 { spare / 2 } else { 0 };
        let (px_x, _) = inside(x, 0);
        let px_y = at.loc.y + i32::try_from(m::BORDER).unwrap_or(0) + down;
        if let Some(one) = shaped.placed(px_x, px_y, limit) {
            inked.push(one);
        }
    };

    // The heading: the title, what kind of task it is, and the status.
    let identity_is_wide = rows.heading.wide.saturating_sub(m::BESIDE_A_ROW * 2);
    write(
        words.title,
        AStyle::Heading,
        palette.surface,
        palette.ink,
        m::BESIDE_A_ROW,
        (
            rows.heading.from_the_top + m::GAP,
            m::THE_HEADING_IS_TALL / 2,
        ),
        identity_is_wide,
    );
    write(
        words.kind,
        AStyle::SmallLabel,
        palette.surface,
        palette.ink_secondary,
        m::BESIDE_A_ROW,
        (
            rows.heading.from_the_top + m::THE_HEADING_IS_TALL / 2,
            m::THE_HEADING_IS_TALL / 2,
        ),
        identity_is_wide,
    );
    // **The status in teal while alo is working**, which is the caller's
    // wording and this crate's colour: ADR 0067's teal marks alo acting.
    write(
        words.status,
        AStyle::Label,
        palette.surface,
        palette.alo,
        m::BESIDE_A_ROW,
        (rows.heading.from_the_top, m::THE_HEADING_IS_TALL),
        identity_is_wide,
    );

    // Each chip's label, on the chip's own cool ground.
    for (which, label) in words.files.iter().enumerate().take(how_many_files) {
        let along = u32::try_from(which).unwrap_or(0) * (m::A_CHIP_IS_WIDE + m::GAP);
        let x = m::BESIDE_A_ROW + along;
        if x + m::A_CHIP_IS_WIDE > rows.shared_files.wide {
            break;
        }
        write(
            label,
            AStyle::Label,
            palette.cool,
            palette.ink,
            x + m::BEFORE_A_CHIPS_LABEL,
            (rows.shared_files.from_the_top, m::THE_SHARED_FILES_ARE_TALL),
            m::A_CHIP_IS_WIDE.saturating_sub(m::BEFORE_A_CHIPS_LABEL * 2),
        );
    }

    // The composer: the invitation, the model's name, and where it happens.
    write(
        words.invitation,
        AStyle::Body,
        palette.surface,
        palette.ink_quiet,
        m::BESIDE_A_ROW + m::INSIDE_THE_FIELD,
        (
            rows.composer.from_the_top + m::ABOVE_THE_FIELD + m::ABOVE_THE_FIELDS_CONTENTS,
            m::THE_FIELD_IS_TALL / 3,
        ),
        field_is_wide.saturating_sub(m::INSIDE_THE_FIELD * 2),
    );
    write(
        words.model,
        AStyle::Label,
        palette.cool,
        palette.ink,
        controls.model + m::GAP,
        (controls.from_the_top, m::A_CONTROL_IS),
        m::THE_MODEL_CONTROL_IS_WIDE.saturating_sub(m::A_CHEVRON + m::GAP * 2),
    );
    write(
        words.destination,
        AStyle::Destination,
        palette.surface,
        palette.ink_quiet,
        controls.destination,
        (controls.from_the_top, m::A_CONTROL_IS),
        m::THE_DESTINATION_IS_WIDE,
    );

    for (one, block) in said.iter().zip(blocks.each()) {
        if !block.is_in_view(scrolled, view) {
            continue;
        }
        let padding = if block.whose.is_in_a_bubble() {
            m::INSIDE_A_BLOCK
        } else {
            0
        };
        // The block's place in the content, less how far the view has scrolled.
        let down = top_of_the_column + block.from_the_top.saturating_sub(scrolled) + padding;
        write(
            one.words,
            style_of(block.whose),
            ground_of(block.whose, &palette),
            palette.ink,
            m::INSIDE_THE_CONVERSATION + block.from_the_left + padding,
            (down, block.tall.saturating_sub(padding * 2)),
            block.wide.saturating_sub(padding * 2),
        );
    }

    // *Permissions and model choices apply to this task.*
    write(
        words.scope_note,
        AStyle::SmallLabel,
        palette.surface,
        palette.ink_quiet,
        m::BESIDE_A_ROW,
        (rows.scope_note.from_the_top, m::THE_SCOPE_NOTE_IS_TALL),
        rows.scope_note.wide.saturating_sub(m::BESIDE_A_ROW * 2),
    );

    Ok(TaskPicture { panel, at, inked })
}

#[cfg(test)]
#[path = "task_raster_tests.rs"]
mod tests;
