//! **Every first-start sheet fits a small screen**, with every control on it
//! and every row of every list reachable.
//!
//! Until 2026-10-30 the keyboard sheet was refused on a 1024 × 768 display at
//! 100 per cent, and a refused sheet is a blank screen: a person with that
//! laptop could not choose a keyboard or reach the account sheets. The sizes
//! here are that display; 1366 × 768, the smallest screen alo OS lays its Dock
//! out for (`alo_dock::screen::Screen::the_smallest`); and
//! [`THE_SMALLEST_SHEET_IS`], the smallest a setup sheet lays out on at all.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use smithay::utils::{Physical, Rectangle, Size};

use super::the_first_start_at_one_hundred_tests::{every_sheet, light, the_sheet};
use crate::a_press_on_a_setup_sheet::{OnASheet, SheetControl};
use crate::display_scale::DisplayScale;
use crate::setup_sheet_layout::{THE_SMALLEST_SHEET_IS, TheTextIsTall, setup_sheet_laid_out};
use crate::sign_in_raster::{Measure, SignInPicture};
use crate::{SignInKey, WindowControlLabels};

/// The small screens every sheet must fit.
const SMALL_SCREENS: [(i32, i32); 3] = [(1024, 768), (1366, 768), THE_SMALLEST_SHEET_IS];

/// Every sheet a fresh machine shows, by the keys that reach it: the six the
/// 100 per cent hashes walk, and *every region* behind *Other region*.
fn every_sheet_and_every_region() -> Vec<Vec<SignInKey>> {
    let mut sheets: Vec<Vec<SignInKey>> = every_sheet().into_iter().collect();
    sheets.push(vec![
        SignInKey::Later,
        SignInKey::Enter,
        SignInKey::Later,
        SignInKey::Enter,
    ]);
    sheets
}

/// Whether one rectangle lies wholly inside another.
fn inside(it: Rectangle<i32, Physical>, screen: (i32, i32)) -> bool {
    it.loc.x >= 0
        && it.loc.y >= 0
        && it.loc.x + it.size.w <= screen.0
        && it.loc.y + it.size.h <= screen.1
}

/// Whether two rectangles share any pixel.
fn overlap(a: Rectangle<i32, Physical>, b: Rectangle<i32, Physical>) -> bool {
    a.loc.x < b.loc.x + b.size.w
        && b.loc.x < a.loc.x + a.size.w
        && a.loc.y < b.loc.y + b.size.h
        && b.loc.y < a.loc.y + a.size.h
}

/// Every control on screen, none on another, and the ones a sheet cannot do
/// without all there: at least one row, the way on and *Accessibility*.
fn every_control_is_on_screen(drawn: &SignInPicture, screen: (i32, i32), on: &str) {
    let controls: &[SheetControl] = &drawn.controls;
    for control in controls {
        assert!(
            inside(control.area, screen),
            "{on}: {control:?} is off a {screen:?} screen"
        );
    }
    for (which, a) in controls.iter().enumerate() {
        for b in controls.iter().skip(which + 1) {
            assert!(!overlap(a.area, b.area), "{on}: {a:?} is on {b:?}");
        }
    }
    for needed in [OnASheet::WayOn, OnASheet::Accessibility] {
        assert!(
            controls.iter().any(|it| it.what == needed),
            "{on}: no {needed:?}"
        );
    }
    assert!(
        controls
            .iter()
            .any(|it| matches!(it.what, OnASheet::Choice(_))),
        "{on}: no row at all"
    );
}

/// **The fault, measured where it was**: the keyboard sheet's blocks as
/// captured — a one-line heading and introduction, three keyboards and the
/// test row, two actions — do not fit above *Accessibility* at 1024 × 768.
/// This is the layout `main` refused, and so drew nothing.
#[test]
fn as_captured_the_keyboard_sheet_does_not_fit_1024_by_768() {
    let keyboard_sheet = TheTextIsTall {
        title: 44,
        introduction: 24,
        consequence: 0,
        choices: 4,
        actions: 2,
    };
    assert!(setup_sheet_laid_out(Size::from((1024, 768)), keyboard_sheet).is_none());
    assert!(
        setup_sheet_laid_out(Size::from((1440, 960)), keyboard_sheet).is_some(),
        "the control: at the design's own size it fits"
    );
}

/// **Every sheet draws on every small screen, with every control on it.**
#[test]
fn every_sheet_fits_every_small_screen_with_every_control_on_it() {
    for screen in SMALL_SCREENS {
        for keys in every_sheet_and_every_region() {
            let on = format!("{keys:?} at {screen:?}");
            let drawn = the_sheet(&keys, screen, DisplayScale::ONE)
                .unwrap_or_else(|why| panic!("{on} was refused: {why:?}"));
            every_control_is_on_screen(&drawn, screen, &on);
        }
    }
}

/// **The time-zone sheet too**, for a region with many zones, which no key
/// walk above reaches.
#[test]
fn the_zone_sheet_fits_every_small_screen() {
    let words = crate::what_this_machine_can_say().unwrap();
    let zones =
        alo_formats::zones::the_zones_in(&alo_formats::RegionCode::named("US").unwrap(), "en");
    assert!(zones.len() > 3, "the control: a list longer than a window");
    let picker = crate::the_zone_picker::TheZonePicker::of(zones);
    for screen in SMALL_SCREENS {
        let mut labels = WindowControlLabels::new().unwrap();
        let drawn = crate::zone_sheet::drawn(
            &picker,
            &mut labels.fonts,
            &words,
            screen,
            crate::sign_in_surface::TheColours::of(
                crate::Contrast::AsDesigned,
                alo_appearance::Scheme::Light,
            ),
            &Measure::at(light().scale, DisplayScale::ONE),
        )
        .unwrap_or_else(|why| panic!("the zone sheet at {screen:?} was refused: {why:?}"));
        every_control_is_on_screen(&drawn, screen, &format!("the zone sheet at {screen:?}"));
    }
}

/// **Every language is reachable on the smallest sheet**: walked to the end
/// with the arrow key, the last one is drawn and can be pressed.
#[test]
fn on_the_smallest_sheet_the_last_language_is_reachable() {
    let every = crate::the_languages_offered::TheLanguagesOffered::every_official();
    let last = every.window_of(usize::MAX).rows.len() - 1;
    let mut keys = vec![SignInKey::Enter];
    keys.extend(std::iter::repeat_n(SignInKey::Later, last));
    let drawn = the_sheet(&keys, THE_SMALLEST_SHEET_IS, DisplayScale::ONE).unwrap();
    assert!(
        drawn
            .controls
            .iter()
            .any(|it| it.what == OnASheet::Choice(last)),
        "the last language is not drawn after walking to it: {:?}",
        drawn.controls
    );
}

/// **A short screen shows fewer rows, not smaller ones**: a row is as tall at
/// 1024 × 768 as at the design's size, so the text in it is not shrunk to fit.
#[test]
fn a_short_screen_shows_fewer_rows_and_never_smaller_ones() {
    let keyboard = [SignInKey::Later, SignInKey::Later, SignInKey::Enter];
    let rows = |drawn: &SignInPicture| -> Vec<Rectangle<i32, Physical>> {
        drawn
            .controls
            .iter()
            .filter(|it| matches!(it.what, OnASheet::Choice(_)))
            .map(|it| it.area)
            .collect()
    };
    let large = rows(&the_sheet(&keyboard, (1440, 960), DisplayScale::ONE).unwrap());
    let small = rows(&the_sheet(&keyboard, (1024, 768), DisplayScale::ONE).unwrap());
    assert!(small.len() < large.len(), "{small:?} beside {large:?}");
    for row in &small {
        assert_eq!(row.size.h, large[0].size.h, "a row was shrunk to fit");
    }
}
