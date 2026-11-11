//! **A running task waits in the panel beside the windows** — the owner's approved design
//! `07 / A running task in the put-aside panel` (`23:2`, *"Approve as drawn"*, 2026-11-11).
//!
//! Windows and tasks together, newest put aside first; a task remembers the Place it was
//! asked on; one task is one card; and taking a task out touches no window.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_canvas::{Place, Zoom};
use alo_dock::on_the_canvas::{Patch, Spot};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_put_aside::{Aside, NotPutAside, Panel, Privacy, TaskNumber};

/// Not `Place::FIRST`, so a panel that kept no Place and answered the default fails.
fn a_place() -> Place {
    Place::numbered(7).expect("7 is not zero, so it is a Place")
}

fn window(id: u64) -> Window {
    Window::of(
        WindowId::numbered(id),
        Some(AppId::named("Dolphin").expect("a fixture names its application")),
        "Home",
        Patch::of(Spot::at(0, 0), 800, 600).expect("a fixture gives its patch an extent"),
        HowItSits::OnTheCanvas,
    )
}

/// What the panel holds, in order, as short names a failure can print.
fn in_order(panel: &Panel) -> Vec<String> {
    panel
        .everything()
        .iter()
        .map(|one| match one {
            Aside::AWindow(preview) => format!("window {}", preview.window().number()),
            Aside::ATask(task) => format!("task {}", task.task().at()),
        })
        .collect()
}

/// **Windows and tasks together, newest put aside first** — not windows first, not tasks
/// first.
#[test]
fn windows_and_tasks_are_held_together_newest_first() {
    let mut panel = Panel::new();
    panel
        .put_aside(&window(1), Zoom::LIFE_SIZE, a_place(), Privacy::Ordinary)
        .unwrap();
    panel
        .put_a_task_aside(TaskNumber::numbered(0), a_place())
        .unwrap();
    panel
        .put_aside(&window(2), Zoom::LIFE_SIZE, a_place(), Privacy::Ordinary)
        .unwrap();

    assert_eq!(in_order(&panel), ["window 2", "task 0", "window 1"]);
    assert_eq!(panel.holding_everything(), 3);
    assert_eq!(panel.holding(), 2, "the windows alone are still two");
}

/// **A task goes back to the Place it was asked on**, whatever Place the person is on now.
#[test]
fn a_task_remembers_the_place_it_was_asked_on() {
    let mut panel = Panel::new();
    panel
        .put_a_task_aside(TaskNumber::numbered(3), a_place())
        .unwrap();

    let back = panel.bring_a_task_back(TaskNumber::numbered(3)).unwrap();

    assert_eq!(back.place(), a_place());
    assert!(!panel.holds_the_task(TaskNumber::numbered(3)));
    assert_eq!(panel.holding_everything(), 0);
}

/// **One task is one card**: putting it aside twice is refused, not a second card.
#[test]
fn a_task_cannot_be_put_aside_twice() {
    let mut panel = Panel::new();
    panel
        .put_a_task_aside(TaskNumber::numbered(0), a_place())
        .unwrap();

    assert_eq!(
        panel.put_a_task_aside(TaskNumber::numbered(0), a_place()),
        Err(NotPutAside::ItIsAlreadyThere)
    );
    assert_eq!(panel.holding_everything(), 1);
}

/// **Bringing back a task the panel does not hold is refused**, and is a different answer
/// from an empty panel's.
#[test]
fn a_task_that_is_not_there_cannot_be_brought_back() {
    let mut panel = Panel::new();
    panel
        .put_aside(&window(1), Zoom::LIFE_SIZE, a_place(), Privacy::Ordinary)
        .unwrap();

    assert_eq!(
        panel.bring_a_task_back(TaskNumber::numbered(9)),
        Err(NotPutAside::ItIsNotThere)
    );
    assert_eq!(
        panel.holding_everything(),
        1,
        "a refused bring-back took something out"
    );
}

/// **Taking a task out leaves every window where it was**, and bringing a window back
/// leaves the task.
#[test]
fn a_task_and_a_window_come_out_without_disturbing_each_other() {
    let mut panel = Panel::new();
    panel
        .put_aside(&window(1), Zoom::LIFE_SIZE, a_place(), Privacy::Ordinary)
        .unwrap();
    panel
        .put_a_task_aside(TaskNumber::numbered(0), a_place())
        .unwrap();
    panel
        .put_aside(&window(2), Zoom::LIFE_SIZE, a_place(), Privacy::Ordinary)
        .unwrap();

    panel.bring_a_task_back(TaskNumber::numbered(0)).unwrap();
    assert_eq!(in_order(&panel), ["window 2", "window 1"]);

    panel
        .put_a_task_aside(TaskNumber::numbered(0), a_place())
        .unwrap();
    panel.bring_back(WindowId::numbered(2)).unwrap();
    assert_eq!(in_order(&panel), ["task 0", "window 1"]);
}

/// **A panel holding only a task is not empty.**
#[test]
fn a_panel_holding_only_a_task_is_not_empty() {
    let mut panel = Panel::new();
    panel
        .put_a_task_aside(TaskNumber::numbered(0), a_place())
        .unwrap();

    assert_ne!(
        panel.showing(a_place()),
        Panel::new().showing(a_place()),
        "a panel holding a running task showed as empty, so the task had nowhere to be found"
    );
}
