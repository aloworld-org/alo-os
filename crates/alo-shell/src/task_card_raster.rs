//! **A running task's square and card in the right-hand panel, drawn** — the
//! owner's approved design `07 / A running task in the put-aside panel`
//! (`23:2`, *"Approve as drawn"*, 2026-11-11).
//!
//! `crate::panel_raster` lays the rail out and says which slot is a task's;
//! this file draws into those slots and, where the person pressed a square,
//! the 216 × 106 card beside the rail. Every figure is a logical measure from
//! `crate::a_task_card_measures`, taken through the person's text size and the
//! display's scale before it reaches a pixel.
//!
//! # Never colour alone
//!
//! Each state is also a word: the card's first line says it, and the square's
//! name for a screen reader says it (`crate::a_task_in_the_panel`). The colours
//! are the design's roles, and **teal only while alo is acting** (ADR 0067).
//!
//! # Two departures from the drawing, written down
//!
//! - **alo's mark is drawn at SemiBold, not Bold.** No Manrope Bold is bundled
//!   (`fonts/README.md`), and asking cosmic-text for a weight no face has
//!   leaves the family and draws Inter; a mark in the wrong typeface is worse
//!   than one a step lighter.
//! - **The light palette in both schemes.** The frame is drawn in light only,
//!   and a dark card is a design nobody has drawn yet.

use alo_appearance::Role;
use alo_dock::measures::MARGIN;
use alo_put_aside::TaskNumber;
use alo_put_aside::the_region_the_panel_claims::WhichEdge;
use alo_tasks::{HowItStands, TheCardsButton};
use cosmic_text::{FontSystem, Metrics, Weight};
use smithay::utils::{Physical, Rectangle};

use crate::a_task_card_measures as m;
use crate::a_task_in_the_panel::ATaskAsideOnTheDisplay;
use crate::desktop_look::Measure;
use crate::painted_text::{Shaped, alos_own_text, sentence_in_runs};
use crate::panel_raster::PanelPicture;
use crate::settings_canvas::{AShadow, Area, Canvas};

/// The card a person opened from a task's square, as drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheOpenCard {
    /// Which task's card it is.
    pub(crate) task: TaskNumber,
    /// The card, in display pixels.
    pub(crate) area: Rectangle<i32, Physical>,
    /// Its button and where it is, or [`None`] while a Stop is on its way.
    pub(crate) button: Option<(TheCardsButton, Rectangle<i32, Physical>)>,
}

/// A role's colour as three bytes.
fn rgb(role: Role) -> [u8; 3] {
    let colour = role.colour();
    [colour.red(), colour.green(), colour.blue()]
}

/// The colour of a card's first line and left edge, and of alo's mark on it.
fn the_cards_colours(stands: HowItStands) -> (Role, Role) {
    match stands {
        HowItStands::Working => (Role::AccentDefault, Role::AccentDefault),
        HowItStands::NeedsYourOk => (Role::TextPrimary, Role::TextPrimary),
        HowItStands::Stopping => (Role::TextMuted, Role::TextMuted),
        HowItStands::Stopped { .. } => (Role::BorderDefault, Role::TextSecondary),
        HowItStands::StopNotConfirmed => (Role::StatusWarning, Role::StatusWarning),
        HowItStands::Done => (Role::BorderDefault, Role::StatusPositive),
    }
}

/// How a square's ring is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ring {
    /// Unbroken.
    Solid,
    /// In dashes: a Stop on its way, or one not confirmed.
    Dashed,
}

/// A square's ground, ring colour, ring width, ring kind and mark colour.
fn the_squares_look(stands: HowItStands) -> (Role, Role, u32, Ring, Role) {
    match stands {
        HowItStands::Working => (
            Role::AccentSoft,
            Role::AccentDefault,
            m::STRONG_RING,
            Ring::Solid,
            Role::AccentDefault,
        ),
        HowItStands::NeedsYourOk => (
            Role::BgSurface,
            Role::TextPrimary,
            m::STRONG_RING,
            Ring::Solid,
            Role::TextPrimary,
        ),
        HowItStands::Stopping => (
            Role::BgSurface,
            Role::TextMuted,
            m::QUIET_RING,
            Ring::Dashed,
            Role::TextMuted,
        ),
        // **Not drawn in the frame**, which shows the rail for four states:
        // the dashes of a Stop still in the air, in the amber the card's own
        // edge uses for this state.
        HowItStands::StopNotConfirmed => (
            Role::BgSurface,
            Role::StatusWarning,
            m::QUIET_RING,
            Ring::Dashed,
            Role::StatusWarning,
        ),
        HowItStands::Stopped { .. } | HowItStands::Done => (
            Role::BgSurface,
            Role::BorderDefault,
            m::QUIET_RING,
            Ring::Solid,
            Role::TextMuted,
        ),
    }
}

/// One logical measure in this display's pixels.
fn px(measure: Measure, logical: u32) -> i32 {
    measure.px(i32::try_from(logical).unwrap_or(i32::MAX))
}

/// One logical measure in this display's pixels, as a float for shapes.
#[expect(
    clippy::cast_precision_loss,
    reason = "a measure in pixels is a few hundred at most, well inside an f32"
)]
fn fx(measure: Measure, logical: u32) -> f32 {
    px(measure, logical) as f32
}

/// Words at a weight and type size, inked on `ground`, cut to `wide`, one
/// line, ending in an ellipsis when they do not fit.
#[expect(
    clippy::too_many_arguments,
    reason = "each is one thing the design sets for a line: what, how big, how heavy, how \
              wide, on what and in what"
)]
fn one_line(
    fonts: &mut FontSystem,
    text: &str,
    measure: Measure,
    (size, tall): (u32, u32),
    weight: Weight,
    wide: i32,
    ground: [u8; 3],
    ink: [u8; 3],
) -> Shaped {
    let metrics = Metrics::new(fx(measure, size), fx(measure, tall));
    let fits = crate::painted_text::how_wide(fonts, text, metrics) <= wide;
    let line = if fits {
        text.to_owned()
    } else {
        crate::painted_text_wrapping::ending_in_an_ellipsis(text, wide, fonts, metrics)
    };
    let attrs = alos_own_text().weight(weight);
    sentence_in_runs(
        fonts,
        &[(line.as_str(), attrs.clone())],
        &attrs,
        wide.max(1),
        metrics,
        ground,
        ink,
    )
    .cropped_to_its_ink()
}

/// alo's mark, centred in `square`.
fn the_mark(canvas: &mut Canvas, fonts: &mut FontSystem, square: Area, measure: Measure, ground: [u8; 3], ink: [u8; 3]) {
    let mark = one_line(
        fonts,
        "alo",
        measure,
        (m::MARK_TYPE, m::MARK_TYPE + m::MARK_TYPE / 2),
        crate::painted_text::ALOS_OWN_WEIGHT,
        square.size.w,
        ground,
        ink,
    );
    let x = square.loc.x + (square.size.w - mark.width_of_box) / 2;
    let y = square.loc.y + (square.size.h - mark.height) / 2;
    canvas.words(x, y, &mark);
}

/// A dashed rounded ring: the ring drawn whole, kept only inside every other
/// stretch of `dash` along each side.
fn a_dashed_ring(canvas: &mut Canvas, at: Area, corner: f32, width: f32, colour: [u8; 3], dash: i32) {
    let dash = dash.max(1);
    let mut stretches = Vec::new();
    let mut along = 0;
    while along < at.size.w {
        let wide = dash.min(at.size.w - along);
        stretches.push(Rectangle::new((at.loc.x + along, at.loc.y).into(), (wide, at.size.h).into()));
        along += 2 * dash;
    }
    let mut down = 0;
    while down < at.size.h {
        let tall = dash.min(at.size.h - down);
        stretches.push(Rectangle::new((at.loc.x, at.loc.y + down).into(), (at.size.w, tall).into()));
        down += 2 * dash;
    }
    for stretch in stretches {
        canvas.clipped_to(stretch);
        canvas.border(at, corner, width, colour);
    }
    canvas.unclipped();
}

/// One task's square, at `at`.
fn a_square(canvas: &mut Canvas, fonts: &mut FontSystem, at: Area, stands: HowItStands, measure: Measure) {
    let (ground, ring, width, kind, mark) = the_squares_look(stands);
    let corner = fx(measure, m::CORNER);
    canvas.fill(at, corner, rgb(ground), 1.0);
    match kind {
        Ring::Solid => canvas.border(at, corner, fx(measure, width), rgb(ring)),
        Ring::Dashed => a_dashed_ring(canvas, at, corner, fx(measure, width), rgb(ring), px(measure, m::DASH)),
    }
    if stands == HowItStands::NeedsYourOk {
        let radius = fx(measure, m::DOT) / 2.0;
        let (dx, dy) = m::DOT_AT;
        #[expect(
            clippy::cast_precision_loss,
            reason = "a position in pixels is a few thousand at most, well inside an f32"
        )]
        let (cx, cy) = (
            at.loc.x as f32 + fx(measure, dx) + radius,
            at.loc.y as f32 + fx(measure, dy) + radius,
        );
        canvas.disc(cx, cy, radius, rgb(Role::TextPrimary));
    }
    the_mark(canvas, fonts, at, measure, rgb(ground), rgb(mark));
}

/// One task's card, at `at`. Answers where its button is.
fn a_card(
    canvas: &mut Canvas,
    fonts: &mut FontSystem,
    at: Area,
    card: &ATaskAsideOnTheDisplay,
    measure: Measure,
) -> Option<(TheCardsButton, Area)> {
    let corner = fx(measure, m::CORNER);
    let white = rgb(Role::BgSurface);
    canvas.shadow(
        at,
        corner,
        AShadow {
            falls: fx(measure, m::SHADOW_FALLS),
            blurs: fx(measure, m::SHADOW_BLURS),
            spreads: 0.0,
            darkness: m::SHADOW_DARKNESS,
        },
    );
    canvas.fill(at, corner, white, 1.0);
    // **The state's colour down the left edge**, inside the card's own corner
    // so it is cut by the same curve.
    let (edge, word) = the_cards_colours(card.stands);
    canvas.clipped_to(Rectangle::new(at.loc, (px(measure, m::EDGE), at.size.h).into()));
    canvas.fill(at, corner, rgb(edge), 1.0);
    canvas.unclipped();
    canvas.border(at, corner, fx(measure, 1), rgb(Role::BorderDefault));

    let button = card.button.as_ref().map(|(which, _)| {
        let wide = px(measure, m::BUTTON_WIDE);
        let tall = px(measure, m::BUTTON_TALL);
        let area = Rectangle::new(
            (
                at.loc.x + at.size.w - px(measure, m::BUTTON_FROM_RIGHT) - wide,
                at.loc.y + at.size.h - px(measure, m::BUTTON_FROM_BOTTOM) - tall,
            )
                .into(),
            (wide, tall).into(),
        );
        (*which, area)
    });
    let beside = if button.is_some() { m::LINE_BESIDE_A_BUTTON } else { m::LINE_WIDE };
    let lines: [(&str, u32, u32, Weight, [u8; 3]); 5] = [
        ("alo", m::INSET, m::LINE_WIDE, crate::painted_text::ALOS_OWN_WEIGHT, rgb(word)),
        (&card.state, m::STATE_FROM_LEFT, m::STATE_WIDE, crate::painted_text::ALOS_OWN_WEIGHT, rgb(word)),
        (&card.title, m::INSET, m::LINE_WIDE, crate::painted_text::ALOS_OWN_WEIGHT, rgb(Role::TextPrimary)),
        (&card.how_far, m::INSET, beside, Weight::NORMAL, rgb(Role::TextSecondary)),
        (&card.last, m::INSET, beside, Weight::NORMAL, rgb(Role::TextMuted)),
    ];
    for (which, (text, from_left, wide, weight, ink)) in lines.into_iter().enumerate() {
        if text.is_empty() {
            continue;
        }
        // The mark and the state share the first line.
        let row = which.saturating_sub(1);
        let (Some(top), Some(tall), Some(size)) =
            (m::LINE_TOPS.get(row), m::LINE_TALL.get(row), m::LINE_TYPE.get(row))
        else {
            continue;
        };
        let shaped = one_line(fonts, text, measure, (*size, *tall), weight, px(measure, wide), white, ink);
        canvas.words(at.loc.x + px(measure, from_left), at.loc.y + px(measure, *top), &shaped);
    }

    if let (Some((which, area)), Some((_, said))) = (button, card.button.as_ref()) {
        let navy = rgb(Role::TextPrimary);
        let corner = fx(measure, m::BUTTON_CORNER);
        let (ground, ink) = if which == TheCardsButton::Stop { (navy, white) } else { (white, navy) };
        canvas.fill(area, corner, ground, 1.0);
        canvas.border(area, corner, fx(measure, 1), navy);
        let shaped = one_line(
            fonts,
            said,
            measure,
            (m::BUTTON_TYPE, m::BUTTON_TYPE + m::BUTTON_TYPE / 3),
            crate::painted_text::ALOS_OWN_WEIGHT,
            area.size.w,
            ground,
            ink,
        );
        canvas.words(
            area.loc.x + (area.size.w - shaped.width_of_box) / 2,
            area.loc.y + (area.size.h - shaped.height) / 2,
            &shaped,
        );
    }
    button
}

/// **Draw the tasks into a laid-out panel**: each task's square in its slot,
/// and the card of the one the person opened, beside the rail.
///
/// A task the panel holds with no card said for it — closed between the lane
/// saying the words and this frame — draws an empty slot rather than a square
/// that says something nobody worked out.
pub(crate) fn with_the_tasks(
    mut picture: PanelPicture,
    cards: &[ATaskAsideOnTheDisplay],
    open: Option<TaskNumber>,
    fonts: &mut FontSystem,
    measure: Measure,
    edge: WhichEdge,
) -> PanelPicture {
    if picture.task_squares.is_empty() {
        return picture;
    }
    let card_of = |task: TaskNumber| cards.iter().find(|card| card.task == task);
    let mut squares = Canvas::over(picture.rail);
    for (task, at) in &picture.task_squares {
        if let Some(card) = card_of(*task) {
            a_square(&mut squares, fonts, *at, card.stands, measure);
        }
    }
    picture.washed.push(squares.washed().premultiplied());

    let opened = open.and_then(|task| {
        let (_, square) = picture.task_squares.iter().find(|(its, _)| *its == task)?;
        Some((card_of(task)?, *square))
    });
    if let Some((card, square)) = opened {
        let wide = px(measure, m::CARD_WIDE);
        let tall = px(measure, m::CARD_TALL);
        let gap = px(measure, MARGIN);
        let x = match edge {
            WhichEdge::Right => picture.rail.loc.x - gap - wide,
            WhichEdge::Left => picture.rail.loc.x + picture.rail.size.w + gap,
        };
        let lowest = (picture.size.1 - tall).max(0);
        let area = Rectangle::new((x.max(0), square.loc.y.min(lowest)).into(), (wide, tall).into());
        let reach = px(measure, m::SHADOW_BLURS) + px(measure, m::SHADOW_FALLS);
        let around = Rectangle::new(
            (area.loc.x - reach, area.loc.y - reach).into(),
            (wide + 2 * reach, tall + 2 * reach).into(),
        );
        let mut canvas = Canvas::over(around);
        let button = a_card(&mut canvas, fonts, area, card, measure);
        picture.washed.push(canvas.washed().premultiplied());
        picture.open_card = Some(TheOpenCard {
            task: card.task,
            area,
            button,
        });
    }
    picture
}

#[cfg(test)]
#[path = "task_card_raster_tests.rs"]
mod tests;
