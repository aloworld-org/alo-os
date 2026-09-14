//! What one key press means while a change is waiting for a person's answer.
//!
//! The seat's own XKB state has already turned the press into a symbol
//! (`crate::approval_seat`); this is the short list of what the approval
//! surface does with one. Three things, and no fourth:
//!
//! - **move** between the two answers, forwards or back;
//! - **choose** the answer that is selected — which does nothing while none
//!   is, because nothing on this surface is preselected;
//! - **nothing** at all, for every other key.
//!
//! There is no letter here, so there is nowhere to type a reason for saying
//! no; no Escape that dismisses, so there is no third answer; and no shortcut
//! for either answer, so a key pressed for something else — a `y` meant for a
//! document, an Enter held down from the last dialogue — cannot answer a
//! question the person has not read.

use smithay::input::keyboard::{Keysym, ModifiersState};

/// One key press, as the approval surface understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalKey {
    /// Select the next answer, in reading order. Tab.
    NextAnswer,
    /// Select the previous answer, in reading order. Shift+Tab.
    PreviousAnswer,
    /// Give the selected answer, or acknowledge a sentence. Enter or Space.
    Choose,
    /// Anything else, which does nothing at all.
    Nothing,
}

impl ApprovalKey {
    /// What a symbol means, with the modifiers that were held.
    ///
    /// Moving is Tab rather than the arrow keys, because Tab has no direction
    /// on the page and an arrow has one that depends on which way the person
    /// reads. A chord — Control, Alt or the logo key held — is **nothing**, so
    /// a shortcut meant for an application never answers a question.
    #[must_use]
    pub fn of(symbol: Keysym, held: &ModifiersState) -> Self {
        if held.ctrl || held.alt || held.logo {
            return Self::Nothing;
        }
        match symbol {
            Keysym::ISO_Left_Tab => Self::PreviousAnswer,
            Keysym::Tab if held.shift => Self::PreviousAnswer,
            Keysym::Tab => Self::NextAnswer,
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

    /// **The keys that do something do it.**
    #[test]
    fn the_keys_that_do_something() {
        let held = nothing_held();
        assert_eq!(ApprovalKey::of(Keysym::Tab, &held), ApprovalKey::NextAnswer);
        assert_eq!(
            ApprovalKey::of(Keysym::ISO_Left_Tab, &held),
            ApprovalKey::PreviousAnswer
        );
        let shifted = ModifiersState {
            shift: true,
            ..nothing_held()
        };
        assert_eq!(
            ApprovalKey::of(Keysym::Tab, &shifted),
            ApprovalKey::PreviousAnswer
        );
        for choosing in [Keysym::Return, Keysym::KP_Enter, Keysym::space] {
            assert_eq!(ApprovalKey::of(choosing, &held), ApprovalKey::Choose);
        }
    }

    /// **No key is an answer by itself, and none is a third answer.** Letters
    /// that look like *yes* and *no* in some language, Escape, the arrows and
    /// Delete all do nothing — so there is no shortcut to approving, no
    /// dismissing, and no letter with which to type a reason.
    #[test]
    fn no_key_is_an_answer_by_itself_and_none_is_a_third_answer() {
        let held = nothing_held();
        for symbol in [
            Keysym::y,
            Keysym::Y,
            Keysym::n,
            Keysym::a,
            Keysym::j,
            Keysym::o,
            Keysym::Escape,
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
                ApprovalKey::of(symbol, &held),
                ApprovalKey::Nothing,
                "{symbol:?}"
            );
        }
    }

    /// **A chord does nothing**, including Control+Enter, which some
    /// applications use to send.
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
                assert_eq!(ApprovalKey::of(symbol, &held), ApprovalKey::Nothing);
            }
        }
    }
}
