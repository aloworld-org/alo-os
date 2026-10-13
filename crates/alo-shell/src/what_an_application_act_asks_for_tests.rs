//! What each act asks for, over the windows there are.
//!
//! These are the questions that have nothing to do with a compositor: which
//! window of three, every window or one, and start or refuse. The doing is the
//! nested fixture's road.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err or None is the failure being reported"
)]

use alo_acting::Acting;
use alo_applications::Application;
use alo_dock::{AppId, HowItSits, Patch, Spot, Window, WindowId, Windows};

use crate::what_an_application_act_asks_for::{WhatToDo, what_an_application_act_asks_for};

/// The mail client, as both crates name it.
fn mail() -> AppId {
    AppId::named("org.gnome.Geary").expect("a named application")
}

/// An act about the mail client.
fn about_mail(spelt: &str) -> Acting {
    let application = Application::identified("org.gnome.Geary").expect("an identifier");
    Acting::off_the_wire(spelt, application).expect("one of the three acts")
}

/// A window of `app`, numbered `number`.
fn window(number: u64, app: Option<&str>, sits: HowItSits) -> Window {
    let patch = Patch::of(Spot::at(0, 0), 1, 1).expect("one by one is a patch");
    Window::of(
        WindowId::numbered(number),
        app.map(|app| AppId::named(app).expect("a named application")),
        "",
        patch,
        sits,
    )
}

/// Windows opened in this order, oldest first.
fn opened(these: &[(u64, Option<&str>)]) -> Windows {
    let mut windows = Windows::none();
    for (number, app) in these {
        windows.opened(window(*number, *app, HowItSits::OnTheCanvas));
    }
    windows
}

/// **`open` with nothing open starts it**, which is what a press on a Dock
/// icon does with nothing open.
#[test]
fn open_with_nothing_open_starts_it() {
    let windows = opened(&[(1, Some("org.kde.dolphin"))]);
    assert_eq!(
        what_an_application_act_asks_for(&windows, &mail(), &about_mail("open")),
        WhatToDo::StartIt
    );
}

/// **`open` on something already running brings it up and starts nothing.** An
/// `open` that started a second copy would be a verb doing something nobody
/// approved a sentence for.
#[test]
fn open_on_something_running_brings_it_up_and_starts_nothing() {
    let windows = opened(&[(1, Some("org.gnome.Geary"))]);
    assert_eq!(
        what_an_application_act_asks_for(&windows, &mail(), &about_mail("open")),
        WhatToDo::BringThisForward(WindowId::numbered(1))
    );
}

/// **`focus` never starts anything** — that is `open`'s, and a `focus` that
/// started an application would carry out a different sentence from the one a
/// person read.
#[test]
fn focus_never_starts_anything() {
    let windows = opened(&[(1, Some("org.kde.dolphin"))]);
    assert_eq!(
        what_an_application_act_asks_for(&windows, &mail(), &about_mail("focus")),
        WhatToDo::ThereIsNoWindow
    );
}

/// **With several windows it is the most recently used one, never a list.** A
/// person is offered a list because they are standing there to choose; an agent
/// is not.
#[test]
fn several_windows_means_the_one_the_person_last_looked_at() {
    let mut windows = opened(&[
        (1, Some("org.gnome.Geary")),
        (2, Some("org.gnome.Geary")),
        (3, Some("org.gnome.Geary")),
    ]);
    // The person goes back to the first one they opened.
    assert!(windows.used(WindowId::numbered(1)));

    for spelt in ["open", "focus"] {
        assert_eq!(
            what_an_application_act_asks_for(&windows, &mail(), &about_mail(spelt)),
            WhatToDo::BringThisForward(WindowId::numbered(1)),
            "{spelt} did not follow what the person last used"
        );
    }
}

/// **`close` asks every window of it, oldest first**, because *close Geary*
/// means Geary rather than one of its windows.
#[test]
fn close_asks_every_window_oldest_first() {
    let mut windows = opened(&[
        (1, Some("org.gnome.Geary")),
        (2, Some("org.kde.dolphin")),
        (3, Some("org.gnome.Geary")),
        (4, Some("org.gnome.Geary")),
    ]);
    // What the person last used must not change the order they are asked in.
    assert!(windows.used(WindowId::numbered(4)));

    assert_eq!(
        what_an_application_act_asks_for(&windows, &mail(), &about_mail("close")),
        WhatToDo::AskTheseToClose(vec![
            WindowId::numbered(1),
            WindowId::numbered(3),
            WindowId::numbered(4),
        ]),
        "close did not ask every window of it in the order they opened"
    );
}

/// **`close` with no window of it refuses**, and does not reach for somebody
/// else's.
#[test]
fn close_with_no_window_of_it_refuses() {
    let windows = opened(&[(1, Some("org.kde.dolphin")), (2, None)]);
    assert_eq!(
        what_an_application_act_asks_for(&windows, &mail(), &about_mail("close")),
        WhatToDo::ThereIsNoWindow
    );
}

/// **A window with no application identity is nobody's**, and no act reaches
/// it. `alo_dock::Window::app` is an `Option` by the owner's direction —
/// *do not substitute a fabricated app identity* — and two unnamed windows must
/// not collapse into one application an act could name.
#[test]
fn a_window_with_no_identity_is_reached_by_no_act() {
    let windows = opened(&[(1, None), (2, None)]);
    for spelt in ["focus", "close"] {
        assert_eq!(
            what_an_application_act_asks_for(&windows, &mail(), &about_mail(spelt)),
            WhatToDo::ThereIsNoWindow,
            "{spelt} reached a window belonging to nobody"
        );
    }
    assert_eq!(
        what_an_application_act_asks_for(&windows, &mail(), &about_mail("open")),
        WhatToDo::StartIt
    );
}

/// **No act reaches another application's windows**, which is the thing a
/// grant is about: a turn granted the mail client must not close the browser.
#[test]
fn no_act_reaches_another_applications_windows() {
    let windows = opened(&[
        (1, Some("org.kde.dolphin")),
        (2, Some("org.mozilla.firefox")),
    ]);
    for spelt in ["focus", "close"] {
        assert_eq!(
            what_an_application_act_asks_for(&windows, &mail(), &about_mail(spelt)),
            WhatToDo::ThereIsNoWindow,
            "{spelt} reached somebody else's window"
        );
    }
}
