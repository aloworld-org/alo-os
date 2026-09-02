//! The key a shortcut is bound to, and the closed list of the ones that can be.
//!
//! **A key here is the one that prints this character on the keyboard in front
//! of the person**, not a position on an American keyboard. [`Key::Q`] is the Q
//! key: on a French keyboard it is where an English one has A, and a French
//! person setting `Super+Q` presses the key marked Q and gets `Super+Q`. Binding
//! to positions would mean a shortcut a person cannot read off their own
//! keyboard, and in a product whose first target is 24 languages that is not a
//! detail.
//!
//! It leaves one thing for whatever reads the keyboard, and it is written here
//! rather than discovered later: **a layout that prints no Latin letters** —
//! Greek, Bulgarian, Ukrainian — has no key marked Q at all. Every desktop
//! answers this the same way, by matching the shortcut against the Latin layout
//! the person also has, and the compositor is where that lookup belongs. This
//! type is what it produces, not how it gets there.
//!
//! **The list is closed** and it is short on purpose. Everything in it is a key
//! a person could sensibly ask a system shortcut to be built on; the volume and
//! brightness keys are not here because they are not chords and will arrive with
//! the status area that shows what they changed.
//!
//! # Two kinds of key, and only one of them is a string
//!
//! A key is shown as what is printed on it, and that divides the list in two.
//! Fifty-three of them print a **mark** — `Q`, `7`, `,`, `F1` — which is the
//! same mark on every keyboard in the union, and translating it would be
//! renaming a *position*, which is the model this file exists to reject. The
//! other sixteen print a **word**, and it is a different word on almost every
//! keyboard: a German one says *Entf* for Delete and *Bild ↑* for Page Up. Only
//! those sixteen are declared in [`crate::words`], and the reasoning in full is
//! in that module.
//!
//! So there is no `label` and no `Display`. [`Key::mark`] answers for the first
//! kind, [`Key::said`] for the second, and [`Key::shown`] is what a settings
//! panel draws for either.
//!
//! One macro declares the list once. Three parallel lists — the variants, what
//! is printed on them, and the array a settings panel walks — would drift, and
//! the way that drift shows up is a key nobody can pick from a list that still
//! compiles.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words::{self, Word};

/// Declares the closed list of keys, what is printed on each, and the array of
/// all of them, from two lists: the keys that print a mark and the keys that
/// print a word.
macro_rules! keys {
    (
        marked { $($marked:ident => $mark:literal),* $(,)? }
        worded { $($worded:ident => $word:expr),* $(,)? }
    ) => {
        /// A key a shortcut can be built on.
        ///
        /// The names are a stored format — they are what a settings file holds
        /// — so they change additively or not at all.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub enum Key {
            $(
                #[doc = concat!("The ", $mark, " key.")]
                $marked,
            )*
            $(
                #[doc = concat!("The ", stringify!($worded), " key, which prints a word.")]
                $worded,
            )*
        }

        impl Key {
            /// Every key a shortcut can be built on, in the order a settings
            /// panel offers them: the ones that print a mark, then the ones
            /// that print a word.
            pub const ALL: &'static [Self] = &[$(Self::$marked,)* $(Self::$worded,)*];

            /// What is printed on this key, when it is a mark rather than a
            /// word — the same on every keyboard, and never translated.
            ///
            /// `None` for the sixteen that print a word.
            #[must_use]
            pub fn mark(self) -> Option<&'static str> {
                match self {
                    $(Self::$marked => Some($mark),)*
                    $(Self::$worded => None,)*
                }
            }

            /// The string this crate declares for this key, when it prints a
            /// word.
            ///
            /// `None` for the keys that print a mark, which are not strings —
            /// [`crate::words`] says why.
            #[must_use]
            pub fn word(self) -> Option<Word> {
                match self {
                    $(Self::$marked => None,)*
                    $(Self::$worded => Some($word),)*
                }
            }

            /// What a person is shown for this key, in the language they read.
            ///
            /// A `String` rather than a `Said` because half the answers are not
            /// strings at all: whether the word half was translated is
            /// [`Key::said`]'s to answer, and for a mark the question does not
            /// arise.
            #[must_use]
            pub fn shown(self, strings: &Strings) -> String {
                match self {
                    $(Self::$marked => $mark.to_owned(),)*
                    $(Self::$worded => strings
                        .say(&$word.key(), &Filling::nothing())
                        .into_text(),)*
                }
            }
        }
    };
}

keys! {
    marked {
        A => "A",
        B => "B",
        C => "C",
        D => "D",
        E => "E",
        F => "F",
        G => "G",
        H => "H",
        I => "I",
        J => "J",
        K => "K",
        L => "L",
        M => "M",
        N => "N",
        O => "O",
        P => "P",
        Q => "Q",
        R => "R",
        S => "S",
        T => "T",
        U => "U",
        V => "V",
        W => "W",
        X => "X",
        Y => "Y",
        Z => "Z",
        Digit0 => "0",
        Digit1 => "1",
        Digit2 => "2",
        Digit3 => "3",
        Digit4 => "4",
        Digit5 => "5",
        Digit6 => "6",
        Digit7 => "7",
        Digit8 => "8",
        Digit9 => "9",
        F1 => "F1",
        F2 => "F2",
        F3 => "F3",
        F4 => "F4",
        F5 => "F5",
        F6 => "F6",
        F7 => "F7",
        F8 => "F8",
        F9 => "F9",
        F10 => "F10",
        F11 => "F11",
        F12 => "F12",
        Comma => ",",
        Period => ".",
        Slash => "/",
        Minus => "-",
        Equals => "=",
    }
    worded {
        Space => words::SPACE,
        Tab => words::TAB,
        Enter => words::ENTER,
        Escape => words::ESCAPE,
        Backspace => words::BACKSPACE,
        Delete => words::DELETE,
        Insert => words::INSERT,
        Home => words::HOME,
        End => words::END,
        PageUp => words::PAGE_UP,
        PageDown => words::PAGE_DOWN,
        Left => words::LEFT,
        Right => words::RIGHT,
        Up => words::UP,
        Down => words::DOWN,
        Print => words::PRINT,
    }
}

impl Key {
    /// What this key is called, in the language the person reads — for the
    /// sixteen that print a word.
    ///
    /// `None` for a key that prints a mark, because there is nothing there to
    /// translate and a `Said` about one would be a claim that somebody should
    /// have.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Option<Said> {
        self.word()
            .map(|word| strings.say(&word.key(), &Filling::nothing()))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use std::collections::BTreeSet;

    /// Every key can be shown, and no two of them are shown the same way. A
    /// picker with two rows reading `F1` is a picker nobody can use.
    #[test]
    fn every_key_is_shown_in_a_way_of_its_own() {
        let strings = in_english();
        let mut shown = BTreeSet::new();
        for key in Key::ALL {
            let drawn = key.shown(&strings);
            assert!(!drawn.is_empty(), "{key:?}");
            assert!(shown.insert(drawn.clone()), "two keys are both {drawn}");
        }
        assert_eq!(shown.len(), Key::ALL.len());
    }

    /// **Every key is one kind or the other, and never both.** The macro writes
    /// both answers from two lists that cannot overlap, and this is what says
    /// the two lists together are the whole enum.
    #[test]
    fn a_key_prints_a_mark_or_a_word_and_never_both() {
        let mut marks = 0_usize;
        let mut words = 0_usize;
        for key in Key::ALL {
            let prints_a_mark = key.mark().is_some();
            assert_eq!(
                prints_a_mark,
                key.word().is_none(),
                "{key:?} prints {:?} and is called {:?}",
                key.mark(),
                key.word()
            );
            if prints_a_mark {
                marks = marks.saturating_add(1);
            } else {
                words = words.saturating_add(1);
            }
        }
        assert_eq!(marks, 53);
        assert_eq!(words, 16);
        assert_eq!(marks + words, Key::ALL.len());
    }

    /// The list is one list: the array a panel walks and the variants that
    /// exist cannot come apart, because the macro writes both from the same
    /// line.
    #[test]
    fn the_array_holds_each_key_once() {
        let unique: BTreeSet<Key> = Key::ALL.iter().copied().collect();
        assert_eq!(unique.len(), Key::ALL.len());
        assert!(Key::ALL.contains(&Key::Q));
        assert!(Key::ALL.contains(&Key::PageDown));
    }

    /// A settings file holds the name, not the number, so a key means the same
    /// thing after a release that adds one.
    #[test]
    fn a_settings_file_holds_the_name() {
        assert_eq!(serde_json::to_string(&Key::PageUp).unwrap(), r#""PageUp""#);
        assert_eq!(
            serde_json::from_str::<Key>(r#""Digit4""#).unwrap(),
            Key::Digit4
        );
        assert!(serde_json::from_str::<Key>(r#""AnyOldKey""#).is_err());
    }

    /// What a person reads is the character on the key, and for the keys that
    /// print nothing, the word for it.
    #[test]
    fn a_key_is_shown_as_it_is_marked() {
        let strings = in_english();
        assert_eq!(Key::Q.shown(&strings), "Q");
        assert_eq!(Key::Digit7.shown(&strings), "7");
        assert_eq!(Key::Comma.shown(&strings), ",");
        assert_eq!(Key::F4.shown(&strings), "F4");
        assert_eq!(Key::PageDown.shown(&strings), "Page Down");
    }

    /// **A key that prints a word is read in the person's own language**, and a
    /// key that prints a mark is the same mark in every one of them — which is
    /// the whole of the division this file makes.
    #[test]
    fn the_words_are_translated_and_the_marks_are_not() {
        let strings = translated(&[(words::DELETE, "Entf"), (words::HOME, "Pos1")]);
        assert_eq!(Key::Delete.shown(&strings), "Entf");
        assert_eq!(Key::Home.shown(&strings), "Pos1");
        assert!(
            Key::Delete
                .said(&strings)
                .is_some_and(|s| s.is_translated())
        );

        // The letter is the letter. Nothing was translated, because there is
        // nothing there for anybody to translate.
        assert_eq!(Key::Q.shown(&strings), "Q");
        assert_eq!(Key::Q.said(&strings), None);
        assert_eq!(Key::Q.word(), None);
    }

    /// Every word this file names is one the vocabulary declares. A key
    /// pointing at a string nothing declares would draw its own key in a
    /// picker, marked as a bug, and nothing would fail until somebody looked.
    #[test]
    fn every_word_a_key_names_is_declared() {
        let strings = in_english();
        for key in Key::ALL {
            let Some(said) = key.said(&strings) else {
                continue;
            };
            assert!(!said.is_a_bug(), "{key:?}: {said}");
            assert_eq!(Some(said.text()), key.word().map(|word| word.says()));
        }
    }
}
