//! What one key press means while the record window is open.
//!
//! The seat's own XKB state has already turned the press into a symbol
//! (`crate::record_seat`); this is the short list of what the window does with
//! one. It moves through the account, reads the record again, and closes — and
//! nothing else, because the window is a reading of the record and nothing on
//! it can change the record, narrow it or word it:
//!
//! - **no letters**, so there is nowhere to type a search, and no search that
//!   could change what *today* means;
//! - **nothing that hides a line**, so no key could leave a refusal out of what
//!   is in front of the person;
//! - **nothing that writes**, because there is nothing on the record a person
//!   could edit from a screen that shows it.

use smithay::input::keyboard::{Keysym, ModifiersState};

/// One key press, as the record window understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKey {
    /// Move the view one entry towards what happened most recently. Up.
    Newer,
    /// Move the view one entry towards what happened longest ago. Down.
    Older,
    /// Move the view to the most recent entry. Home.
    Newest,
    /// Move the view to the entry longest ago. End.
    Oldest,
    /// Read the record off the disk again. F5.
    ReadAgain,
    /// Close the window. Escape.
    Close,
    /// Anything else, which does nothing at all.
    Nothing,
}

impl RecordKey {
    /// What a symbol means, with the modifiers that were held.
    ///
    /// Up and Down move through time rather than across the page, so their
    /// meaning does not depend on which way the person reads. A chord —
    /// Control, Alt or the logo key held — is **nothing**, so a shortcut meant
    /// for something else never moves or closes the window.
    #[must_use]
    pub fn of(symbol: Keysym, held: &ModifiersState) -> Self {
        if held.ctrl || held.alt || held.logo {
            return Self::Nothing;
        }
        match symbol {
            Keysym::Up | Keysym::KP_Up => Self::Newer,
            Keysym::Down | Keysym::KP_Down => Self::Older,
            Keysym::Home | Keysym::KP_Home => Self::Newest,
            Keysym::End | Keysym::KP_End => Self::Oldest,
            Keysym::F5 => Self::ReadAgain,
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
            (Keysym::Up, RecordKey::Newer),
            (Keysym::KP_Up, RecordKey::Newer),
            (Keysym::Down, RecordKey::Older),
            (Keysym::KP_Down, RecordKey::Older),
            (Keysym::Home, RecordKey::Newest),
            (Keysym::End, RecordKey::Oldest),
            (Keysym::F5, RecordKey::ReadAgain),
            (Keysym::Escape, RecordKey::Close),
        ] {
            assert_eq!(RecordKey::of(symbol, &held), meant, "{symbol:?}");
        }
    }

    /// **No key types, deletes or narrows.** Letters, Delete, Backspace and
    /// Enter do nothing, so there is no search, no filter and no edit.
    #[test]
    fn no_key_types_deletes_or_narrows() {
        let held = nothing_held();
        for symbol in [
            Keysym::a,
            Keysym::slash,
            Keysym::f,
            Keysym::Delete,
            Keysym::BackSpace,
            Keysym::Return,
            Keysym::Tab,
            Keysym::space,
            Keysym::NoSymbol,
        ] {
            assert_eq!(
                RecordKey::of(symbol, &held),
                RecordKey::Nothing,
                "{symbol:?}"
            );
        }
    }

    /// **A chord does nothing**, including Control+F, which elsewhere is a
    /// search.
    #[test]
    fn a_chord_does_nothing() {
        for held in [
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
        ] {
            for symbol in [Keysym::f, Keysym::Escape, Keysym::Down, Keysym::F5] {
                assert_eq!(RecordKey::of(symbol, &held), RecordKey::Nothing);
            }
        }
    }
}
