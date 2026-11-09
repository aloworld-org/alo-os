//! The password sheet as drawn: the command whole, the sentence the person is
//! owed, nothing about the password but whether there is one, and a display
//! too small refused rather than drawn in part.

use std::sync::mpsc::channel;

use alo_appearance::{Scheme, TextScale};
use alo_strings::{Strings, Vocabulary};

use super::*;
use crate::SignInKey;
use crate::the_password_sheet::{PasswordAsked, ThePasswordSheet, ToTheSheet};

/// One to one.
const ONE: crate::display_scale::DisplayScale = crate::display_scale::DisplayScale::ONE;

/// The machine's words with the sheet's declared.
fn words() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    crate::the_password_sheets_words::declare_the_password_sheets_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The ordinary light look.
fn light() -> TheSheetsLook {
    TheSheetsLook {
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
        contrast: crate::Contrast::AsDesigned,
    }
}

/// A sheet with alo's command up, and `typed` typed.
fn a_sheet(command: &str, typed: &str) -> ThePasswordSheet {
    let (answer, _answered) = channel();
    let mut sheet = ThePasswordSheet::none();
    sheet.told(ToTheSheet::Ask(PasswordAsked {
        cookie: "c1".to_owned(),
        what_for: WhatItIsFor::ACommandAloWasAsked(command.to_owned()),
        answer,
    }));
    for letter in typed.chars() {
        sheet.pressed(SignInKey::Letter(letter));
    }
    sheet
}

/// The sheet drawn on `size`.
fn on(sheet: &ThePasswordSheet, size: (i32, i32)) -> Result<ApprovalPicture, RenderError> {
    let mut labels = WindowControlLabels::new().unwrap();
    drawn(sheet.shows().unwrap(), &words(), &mut labels, size, light(), ONE)
}

/// **The command is drawn whole, as it will run**, inside the panel, and the
/// panel is in the middle of the display.
#[test]
fn the_command_is_drawn_whole() {
    let command = "dnf install -y htop";
    let picture = on(&a_sheet(command, ""), (1920, 1080)).unwrap();
    let (text, area) = picture.sentence.clone().unwrap();
    assert_eq!(text, command);
    let panel = picture.panel.unwrap();
    assert!(panel.contains_rect(area), "{area:?} outside {panel:?}");
    assert_eq!(panel.loc.x * 2 + panel.size.w, 1920);
    assert!(!picture.inked.is_empty());
}

/// **How long the password is, is never drawn**: one letter and twenty draw
/// the same field.
#[test]
fn how_long_the_password_is_is_never_drawn() {
    let short = on(&a_sheet("true", "1"), (1920, 1080)).unwrap();
    let long = on(&a_sheet("true", "correct horse battery staple"), (1920, 1080)).unwrap();
    assert_eq!(short.solids, long.solids);
    let empty = on(&a_sheet("true", ""), (1920, 1080)).unwrap();
    assert_ne!(empty.solids, short.solids, "typed and not typed are told apart");
}

/// **A display too small for the sheet is refused, never drawn in part.**
#[test]
fn a_display_too_small_is_refused() {
    let long = "echo ".to_owned() + &"a very long argument ".repeat(150);
    assert!(matches!(
        on(&a_sheet(&long, ""), (320, 240)),
        Err(RenderError::ApprovalScene)
    ));
    assert!(matches!(
        on(&a_sheet("true", ""), (100, 800)),
        Err(RenderError::ApprovalScene)
    ));
}

/// **It is laid out on a small screen and a large one alike.**
#[test]
fn it_is_laid_out_on_small_and_large_displays() {
    for size in [(1280, 720), (1440, 960), (3840, 2160)] {
        assert!(on(&a_sheet("dnf upgrade -y", "12"), size).is_ok(), "{size:?}");
    }
}
