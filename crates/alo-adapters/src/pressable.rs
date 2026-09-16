//! The kinds of control an agent may ask to press in an application's window.
//!
//! **A closed list, written before any model exists.** The agent names a
//! control by what kind it is and what it is called, and the kind is one of
//! these — offered as a choice, so the sentence a person approves reads *press
//! the button named “Sign in”* in their own language rather than an identifier
//! out of a toolkit. Nothing that takes text is here: a field a person types
//! into is not something an agent presses, and filling one would be free text
//! arriving in an application, which is exactly what an adapter may not take.

use alo_capability::{Offered, Takes};
use alo_strings::Word;

use crate::role::Role;

/// A kind of control that can be pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pressable {
    /// A button — including one that stays down, and one that opens a menu.
    Button,
    /// A box that is ticked or not.
    CheckBox,
    /// One of a set of options, only one of which is chosen.
    RadioButton,
    /// A switch that is on or off.
    Switch,
    /// One item of a menu.
    MenuItem,
    /// One tab of a set of tabs.
    Tab,
    /// A link.
    Link,
}

impl Pressable {
    /// All seven, in the order they are offered.
    pub const ALL: [Self; 7] = [
        Self::Button,
        Self::CheckBox,
        Self::RadioButton,
        Self::Switch,
        Self::MenuItem,
        Self::Tab,
        Self::Link,
    ];

    /// The name a model sends, and the record keeps.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::CheckBox => "check_box",
            Self::RadioButton => "radio_button",
            Self::Switch => "switch",
            Self::MenuItem => "menu_item",
            Self::Tab => "tab",
            Self::Link => "link",
        }
    }

    /// The kind whose name a model sent, if it is one of these.
    #[must_use]
    pub fn called(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.named() == name)
    }

    /// What a person reads for it.
    #[must_use]
    pub const fn words(self) -> Word {
        use crate::fallback_words as words;
        match self {
            Self::Button => words::A_BUTTON,
            Self::CheckBox => words::A_CHECK_BOX,
            Self::RadioButton => words::A_RADIO_BUTTON,
            Self::Switch => words::A_SWITCH,
            Self::MenuItem => words::A_MENU_ITEM,
            Self::Tab => words::A_TAB,
            Self::Link => words::A_LINK,
        }
    }

    /// The role a control of this kind has in an application's tree.
    #[must_use]
    pub const fn role(self) -> Role {
        match self {
            Self::Button => Role::Button,
            Self::CheckBox => Role::CheckBox,
            Self::RadioButton => Role::RadioButton,
            Self::Switch => Role::Switch,
            Self::MenuItem => Role::MenuItem,
            Self::Tab => Role::Tab,
            Self::Link => Role::Link,
        }
    }

    /// The kind of control a role is, when it is one that can be pressed.
    #[must_use]
    pub fn of(role: Role) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.role() == role)
    }

    /// The choice a verb offers: every kind, each with its words.
    #[must_use]
    pub fn offered() -> Takes {
        Takes::choice(
            Self::ALL
                .into_iter()
                .map(|kind| Offered::called(kind.named(), kind.words())),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_is_found_by_its_name_and_by_its_role_and_by_nothing_else() {
        for kind in Pressable::ALL {
            assert_eq!(Pressable::called(kind.named()), Some(kind));
            assert_eq!(Pressable::of(kind.role()), Some(kind));
        }
        assert_eq!(Pressable::called("text_field"), None);
        assert_eq!(Pressable::called("Button"), None);
        for role in [
            Role::Window,
            Role::Label,
            Role::TextField,
            Role::PasswordField,
            Role::Document,
            Role::Image,
            Role::Canvas,
            Role::Part,
        ] {
            assert_eq!(Pressable::of(role), None, "{role:?} is pressable");
        }
    }
}
