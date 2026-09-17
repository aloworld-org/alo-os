//! Every sentence the software, proxy and adapter crates can say: in the
//! vocabulary the machine speaks from, with a translator's note, one line each,
//! and naming none of the machinery rented to do the work.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 7: *every sentence
//! these crates can say is in the vocabulary with a translator's note* and *no
//! sentence names Flatpak, Flathub by its tooling, AT-SPI, D-Bus or a browser
//! engine.* Each crate's own `words.rs` holds a list of words it forbids; those
//! lists grew one crate at a time and each forbids what its author thought of
//! (the proxy's never mentioned the packaging, the adapters' never mentioned a
//! browser engine). This file asks **one** question of all three at once, and
//! asks it of the vocabulary `alo-saying` collects rather than of the lists the
//! crates hand it — so a word a crate declared and the machine never collected
//! is as much a failure as a word that names the plumbing.
//!
//! The name of a place applications come from, and an application's
//! identifier, are data filled into a sentence exactly as this machine knows
//! them, and are not what this reads: a sentence and the note beside it are.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeSet;

use alo_strings::{Key, Vocabulary, Word};

/// The three crates' areas in the vocabulary, as each names its keys.
const THE_AREAS: [&str; 3] = ["software", "proxy", "adapters"];

/// What no sentence and no note may name, with what each is.
///
/// Lower case, matched against lower-cased text. A browser engine is named by
/// every engine a browser a person might install is built on, and by the parts
/// of one a translator could be told about.
const THE_MACHINERY: &[(&str, &str)] = &[
    ("flatpak", "the packaging and sandboxing tool"),
    (
        "flathub",
        "the service applications come from, by its tooling's name",
    ),
    ("ostree", "what the packaging tool stores applications in"),
    ("at-spi", "the accessibility interface"),
    ("atspi", "the accessibility interface, unhyphenated"),
    ("d-bus", "the message bus"),
    ("dbus", "the message bus, unhyphenated"),
    ("gecko", "a browser engine"),
    ("spidermonkey", "a browser engine's script engine"),
    ("webkit", "a browser engine"),
    ("blink", "a browser engine"),
    ("chromium", "a browser engine and the browser built on it"),
    ("servo", "a browser engine"),
    ("webengine", "a browser engine embedded in a toolkit"),
    ("trident", "a browser engine"),
];

/// Every word the three crates declare, as each crate lists it.
fn every_word_the_three_declare() -> Vec<Word> {
    alo_software::words::EVERY_WORD
        .iter()
        .chain(alo_proxy::EVERY_WORD.iter())
        .chain(alo_adapters::EVERY_WORD.iter())
        .chain(alo_adapters::text_editor::WORDS.iter())
        .chain(alo_adapters::fallback_words::EVERY_WORD.iter())
        .copied()
        .collect()
}

/// The machine's own vocabulary.
fn the_machines() -> Vocabulary {
    alo_saying::everything_this_machine_can_say().unwrap()
}

/// What in this text names the machinery, if anything does.
fn names_the_machinery(text: &str) -> Option<&'static str> {
    let text = text.to_lowercase();
    THE_MACHINERY
        .iter()
        .find(|(named, _)| text.contains(named))
        .map(|(named, _)| *named)
}

/// Why this text does not read as one line with single blanks, if it does not.
fn not_one_line(text: &str) -> Option<&'static str> {
    if text.contains('\n') || text.contains('\r') {
        Some("a line break")
    } else if text.contains("  ") {
        Some("two blanks side by side")
    } else if text != text.trim() {
        Some("a blank at an end")
    } else {
        None
    }
}

/// **Every sentence these crates can say is in the machine's vocabulary, with
/// a translator's note** — the same English and the same note the crate wrote,
/// under the key it wrote — and the machine holds nothing under these areas
/// that the crates do not list, so neither direction can drift.
#[test]
fn every_sentence_is_in_the_machines_vocabulary_with_a_note() {
    let machine = the_machines();
    let declared = every_word_the_three_declare();
    let mut named = BTreeSet::new();
    for word in &declared {
        assert!(named.insert(word.named()), "{} twice", word.named());
        let phrase = machine
            .phrase(&Key::named(word.named()).unwrap())
            .unwrap_or_else(|| {
                panic!(
                    "{} is declared and the machine never collected it",
                    word.named()
                )
            });
        assert_eq!(
            phrase.source().as_written(),
            word.says(),
            "{}",
            word.named()
        );
        let note = phrase
            .note()
            .unwrap_or_else(|| panic!("{} reaches a translator with no note", word.named()));
        assert_eq!(Some(note), word.note(), "{}", word.named());
        assert!(
            note.split_whitespace().count() >= 5,
            "{}: a note of a word or two tells a translator nothing: {note:?}",
            word.named()
        );
    }
    let collected: BTreeSet<&str> = machine
        .phrases()
        .map(|phrase| phrase.key().as_str())
        .filter(|key| {
            THE_AREAS
                .iter()
                .any(|area| key.starts_with(&format!("{area}.")))
        })
        .collect();
    assert_eq!(
        collected, named,
        "the machine says something under these crates' areas that no list declares, or the \
         other way round"
    );
    assert!(
        machine
            .counted()
            .all(|plural| !THE_AREAS.contains(&plural.key().area())),
        "a counted sentence under these areas is on no list this file reads"
    );
}

/// **No sentence and no note names the machinery**: not the packaging, not the
/// service applications come from by its tooling, not the accessibility
/// interface, not the bus and not a browser engine.
#[test]
fn no_sentence_or_note_names_the_machinery() {
    for word in every_word_the_three_declare() {
        for text in [word.says(), word.note().unwrap_or_default()] {
            assert_eq!(
                names_the_machinery(text),
                None,
                "{}: {text:?}",
                word.named()
            );
        }
    }
}

/// **Every sentence and every note is one line with single blanks.** A string
/// continued onto the next line of a source file without its trailing
/// backslash carries the break and the indentation into what a person reads,
/// which is how two of the proxy's arrived.
#[test]
fn every_sentence_and_note_reads_as_one_line() {
    for word in every_word_the_three_declare() {
        for text in [word.says(), word.note().unwrap_or_default()] {
            assert_eq!(not_one_line(text), None, "{}: {text:?}", word.named());
        }
    }
}

/// **The machinery check refuses what it says it refuses**, each name in any
/// case and inside other words — held against sentences written here, so the
/// rule is seen refusing without planting a name in the vocabulary.
#[test]
fn the_machinery_check_finds_every_name_it_forbids() {
    for (named, _) in THE_MACHINERY {
        let upper = format!("Installed through {} today", named.to_uppercase());
        assert_eq!(names_the_machinery(&upper), Some(*named), "{upper}");
    }
    for (text, found) in [
        ("installed with Flatpak from Flathub", "flatpak"),
        ("from the flathub remote", "flathub"),
        ("the AT-SPI tree of the window", "at-spi"),
        ("sent over D-Bus to the application", "d-bus"),
        ("Firefox, which is built on Gecko", "gecko"),
        ("a QtWebEngine view", "webengine"),
    ] {
        assert_eq!(names_the_machinery(text), Some(found), "{text}");
    }
    for text in [
        "org.gnome.TextEditor is installed. It has been given nothing",
        "No application on this machine opens web addresses",
        "the part of the machine that tells screen readers what is on the screen",
    ] {
        assert_eq!(names_the_machinery(text), None, "{text}");
    }
}

/// **The one-line check refuses what it says it refuses** — the exact shape of
/// the two sentences it was written after, beside a sentence that is fine.
#[test]
fn the_one_line_check_finds_a_continued_line_without_its_backslash() {
    assert_eq!(
        not_one_line("without http:// in \n     front of it"),
        Some("a line break")
    );
    assert_eq!(
        not_one_line("a sentence  with a doubled blank"),
        Some("two blanks side by side")
    );
    assert_eq!(not_one_line("a sentence "), Some("a blank at an end"));
    assert_eq!(
        not_one_line("That is not the name of a proxy. Type the name of the machine on its own"),
        None
    );
}
