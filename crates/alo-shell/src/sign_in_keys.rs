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
    /// **Take back everything typed**, in both fields, and wait at the name
    /// again. Escape, which `alo_access::leaving` says clears what was typed on
    /// this screen — there is nowhere further back to go from the screen a
    /// machine starts at, and somebody who cannot see the password field must
    /// be able to start it again with one key rather than counting erasures.
    Clear,
    /// Move to the row above, on a screen that offers a list.
    ///
    /// **A list needs these and the two fields never did**, which is why they
    /// arrived with the language sheet rather than with the sign-in screen.
    /// On a screen with no list they do nothing, by the same rule as
    /// [`Self::Nothing`] — an arrow key is not a letter and must never become
    /// one in a password field.
    Earlier,
    /// Move to the row below, on a screen that offers a list.
    Later,
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
            Keysym::Escape => return Self::Clear,
            Keysym::Up | Keysym::KP_Up => return Self::Earlier,
            Keysym::Down | Keysym::KP_Down => return Self::Later,
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

    /// **Keys with no letter do nothing** — arrows, function keys — and
    /// nothing here opens, runs or reaches anything. Escape is not among them:
    /// it clears what was typed, which is the next test.
    #[test]
    fn keys_with_no_letter_do_nothing() {
        let held = nothing_held();
        for symbol in [
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

    /// **Escape clears what was typed**, which is what `alo_access::leaving`
    /// says this screen's Escape does — there is nowhere further back to go
    /// from the screen a machine starts at.
    #[test]
    fn escape_clears_what_was_typed_and_the_two_crates_agree_that_it_does() {
        assert_eq!(
            SignInKey::of(Keysym::Escape, &nothing_held()),
            SignInKey::Clear
        );
        assert_eq!(
            alo_access::leaving(alo_access::tree::Surface::SignIn),
            alo_access::Leaving::ClearsWhatWasTyped
        );
    }
}

#[cfg(test)]
mod the_arrows_move_a_list {
    use super::{Keysym, ModifiersState, SignInKey};

    /// Up and down move, from both the arrow block and the keypad.
    #[test]
    fn up_and_down_move_a_row_at_a_time() {
        let held = ModifiersState::default();
        assert_eq!(SignInKey::of(Keysym::Up, &held), SignInKey::Earlier);
        assert_eq!(SignInKey::of(Keysym::KP_Up, &held), SignInKey::Earlier);
        assert_eq!(SignInKey::of(Keysym::Down, &held), SignInKey::Later);
        assert_eq!(SignInKey::of(Keysym::KP_Down, &held), SignInKey::Later);
    }

    /// **And they are not letters**, which is the thing that would be worst.
    /// `key_char` answers `None` for an arrow, so this would pass without the
    /// arms above by falling through to `Nothing` — the assertion that makes
    /// it mean something is that they are `Earlier` and `Later` rather than
    /// merely *not a letter*.
    #[test]
    fn an_arrow_never_types_a_character() {
        let held = ModifiersState::default();
        for arrow in [Keysym::Up, Keysym::Down, Keysym::KP_Up, Keysym::KP_Down] {
            assert!(
                !matches!(SignInKey::of(arrow, &held), SignInKey::Letter(_)),
                "an arrow key typed a character into a field"
            );
        }
    }

    /// **A chord does not suppress an arrow, and that is deliberate.**
    ///
    /// The modifier check exists for one reason, which [`SignInKey::of`] gives:
    /// a letter typed with Control held would put a character into a password
    /// field that the person did not mean and cannot see. An arrow is never a
    /// character, so there is nothing to protect against — and `Tab`, `Enter`,
    /// `Escape` and `BackSpace` are all answered before that check for the
    /// same reason. An arrow answered after it would be the odd one out.
    ///
    /// This test first asserted the opposite, on the assumption that a chord
    /// should always mean *nothing*. It is written down this way round so the
    /// next reader sees a decision rather than an oversight.
    #[test]
    fn a_chord_does_not_stop_an_arrow_moving() {
        let held = ModifiersState {
            ctrl: true,
            ..ModifiersState::default()
        };
        assert_eq!(SignInKey::of(Keysym::Up, &held), SignInKey::Earlier);
        assert_eq!(SignInKey::of(Keysym::Down, &held), SignInKey::Later);
        // And the rule it is contrasted with still holds: a letter is not.
        assert_eq!(SignInKey::of(Keysym::a, &held), SignInKey::Nothing);
    }
}
