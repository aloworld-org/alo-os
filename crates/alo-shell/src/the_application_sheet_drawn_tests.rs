//! The sheet as drawn: its words naming the application, the three lengths
//! each in its own card at every scale and in order, *Not now* before the navy
//! button from the left at the frame's least widths, the keyboard's length
//! outlined, and a display too small refused rather than drawn in part.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::Instant;

use alo_strings::Vocabulary;

use super::*;
use crate::the_application_sheet::{AnApplicationWanted, ApplicationSheetKey, TheApplicationSheet};
use crate::the_tasks_a_person_opened::WhichTask;

fn words() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    crate::the_application_sheets_words::declare_the_application_sheets_words(&mut vocabulary)
        .unwrap();
    Strings::of(vocabulary)
}

/// A sheet up for VSCodium.
fn a_sheet() -> TheApplicationSheet {
    let mut sheet = TheApplicationSheet::default();
    let _ = sheet.wanted(
        AnApplicationWanted::by(WhichTask::numbered(0), "com.vscodium.codium", "VSCodium"),
        Instant::now(),
    );
    sheet
}

fn drawn_at(
    sheet: &TheApplicationSheet,
    size: (i32, i32),
    per_cent: u16,
) -> Result<ApplicationSheetPicture, RenderError> {
    let mut labels = WindowControlLabels::new().unwrap();
    drawn(
        sheet.shows().unwrap(),
        &words(),
        &mut labels,
        size,
        crate::settings_testing::light(),
        DisplayScale::of_per_cent(per_cent),
    )
}

/// Where `on` is drawn.
fn where_is(drawn: &ApplicationSheetPicture, on: OnTheApplicationSheet) -> Area {
    drawn
        .targets
        .iter()
        .find(|(it, _)| *it == on)
        .map(|(_, area)| *area)
        .unwrap()
}

/// **Every sentence names the application, and the lengths, the note and the
/// buttons are laid out as the frame lays them out, at one scale and two.**
#[test]
fn the_sheet_says_what_the_frame_says_at_every_scale() {
    let sheet = a_sheet();
    for (size, per_cent, factor) in [((1440, 960), 100, 1), ((2880, 1920), 200, 2)] {
        let drawn = drawn_at(&sheet, size, per_cent).unwrap();
        for said in [
            "Let alo use VSCodium?",
            "alo asked to use VSCodium. Choose how long.",
            "This task",
            "alo may use it in any task until this task's window closes or you stop it",
            "This session",
            "Until you sign out",
            "Remember",
            "Until you take it back in Settings",
            "Only VSCodium is included. Press Escape to say not now.",
            "Not now",
            "Let alo use VSCodium",
        ] {
            assert!(
                drawn.texts.iter().any(|it| it == said),
                "{said}: {:?}",
                drawn.texts
            );
        }
        let lengths: Vec<Area> = HowLongForAnApplication::ALL
            .iter()
            .map(|how_long| where_is(&drawn, OnTheApplicationSheet::For(*how_long)))
            .collect();
        for pair in lengths.windows(2) {
            let [above, below] = pair else {
                unreachable!("a window of two is two");
            };
            assert_eq!(
                below.loc.y - (above.loc.y + above.size.h),
                16 * factor,
                "the lengths are not 16 apart: {pair:?}"
            );
        }
        let not_now = where_is(&drawn, OnTheApplicationSheet::NotNow);
        let navy = where_is(&drawn, OnTheApplicationSheet::LetAloUseIt);
        assert_eq!(
            not_now.loc.x,
            drawn.sheet.loc.x + 25 * factor,
            "not from the left"
        );
        assert_eq!(navy.loc.x, not_now.loc.x + not_now.size.w + 12 * factor);
        assert!(not_now.size.w >= 160 * factor);
        assert!(navy.size.w >= 280 * factor);
        assert_eq!(navy.size.h, 48 * factor);
        assert_eq!(drawn.sheet.size.w, 680 * factor);
        assert!(
            drawn.sheet.contains(navy.loc)
                && drawn.sheet.loc.y + drawn.sheet.size.h >= navy.loc.y + navy.size.h,
            "a button falls off the sheet"
        );
    }
}

/// **Moving the keyboard changes the picture**, so the length it is on is
/// drawn rather than only remembered.
#[test]
fn the_length_the_keyboard_is_on_is_drawn() {
    let mut sheet = a_sheet();
    let before = drawn_at(&sheet, (1440, 960), 100).unwrap();
    let _ = sheet.pressed(ApplicationSheetKey::Next, Instant::now());
    let after = drawn_at(&sheet, (1440, 960), 100).unwrap();
    assert_ne!(before.washed, after.washed);
    assert_eq!(
        before.targets, after.targets,
        "moving the keyboard moved a length"
    );
}

/// **A narrow display draws a narrower sheet, whole**; one too small is
/// refused, never drawn in part.
#[test]
fn a_narrow_display_draws_it_narrower_and_one_too_small_is_refused() {
    let sheet = a_sheet();
    let narrow = drawn_at(&sheet, (600, 900), 100).unwrap();
    assert!(narrow.sheet.size.w <= 600 - 2 * 16);
    assert!(drawn_at(&sheet, (200, 960), 100).is_err());
    assert!(drawn_at(&sheet, (1440, 200), 100).is_err());
}
