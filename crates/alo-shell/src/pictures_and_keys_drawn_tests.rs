//! **The switches for pictures and keys, in the bytes a display is handed** —
//! the Settings raster put in the native layers over the desktop and painted by
//! the compositor's own software renderer, as
//! `giving_an_application_drawn_tests` does for the list of applications.
//!
//! When `ALO_PICTURES_AND_KEYS_PICTURES` names a folder the frames are written
//! there as PNGs, which is a check somebody runs by hand. What runs on every
//! build is that a press on a switch changes the frame the card is handed, and
//! that the press was written to the file before it was drawn.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::giving_an_application_drawn_tests::{focused_on, handed, press, written_where};
use crate::settings_testing::{CountingDoor, a_persons_machine};
use crate::{SettingsKey, SettingsRow};

/// Where the pictures go, when somebody asks for them.
const THE_FOLDER: &str = "ALO_PICTURES_AND_KEYS_PICTURES";

/// A game, a browser and a terminal, as the seat would tell Settings of them.
fn a_game_a_browser_and_a_terminal() -> crate::settings_default_apps::WhatCouldOpenLinks {
    use alo_applications::Application;
    let game = Application::called("net.supertuxkart.SuperTuxKart", "SuperTuxKart").unwrap();
    let firefox = Application::called("org.mozilla.firefox", "Firefox").unwrap();
    let terminal = Application::called("app.devsuite.Ptyxis", "Terminal").unwrap();
    crate::settings_default_apps::WhatCouldOpenLinks {
        installed: alo_applications::Installed::holding([game, firefox, terminal]),
        declared: alo_applications::Declared::nothing(),
        artwork: std::sync::Arc::new(alo_dock::artwork::TheArtwork::none()),
    }
}

/// **Off, then on in SuperTuxKart alone, each reaching the frame**: the frame
/// with the keyboard on the switch and the frame after Enter differ, so a
/// focus ring moving is never what made them differ.
#[test]
fn a_switch_for_pictures_and_keys_reaches_the_frame_a_display_is_handed() {
    let machine = a_persons_machine("pictures-drawn");
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    window.the_applications_are(a_game_a_browser_and_a_terminal());
    let game = SettingsRow::PicturesAndKeys("net.supertuxkart.SuperTuxKart".to_owned());
    focused_on(&mut window, &game, &door);
    let before = handed(&window);
    written_where(&before, THE_FOLDER, "1-pictures-and-keys-off.png");
    press(&mut window, SettingsKey::Choose, &door);
    let after = handed(&window);
    written_where(
        &after,
        THE_FOLDER,
        "2-pictures-and-keys-on-in-supertuxkart.png",
    );
    assert_ne!(
        before.0, after.0,
        "Enter on the switch did not reach the frame"
    );
    assert!(
        alo_remembering::pictures_and_keys_remembered(machine.places.how_code_runs())
            .unwrap()
            .allows("net.supertuxkart.SuperTuxKart"),
        "the frame shows a switch the file does not say"
    );
}
