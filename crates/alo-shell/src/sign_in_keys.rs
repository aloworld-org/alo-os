//! What one key press means at the sign-in screen.
//!
//! The seat's own XKB state has already turned the press into a symbol with
//! the person's layout and modifiers applied (`crate::sign_in_seat`); this is
//! only the short list of what a sign-in screen does with a symbol. Five
//! things, and no sixth: there is no shortcut here that opens anything, runs
//! anything or reaches past the screen, because nobody is signed in to own
//! what it would reach.

use smithay::input::keyboard::{Keysym, ModifiersState};

/// One key press, as the sign-in screen understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignInKey {
    /// A letter to type into the field that is waiting.
    Letter(char),
    /// Take the last letter of the waiting field back.
    Erase,
    /// Move to the other field. Tab and Shift+Tab both do, because there are
    /// two fields.
    OtherField,
    /// Enter: from the name, move to the password; from the password, sign in.
    Enter,
    /// Anything else, which does nothing at all.
    Nothing,
}

impl SignInKey {
    /// What a symbol means, with the modifiers that were held.
    ///
    /// A letter typed with Control, Alt or the logo key held is **nothing**
    /// rather than a letter: those are chords, and a chord that typed its
    /// letter into a password field would put a character there the person
    /// did not mean to type and cannot see.
    #[must_use]
    pub fn of(symbol: Keysym, held: &ModifiersState) -> Self {
        match symbol {
            Keysym::BackSpace => return Self::Erase,
            Keysym::Tab | Keysym::ISO_Left_Tab => return Self::OtherField,
            Keysym::Return | Keysym::KP_Enter => return Self::Enter,
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
mod tests {
    use super::*;

    /// No modifiers held.
    fn nothing_held() -> ModifiersState {
        ModifiersState::default()
    }

    /// **The four keys that do something do it**, and letters are letters.
    #[test]
    fn the_keys_that_do_something() {
        let held = nothing_held();
        assert_eq!(SignInKey::of(Keysym::BackSpace, &held), SignInKey::Erase);
        assert_eq!(SignInKey::of(Keysym::Tab, &held), SignInKey::OtherField);
        assert_eq!(
            SignInKey::of(Keysym::ISO_Left_Tab, &held),
            SignInKey::OtherField
        );
        assert_eq!(SignInKey::of(Keysym::Return, &held), SignInKey::Enter);
        assert_eq!(SignInKey::of(Keysym::KP_Enter, &held), SignInKey::Enter);
        assert_eq!(SignInKey::of(Keysym::a, &held), SignInKey::Letter('a'));
        assert_eq!(SignInKey::of(Keysym::A, &held), SignInKey::Letter('A'));
        assert_eq!(SignInKey::of(Keysym::eacute, &held), SignInKey::Letter('é'));
        assert_eq!(SignInKey::of(Keysym::space, &held), SignInKey::Letter(' '));
    }

    /// **A chord types nothing.** Control+A in a password field is not an
    /// `a` the person cannot see.
    #[test]
    fn a_chord_types_nothing() {
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
            assert_eq!(SignInKey::of(Keysym::a, &held), SignInKey::Nothing);
        }
    }

    /// **Keys with no letter do nothing** — Escape, arrows, function keys —
    /// and nothing here opens, runs or reaches anything.
    #[test]
    fn keys_with_no_letter_do_nothing() {
        let held = nothing_held();
        for symbol in [
            Keysym::Escape,
            Keysym::Left,
            Keysym::F1,
            Keysym::Delete,
            Keysym::Shift_L,
            Keysym::NoSymbol,
        ] {
            assert_eq!(
                SignInKey::of(symbol, &held),
                SignInKey::Nothing,
                "{symbol:?}"
            );
        }
    }
}
