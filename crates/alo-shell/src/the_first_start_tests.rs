//! The first-start sheets, walked with the keys a person has.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_formats::RegionCode;
use alo_keyboards::Layout;

use super::{Row, Sheet, TheFirstStart, Went};
use crate::sign_in_keys::SignInKey;
use crate::the_keyboard_picker::TheRing;

fn walked(first_start: &mut TheFirstStart, keys: &[SignInKey]) -> Went {
    let mut went = Went::Stayed;
    for key in keys {
        went = first_start.pressed(*key);
    }
    went
}

fn german() -> Layout {
    Layout::named("de").unwrap()
}

/// **The overview arrives with what the machine would do**: the language it
/// speaks, the region its clock says, and that language's keyboard — and
/// reports nothing chosen, so nothing is written before a person chooses.
#[test]
fn the_overview_arrives_with_what_the_machine_would_do() {
    let first_start = TheFirstStart::arriving(RegionCode::named("IE").ok());
    assert_eq!(first_start.sheet(), &Sheet::Overview);
    assert_eq!(first_start.the_language(), "en");
    assert_eq!(
        first_start
            .the_region()
            .map(|it| it.code().to_owned())
            .as_deref(),
        Some("IE")
    );
    assert_eq!(first_start.the_keyboard(), Layout::named("us").unwrap());
    assert_eq!(first_start.the_language_chosen(), None);
    assert_eq!(first_start.the_keyboard_chosen(), None);
    assert_eq!(first_start.the_region_chosen(), None);
}

/// **English read on a German keyboard** — the owner's own example. The
/// language stays English, the keyboard is German, and it is applied the
/// moment it is chosen, before *Use this keyboard*.
#[test]
fn english_read_on_a_german_keyboard() {
    let mut first_start = TheFirstStart::arriving(None);
    walked(
        &mut first_start,
        &[SignInKey::Later, SignInKey::Later, SignInKey::Enter],
    );
    let Sheet::Keyboard(picker) = first_start.sheet() else {
        panic!("the keyboard row did not open the keyboard sheet");
    };
    assert_eq!(picker.chosen().map(Layout::name), Some("us"));
    for _ in 0..200 {
        let Sheet::Keyboard(picker) = first_start.sheet() else {
            panic!("the sheet closed");
        };
        if picker.window().get(picker.cursor_in_the_window().unwrap()) == Some(&german()) {
            break;
        }
        first_start.pressed(SignInKey::Later);
    }
    first_start.pressed(SignInKey::Enter);
    assert_eq!(
        first_start.the_keyboard_chosen(),
        Some(german()),
        "not applied before use"
    );
    // The test row has the ring now; a letter typed there is the test.
    first_start.pressed(SignInKey::Letter('ö'));
    let Sheet::Keyboard(picker) = first_start.sheet() else {
        panic!("the sheet closed");
    };
    assert_eq!(picker.ring(), TheRing::OnTheTest);
    assert_eq!(picker.typed(), "ö");
    // Tab to `Use this keyboard`, and take it.
    walked(&mut first_start, &[SignInKey::OtherField, SignInKey::Enter]);
    assert_eq!(first_start.sheet(), &Sheet::Overview);
    assert_eq!(first_start.the_keyboard_chosen(), Some(german()));
    assert_eq!(first_start.the_language(), "en");
    assert_eq!(
        first_start.the_language_chosen(),
        None,
        "the keyboard chose a language"
    );
}

/// **A language chosen moves the defaults it owns, and not what was chosen.**
/// German makes the keyboard `de` while nobody has chosen one; a keyboard
/// chosen first stays when the language changes after.
#[test]
fn a_language_moves_its_defaults_and_not_what_was_chosen() {
    let mut first_start = TheFirstStart::arriving(None);
    // Language row: from English, two up is Deutsch; choose it and use it.
    walked(
        &mut first_start,
        &[
            SignInKey::Enter,
            SignInKey::Earlier,
            SignInKey::Earlier,
            SignInKey::Enter,
            SignInKey::Enter,
        ],
    );
    assert_eq!(first_start.sheet(), &Sheet::Overview);
    assert_eq!(first_start.the_language_chosen(), Some("de"));
    assert_eq!(first_start.the_keyboard(), german());
    assert_eq!(
        first_start
            .the_region()
            .map(|it| it.code().to_owned())
            .as_deref(),
        Some("DE")
    );

    // A keyboard chosen now — the first offered after German's own — stays
    // when the language changes back.
    walked(
        &mut first_start,
        &[
            SignInKey::Later,
            SignInKey::Later,
            SignInKey::Enter,
            SignInKey::Later,
            SignInKey::Enter,
        ],
    );
    walked(&mut first_start, &[SignInKey::OtherField, SignInKey::Enter]);
    let chosen = first_start.the_keyboard();
    assert_ne!(chosen, german());
    walked(
        &mut first_start,
        &[
            SignInKey::Earlier,
            SignInKey::Earlier,
            SignInKey::Enter,
            SignInKey::Later,
            SignInKey::Later,
            SignInKey::Enter,
            SignInKey::Enter,
        ],
    );
    assert_eq!(first_start.the_language_chosen(), Some("en"));
    assert_eq!(
        first_start.the_keyboard(),
        chosen,
        "a chosen keyboard followed the language"
    );
}

/// **Back on any picker changes nothing**, and the overview's Escape goes
/// nowhere because there is nowhere further back.
#[test]
fn back_changes_nothing() {
    let mut first_start = TheFirstStart::arriving(None);
    walked(
        &mut first_start,
        &[
            SignInKey::Enter,
            SignInKey::Earlier,
            SignInKey::Enter,
            SignInKey::Clear,
        ],
    );
    assert_eq!(first_start.sheet(), &Sheet::Overview);
    assert_eq!(
        first_start.the_language_chosen(),
        None,
        "back kept a language"
    );
    walked(
        &mut first_start,
        &[
            SignInKey::Later,
            SignInKey::Later,
            SignInKey::Enter,
            SignInKey::Later,
            SignInKey::Enter,
            SignInKey::Clear,
        ],
    );
    assert_eq!(first_start.sheet(), &Sheet::Overview);
    assert_eq!(
        first_start.the_keyboard_chosen(),
        None,
        "back kept a keyboard"
    );
    first_start.pressed(SignInKey::Clear);
    assert_eq!(first_start.sheet(), &Sheet::Overview);
}

/// **The region row takes a region**, and *Continue* goes on.
#[test]
fn a_region_is_taken_and_continue_goes_on() {
    let mut first_start = TheFirstStart::arriving(RegionCode::named("AT").ok());
    walked(&mut first_start, &[SignInKey::Later, SignInKey::Enter]);
    assert!(matches!(first_start.sheet(), Sheet::Region(_)));
    walked(&mut first_start, &[SignInKey::Enter, SignInKey::Enter]);
    assert_eq!(first_start.sheet(), &Sheet::Overview);
    assert_eq!(
        first_start.the_region_chosen().map(RegionCode::code),
        Some("AT")
    );
    assert_eq!(first_start.row(), Row::Region);
    assert_eq!(
        walked(&mut first_start, &[SignInKey::OtherField, SignInKey::Enter]),
        Went::Onward
    );
}

/// **A region with one zone sets it**: Germany is Berlin, with nothing asked.
#[test]
fn a_region_with_one_zone_sets_it() {
    let mut first_start = TheFirstStart::arriving(RegionCode::named("DE").ok());
    walked(
        &mut first_start,
        &[
            SignInKey::Later,
            SignInKey::Enter,
            SignInKey::Enter,
            SignInKey::Enter,
        ],
    );
    assert_eq!(
        first_start.sheet(),
        &Sheet::Overview,
        "Germany asked which zone"
    );
    assert_eq!(first_start.the_zone_chosen(), Some("Europe/Berlin"));
    assert_eq!(first_start.the_zone_named(), None);
}

/// **A region with several asks which**: the United States opens the zone
/// sheet with nothing chosen; one chosen and used is the machine's zone, and
/// the region row names it.
#[test]
fn a_region_with_several_zones_asks_which() {
    let mut first_start = TheFirstStart::arriving(RegionCode::named("US").ok());
    walked(
        &mut first_start,
        &[
            SignInKey::Later,
            SignInKey::Enter,
            SignInKey::Enter,
            SignInKey::Enter,
        ],
    );
    let Sheet::Zone(picker) = first_start.sheet() else {
        panic!("the United States did not ask which zone");
    };
    assert_eq!(picker.chosen(), None, "a zone arrived chosen");
    assert_eq!(first_start.the_zone_chosen(), None);
    walked(
        &mut first_start,
        &[SignInKey::Later, SignInKey::Enter, SignInKey::Enter],
    );
    assert_eq!(first_start.sheet(), &Sheet::Overview);
    let chosen = first_start.the_zone_chosen().unwrap().to_owned();
    assert!(
        chosen.starts_with("America/") || chosen.starts_with("Pacific/"),
        "{chosen}"
    );
    assert!(first_start.the_zone_named().is_some());
}

/// **Escape from the zone sheet sets no zone**: the machine keeps the one it
/// had, and the region taken stays taken.
#[test]
fn escape_from_the_zone_sheet_sets_no_zone() {
    let mut first_start = TheFirstStart::arriving(RegionCode::named("RU").ok());
    walked(
        &mut first_start,
        &[
            SignInKey::Later,
            SignInKey::Enter,
            SignInKey::Enter,
            SignInKey::Enter,
        ],
    );
    assert!(matches!(first_start.sheet(), Sheet::Zone(_)));
    walked(&mut first_start, &[SignInKey::Enter, SignInKey::Clear]);
    assert_eq!(first_start.sheet(), &Sheet::Overview);
    assert_eq!(first_start.the_zone_chosen(), None);
    assert_eq!(
        first_start.the_region_chosen().map(RegionCode::code),
        Some("RU")
    );
}

/// **Continue chooses the keyboard the overview names**, with no row touched
/// — and a language chosen after still moves it, because what was accepted
/// was the overview's answer and not one layout frozen at that moment.
#[test]
fn continue_chooses_the_keyboard_the_overview_names() {
    let mut first_start = TheFirstStart::arriving(None);
    assert_eq!(first_start.the_keyboard_chosen(), None);
    assert_eq!(
        walked(&mut first_start, &[SignInKey::OtherField, SignInKey::Enter]),
        Went::Onward
    );
    assert_eq!(
        first_start.the_keyboard_chosen(),
        Some(Layout::named("us").unwrap()),
        "Continue on English (US) chose no keyboard"
    );
    // Back at the overview (the name sheet's Back), choose German.
    walked(
        &mut first_start,
        &[
            SignInKey::OtherField,
            SignInKey::Enter,
            SignInKey::Earlier,
            SignInKey::Earlier,
            SignInKey::Enter,
            SignInKey::Enter,
        ],
    );
    assert_eq!(first_start.the_language_chosen(), Some("de"));
    assert_eq!(first_start.the_keyboard_chosen(), Some(german()));
}

/// **Continue accepts the language, the region and the region's one zone the
/// overview names**, with no row touched — and a person who goes back,
/// changes language and continues again has that language's region instead.
/// A region with several zones names none on the overview, so Continue
/// accepts none there.
#[test]
fn continue_accepts_the_language_region_and_zone_the_overview_names() {
    let mut first_start = TheFirstStart::arriving(RegionCode::named("AT").ok());
    assert_eq!(first_start.the_language_chosen(), None);
    assert_eq!(first_start.the_region_chosen(), None);
    assert_eq!(first_start.the_zone_chosen(), None);
    assert_eq!(
        walked(&mut first_start, &[SignInKey::OtherField, SignInKey::Enter]),
        Went::Onward
    );
    assert_eq!(first_start.the_language_chosen(), Some("en"));
    assert_eq!(
        first_start.the_region_chosen().map(RegionCode::code),
        Some("AT")
    );
    assert_eq!(first_start.the_zone_chosen(), Some("Europe/Vienna"));

    // Nothing on the machine's clock: English's usual region, the United
    // States, has several zones and the overview names none.
    let mut first_start = TheFirstStart::arriving(None);
    walked(&mut first_start, &[SignInKey::OtherField, SignInKey::Enter]);
    assert_eq!(
        first_start.the_region_chosen().map(RegionCode::code),
        Some("US")
    );
    assert_eq!(first_start.the_zone_chosen(), None, "a zone nobody saw");

    // Back at the overview, choose German and continue again.
    walked(
        &mut first_start,
        &[
            SignInKey::OtherField,
            SignInKey::Enter,
            SignInKey::Earlier,
            SignInKey::Earlier,
            SignInKey::Enter,
            SignInKey::Enter,
        ],
    );
    assert_eq!(first_start.the_language_chosen(), Some("de"));
    assert_eq!(
        walked(&mut first_start, &[SignInKey::OtherField, SignInKey::Enter]),
        Went::Onward
    );
    assert_eq!(
        first_start.the_region_chosen().map(RegionCode::code),
        Some("DE")
    );
    assert_eq!(first_start.the_zone_chosen(), Some("Europe/Berlin"));
}
