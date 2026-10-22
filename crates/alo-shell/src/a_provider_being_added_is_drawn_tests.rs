//! The form read back as the words a person would see on the screen.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_setting_up::a_provider_is_filled_in::{AProviderBeingAddded, WhichField};
use alo_strings::{Strings, Vocabulary};

use super::the_form;

/// A vocabulary holding this form's own sentences and nothing else.
fn strings() -> Strings {
    let mut vocabulary = Vocabulary::default();
    crate::the_providers_words::declare_the_providers_words(&mut vocabulary)
        .expect("this form's own words declare");
    Strings::of(vocabulary)
}

/// Type a whole string into the field the keyboard is in.
fn types(form: &mut AProviderBeingAddded, what: &str) {
    for letter in what.chars() {
        form.typed(letter);
    }
}

/// What is drawn to the right of the label on this field's line.
fn what_is_shown(form: &AProviderBeingAddded, field: WhichField) -> String {
    let drawn = the_form(form, &strings());
    let at = WhichField::EVERY
        .iter()
        .position(|each| *each == field)
        .expect("one of the four");
    drawn
        .lines
        .get(at)
        .expect("a line per field")
        .parts
        .get(1)
        .expect("a label and what is in it")
        .clone()
}

#[test]
fn there_is_a_line_for_each_of_the_four_fields() {
    let drawn = the_form(&AProviderBeingAddded::empty(), &strings());
    assert_eq!(drawn.lines.len(), 4);
    // And a heading and a line saying what to do, above them.
    assert_eq!(drawn.said.len(), 2);
}

#[test]
fn every_label_is_a_sentence_and_not_a_literal() {
    // The i18n law: a label written into the drawing is a label nobody can
    // translate. Each of these comes back from the vocabulary.
    let drawn = the_form(&AProviderBeingAddded::empty(), &strings());
    let labels: Vec<&str> = drawn
        .lines
        .iter()
        .map(|line| line.parts.first().expect("a label").as_str())
        .collect();
    assert_eq!(labels, vec!["Name", "Address", "Key", "Model"]);
}

#[test]
fn what_a_person_typed_is_drawn_back_to_them() {
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "OpenAI");
    form.to_the_next_field();
    types(&mut form, "https://api.openai.com");

    assert_eq!(what_is_shown(&form, WhichField::Name), "OpenAI");
    assert_eq!(
        what_is_shown(&form, WhichField::Address),
        "https://api.openai.com▏",
        "and the field with the keyboard carries the caret"
    );
}

#[test]
fn the_key_is_never_drawn_in_clear() {
    // **The assertion this file exists for.** A key on a screen is a key on
    // anybody's shoulder, on a video call, and in the demo recording.
    let mut form = AProviderBeingAddded::empty();
    form.the_keyboard_is_in(WhichField::Key);
    types(&mut form, "sk-proj-a-real-looking-key");

    let shown = what_is_shown(&form, WhichField::Key);
    assert!(
        !shown.contains("sk-proj"),
        "the key reached the screen: {shown}"
    );
    assert!(
        shown.starts_with("••••"),
        "and what is drawn instead is marks: {shown}"
    );
}

#[test]
fn one_mark_is_drawn_for_each_letter_a_reader_would_count() {
    // Per grapheme and not per byte: a key with a letter outside ASCII in it
    // would otherwise draw more marks than it has letters, which tells a
    // person their key is longer than it is.
    let mut form = AProviderBeingAddded::empty();
    form.the_keyboard_is_in(WhichField::Key);
    types(&mut form, "abcd");
    assert_eq!(what_is_shown(&form, WhichField::Key), "••••▏");

    let mut wider = AProviderBeingAddded::empty();
    wider.the_keyboard_is_in(WhichField::Key);
    types(&mut wider, "नमस्ते");
    assert_eq!(
        what_is_shown(&wider, WhichField::Key),
        "•••▏",
        "three graphemes, not the six chars or the eighteen bytes"
    );
}

#[test]
fn the_other_three_fields_are_not_masked() {
    // The control at the other end: masking is asked of the field, so a
    // surface that masked everything would pass the test above and be useless.
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "OpenAI");
    assert_eq!(what_is_shown(&form, WhichField::Name), "OpenAI▏");
    assert!(!what_is_shown(&form, WhichField::Name).contains('•'));
}

#[test]
fn the_caret_is_only_in_the_field_with_the_keyboard() {
    let mut form = AProviderBeingAddded::empty();
    types(&mut form, "OpenAI");
    assert!(what_is_shown(&form, WhichField::Name).ends_with('▏'));
    assert_eq!(
        what_is_shown(&form, WhichField::Address),
        "",
        "a field nobody is in shows nothing, not an empty caret"
    );
}

#[test]
fn an_empty_field_with_the_keyboard_still_shows_where_they_are() {
    // So a person can see which field they are in before typing anything.
    let form = AProviderBeingAddded::empty();
    assert_eq!(what_is_shown(&form, WhichField::Name), "▏");
}

#[test]
fn the_field_with_the_keyboard_is_the_one_marked() {
    // `settings_raster` draws `chosen` as the filled square at the start of a
    // row, which is the one piece of state this section has.
    let mut form = AProviderBeingAddded::empty();
    form.to_the_next_field();
    let drawn = the_form(&form, &strings());
    let marked: Vec<bool> = drawn.lines.iter().map(|line| line.chosen).collect();
    assert_eq!(marked, vec![false, true, false, false]);
}

#[test]
fn no_line_is_a_row_a_person_acts_on() {
    // Enter saves the whole form rather than acting on the line the keyboard
    // is in, so a row here would put one action on four lines.
    let drawn = the_form(&AProviderBeingAddded::empty(), &strings());
    assert!(drawn.lines.iter().all(|line| line.row.is_none()));
}
