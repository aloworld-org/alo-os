//! **A reader is told which way each setting is set, and by name — on the page
//! that draws it.**
//!
//! EN 301 549 clause 5.6.1 — *a control that locks or toggles must say which
//! way it is set without being looked at* — and 11.4.1.2's *what state it is
//! in*. `crates/alo-conforming` pointed both at task 8 of
//! `docs/autonomy/access-and-language-plan.md`.
//!
//! # What changed on 2026-11-12, and why this file still has this name
//!
//! Until then [`Surface::Settings`] listed one switch per [`Setting`], named by
//! [`Setting::word`]. Eight of those nine are drawn on no Settings page, so a
//! reader was told of switches nobody could see or change, while the switches
//! that **are** drawn were never told at all. The owner chose *live*: a reader
//! hears each Settings page as it is drawn, and `alo-shell`'s
//! `settings_as_read` builds that from the page itself — which is where each
//! switch is now told which way it is set, with its value read from the same
//! file the page draws it from.
//!
//! So what this crate can still hold, and holds here, is the other side: that
//! the fixed description no longer claims a switch, so nothing published by it
//! can be a switch a person cannot reach; and that every setting keeps one name
//! a command and a reader can find it by (`alo_access::old_names`).

use alo_access::{Setting, State, Surface};

/// **The settings surface names no switch of its own.** What is on a page is
/// the page's; a switch listed here would be one the shell publishes whether
/// or not anything draws it.
#[test]
fn the_settings_surface_is_its_window_and_its_pages_and_no_switch() {
    for control in Surface::Settings.read_aloud() {
        assert_ne!(
            control.state,
            State::OnOrOff,
            "Settings lists a switch no page may draw"
        );
        assert!(control.setting.is_none(), "Settings names a setting");
    }
}

/// **No surface this crate describes has a switch**, so no value is looked up
/// for a control nobody draws. A switch arriving here should be the thing that
/// says so, rather than a reader announcing a state nobody supplied.
#[test]
fn no_surface_this_crate_describes_has_a_switch() {
    for surface in Surface::ALL {
        for control in surface.read_aloud() {
            assert_ne!(
                control.state,
                State::OnOrOff,
                "{surface:?} has a switch and this file assumed none did"
            );
            assert!(control.setting.is_none(), "{surface:?} names a setting");
        }
    }
}

/// **Every setting keeps a name of its own to be found by**: its word and its
/// key on Settings' node are each its own, so an old name never reaches the
/// wrong setting.
#[test]
fn every_setting_keeps_a_name_of_its_own() {
    let mut words: Vec<String> = Setting::ALL
        .into_iter()
        .map(|setting| setting.word().key().to_string())
        .collect();
    let mut keys: Vec<String> = Setting::ALL
        .into_iter()
        .map(alo_access::the_not_drawn_yet_attribute)
        .collect();
    for list in [&mut words, &mut keys] {
        list.sort_unstable();
        list.dedup();
        assert_eq!(list.len(), Setting::ALL.len(), "two settings share a name");
    }
}
