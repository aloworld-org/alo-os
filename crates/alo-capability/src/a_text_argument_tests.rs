//! A text argument from arriving to being approved (ADR 0001, amended
//! 2026-11-13): no maximum length, many lines, nothing hidden refused, and the
//! whole text in front of the person beside a sentence that stays one line.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::Duration;

use alo_strings::Word;

use crate::arg::{Arg, ArgError, Given, Takes, Value};
use crate::call::Call;
use crate::grant::Grantee;
use crate::grants::Grants;
use crate::proposal::Proposal;
use crate::test_calls::noon;
use crate::testing::speaking;
use crate::verb::{Effect, Requires, Verb, VerbError};

/// What the fixture verb does.
const TYPING: Word = Word::saying("testing.verb.type-text.purpose", "type text into a window");
/// The sentence a person approves for it.
const TYPING_SENTENCE: Word = Word::saying(
    "testing.verb.type-text.sentence",
    "type {text} into {application}",
);
/// What its text argument is for.
const THE_TEXT: Word = Word::saying("testing.verb.type-text.argument.text", "what to type");
/// What its application argument is for.
const THE_APPLICATION: Word = Word::saying(
    "testing.verb.type-text.argument.application",
    "where to type it",
);

/// A text argument on its own.
fn text() -> Arg {
    Arg::taking("text", THE_TEXT, Takes::Text)
}

/// A change that types text into an application, needing no grant so the
/// tests ask only about the text.
fn type_text() -> Verb {
    Verb::checked(
        "type_text",
        TYPING,
        Effect::Change,
        vec![
            Arg::taking("application", THE_APPLICATION, Takes::Application),
            text(),
        ],
        Requires::nothing_because("a fixture: what these tests ask is about the text"),
        TYPING_SENTENCE,
    )
    .unwrap()
}

/// **No input limit.** A megabyte of text on thousands of lines arrives exactly
/// as it was sent — not trimmed, not cut.
#[test]
fn a_very_long_text_with_many_lines_is_accepted_exactly() {
    let given = format!("  indented\n{}\n\n", "fn main() {}\n".repeat(80_000));
    assert!(given.len() > 1_000_000);
    assert_eq!(
        text().validate(&Given::text(given.clone())).unwrap(),
        Value::Text(given)
    );
}

/// **Hidden characters are not refused** — every kind the approval marks
/// arrives, and the approval is what points at them.
#[test]
fn invisible_and_confusable_characters_are_accepted() {
    let given = "p\u{0430}ypal\u{200B} \u{202E}fdp.exe\u{1b}[2K\r\u{00A0}";
    let value = text().validate(&Given::text(given)).unwrap();
    assert_eq!(value, Value::Text(given.to_owned()));
    assert_eq!(value.the_whole_text().unwrap().unseen(), 6);
}

/// What is still refused is what is not text at all: nothing, or a number.
/// Spaces alone are something, because a person would see them typed.
#[test]
fn nothing_and_a_number_are_refused_and_spaces_are_not() {
    assert_eq!(
        text().validate(&Given::text("")).unwrap_err(),
        ArgError::Empty {
            argument: "text".to_owned()
        }
    );
    assert_eq!(
        text().validate(&Given::number(7)).unwrap_err(),
        ArgError::WantedText {
            argument: "text".to_owned()
        }
    );
    assert_eq!(
        text().validate(&Given::text("   ")).unwrap(),
        Value::Text("   ".to_owned())
    );
}

/// **Bytes that are not UTF-8 never become an argument**: the wire refuses
/// them before any verb is asked, raw or as a lone surrogate escape.
#[test]
fn what_is_not_valid_text_is_refused_at_the_wire() {
    assert!(serde_json::from_slice::<Given>(b"\"\xff\xfe\"").is_err());
    assert!(serde_json::from_str::<Given>(r#""\ud800""#).is_err());
    assert_eq!(
        serde_json::from_str::<Given>(r#""line one\nline two""#).unwrap(),
        Given::text("line one\nline two")
    );
}

/// A text says nothing about where it may reach, so no grant can be over one.
#[test]
fn a_text_cannot_be_what_a_grant_is_over() {
    assert!(!Takes::Text.can_be_a_grant());
    assert!(
        text()
            .validate(&Given::text("x"))
            .unwrap()
            .as_ask()
            .is_none()
    );
    assert!(matches!(
        Verb::checked(
            "type_text",
            TYPING,
            Effect::Change,
            vec![text()],
            Requires::grants_over(["text"]),
            Word::saying("testing.verb.type-text.alone", "type {text}"),
        ),
        Err(VerbError::CannotBeAGrant { .. })
    ));
}

/// **The approval shows the whole text.** The sentence stays one line and
/// points at *the text below*; the proposal carries every line of it, with its
/// length said and its hidden character marked.
#[test]
fn the_approval_shows_the_whole_text_beneath_a_one_line_sentence() {
    let page = format!("<h1>Hello</h1>\u{200B}\n{}", "<p>more</p>\n".repeat(5_000));
    let call = Call::of(
        &type_text(),
        &[
            ("application", Given::text("org.gnome.TextEditor")),
            ("text", Given::text(page.clone())),
        ],
    )
    .unwrap();
    let proposal = Proposal::checked(
        &call,
        &Grantee::named("@files"),
        &Grants::default(),
        noon(),
        Duration::from_secs(60),
    )
    .unwrap();
    let strings = speaking(&[TYPING, TYPING_SENTENCE, THE_TEXT, THE_APPLICATION]);

    let sentence = proposal.sentence(&strings);
    assert_eq!(
        sentence.text(),
        "type the text below into org.gnome.TextEditor"
    );
    assert!(!sentence.text().contains('\n'));

    let texts = proposal.texts();
    assert_eq!(texts.len(), 1);
    let (argument, shown) = &texts[0];
    assert_eq!(*argument, "text");
    assert_eq!(shown.as_given(), page);
    assert_eq!(shown.how_many_lines(), 5_001);
    assert_eq!(
        shown.how_long(&strings).text(),
        format!("5001 lines, {} characters", page.chars().count())
    );
    assert_eq!(shown.unseen(), 1);
    assert!(shown.what_is_marked(&strings).is_some());
}

/// **The record keeps the text** — every line of it survives being written
/// down, which is the direction the record needs.
#[test]
fn the_whole_text_can_be_written_down() {
    let given = "first line\nsecond line\u{200B}\n";
    let written = serde_json::to_string(&text().validate(&Given::text(given)).unwrap()).unwrap();
    assert_eq!(
        written,
        format!("{{\"text\":\"first line\\nsecond line{}\\n\"}}", '\u{200B}')
    );
}

/// A call without a text argument shows no text — every call on this machine
/// today.
#[test]
fn a_call_with_no_text_shows_none() {
    assert!(crate::test_calls::archiving_march().texts().is_empty());
}
