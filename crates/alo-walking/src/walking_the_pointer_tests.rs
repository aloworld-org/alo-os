//! The pointer checks carried out on a screen whose pointer is drawn by the real
//! artwork: they pass where alo's pointer is what the person chose, and fail —
//! saying what they saw — where it is missing, another shape, another colour or
//! another size.

use std::time::Duration;

use alo_appearance::{PointerColour, PointerShape, PointerSize};
use alo_cursors::CursorIcon;

use crate::a_place_on_the_screen::Target;
use crate::a_screen_with_a_pointer::{AScreenWithAPointer, Patch};
use crate::keys::Chord;
use crate::report::{Check, Report, Verdict};
use crate::step::{Pointing, Walk};
use crate::strokes::Pace;
use crate::the_pointer_seen::Look;
use crate::walking::walk;

/// No waiting between keys.
const QUICK: Pace = Pace {
    hold: Duration::ZERO,
    between: Duration::ZERO,
};

/// Magenta, asking for a text cursor, as the walk's page draws its text field.
const MAGENTA: [u8; 3] = [255, 0, 255];

/// Lime, asking for a hand, as the walk's page draws its link.
const LIME: [u8; 3] = [0, 255, 0];

/// A grey screen with a text field and a link, away from the middle.
fn a_page() -> AScreenWithAPointer {
    AScreenWithAPointer::grey()
        .with(Patch {
            area: (20, 20, 200, 120),
            rgb: MAGENTA,
            icon: CursorIcon::Text,
        })
        .with(Patch {
            area: (420, 260, 200, 120),
            rgb: LIME,
            icon: CursorIcon::Pointer,
        })
}

/// No secret is set.
fn nothing_set(_: &str) -> Option<String> {
    None
}

/// A pointing step.
fn point_at(check: &str, at: Target) -> Pointing {
    Pointing::PointAt {
        check: check.to_owned(),
        at,
    }
}

/// A shape check.
fn is_drawn_as(check: &str, icon: Option<CursorIcon>) -> Pointing {
    Pointing::IsDrawnAs {
        check: check.to_owned(),
        icon,
    }
}

/// The checks, by name and verdict.
fn verdicts(report: &Report) -> Vec<(String, Verdict)> {
    report
        .checks()
        .into_iter()
        .map(|it| (it.name.clone(), it.verdict))
        .collect()
}

/// The check named, or nothing.
fn the_check<'a>(report: &'a Report, name: &str) -> Option<&'a Check> {
    report.checks().into_iter().find(|it| it.name == name)
}

/// **The whole road passes where the pointer is alo's**: seen at all, moved to
/// the middle, the kite there, the text cursor over the field and the hand
/// over the link — each found again after a nudge.
#[test]
fn every_check_passes_where_the_pointer_is_alos() {
    let mut machine = a_page();
    let the_walk = Walk::named("pointer")
        .pointer(Pointing::SeenAtAll {
            check: "seen".to_owned(),
        })
        .pointer(point_at("to the middle", Target::Middle))
        .pointer(is_drawn_as("the kite", Some(CursorIcon::Default)))
        .pointer(point_at("to the field", Target::Colour(MAGENTA)))
        .pointer(is_drawn_as("the text cursor", Some(CursorIcon::Text)))
        .pointer(point_at("to the link", Target::Colour(LIME)))
        .pointer(is_drawn_as("the hand", Some(CursorIcon::Pointer)))
        .pointer(is_drawn_as("any of alo's", None));
    let report = walk(&mut machine, &the_walk, &nothing_set, QUICK);
    for (name, verdict) in verdicts(&report) {
        assert_eq!(verdict, Verdict::Passed, "{name}: {}", report.as_text());
    }
    assert_eq!(report.checks().len(), 8);
    assert!(
        the_check(&report, "the hand").is_some_and(|it| it.evidence.contains("\"pointer\"")),
        "{}",
        report.as_text()
    );
}

/// **The refusal the others stand on: a capture without the pointer fails the
/// first check, and nothing after it is reached** — so a hardware cursor plane
/// the capture cannot see is never read as a pass.
#[test]
fn a_capture_without_the_pointer_fails_and_stops() {
    let mut machine = AScreenWithAPointer {
        draws_the_pointer: false,
        ..a_page()
    };
    let the_walk = Walk::named("pointer")
        .pointer(Pointing::SeenAtAll {
            check: "seen".to_owned(),
        })
        .pointer(point_at("to the middle", Target::Middle))
        .pointer(is_drawn_as("the kite", Some(CursorIcon::Default)));
    let report = walk(&mut machine, &the_walk, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("seen".to_owned(), Verdict::Failed),
            ("to the middle".to_owned(), Verdict::NotReached),
            ("the kite".to_owned(), Verdict::NotReached),
        ]
    );
    assert!(machine.kept.iter().any(|it| it == "pointer-seen"));
}

/// **The familiar arrow where the kite is expected fails, and says so** — and
/// the walk goes on, because looking changed nothing on the screen.
#[test]
fn the_wrong_arrow_fails_saying_which_and_the_walk_goes_on() {
    let mut machine = AScreenWithAPointer {
        look: Look {
            shape: PointerShape::Classic,
            ..Look::SHIPPED
        },
        ..AScreenWithAPointer::grey()
    };
    let the_walk = Walk::named("pointer")
        .pointer(is_drawn_as("the kite", Some(CursorIcon::Default)))
        .pointer(Pointing::ThePersonChose(Look {
            shape: PointerShape::Classic,
            ..Look::SHIPPED
        }))
        .pointer(is_drawn_as("the familiar arrow", Some(CursorIcon::Default)));
    let report = walk(&mut machine, &the_walk, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("the kite".to_owned(), Verdict::Failed),
            ("the familiar arrow".to_owned(), Verdict::Passed),
        ],
        "{}",
        report.as_text()
    );
    assert!(
        the_check(&report, "the kite").is_some_and(|it| it.evidence.contains("familiar arrow")),
        "{}",
        report.as_text()
    );
}

/// **A size and a colour the person chose are held to**: the large white kite
/// passes where it is drawn, and the navy one at the default size fails where
/// the large white one was chosen, naming what it found.
#[test]
fn the_size_and_colour_chosen_are_what_is_checked() {
    let chosen = Look {
        colour: PointerColour::White,
        size: PointerSize::Large,
        ..Look::SHIPPED
    };
    let checks = Walk::named("pointer")
        .pointer(Pointing::ThePersonChose(chosen))
        .pointer(is_drawn_as("large and white", Some(CursorIcon::Default)));

    let mut right = AScreenWithAPointer {
        look: chosen,
        ..AScreenWithAPointer::grey()
    };
    let report = walk(&mut right, &checks, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![("large and white".to_owned(), Verdict::Passed)],
        "{}",
        report.as_text()
    );

    let mut unchanged = AScreenWithAPointer::grey();
    let report = walk(&mut unchanged, &checks, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![("large and white".to_owned(), Verdict::Failed)]
    );
    assert!(
        the_check(&report, "large and white").is_some_and(|it| it
            .evidence
            .contains("the navy kite, default size, 24 pixels")),
        "{}",
        report.as_text()
    );
}

/// **A place that is not on the screen fails and stops**, rather than checking
/// the pointer wherever it happens to be.
#[test]
fn a_place_not_on_the_screen_fails_and_stops() {
    let mut machine = AScreenWithAPointer::grey();
    let the_walk = Walk::named("pointer")
        .pointer(point_at("to the field", Target::Colour(MAGENTA)))
        .pointer(is_drawn_as("the text cursor", Some(CursorIcon::Text)));
    let report = walk(&mut machine, &the_walk, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("to the field".to_owned(), Verdict::Failed),
            ("the text cursor".to_owned(), Verdict::NotReached),
        ]
    );
}

/// **What changed is a window that opened**: the screen is kept, a key opens
/// a window, and the pointer is steered onto it — and with nothing changed
/// there is nowhere to go.
#[test]
fn the_pointer_goes_onto_what_changed_and_nowhere_without_a_change() {
    let window = Patch {
        area: (300, 40, 240, 200),
        rgb: [250, 250, 250],
        icon: CursorIcon::Text,
    };
    let mut machine = AScreenWithAPointer {
        on_a_key: vec![window],
        ..AScreenWithAPointer::grey()
    };
    let the_walk = Walk::named("pointer")
        .pointer(Pointing::Remember)
        .press(Chord::key("ret"))
        .pointer(point_at("onto the window", Target::WhatChanged))
        .pointer(is_drawn_as("its text cursor", Some(CursorIcon::Text)));
    let report = walk(&mut machine, &the_walk, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("onto the window".to_owned(), Verdict::Passed),
            ("its text cursor".to_owned(), Verdict::Passed),
        ],
        "{}",
        report.as_text()
    );

    let mut nothing_opens = AScreenWithAPointer::grey();
    let report = walk(&mut nothing_opens, &the_walk, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("onto the window".to_owned(), Verdict::Failed),
            ("its text cursor".to_owned(), Verdict::NotReached),
        ]
    );
}

/// **A click is the left button down, then up**, where the pointer is, and
/// moves nothing.
#[test]
fn a_click_is_the_button_down_then_up() {
    let mut machine = AScreenWithAPointer::grey();
    let the_walk = Walk::named("pointer").pointer(Pointing::Click);
    let report = walk(&mut machine, &the_walk, &nothing_set, QUICK);
    assert!(report.checks().is_empty());
    assert_eq!(machine.buttons, vec![true, false]);
    assert!(machine.pointed.is_empty());
}
