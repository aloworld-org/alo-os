//! **A reader is told which way each setting is set, and by name.**
//!
//! EN 301 549 clause 5.6.1 — *a control that locks or toggles must say which
//! way it is set without being looked at* — and 11.4.1.2's *what state it is
//! in*. `crates/alo-conforming` pointed both at task 8 of
//! `docs/autonomy/access-and-language-plan.md`.
//!
//! # What this is for, which is narrower than it sounds
//!
//! The settings surface read as **one switch called "a setting"** until
//! 2026-09-30 — a placeholder standing for all nine. A value plumbed to that
//! control would have had a reader announce *"a setting, on"*: true, and
//! useless, because a person told the state of a switch whose identity they
//! were never told has been told nothing.
//!
//! So the first thing held here is that **every setting is its own control,
//! named by the setting itself**. The names were never the missing part —
//! `Setting::word` has had them all along — only the tree was not using them.
//!
//! # A tenth setting cannot be forgotten
//!
//! The controls come from [`Setting::ALL`], so a setting added to that closed
//! list arrives in the tree already named. This file asserts the two are the
//! same set rather than the same number: a count would pass while one setting
//! was silently swapped for another.

use alo_access::{Setting, State, Surface};

/// Every control the settings surface reads aloud.
fn the_settings_surface() -> Vec<alo_access::Control> {
    Surface::Settings.read_aloud()
}

/// **Every setting is a control of its own, and the set is exactly
/// `Setting::ALL`.**
///
/// Asserted as a set rather than a count, because a count is satisfied by nine
/// of the wrong ones.
#[test]
fn every_setting_is_its_own_control_named_by_itself() {
    let read = the_settings_surface();
    let switches: Vec<Setting> = read.iter().filter_map(|control| control.setting).collect();

    for setting in Setting::ALL {
        assert!(
            switches.contains(&setting),
            "{setting:?} is a setting a person can turn on and no control names it, \
             so a reader cannot tell them it exists"
        );
    }
    assert_eq!(
        switches.len(),
        Setting::ALL.len(),
        "the settings surface reads {} switches and there are {} settings",
        switches.len(),
        Setting::ALL.len()
    );
}

/// **Each switch is named by its own setting**, not by a word chosen here.
///
/// The name comes from `Setting::word`, so there is no second list to drift.
#[test]
fn each_switch_carries_the_name_its_setting_gives() {
    for control in the_settings_surface() {
        let Some(setting) = control.setting else {
            continue;
        };
        assert_eq!(
            control.name,
            setting.word(),
            "{setting:?}'s control is named by something other than the setting"
        );
    }
}

/// **A switch is on or off and says so**, which is what `State::OnOrOff` is
/// for. A setting's control carrying any other state would be a switch a
/// reader announces as something it cannot toggle.
#[test]
fn every_switch_is_on_or_off() {
    for control in the_settings_surface() {
        if let Some(setting) = control.setting {
            assert_eq!(
                control.state,
                State::OnOrOff,
                "{setting:?}'s control is not a switch"
            );
        }
    }
}

/// **Nothing that is not a setting claims to be one.** The window and the list
/// on that surface have no value to be told, and a control carrying a setting
/// it is not would send a reader the wrong switch's state.
#[test]
fn only_switches_carry_a_setting() {
    for control in the_settings_surface() {
        if control.setting.is_some() {
            assert_eq!(control.state, State::OnOrOff);
        } else {
            assert_ne!(
                control.state,
                State::OnOrOff,
                "a control is on-or-off and names no setting, so its value cannot be looked up"
            );
        }
    }
}

/// **No other surface has a switch**, so nothing else needs a value looked up.
///
/// Held because the day another surface grows one, this file should be the
/// thing that says so rather than a reader announcing a state nobody supplied.
#[test]
fn only_the_settings_surface_has_switches() {
    for surface in Surface::ALL {
        if surface == Surface::Settings {
            continue;
        }
        for control in surface.read_aloud() {
            assert_ne!(
                control.state,
                State::OnOrOff,
                "{surface:?} has a switch and this file assumed only the settings surface did"
            );
            assert!(control.setting.is_none(), "{surface:?} names a setting");
        }
    }
}
