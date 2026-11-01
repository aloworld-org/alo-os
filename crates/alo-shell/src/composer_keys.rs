//! What one key press means in the Dock's composer — *Find, open, or ask…*.
//!
//! The seat's XKB state has already turned the press into a symbol with the
//! person's layout and modifiers applied; this is the short list of what the
//! composer does with one. It is `crate::sign_in_keys`' shape, for the same
//! reason that file gives: a surface's keys are a closed list somebody can read
//! in one sitting, and a chord that typed its letter into a field would put a
//! character there the person did not mean.
//!
//! # Two keys that look like one
//!
//! **Enter asks. Shift+Enter adds a line.** The interaction contract, frame
//! `418:28202`. They arrive as the same symbol, so the modifier is the whole of
//! the difference and the order of the tests below is load-bearing.
//!
//! # And one key that must be refused while something else owns it
//!
//! **Enter must not ask while an input method is composing.** Somebody typing
//! Japanese, Chinese or Korean presses Enter to *accept the candidate they are
//! composing* — and a field that read that as *send* would fire a request off
//! to an agent in the middle of a word. The refusal is a parameter rather than
//! a comment, because a rule that depends on a caller remembering it is a rule
//! that will be forgotten in exactly the locales nobody here tests in.

use smithay::input::keyboard::{Keysym, ModifiersState};

/// Whether an input method is in the middle of composing a character.
///
/// **A named pair rather than a `bool` at the call site**, which is this
/// crate's rule wherever a yes-or-no fact reads alike in both directions:
/// `of(symbol, held, true)` cannot be read without going to look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Composing {
    /// No input method is composing; Enter means what it says.
    No,
    /// An input method is composing, and Enter belongs to it.
    InProgress,
}

/// One key press, as the composer understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposerKey {
    /// A letter to add to what has been typed.
    Letter(char),
    /// Take the last grapheme back.
    Erase,
    /// Add a line break without asking for anything.
    NewLine,
    /// **Ask alo for what has been typed.** Enter, with nothing held and no
    /// input method composing.
    Ask,
    /// Take back everything typed and leave the field.
    Clear,
    /// Up: the row above, in the launcher's list.
    Up,
    /// Down: the row below, in the launcher's list.
    Down,
    /// Anything else, which does nothing at all.
    Nothing,
}

impl ComposerKey {
    /// What a symbol means, with the modifiers held and the input method's
    /// state.
    ///
    /// The order is the rule:
    ///
    /// 1. **Shift+Enter is a line**, tested before Enter because they are one
    ///    symbol.
    /// 2. **Enter while composing is nothing**, tested before Enter asks, so a
    ///    candidate being accepted never sends a request.
    /// 3. **Enter asks.**
    /// 4. A letter held with Control, Alt or the logo key is **nothing**, not a
    ///    letter: those are chords.
    #[must_use]
    pub fn of(symbol: Keysym, held: &ModifiersState, composing: Composing) -> Self {
        match symbol {
            Keysym::BackSpace => return Self::Erase,
            Keysym::Escape => return Self::Clear,
            // **The launcher's ↑ ↓ *choose***, which its foot names
            // (`561:351`). Not a letter, so held keys make no difference.
            Keysym::Up | Keysym::KP_Up => return Self::Up,
            Keysym::Down | Keysym::KP_Down => return Self::Down,
            Keysym::Return | Keysym::KP_Enter => {
                if held.shift {
                    return Self::NewLine;
                }
                if matches!(composing, Composing::InProgress) {
                    return Self::Nothing;
                }
                return Self::Ask;
            }
            _ => {}
        }
        if held.ctrl || held.alt || held.logo {
            return Self::Nothing;
        }
        match symbol.key_char() {
            Some(letter) if !letter.is_control() => Self::Letter(letter),
            _ => Self::Nothing,
        }
    }
}

#[cfg(test)]
#[path = "composer_keys_tests.rs"]
mod tests;
