#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_applications::Application;

use super::{NotToType, ToType, Typed, the_first_untypable};

fn codium() -> Application {
    Application::identified("com.vscodium.codium").unwrap()
}

#[test]
fn a_text_and_a_save_are_read_back_as_they_were_written() {
    for asked in [
        ToType::Text {
            application: codium(),
            bytes: 61,
        },
        ToType::Save {
            application: codium(),
        },
    ] {
        assert_eq!(ToType::read(&asked.written()), Some(Ok(asked)));
    }
}

#[test]
fn a_line_about_something_else_is_left_for_the_other_acts() {
    for line in ["open com.vscodium.codium", "front", "typed", ""] {
        assert_eq!(ToType::read(line), None, "{line}");
    }
}

#[test]
fn a_text_must_say_how_many_bytes_follow() {
    for line in [
        "type-into com.vscodium.codium",
        "type-into com.vscodium.codium 0",
        "type-into com.vscodium.codium -3",
        "type-into com.vscodium.codium +3",
        "type-into com.vscodium.codium three",
        "type-into com.vscodium.codium 3 4",
        "type-into com.vscodium.codium 99999999999999999999999999",
    ] {
        assert_eq!(ToType::read(line), Some(Err(NotToType::NoLength)), "{line}");
    }
}

#[test]
fn exactly_one_application_is_named() {
    for line in [
        "type-into",
        "save-in",
        "save-in com.vscodium.codium org.mozilla.firefox",
    ] {
        assert_eq!(
            ToType::read(line),
            Some(Err(NotToType::NotOneApplication)),
            "{line}"
        );
    }
}

#[test]
fn a_line_longer_than_the_door_allows_is_refused() {
    let line = format!("save-in {}", "a".repeat(600));
    assert_eq!(ToType::read(&line), Some(Err(NotToType::TooLong)));
}

#[test]
fn every_answer_is_read_back_as_it_was_written() {
    for answer in [
        Typed::Typed,
        Typed::AskedToSave,
        Typed::NotInFront,
        Typed::NotAlosCopy,
        Typed::NoKeyboard,
        Typed::CannotType('\u{7}'),
        Typed::CannotType('\u{1b}'),
    ] {
        assert_eq!(Typed::read(&answer.written()), Some(answer));
    }
    assert_eq!(Typed::CannotType('\u{7}').written(), "cannot-type U+0007");
    assert_eq!(Typed::read("cannot-type U+"), None);
    assert_eq!(Typed::read("cannot-type U+D800"), None);
    assert_eq!(Typed::read("acted"), None);
}

#[test]
fn new_lines_and_tabs_are_typed_and_other_control_characters_are_not() {
    assert_eq!(the_first_untypable("a\n\tb é 日本 — ✓"), None);
    assert_eq!(the_first_untypable("bell\u{7}here"), Some('\u{7}'));
    assert_eq!(the_first_untypable("line\r\n"), Some('\r'));
}
