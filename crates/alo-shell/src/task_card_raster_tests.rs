//! The square and the card where the approved design `23:2` puts them, at the
//! display's scale — and nothing drawn for a task with no card said, or for a
//! panel holding no task.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_models::InferenceSource;
use alo_put_aside::the_region_the_panel_claims::WhichEdge;
use alo_put_aside::{Panel, TaskNumber};
use alo_strings::{Direction, Strings, Vocabulary};
use alo_tasks::{ATask, Act, TheCardsButton};

use super::with_the_tasks;
use crate::WindowControlLabels;
use crate::a_task_in_the_panel::{ATaskAsideOnTheDisplay, said};
use crate::desktop_testing::{an_appearance, noon_look};
use crate::display_scale::DisplayScale;
use crate::panel_raster::{PanelPicture, WhetherRevealed, picture};

/// The design's own frame, used as *a* display rather than *the* display.
const AS_DRAWN: (i32, i32) = (1440, 960);

fn a_card(working: bool) -> ATaskAsideOnTheDisplay {
    let mut vocabulary = Vocabulary::default();
    alo_tasks::declare_into(&mut vocabulary).unwrap();
    let mut task = ATask::asked_for("Summarise the Q3 report", InferenceSource::ThisMachine);
    task.and_then(Act::doing("Reading report.pdf"));
    if !working {
        task.stop();
    }
    said(TaskNumber::numbered(0), &task, &Strings::of(vocabulary))
}

fn a_panel_holding_one_task() -> Panel {
    let mut panel = Panel::new();
    panel
        .put_a_task_aside(TaskNumber::numbered(0), alo_canvas::Place::FIRST)
        .unwrap();
    panel
}

fn drawn(
    panel: &Panel,
    cards: &[ATaskAsideOnTheDisplay],
    open: Option<TaskNumber>,
    scale: DisplayScale,
) -> PanelPicture {
    let look = noon_look(&an_appearance(), Direction::LeftToRight);
    let laid = picture(
        panel,
        look,
        AS_DRAWN,
        WhichEdge::Right,
        WhetherRevealed::Revealed,
        scale,
    )
    .unwrap();
    let mut labels = WindowControlLabels::new().unwrap();
    with_the_tasks(
        laid,
        cards,
        open,
        &mut labels.fonts,
        look.measure().on(scale),
        WhichEdge::Right,
    )
}

/// **A task takes a slot in the rail, as a window would**, and is drawn there
/// rather than painted the accent square a window gets.
#[test]
fn a_task_takes_a_slot_and_is_drawn_in_it() {
    let panel = a_panel_holding_one_task();
    let picture = drawn(&panel, &[a_card(true)], None, DisplayScale::ONE);

    assert_eq!(picture.task_squares.len(), 1);
    assert!(picture.slots.is_empty(), "a task was given a window's slot");
    let (_, square) = picture.task_squares.first().unwrap();
    assert_eq!((square.size.w, square.size.h), (48, 48));
    assert_eq!(picture.washed.len(), 1, "the square was not drawn");
    assert!(picture.open_card.is_none());
}

/// **Pressing the square opens a 216 × 106 card beside the rail, inward**,
/// top-aligned with its square, with Stop 64 × 30 at its bottom right, 12 from
/// the right and 10 from the bottom.
#[test]
fn the_open_card_sits_beside_the_rail_where_the_design_puts_it() {
    let panel = a_panel_holding_one_task();
    let picture = drawn(
        &panel,
        &[a_card(true)],
        Some(TaskNumber::numbered(0)),
        DisplayScale::ONE,
    );

    let card = picture.open_card.expect("the card was opened");
    let (_, square) = picture.task_squares.first().unwrap();
    assert_eq!((card.area.size.w, card.area.size.h), (216, 106));
    assert_eq!(card.area.loc.y, square.loc.y);
    assert_eq!(
        card.area.loc.x + card.area.size.w + 8,
        picture.rail.loc.x,
        "the card is not a margin inward of the rail"
    );
    let (which, button) = card.button.expect("a working task offers Stop");
    assert_eq!(which, TheCardsButton::Stop);
    assert_eq!((button.size.w, button.size.h), (64, 30));
    assert_eq!(
        button.loc.x + button.size.w,
        card.area.loc.x + card.area.size.w - 12
    );
    assert_eq!(
        button.loc.y + button.size.h,
        card.area.loc.y + card.area.size.h - 10
    );
    assert_eq!(picture.washed.len(), 2, "the card was not drawn");
}

/// **No button while a Stop is on its way** — nothing to press twice.
#[test]
fn a_stopping_tasks_card_has_no_button() {
    let panel = a_panel_holding_one_task();
    let picture = drawn(
        &panel,
        &[a_card(false)],
        Some(TaskNumber::numbered(0)),
        DisplayScale::ONE,
    );

    assert_eq!(picture.open_card.expect("the card was opened").button, None);
}

/// **Twice the pixels draws the same card sharper, not half as big.**
#[test]
fn a_dense_display_draws_the_card_at_its_scale() {
    let panel = a_panel_holding_one_task();
    let picture = drawn(
        &panel,
        &[a_card(true)],
        Some(TaskNumber::numbered(0)),
        DisplayScale::of_per_cent(200),
    );

    let card = picture.open_card.expect("the card was opened");
    assert_eq!((card.area.size.w, card.area.size.h), (432, 212));
}

/// **A task with no card said draws nothing**, and a card asked open for it
/// does not open: a square that says something nobody worked out is worse
/// than an empty slot.
#[test]
fn a_task_with_no_card_said_draws_nothing() {
    let panel = a_panel_holding_one_task();
    let picture = drawn(
        &panel,
        &[],
        Some(TaskNumber::numbered(0)),
        DisplayScale::ONE,
    );

    assert!(picture.open_card.is_none());
    let rail = picture.washed.first().expect("the squares' raster");
    assert!(
        rail.pixels.chunks(4).all(|pixel| pixel.get(3) == Some(&0)),
        "something was drawn for a task nobody said anything about"
    );
}

/// **A panel holding no task draws no task raster at all.**
#[test]
fn a_panel_with_no_task_draws_no_task() {
    let picture = drawn(&Panel::new(), &[a_card(true)], None, DisplayScale::ONE);

    assert!(picture.washed.is_empty());
    assert!(picture.task_squares.is_empty());
}
