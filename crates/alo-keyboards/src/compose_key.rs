//! Which key composes, and why alo OS does not choose one for you.
//!
//! A compose key is one key that means *the next few keys make a letter*:
//! compose, `s`, `s` writes `ß`. Which key it is has to come from somewhere,
//! and the somewhere is a closed list of the rented options
//! `xkeyboard-config` ships — because turning a key into the compose key is
//! done by naming one of those options, and a name that is not on the list is
//! a keyboard that does not load.
//!
//! # No compose key is set until a person sets one
//!
//! [`ComposeKey::None`] is what alo OS ships, and it is a decision rather than
//! an omission. Every candidate is a key somebody is already using: Right Alt
//! is AltGr, which is how half of Europe types `€`, `@`, `ł` and `ß` — taking
//! it would break typing on the keyboards this product exists for. Caps Lock,
//! the Menu key and Right Ctrl are each somebody's. And the letters the plan
//! names are all reachable on the layouts offered with their own languages
//! without a compose key at all: `ü` is a key in Germany, `ő` is a key in
//! Hungary, `ġ` is a key in Malta. A compose key is what a person who types
//! across languages adds, and the list below is what they choose from.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words::{self, Word};

/// Which key composes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComposeKey {
    /// No key composes. What alo OS ships, for the reason in this file's head.
    #[default]
    None,
    /// The Menu key, between Right Alt and Right Ctrl on most keyboards.
    Menu,
    /// Caps Lock, which then no longer locks capitals.
    CapsLock,
    /// The right-hand Ctrl.
    RightCtrl,
    /// The right-hand Super — the key with a Windows logo printed on it.
    RightSuper,
    /// Scroll Lock, which does nothing else on a machine made this century.
    ScrollLock,
}

impl ComposeKey {
    /// Every choice, in the order a settings surface offers them.
    pub const ALL: [Self; 6] = [
        Self::None,
        Self::Menu,
        Self::CapsLock,
        Self::RightCtrl,
        Self::RightSuper,
        Self::ScrollLock,
    ];

    /// The rented option that makes this key the compose key, or `None` when
    /// no key composes and no option is set.
    ///
    /// Every one of these is checked against the rented data on this machine by
    /// `tests/the_switch_is_one_shortcut.rs`.
    #[must_use]
    pub const fn option(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Menu => Some("compose:menu"),
            Self::CapsLock => Some("compose:caps"),
            Self::RightCtrl => Some("compose:rctrl"),
            Self::RightSuper => Some("compose:rwin"),
            Self::ScrollLock => Some("compose:sclk"),
        }
    }

    /// The string this crate declares for this choice.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::None => words::COMPOSE_NONE,
            Self::Menu => words::COMPOSE_MENU,
            Self::CapsLock => words::COMPOSE_CAPS_LOCK,
            Self::RightCtrl => words::COMPOSE_RIGHT_CTRL,
            Self::RightSuper => words::COMPOSE_RIGHT_SUPER,
            Self::ScrollLock => words::COMPOSE_SCROLL_LOCK,
        }
    }

    /// What this is called, in the language the person reads.
    ///
    /// Never fails and never panics.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use std::collections::BTreeSet;

    /// Nothing composes until somebody says so.
    #[test]
    fn no_key_composes_until_a_person_chooses_one() {
        assert_eq!(ComposeKey::default(), ComposeKey::None);
        assert_eq!(ComposeKey::None.option(), None);
    }

    /// **Every choice is a different key, a different option and a different
    /// sentence** — a list with one row twice is a list nobody can use.
    #[test]
    fn every_choice_is_its_own_key_option_and_sentence() {
        let strings = in_english();
        let mut options = BTreeSet::new();
        let mut said = BTreeSet::new();
        for key in ComposeKey::ALL {
            if let Some(option) = key.option() {
                assert!(option.starts_with("compose:"), "{key:?}");
                assert!(options.insert(option), "{key:?}");
            }
            let sentence = key.said(&strings);
            assert!(!sentence.is_a_bug(), "{key:?}");
            assert!(sentence.unfilled().is_empty(), "{key:?}");
            assert!(said.insert(sentence.text().to_owned()), "{key:?}");
        }
        assert_eq!(options.len(), ComposeKey::ALL.len().saturating_sub(1));
    }

    /// A choice is read back from a person's own file as the same choice.
    #[test]
    fn a_choice_reads_back_as_itself() {
        for key in ComposeKey::ALL {
            let written = toml::to_string(&Held { key }).unwrap();
            assert_eq!(toml::from_str::<Held>(&written).unwrap().key, key);
        }
        assert!(toml::from_str::<Held>("key = \"RightAlt\"").is_err());
    }

    /// A compose key wearing a key, so that TOML has a table to hold it.
    #[derive(Debug, Serialize, Deserialize)]
    struct Held {
        key: ComposeKey,
    }
}
