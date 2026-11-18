//! **Pressing one key in an application's window**, for one approval —
//! `accessible.press_key` (2026-11-17, the owner's *"Yes do all"*).
//!
//! The key is one of `alo_acting::AKey`'s seven — Enter, Escape, Tab and the
//! four arrows — offered as a choice, so no combination, letter or shortcut can
//! arrive. This file holds what is decided before the key goes; the key itself
//! goes by the compositor, to the one client whose window has the keyboard, and
//! only once that window is the named application's (alo's own copy at the
//! default), which `alo_turn` asks it.
//!
//! - [`PressingAKey::of`] asks what the fallback asks before any window is
//!   touched: the grants at this moment, the application installed, no adapter
//!   of its own.
//! - [`PressingAKey::the_keyboard_is_safe`] looks at the application's windows
//!   **then**: where the place with the keyboard is a password field or a
//!   terminal, nothing is pressed. Enter in a terminal runs a command, and a
//!   key in a password field acts on it.

use alo_acting::AKey;
use alo_applications::Installed;
use alo_capability::{Authorised, Grants, Offered, Refused, Takes, Value};
use alo_strings::{Filling, Strings, Word};

use crate::accessibility_tree::{AccessibilityTree, TreeFault};
use crate::adapters::Adapters;
use crate::fallback_reach::reached;
use crate::fallback_verbs::{KEY, PRESS_KEY};
use crate::fallback_words as fallback;
use crate::keys_words as words;
use crate::not_walked::NotWalked;
use crate::places_for_text::{TypingRefuses, the_places_for_text, why_typing_refuses};

/// What a person reads for a key.
#[must_use]
pub const fn the_words_for(key: AKey) -> Word {
    match key {
        AKey::Enter => words::ENTER,
        AKey::Escape => words::ESCAPE,
        AKey::Tab => words::TAB,
        AKey::Up => words::UP,
        AKey::Down => words::DOWN,
        AKey::Left => words::LEFT,
        AKey::Right => words::RIGHT,
    }
}

/// The choice the verb offers: the seven keys, each with its words.
#[must_use]
pub fn the_keys_offered() -> Takes {
    Takes::choice(
        AKey::ALL
            .into_iter()
            .map(|key| Offered::called(key.named(), the_words_for(key))),
    )
}

/// An approved press of one key that has not happened yet.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it.
#[derive(Debug)]
pub struct PressingAKey {
    /// What may run.
    authorised: Authorised,
    /// The application, as installed.
    application: String,
    /// The key.
    key: AKey,
}

impl PressingAKey {
    /// Ask everything that is asked before a window is looked at.
    ///
    /// # Errors
    /// [`Refused`]: another verb's authority, the application not granted, not
    /// installed, or with an adapter of its own.
    pub fn of(
        authorised: Authorised,
        adapters: &Adapters,
        grants: &Grants,
        installed: &Installed,
        strings: &Strings,
    ) -> Result<Self, Refused> {
        let reached = reached(authorised, PRESS_KEY, adapters, grants, installed, strings)?;
        let call = reached.authorised.call();
        let key = match call.value(KEY) {
            Some(Value::Choice { chosen, .. }) => AKey::called(chosen),
            _ => None,
        };
        let Some(key) = key else {
            let said = strings.say(
                &fallback::NOT_THIS_VERB.key(),
                &Filling::of(fallback::VERB, call.verb().to_owned()),
            );
            return Err(Refused::worded_elsewhere(call.clone(), said));
        };
        Ok(Self {
            authorised: reached.authorised,
            application: reached.application,
            key,
        })
    }

    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The key.
    #[must_use]
    pub const fn key(&self) -> AKey {
        self.key
    }

    /// This press refused in these words, with the application filled in.
    #[must_use]
    pub fn refused(&self, word: Word, strings: &Strings) -> Refused {
        Refused::worded_elsewhere(
            self.authorised.call().clone(),
            strings.say(
                &word.key(),
                &Filling::of(fallback::APPLICATION, self.application.clone()),
            ),
        )
    }

    /// Whether the place with the keyboard in the application's windows may be
    /// sent a key: never a password field, never a terminal.
    ///
    /// # Errors
    /// [`Refused`], in the words a person is told; and when the windows could
    /// not be read at all, for the same reason typing refuses then.
    pub fn the_keyboard_is_safe(
        &self,
        tree: &dyn AccessibilityTree,
        strings: &Strings,
    ) -> Result<(), Refused> {
        let places = the_places_for_text(tree, &self.application).map_err(|fault: TreeFault| {
            Refused::worded_elsewhere(
                self.authorised.call().clone(),
                NotWalked::Tree(fault).said(&self.application, None, strings),
            )
        })?;
        for place in places.iter().filter(|place| place.has_the_keyboard()) {
            match why_typing_refuses(tree, &place.at, place.role, &place.states) {
                Some(TypingRefuses::APasswordField) => {
                    return Err(self.refused(words::A_PASSWORD_FIELD_HAS_THE_KEYBOARD, strings));
                }
                Some(TypingRefuses::ATerminal) => {
                    return Err(self.refused(words::A_TERMINAL_HAS_THE_KEYBOARD, strings));
                }
                Some(TypingRefuses::NotEditable) | None => {}
            }
        }
        Ok(())
    }

    /// What ran, once it has.
    #[must_use]
    pub fn into_authorised(self) -> Authorised {
        self.authorised
    }
}
