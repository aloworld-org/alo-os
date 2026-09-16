//! What one key press means while Settings is open.
//!
//! The seat's own XKB state has already turned the press into a symbol
//! (`crate::settings_seat`). Settings is operated whole from the keyboard,
//! and the list is short on purpose:
//!
//! - **Up, Down, Home, End** move through the rows; **Tab** and **Shift+Tab**
//!   move to the next and the previous section;
//! - **Enter** or **Space** acts on the row: chooses it, waits for a chord for
//!   a shortcut, or revokes a grant or a pairing;
//! - **Backspace** puts the focused shortcut back to what it ships with;
//! - **Delete** puts a section whose file did not read back as shipped — the
//!   one act that replaces such a file, and one the section is told it may
//!   take in its own crate's sentence;
//! - **Escape** closes the window.
//!
//! Nothing is a letter, so nothing types; and a chord with Control, Alt or the
//! logo key held is **nothing**, so a shortcut meant for something else never
//! revokes a grant or moves the dock. While Settings waits for a chord the
//! press is read as the chord instead (`crate::settings_chord`).

use smithay::input::keyboard::{Keysym, ModifiersState};

/// One key press, as Settings understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsKey {
    /// The row before. Up.
    Previous,
    /// The row after. Down.
    Next,
    /// The first row. Home.
    First,
    /// The last row. End.
    Last,
    /// The first row of the next section. Tab.
    NextSection,
    /// The first row of the section before. Shift+Tab.
    PreviousSection,
    /// Act on the row. Enter or Space.
    Choose,
    /// Put the focused shortcut back to what it ships with. Backspace.
    PutBackThisOne,
    /// Put the section back as shipped, when its file did not read. Delete.
    PutBackAsShipped,
    /// Close the window. Escape.
    Close,
    /// Anything else, which does nothing at all.
    Nothing,
}

impl SettingsKey {
    /// What a symbol means, with the modifiers that were held.
    #[must_use]
    pub fn of(symbol: Keysym, held: &ModifiersState) -> Self {
        if held.ctrl || held.alt || held.logo {
            return Self::Nothing;
        }
        match symbol {
            Keysym::Tab if held.shift => Self::PreviousSection,
            Keysym::ISO_Left_Tab => Self::PreviousSection,
            Keysym::Tab => Self::NextSection,
            _ if held.shift => Self::Nothing,
            Keysym::Up | Keysym::KP_Up => Self::Previous,
            Keysym::Down | Keysym::KP_Down => Self::Next,
            Keysym::Home | Keysym::KP_Home => Self::First,
            Keysym::End | Keysym::KP_End => Self::Last,
            Keysym::Return | Keysym::KP_Enter | Keysym::space => Self::Choose,
            Keysym::BackSpace => Self::PutBackThisOne,
            Keysym::Delete | Keysym::KP_Delete => Self::PutBackAsShipped,
            Keysym::Escape => Self::Close,
            _ => Self::Nothing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No modifiers held.
    fn nothing_held() -> ModifiersState {
        ModifiersState::default()
    }

    /// **The keys that do something do it.**
    #[test]
    fn the_keys_that_do_something() {
        let held = nothing_held();
        for (symbol, meant) in [
            (Keysym::Up, SettingsKey::Previous),
            (Keysym::Down, SettingsKey::Next),
            (Keysym::Home, SettingsKey::First),
            (Keysym::End, SettingsKey::Last),
            (Keysym::Tab, SettingsKey::NextSection),
            (Keysym::ISO_Left_Tab, SettingsKey::PreviousSection),
            (Keysym::Return, SettingsKey::Choose),
            (Keysym::space, SettingsKey::Choose),
            (Keysym::BackSpace, SettingsKey::PutBackThisOne),
            (Keysym::Delete, SettingsKey::PutBackAsShipped),
            (Keysym::Escape, SettingsKey::Close),
        ] {
            assert_eq!(SettingsKey::of(symbol, &held), meant, "{symbol:?}");
        }
        let shift = ModifiersState {
            shift: true,
            ..nothing_held()
        };
        assert_eq!(
            SettingsKey::of(Keysym::Tab, &shift),
            SettingsKey::PreviousSection
        );
    }

    /// **No letter types, and a chord does nothing** — Control+Enter does not
    /// revoke, Alt+Delete does not put a section back, and Shift+Enter is not
    /// Enter.
    #[test]
    fn no_letter_types_and_a_chord_does_nothing() {
        let held = nothing_held();
        for symbol in [Keysym::a, Keysym::slash, Keysym::F5, Keysym::NoSymbol] {
            assert_eq!(SettingsKey::of(symbol, &held), SettingsKey::Nothing);
        }
        for chord in [
            ModifiersState {
                ctrl: true,
                ..nothing_held()
            },
            ModifiersState {
                alt: true,
                ..nothing_held()
            },
            ModifiersState {
                logo: true,
                ..nothing_held()
            },
            ModifiersState {
                shift: true,
                ..nothing_held()
            },
        ] {
            for symbol in [Keysym::Return, Keysym::Delete, Keysym::Escape, Keysym::Down] {
                assert_eq!(
                    SettingsKey::of(symbol, &chord),
                    SettingsKey::Nothing,
                    "{symbol:?} with {chord:?}"
                );
            }
        }
    }
}
