//! Why a keyboard was not added, removed or set up, and what a person is told.
//!
//! **Every refusal leaves the keyboards exactly as they were.** Each change in
//! [`crate::Keyboards`] is decided before anything is moved, so a person told
//! *nothing has been added* is told the truth.
//!
//! Like `alo_shortcuts`' refusals, a [`Refused`] has no `Display`: the only
//! road to words is [`Refused::said`], in the language the person reads.
//!
//! # No sentence here names a layout, a keysym or a framework
//!
//! *That keyboard*, not `us(intl)`. A person chose a row in a list and the row
//! had a name in it; repeating the rented code back at them tells them nothing
//! they can act on and tells them alo OS is made of somebody else's parts. The
//! one thing a sentence here does name is **a language, in its own language** —
//! `Ελληνικά`, not *Greek* — which is `alo_strings::Language::in_its_own_language`'s
//! rule and the one name a person reading the sentence definitely recognises.

use alo_strings::{Filling, Language, Said, Strings};

use crate::layout::Layout;
use crate::methods::Writing;
use crate::words::{self, Word};

/// Why the keyboards were left as they were.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// This machine's rented keyboard data does not have that keyboard.
    NotAKeyboardWeHave(Layout),
    /// That keyboard is already one of the person's.
    AlreadyOneOfYours(Layout),
    /// That keyboard is not one of the person's, so it cannot be removed.
    NotOneOfYours(Layout),
    /// The last keyboard cannot be removed: a person needs one to type with.
    TheLastKeyboard,
    /// alo OS has no keyboard to offer for that language.
    NoKeyboardForThisLanguage(Language),
    /// That language is typed on a keyboard, so an input method is not what it
    /// needs.
    TypedOnAKeyboard {
        /// The language asked for.
        language: Language,
        /// The keyboard it is typed on, which is what to add instead.
        keyboard: Layout,
    },
    /// That way of typing is already one of the person's.
    AlreadyTyping(Writing),
}

impl Refused {
    /// The keyboard this is about, when it is about one — so a surface can mark
    /// the row, since the sentence never names it.
    #[must_use]
    pub fn keyboard(&self) -> Option<&Layout> {
        match self {
            Self::NotAKeyboardWeHave(layout)
            | Self::AlreadyOneOfYours(layout)
            | Self::NotOneOfYours(layout)
            | Self::TypedOnAKeyboard {
                keyboard: layout, ..
            } => Some(layout),
            Self::TheLastKeyboard | Self::NoKeyboardForThisLanguage(_) | Self::AlreadyTyping(_) => {
                None
            }
        }
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::NotAKeyboardWeHave(_) => words::NOT_A_KEYBOARD_WE_HAVE,
            Self::AlreadyOneOfYours(_) => words::ALREADY_ONE_OF_YOURS,
            Self::NotOneOfYours(_) => words::NOT_ONE_OF_YOURS,
            Self::TheLastKeyboard => words::THE_LAST_KEYBOARD,
            Self::NoKeyboardForThisLanguage(_) => words::NO_KEYBOARD_FOR_THIS_LANGUAGE,
            Self::TypedOnAKeyboard { .. } => words::TYPED_ON_A_KEYBOARD,
            Self::AlreadyTyping(_) => words::ALREADY_TYPING,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NoKeyboardForThisLanguage(language) | Self::TypedOnAKeyboard { language, .. } => {
                Filling::of("language", in_itself(language))
            }
            _ => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// A language in its own language, or its tag when nobody has written its name
/// down yet — which is what [`alo_strings::Language::in_its_own_language`]
/// answers `None` for, and what a picker shows in the same place.
fn in_itself(language: &Language) -> String {
    language
        .in_its_own_language()
        .map_or_else(|| language.tag().to_owned(), ToOwned::to_owned)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use std::collections::BTreeSet;

    fn every_refusal() -> Vec<Refused> {
        let keyboard = Layout::named("de").unwrap();
        vec![
            Refused::NotAKeyboardWeHave(keyboard.clone()),
            Refused::AlreadyOneOfYours(keyboard.clone()),
            Refused::NotOneOfYours(keyboard.clone()),
            Refused::TheLastKeyboard,
            Refused::NoKeyboardForThisLanguage(Language::written("is").unwrap()),
            Refused::TypedOnAKeyboard {
                language: Language::written("el").unwrap(),
                keyboard,
            },
            Refused::AlreadyTyping(Writing::Japanese),
        ]
    }

    /// Every refusal says something of its own, declared, with nothing left to
    /// fill in — a person told the wrong reason tries the same thing again.
    #[test]
    fn every_refusal_says_something_of_its_own() {
        let strings = in_english();
        let mut seen = BTreeSet::new();
        for refused in every_refusal() {
            let said = refused.said(&strings);
            assert!(!said.is_a_bug(), "{refused:?}");
            assert!(said.unfilled().is_empty(), "{refused:?}: {said}");
            assert!(seen.insert(said.text().to_owned()), "{refused:?}");
        }
    }

    /// **No sentence names a layout**, and the keyboard a refusal is about is
    /// carried beside it for the surface to mark instead.
    #[test]
    fn no_sentence_names_a_layout_and_the_keyboard_is_carried_beside_it() {
        let strings = in_english();
        for refused in every_refusal() {
            let said = refused.said(&strings);
            for named in ["us(intl)", "de", "xkb", "XKB", "ibus", "mozc"] {
                assert!(
                    !said.text().split_whitespace().any(|word| word == named),
                    "{refused:?} says {named}: {said}"
                );
            }
        }
        assert_eq!(
            Refused::NotAKeyboardWeHave(Layout::named("qwertz").unwrap()).keyboard(),
            Some(&Layout::named("qwertz").unwrap())
        );
        assert_eq!(Refused::TheLastKeyboard.keyboard(), None);
    }

    /// **A language is named in its own language**, and a language nobody has
    /// written a name for keeps its tag rather than becoming a gap.
    #[test]
    fn a_language_is_named_in_its_own_language() {
        let strings = in_english();
        let greek = Refused::TypedOnAKeyboard {
            language: Language::written("el").unwrap(),
            keyboard: Layout::named("gr").unwrap(),
        };
        assert!(
            greek.said(&strings).text().contains("Ελληνικά"),
            "{greek:?}"
        );
        let icelandic = Refused::NoKeyboardForThisLanguage(Language::written("is").unwrap());
        assert!(icelandic.said(&strings).text().contains("is"));
    }
}
