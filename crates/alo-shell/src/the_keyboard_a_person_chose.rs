//! Keeping the keyboard a person picked on the language sheet, and saying when
//! it needs applying.
//!
//! The sheet carries a choice out on `SignInScreen::the_keyboard_to_use`; this
//! decides whether anything has to happen about it and writes
//! `alo_keyboards::THE_MACHINES_KEYBOARDS` when it does. Handing the layout to
//! the keymap is the caller's, because that needs the `Server` and this needs
//! only a path — which is what lets the deciding be tested without a
//! compositor.
//!
//! # Why it remembers what it applied
//!
//! A person's keys arrive one at a time and the choice is asked after every
//! one, so without a memory this would rewrite the file and rebuild the keymap
//! on each press of an arrow. Rebuilding a keymap is not free, and
//! `Server::the_keyboard_is_now` releases every held key first — so doing it
//! on a keystroke nobody meant as a change would drop a key somebody was
//! holding.
//!
//! # Why the file is written before the keymap changes
//!
//! If the write fails, the layout is **not** applied: a machine whose keyboard
//! changed and whose file did not would type through one layout now and
//! another after a restart, and a person who set their keyboard and found it
//! back to `gb` tomorrow has been told a lie by a screen. The other order
//! fails more honestly — the keyboard is still the old one, which is what the
//! file says, and the log carries the reason.

use std::path::Path;

use alo_keyboards::{Keyboards, Layout};
use alo_strings::Language;

/// What was done about the choice, for a caller that has to apply it and a log
/// that has to explain it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AboutTheKeyboard {
    /// Nothing to do: no choice yet, or the same one as last time.
    Unchanged,
    /// Kept, and now the caller's to hand to the keymap.
    Apply(Layout),
    /// Chosen, and **not** applied, because it could not be written down.
    ///
    /// Carries the sentence for the log. The keyboard stays as it was, which
    /// is what the file still says.
    NotKept(String),
}

/// The language this screen has already acted on.
#[derive(Debug, Default)]
pub(crate) struct TheKeyboardAPersonChose {
    /// What was last written and applied, so a press that changes nothing does
    /// nothing.
    applied: Option<String>,
}

impl TheKeyboardAPersonChose {
    /// Nothing chosen and nothing applied, which is every machine at its first
    /// frame.
    pub(crate) fn nothing_yet() -> Self {
        Self::default()
    }

    /// Decide what this choice needs, keeping it when it is new.
    ///
    /// `at` is `alo_keyboards::THE_MACHINES_KEYBOARDS` on a machine and a path
    /// a test writes in a test, for the same reason every other path in this
    /// crate is a parameter.
    ///
    /// **One layout is kept, which is the language's first offer.**
    /// `Keyboards::offered_with` sets `These::just(offer.first())` — French
    /// gets `fr` and not also `be`, `ca` and `ch(fr)`. Those are what a
    /// *setting* offers next through `Keyboards::add_for`, and installing all
    /// four here would give a person a switch key cycling through three
    /// keyboards they never asked for. This comment claimed the opposite until
    /// a test read the file back and found one layout in it.
    pub(crate) fn chose(&mut self, chosen: Option<&str>, at: &Path) -> AboutTheKeyboard {
        let Some(tag) = chosen else {
            return AboutTheKeyboard::Unchanged;
        };
        if self.applied.as_deref() == Some(tag) {
            return AboutTheKeyboard::Unchanged;
        }
        let Ok(language) = Language::written(tag) else {
            return AboutTheKeyboard::NotKept(format!("{tag} is not a language tag"));
        };
        let keyboards = Keyboards::offered_with(&language);
        let layout = keyboards.in_use().clone();
        if let Err(why) = alo_keyboards::keeping::keep_the_machines(at, &keyboards) {
            return AboutTheKeyboard::NotKept(format!("{} — {:?}", at.display(), why.why()));
        }
        self.applied = Some(tag.to_owned());
        AboutTheKeyboard::Apply(layout)
    }
}

#[cfg(test)]
#[path = "the_keyboard_a_person_chose_tests.rs"]
mod tests;
