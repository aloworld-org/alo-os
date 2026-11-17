//! **When alo's task window closes, the keyboard goes to the window the person
//! sees in front** — and alo still never takes the keyboard while the person
//! is working elsewhere. On real client connections.
//!
//! The owner's decision of 2026-11-17, choosing between two promises that met
//! on a walked machine (run 43990841766): a task opened Papers, the person
//! closed the finished task with Alt+F4, Papers was the window they saw, and
//! the next Alt+F4 went nowhere — typing stayed where it had been before the
//! task. `docs/design/where-a-new-window-opens.md`'s rule 8 (*alo must not
//! steal focus*) still stands for a window alo opens; what changes is the
//! closing: *like every desktop, the window you can see in front gets your
//! typing* (the owner, 2026-10-30), now for an application too.

#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use alo_shell::closing_a_task_window::WhatClosingCameTo;
use alo_shell::the_tasks_a_person_opened::WhichTask;
use wayland_client::Proxy as _;

use super::support::{Application, Fixture};

/// A window of `app_id`, through the real XDG handshake, kept alive.
fn mapped(f: &Fixture, app_id: &str) -> Application {
    let mut app = Application::new(f);
    app.toplevel.set_app_id(app_id.to_owned());
    app.configure();
    app.attach();
    app.sync();
    app
}

/// A task window the person asked for, placed and holding the keyboard.
fn a_task_holding_the_keyboard(f: &Fixture) -> WhichTask {
    f.backend(|s| {
        let which = s.tasks_mut().and_now_this_one(alo_tasks::ATask::asked_for(
            "Open Papers.",
            alo_models::InferenceSource::ThisMachine,
        ));
        let view = smithay::utils::Rectangle::new((0, 0).into(), (1280, 800).into());
        s.tasks_mut().put_whatever_has_nowhere_to_be(
            view,
            alo_strings::Direction::LeftToRight,
            &[],
            100,
        );
        s.this_task_took_the_keyboard(which);
        which
    })
}

/// **Closing the task window gives the window in front the keyboard.** An
/// application window with no keyboard, a task window holding it, the task
/// finished and closed: the application now has the keyboard.
#[test]
fn closing_the_task_window_gives_the_window_in_front_the_keyboard() {
    let f = Fixture::keyboard();
    let mut papers = mapped(&f, "org.gnome.Papers");
    assert!(
        papers.events.keyboard.surfaces.is_empty(),
        "the control: the window had the keyboard before the task closed"
    );
    let which = a_task_holding_the_keyboard(&f);
    f.backend(move |s| {
        s.tasks_mut()
            .the_mut(which)
            .expect("the task is open")
            .alo_says(alo_tasks::conversation::Turn::from_alo("Papers is open."));
    });

    let closed = f.backend(move |s| s.the_person_closes_a_task(which));
    papers.sync();

    assert_eq!(closed, WhatClosingCameTo::Closed);
    assert!(
        !f.backend(|s| s.a_task_is_taking_the_keys()),
        "a closed task still takes the keys"
    );
    assert_eq!(
        papers.events.keyboard.surfaces.last(),
        Some(&papers.surface.id().protocol_id()),
        "the window the person sees in front did not get the keyboard"
    );
}

/// **But alo opening a window while the person is in the task does not take
/// the keyboard**: the window comes, the task keeps the keys, and the new
/// window hears nothing — rule 8, unchanged.
#[test]
fn a_window_alo_opens_while_the_person_is_in_the_task_does_not_take_the_keyboard() {
    let f = Fixture::keyboard();
    let _which = a_task_holding_the_keyboard(&f);
    let mut papers = mapped(&f, "org.gnome.Papers");

    f.backend(|s| s.what_the_person_opened_comes_in_front());
    papers.sync();

    assert!(
        f.backend(|s| s.a_task_is_taking_the_keys()),
        "the task lost the keyboard to a window alo opened"
    );
    assert!(
        papers.events.keyboard.surfaces.is_empty(),
        "a window alo opened took the keyboard"
    );
}
