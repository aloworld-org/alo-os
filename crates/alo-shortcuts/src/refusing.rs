//! Why a key combination cannot be a shortcut, and what a person is told.
//!
//! Three refusals, and a person meets all of them in the middle of setting a
//! shortcut: they held nothing, they held only Shift, or they pressed one of
//! the three combinations every application's clipboard is worked by. So each
//! says what to press instead rather than what was wrong.
//!
//! This is a file of its own rather than the bottom of [`crate::chord`] because
//! the two change for different reasons. A chord changes when what a person can
//! press changes; a refusal changes when what we say to them does — a new
//! language, a better sentence, a promise elsewhere that becomes a refusal here.
//!
//! # There is no way to turn one of these into English by accident
//!
//! A [`ChordError`] has no `Display` and is therefore not a `std::error::Error`.
//! The only road to words is [`ChordError::said`], which takes the strings the
//! person in front of the machine reads and answers with a `Said` that says
//! whether anybody translated it. That is item 9b's decision reaching the
//! keyboard: a `Display` here would be an English sentence one `to_string()`
//! away from a settings panel whose author had no reason to think about it. What
//! is lost is `std::error::Error` on a type that was never an error a programmer
//! handles — it is a sentence a person reads.
//!
//! One caller does need something to write when it cannot ask: the deserialiser
//! in [`crate::chord`], which has no `Strings` and never will. It writes the
//! *key* of the refusal, so that whoever reports the settings file that did not
//! read can look up the same words a panel would have shown.

use alo_strings::{Filling, Said, Strings};

use crate::key::Key;
use crate::modifier::{Modifier, Modifiers};
use crate::words::{self, Word};

/// One of the three things every application's clipboard does.
///
/// A closed list rather than a word, because each of the three is its own
/// sentence: the key is fixed by which one it is, and *cutting* is a different
/// verb from *copying* in every language including this one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clipboard {
    /// `Ctrl+C`.
    Copy,
    /// `Ctrl+X`.
    Cut,
    /// `Ctrl+V`.
    Paste,
}

impl Clipboard {
    /// The key this one is worked by. Ctrl is held; that is the only chord it
    /// is.
    #[must_use]
    pub fn key(self) -> Key {
        match self {
            Self::Copy => Key::C,
            Self::Cut => Key::X,
            Self::Paste => Key::V,
        }
    }

    /// The string this crate declares for refusing it.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::Copy => words::THE_CLIPBOARD_COPIES,
            Self::Cut => words::THE_CLIPBOARD_CUTS,
            Self::Paste => words::THE_CLIPBOARD_PASTES,
        }
    }
}

/// Why a key combination cannot be a shortcut.
///
/// Each says what to do about it, because a person reading this is in the
/// middle of setting a shortcut and wants the next thing to try.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordError {
    /// Nothing was held down.
    NothingHeld(Key),
    /// Only Shift was held down, which leaves the key printing a character.
    ShiftIsNotEnough(Key),
    /// One of the three chords the clipboard is worked by.
    TheClipboardNeedsIt(Clipboard),
}

impl ChordError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::NothingHeld(_) => words::NOTHING_HELD,
            Self::ShiftIsNotEnough(_) => words::SHIFT_IS_NOT_ENOUGH,
            Self::TheClipboardNeedsIt(clipboard) => clipboard.word(),
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    /// A `Strings` that was never given [`crate::shortcut_words`] answers with
    /// the key, marked, and `Said::is_a_bug` — which is the honest answer to
    /// *the shell forgot to declare what this crate can say*, and is not
    /// something this crate can paper over with a sentence of its own.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NothingHeld(key) | Self::ShiftIsNotEnough(key) => {
                Filling::of("key", key.shown(strings))
            }
            Self::TheClipboardNeedsIt(_) => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// Which clipboard action a combination is, when it is one.
pub(crate) fn the_clipboard(modifiers: Modifiers, key: Key) -> Option<Clipboard> {
    if modifiers != Modifiers::just(Modifier::Ctrl) {
        return None;
    }
    match key {
        Key::C => Some(Clipboard::Copy),
        Key::X => Some(Clipboard::Cut),
        Key::V => Some(Clipboard::Paste),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Every refusal names a string this crate declares, and no two of them
    /// name the same one — a person told the wrong thing about which rule they
    /// broke would try the same combination again.
    #[test]
    fn every_refusal_says_something_of_its_own() {
        let strings = in_english();
        let all = [
            ChordError::NothingHeld(Key::Q),
            ChordError::ShiftIsNotEnough(Key::Q),
            ChordError::TheClipboardNeedsIt(Clipboard::Copy),
            ChordError::TheClipboardNeedsIt(Clipboard::Cut),
            ChordError::TheClipboardNeedsIt(Clipboard::Paste),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for refused in all {
            let said = refused.said(&strings);
            assert!(!said.is_a_bug(), "{refused:?}");
            assert!(said.unfilled().is_empty(), "{refused:?}: {said}");
            assert!(seen.insert(said.text().to_owned()), "{refused:?}");
        }
    }

    /// **The key that was pressed is in the sentence**, in the language the
    /// person reads it in, because a refusal that did not name it would leave
    /// somebody pressing keys to find out which one it meant.
    #[test]
    fn the_key_that_was_pressed_is_named_in_the_readers_language() {
        let strings = translated(&[
            (words::PAGE_UP, "Bild ↑"),
            (
                words::NOTHING_HELD,
                "halten Sie zusätzlich Super, Strg oder Alt — {key} allein wäre eine Taste \
                 weniger für alles, was Sie schreiben",
            ),
        ]);
        let said = ChordError::NothingHeld(Key::PageUp).said(&strings);
        assert!(said.text().contains("Bild ↑"), "{said}");
        assert!(said.is_translated());
        assert!(said.unfilled().is_empty());
    }

    /// The three the clipboard needs are three sentences rather than one with
    /// the verb pushed into a gap: *copying* and *cutting* are different words
    /// in every language, and one of them declined into a hole is how a
    /// sentence reads like a machine wrote it.
    #[test]
    fn each_clipboard_action_is_its_own_sentence() {
        let strings = in_english();
        for (clipboard, key, what) in [
            (Clipboard::Copy, Key::C, "copying"),
            (Clipboard::Cut, Key::X, "cutting"),
            (Clipboard::Paste, Key::V, "pasting"),
        ] {
            assert_eq!(clipboard.key(), key);
            let said = ChordError::TheClipboardNeedsIt(clipboard).said(&strings);
            assert!(said.text().contains(what), "{said}");
            assert!(said.text().contains(key.shown(&strings).as_str()), "{said}");
        }
    }

    /// Only `Ctrl` and one of those three is the clipboard. Anything else held
    /// with them is a chord like any other, which is what lets a person keep
    /// `Ctrl+Shift+V`.
    #[test]
    fn nothing_but_ctrl_and_those_three_is_the_clipboard() {
        assert_eq!(
            the_clipboard(Modifiers::just(Modifier::Ctrl), Key::C),
            Some(Clipboard::Copy)
        );
        assert_eq!(
            the_clipboard(Modifiers::just(Modifier::Ctrl).and(Modifier::Shift), Key::C),
            None
        );
        assert_eq!(
            the_clipboard(Modifiers::just(Modifier::Super), Key::V),
            None
        );
        assert_eq!(the_clipboard(Modifiers::just(Modifier::Ctrl), Key::B), None);
    }

    /// A machine that was never given this crate's words still refuses exactly
    /// what it refused before, and says which rule it broke rather than
    /// pretending to a sentence. **A refusal never depends on a string table.**
    #[test]
    fn a_refusal_without_the_words_still_names_the_rule() {
        let nothing_declared = Strings::of(alo_strings::Vocabulary::empty());
        let said = ChordError::ShiftIsNotEnough(Key::Digit2).said(&nothing_declared);
        assert!(said.is_a_bug());
        assert!(
            said.text().contains("shortcuts.chord.shift-is-not-enough"),
            "{said}"
        );
    }
}
