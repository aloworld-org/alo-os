//! Tests for [`super`]: the two steps a person takes in Settings to give alo
//! one application — which one, then how long — and what neither step may do.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or variant is the failure being reported"
)]

use super::*;
use crate::settings_testing::settings_words;

/// The editor of the VSCodium demo.
const CODIUM: &str = "com.vscodium.codium";

/// The browser.
const FIREFOX: &str = "org.mozilla.firefox";

/// A machine with a browser, a terminal and an editor, as Settings is told it.
fn this_machine() -> AnApplicationBeingPicked {
    AnApplicationBeingPicked::among(alo_picking::Applications::of([
        (FIREFOX.to_owned(), Some("Firefox".to_owned())),
        (
            "app.devsuite.Ptyxis".to_owned(),
            Some("Terminal".to_owned()),
        ),
        (CODIUM.to_owned(), Some("VSCodium".to_owned())),
    ]))
}

/// What the person reads on each row, in order.
fn read(picking: &AnApplicationBeingPicked) -> Vec<String> {
    let strings = settings_words();
    picking
        .rows()
        .iter()
        .map(|row| picking.said(row, &strings))
        .collect()
}

/// **The first step lists the applications by name, and nothing else**: no
/// terminal, no length, no row that gives anything without a name on it.
#[test]
fn the_first_step_is_the_applications_by_name() {
    let picking = this_machine();
    assert!(!picking.is_saying_how_long());
    assert_eq!(
        picking.rows(),
        [
            ApplicationRow::This(FIREFOX.to_owned()),
            ApplicationRow::This(CODIUM.to_owned()),
        ]
    );
    assert_eq!(read(&picking), ["Firefox", "VSCodium"]);
}

/// **The words are the design's**, declared and not English typed into the
/// code: the heading, the line under it, and the line under the list.
#[test]
fn the_first_step_says_what_the_design_says() {
    let strings = settings_words();
    let picking = this_machine();
    let asking = picking.asking(&strings);
    assert!(!asking.is_a_bug());
    assert_eq!(asking.into_text(), "Select an application");
    assert_eq!(
        picking.beneath(&strings),
        Some("Choose one application to grant access.".to_owned())
    );
    assert_eq!(
        picking.at_the_end(&strings),
        "Only the selected application is included. Press Escape to cancel without granting \
         access."
    );
}

/// **Choosing an application grants nothing yet**: it moves the list on to
/// the four lengths, asked about that application by name.
#[test]
fn choosing_an_application_asks_how_long_and_grants_nothing() {
    let strings = settings_words();
    let mut picking = this_machine();
    let did = picking.act(&ApplicationRow::This(CODIUM.to_owned()));
    assert_eq!(did, WhatChoosingDid::ItChanged);
    assert!(picking.is_saying_how_long());
    assert_eq!(
        picking.rows(),
        HowLong::ALL.map(ApplicationRow::For).to_vec(),
        "the second step is the four lengths and nothing else"
    );
    assert_eq!(
        picking.asking(&strings).into_text(),
        "How long may alo use VSCodium? You can take it back whenever you like"
    );
    assert_eq!(picking.beneath(&strings), None);
    // **The four lengths in the words a person reads**, the folder picker's.
    let lengths: Vec<String> = HowLong::ALL
        .iter()
        .map(|it| {
            strings
                .say(&it.word().key(), &alo_strings::Filling::nothing())
                .into_text()
        })
        .collect();
    assert_eq!(read(&picking), lengths);
    assert!(
        lengths
            .iter()
            .all(|it| !it.is_empty() && !it.contains('\u{ab}'))
    );
}

/// **Saying how long hands back the application that was chosen**, sealed by
/// `alo-picking`, for the section to grant — and only then.
#[test]
fn saying_how_long_hands_back_the_chosen_application() {
    let mut picking = this_machine();
    picking.act(&ApplicationRow::This(CODIUM.to_owned()));
    let WhatChoosingDid::GiveItThisApplication { chosen, how_long } =
        picking.act(&ApplicationRow::For(HowLong::ADay))
    else {
        panic!("saying how long gave nothing");
    };
    assert_eq!(how_long, HowLong::ADay);
    assert_eq!(chosen.application().unwrap().identifier(), CODIUM);
}

/// **A row that is not in front of the person does nothing**: a length before
/// an application is chosen, an application that is not listed, the terminal
/// that was left out, and an application once the list has moved on.
#[test]
fn a_row_not_in_front_of_the_person_does_nothing() {
    let mut picking = this_machine();
    for row in [
        ApplicationRow::For(HowLong::AnHour),
        ApplicationRow::This("org.example.NotHere".to_owned()),
        ApplicationRow::This("app.devsuite.Ptyxis".to_owned()),
    ] {
        assert_eq!(picking.act(&row), WhatChoosingDid::Nothing, "{row:?}");
        assert!(!picking.is_saying_how_long(), "{row:?} moved the list on");
    }
    picking.act(&ApplicationRow::This(FIREFOX.to_owned()));
    assert_eq!(
        picking.act(&ApplicationRow::This(CODIUM.to_owned())),
        WhatChoosingDid::Nothing,
        "a second application was chosen over the first"
    );
    let WhatChoosingDid::GiveItThisApplication { chosen, .. } =
        picking.act(&ApplicationRow::For(HowLong::AnHour))
    else {
        panic!("saying how long gave nothing");
    };
    assert_eq!(chosen.application().unwrap().identifier(), FIREFOX);
}
