//! One key combination, and what a person reads it as.
//!
//! A [`Chord`] is what a person presses: some modifiers, and one key. It cannot
//! be built out of anything else, and three combinations cannot be built at all
//! — those are [`crate::refusing`], which holds the three refusals and the
//! words they are said in.
//!
//! **Every chord holds Super, Ctrl or Alt.** A system shortcut is a key the
//! compositor takes *before* any application sees it, so every one of them is a
//! key taken away from every application on the machine — which is why the list
//! of actions is short, and why a bare `Q` is not a shortcut this model can
//! express. Shift does not count: `Shift+2` is `@`, and the difference between a
//! modifier that changes what a key means and one that changes which character
//! it prints is the whole of [`Modifiers::enough`].
//!
//! **Copy, cut and paste cannot be taken.** `docs/features.md` promises at
//! v0.01 that copy, cut and paste work across applications; a system shortcut on
//! `Ctrl+V` would break that promise everywhere at once, and the person who set
//! it would never connect the two. So the promise is a refusal rather than a
//! sentence in a document. It is exactly those three: `Ctrl+Shift+C` is a
//! different chord and is nobody's clipboard.
//!
//! A chord is checked when it is made and checked again when it is read back —
//! [`Chord`] deserialises through [`Chord::checked`], so a settings file that
//! was hand-edited into `Q` is refused where it is read rather than believed.
//!
//! # A chord is read, not printed
//!
//! There is no `Display`. [`Chord::shown`] writes the chord in the language the
//! person reads — `Strg+Entf` on a German machine — and the derived `Debug`
//! writes the names a settings file holds, which are nobody's language. The two
//! are for two different readers and neither is the other's fallback.

use std::fmt;

use alo_strings::Strings;
use serde::{Deserialize, Serialize};

use crate::key::Key;
use crate::modifier::Modifiers;
use crate::refusing::{ChordError, the_clipboard};

/// A key combination as a person presses it.
///
/// Serialises as its parts. Reading one back goes through [`Chord::checked`],
/// so nothing that arrives from a file is a chord this crate would have made.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Parts")]
pub struct Chord {
    /// What is held down.
    modifiers: Modifiers,
    /// What is pressed.
    key: Key,
}

impl Chord {
    /// Check a combination somebody has just pressed in a settings panel.
    ///
    /// # Errors
    /// [`ChordError`], which says what to press instead — in the language the
    /// person reads, through `ChordError::said`.
    pub fn checked(modifiers: Modifiers, key: Key) -> Result<Self, ChordError> {
        if modifiers.is_empty() {
            return Err(ChordError::NothingHeld(key));
        }
        if !modifiers.enough() {
            return Err(ChordError::ShiftIsNotEnough(key));
        }
        if let Some(clipboard) = the_clipboard(modifiers, key) {
            return Err(ChordError::TheClipboardNeedsIt(clipboard));
        }
        Ok(Self { modifiers, key })
    }

    /// A chord the compiler can build, for the list this crate ships.
    ///
    /// Unchecked, and the only caller is [`crate::defaults`] — which is held to
    /// the same rules by a test that puts every shipped default back through
    /// [`Chord::checked`].
    pub(crate) const fn shipped(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    /// What is held down.
    #[must_use]
    pub fn modifiers(self) -> Modifiers {
        self.modifiers
    }

    /// What is pressed.
    #[must_use]
    pub fn key(self) -> Key {
        self.key
    }

    /// The chord as a person reads it: what is held, then what is pressed,
    /// joined with `+`, every name in the language they read.
    #[must_use]
    pub fn shown(self, strings: &Strings) -> String {
        let mut written = self.modifiers.shown(strings);
        if !written.is_empty() {
            written.push('+');
        }
        written.push_str(&self.key.shown(strings));
        written
    }
}

impl fmt::Debug for Chord {
    /// `Super+Left` — the names a settings file holds, which is what a
    /// programmer reading a contradictory set of defaults needs and is not a
    /// sentence in anybody's language.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.is_empty() {
            return write!(f, "{:?}", self.key);
        }
        write!(f, "{:?}+{:?}", self.modifiers, self.key)
    }
}

/// A chord as it sits in a settings file, before anything has been checked.
#[derive(Deserialize)]
struct Parts {
    /// What was held down.
    modifiers: Modifiers,
    /// What was pressed.
    key: Key,
}

/// Why a chord in a settings file did not read.
///
/// **It is not a sentence, and that is the point.** There is no `Strings` at the
/// bottom of a deserialiser — nothing there knows which language this machine is
/// read in — so a message composed here would be English that nothing could
/// translate, which is exactly what [`ChordError`] losing its `Display` was for.
/// What it writes instead is the *key* of the refusal, which is what whoever
/// shows this to a person looks up to get the same words a settings panel shows
/// for the same refusal.
struct NotAChord(ChordError);

impl fmt::Display for NotAChord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.word().key())
    }
}

impl TryFrom<Parts> for Chord {
    type Error = NotAChord;

    fn try_from(parts: Parts) -> Result<Self, Self::Error> {
        Self::checked(parts.modifiers, parts.key).map_err(NotAChord)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::modifier::Modifier;
    use crate::refusing::Clipboard;
    use crate::testing::{in_english, translated};
    use crate::words;

    fn zuper() -> Modifiers {
        Modifiers::just(Modifier::Super)
    }

    /// The ordinary path: something is held, one key is pressed, and the chord
    /// remembers both.
    #[test]
    fn a_chord_is_what_was_held_and_what_was_pressed() {
        let chord = Chord::checked(zuper().and(Modifier::Shift), Key::Tab).unwrap();
        assert_eq!(chord.modifiers(), zuper().and(Modifier::Shift));
        assert_eq!(chord.key(), Key::Tab);
        assert_eq!(chord.shown(&in_english()), "Super+Shift+Tab");
    }

    /// **A chord is read in the language the person reads.** Everything in it
    /// is a string of its own — three modifier names and one key — so a machine
    /// with a German shell shows the keys a German keyboard is printed with.
    #[test]
    fn a_chord_is_read_in_the_persons_own_language() {
        let strings = translated(&[
            (words::CTRL, "Strg"),
            (words::SHIFT, "Umschalt"),
            (words::DELETE, "Entf"),
        ]);
        let chord = Chord::checked(
            Modifiers::just(Modifier::Ctrl).and(Modifier::Shift),
            Key::Delete,
        )
        .unwrap();
        assert_eq!(chord.shown(&strings), "Strg+Umschalt+Entf");

        // A key that prints a mark is the same mark in every language.
        let letter = Chord::checked(Modifiers::just(Modifier::Ctrl), Key::B).unwrap();
        assert_eq!(letter.shown(&strings), "Strg+B");
    }

    /// **A bare key is refused.** Every system shortcut is a key taken away from
    /// every application, and a shortcut on `Q` would take Q.
    #[test]
    fn a_key_by_itself_is_not_a_shortcut() {
        let refused = Chord::checked(Modifiers::none(), Key::Q).unwrap_err();
        assert_eq!(refused, ChordError::NothingHeld(Key::Q));
        assert!(
            refused
                .said(&in_english())
                .text()
                .contains("hold Super, Ctrl or Alt")
        );
    }

    /// **Shift is refused too**, and with a different sentence, because a person
    /// who held Shift did hold something and needs to be told why it was not
    /// enough.
    #[test]
    fn shift_and_a_key_is_still_the_character_that_key_prints() {
        let refused = Chord::checked(Modifiers::just(Modifier::Shift), Key::Digit2).unwrap_err();
        assert_eq!(refused, ChordError::ShiftIsNotEnough(Key::Digit2));
        assert!(
            refused
                .said(&in_english())
                .text()
                .contains("only changes which character")
        );

        // Shift beside one of the other three is an ordinary chord.
        assert!(Chord::checked(zuper().and(Modifier::Shift), Key::Digit2).is_ok());
    }

    /// **The clipboard cannot be taken.** `docs/features.md` promises copy, cut
    /// and paste across applications at v0.01, and this is that promise as a
    /// refusal rather than as a sentence.
    #[test]
    fn the_three_the_clipboard_needs_are_refused() {
        let strings = in_english();
        for (key, clipboard, what) in [
            (Key::C, Clipboard::Copy, "copying"),
            (Key::X, Clipboard::Cut, "cutting"),
            (Key::V, Clipboard::Paste, "pasting"),
        ] {
            let refused = Chord::checked(Modifiers::just(Modifier::Ctrl), key).unwrap_err();
            assert_eq!(refused, ChordError::TheClipboardNeedsIt(clipboard));
            let said = refused.said(&strings);
            assert!(said.text().contains(what), "{said}");
        }
    }

    /// It is those three chords and not those three keys: a person may still
    /// have `Ctrl+Shift+V` or `Super+C`, which no application's clipboard uses.
    #[test]
    fn a_different_chord_on_the_same_key_is_nobodys_clipboard() {
        assert!(
            Chord::checked(Modifiers::just(Modifier::Ctrl).and(Modifier::Shift), Key::V).is_ok()
        );
        assert!(Chord::checked(zuper(), Key::C).is_ok());
        assert!(Chord::checked(Modifiers::just(Modifier::Alt), Key::X).is_ok());
    }

    /// A chord read back off a disk is checked exactly as one typed into a
    /// settings panel is, so a file nobody validated cannot introduce a chord
    /// this crate would have refused.
    ///
    /// **And what it says is the key of the refusal**, not a sentence: whoever
    /// reports a settings file that did not read looks that key up and shows
    /// the same words the panel would have shown.
    #[test]
    fn a_chord_from_a_file_goes_through_the_same_door() {
        let chord = Chord::checked(zuper(), Key::Q).unwrap();
        let written = serde_json::to_string(&chord).unwrap();
        assert_eq!(written, r#"{"modifiers":["Super"],"key":"Q"}"#);
        assert_eq!(serde_json::from_str::<Chord>(&written).unwrap(), chord);

        for (hand_edited, key) in [
            (
                r#"{"modifiers":[],"key":"Q"}"#,
                "shortcuts.chord.nothing-held",
            ),
            (
                r#"{"modifiers":["Shift"],"key":"Q"}"#,
                "shortcuts.chord.shift-is-not-enough",
            ),
            (
                r#"{"modifiers":["Ctrl"],"key":"V"}"#,
                "shortcuts.chord.the-clipboard-pastes",
            ),
        ] {
            let refused = serde_json::from_str::<Chord>(hand_edited).unwrap_err();
            assert!(refused.to_string().contains(key), "{refused}");
        }
    }

    /// A programmer reads the names the file holds, which do not move when
    /// somebody contributes a language.
    #[test]
    fn a_programmer_reads_the_names_the_file_holds() {
        let chord = Chord::checked(zuper().and(Modifier::Shift), Key::Tab).unwrap();
        assert_eq!(format!("{chord:?}"), "Super+Shift+Tab");
        assert_eq!(
            format!("{:?}", Chord::shipped(Modifiers::none(), Key::Q)),
            "Q"
        );
    }
}
