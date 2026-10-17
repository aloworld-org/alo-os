//! What one key press means while the window of what a person can install is
//! open.
//!
//! It moves through the list, narrows to a kind of work, asks for an
//! application, and closes — and nothing else.
//!
//! **No key here installs anything.** `Install` says the person asked; the
//! host does the installing, on the indicator, which `alo_software::install`
//! refuses to run without. A window cannot reach the machine and this key does
//! not change that — `crate::MarketWindow` holds what was asked for and waits
//! to be told what became of it.

use smithay::input::keyboard::{Keysym, ModifiersState};

/// One key press, as the window of what a person can install understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketKey {
    /// Move to the application above. Up.
    Up,
    /// Move to the application below. Down.
    Down,
    /// Move to the first application shown. Home.
    First,
    /// Move to the last application shown. End.
    Last,
    /// Narrow to the kind of work before this one. Left.
    Before,
    /// Narrow to the kind of work after this one. Right.
    After,
    /// Ask for the application the person is on. Enter.
    ///
    /// **Asks; it does not install.** The window has no way to change this
    /// machine, and this key is how a person says what they want rather than
    /// how it happens.
    Install,
    /// Close the window. Escape.
    Close,
    /// Anything else, which does nothing at all.
    Nothing,
}

impl MarketKey {
    /// What a symbol means, with the modifiers that were held.
    ///
    /// A chord — Control, Alt or the logo key held — is **nothing**, so a
    /// shortcut meant for something else never moves or closes this window.
    /// `crate::RunningKey` reads its keys the same way and for the same
    /// reason.
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
            Keysym::Left | Keysym::KP_Left => Self::Before,
            Keysym::Right | Keysym::KP_Right => Self::After,
            Keysym::Return | Keysym::KP_Enter => Self::Install,
            Keysym::Escape => Self::Close,
            _ => Self::Nothing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nothing_held() -> ModifiersState {
        ModifiersState::default()
    }

    #[test]
    fn the_keys_that_move_and_the_key_that_closes() {
        let held = nothing_held();
        assert_eq!(MarketKey::of(Keysym::Up, &held), MarketKey::Up);
        assert_eq!(MarketKey::of(Keysym::Down, &held), MarketKey::Down);
        assert_eq!(MarketKey::of(Keysym::Home, &held), MarketKey::First);
        assert_eq!(MarketKey::of(Keysym::End, &held), MarketKey::Last);
        assert_eq!(MarketKey::of(Keysym::Left, &held), MarketKey::Before);
        assert_eq!(MarketKey::of(Keysym::Right, &held), MarketKey::After);
        assert_eq!(MarketKey::of(Keysym::Escape, &held), MarketKey::Close);
    }

    /// The keypad reaches this window exactly as the main block does.
    #[test]
    fn the_keypad_says_the_same_things() {
        let held = nothing_held();
        assert_eq!(MarketKey::of(Keysym::KP_Up, &held), MarketKey::Up);
        assert_eq!(MarketKey::of(Keysym::KP_Down, &held), MarketKey::Down);
        assert_eq!(MarketKey::of(Keysym::KP_Home, &held), MarketKey::First);
        assert_eq!(MarketKey::of(Keysym::KP_End, &held), MarketKey::Last);
        assert_eq!(MarketKey::of(Keysym::KP_Left, &held), MarketKey::Before);
        assert_eq!(MarketKey::of(Keysym::KP_Right, &held), MarketKey::After);
    }

    /// **A chord is nothing**, so a shortcut meant for the desktop does not
    /// move this list or close it on the way past.
    #[test]
    fn a_chord_does_nothing_at_all() {
        for held in [
            ModifiersState {
                ctrl: true,
                ..ModifiersState::default()
            },
            ModifiersState {
                alt: true,
                ..ModifiersState::default()
            },
            ModifiersState {
                logo: true,
                ..ModifiersState::default()
            },
        ] {
            for symbol in [
                Keysym::Up,
                Keysym::Down,
                Keysym::Home,
                Keysym::End,
                Keysym::Left,
                Keysym::Right,
                Keysym::Escape,
            ] {
                assert_eq!(MarketKey::of(symbol, &held), MarketKey::Nothing);
            }
        }
    }

    /// Every other key is nothing, including the ones that look like they
    /// ought to do something here.
    #[test]
    fn every_other_key_is_nothing() {
        let held = nothing_held();
        for symbol in [
            Keysym::space,
            Keysym::Tab,
            Keysym::Delete,
            Keysym::BackSpace,
            Keysym::a,
            Keysym::F5,
        ] {
            assert_eq!(
                MarketKey::of(symbol, &held),
                MarketKey::Nothing,
                "{symbol:?}"
            );
        }
    }
}
