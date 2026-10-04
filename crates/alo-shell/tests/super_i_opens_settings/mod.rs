//! **`⊞`+I opens Settings**, which is the half of the sixteenth action that
//! happens rather than the half that is named.
//!
//! [ADR 0085](../../../../docs/decisions/0085-how-a-person-reaches-settings.md)
//! decided the chord; `alo_shortcuts` carries the action, its default binding
//! and its word; and `Server::dispatch_settings_command` is what makes pressing
//! it do something. What is held here is everything the arithmetic of a window
//! cannot see: that the chord reaches it, that every other chord still reaches
//! the layers below, and that opening Settings asks the machine for nothing it
//! might not have.
//!
//! The window's own behaviour — its sections, its rows, what a key inside it
//! does — is `settings_window_tests.rs`'s and is not retested here.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use std::path::Path;
use std::time::{Duration, SystemTime};

use super::support::Fixture;
use alo_shell::{SettingsPlaces, SettingsWindow, WhatTheChordDid};
use alo_shortcuts::{Action, Chord, Key, Modifier, Modifiers, Shortcuts};

/// A moment, so two openings can be told apart by nothing but their order.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// Where Settings looks on a machine where nothing has been kept yet.
///
/// Deliberately a machine with no files rather than a furnished one: what this
/// module tests is the road to the window, and a road that only works on a
/// machine somebody has already used is not a road a person arrives by.
fn nothing_kept_yet(dir: &Path) -> SettingsPlaces {
    SettingsPlaces::of(None, None, &dir.join("grants"), &dir.join("pairings"))
}

/// The chord `Shortcuts::shipped()` gives Settings.
fn the_settings_chord() -> Chord {
    Shortcuts::shipped()
        .chord_for(Action::Settings)
        .expect("the shipped bindings reach Settings")
}

/// A chord the shipped bindings give to nothing.
fn bound_to_nothing() -> Chord {
    Chord::checked(
        Modifiers::just(Modifier::Ctrl).and(Modifier::Alt),
        Key::Space,
    )
    .expect("a chord")
}

/// **The chord opens Settings**, and says so with what did not read while it
/// opened rather than with a bare yes.
#[test]
fn the_chord_a_person_presses_opens_settings() {
    let f = Fixture::keyboard();
    let (did, open) = f.backend(move |server| {
        let held = tempfile::tempdir().expect("a folder");
        let places = nothing_kept_yet(held.path());
        let mut window = SettingsWindow::closed();
        let did = server
            .dispatch_settings_command(
                &Shortcuts::shipped(),
                the_settings_chord(),
                &mut window,
                &places,
                noon(),
            )
            .expect("Settings refuses nothing");
        let said = match did {
            Some(WhatTheChordDid::SettingsOpened(opened)) => {
                format!("opened, {} not remembered", opened.not_remembered.len())
            }
            other => format!("{other:?}"),
        };
        (said, window.is_open())
    });
    assert!(open, "the window is open after the chord: {did}");
    assert!(did.starts_with("opened,"), "{did}");
}

/// **Pressing it again reads everything again**, and does not close what is
/// open. The chord means *show me my settings*; closing is a key inside the
/// window.
#[test]
fn pressing_it_twice_reads_again_rather_than_closing() {
    let f = Fixture::keyboard();
    let (both_opened, still_open) = f.backend(move |server| {
        let held = tempfile::tempdir().expect("a folder");
        let places = nothing_kept_yet(held.path());
        let mut window = SettingsWindow::closed();
        let shortcuts = Shortcuts::shipped();
        let mut opened = 0;
        for _ in 0..2 {
            let did = server
                .dispatch_settings_command(
                    &shortcuts,
                    the_settings_chord(),
                    &mut window,
                    &places,
                    noon(),
                )
                .expect("Settings refuses nothing");
            if matches!(did, Some(WhatTheChordDid::SettingsOpened(_))) {
                opened += 1;
            }
        }
        (opened, window.is_open())
    });
    assert_eq!(both_opened, 2, "both presses opened it");
    assert!(still_open);
}

/// **A chord bound to nothing opens nothing**, and touches no client.
#[test]
fn a_chord_bound_to_nothing_opens_nothing() {
    let f = Fixture::keyboard();
    let (answered, open) = f.backend(move |server| {
        let held = tempfile::tempdir().expect("a folder");
        let places = nothing_kept_yet(held.path());
        let mut window = SettingsWindow::closed();
        let did = server
            .dispatch_settings_command(
                &Shortcuts::shipped(),
                bound_to_nothing(),
                &mut window,
                &places,
                noon(),
            )
            .expect("an unbound chord is not a refusal");
        (did.is_some(), window.is_open())
    });
    assert!(!answered, "an unbound chord answers with nothing");
    assert!(!open, "and opens nothing");
}

/// **Every other chord still reaches the layers below**, and none of them
/// opens Settings.
///
/// The window action is asked of a machine with nothing open, so what comes
/// back may be a refusal — that is the layer below's business and this test
/// does not judge it. What it holds is that the chord went **there** rather
/// than here, and that Settings stayed closed either way.
#[test]
fn another_chord_goes_to_the_layer_below_and_leaves_settings_closed() {
    let f = Fixture::keyboard();
    let (was_settings, open) = f.backend(move |server| {
        let held = tempfile::tempdir().expect("a folder");
        let places = nothing_kept_yet(held.path());
        let mut window = SettingsWindow::closed();
        let shortcuts = Shortcuts::shipped();
        let chord = shortcuts
            .chord_for(Action::NextWindow)
            .expect("the shipped bindings reach the next window");
        let did = server.dispatch_settings_command(&shortcuts, chord, &mut window, &places, noon());
        let was_settings = matches!(did, Ok(Some(WhatTheChordDid::SettingsOpened(_))));
        (was_settings, window.is_open())
    });
    assert!(!was_settings, "the next-window chord is not Settings'");
    assert!(!open, "and it opened nothing");
}

/// **Settings opens on a machine with no keyboard and nothing in front.**
///
/// This is the difference between opening a window of this compositor's own and
/// acting on somebody else's: closing the window in front needs a seat and a
/// focused root, and refuses without them. A person reaching their own settings
/// needs neither, so a machine with no keyboard seat at all still opens them —
/// and the same fixture is used for both halves so the contrast is measured
/// rather than asserted.
#[test]
fn settings_opens_where_closing_a_window_would_refuse() {
    let f = Fixture::new();
    let (closing_refused, settings_open) = f.backend(move |server| {
        let held = tempfile::tempdir().expect("a folder");
        let places = nothing_kept_yet(held.path());
        let mut window = SettingsWindow::closed();
        let shortcuts = Shortcuts::shipped();
        let close = shortcuts
            .chord_for(Action::CloseWindow)
            .expect("the shipped bindings close a window");
        let closing_refused = server
            .dispatch_settings_command(&shortcuts, close, &mut window, &places, noon())
            .is_err();
        let opened = server
            .dispatch_settings_command(
                &shortcuts,
                the_settings_chord(),
                &mut window,
                &places,
                noon(),
            )
            .expect("Settings needs no seat");
        (
            closing_refused,
            matches!(opened, Some(WhatTheChordDid::SettingsOpened(_))) && window.is_open(),
        )
    });
    assert!(
        closing_refused,
        "closing the window in front refuses with no seat"
    );
    assert!(settings_open, "and Settings opens anyway");
}
