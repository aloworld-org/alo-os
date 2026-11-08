//! **Alt+F4 at one of alo's own windows never closes the application behind
//! it**, over a real client socket.
//!
//! alo's terminal, Settings and the record are drawn by the compositor and
//! take the keys without moving the seat's focus, so the application that had
//! the keyboard keeps it underneath. Until 2026-11-08 Alt+F4 went to the road
//! that closes *the application with the seat's focus*: walking a booted
//! machine, Alt+F4 in alo's terminal left the terminal up and the file manager
//! behind it was gone. Settings is the window used here because opening it is
//! a public road (`⊞`+I) and it needs no shell; the terminal and the record
//! take the same link, held in the crate's own tests.
//!
//! The control is the second press: with Settings closed, Alt+F4 is the
//! application's, and it is asked to close exactly once.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use smithay::backend::input::KeyState;

use super::support::{Application, Fixture};
use alo_shell::SettingsPlaces;
use alo_shortcuts::Shortcuts;

/// evdev's codes.
const SUPER: u32 = 125;
const LETTER_I: u32 = 23;
const LEFT_ALT: u32 = 56;
const F4: u32 = 62;

/// One mapped application window with a connected client.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// `modifier` and `key`, held and released the way a person presses them.
fn chord(f: &Fixture, modifier: u32, key: u32) {
    for (code, state) in [
        (modifier, KeyState::Pressed),
        (key, KeyState::Pressed),
        (key, KeyState::Released),
        (modifier, KeyState::Released),
    ] {
        let _ = f.key(code, state).expect("the seat took the key");
    }
}

/// Where Settings looks on a machine where nothing has been kept yet.
fn nothing_kept_yet(dir: &std::path::Path) -> SettingsPlaces {
    SettingsPlaces::of(
        None,
        None,
        None,
        &dir.join("grants"),
        &dir.join("pairings"),
        &dir.join("agentd.sock"),
        &dir.join("how-code-runs.toml"),
        None,
    )
}

#[test]
fn alt_f4_in_settings_closes_settings_and_not_the_application_behind() {
    let f = Fixture::keyboard();
    let mut app = mapped(&f);
    let root = f.root();
    f.focus_surface(root)
        .expect("the application has the keyboard");
    let held = tempfile::tempdir().expect("a folder");
    let places = nothing_kept_yet(held.path());
    f.backend(move |server| {
        server.the_shortcuts_are(Shortcuts::shipped());
        server.the_settings_places_are(places);
    });

    chord(&f, SUPER, LETTER_I);
    assert!(
        f.backend(|server| server.settings_is_taking_the_keys()),
        "the control: ⊞+I opened Settings in front"
    );

    chord(&f, LEFT_ALT, F4);
    app.sync();
    assert!(
        !f.backend(|server| server.settings_is_taking_the_keys()),
        "Alt+F4 left Settings open"
    );
    assert_eq!(
        app.events.close_requests, 0,
        "Alt+F4 at Settings asked the application behind it to close"
    );

    chord(&f, LEFT_ALT, F4);
    app.sync();
    assert_eq!(
        app.events.close_requests, 1,
        "with Settings gone, Alt+F4 is the application's and asks it once"
    );
}
