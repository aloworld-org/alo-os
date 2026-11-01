//! **A window the person just opened comes in front and takes the keyboard**,
//! on real client connections.
//!
//! Found on a booted machine on 2026-11-01: Dolphin started from the alo Bar
//! and nothing appeared until Super+Tab, because a new window went to the
//! bottom of the stack, behind alo's terminal, without the keyboard. The rule
//! is `docs/design/where-a-new-window-opens.md`'s rule 8: *a person-initiated
//! opening activates the new window; background applications and alo must not
//! steal focus.* Each test below has its refusal beside it; the agent's own
//! start, which must not move the front, is in the crate's unit tests because
//! its road is not public.

#![expect(
    clippy::panic,
    reason = "in a test, a panic names the fixture that did not hold"
)]

use super::support::{Application, Fixture};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use wayland_client::Proxy as _;

/// An application by its identifier.
fn app(named: &str) -> alo_dock::AppId {
    match alo_dock::AppId::named(named) {
        Ok(app) => app,
        Err(why) => panic!("{named} is not an application name: {why:?}"),
    }
}

/// The windows on the screen, front first, as the compositor holds them.
fn front_to_back(f: &Fixture) -> Vec<WlSurface> {
    f.backend(|s| s.mapped_surfaces().cloned().collect())
}

/// The window in front.
fn the_front_one(f: &Fixture) -> Option<WlSurface> {
    front_to_back(f).first().cloned()
}

/// A window of `app_id`, mapped on a connection of its own, and the
/// compositor's own handle on it.
fn mapped(f: &Fixture, app_id: &str) -> (Application, WlSurface) {
    let before = front_to_back(f);
    let mut app = Application::new(f);
    app.toplevel.set_app_id(app_id.to_owned());
    app.configure();
    app.attach();
    app.sync();
    let theirs = the_new_one(f, &before);
    (app, theirs)
}

/// The one window that is on the screen now and was not in `before`.
fn the_new_one(f: &Fixture, before: &[WlSurface]) -> WlSurface {
    let new: Vec<WlSurface> = front_to_back(f)
        .into_iter()
        .filter(|surface| !before.contains(surface))
        .collect();
    match new.as_slice() {
        [one] => one.clone(),
        other => panic!("expected one new window, found {}", other.len()),
    }
}

/// A desktop with one application holding the keyboard, and the person
/// having asked for `started` by their own road, which `answer` either
/// starts or refuses.
fn a_desktop_where_the_person_asked_for(
    started: Option<&str>,
    answer: bool,
) -> (Fixture, Application) {
    let f = Fixture::keyboard();
    let (editor, root) = mapped(&f, "org.example.Editor");
    assert!(
        f.focus_surface(root).is_ok(),
        "the control: the editor has the keyboard"
    );
    if let Some(started) = started {
        let started = app(started);
        let not_started = f.backend(move |s| {
            s.the_person_asked_to_start(started);
            s.start_what_was_asked_for(&|_, _| answer)
        });
        assert_eq!(
            not_started.is_empty(),
            answer,
            "the control: the start was answered"
        );
    }
    (f, editor)
}

/// **The application the person started comes in front with the keyboard.**
#[test]
fn the_application_the_person_started_comes_in_front_with_the_keyboard() {
    let (f, mut editor) = a_desktop_where_the_person_asked_for(Some("org.kde.dolphin"), true);
    let (mut files, theirs) = mapped(&f, "org.kde.dolphin");
    assert_ne!(
        the_front_one(&f),
        Some(theirs.clone()),
        "the control: a new window opened in front before anything brought it there"
    );
    assert!(
        !f.backend(|s| s.the_applications_are_in_front()),
        "the control: the applications were already in front of alo's own windows"
    );

    f.backend(|s| s.what_the_person_opened_comes_in_front());
    files.sync();
    editor.sync();

    assert_eq!(
        the_front_one(&f),
        Some(theirs),
        "the window the person opened is not in front"
    );
    assert_eq!(
        files.events.keyboard.surfaces.last(),
        Some(&files.surface.id().protocol_id()),
        "the window the person opened did not get the keyboard"
    );
    assert_eq!(
        editor.events.keyboard.leaves, 1,
        "the editor kept the keyboard"
    );
    assert!(
        f.backend(|s| s.the_applications_are_in_front()),
        "the applications stayed behind alo's terminal and task windows"
    );
}

/// **Only once**: a second window of the same application does not take the
/// person away again.
#[test]
fn a_start_brings_one_window_in_front_and_is_then_spent() {
    let (f, _editor) = a_desktop_where_the_person_asked_for(Some("org.kde.dolphin"), true);
    let (_files, theirs) = mapped(&f, "org.kde.dolphin");
    f.backend(|s| s.what_the_person_opened_comes_in_front());
    assert_eq!(
        the_front_one(&f),
        Some(theirs.clone()),
        "the control: the first came in front"
    );
    let (mut second, _) = mapped(&f, "org.kde.dolphin");

    f.backend(|s| s.what_the_person_opened_comes_in_front());
    second.sync();

    assert_eq!(
        the_front_one(&f),
        Some(theirs),
        "a spent start brought a second window in front"
    );
    assert!(
        second.events.keyboard.surfaces.is_empty(),
        "a spent start gave a second window the keyboard"
    );
}

/// **A window nobody asked for opens behind, without the keyboard.**
#[test]
fn a_window_nobody_asked_for_does_not_take_the_front() {
    let (f, mut editor) = a_desktop_where_the_person_asked_for(None, true);
    let front = the_front_one(&f);
    let (mut stranger, _) = mapped(&f, "org.kde.dolphin");

    f.backend(|s| s.what_the_person_opened_comes_in_front());
    stranger.sync();
    editor.sync();

    assert_eq!(
        the_front_one(&f),
        front,
        "a window nobody asked for came in front"
    );
    assert!(
        stranger.events.keyboard.surfaces.is_empty(),
        "a window nobody asked for took the keyboard"
    );
    assert_eq!(
        editor.events.keyboard.leaves, 0,
        "the editor lost the keyboard to a window nobody asked for"
    );
}

/// **A start that failed waits for nothing.**
#[test]
fn a_start_that_failed_brings_nothing_in_front() {
    let (f, _editor) = a_desktop_where_the_person_asked_for(Some("org.kde.dolphin"), false);
    let front = the_front_one(&f);
    let (mut files, _) = mapped(&f, "org.kde.dolphin");

    f.backend(|s| s.what_the_person_opened_comes_in_front());
    files.sync();

    assert_eq!(
        the_front_one(&f),
        front,
        "a refused start brought a window in front"
    );
    assert!(files.events.keyboard.surfaces.is_empty());
}

/// **A dialog the person's application opens comes in front of it, with the
/// keyboard**: a new window of the application that holds the keyboard.
#[test]
fn a_dialog_of_the_application_the_person_is_in_comes_in_front() {
    let (f, mut editor) = a_desktop_where_the_person_asked_for(None, true);
    let before = front_to_back(&f);
    let (dialog, _xdg, _toplevel) = editor.another_window("org.example.Editor");
    let theirs = the_new_one(&f, &before);

    f.backend(|s| s.what_the_person_opened_comes_in_front());
    editor.sync();

    assert_eq!(
        the_front_one(&f),
        Some(theirs),
        "the dialog the person's application opened is not in front"
    );
    assert_eq!(
        editor.events.keyboard.surfaces.last(),
        Some(&dialog.id().protocol_id()),
        "the dialog did not get the keyboard"
    );
}

/// **A dialog of an application that does not hold the keyboard stays
/// behind** — a background application must not steal it.
#[test]
fn a_dialog_of_a_background_application_stays_behind() {
    let (f, mut editor) = a_desktop_where_the_person_asked_for(None, true);
    let (mut background, _) = mapped(&f, "org.example.Background");
    let front = the_front_one(&f);
    let before = front_to_back(&f);
    let _dialog = background.another_window("org.example.Background");
    let theirs = the_new_one(&f, &before);

    f.backend(|s| s.what_the_person_opened_comes_in_front());
    background.sync();
    editor.sync();

    assert_eq!(the_front_one(&f), front);
    assert_ne!(the_front_one(&f), Some(theirs));
    assert!(
        background.events.keyboard.surfaces.is_empty(),
        "a background application's dialog took the keyboard"
    );
    assert_eq!(editor.events.keyboard.leaves, 0);
}
