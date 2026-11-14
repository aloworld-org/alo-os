//! The words on the compositor's door for **alo typing into the window in
//! front** —
//! [ADR 0153](../../../docs/decisions/0153-alo-types-by-keys-in-its-own-copy-and-reads-the-text-back.md)
//! and `docs/contracts/application-door.md`, *`type-into` and `save-in`*.
//!
//! ```text
//! → type-into com.vscodium.codium 61
//! → <exactly 61 bytes of UTF-8: the text>
//! ← typed
//!   (or: not-in-front, not-alos-copy, no-keyboard, cannot-type U+0007)
//! → save-in com.vscodium.codium
//! ← asked-to-save
//!   (or: not-in-front, not-alos-copy, no-keyboard)
//! ```
//!
//! **The length is said, not the end.** The text has no maximum (ADR 0001's
//! amendment of 2026-11-13) and holds new lines, so it cannot be one line of
//! this door; the line says how many bytes follow, and the door reads exactly
//! that many.

use alo_applications::Application;

use crate::request::Request;

/// The word that begins typing a text.
pub const TYPE_INTO: &str = "type-into";

/// The word that asks the window in front to save.
pub const SAVE_IN: &str = "save-in";

/// The word that asks whether the window in front may be typed into, and
/// sends it nothing — for typing that goes through the accessibility tree
/// rather than the keyboard, which must meet the same rule.
pub const MAY_TYPE_INTO: &str = "may-type-into";

/// alo asking the window in front to be typed into, or to save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToType {
    /// Type a text of this many bytes, which follows the line.
    Text {
        /// The application whose window must have the keyboard.
        application: Application,
        /// How many bytes of text follow.
        bytes: usize,
    },
    /// Press the application's own *save*: `Ctrl+S`.
    Save {
        /// The application whose window must have the keyboard.
        application: Application,
    },
    /// Only ask whether the window in front may be typed into; send nothing.
    Check {
        /// The application whose window must have the keyboard.
        application: Application,
    },
}

impl ToType {
    /// The application whose window must have the keyboard.
    #[must_use]
    pub const fn application(&self) -> &Application {
        match self {
            Self::Text { application, .. }
            | Self::Save { application }
            | Self::Check { application } => application,
        }
    }

    /// The line, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        match self {
            Self::Text { application, bytes } => {
                format!("{TYPE_INTO} {} {bytes}", application.identifier())
            }
            Self::Save { application } => format!("{SAVE_IN} {}", application.identifier()),
            Self::Check { application } => {
                format!("{MAY_TYPE_INTO} {}", application.identifier())
            }
        }
    }

    /// One line, read — [`None`] when it is not about typing at all, so the
    /// door can read it as one of the other acts.
    #[must_use]
    pub fn read(line: &str) -> Option<Result<Self, NotToType>> {
        let line = line.trim_end_matches(['\r', '\n']);
        let mut words = line.split(' ');
        let first = words.next()?;
        if first != TYPE_INTO && first != SAVE_IN && first != MAY_TYPE_INTO {
            return None;
        }
        if line.len() > Request::THE_MOST {
            return Some(Err(NotToType::TooLong));
        }
        let Some(named) = words.next() else {
            return Some(Err(NotToType::NotOneApplication));
        };
        let Ok(application) = Application::identified(named) else {
            return Some(Err(NotToType::NotOneApplication));
        };
        if first == SAVE_IN || first == MAY_TYPE_INTO {
            return Some(match (words.next(), first == SAVE_IN) {
                (None, true) => Ok(Self::Save { application }),
                (None, false) => Ok(Self::Check { application }),
                (Some(_), _) => Err(NotToType::NotOneApplication),
            });
        }
        let (Some(bytes), None) = (words.next(), words.next()) else {
            return Some(Err(NotToType::NoLength));
        };
        // Digits only: `+5` and ` 5` are not a length a caller meant.
        if bytes.is_empty() || !bytes.bytes().all(|byte| byte.is_ascii_digit()) {
            return Some(Err(NotToType::NoLength));
        }
        Some(match bytes.parse::<usize>() {
            Ok(0) | Err(_) => Err(NotToType::NoLength),
            Ok(bytes) => Ok(Self::Text { application, bytes }),
        })
    }
}

/// Why a line that began as typing is not one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotToType {
    /// Longer than any line on this door may be.
    #[error("a line on this door is at most {} bytes", Request::THE_MOST)]
    TooLong,
    /// It named no application, more than one, or one no verb could name.
    #[error("typing names exactly one application, by its identifier")]
    NotOneApplication,
    /// `type-into` did not say how many bytes follow, as a number above zero.
    #[error("type-into says how many bytes of text follow, as a number above zero")]
    NoLength,
}

/// What the door answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Typed {
    /// Every key of the text was delivered to the window in front.
    Typed,
    /// `Ctrl+S` was delivered to the window in front.
    AskedToSave,
    /// The window in front may be typed into; nothing was sent.
    MayType,
    /// The window that has the keyboard is not one of that application's, or
    /// no window has it. Nothing was sent.
    NotInFront,
    /// It is the person's own window of that application, and at their
    /// setting alo works only in its own copy. Nothing was sent.
    NotAlosCopy,
    /// This desktop has no keyboard to deliver through.
    NoKeyboard,
    /// The text holds a character no key carries. Nothing was typed.
    CannotType(char),
}

impl Typed {
    /// The words on the wire.
    #[must_use]
    pub fn written(self) -> String {
        match self {
            Self::Typed => "typed".to_owned(),
            Self::AskedToSave => "asked-to-save".to_owned(),
            Self::MayType => "may".to_owned(),
            Self::NotInFront => "not-in-front".to_owned(),
            Self::NotAlosCopy => "not-alos-copy".to_owned(),
            Self::NoKeyboard => "no-keyboard".to_owned(),
            Self::CannotType(character) => {
                format!("cannot-type U+{:04X}", u32::from(character))
            }
        }
    }

    /// One answer, read — [`None`] for a line that is not one.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        let line = line.trim_end_matches(['\r', '\n']);
        Some(match line {
            "typed" => Self::Typed,
            "asked-to-save" => Self::AskedToSave,
            "may" => Self::MayType,
            "not-in-front" => Self::NotInFront,
            "not-alos-copy" => Self::NotAlosCopy,
            "no-keyboard" => Self::NoKeyboard,
            other => {
                let point = other.strip_prefix("cannot-type U+")?;
                if point.is_empty() || !point.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return None;
                }
                Self::CannotType(char::from_u32(u32::from_str_radix(point, 16).ok()?)?)
            }
        })
    }
}

/// The first character of `text` that no key can carry — a control character
/// other than a new line or a tab — or [`None`] when every one can be typed.
#[must_use]
pub fn the_first_untypable(text: &str) -> Option<char> {
    text.chars()
        .find(|character| character.is_control() && !matches!(character, '\n' | '\t'))
}

#[cfg(test)]
#[path = "typing_into_tests.rs"]
mod tests;
