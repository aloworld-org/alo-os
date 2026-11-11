//! A running task goes into the panel and keeps running; its square opens its
//! card; Stop from the card stops the work and leaves the task in the panel;
//! Open brings it back exactly where it was. And the refusals: a press on a
//! task no longer in the panel does nothing, a task cannot be put aside twice,
//! and a task with no window on the screen cannot be put aside at all.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_models::InferenceSource;
use alo_put_aside::{Panel, TaskNumber};
use alo_tasks::{ATask, Act, HowItStands, TheCardsButton};
use smithay::input::keyboard::XkbConfig;

use super::{OnATask, WhatThePressDid};
use crate::Server;
use crate::the_tasks_a_person_opened::WhichTask;

/// A session with one task alo is working on, its window placed.
fn a_placed_working_task() -> (tempfile::TempDir, Server, WhichTask) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "a-task-in-the-panel",
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    let mut task = ATask::asked_for("Summarise the Q3 report", InferenceSource::ThisMachine);
    task.and_then(Act::doing("Reading report.pdf"));
    let which = server.tasks_mut().and_now_this_one(task);
    let view = smithay::utils::Rectangle::new((0, 0).into(), (1280, 800).into());
    server.tasks_mut().put_whatever_has_nowhere_to_be(
        view,
        alo_strings::Direction::LeftToRight,
        &[],
        100,
    );
    (runtime, server, which)
}

fn number(which: WhichTask) -> TaskNumber {
    TaskNumber::numbered(which.at())
}

/// **Put aside, it keeps running**, leaves the screen, and is in the panel.
#[test]
fn a_task_put_aside_keeps_running_and_is_in_the_panel() {
    let (_runtime, mut server, which) = a_placed_working_task();
    let mut panel = Panel::new();

    assert!(server.put_a_task_aside(&mut panel, which));

    assert!(panel.holds_the_task(number(which)));
    assert!(server.tasks().is_put_aside(which));
    assert!(server.tasks().is_open(which), "putting a task aside closed it");
    assert!(
        server.tasks().each_open_window().is_empty(),
        "a task put aside is still drawn on the screen"
    );
    assert_eq!(
        server.tasks().the(which).expect("the task").how_it_stands(),
        HowItStands::Working,
        "putting a task aside stopped its work"
    );
    assert!(server.take_the_stops().is_empty(), "putting a task aside sent a Stop");
}

/// **A task cannot be put aside twice**, and one with no window on the screen
/// cannot be put aside at all.
#[test]
fn a_task_is_put_aside_once_and_only_from_the_screen() {
    let (_runtime, mut server, which) = a_placed_working_task();
    let mut panel = Panel::new();
    assert!(server.put_a_task_aside(&mut panel, which));

    assert!(!server.put_a_task_aside(&mut panel, which), "put aside twice");
    assert_eq!(panel.holding_everything(), 1);
    assert!(
        !server.put_a_task_aside(&mut panel, WhichTask::numbered(9)),
        "a task that does not exist was put aside"
    );
}

/// **Pressing the square opens its card, and pressing it again closes it.**
#[test]
fn the_square_opens_and_closes_its_card() {
    let (_runtime, mut server, which) = a_placed_working_task();
    let mut panel = Panel::new();
    assert!(server.put_a_task_aside(&mut panel, which));

    assert_eq!(
        server.a_press_on_a_task(&mut panel, OnATask::ItsSquare(number(which))),
        WhatThePressDid::OpenedItsCard
    );
    assert_eq!(server.the_task_card_open, Some(number(which)));
    assert_eq!(
        server.a_press_on_a_task(&mut panel, OnATask::ItsSquare(number(which))),
        WhatThePressDid::ClosedItsCard
    );
    assert_eq!(server.the_task_card_open, None);
}

/// **Stop from the card stops the work, once, and leaves the task in the
/// panel** saying *Stopping…* — it does not close it, and it does not bring
/// the window back.
#[test]
fn stop_on_the_card_stops_the_work_and_leaves_the_task_aside() {
    let (_runtime, mut server, which) = a_placed_working_task();
    let mut panel = Panel::new();
    assert!(server.put_a_task_aside(&mut panel, which));

    assert_eq!(
        server.a_press_on_a_task(
            &mut panel,
            OnATask::ItsButton(number(which), TheCardsButton::Stop)
        ),
        WhatThePressDid::Stopped
    );

    assert_eq!(server.take_the_stops().len(), 1, "one press, one Stop");
    assert!(panel.holds_the_task(number(which)), "Stop took the task out of the panel");
    assert!(server.tasks().is_put_aside(which));
    assert_eq!(
        server.tasks().the(which).expect("the task").how_it_stands(),
        HowItStands::Stopping
    );
    // A second press while the first is honoured sends nothing more.
    let _ = server.a_press_on_a_task(
        &mut panel,
        OnATask::ItsButton(number(which), TheCardsButton::Stop),
    );
    assert!(server.take_the_stops().is_empty(), "a second Stop was sent for one");
}

/// **Open brings the window back exactly where it was**, out of the panel.
#[test]
fn open_brings_the_window_back_where_it_was() {
    let (_runtime, mut server, which) = a_placed_working_task();
    let before = server.tasks().each_open_window();
    let mut panel = Panel::new();
    assert!(server.put_a_task_aside(&mut panel, which));
    server.the_task_card_open = Some(number(which));

    assert_eq!(
        server.a_press_on_a_task(&mut panel, OnATask::ItsCard(number(which))),
        WhatThePressDid::BroughtItBack
    );

    assert!(!panel.holds_the_task(number(which)));
    assert_eq!(server.tasks().each_open_window(), before, "it came back somewhere else");
    assert_eq!(server.the_task_card_open, None, "the card stayed open on a task not in the panel");
}

/// **A press on a task no longer in the panel does nothing** — not Stop, not
/// bring back.
#[test]
fn a_press_on_a_task_not_in_the_panel_does_nothing() {
    let (_runtime, mut server, which) = a_placed_working_task();
    let mut panel = Panel::new();

    for on in [
        OnATask::ItsSquare(number(which)),
        OnATask::ItsCard(number(which)),
        OnATask::ItsButton(number(which), TheCardsButton::Stop),
        OnATask::ItsButton(number(which), TheCardsButton::Open),
    ] {
        assert_eq!(server.a_press_on_a_task(&mut panel, on), WhatThePressDid::Nothing);
    }
    assert!(server.take_the_stops().is_empty());
    assert_eq!(server.the_task_card_open, None);
}
