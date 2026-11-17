//! **Who reads the pictures, in the bytes a display is handed** — the Settings
//! raster put in the native layers over the desktop and painted by the
//! compositor's own software renderer, as `pictures_and_keys_drawn_tests` does
//! for the switches above it.
//!
//! When `ALO_WHO_READS_PICTURES_PICTURES` names a folder the frames are written
//! there as PNGs, which is a check somebody runs by hand. What runs on every
//! build is that choosing a service changes the frame the card is handed, and
//! that the choice was written to the file before it was drawn.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::giving_an_application_drawn_tests::{focused_on, handed, press, written_where};
use crate::settings_testing::{CountingDoor, a_persons_machine};
use crate::{SettingsKey, SettingsRow};
use alo_capability::WhoReadsPictures;

/// Where the pictures go, when somebody asks for them.
const THE_FOLDER: &str = "ALO_WHO_READS_PICTURES_PICTURES";

/// A game and a browser, as the seat would tell Settings of them.
fn a_game_and_a_browser() -> crate::settings_default_apps::WhatCouldOpenLinks {
    use alo_applications::Application;
    let game = Application::called("net.supertuxkart.SuperTuxKart", "SuperTuxKart").unwrap();
    let firefox = Application::called("org.mozilla.firefox", "Firefox").unwrap();
    crate::settings_default_apps::WhatCouldOpenLinks {
        installed: alo_applications::Installed::holding([game, firefox]),
        declared: alo_applications::Declared::nothing(),
        artwork: std::sync::Arc::new(alo_dock::artwork::TheArtwork::none()),
    }
}

/// **No one, then a service the person added, each reaching the frame**: the
/// frame with the keyboard on the service and the frame after Enter differ, so
/// a focus ring moving is never what made them differ.
#[test]
fn choosing_who_reads_the_pictures_reaches_the_frame_a_display_is_handed() {
    let machine = a_persons_machine("readers-drawn");
    alo_choosing::Choosing::at(&machine.choosing())
        .unwrap()
        .adding(
            alo_models::Provider::checked(
                "Mistral",
                "https://api.mistral.ai",
                alo_models::Region::Declared("the EU".to_owned()),
                None,
            )
            .unwrap(),
        )
        .unwrap();
    let door = CountingDoor::revoking();
    let mut window = machine.opened();
    window.the_applications_are(a_game_and_a_browser());
    let mistral = WhoReadsPictures::a_provider("Mistral").unwrap();
    let row = SettingsRow::WhoReadsPictures(mistral.clone());
    focused_on(&mut window, &row, &door);
    let before = handed(&window);
    written_where(&before, THE_FOLDER, "1-no-one-reads-the-pictures.png");
    press(&mut window, SettingsKey::Choose, &door);
    let after = handed(&window);
    written_where(&after, THE_FOLDER, "2-mistral-reads-the-pictures.png");
    assert_ne!(
        before.0, after.0,
        "Enter on the service did not reach the frame"
    );
    assert_eq!(
        alo_remembering::who_reads_pictures_remembered(machine.places.how_code_runs()).unwrap(),
        mistral,
        "the frame shows a choice the file does not say"
    );
}
