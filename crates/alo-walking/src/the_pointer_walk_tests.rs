//! What the pointer walk will do, read without a machine.

use std::collections::HashSet;

use alo_appearance::{PointerColour, PointerShape, PointerSize};
use alo_cursors::CursorIcon;

use super::{LARGE_AND_WHITE, rows, the_page};
use crate::keys;
use crate::step::{Pointing, Step};
use crate::the_first_walk::the_first_walk;

/// **It is part of the walk every run takes**, before the section that needs
/// a provider's key: a run without the secret still checks the pointer.
#[test]
fn it_runs_before_the_provider_gate() {
    let steps = the_first_walk().steps().to_vec();
    let first = steps.iter().position(|it| matches!(it, Step::Pointer(_)));
    let gate = steps.iter().position(|it| matches!(it, Step::OnlyWith(_)));
    assert!(
        matches!((first, gate), (Some(first), Some(gate)) if first < gate),
        "pointer {first:?}, gate {gate:?}"
    );
}

/// **The capture is measured before anything is asked of it**: the first
/// pointer check is the one that says the pointer is in the picture at all.
#[test]
fn the_first_check_is_that_the_capture_shows_the_pointer() {
    let first = the_first_walk().steps().iter().find_map(|it| match it {
        Step::Pointer(pointing) if pointing.check().is_some() => Some(pointing.clone()),
        _ => None,
    });
    assert!(
        matches!(first, Some(Pointing::SeenAtAll { .. })),
        "the first pointer check is {first:?}"
    );
}

/// Every pointer check has its own name, so none overwrites another's picture.
#[test]
fn every_pointer_check_has_its_own_name() {
    let mut seen = HashSet::new();
    for step in the_first_walk().steps() {
        if let Step::Pointer(pointing) = step
            && let Some(check) = pointing.check()
        {
            assert!(seen.insert(check.to_owned()), "{check} twice");
        }
    }
    assert!(seen.len() >= 15, "only {} pointer checks", seen.len());
}

/// **Firefox is opened after the person chose the large white pointer**, so
/// the check over it is of an application told the choice when it started.
#[test]
fn firefox_opens_after_the_choice() {
    let steps = the_first_walk().steps().to_vec();
    let chose = steps
        .iter()
        .rposition(|it| *it == Step::Pointer(Pointing::ThePersonChose(LARGE_AND_WHITE)));
    let firefox = steps
        .iter()
        .position(|it| *it == Step::Type("Firefox".to_owned()));
    assert!(
        matches!((chose, firefox), (Some(chose), Some(firefox)) if chose < firefox),
        "chose {chose:?}, Firefox {firefox:?}"
    );
}

/// **Each of the three shapes a person would notice is checked by name**: the
/// arrow, the text cursor and the hand.
#[test]
fn the_arrow_the_text_cursor_and_the_hand_are_each_checked() {
    let named: HashSet<CursorIcon> = the_first_walk()
        .steps()
        .iter()
        .filter_map(|it| match it {
            Step::Pointer(Pointing::IsDrawnAs { icon, .. }) => *icon,
            _ => None,
        })
        .collect();
    for icon in [CursorIcon::Default, CursorIcon::Text, CursorIcon::Pointer] {
        assert!(named.contains(&icon), "{icon:?} is never checked");
    }
}

/// **The page can be typed**, and names no character an address reads as
/// something else.
#[test]
fn the_page_can_be_typed_and_is_only_an_address() {
    let page = the_page();
    assert!(keys::typing(&page).is_ok());
    assert!(page.starts_with("data:text/html,"));
    assert!(!page.contains('#') && !page.contains('%'));
    assert!(!page.contains("  "), "a run of spaces in {page:?}");
    for colour in ["rgb(255,0,255)", "rgb(0,255,255)", "rgb(0,255,0)"] {
        assert!(page.contains(colour), "{colour} is not on the page");
    }
}

/// **The rows are where Settings draws them**, counted from the same lists:
/// the screen reader and the text size first, then the three sizes, the two
/// arrows and the three colours (`alo_shell::settings_window`, read on
/// 2026-11-14).
#[test]
fn the_rows_are_counted_from_the_appearance_lists() {
    assert_eq!(rows::size(PointerSize::Default), 2);
    assert_eq!(rows::size(PointerSize::Large), 3);
    assert_eq!(rows::shape(PointerShape::Kite), 5);
    assert_eq!(rows::shape(PointerShape::Classic), 6);
    assert_eq!(rows::colour(PointerColour::Navy), 7);
    assert_eq!(rows::colour(PointerColour::White), 8);
}

/// **The address is typed only once Firefox has drawn its address bar holding
/// the keyboard.** Run 43801984900 pressed Ctrl+L and typed at once; the first
/// key went elsewhere, the bar read `ata:text/html,…`, and Firefox asked to
/// open *the ata link with System Handler*. So the step after Ctrl+L is a look
/// — which fails if the screen does not change — and the address is the step
/// after that look.
#[test]
fn the_address_is_typed_after_the_address_bar_is_seen_to_take_the_keyboard() {
    let steps = the_first_walk().steps().to_vec();
    let address = steps.iter().position(|it| *it == Step::Type(the_page()));
    assert!(address.is_some(), "the walk never types the page");
    let address = address.unwrap_or_default();
    let before: Vec<&Step> = steps.iter().take(address).rev().take(2).collect();
    assert!(
        matches!(
            before.as_slice(),
            [Step::Look { .. }, Step::Press(chord)] if *chord == keys::Chord::with("ctrl", "l")
        ),
        "the two steps before the address are {before:?}"
    );
}
