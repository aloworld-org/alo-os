//! What each application is, read from entries shaped like the real ones.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::path::PathBuf;

use super::*;

/// A folder of this test's own, holding these entries.
fn a_place(called: &str, entries: &[(&str, &str)]) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-installed-kinds-{}-{called}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    for (named, text) in entries {
        std::fs::write(at.join(named), text).unwrap();
    }
    at
}

/// Dolphin's own entry, abridged, with one translation.
const DOLPHIN: &str = "[Desktop Entry]
Type=Application
Name=Dolphin
GenericName=File Manager
GenericName[fr]=Gestionnaire de fichiers
Keywords=files;file management;
Keywords[fr]=fichiers;gestion de fichiers;
Categories=Qt;KDE;System;FileManager;

[Desktop Action new-window]
Name=Open a New Window
Keywords=window;
";

/// The words, owned.
fn words(these: &[&str]) -> Vec<String> {
    these.iter().map(|it| (*it).to_owned()).collect()
}

#[test]
fn an_entry_says_what_it_is_in_its_own_words() {
    let place = a_place("own-words", &[("org.kde.dolphin.desktop", DOLPHIN)]);
    let each = what_they_are(std::slice::from_ref(&place), &[]);
    assert_eq!(
        each.get("org.kde.dolphin"),
        Some(&words(&[
            "File Manager",
            "files",
            "file management",
            "Qt",
            "KDE",
            "System",
            "FileManager",
        ])),
        "an action's keywords belong to a right-click entry, not to the application"
    );
}

#[test]
fn the_persons_language_is_read_first_and_the_entrys_own_words_are_kept() {
    let place = a_place("in-french", &[("org.kde.dolphin.desktop", DOLPHIN)]);
    let each = what_they_are(std::slice::from_ref(&place), &["fr-FR"]);
    assert_eq!(
        each.get("org.kde.dolphin"),
        Some(&words(&[
            "Gestionnaire de fichiers",
            "File Manager",
            "fichiers",
            "gestion de fichiers",
            "files",
            "file management",
            "Qt",
            "KDE",
            "System",
            "FileManager",
        ]))
    );
    let in_german = what_they_are(std::slice::from_ref(&place), &["de"]);
    assert!(
        !in_german
            .get("org.kde.dolphin")
            .is_some_and(|it| it.iter().any(|word| word == "fichiers")),
        "a language the person does not read was offered"
    );
}

/// `NoDisplay=true` and `Hidden=true` are the entry's author saying *do not
/// list me*, so they are not found by what they are either.
#[test]
fn an_entry_that_asks_not_to_be_listed_is_never_offered() {
    let place = a_place(
        "not-listed",
        &[
            (
                "helper.desktop",
                "[Desktop Entry]\nType=Application\nName=Helper\nNoDisplay=true\nKeywords=files;\n",
            ),
            (
                "gone.desktop",
                "[Desktop Entry]\nType=Application\nName=Gone\nHidden=true\nKeywords=files;\n",
            ),
        ],
    );
    assert_eq!(what_they_are(std::slice::from_ref(&place), &[]), WhatEachIs::new());
}

/// A person's own copy wins, even where it says less.
#[test]
fn the_first_place_with_the_entry_is_the_one_read() {
    let persons = a_place(
        "persons",
        &[(
            "org.kde.dolphin.desktop",
            "[Desktop Entry]\nType=Application\nName=Dolphin\nKeywords=mine;\n",
        )],
    );
    let systems = a_place("systems", &[("org.kde.dolphin.desktop", DOLPHIN)]);
    let each = what_they_are(&[persons, systems], &[]);
    assert_eq!(each.get("org.kde.dolphin"), Some(&words(&["mine"])));
}

#[test]
fn a_language_tag_becomes_the_keys_an_entry_is_localised_by() {
    assert_eq!(the_keys_a_language_reads("pt-BR"), words(&["pt_BR", "pt"]));
    assert_eq!(
        the_keys_a_language_reads("sr-Latn-RS"),
        words(&["sr_RS@latin", "sr_RS", "sr@latin", "sr"])
    );
    assert_eq!(the_keys_a_language_reads("de"), words(&["de"]));
    assert_eq!(the_keys_a_language_reads("es-419"), words(&["es_419", "es"]));
    assert!(the_keys_a_language_reads("").is_empty());
}
