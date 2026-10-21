//! Every character this environment can put on a screen has a glyph in the one
//! face it carries.
//!
//! # The fault this exists to stop, which shipped
//!
//! The caret on a text field was `▌` — U+2590 LEFT HALF BLOCK. The bundled
//! `Manrope-SemiBold.ttf` does not have that character: its own `cmap` maps it
//! to glyph 0. So the caret drew nothing, and because the caret is the **only**
//! thing on an empty field, the password screen showed no field at all. A
//! person was asked for a password and given nowhere to type it.
//!
//! **Nothing caught it and nothing could have.** The sentence was built
//! correctly. The screen laid out. Every test asked whether it laid out.
//! `how_wide` treats a character with no glyph as adding nothing, which is the
//! right arithmetic and is also exactly what makes the fault silent: the
//! missing character has zero width, so the line is not even the wrong length.
//!
//! It was found by rendering a screen to an image and looking at it, after a
//! day of green tests — and the owner asking why the screen was black found a
//! second one the same way. This file is the half of that which runs forever.
//!
//! # What it covers
//!
//! Every character of every sentence `alo-installing` publishes, plus the
//! symbols the crate draws that are not in any sentence, plus the layout names.
//! **The layout names are the part a reader would skip**: they are endonyms —
//! *Français*, *Español* — data rather than words, never translated, and so
//! they are in no vocabulary and no other check sees them.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use alo_installing::AlosOwnFace;

/// The symbols this crate draws that are in no published sentence.
///
/// Kept here deliberately rather than read out of the source: a constant that
/// changes should fail this test by being absent from the list, not quietly
/// take the new value with it.
const THE_SYMBOLS: [(&str, char); 2] = [("the caret on a field", '|'), ("a hidden character", '•')];

/// The eight layout names `main.rs` offers, which are endonyms and so appear in
/// no vocabulary.
const THE_LAYOUT_NAMES: [&str; 8] = [
    "Belgisch",
    "Français",
    "Deutsch",
    "Español",
    "Italiano",
    "Nederlands",
    "English (UK)",
    "English (US)",
];

#[test]
fn every_symbol_this_crate_draws_has_a_glyph() {
    let face = AlosOwnFace::carried().expect("the typeface compiled into this program");
    let mut missing = Vec::new();
    for (what, character) in THE_SYMBOLS {
        if face.the_glyph_for(character).is_none() {
            missing.push(format!(
                "{what}: {character:?} (U+{:04X}) has no glyph",
                character as u32
            ));
        }
    }
    assert!(
        missing.is_empty(),
        "the face cannot draw {} of the symbols this crate puts on a screen, \
         and a character with no glyph draws nothing and takes no width — so \
         the screen looks finished and is missing something:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
}

#[test]
fn every_layout_name_offered_can_be_drawn() {
    // These are what the keyboard list shows. A name whose accented letter has
    // no glyph is a layout drawn as `Franais`, which looks like a typo in our
    // product rather than a missing glyph.
    let face = AlosOwnFace::carried().expect("the typeface compiled into this program");
    let mut missing = Vec::new();
    for name in THE_LAYOUT_NAMES {
        for character in name.chars() {
            if face.the_glyph_for(character).is_none() {
                missing.push(format!("{name:?} needs {character:?}"));
            }
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

#[test]
fn every_sentence_this_environment_says_can_be_drawn() {
    // The sentences themselves, from the vocabulary this machine collects —
    // the same source `alo-saying`'s own checks read, so this cannot drift from
    // what is actually published.
    let vocabulary = alo_saying::everything_this_machine_can_say().expect("alo OS's own words");
    let face = AlosOwnFace::carried().expect("the typeface compiled into this program");

    let mut missing: Vec<String> = Vec::new();
    let mut looked_at = 0_usize;
    for phrase in vocabulary.phrases() {
        let key = phrase.key();
        let key = key.as_str();
        if !key.starts_with("installing.") {
            continue;
        }
        looked_at += 1;
        let english = phrase.source().as_written();
        for character in english.chars() {
            // A placeholder's braces are consumed before anything is drawn, and
            // a newline is already refused by `a_sentence_is_not_padded`.
            if matches!(character, '{' | '}' | ' ') {
                continue;
            }
            if face.the_glyph_for(character).is_none() {
                missing.push(format!(
                    "{key} needs {character:?} (U+{:04X})",
                    character as u32
                ));
            }
        }
    }
    missing.sort_unstable();
    missing.dedup();

    // **The control.** All of the above passes trivially if no sentence was
    // looked at — and this crate has already had a search that under-reported
    // by matching nothing and saying so quietly.
    assert!(
        looked_at > 50,
        "only {looked_at} of this crate's sentences were found, so this check \
         is not reading the environment's words"
    );
    assert!(
        missing.is_empty(),
        "{} character(s) in this environment's own sentences have no glyph in \
         the face it carries. Each one draws nothing and takes no width, so \
         the sentence appears complete and is not:\n  {}",
        missing.len(),
        missing.join("\n  ")
    );
}
