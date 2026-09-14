//! What one key press means while the window of what is filling the disk is
//! open.
//!
//! It moves between rows, opens and closes folders, counts again, and closes —
//! and nothing else. There is no key that deletes, moves or empties anything:
//! the window ends at the answer, and what a person does about a large folder
//! is theirs to do somewhere that is not a measurement.
//!
//! **Opening follows the reading.** The arrow that points the way a line is
//! read opens a folder and the other closes it: Right opens for somebody
//! reading left to right, and Left opens for somebody reading right to left.

use alo_strings::Direction;
use smithay::input::keyboard::{Keysym, ModifiersState};

/// One key press, as the window of what is filling the disk understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillingKey {
    /// Select the row above. Up.
    Previous,
    /// Select the row below. Down.
    Next,
    /// Select the first row. Home.
    First,
    /// Select the last row. End.
    Last,
    /// Open the selected folder. The arrow pointing the way a line is read.
    Open,
    /// Close the selected folder. The other arrow.
    Shut,
    /// Open the selected folder if it is closed, and close it if it is open.
    /// Enter or Space.
    Toggle,
    /// Count the folder again. F5.
    CountAgain,
    /// Close the window. Escape.
    Close,
    /// Anything else, which does nothing at all.
    Nothing,
}

impl FillingKey {
    /// What a symbol means, with the modifiers that were held, for somebody
    /// reading `reading`.
    ///
    /// A chord — Control, Alt or the logo key held — is **nothing**.
    #[must_use]
    pub fn of(symbol: Keysym, held: &ModifiersState, reading: Direction) -> Self {
        if held.ctrl || held.alt || held.logo {
            return Self::Nothing;
        }
        let (forward, back) = match reading {
            Direction::LeftToRight => (Self::Open, Self::Shut),
            Direction::RightToLeft => (Self::Shut, Self::Open),
        };
        match symbol {
            Keysym::Up | Keysym::KP_Up => Self::Previous,
            Keysym::Down | Keysym::KP_Down => Self::Next,
            Keysym::Home | Keysym::KP_Home => Self::First,
            Keysym::End | Keysym::KP_End => Self::Last,
            Keysym::Right | Keysym::KP_Right => forward,
            Keysym::Left | Keysym::KP_Left => back,
            Keysym::Return | Keysym::KP_Enter | Keysym::space => Self::Toggle,
            Keysym::F5 => Self::CountAgain,
            Keysym::Escape => Self::Close,
            _ => Self::Nothing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Opening follows the reading**: Right opens and Left closes for
    /// somebody reading left to right, and the other way round for somebody
    /// reading right to left.
    #[test]
    fn opening_follows_the_way_a_line_is_read() {
        let held = ModifiersState::default();
        let ltr = Direction::LeftToRight;
        let rtl = Direction::RightToLeft;
        assert_eq!(FillingKey::of(Keysym::Right, &held, ltr), FillingKey::Open);
        assert_eq!(FillingKey::of(Keysym::Left, &held, ltr), FillingKey::Shut);
        assert_eq!(FillingKey::of(Keysym::Left, &held, rtl), FillingKey::Open);
        assert_eq!(FillingKey::of(Keysym::Right, &held, rtl), FillingKey::Shut);
        for (symbol, meant) in [
            (Keysym::Up, FillingKey::Previous),
            (Keysym::Down, FillingKey::Next),
            (Keysym::Home, FillingKey::First),
            (Keysym::End, FillingKey::Last),
            (Keysym::Return, FillingKey::Toggle),
            (Keysym::space, FillingKey::Toggle),
            (Keysym::F5, FillingKey::CountAgain),
            (Keysym::Escape, FillingKey::Close),
        ] {
            assert_eq!(FillingKey::of(symbol, &held, ltr), meant, "{symbol:?}");
        }
    }

    /// **No key deletes, moves or empties**, and a chord does nothing at all.
    #[test]
    fn no_key_deletes_and_a_chord_does_nothing() {
        let held = ModifiersState::default();
        for symbol in [
            Keysym::Delete,
            Keysym::BackSpace,
            Keysym::x,
            Keysym::d,
            Keysym::Tab,
            Keysym::F2,
        ] {
            assert_eq!(
                FillingKey::of(symbol, &held, Direction::LeftToRight),
                FillingKey::Nothing,
                "{symbol:?}"
            );
        }
        let chord = ModifiersState {
            alt: true,
            ..ModifiersState::default()
        };
        for symbol in [Keysym::Right, Keysym::Escape, Keysym::F5, Keysym::Return] {
            assert_eq!(
                FillingKey::of(symbol, &chord, Direction::LeftToRight),
                FillingKey::Nothing
            );
        }
    }
}
