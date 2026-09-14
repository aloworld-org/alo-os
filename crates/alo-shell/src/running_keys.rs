//! What one key press means while the window of what is running is open.
//!
//! It moves the view, asks for a new reading, and closes — and nothing else.
//! There is no key that stops, pauses or signals a process: *what is using the
//! machine* is a measurement, and acting on it would be a verb that needs a
//! grant under ADR 0001, which a window a person glances at is not.

use smithay::input::keyboard::{Keysym, ModifiersState};

/// One key press, as the window of what is running understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunningKey {
    /// Move the view one row up. Up.
    Up,
    /// Move the view one row down. Down.
    Down,
    /// Move the view to the first row. Home.
    First,
    /// Move the view to the last row. End.
    Last,
    /// Ask for a new reading. F5.
    ReadAgain,
    /// Close the window. Escape.
    Close,
    /// Anything else, which does nothing at all.
    Nothing,
}

impl RunningKey {
    /// What a symbol means, with the modifiers that were held.
    ///
    /// A chord — Control, Alt or the logo key held — is **nothing**, so a
    /// shortcut meant for something else never moves or closes the window.
    #[must_use]
    pub fn of(symbol: Keysym, held: &ModifiersState) -> Self {
        if held.ctrl || held.alt || held.logo {
            return Self::Nothing;
        }
        match symbol {
            Keysym::Up | Keysym::KP_Up => Self::Up,
            Keysym::Down | Keysym::KP_Down => Self::Down,
            Keysym::Home | Keysym::KP_Home => Self::First,
            Keysym::End | Keysym::KP_End => Self::Last,
            Keysym::F5 => Self::ReadAgain,
            Keysym::Escape => Self::Close,
            _ => Self::Nothing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The keys that do something do it**, and a chord does none of it.
    #[test]
    fn the_keys_that_do_something_and_a_chord_that_does_nothing() {
        let held = ModifiersState::default();
        for (symbol, meant) in [
            (Keysym::Up, RunningKey::Up),
            (Keysym::Down, RunningKey::Down),
            (Keysym::Home, RunningKey::First),
            (Keysym::End, RunningKey::Last),
            (Keysym::F5, RunningKey::ReadAgain),
            (Keysym::Escape, RunningKey::Close),
        ] {
            assert_eq!(RunningKey::of(symbol, &held), meant, "{symbol:?}");
            let chord = ModifiersState {
                ctrl: true,
                ..ModifiersState::default()
            };
            assert_eq!(RunningKey::of(symbol, &chord), RunningKey::Nothing);
        }
    }

    /// **No key acts on a process.** Delete, `k` for kill, Enter and the letters
    /// a task manager elsewhere uses do nothing here.
    #[test]
    fn no_key_acts_on_a_process() {
        let held = ModifiersState::default();
        for symbol in [
            Keysym::Delete,
            Keysym::k,
            Keysym::K,
            Keysym::Return,
            Keysym::s,
            Keysym::space,
            Keysym::BackSpace,
            Keysym::F9,
        ] {
            assert_eq!(
                RunningKey::of(symbol, &held),
                RunningKey::Nothing,
                "{symbol:?}"
            );
        }
    }
}
