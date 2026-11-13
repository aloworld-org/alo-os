//! The GitHub sign-in sheet as drawn: the proposal's words in each state, its
//! buttons 48 tall at one scale and 96 at two, the navy button at the right,
//! everything on the sheet and the sheet on the display — at 1280 × 800 and
//! 1440 × 960, at scale 1 and 2 — and a display too small refused.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::Strings;

use super::*;
use crate::asking_github::GitHubAnswered;
use crate::the_github_sheet::TheGitHubSheet;

/// Settings' words and the sheet's.
fn words() -> Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::the_settings_words::declare_the_settings_words(&mut vocabulary).unwrap();
    crate::the_github_sheets_words::declare_the_github_sheets_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The sheet waiting, with the code, and signed in.
fn every_state() -> [TheGitHubSheet; 3] {
    let mut waiting = TheGitHubSheet::default();
    assert!(waiting.opened());
    let mut with_the_code = waiting.clone();
    assert!(with_the_code.answered(&GitHubAnswered::TheCodeIs {
        code: "4F2A-9C1D".to_owned(),
        page: "https://github.com/login/device".to_owned(),
    }));
    let mut signed_in = with_the_code.clone();
    assert!(!signed_in.answered(&GitHubAnswered::SignedIn("anna".to_owned())));
    [waiting, with_the_code, signed_in]
}

/// `sheet` drawn for a display of `size` at `per_cent`.
fn drawn_at(
    sheet: &TheGitHubSheet,
    size: (i32, i32),
    per_cent: u16,
) -> Result<GitHubSheetPicture, RenderError> {
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

/// Whether `inner` lies wholly inside `outer`.
fn inside(inner: Area, outer: Area) -> bool {
    inner.loc.x >= outer.loc.x
        && inner.loc.y >= outer.loc.y
        && inner.loc.x + inner.size.w <= outer.loc.x + outer.size.w
        && inner.loc.y + inner.size.h <= outer.loc.y + outer.size.h
}

/// The displays the sheet is drawn on: 1280 × 800 and 1440 × 960 at scale 1,
/// and the same at scale 2, with the factor a logical measure is multiplied by.
const DISPLAYS: [((i32, i32), u16, i32); 4] = [
    ((1280, 800), 100, 1),
    ((1440, 960), 100, 1),
    ((2560, 1600), 200, 2),
    ((2880, 1920), 200, 2),
];

/// **Each state says what the proposal draws**, at every display.
#[test]
fn each_state_says_what_the_proposal_draws() {
    let [waiting, with_the_code, signed_in] = every_state();
    let risks = "This sign-in lets GitHub's own command, gh, read and change every repository \
                 your GitHub account can, public and private, and create new ones. It is kept on \
                 this computer until you sign out. Anything you run can use it — and at full \
                 access, so can alo.";
    let sentence =
        "GitHub will ask for this code. Enter it on the page that opens in your browser.";
    for (size, per_cent, _) in DISPLAYS {
        for (sheet, said, buttons) in [
            (
                &waiting,
                vec![
                    "Sign in to GitHub",
                    sentence,
                    "Checking…",
                    "What it risks",
                    risks,
                    "Cancel",
                ],
                vec![OnTheGitHubSheet::Cancel],
            ),
            (
                &with_the_code,
                vec![
                    "Sign in to GitHub",
                    sentence,
                    "4F2A-9C1D",
                    "Copy",
                    "What it risks",
                    risks,
                    "Cancel",
                    "Open github.com/login/device",
                ],
                vec![
                    OnTheGitHubSheet::Copy,
                    OnTheGitHubSheet::Cancel,
                    OnTheGitHubSheet::OpenThePage,
                ],
            ),
            (
                &signed_in,
                vec![
                    "Signed in to GitHub as anna",
                    "Signed in. You can sign out in Settings → Accounts.",
                    "What it risks",
                    risks,
                    "Done",
                ],
                vec![OnTheGitHubSheet::Done],
            ),
        ] {
            let drawn = drawn_at(sheet, size, per_cent).unwrap();
            let texts: Vec<&str> = drawn.texts.iter().map(|(text, _)| text.as_str()).collect();
            let mut sorted_said = said.clone();
            sorted_said.sort_unstable();
            let mut sorted_texts = texts.clone();
            sorted_texts.sort_unstable();
            assert_eq!(sorted_texts, sorted_said, "{size:?} at {per_cent}");
            let mut on: Vec<OnTheGitHubSheet> = drawn.buttons.iter().map(|(on, _)| *on).collect();
            on.sort_by_key(|it| format!("{it:?}"));
            let mut expected = buttons.clone();
            expected.sort_by_key(|it| format!("{it:?}"));
            assert_eq!(on, expected, "{size:?} at {per_cent}");
            assert!(
                !texts.iter().any(|text| text.contains("assword")),
                "never a password field"
            );
        }
    }
}

/// **Nothing is off the sheet and the sheet is not off the display**, the
/// buttons are 48 tall at one scale and 96 at two, the sheet is the proposal's
/// 480 wide, and the navy button is at the right of *Cancel*.
#[test]
fn nothing_is_off_the_sheet_at_any_display() {
    for (size, per_cent, factor) in DISPLAYS {
        for sheet in every_state() {
            let drawn = drawn_at(&sheet, size, per_cent).unwrap();
            let screen: Area = Rectangle::new((0, 0).into(), size.into());
            assert!(inside(drawn.sheet, screen), "{size:?} at {per_cent}");
            assert_eq!(drawn.sheet.size.w, 480 * factor);
            for (text, area) in &drawn.texts {
                assert!(inside(*area, drawn.sheet), "{text} at {size:?} {per_cent}");
            }
            for (on, area) in &drawn.buttons {
                assert!(inside(*area, drawn.sheet), "{on:?} at {size:?} {per_cent}");
                assert_eq!(area.size.h, 48 * factor, "{on:?}");
            }
            let at = |which: OnTheGitHubSheet| {
                drawn
                    .buttons
                    .iter()
                    .find(|(on, _)| *on == which)
                    .map(|(_, area)| *area)
            };
            if let (Some(cancel), Some(open)) = (
                at(OnTheGitHubSheet::Cancel),
                at(OnTheGitHubSheet::OpenThePage),
            ) {
                assert!(cancel.loc.x + cancel.size.w <= open.loc.x);
                assert_eq!(cancel.loc.y, open.loc.y);
            }
            assert!(inside(
                crate::painted::on_the_frame(drawn.washed.area),
                screen
            ));
        }
    }
}

/// **The sheet is drawn over the desktop and dims nothing**: the raster is the
/// sheet and its shadow, never the whole display.
#[test]
fn nothing_behind_the_sheet_is_dimmed() {
    for (size, per_cent, _) in DISPLAYS {
        for sheet in every_state() {
            let drawn = drawn_at(&sheet, size, per_cent).unwrap();
            assert!(drawn.washed.area.size.w < size.0, "{size:?}");
            assert!(drawn.washed.area.size.h < size.1, "{size:?}");
        }
    }
}

/// **A display too small to hold the sheet is refused**, never drawn in part.
#[test]
fn a_display_too_small_is_refused() {
    for sheet in every_state() {
        assert!(drawn_at(&sheet, (200, 960), 100).is_err());
        assert!(drawn_at(&sheet, (1440, 200), 100).is_err());
    }
}
