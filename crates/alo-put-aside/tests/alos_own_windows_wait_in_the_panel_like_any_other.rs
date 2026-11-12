//! **alo's own windows wait in the panel like any other window** — the owner's approved
//! design `10 / Minimised windows, Mac-style` (`27:2`, 2026-11-12): the terminal, Settings
//! and the record minimise into the same right-hand panel as an application's window.
//!
//! Newest put aside first among windows and tasks alike; each remembers the Place it was on;
//! one of each is one square; and taking one out touches nothing else in the panel.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_canvas::{Place, Zoom};
use alo_dock::on_the_canvas::{Patch, Spot};
use alo_dock::window::{AppId, HowItSits, Window, WindowId};
use alo_put_aside::{AlosOwnWindow, Aside, NotPutAside, Panel, Privacy, TaskNumber};

/// Not `Place::FIRST`, so a panel that kept no Place and answered the default fails.
fn a_place() -> Place {
    Place::numbered(5).expect("5 is not zero, so it is a Place")
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

fn in_order(panel: &Panel) -> Vec<String> {
    panel
        .everything()
        .iter()
        .map(|one| match one {
            Aside::AWindow(preview) => format!("window {}", preview.window().number()),
            Aside::ATask(task) => format!("task {}", task.task().at()),
            Aside::AnAloWindow(aside) => format!("{:?}", aside.which()),
        })
        .collect()
}

/// **In the one order, newest first**, beside an application's window and a task.
#[test]
fn alos_windows_are_held_with_the_others_newest_first() {
    let mut panel = Panel::new();
    panel
        .put_aside(&window(1), Zoom::LIFE_SIZE, a_place(), Privacy::Ordinary)
        .unwrap();
    panel
        .put_alos_window_aside(AlosOwnWindow::Settings, a_place())
        .unwrap();
    panel
        .put_a_task_aside(TaskNumber::numbered(0), a_place())
        .unwrap();
    panel
        .put_alos_window_aside(AlosOwnWindow::TheTerminal, a_place())
        .unwrap();

    assert_eq!(
        in_order(&panel),
        ["TheTerminal", "task 0", "Settings", "window 1"]
    );
    assert_eq!(panel.holding_everything(), 4);
    assert_eq!(panel.holding(), 1, "only one is an application's window");
}

/// **It goes back to the Place it was on**, whatever Place the person is on now.
#[test]
fn alos_window_remembers_its_place() {
    let mut panel = Panel::new();
    panel
        .put_alos_window_aside(AlosOwnWindow::TheRecord, a_place())
        .unwrap();

    let back = panel
        .bring_alos_window_back(AlosOwnWindow::TheRecord)
        .unwrap();

    assert_eq!(back.place(), a_place());
    assert_eq!(back.which(), AlosOwnWindow::TheRecord);
    assert!(!panel.holds_alos_window(AlosOwnWindow::TheRecord));
    assert_eq!(panel.holding_everything(), 0);
}

/// **One of each is one square**: a second put aside is refused, not a second square.
#[test]
fn the_same_window_twice_is_refused() {
    let mut panel = Panel::new();
    panel
        .put_alos_window_aside(AlosOwnWindow::Settings, a_place())
        .unwrap();
    assert_eq!(
        panel.put_alos_window_aside(AlosOwnWindow::Settings, a_place()),
        Err(NotPutAside::ItIsAlreadyThere)
    );
    assert_eq!(panel.holding_everything(), 1);
}

/// **Bringing back one that is not there is refused**, and leaves the panel as it was.
#[test]
fn bringing_back_one_that_is_not_there_is_refused() {
    let mut panel = Panel::new();
    panel
        .put_alos_window_aside(AlosOwnWindow::Settings, a_place())
        .unwrap();
    assert_eq!(
        panel.bring_alos_window_back(AlosOwnWindow::TheTerminal),
        Err(NotPutAside::ItIsNotThere)
    );
    assert!(panel.holds_alos_window(AlosOwnWindow::Settings));
}

/// **Taking one of alo's windows out touches nothing else**: the application's window and
/// the task stay, in their order.
#[test]
fn taking_one_out_leaves_everything_else() {
    let mut panel = Panel::new();
    panel
        .put_aside(&window(2), Zoom::LIFE_SIZE, a_place(), Privacy::Ordinary)
        .unwrap();
    panel
        .put_alos_window_aside(AlosOwnWindow::TheTerminal, a_place())
        .unwrap();
    panel
        .put_a_task_aside(TaskNumber::numbered(4), a_place())
        .unwrap();

    panel
        .bring_alos_window_back(AlosOwnWindow::TheTerminal)
        .unwrap();

    assert_eq!(in_order(&panel), ["task 4", "window 2"]);
    // And a task's own door does not reach alo's windows, nor the other way round.
    assert_eq!(
        panel.bring_a_task_back(TaskNumber::numbered(9)),
        Err(NotPutAside::ItIsNotThere)
    );
    assert!(panel.holds_the_task(TaskNumber::numbered(4)));
}
