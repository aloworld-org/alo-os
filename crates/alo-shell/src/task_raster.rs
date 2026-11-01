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
//! field, the grounds of the four composer controls, the circle the one
//! Send-or-Stop button sits in, and the icons on them — drawn as vectors from
//! the design file's own SVG since 2026-10-30 (`crate::the_reply_boxs_icons`).
//! Everything whose place comes from the layout alone.
//!
//! **Not the words, and not the conversation's blocks.** How tall a message is
//! depends on how many lines its text wraps to, so `alo_tasks::TheBlocks` takes
//! heights measured from a font — and nothing here holds one yet. A window
//! drawn by this is a window with its furniture and no text in it, which is
//! **not** a window a person could use, and the next change is the one that
//! shapes text into it.
//!
//! # At its display's scale
//!
//! The window's rectangle is in its display's pixels. It is laid out in
//! **logical** units — its size divided by the display's scale — so
//! `alo_tasks::layout` and the scroll keep one meaning on every screen, and
//! every logical figure goes back to pixels through the one
//! `crate::display_scale::DisplayScale` the window was handed: the rows, the
//! chips, the field, the controls, the bubbles, the answers, the focus rings,
//! the corner and shadow, and the type, which goes through the person's text
//! size and the display's scale together. Heights and widths measured off
//! shaped words come back in pixels and are taken back to logical units
//! before the layout reads them. On a 200 per cent screen the window is twice
//! the pixels and the same size to the eye as the Dock beside it; at 100 every
//! conversion is the identity and it draws exactly what it drew before.

use alo_tasks::layout::{TheRows, TooSmall};
use alo_tasks::measures as m;
use smithay::utils::{Physical, Rectangle};

use crate::desktop_look::DesktopLook;
use crate::display_scale::DisplayScale;
use crate::dock_surface::{APanelsShapes, AWell, a_panel};
use crate::painted::Washed;
use crate::task_look::TaskPalette;
use crate::task_text::AStyle;
use crate::the_approval_card::{self as card, TheCard};

/// One task window on a display: where it is, and how much it holds.
///
/// **Where it goes is not decided here.** `docs/design/where-a-new-window-opens.md`
/// is the owner's specification for placement and it is the compositor's to
/// apply — *explicit placement wins*, and *restoring is different from
/// opening*. This takes the rectangle somebody already decided on, so that a
/// change to placement cannot reach the rasteriser and a change to the
/// rasteriser cannot move anybody's window.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub said: Vec<crate::task_text::WhatWasSaid<'a>>,
    /// How far the person has scrolled the conversation back from its newest
    /// words.
    pub scroll: alo_tasks::TheScroll,
    /// What its one button shows: the Stop square while alo is working, the
    /// Send arrow otherwise.
    pub the_one_button: alo_tasks::TheOneButton,
    /// Whether a provider answers it, which puts the cloud before the
    /// processing destination.
    pub a_provider_answers: bool,
    /// Its place in the list of tasks, so what this frame measured about it
    /// can be handed back to the right one.
    pub place: usize,
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
    /// How far back from its newest words this conversation can be scrolled,
    /// and how tall its view is — measured here because only a frame that
    /// shaped the words knows either, and handed back to the task through
    /// `alo_tasks::TheScroll::the_frame_measured`.
    pub(crate) the_conversation: (u32, u32),
    /// Which task this is, by its place in the list — set by whoever drew
    /// it from an `ATaskOnTheDisplay`, so the measurement above reaches the
    /// task it is about.
    pub(crate) place: usize,
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

/// The heading's own figures, measured off the design's frame.
///
/// Not in `alo_tasks::measures` because they are this raster's arrangement of
/// one row rather than something another surface asks about, and because the
/// four only mean anything together: `Identity` is 680 × 54 in a row 76 tall,
/// so it begins 11 down; its title is 30, then a gap of 4, then the kind at 16.
/// `Task status` is 260 wide and right aligned.
const THE_IDENTITY_IS_WIDE: u32 = 680;
/// How far down the heading the identity column begins.
const IDENTITY_STARTS: u32 = 11;
/// The title's own line.
const TITLE_IS_TALL: u32 = 30;
/// Between the title and what kind of task it is.
const BETWEEN_THEM: u32 = 4;
/// The line saying what kind of task it is.
const THE_KIND_IS_TALL: u32 = 16;
/// The status's column on the right.
const THE_STATUS_IS_WIDE: u32 = 260;

/// How much taller a card is for its two answers: the gap above the row, and
/// the row.
const THE_ANSWERS_TAKE: u32 = m::GAP + m::A_CONTROL_IS;

/// One line saying what a change does or leaves alone, placed in its card:
/// across and down from where those lines start, as measured while the font
/// system was free.
#[derive(Debug, Clone)]
struct ALineOfWhatItDoes {
    /// The words, already in the person's language.
    words: String,
    /// How far across from the start of the lines its tick is.
    across: u32,
    /// How far down from the start of the lines it is.
    down: u32,
    /// The room its words have.
    wide: u32,
    /// How tall its words are.
    tall: u32,
}

/// Where a card's two answers were placed, measured while the font system was
/// free, for their labels to be centred on them once the panel is made.
#[derive(Debug, Clone, Copy)]
struct TheAnswersPlace {
    /// Allow once's left edge, in the window.
    left: u32,
    /// The row's top, in the window.
    row: u32,
    /// Allow once's label, measured.
    allow_is: u32,
    /// Deny's label, measured.
    deny_is: u32,
    /// Deny's button.
    deny_is_wide: u32,
    /// The ground Deny's label is written on.
    deny_ground: [u8; 3],
}

/// Rasterise the furniture of a task window of this size, at this corner, on a
/// display drawn at `scale`.
///
/// `how_many_files` is how many shared-file chips to draw — the row above the
/// conversation. Taken as a count rather than as names because this draws their
/// grounds and not their labels.
///
/// # Errors
/// [`NoTaskPicture::TooSmall`] for a window the layout refuses, carrying the
/// least it would need; [`NoTaskPicture::AccentRefused`] for an accent nobody
/// may choose.
#[expect(
    clippy::too_many_arguments,
    reason = "what a frame hands over about one window, and the display it is on: each is \
              one fact a caller already holds apart, and a struct for one caller would \
              be a second name for `ATaskOnTheDisplay`"
)]
pub(crate) fn furniture(
    at: Rectangle<i32, Physical>,
    look: DesktopLook,
    how_many_files: usize,
    words: &crate::task_text::TheWordsInATaskWindow<'_>,
    said: &[crate::task_text::WhatWasSaid<'_>],
    scroll: alo_tasks::TheScroll,
    button: alo_tasks::TheOneButton,
    a_cloud: bool,
    fonts: &mut cosmic_text::FontSystem,
    scale: DisplayScale,
) -> Result<TaskPicture, NoTaskPicture> {
    // **A logical measure as this display's pixels**, and the one road every
    // figure below takes to the screen.
    let px = |of: u32| scale.px(of);
    // **And pixels measured off shaped words, back as logical units**, for the
    // layout that reads them.
    let logical = |pixels: i32| scale.logical(pixels);
    let (wide, tall) = (logical(at.size.w), logical(at.size.h));
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
    // **An outline, not a fill.** The design gives the reply field `bg/surface`
    // with a `border/default` hairline, and the panel it sits on is already that
    // colour — so what distinguishes it is its edge. It was a `bg/cool` well
    // until a picture of a real draw showed what that costs: the field read as a
    // grey slab and the three composer controls, which are `bg/cool` themselves,
    // vanished into it. Cool on cool is invisible, and no test was about colour.
    let field = Rectangle::new(
        (fx, fy).into(),
        (px(field_is_wide), px(m::THE_FIELD_IS_TALL)).into(),
    );

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
    // From the one place that says where it is, which a press is tested
    // against too (`crate::a_press_on_the_one_button`).
    let the_button: Vec<Rectangle<i32, Physical>> =
        crate::a_press_on_the_one_button::the_one_buttons_place(at, scale)
            .into_iter()
            .collect();

    let measure = look.measure().on(scale);
    // **Measured before the panel is rasterised, and that order is forced twice
    // over.** Both
    // the measuring and the writing need the font system mutably, and two
    // closures holding it at once is what the compiler refused — so the
    // measuring finishes and gives the borrow up before the writer takes it.
    // And a bubble is a shape **in the panel's own raster**, so the heights
    // that decide where bubbles go have to be known before the panel is made.
    //
    // **The conversation, in two passes.** Every block has to be measured
    // before any can be stacked — a message's height depends on how many lines
    // it wrapped to — and only then can each be placed. One pass cannot do it:
    // the second block's position needs the first block's height.
    //
    // `alo_tasks::TheBlocks` owns the stacking, the sides and the widths, and
    // takes the heights it is told. This measures them.
    let row_is_wide = rows.conversation.wide;
    // **An approval card's action, measured in the heading size it is drawn
    // in** (`crate::the_approval_card`), for the cards there are; [`None`] for
    // every other block.
    // **And where the lines saying what it does and leaves alone go**
    // (`549:209`): side by side as the frame draws them when both fit on one
    // line, one under the other when they do not.
    let mut the_cards: Vec<Option<TheCard>> = Vec::new();
    let mut what_each_does: Vec<Vec<ALineOfWhatItDoes>> = Vec::new();
    for one in said {
        let mut lines = Vec::new();
        let the_card = one
            .answers
            .filter(|answers| answers.asking.is_some())
            .map(|answers| {
                let inside = row_is_wide.saturating_sub(m::INSIDE_THE_CONVERSATION * 2);
                let wide = one.whose.how_wide().min(inside);
                let room = TheCard::the_actions_room(wide);
                let tall = crate::task_text::how_tall(
                    fonts,
                    crate::task_text::ToWrite {
                        words: one.words,
                        marks: one.marks,
                        style: AStyle::Heading,
                        ground: palette.surface,
                        ink: palette.ink,
                    },
                    px(room),
                    measure,
                );
                // Its whole width on one line, and its height wrapped at `width`.
                let mut measured = |words: &str, width: u32| {
                    let natural = crate::painted_text::how_wide_as(
                        fonts,
                        words,
                        AStyle::SmallBody.metrics(measure),
                        &AStyle::SmallBody.attrs(),
                    );
                    let shaped = crate::task_text::shaped(
                        fonts,
                        crate::task_text::ToWrite {
                            words,
                            marks: &[],
                            style: AStyle::SmallBody,
                            ground: palette.surface,
                            ink: palette.ink,
                        },
                        px(width),
                        measure,
                    );
                    (logical(natural), logical(shaped.height))
                };
                let beside = card::A_TICK + card::AFTER_A_TICK;
                let sentences: Vec<&str> = [answers.does.0, answers.does.1]
                    .into_iter()
                    .flatten()
                    .filter(|it| !it.is_empty())
                    .collect();
                // Each at its own width on one line, and then whether both fit.
                let natural: Vec<(u32, u32)> =
                    sentences.iter().map(|it| measured(it, room)).collect();
                let in_a_row = natural.iter().map(|(w, _)| beside + w + 2).sum::<u32>()
                    + card::BETWEEN_WHAT_IT_DOES
                        * u32::try_from(natural.len().saturating_sub(1)).unwrap_or(0);
                let one_line = natural.iter().map(|(_, h)| *h).max().unwrap_or(0);
                let fits = in_a_row <= room;
                let mut across = 0;
                let mut down = 0;
                for (which, words) in sentences.iter().enumerate() {
                    if fits {
                        let (w, _) = natural.get(which).copied().unwrap_or((0, 0));
                        lines.push(ALineOfWhatItDoes {
                            words: (*words).to_owned(),
                            across,
                            down: 0,
                            wide: w + 2,
                            tall: one_line,
                        });
                        across += beside + w + 2 + card::BETWEEN_WHAT_IT_DOES;
                    } else {
                        let width = room.saturating_sub(beside);
                        let (_, h) = measured(words, width);
                        if which > 0 {
                            down += card::UNDER_WHAT_IT_DOES;
                        }
                        lines.push(ALineOfWhatItDoes {
                            words: (*words).to_owned(),
                            across: 0,
                            down,
                            wide: width,
                            tall: h,
                        });
                        down += h;
                    }
                }
                let what_it_does_is_tall = if fits { one_line } else { down };
                TheCard {
                    wide,
                    the_action_is_tall: logical(i32::try_from(tall).unwrap_or(i32::MAX)),
                    rows: u32::try_from(answers.touches.len().min(card::THE_MOST_ROWS))
                        .unwrap_or(0),
                    what_it_does_is_tall,
                }
            });
        the_cards.push(the_card);
        what_each_does.push(lines);
    }
    let heights: Vec<(alo_tasks::WhoseBlock, u32)> = said
        .iter()
        .zip(&the_cards)
        .map(|(one, card)| {
            if let Some(card) = card {
                return (one.whose, card.tall());
            }
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
                    marks: one.marks,
                    style: style_of(one.whose),
                    ground: ground_of(one.whose, &palette),
                    ink: palette.ink,
                },
                px(for_words),
                measure,
            );
            let tall = logical(i32::try_from(tall).unwrap_or(i32::MAX));
            // A bubble is taller than its words by its padding.
            let tall = if one.whose.is_in_a_bubble() {
                tall + m::INSIDE_A_BLOCK * 2
            } else {
                tall
            };
            // **And a card with answers under it, by the choices row and the
            // gap above it**, so the row is inside the card rather than over
            // whatever is said next.
            let tall = if one.answers.is_some() {
                tall + THE_ANSWERS_TAKE
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
    // **And then wherever the person scrolled back to**, which
    // `alo_tasks::TheScroll` holds and which returns to the newest when
    // something new is said.
    let the_newest = blocks.scrolled_to_the_newest(view);
    let scrolled = scroll.the_view_starts(the_newest, said.len());
    let top_of_the_column = rows.conversation.from_the_top + m::INSIDE_THE_CONVERSATION;
    // **The conversation's own view, and nothing in it is drawn outside it.**
    // Until 2026-10-28 a block partly above the view was drawn from the top of
    // it in full, and the words were cut only at the window's bottom edge — so
    // a long answer ran down over the reply box and out of the window, which a
    // walk on a booted machine showed.
    let the_view = (top_of_the_column, top_of_the_column + view);

    // **The bubbles, as rounded shapes rather than the text's own ground.**
    // Until now what looked like a bubble was the box `sentence` inks text
    // into: exactly the height of the words, with no padding and no corner. The
    // design draws a real one at `radius/md` with 16 inside, and a picture of a
    // real draw is what showed the difference — no test could, because every
    // test was about where the words are.
    // **The two answers on the waiting card**, placed before the panel is made
    // because they are shapes in its raster. Allow once is a well in
    // `text/primary`; Deny is its border, drawn as a ring like the reply
    // field's; and the answer the keyboard is on has the navy outline.
    let mut the_answers: Vec<AWell> = Vec::new();
    let mut rings = vec![field];
    let mut outlines: Vec<Rectangle<i32, Physical>> = Vec::new();
    let mut where_the_answers_are: Option<TheAnswersPlace> = None;
    for ((one, block), the_card) in said.iter().zip(blocks.each()).zip(&the_cards) {
        let Some(answers) = one.answers else {
            continue;
        };
        // **On the approved card the answers have a row of their own**, 20 in
        // from the card's side and both 144 wide; on the closing question
        // they sit at the foot of its card as they always have.
        let (in_the_content, inset, deny_is_wide, deny_ground) = match the_card {
            Some(the_card) => (
                block.from_the_top + the_card.the_answers(),
                card::BESIDE,
                card::AN_ANSWER_IS_WIDE,
                palette.surface,
            ),
            None => (
                (block.from_the_top + block.tall)
                    .saturating_sub(m::INSIDE_A_BLOCK + m::A_CONTROL_IS),
                m::INSIDE_A_BLOCK,
                m::DENY_IS_WIDE,
                palette.cool,
            ),
        };
        // **The row of answers is drawn whole or not at all.** Half a button
        // is a target a person can see and cannot be sure of pressing; the
        // keyboard still reaches both while the row is scrolled out of view.
        if in_the_content < scrolled || in_the_content + m::A_CONTROL_IS > scrolled + view {
            continue;
        }
        let left = m::INSIDE_THE_CONVERSATION + block.from_the_left + inset;
        let down = top_of_the_column + in_the_content - scrolled;
        // **Each label measured here, to be centred on its button later.** The
        // font system is free now; once the writer below takes it, it is not.
        let mut width_of = |words: &str, ground: [u8; 3], ink: [u8; 3], room: u32| -> u32 {
            let shaped = crate::task_text::shaped(
                fonts,
                crate::task_text::ToWrite {
                    words,
                    style: AStyle::Label,
                    ground,
                    ink,
                    marks: &[],
                },
                px(room),
                measure,
            );
            logical(shaped.width)
        };
        let allow_is = width_of(
            answers.allow_once,
            palette.ink,
            palette.surface,
            m::ALLOW_ONCE_IS_WIDE,
        );
        let deny_is = width_of(answers.deny, deny_ground, palette.ink, deny_is_wide);
        where_the_answers_are = Some(TheAnswersPlace {
            left,
            row: down,
            allow_is,
            deny_is,
            deny_is_wide,
            deny_ground,
        });
        let (ax, ay) = inside(left, down);
        let allow = Rectangle::new(
            (ax, ay).into(),
            (px(m::ALLOW_ONCE_IS_WIDE), px(m::A_CONTROL_IS)).into(),
        );
        let (dx, dy) = inside(left + m::ALLOW_ONCE_IS_WIDE + m::GAP, down);
        let deny = Rectangle::new(
            (dx, dy).into(),
            (px(deny_is_wide), px(m::A_CONTROL_IS)).into(),
        );
        the_answers.push(AWell {
            at: allow,
            corner: m::A_CONTROLS_CORNER,
        });
        rings.push(deny);
        match answers.selected {
            Some(crate::a_change_waiting_reaches_the_person::WhichAnswer::AllowOnce) => {
                outlines.push(crate::dock_raster::a_focus_ring(allow, scale));
            }
            Some(crate::a_change_waiting_reaches_the_person::WhichAnswer::Deny) => {
                outlines.push(crate::dock_raster::a_focus_ring(deny, scale));
            }
            // **Nothing is preselected, so nothing is outlined** until the
            // person moves to an answer.
            None => {}
        }
    }

    // **The approved card's own grounds**, cut to the view as a bubble is: its
    // hairline, its white, its `bg/canvas` row of choices squared off where it
    // meets the white, and the hairline over that row. Drawn in that order,
    // each over the last (`crate::dock_surface::AGround`).
    let mut grounds: Vec<crate::dock_surface::AGround> = Vec::new();
    for (block, the_card) in blocks.each().iter().zip(&the_cards) {
        let Some(the_card) = the_card else {
            continue;
        };
        if !block.is_in_view(scrolled, view) {
            continue;
        }
        let x = m::INSIDE_THE_CONVERSATION + block.from_the_left;
        let top = top_of_the_column + block.from_the_top;
        // One rounded ground, from `from` down `tall` in the card and `inset`
        // in from its sides, cut to the view — or nothing where none of it is
        // in the view.
        // `across` is where it starts in the card and how wide it is.
        let mut ground_at = |from: u32,
                             tall: u32,
                             across: (u32, u32),
                             corner: u32,
                             colour: [u8; 3]| {
            let starts = (top + from).saturating_sub(scrolled).max(the_view.0);
            let ends = (top + from + tall).saturating_sub(scrolled).min(the_view.1);
            if ends <= starts {
                return;
            }
            let (gx, gy) = inside(x + across.0, starts);
            grounds.push(crate::dock_surface::AGround {
                well: AWell {
                    at: Rectangle::new((gx, gy).into(), (px(across.1), px(ends - starts)).into()),
                    corner,
                },
                colour,
            });
        };
        let wide = the_card.wide;
        let mut ground = |from: u32, tall: u32, inset: u32, corner: u32, colour: [u8; 3]| {
            ground_at(
                from,
                tall,
                (inset, wide.saturating_sub(inset * 2)),
                corner,
                colour,
            );
        };
        let hair = card::A_HAIRLINE;
        let the_white_corner = card::THE_CARDS_CORNER - hair;
        let the_row = the_card.the_choices() + hair;
        let the_rows_height = the_card.tall().saturating_sub(the_row + hair);
        ground(0, the_card.tall(), 0, card::THE_CARDS_CORNER, palette.edge);
        ground(
            hair,
            the_card.tall().saturating_sub(hair * 2),
            hair,
            the_white_corner,
            palette.surface,
        );
        ground(
            the_row,
            the_rows_height,
            hair,
            the_white_corner,
            palette.canvas,
        );
        // Square where the row meets the white: only its foot is rounded.
        ground(
            the_row,
            the_white_corner.min(the_rows_height),
            hair,
            0,
            palette.canvas,
        );
        ground(the_card.the_choices(), hair, hair, 0, palette.edge);
        // **A row for each thing the change touches**, in `bg/cool`, with its
        // white tile (`549:200`).
        for which in 0..the_card.rows {
            let row = the_card.a_row(which);
            ground_at(
                row,
                card::A_ROW_IS_TALL,
                (card::BESIDE, wide.saturating_sub(card::BESIDE * 2)),
                card::A_ROWS_CORNER,
                palette.cool,
            );
            ground_at(
                row + card::INSIDE_A_ROW,
                card::A_TILE_IS,
                (card::BESIDE + card::INSIDE_A_ROW, card::A_TILE_IS),
                card::A_TILES_CORNER,
                palette.surface,
            );
        }
    }

    for (block, the_card) in blocks.each().iter().zip(&the_cards) {
        if !block.whose.is_in_a_bubble() || !block.is_in_view(scrolled, view) || the_card.is_some()
        {
            continue;
        }
        // **Cut to the view**, top and bottom: a bubble scrolled half out of
        // view shows the half that is in it, and never the half over the
        // heading or the reply box.
        let top = (top_of_the_column + block.from_the_top)
            .saturating_sub(scrolled)
            .max(the_view.0);
        let bottom = (top_of_the_column + block.past_its_bottom())
            .saturating_sub(scrolled)
            .min(the_view.1);
        if bottom <= top {
            continue;
        }
        let (bx, by) = inside(m::INSIDE_THE_CONVERSATION + block.from_the_left, top);
        wells.push(AWell {
            at: Rectangle::new((bx, by).into(), (px(block.wide), px(bottom - top)).into()),
            corner: m::A_BLOCKS_CORNER,
        });
    }

    let mut panel = a_panel(
        at,
        palette.surface,
        palette.edge,
        (at.loc.x + at.size.w, at.loc.y + at.size.h),
        crate::how_a_panel_is_raised::HowItIsRaised::THE_DOCKS.at(scale.factor()),
        APanelsShapes {
            circles: &the_button,
            circle: palette.the_one_button,
            marks: &[],
            mark: palette.ink,
            wells: &wells,
            well_is: palette.cool,
            rings: &rings,
            ring_is: palette.edge,
            in_primary: crate::dock_surface::InPrimary {
                wells: &the_answers,
                outlines: &outlines,
                colour: palette.ink,
                grounds: &grounds,
            },
        },
    )
    .ok_or(NoTaskPicture::TooSmall(TooSmall::NotTallEnough {
        the_least: alo_tasks::layout::the_least_height(),
    }))?;

    // **The model's name and its arrow, centred together in their control** —
    // the design's `Model` is a row with 8 inside each end and 8 between, its
    // contents centred. The name is measured here, while the font system is
    // free, so the arrow can sit right after it. With no name, which is every
    // task today (`crate::the_tasks_words` says why), the arrow is centred.
    let the_names_room = m::THE_MODEL_CONTROL_IS_WIDE.saturating_sub(m::GAP * 3 + m::A_CHEVRON);
    let the_names_width = if words.model.is_empty() {
        0
    } else {
        let shaped = crate::task_text::shaped(
            fonts,
            crate::task_text::ToWrite {
                words: words.model,
                style: AStyle::Label,
                ground: palette.cool,
                ink: palette.ink,
                marks: &[],
            },
            px(the_names_room),
            measure,
        );
        logical(shaped.width).min(the_names_room)
    };
    let the_models_row = if the_names_width == 0 {
        m::A_CHEVRON
    } else {
        the_names_width + m::GAP + m::A_CHEVRON
    };
    let the_name_starts =
        controls.model + m::THE_MODEL_CONTROL_IS_WIDE.saturating_sub(the_models_row) / 2;
    let the_chevron_starts = if the_names_width == 0 {
        the_name_starts
    } else {
        the_name_starts + the_names_width + m::GAP
    };

    // **The icons, drawn as vectors into the panel's own raster** at this
    // display's scale (`crate::the_reply_boxs_icons`). Each is centred in its
    // control; the chevron in a box of its own size beside the name.
    {
        use crate::the_reply_boxs_icons::{AnIcon, ink_an_icon};
        let a_control = |x: u32, wide: u32| {
            let (cx, cy) = inside(x, controls.from_the_top);
            Rectangle::new((cx, cy).into(), (px(wide), px(m::A_CONTROL_IS)).into())
        };
        for (icon, control) in [
            (AnIcon::Attach, a_control(controls.attach, m::A_CONTROL_IS)),
            (
                AnIcon::TheModelsChevron,
                a_control(the_chevron_starts, m::A_CHEVRON),
            ),
            (
                AnIcon::Permissions,
                a_control(controls.permissions, m::A_CONTROL_IS),
            ),
        ] {
            ink_an_icon(&mut panel, icon, control, scale, palette.ink);
        }
        for circle in &the_button {
            ink_an_icon(
                &mut panel,
                AnIcon::on_the_one_button(button),
                *circle,
                scale,
                palette.on_the_one_button,
            );
        }
        // **The cloud before the processing destination, while a provider
        // answers** — the approved window of 2026-10-31 (`549:113`), in the
        // destination's own quiet ink. Not drawn for an answer on this machine
        // or the network: the design draws the cloud only beside a provider,
        // and a cloud beside *On this computer* would say the opposite.
        if a_cloud {
            ink_an_icon(
                &mut panel,
                AnIcon::Cloud,
                a_control(controls.destination, m::THE_DESTINATIONS_ICON),
                scale,
                palette.ink_quiet,
            );
        }
    }

    // **alo's mark on each approval card**, beside *alo is asking to* and
    // centred on that line — drawn whole or not at all, as the answers are.
    for (block, the_card) in blocks.each().iter().zip(&the_cards) {
        let Some(the_card) = the_card else {
            continue;
        };
        let line = block.from_the_top + the_card.who_is_asking();
        if line < scrolled || line + card::WHO_IS_ASKING_IS_TALL > scrolled + view {
            continue;
        }
        let down = top_of_the_column + line - scrolled
            + (card::WHO_IS_ASKING_IS_TALL - card::THE_MARK_IS) / 2;
        let (mx, my) = inside(
            m::INSIDE_THE_CONVERSATION + block.from_the_left + card::BESIDE,
            down,
        );
        crate::the_approval_cards_icons::ink_a_cards_icon(
            &mut panel,
            crate::the_approval_cards_icons::ACardsIcon::TheMark,
            Rectangle::new(
                (mx, my).into(),
                (px(card::THE_MARK_IS), px(card::THE_MARK_IS)).into(),
            ),
            palette.alo,
        );
    }

    // **The file icon in each row's tile**, at the tile's 20 and in its
    // proportions, drawn whole or not at all.
    for ((one, block), the_card) in said.iter().zip(blocks.each()).zip(&the_cards) {
        let (Some(the_card), Some(answers)) = (the_card, one.answers) else {
            continue;
        };
        for (which, row) in (0..the_card.rows).zip(answers.touches) {
            if !row.is_a_path {
                continue;
            }
            let tile = block.from_the_top + the_card.a_row(which) + card::INSIDE_A_ROW;
            if tile < scrolled || tile + card::A_TILE_IS > scrolled + view {
                continue;
            }
            let Some(shape) = crate::the_approval_cards_icons::ACardsIcon::AFile.shape() else {
                continue;
            };
            // The file's own box, drawn so its taller side is the tile's icon.
            let (wide, tall) = shape.its_box();
            let side = px(card::THE_ICON_IN_A_TILE);
            #[expect(
                clippy::cast_possible_truncation,
                reason = "an icon is a few dozen pixels at any display's scale"
            )]
            let (iw, ih) = if tall >= wide {
                (
                    (f64::from(side) * f64::from(wide / tall)).round() as i32,
                    side,
                )
            } else {
                (
                    side,
                    (f64::from(side) * f64::from(tall / wide)).round() as i32,
                )
            };
            let tile_left = m::INSIDE_THE_CONVERSATION
                + block.from_the_left
                + card::BESIDE
                + card::INSIDE_A_ROW;
            let (tx, ty) = inside(tile_left, top_of_the_column + tile - scrolled);
            let tile_is = px(card::A_TILE_IS);
            crate::the_approval_cards_icons::ink_a_cards_icon(
                &mut panel,
                crate::the_approval_cards_icons::ACardsIcon::AFile,
                Rectangle::new(
                    (tx + (tile_is - iw) / 2, ty + (tile_is - ih) / 2).into(),
                    (iw, ih).into(),
                ),
                palette.ink_secondary,
            );
        }
    }

    // **The tick before each line saying what the change does and leaves
    // alone**, centred on its first line, drawn whole or not at all.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a line of text is a few dozen pixels at any text size"
    )]
    let a_line = logical(AStyle::SmallBody.metrics(measure).line_height.round() as i32);
    for ((block, the_card), lines) in blocks.each().iter().zip(&the_cards).zip(&what_each_does) {
        let Some(the_card) = the_card else {
            continue;
        };
        let Some(tick) = crate::the_approval_cards_icons::ACardsIcon::ATick.shape() else {
            continue;
        };
        let (wide, tall) = tick.its_box();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "an icon is a few dozen pixels at any display's scale"
        )]
        let (tw, th) = (
            (f64::from(wide) * scale.factor()).round() as i32,
            (f64::from(tall) * scale.factor()).round() as i32,
        );
        for line in lines {
            let at = block.from_the_top + the_card.what_it_does() + line.down;
            let centred = at + a_line.min(line.tall).saturating_sub(card::A_TICK) / 2;
            if centred < scrolled || centred + card::A_TICK > scrolled + view {
                continue;
            }
            let (bx, by) = inside(
                m::INSIDE_THE_CONVERSATION + block.from_the_left + card::BESIDE + line.across,
                top_of_the_column + centred - scrolled,
            );
            let box_is = px(card::A_TICK);
            crate::the_approval_cards_icons::ink_a_cards_icon(
                &mut panel,
                crate::the_approval_cards_icons::ACardsIcon::ATick,
                Rectangle::new(
                    (bx + (box_is - tw) / 2, by + (box_is - th) / 2).into(),
                    (tw, th).into(),
                ),
                palette.ink_secondary,
            );
        }
    }

    // **The words, inked onto the ground each sits on.** Each is placed from the
    // window's own inside corner, cut at the window's bottom edge so a long
    // title cannot write past the frame.
    let limit = at.loc.y + at.size.h;
    let mut inked = Vec::new();

    // **The title keeps to its one line, and ends in an ellipsis when it must.**
    // The design's identity is 680 × 54: one Heading line of 30, a gap of 4 and
    // the kind at 16, so a second line has nowhere to go but over the kind —
    // which a walk on a booted machine showed a long request doing. Fitted the
    // way the window edge fits a title (the owner's ruling of 2026-10-10 in
    // `docs/design/the-external-window-edge.md`): grapheme-safe, measured in the
    // face it is drawn in, and to the room this display's window actually has.
    // The whole request is never lost: it is the first thing in the
    // conversation, drawn in full.
    let identity_is_wide = rows.heading.wide.saturating_sub(m::BESIDE_A_ROW * 2);
    let the_titles_room = THE_IDENTITY_IS_WIDE.min(identity_is_wide);
    let one_line = words.title.split_whitespace().collect::<Vec<_>>().join(" ");
    // One short of the room, so the line the writer shapes at the full room
    // can never wrap on a rounding of the measurement.
    let title = crate::window_edge_title::fitted(
        fonts,
        &one_line,
        px(the_titles_room).saturating_sub(1),
        AStyle::Heading.metrics(measure),
    );
    // **Each row's name and where it is, one line each**, fitted as the title
    // is and in the face each is drawn in: a long path ends in an ellipsis
    // rather than running over the row beneath it. Fitted here because the
    // writer below holds the font system.
    let mut the_rows_shown: Vec<Vec<(String, String)>> = Vec::new();
    for (one, the_card) in said.iter().zip(&the_cards) {
        let mut rows = Vec::new();
        if let (Some(the_card), Some(answers)) = (the_card, one.answers) {
            let room = px(the_card.a_rows_words().1).saturating_sub(1);
            for row in answers.touches.iter().take(card::THE_MOST_ROWS) {
                let mut fit = |style: AStyle, text: &str| {
                    crate::window_edge_title::fitted_as(
                        fonts,
                        text,
                        room,
                        style.metrics(measure),
                        &style.attrs(),
                    )
                    .shown
                };
                let name = fit(AStyle::AFilesName, &row.name);
                let where_it_is = fit(AStyle::WhereAFileIs, &row.where_it_is);
                rows.push((name, where_it_is));
            }
        }
        the_rows_shown.push(rows);
    }

    // **Centred in the band it is given, from its own measured height.** Not
    // from a half-of-a-leading constant: the height depends on the font, the
    // person's text size and how many lines it wrapped to, so a constant would
    // be right only at one setting and would drift at the one a person turns on
    // in order to read.
    // **`to_the_right` is how the status sits against the far edge.** The design
    // puts the identity in a 680 column on the left and the status in a 260 one
    // on the right, `justify-end`. Everything was drawn left-aligned until a
    // picture showed all three heading lines stacked on top of one another, the
    // status's own opaque ink box painting over the title's lower half — which
    // read as the title being clipped and was not.
    let mut write = |words: &str,
                     marks: &[crate::alos_marked_words::AMark],
                     style: AStyle,
                     ground: [u8; 3],
                     ink: [u8; 3],
                     x: u32,
                     band: (u32, u32),
                     width: u32,
                     to_the_right: bool,
                     scrolled_in: Option<(u32, (u32, u32))>| {
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
                marks,
            },
            px(width),
            measure,
        );
        let (top, tall) = band;
        let spare = px(tall) - shaped.height;
        // **Lifted by how far the conversation is scrolled**, for words in it.
        let lifted = scrolled_in.map_or(0, |(by, _)| px(by));
        let down = px(top) - lifted + if spare > 0 { spare / 2 } else { 0 };
        // **Right-aligned text is shaped twice**, and the second time at its own
        // width. `sentence` inks into a box exactly as wide as it was told, so
        // shifting that box to the right edge leaves it hanging past it — which
        // is what a picture showed: the status's opaque ground ran 143 pixels
        // beyond the window. Measuring, then re-shaping at the measured width,
        // gives a box the size of the words.
        let (shaped, along) = if to_the_right {
            let fits = shaped.width.max(1);
            let again = crate::task_text::shaped(
                fonts,
                crate::task_text::ToWrite {
                    words,
                    style,
                    ground,
                    ink,
                    marks,
                },
                fits,
                measure,
            );
            let slack = logical(px(width) - fits);
            (again, x + slack)
        } else {
            (shaped, x)
        };
        let (px_x, _) = inside(along, 0);
        let px_y = at.loc.y + px(m::BORDER) + down;
        // **Cut to the conversation's view** for words in it, and to the
        // window's bottom edge for everything else.
        let (from, to) = scrolled_in.map_or((at.loc.y, limit), |(_, (top, bottom))| {
            let (_, top) = inside(0, top);
            let (_, bottom) = inside(0, bottom);
            (top, bottom.min(limit))
        });
        if let Some(one) = shaped.placed_between(px_x, px_y, from, to) {
            inked.push(one);
        }
    };

    // The heading: the title, what kind of task it is, and the status.
    write(
        &title.shown,
        &[],
        AStyle::Heading,
        palette.surface,
        palette.ink,
        m::BESIDE_A_ROW,
        (rows.heading.from_the_top + IDENTITY_STARTS, TITLE_IS_TALL),
        the_titles_room,
        false,
        None,
    );
    write(
        words.kind,
        &[],
        AStyle::SmallLabel,
        palette.surface,
        palette.ink_secondary,
        m::BESIDE_A_ROW,
        (
            rows.heading.from_the_top + IDENTITY_STARTS + TITLE_IS_TALL + BETWEEN_THEM,
            THE_KIND_IS_TALL,
        ),
        THE_IDENTITY_IS_WIDE.min(identity_is_wide),
        false,
        None,
    );
    // **The status in teal while alo is working, and only then**, which is the
    // caller's wording and this crate's colour: ADR 0067's teal marks alo
    // acting. *Complete* was drawn teal too until 2026-11-01; the approved
    // design draws it, and *Waiting for your permission*, in navy.
    write(
        words.status,
        &[],
        AStyle::Label,
        palette.surface,
        if words.alo_is_working {
            palette.alo
        } else {
            palette.ink
        },
        // **In its own column on the right**, which is what the design draws
        // and what stops it painting over the title.
        rows.heading
            .wide
            .saturating_sub(m::BESIDE_A_ROW + THE_STATUS_IS_WIDE),
        (rows.heading.from_the_top, m::THE_HEADING_IS_TALL),
        THE_STATUS_IS_WIDE,
        true,
        None,
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
            &[],
            AStyle::Label,
            palette.cool,
            palette.ink,
            x + m::BEFORE_A_CHIPS_LABEL,
            (rows.shared_files.from_the_top, m::THE_SHARED_FILES_ARE_TALL),
            m::A_CHIP_IS_WIDE.saturating_sub(m::BEFORE_A_CHIPS_LABEL * 2),
            false,
            None,
        );
    }

    // The composer: the invitation, the model's name, and where it happens.
    write(
        words.invitation,
        &[],
        AStyle::Body,
        palette.surface,
        palette.ink_quiet,
        m::BESIDE_A_ROW + m::INSIDE_THE_FIELD,
        (
            rows.composer.from_the_top + m::ABOVE_THE_FIELD + m::ABOVE_THE_FIELDS_CONTENTS,
            m::THE_FIELD_IS_TALL / 3,
        ),
        field_is_wide.saturating_sub(m::INSIDE_THE_FIELD * 2),
        false,
        None,
    );
    write(
        words.model,
        &[],
        AStyle::Label,
        palette.cool,
        palette.ink,
        the_name_starts,
        (controls.from_the_top, m::A_CONTROL_IS),
        the_names_room,
        false,
        None,
    );
    // **After the cloud, when there is one** (`549:112`): the icon frame, then
    // six, then the words, in the room that is left.
    let the_words_start = if a_cloud {
        m::THE_DESTINATIONS_ICON + m::BESIDE_THE_DESTINATIONS_ICON
    } else {
        0
    };
    write(
        words.destination,
        &[],
        AStyle::Destination,
        palette.surface,
        palette.ink_quiet,
        controls.destination + the_words_start,
        (controls.from_the_top, m::A_CONTROL_IS),
        m::THE_DESTINATION_IS_WIDE.saturating_sub(the_words_start),
        false,
        None,
    );

    for ((((one, block), the_card), rows_shown), its_lines) in said
        .iter()
        .zip(blocks.each())
        .zip(&the_cards)
        .zip(&the_rows_shown)
        .zip(&what_each_does)
    {
        if !block.is_in_view(scrolled, view) {
            continue;
        }
        if let (Some(the_card), Some(answers)) = (the_card, one.answers) {
            // **The approved card**: who is asking, then the action in heading
            // size, each cut to the view like any words in the conversation.
            let top = top_of_the_column + block.from_the_top;
            let left = m::INSIDE_THE_CONVERSATION + block.from_the_left;
            let room = TheCard::the_actions_room(the_card.wide);
            let after_the_mark = card::THE_MARK_IS + card::AFTER_THE_MARK;
            write(
                answers.asking.unwrap_or_default(),
                &[],
                AStyle::SmallLabel,
                palette.surface,
                palette.alo,
                left + card::BESIDE + after_the_mark,
                (top + the_card.who_is_asking(), card::WHO_IS_ASKING_IS_TALL),
                room.saturating_sub(after_the_mark),
                false,
                Some((scrolled, the_view)),
            );
            write(
                one.words,
                one.marks,
                AStyle::Heading,
                palette.surface,
                palette.ink,
                left + card::BESIDE,
                (top + the_card.the_action(), the_card.the_action_is_tall),
                room,
                false,
                Some((scrolled, the_view)),
            );
            // **Each thing it touches**: its name, and under it where it is,
            // on the row's `bg/cool`, centred together in the row as the
            // frame draws them. An application has a name and no place.
            let (words_start, words_room) = the_card.a_rows_words();
            for (which, (name, where_it_is)) in (0..the_card.rows).zip(rows_shown) {
                let row_top = top + the_card.a_row(which);
                let (name_top, name_tall) = if where_it_is.is_empty() {
                    (row_top, card::A_ROW_IS_TALL)
                } else {
                    let both = card::A_NAME_IS_TALL + card::UNDER_THE_NAME + card::WHERE_IT_IS_TALL;
                    (
                        row_top + (card::A_ROW_IS_TALL - both) / 2,
                        card::A_NAME_IS_TALL,
                    )
                };
                write(
                    name,
                    &[],
                    AStyle::AFilesName,
                    palette.cool,
                    palette.ink,
                    left + words_start,
                    (name_top, name_tall),
                    words_room,
                    false,
                    Some((scrolled, the_view)),
                );
                write(
                    where_it_is,
                    &[],
                    AStyle::WhereAFileIs,
                    palette.cool,
                    palette.ink_quiet,
                    left + words_start,
                    (
                        name_top + card::A_NAME_IS_TALL + card::UNDER_THE_NAME,
                        card::WHERE_IT_IS_TALL,
                    ),
                    words_room,
                    false,
                    Some((scrolled, the_view)),
                );
            }
            // **What it does and what it leaves alone**, each after its tick,
            // in Body/Medium.
            for line in its_lines {
                write(
                    &line.words,
                    &[],
                    AStyle::SmallBody,
                    palette.surface,
                    palette.ink,
                    left + card::BESIDE + line.across + card::A_TICK + card::AFTER_A_TICK,
                    (top + the_card.what_it_does() + line.down, line.tall),
                    line.wide,
                    false,
                    Some((scrolled, the_view)),
                );
            }
            if let Some(place) = where_the_answers_are {
                let centred = |button: u32, label: u32| button.saturating_sub(label) / 2;
                write(
                    answers.allow_once,
                    &[],
                    AStyle::Label,
                    palette.ink,
                    palette.surface,
                    place.left + centred(m::ALLOW_ONCE_IS_WIDE, place.allow_is),
                    (place.row, m::A_CONTROL_IS),
                    (place.allow_is + 2).min(m::ALLOW_ONCE_IS_WIDE),
                    false,
                    None,
                );
                let deny_starts = place.left + m::ALLOW_ONCE_IS_WIDE + m::GAP;
                write(
                    answers.deny,
                    &[],
                    AStyle::Label,
                    place.deny_ground,
                    palette.ink,
                    deny_starts + centred(place.deny_is_wide, place.deny_is),
                    (place.row, m::A_CONTROL_IS),
                    (place.deny_is + 2).min(place.deny_is_wide),
                    false,
                    None,
                );
                // And beside them, what allowing once means.
                // **Only where it fits on the row**: on a window too narrow
                // for it, the answers keep the row and this is not written.
                let (beside, its_room) = the_card.the_line_beside();
                let once = answers
                    .once
                    .filter(|_| its_room >= card::THE_LEAST_ROOM_BESIDE)
                    .unwrap_or_default();
                write(
                    once,
                    &[],
                    AStyle::SmallBody,
                    palette.canvas,
                    palette.ink_quiet,
                    left + beside,
                    (place.row, m::A_CONTROL_IS),
                    its_room,
                    true,
                    None,
                );
            }
            continue;
        }
        let padding = if block.whose.is_in_a_bubble() {
            m::INSIDE_A_BLOCK
        } else {
            0
        };
        // The block's place in the content; the writer lifts it by how far the
        // view has scrolled and cuts it to the view.
        let down = top_of_the_column + block.from_the_top + padding;
        // **The words keep to the space above the answers.** They are centred in
        // the band they are given, so a band that included the row would put
        // the sentence on top of the buttons.
        let below = if one.answers.is_some() {
            THE_ANSWERS_TAKE
        } else {
            0
        };
        write(
            one.words,
            one.marks,
            style_of(block.whose),
            ground_of(block.whose, &palette),
            palette.ink,
            m::INSIDE_THE_CONVERSATION + block.from_the_left + padding,
            (down, block.tall.saturating_sub(padding * 2 + below)),
            block.wide.saturating_sub(padding * 2),
            false,
            Some((scrolled, the_view)),
        );
        // **The two labels, in the design's Label/Medium**, each on its own
        // button's ground: white on the navy of Allow once, navy on the card
        // under Deny.
        if let (
            Some(answers),
            Some(TheAnswersPlace {
                left,
                row,
                allow_is,
                deny_is,
                ..
            }),
        ) = (one.answers, where_the_answers_are)
        {
            // Centred on the button: the label's own width, measured above,
            // and the half of what is left over on each side. Two pixels of
            // room so the measured width never wraps when shaped again.
            let centred = |button: u32, label: u32| button.saturating_sub(label) / 2;
            write(
                answers.allow_once,
                &[],
                AStyle::Label,
                palette.ink,
                palette.surface,
                left + centred(m::ALLOW_ONCE_IS_WIDE, allow_is),
                (row, m::A_CONTROL_IS),
                (allow_is + 2).min(m::ALLOW_ONCE_IS_WIDE),
                false,
                None,
            );
            let deny_starts = left + m::ALLOW_ONCE_IS_WIDE + m::GAP;
            write(
                answers.deny,
                &[],
                AStyle::Label,
                palette.cool,
                palette.ink,
                deny_starts + centred(m::DENY_IS_WIDE, deny_is),
                (row, m::A_CONTROL_IS),
                (deny_is + 2).min(m::DENY_IS_WIDE),
                false,
                None,
            );
        }
    }

    // *Permissions and model choices apply to this task.*
    write(
        words.scope_note,
        &[],
        AStyle::SmallLabel,
        palette.surface,
        palette.ink_quiet,
        m::BESIDE_A_ROW,
        (rows.scope_note.from_the_top, m::THE_SCOPE_NOTE_IS_TALL),
        rows.scope_note.wide.saturating_sub(m::BESIDE_A_ROW * 2),
        false,
        None,
    );

    Ok(TaskPicture {
        panel,
        at,
        inked,
        the_conversation: (the_newest, view),
        place: 0,
    })
}

#[cfg(test)]
#[path = "task_raster_tests.rs"]
mod tests;

/// The task window at a display's scale against the same window at one.
#[cfg(test)]
#[path = "task_raster_scale_tests.rs"]
mod scale_tests;

/// The reply box's icons, on the window as it is drawn.
#[cfg(test)]
#[path = "task_raster_icons_tests.rs"]
mod icons_tests;
