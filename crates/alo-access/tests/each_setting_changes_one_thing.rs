//! **Each setting changes one named thing, and every one of them can be turned
//! on before anybody has an account.**
//!
//! Task 1 of `docs/autonomy/access-and-language-plan.md`. Two promises are
//! easy to write in a document and hard to keep, so they are held here instead:
//! that a setting turned on produces a value another crate reads — not a
//! sentence somebody has to implement — and that a person who needs one of these
//! to set the machine up can turn it on at the sign-in screen, where there is no
//! account yet to keep it under.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::Duration;

use alo_access::keeping::{at_sign_in, keep_for_this_machine};
use alo_access::{Magnification, Setting, TurnedOn, WhatItChanges};
use alo_appearance::Scheme;

/// A folder of this test's own, named by a counter rather than by the clock:
/// two folders named in the same nanosecond are one folder, and a test that
/// shares a folder with another test is a test that fails for no reason.
fn a_folder() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let path = std::env::temp_dir().join(format!(
        "alo-access-test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

/// **Every setting, turned on alone, changes something named** — and the thing
/// it changes is the one this setting is about.
#[test]
fn every_setting_turned_on_alone_changes_the_value_it_is_about() {
    for setting in Setting::ALL {
        let mut turned_on = TurnedOn::nothing();
        turned_on.turn_on(setting);
        let changed = setting
            .what_it_changes(&turned_on, Scheme::Light)
            .unwrap_or_else(|| panic!("{setting:?} changes nothing"));

        match (setting, &changed) {
            (Setting::ScreenReader, WhatItChanges::TheScreenIsRead)
            | (Setting::ReducedMotion, WhatItChanges::NothingMovesThatNeedNot)
            | (Setting::FocusAlwaysVisible, WhatItChanges::WhereTheKeyboardIsIsAlwaysDrawn) => {}
            (Setting::HighContrast, WhatItChanges::ThePalette(palette)) => {
                assert_eq!(*palette, alo_access::HighContrast::of(Scheme::Light));
                assert_ne!(
                    palette.ground, palette.ink,
                    "a palette whose ground and ink are one colour"
                );
            }
            (Setting::LargerText, WhatItChanges::TheTextScale(scale)) => {
                assert!(
                    *scale > alo_appearance::TextScale::ordinary(),
                    "larger text that is not larger"
                );
            }
            (Setting::Magnifier, WhatItChanges::WhatIsUnderThePointerIsMagnified(by)) => {
                assert!(by.tenths() > 10, "a magnifier that magnifies nothing");
            }
            (Setting::StickyKeys, WhatItChanges::TheKeyFilter(filter)) => {
                assert!(filter.sticky);
                assert_eq!(filter.held_before_it_counts, None);
                assert_eq!(filter.ignored_after_the_same_key, None);
            }
            (Setting::SlowKeys, WhatItChanges::TheKeyFilter(filter)) => {
                assert!(filter.held_before_it_counts.is_some());
                assert!(!filter.sticky);
                assert_eq!(filter.ignored_after_the_same_key, None);
            }
            (Setting::BounceKeys, WhatItChanges::TheKeyFilter(filter)) => {
                assert!(filter.ignored_after_the_same_key.is_some());
                assert!(!filter.sticky);
                assert_eq!(filter.held_before_it_counts, None);
            }
            (setting, changed) => panic!("{setting:?} changes {changed:?}, which is not its own"),
        }

        // And nothing else moved: every other setting is still off.
        for other in Setting::ALL {
            if other != setting {
                assert!(
                    !turned_on.has(other),
                    "turning on {setting:?} turned on {other:?}"
                );
            }
        }
    }
}

/// **There is no accessibility mode.** Turning everything on is nine settings
/// on, each still its own value — not a tenth thing that replaces the machine.
#[test]
fn everything_on_is_nine_settings_on_and_not_a_mode() {
    let mut turned_on = TurnedOn::nothing();
    for setting in Setting::ALL {
        turned_on.turn_on(setting);
    }
    assert_eq!(turned_on.all_on().count(), Setting::ALL.len());
    let filter = turned_on.key_filter();
    assert!(filter.sticky);
    assert!(filter.held_before_it_counts.is_some());
    assert!(filter.ignored_after_the_same_key.is_some());
    for setting in Setting::ALL {
        assert!(
            setting.what_it_changes(&turned_on, Scheme::Dark).is_some(),
            "{setting:?}"
        );
    }
}

/// **Every setting can be turned on before anybody has an account** — the whole
/// list, not the two somebody thought of at the sign-in screen.
#[test]
fn every_setting_can_be_turned_on_where_there_is_no_account_yet() {
    let machine = a_folder();
    for setting in Setting::ALL {
        let mut before_anybody_signs_in = TurnedOn::nothing();
        before_anybody_signs_in.turn_on(setting);
        keep_for_this_machine(&machine, &before_anybody_signs_in).unwrap();

        let (at_the_screen, why) = at_sign_in(&machine);
        assert!(why.is_none(), "{setting:?}: {why:?}");
        assert!(
            at_the_screen.has(setting),
            "{setting:?} cannot be turned on before there is an account"
        );
        assert!(
            setting
                .what_it_changes(&at_the_screen, Scheme::Light)
                .is_some(),
            "{setting:?} is on at sign-in and changes nothing"
        );
    }
}

/// **The values a person chose are kept with the settings**, so the machine
/// does not ask twice and does not quietly forget.
#[test]
fn the_values_a_person_chose_survive_being_written_down() {
    let folder = a_folder();
    let mut turned_on = TurnedOn::nothing();
    turned_on.turn_on(Setting::Magnifier);
    turned_on.magnify_by(Magnification::of_tenths(35).unwrap());
    turned_on.turn_on(Setting::SlowKeys);
    turned_on.hold_keys_for(Duration::from_millis(450));
    alo_access::keeping::keep(&folder, &turned_on).unwrap();

    let read_back = alo_access::keeping::read(&folder).unwrap();
    assert_eq!(read_back.magnification().tenths(), 35);
    assert_eq!(
        read_back.key_filter().held_before_it_counts,
        Some(Duration::from_millis(450))
    );
}

/// **The bounce-keys delay survives too**, which this file promised and did not
/// check.
///
/// *Added 2026-10-20.* `TurnedOn::ignore_the_same_key_for` — the only way to
/// change how long the same key is ignored for — was called by **nothing in the
/// workspace**: no caller, no test, not once. Found by counting, to a fixed
/// point, the live references to every `pub fn` in every crate.
///
/// # Why it is a gap rather than a tidy-up
///
/// The three key settings are a set. `Setting::every()` offers sticky keys,
/// slow keys **and** bounce keys; each has a translated word; `filtering.rs`
/// genuinely honours `ignored_after_the_same_key`, defaulting to `A_MOMENT`
/// where nothing chose. So a person can turn bounce keys on, and the one value
/// it has was reachable by no test — while its twin `hold_keys_for` is asserted
/// to round-trip in the test above.
///
/// **The test above is named *the values a person chose survive being written
/// down*, and it covered two of the three.** That is the shape worth catching:
/// not a missing feature, but a promise whose name is plural and whose
/// assertions are not.
///
/// # What is deliberately not here
///
/// No road that lets a person *choose* the delay. `docs/features.md` offers
/// *sticky keys, slow keys, and keyboard-only operation of everything* and
/// names no duration, so building one would **add** a promise rather than keep
/// one — and never offering is not taking away. This holds what exists.
#[test]
fn the_bounce_keys_delay_survives_being_written_down_as_well() {
    let folder = a_folder();
    let mut turned_on = TurnedOn::nothing();
    turned_on.turn_on(Setting::BounceKeys);
    turned_on.ignore_the_same_key_for(Duration::from_millis(275));
    alo_access::keeping::keep(&folder, &turned_on).unwrap();

    let read_back = alo_access::keeping::read(&folder).unwrap();
    assert_eq!(
        read_back.key_filter().ignored_after_the_same_key,
        Some(Duration::from_millis(275)),
        "the value a person chose for bounce keys did not survive"
    );
    // A control: 275 is not the default, so a filter that ignored the stored
    // value and answered `A_MOMENT` would fail the assertion above rather than
    // pass it by coincidence.
    assert_ne!(
        read_back.key_filter().ignored_after_the_same_key,
        TurnedOn::nothing().key_filter().ignored_after_the_same_key,
        "275ms must differ from what a machine nobody changed would answer"
    );
}

/// **And the two delays are kept apart**, so neither setting reads the other's
/// value.
///
/// One struct holds both durations and `key_filter` builds both from it, so a
/// field crossed in that constructor would give a person their slow-keys delay
/// as their bounce-keys delay. Two different numbers, asserted both ways: the
/// test cannot pass if they are swapped, and cannot pass if either is dropped.
#[test]
fn the_slow_and_bounce_delays_do_not_read_each_others_value() {
    let mut turned_on = TurnedOn::nothing();
    turned_on.turn_on(Setting::SlowKeys);
    turned_on.turn_on(Setting::BounceKeys);
    turned_on.hold_keys_for(Duration::from_millis(450));
    turned_on.ignore_the_same_key_for(Duration::from_millis(275));

    let filter = turned_on.key_filter();
    assert_eq!(
        filter.held_before_it_counts,
        Some(Duration::from_millis(450)),
        "slow keys took the wrong value"
    );
    assert_eq!(
        filter.ignored_after_the_same_key,
        Some(Duration::from_millis(275)),
        "bounce keys took the wrong value"
    );
}
