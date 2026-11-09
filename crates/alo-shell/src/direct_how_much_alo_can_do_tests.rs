//! **A click on the sheet lands on what was drawn, and one Continue writes
//! once** — checked on the lane, which needs no input devices.
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::{Scheme, TextScale};
use alo_capability::HowCodeRuns;

use super::{TheSheet, TheSheetsMeans};
use crate::a_press_on_a_setup_sheet::{OnASheet, SheetControl};
use crate::display_scale::DisplayScale;
use crate::how_much_alo_can_do::{HowMuchAloCanDo, ToAsk, Went};
use crate::sign_in_raster::SignInLook;
use crate::{DirectPointerEvent, SignInKey, WindowControlLabels};

/// The design's own frame size.
const SIZE: (i32, i32) = (1440, 960);

/// The sheet for a person whose level would be kept at `file`.
fn asked(file: &std::path::Path) -> HowMuchAloCanDo {
    match HowMuchAloCanDo::to_ask(file) {
        ToAsk::Ask(it) => it,
        other => panic!("not asked: {other:?}"),
    }
}

/// The middle of the control `what` the last frame drew.
fn the_middle_of(controls: &[SheetControl], what: OnASheet) -> (i32, i32) {
    let it = controls
        .iter()
        .find(|it| it.what == what)
        .unwrap_or_else(|| panic!("{what:?} is not drawn"));
    (
        it.area.loc.x + it.area.size.w / 2,
        it.area.loc.y + it.area.size.h / 2,
    )
}

/// Point at a pixel and press.
fn click(sheet: &mut TheSheet<'_>, (x, y): (i32, i32)) {
    sheet
        .pointed(&DirectPointerEvent::Absolute {
            x: f64::from(x) / f64::from(SIZE.0 - 1),
            y: f64::from(y) / f64::from(SIZE.1 - 1),
            time: 1,
        })
        .expect("a pointer inside the screen");
    sheet.pressed_by_the_pointer();
}

/// **A click on the sealed box and then on Continue writes the sealed box**,
/// once: a second Continue after the sheet is done writes nothing.
#[test]
fn a_click_on_a_card_then_continue_writes_that_level_once() {
    let at = tempfile::tempdir().expect("a temporary directory");
    let file = at.path().join("how-code-runs.toml");
    let mut labels = WindowControlLabels::new().expect("the font");
    let words = crate::what_this_machine_can_say().expect("the words");
    let mut sheet = TheSheet::of(
        asked(&file),
        TheSheetsMeans {
            labels: &mut labels,
            strings: &words,
            look: SignInLook {
                contrast: crate::Contrast::AsDesigned,
                scheme: Scheme::Light,
                scale: TextScale::ordinary(),
            },
            the_sealed_box_runs_here: true,
        },
    );
    // **A press before anything is drawn lands on nothing.**
    click(&mut sheet, (200, 300));
    assert_eq!(sheet.asking().chosen(), HowCodeRuns::AtFullTrust);

    let drawn = sheet.drawn_for(SIZE, DisplayScale::ONE).expect("drawn");
    // The ground is nothing.
    click(&mut sheet, (5, 5));
    assert_eq!(sheet.asking().chosen(), HowCodeRuns::AtFullTrust);
    click(
        &mut sheet,
        the_middle_of(&drawn.controls, OnASheet::Choice(0)),
    );
    assert_eq!(sheet.asking().chosen(), HowCodeRuns::InASealedBox);
    assert!(!file.exists(), "choosing a card wrote the level");

    let drawn = sheet
        .drawn_for(SIZE, DisplayScale::ONE)
        .expect("drawn again");
    click(&mut sheet, the_middle_of(&drawn.controls, OnASheet::WayOn));
    assert_eq!(sheet.went(), Went::Kept(HowCodeRuns::InASealedBox));
    assert!(sheet.finished());
    assert_eq!(
        alo_remembering::the_level_remembered(&file).expect("reads"),
        Some(HowCodeRuns::InASealedBox)
    );

    // **One Continue, one write**: the file taken away, another Enter, and
    // nothing comes back.
    std::fs::remove_file(&file).expect("removed");
    sheet.pressed(SignInKey::Enter);
    assert!(!file.exists(), "a finished sheet wrote again");
}
