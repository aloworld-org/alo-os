//! What one key press means on the recovery screen.
//!
//! The seat's own XKB state has already turned the press into a symbol; this
//! is the short list of what the recovery screen does with one. Three things,
//! and no fourth:
//!
//! - **move** between the two moments going back can happen at;
//! - **choose** the one that is selected — which does nothing while none is,
//!   because nothing here is preselected either;
//! - **nothing** at all, for every other key.
//!
//! # There is no Escape
//!
//! The approval surface has no Escape because dismissing would be a third
//! answer. This screen has none for a different reason: it is what a person
//! reaches **when the desktop will not start**, and a key that took the screen
//! away would leave them looking at a machine that does nothing, with no way
//! back to the one road out. Leaving this screen is restarting the machine,
//! which is a thing a person does to the machine and not a key.
//!
//! # And no shortcut to going back
//!
//! Going back replaces the whole operating system a person is running. There is
//! no letter that does it, so a key pressed for something else — an Enter held
//! down from the last screen, a `y` meant for a password — cannot start it.

use smithay::input::keyboard::{Keysym, ModifiersState};

/// One key press, as the recovery screen understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryKey {
    /// Select the next moment, in reading order. Tab.
    Next,
    /// Select the previous moment, in reading order. Shift+Tab.
    Previous,
    /// Choose the moment that is selected. Enter or Space.
    Choose,
    /// Anything else, which does nothing at all.
    Nothing,
}

impl RecoveryKey {
    /// What a symbol means, with the modifiers that were held.
    ///
    /// Moving is Tab rather than the arrow keys, because Tab has no direction
    /// on the page and an arrow has one that depends on which way the person
    /// reads. A chord — Control, Alt or the logo key held — is **nothing**.
    #[must_use]
    pub fn of(symbol: Keysym, held: &ModifiersState) -> Self {
        if held.ctrl || held.alt || held.logo {
            return Self::Nothing;
        }
        match symbol {
            Keysym::ISO_Left_Tab => Self::Previous,
            Keysym::Tab if held.shift => Self::Previous,
            Keysym::Tab => Self::Next,
            Keysym::Return | Keysym::KP_Enter | Keysym::space => Self::Choose,
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

    /// **The keys that do something do it**, and the screen is operable with
    /// Tab and Enter alone.
    #[test]
    fn the_keys_that_do_something() {
        let held = nothing_held();
        assert_eq!(RecoveryKey::of(Keysym::Tab, &held), RecoveryKey::Next);
        assert_eq!(
            RecoveryKey::of(Keysym::ISO_Left_Tab, &held),
            RecoveryKey::Previous
        );
        let shifted = ModifiersState {
            shift: true,
            ..nothing_held()
        };
        assert_eq!(
            RecoveryKey::of(Keysym::Tab, &shifted),
            RecoveryKey::Previous
        );
        for choosing in [Keysym::Return, Keysym::KP_Enter, Keysym::space] {
            assert_eq!(RecoveryKey::of(choosing, &held), RecoveryKey::Choose);
        }
    }

    /// **No key goes back by itself, and none takes the screen away.** Escape
    /// does nothing, because a person whose desktop will not start has nowhere
    /// to be sent; the arrows and every letter do nothing, because replacing the
    /// operating system is not something a stray key press does.
    #[test]
    fn no_key_goes_back_by_itself_and_none_takes_the_screen_away() {
        let held = nothing_held();
        for symbol in [
            Keysym::Escape,
            Keysym::y,
            Keysym::Y,
            Keysym::n,
            Keysym::r,
            Keysym::Left,
            Keysym::Right,
            Keysym::Up,
            Keysym::Down,
            Keysym::Delete,
            Keysym::BackSpace,
            Keysym::F1,
            Keysym::NoSymbol,
        ] {
            assert_eq!(
                RecoveryKey::of(symbol, &held),
                RecoveryKey::Nothing,
                "{symbol:?}"
            );
        }
    }

    /// **A chord does nothing**, including Control+Enter.
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
            for symbol in [Keysym::Return, Keysym::Tab, Keysym::space] {
                assert_eq!(RecoveryKey::of(symbol, &held), RecoveryKey::Nothing);
            }
        }
    }
}
