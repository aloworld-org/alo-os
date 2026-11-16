//! Tests for [`super`]: what the card shows of a text, which is every line in
//! view with every hidden character a tag, and how long it is.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use alo_capability::TheWholeText;

use super::*;
use crate::alos_marked_words::AMark;

/// What this machine says, with the capability crate's words declared.
fn words() -> Strings {
    let mut vocabulary = alo_strings::Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// `how_many` numbered lines.
fn lines(how_many: usize) -> String {
    (1..=how_many)
        .map(|it| format!("line {it}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The stretches of `shown` marked as tags, as text.
fn tags(shown: &MarkedWords) -> Vec<&str> {
    let mut from = 0;
    shown
        .marks()
        .iter()
        .filter_map(|AMark { ends, how }| {
            let words = &shown.text()[from..*ends];
            from = *ends;
            (*how == HowItIsMarked::Unseen).then_some(words)
        })
        .collect()
}

/// **A hidden character is its tag, in its place, and marked as a tag**; a
/// tab and the letters round it are drawn as they are, and nothing is cut.
#[test]
fn a_hidden_character_is_a_tag_in_its_place() {
    let text = TheWholeText::of("hé\u{200B}llo\tworld");
    let card = ATextOnTheCard::of(&text, 0, &words());
    assert_eq!(card.shown.text(), "héU+200Bllo\tworld");
    assert_eq!(tags(&card.shown), ["U+200B"]);
    assert_eq!(card.said_lines(), 2, "how long it is, and that one is marked");
    assert!(card.marked.is_some());
}

/// **A letter from another alphabet is drawn, and its tag follows it**, so the
/// word still reads as it will be typed.
#[test]
fn a_letter_from_another_alphabet_is_drawn_and_then_named() {
    // A Cyrillic а in a Latin word.
    let text = TheWholeText::of("p\u{0430}ypal");
    let card = ATextOnTheCard::of(&text, 0, &words());
    assert_eq!(card.shown.text(), "p\u{0430}U+0430ypal");
    assert_eq!(tags(&card.shown), ["U+0430"]);
}

/// **A text with nothing hidden says only how long it is**, and draws exactly
/// what arrived.
#[test]
fn a_plain_text_says_how_long_it_is_and_draws_what_arrived() {
    let text = TheWholeText::of("one\ntwo\nthree");
    let card = ATextOnTheCard::of(&text, 0, &words());
    assert_eq!(card.shown.text(), "one\ntwo\nthree");
    assert!(tags(&card.shown).is_empty());
    assert_eq!(card.marked, None);
    assert_eq!(card.said_lines(), 1);
    assert_eq!(card.how_long, "3 lines, 13 characters");
}

/// **Twelve lines are in view, from where it is scrolled to**, and scrolling
/// past the end shows the last twelve rather than fewer.
#[test]
fn twelve_lines_are_in_view_from_where_it_is_scrolled() {
    let text = TheWholeText::of(&lines(40));
    let strings = words();
    let at_the_top = ATextOnTheCard::of(&text, 0, &strings);
    assert_eq!(at_the_top.shown.text(), lines(12));
    let further = ATextOnTheCard::of(&text, 5, &strings);
    assert_eq!(
        further.shown.text().lines().next().unwrap(),
        "line 6",
        "{further:?}"
    );
    assert_eq!(further.shown.text().lines().count(), 12);
    let past = ATextOnTheCard::of(&text, 99, &strings);
    assert_eq!(past.shown.text().lines().next().unwrap(), "line 29");
    assert_eq!(past.shown.text().lines().last().unwrap(), "line 40");
    // **How long it is counts the whole text**, not the twelve in view.
    assert!(at_the_top.how_long.starts_with("40 lines"), "{}", at_the_top.how_long);
}

/// **A very long line is kept whole** — the box wraps it; nothing here cuts.
#[test]
fn a_long_line_is_kept_whole() {
    let long = "x".repeat(5000);
    let card = ATextOnTheCard::of(&TheWholeText::of(&long), 0, &words());
    assert_eq!(card.shown.text(), long);
}
