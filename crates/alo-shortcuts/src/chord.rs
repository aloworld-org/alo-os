//! One key combination, and the three it refuses to be.
//!
//! A [`Chord`] is what a person presses: some modifiers, and one key. It cannot
//! be built out of anything else, and three combinations cannot be built at all.
//!
//! **Every chord holds Super, Ctrl or Alt.** A system shortcut is a key the
//! compositor takes *before* any application sees it, so every one of them is a
//! key taken away from every application on the machine — which is why this list
//! is short, and why a bare `Q` is not a shortcut this model can express. Shift
//! does not count: `Shift+2` is `@`, and the difference between a modifier that
//! changes what a key means and one that changes which character it prints is
//! the whole of [`Modifiers::enough`].
//!
//! **Copy, cut and paste cannot be taken.** `docs/features.md` promises at v0.01
//! that copy, cut and paste work across applications; a system shortcut on
//! `Ctrl+V` would break that promise everywhere at once, and the person who set
//! it would never connect the two. So the promise is a refusal here rather than
//! a sentence in a document. It is exactly those three: `Ctrl+Shift+C` is a
//! different chord and is nobody's clipboard.
//!
//! A chord is checked when it is made and checked again when it is read back —
//! [`Chord`] deserialises through [`Chord::checked`], so a settings file that
//! was hand-edited into `Q` is refused where it is read rather than believed.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::key::Key;
use crate::modifier::{Modifier, Modifiers};

/// Why a key combination cannot be a shortcut.
///
/// Each says what to do about it, because a person reading this is in the
/// middle of setting a shortcut and wants the next thing to try.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ChordError {
    /// Nothing was held down.
    #[error(
        "hold Super, Ctrl or Alt as well — a shortcut on {0} by itself would take that key away from everything you type"
    )]
    NothingHeld(Key),
    /// Only Shift was held down, which leaves the key printing a character.
    #[error("Shift only changes which character {0} prints — hold Super, Ctrl or Alt as well")]
    ShiftIsNotEnough(Key),
    /// One of the three chords the clipboard is worked by.
    #[error("Ctrl+{0} is how {1} works in every application — pick another key")]
    TheApplicationNeedsIt(Key, &'static str),
}

/// A key combination as a person presses it.
///
/// Serialises as its parts. Reading one back goes through [`Chord::checked`],
/// so nothing that arrives from a file is a chord this crate would have made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// [`ChordError`], which says what to press instead.
    pub fn checked(modifiers: Modifiers, key: Key) -> Result<Self, ChordError> {
        if modifiers.is_empty() {
            return Err(ChordError::NothingHeld(key));
        }
        if !modifiers.enough() {
            return Err(ChordError::ShiftIsNotEnough(key));
        }
        if let Some(what) = the_clipboard(modifiers, key) {
            return Err(ChordError::TheApplicationNeedsIt(key, what));
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
}

/// Which clipboard action a combination is, when it is one.
fn the_clipboard(modifiers: Modifiers, key: Key) -> Option<&'static str> {
    if modifiers != Modifiers::just(Modifier::Ctrl) {
        return None;
    }
    match key {
        Key::C => Some("copy"),
        Key::X => Some("cut"),
        Key::V => Some("paste"),
        _ => None,
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

impl TryFrom<Parts> for Chord {
    type Error = ChordError;

    fn try_from(parts: Parts) -> Result<Self, Self::Error> {
        Self::checked(parts.modifiers, parts.key)
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}+{}", self.modifiers, self.key)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

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
        assert_eq!(chord.to_string(), "Super+Shift+Tab");
    }

    /// **A bare key is refused.** Every system shortcut is a key taken away from
    /// every application, and a shortcut on `Q` would take Q.
    #[test]
    fn a_key_by_itself_is_not_a_shortcut() {
        let refused = Chord::checked(Modifiers::none(), Key::Q).unwrap_err();
        assert_eq!(refused, ChordError::NothingHeld(Key::Q));
        assert!(refused.to_string().contains("hold Super, Ctrl or Alt"));
    }

    /// **Shift is refused too**, and with a different sentence, because a person
    /// who held Shift did hold something and needs to be told why it was not
    /// enough.
    #[test]
    fn shift_and_a_key_is_still_the_character_that_key_prints() {
        let refused = Chord::checked(Modifiers::just(Modifier::Shift), Key::Digit2).unwrap_err();
        assert_eq!(refused, ChordError::ShiftIsNotEnough(Key::Digit2));
        assert!(refused.to_string().contains("only changes which character"));

        // Shift beside one of the other three is an ordinary chord.
        assert!(Chord::checked(zuper().and(Modifier::Shift), Key::Digit2).is_ok());
    }

    /// **The clipboard cannot be taken.** `docs/features.md` promises copy, cut
    /// and paste across applications at v0.01, and this is that promise as a
    /// refusal rather than as a sentence.
    #[test]
    fn the_three_the_clipboard_needs_are_refused() {
        for (key, what) in [(Key::C, "copy"), (Key::X, "cut"), (Key::V, "paste")] {
            let refused = Chord::checked(Modifiers::just(Modifier::Ctrl), key).unwrap_err();
            assert_eq!(refused, ChordError::TheApplicationNeedsIt(key, what));
            assert!(refused.to_string().contains(what), "{refused}");
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
    #[test]
    fn a_chord_from_a_file_goes_through_the_same_door() {
        let chord = Chord::checked(zuper(), Key::Q).unwrap();
        let written = serde_json::to_string(&chord).unwrap();
        assert_eq!(written, r#"{"modifiers":["Super"],"key":"Q"}"#);
        assert_eq!(serde_json::from_str::<Chord>(&written).unwrap(), chord);

        for hand_edited in [
            r#"{"modifiers":[],"key":"Q"}"#,
            r#"{"modifiers":["Shift"],"key":"Q"}"#,
            r#"{"modifiers":["Ctrl"],"key":"V"}"#,
        ] {
            let refused = serde_json::from_str::<Chord>(hand_edited).unwrap_err();
            assert!(!refused.to_string().is_empty(), "{hand_edited}");
        }
    }
}
