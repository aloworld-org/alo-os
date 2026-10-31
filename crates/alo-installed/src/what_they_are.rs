//! **What each application is**, in the words its entry gives for that.
//!
//! `docs/features.md`: *the launcher finds an application by what it is, as
//! well as by its name* — the owner's answer of 2026-10-31, "Yes, as other
//! operating systems." Every desktop reads the same three keys for it:
//!
//! - `GenericName` — *File Manager*, *Web Browser*, *Spreadsheet*;
//! - `Keywords` — the entry author's own search words, `;`-separated;
//! - `Categories` — the freedesktop menu registry's names, `FileManager`,
//!   `WebBrowser`, `Spreadsheet`.
//!
//! **In the person's language, and in the entry's own words too.** The first
//! two are localised — `Keywords[fr]=fichiers;…` — and the value read is the
//! best match for each language the person reads, by the specification's
//! `lang_COUNTRY@MODIFIER`, `lang_COUNTRY`, `lang@MODIFIER`, `lang` order
//! ([`the_keys_a_language_reads`]). The unlocalised value is kept beside it,
//! so a person reading French who types *files* still finds Dolphin; finding
//! more than the person's language would offer costs them nothing, finding
//! less would. `Categories` are registry names and are never translated.
//!
//! **Which entries count is not decided here.** Only an entry the walk in
//! `lib.rs` lists is read — so an entry saying `NoDisplay=true`,
//! `Hidden=true` or a `Type` other than `Application` is never offered by what
//! it is any more than by its name, and the two answers cannot disagree about
//! which applications exist.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// The words each application's entry gives for what it is, by identifier.
pub type WhatEachIs = BTreeMap<String, Vec<String>>;

/// What every application these places list says it is, earlier places
/// winning, read for these languages (BCP 47 tags, best first).
///
/// An application whose entry says nothing about what it is is absent, which
/// is ordinary: it is still found by its name.
#[must_use]
pub fn what_they_are(places: &[PathBuf], languages: &[&str]) -> WhatEachIs {
    let keys: Vec<String> = languages
        .iter()
        .flat_map(|tag| the_keys_a_language_reads(tag))
        .collect();
    let mut each = WhatEachIs::new();
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
            let words = the_words_in(&text, &keys);
            if !words.is_empty() {
                each.insert(entry.identifier, words);
            }
        }
    }
    each
}

/// The locale suffixes a desktop entry might carry for this language, most
/// specific first: `pt-BR` reads `pt_BR` then `pt`; `sr-Latn-RS` reads
/// `sr_RS@latin`, `sr_RS`, `sr@latin`, `sr`.
///
/// Empty for a tag that names no language.
#[must_use]
pub fn the_keys_a_language_reads(tag: &str) -> Vec<String> {
    let mut parts = tag.split(['-', '_']).filter(|it| !it.is_empty());
    let Some(language) = parts
        .next()
        .filter(|it| it.chars().all(|letter| letter.is_ascii_alphabetic()))
        .map(str::to_ascii_lowercase)
    else {
        return Vec::new();
    };
    let mut region = None;
    let mut modifier = None;
    for part in parts {
        let letters = part.chars().all(|letter| letter.is_ascii_alphabetic());
        match part.len() {
            2 if letters => region = Some(part.to_ascii_uppercase()),
            3 if part.chars().all(|letter| letter.is_ascii_digit()) => {
                region = Some(part.to_owned());
            }
            4 if letters => {
                modifier = match part.to_ascii_lowercase().as_str() {
                    "latn" => Some("latin"),
                    "cyrl" => Some("cyrillic"),
                    _ => None,
                };
            }
            _ => {}
        }
    }
    let mut keys = Vec::new();
    if let (Some(region), Some(modifier)) = (&region, modifier) {
        keys.push(format!("{language}_{region}@{modifier}"));
    }
    if let Some(region) = &region {
        keys.push(format!("{language}_{region}"));
    }
    if let Some(modifier) = modifier {
        keys.push(format!("{language}@{modifier}"));
    }
    keys.push(language);
    keys
}

/// The words one entry gives for what it is, for these locale keys.
fn the_words_in(text: &str, keys: &[String]) -> Vec<String> {
    let said = the_keys_said(text);
    let mut words: Vec<String> = Vec::new();
    for key in ["GenericName", "Keywords"] {
        let best = keys
            .iter()
            .find_map(|locale| said.get(&format!("{key}[{locale}]")));
        for value in best.into_iter().chain(said.get(key)) {
            words.extend(the_items_in(key, value));
        }
    }
    if let Some(value) = said.get("Categories") {
        words.extend(the_items_in("Categories", value));
    }
    let mut kept = BTreeSet::new();
    words.retain(|word| kept.insert(word.clone()));
    words
}

/// One key's value as the separate things it names.
///
/// `Keywords` and `Categories` are lists; `GenericName` is one phrase.
fn the_items_in(key: &str, value: &str) -> Vec<String> {
    if key == "GenericName" {
        return vec![value.to_owned()];
    }
    value
        .split(';')
        .map(str::trim)
        .filter(|it| !it.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Every key the `[Desktop Entry]` group says that this reader wants, as
/// written.
fn the_keys_said(text: &str) -> BTreeMap<String, String> {
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
        let (key, value) = (key.trim(), value.trim());
        let wanted = ["GenericName", "Keywords", "Categories"]
            .iter()
            .any(|it| key == *it || key.strip_prefix(it).is_some_and(|rest| rest.starts_with('[')));
        if wanted && !value.is_empty() {
            said.insert(key.to_owned(), value.to_owned());
        }
    }
    said
}

#[cfg(test)]
#[path = "what_they_are_tests.rs"]
mod tests;
