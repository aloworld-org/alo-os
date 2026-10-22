//! **What one key press means while a person is filling in a provider**, which
//! is not what the same press means anywhere else in Settings.
//!
//! `crate::settings_keys` opens with a sentence this file exists to qualify:
//! *nothing is a letter, so nothing types*. That was true of every row in
//! Settings and is the reason a person could not add a provider — the window
//! had `Previous`, `Next`, `Choose`, `Close` and no way to enter a character.
//!
//! # Two readings of one press, and the mode decides
//!
//! The same keys mean opposite things in the two states, which is why this is
//! a separate reader rather than more arms on [`crate::SettingsKey`]:
//!
//! | press | a row | a field |
//! |---|---|---|
//! | space | **act on the row** | a space |
//! | backspace | put the shortcut back as shipped | rub out one letter |
//! | Enter | choose | **save**, when every field is filled |
//! | Escape | close the window | give up on the form, leave Settings open |
//!
//! A single enum answering both would be an enum whose meaning depends on a
//! flag somebody remembered to pass, and the press that goes wrong is
//! backspace: read as the wrong one, it puts a person's shortcut back to what
//! it ships with while they thought they were correcting a typo in a key.
//!
//! This is `crate::composer_keys`' shape, and for its reason: the alo Bar
//! already has one reader for *the keyboard is in a field* and another for
//! everywhere else.
//!
//! # Escape gives up on the form and does not close Settings
//!
//! A person who pressed Escape to get out of a half-typed key does not mean
//! *and throw away everything else I came here for*. The window stays; the
//! form goes.

use smithay::input::keyboard::{Keysym, ModifiersState};

/// One key press, while a person is filling in a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillingInKey {
    /// A character to put in the field the keyboard is in.
    Typed(char),
    /// Take back the last thing typed. Backspace.
    RubOut,
    /// The next field. Tab, or Down.
    NextField,
    /// The field before. Shift+Tab, or Up.
    FieldBefore,
    /// Save it. Enter.
    Save,
    /// Put what is on the clipboard into this field. Control+V.
    ///
    /// **Its own press rather than typing**, because what arrives is not a key
    /// press at all: the selection belongs to whichever client owns it, and
    /// reading it is a conversation over the bus that cannot be answered in
    /// the time one press takes.
    Paste,
    /// Give up on the form. Escape. **Settings stays open.**
    GiveUp,
    /// A press that means nothing here.
    Nothing,
}

impl FillingInKey {
    /// What this press means while a field has the keyboard.
    ///
    /// `typed` is what the seat's own XKB state made of the press — the
    /// character for this person's layout, which is the one thing this file
    /// must not work out for itself. A `ß`, a `é` or a Devanagari letter is
    /// whatever their keyboard says it is.
    #[must_use]
    pub fn of(key: Keysym, held: ModifiersState, typed: Option<char>) -> Self {
        // **Control+V before anything else**, because `v` is also a letter and
        // the chord has to be read as a chord. The logo key and Alt are not
        // this form's: a chord meant for the shell must not land in a field as
        // a character.
        if held.ctrl && matches!(key, Keysym::v | Keysym::V) {
            return Self::Paste;
        }
        if held.ctrl || held.alt || held.logo {
            return Self::Nothing;
        }
        match key {
            Keysym::BackSpace => Self::RubOut,
            Keysym::Tab if held.shift => Self::FieldBefore,
            Keysym::ISO_Left_Tab => Self::FieldBefore,
            Keysym::Tab | Keysym::Down | Keysym::KP_Down => Self::NextField,
            Keysym::Up | Keysym::KP_Up => Self::FieldBefore,
            Keysym::Return | Keysym::KP_Enter => Self::Save,
            Keysym::Escape => Self::GiveUp,
            // **The character the seat made, and nothing worked out here.**
            // A control character is not typing and is refused by the form
            // itself, so both ends agree without this file deciding what a
            // character is.
            _ => typed.map_or(Self::Nothing, Self::Typed),
        }
    }
}

#[cfg(test)]
#[path = "a_key_fills_in_a_provider_tests.rs"]
mod tests;
