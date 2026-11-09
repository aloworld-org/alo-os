//! **What each application is called, in the person's language.**
//!
//! `docs/design/the-window-controls.md` §3: a window's band shows its title and
//! then **the application's name** — *The next chapter · Notes*. The name a
//! person reads there is the desktop entry's `Name`, and an entry carries it
//! translated — `Name[fr]=Éditeur de texte` — so a person reading French is
//! shown the French name, as every other desktop shows it.
//!
//! The walk in `lib.rs` reads only the unlocalised `Name` and says why: which
//! language the person reads is not that walk's to know. **It is handed in
//! here**, as [`crate::what_they_are`] is handed it, and read by the same
//! `lang_COUNTRY@MODIFIER`, `lang_COUNTRY`, `lang@MODIFIER`, `lang` order
//! ([`crate::the_keys_a_language_reads`]).
//!
//! **Only an entry the walk lists is read**, so an entry asking not to be
//! listed has no name here either, and the two readings cannot disagree about
//! which applications exist. **An entry with no translation for the person's
//! languages is absent**: its unlocalised `Name` is the walk's answer already,
//! and a second copy of it here would be two places to keep in step.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Each application's translated name, by identifier.
pub type WhatEachIsCalled = BTreeMap<String, String>;

/// The translated `Name` of every application these places list, earlier
/// places winning, for these languages (BCP 47 tags, best first).
///
/// An application whose entry has no `Name` for any of these languages is
/// absent, and is named by its unlocalised `Name`.
#[must_use]
pub fn what_they_are_called(places: &[PathBuf], languages: &[&str]) -> WhatEachIsCalled {
    let keys: Vec<String> = languages
        .iter()
        .flat_map(|tag| crate::the_keys_a_language_reads(tag))
        .collect();
    let mut each = WhatEachIsCalled::new();
    if keys.is_empty() {
        return each;
    }
    let mut seen = BTreeSet::new();
    for place in places {
        for entry in crate::entries_under(place) {
            // **The first entry for an identifier is the one**, the walk's own
            // rule: a person's copy wins even where it says less.
            if !seen.insert(entry.identifier.clone()) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&entry.at) else {
                continue;
            };
            if let Some(called) = the_name_in(&text, &keys) {
                each.insert(entry.identifier, called);
            }
        }
    }
    each
}

/// The best translated `Name` one entry gives for these locale keys.
///
/// Reads only the `[Desktop Entry]` group: an action's group has its own
/// `Name`, and naming an application after its right-click action is the
/// fault the walk's own reader names.
fn the_name_in(text: &str, keys: &[String]) -> Option<String> {
    let mut said = BTreeMap::new();
    let mut inside = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            if inside {
                break;
            }
            inside = line == "[Desktop Entry]";
            continue;
        }
        if !inside {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let Some(locale) = key
            .trim()
            .strip_prefix("Name[")
            .and_then(|rest| rest.strip_suffix(']'))
        else {
            continue;
        };
        let value = value.trim();
        if !value.is_empty() {
            said.insert(locale.to_owned(), value.to_owned());
        }
    }
    keys.iter().find_map(|locale| said.remove(locale))
}

#[cfg(test)]
#[path = "what_they_are_called_tests.rs"]
mod tests;
