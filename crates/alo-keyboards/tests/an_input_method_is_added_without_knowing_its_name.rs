//! **A person adds a way of typing Japanese by choosing Japanese**, and the
//! rented framework starts with their session because they did — not because
//! the machine starts it for everybody in case somebody needs it.
//!
//! Chinese, Japanese and Korean are not typed by pressing the key with the
//! letter on it. What does that is a large piece of rented software, and the
//! thing this crate owes a person is that they never have to learn its name,
//! nor the name of the engine inside it.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_keyboards::{IT_IS_STARTED_BY, Keyboards, Refused, THE_FRAMEWORK, Writing, keyboard_words};
use alo_strings::{Language, Strings};

/// A language, written.
fn language(tag: &str) -> Language {
    Language::written(tag).unwrap()
}

/// **A language is all a person names**, and what starts with the session is
/// the rented framework.
#[test]
fn a_person_names_a_language_and_never_an_engine() {
    let mut keyboards = Keyboards::shipped();
    assert_eq!(keyboards.what_the_session_starts(), None);

    for (tag, writing) in [
        ("ja", Writing::Japanese),
        ("ko", Writing::Korean),
        ("zh-CN", Writing::ChineseSimplified),
        ("zh-TW", Writing::ChineseTraditional),
    ] {
        assert_eq!(
            keyboards.type_language(&language(tag)),
            Ok(writing),
            "{tag}"
        );
    }
    assert_eq!(keyboards.input_methods().len(), 4);
    assert_eq!(keyboards.what_the_session_starts(), Some(IT_IS_STARTED_BY));
    assert!(IT_IS_STARTED_BY.starts_with(THE_FRAMEWORK));
}

/// **Nothing is started on a machine that does not need one.** A framework
/// running for everybody would be a process reading every keystroke on machines
/// where nobody asked for it.
#[test]
fn nothing_is_started_on_a_machine_that_needs_no_input_method() {
    let mut keyboards = Keyboards::offered_with(&language("de"));
    assert_eq!(keyboards.what_the_session_starts(), None);
    keyboards.type_language(&language("ja")).unwrap();
    assert_eq!(keyboards.what_the_session_starts(), Some(IT_IS_STARTED_BY));
    assert!(keyboards.stop_typing(Writing::Japanese));
    assert_eq!(keyboards.what_the_session_starts(), None);
}

/// **A language typed on a keyboard is refused and sent to the keyboard.**
///
/// The refusal somebody actually meets: a Greek speaker looking for Greek in
/// the list of ways to type, because Greek is not written in Latin letters
/// either. They are told what is true — Greek has a keyboard — rather than
/// being given an input method that would do nothing for them.
#[test]
fn a_language_typed_on_a_keyboard_is_sent_to_its_keyboard() {
    let strings = Strings::of(keyboard_words().unwrap());
    let mut keyboards = Keyboards::shipped();

    for tag in ["el", "bg", "de", "mt", "ga"] {
        let refused = keyboards.type_language(&language(tag)).unwrap_err();
        let Refused::TypedOnAKeyboard {
            language: named,
            keyboard,
        } = &refused
        else {
            unreachable!("{tag} was not sent to a keyboard: {refused:?}")
        };
        assert_eq!(named, &language(tag));
        assert_eq!(refused.keyboard(), Some(keyboard));

        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{tag}: {said}");
        assert!(said.unfilled().is_empty(), "{tag}: {said}");
        assert!(
            said.text()
                .contains(language(tag).in_its_own_language().unwrap()),
            "{tag}: {said}"
        );
        for named in ["XKB", "ibus", "mozc", "hangul", "libpinyin"] {
            assert!(!said.text().contains(named), "{tag}: {said} names {named}");
        }
    }
    assert!(keyboards.input_methods().is_empty());
}

/// **A language alo OS knows neither way of typing is refused, and says so.**
/// Nothing is invented for it, and nothing is quietly added.
#[test]
fn a_language_alo_os_cannot_type_yet_is_refused_and_says_so() {
    let strings = Strings::of(keyboard_words().unwrap());
    let mut keyboards = Keyboards::shipped();
    let refused = keyboards.type_language(&language("is")).unwrap_err();
    assert_eq!(
        refused,
        Refused::NoKeyboardForThisLanguage(language("is")),
        "Icelandic"
    );
    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.unfilled().is_empty(), "{said}");
    assert!(keyboards.input_methods().is_empty());
}

/// **The same way of typing is not added twice.**
#[test]
fn the_same_way_of_typing_is_not_added_twice() {
    let mut keyboards = Keyboards::shipped();
    assert_eq!(
        keyboards.type_language(&language("ja")),
        Ok(Writing::Japanese)
    );
    assert_eq!(
        keyboards.type_language(&language("ja-JP")),
        Err(Refused::AlreadyTyping(Writing::Japanese))
    );
    assert_eq!(keyboards.input_methods(), &[Writing::Japanese]);
}
