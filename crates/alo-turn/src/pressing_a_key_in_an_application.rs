//! **`accessible.press_key`**: one key of a closed list — Enter, Escape, Tab or
//! an arrow — pressed once in the application's window that has the keyboard
//! (2026-11-17). A change, so at *ask each time* it runs only after one
//! approval of *press {key} in {application}*, and at full access it runs at
//! once, recorded the same way.
//!
//! The order, and why each step is where it is:
//!
//! 1. `alo_adapters::PressingAKey::of` — the grants asked again **now**, the
//!    application installed, no adapter of its own; refused before any window
//!    is looked at.
//! 2. The tree opened for this one press: the place with the keyboard must be
//!    neither a password field nor a terminal.
//! 3. **The compositor's door**, `press-in`: the key goes only to the client
//!    whose window has the keyboard, only once that window is the named
//!    application's — alo's own copy at the default, the person's own window
//!    only where they chose that (ADR 0143 Q6) — and is never moved there.
//!
//! **One approval presses once.** Nothing here repeats a key.

use alo_acting::{AKey, ToType, Typed};
use alo_adapters::keys_words as words;
use alo_adapters::{NotWalked, PressingAKey, TreeFault};
use alo_applications::Application;
use alo_capability::{Authorised, Grants, Refused};
use alo_record::Entry;

use crate::machine::Machine;
use crate::refusing::NotDone;

/// One key alo pressed in an application's window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AKeyPressed {
    /// The application.
    application: String,
    /// The key.
    key: AKey,
}

impl AKeyPressed {
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
}

/// Press one approved key, once.
///
/// **A key the compositor says it pressed is recorded as having run**; every
/// other way this ends is a refusal, written down as one, and no key went.
pub(crate) fn pressing_a_key_in_an_application(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<AKeyPressed, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refusing = |refused: Refused| {
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

    let pressing = match PressingAKey::of(
        authorised,
        machine.the_adapters(),
        grants,
        &machine.the_applications_installed_now(),
        strings,
    ) {
        Ok(pressing) => pressing,
        Err(refused) => return refusing(refused),
    };
    let Ok(application) = Application::identified(pressing.application()) else {
        return refusing(pressing.refused(words::NO_KEYBOARD, strings));
    };

    // Opened now, for this one press, and dropped when it ends.
    let tree = match machine.the_windows_on_the_screen() {
        Some(windows) => windows.opening(),
        None => Err(TreeFault::NotThere),
    };
    let tree = match tree {
        Ok(tree) => tree,
        Err(fault) => {
            let said = NotWalked::Tree(fault).said(pressing.application(), None, strings);
            return refusing(Refused::worded_elsewhere(call, said));
        }
    };
    if let Err(refused) = pressing.the_keyboard_is_safe(tree.as_ref(), strings) {
        return refusing(refused);
    }
    drop(tree);

    let Some(door) = machine.at_the_compositors_door() else {
        return refusing(pressing.refused(words::NO_KEYBOARD, strings));
    };
    let asked = ToType::Key {
        application,
        key: pressing.key(),
    };
    let answered = door.typing(&asked, "").unwrap_or(Typed::NoKeyboard);
    let word = match answered {
        Typed::PressedTheKey => None,
        Typed::NotInFront => Some(words::NOT_IN_FRONT),
        Typed::NotAlosCopy => Some(words::NOT_ALOS_COPY),
        Typed::Typed
        | Typed::AskedToSave
        | Typed::MayType
        | Typed::NoKeyboard
        | Typed::CannotType(_) => Some(words::NO_KEYBOARD),
    };
    if let Some(word) = word {
        return refusing(pressing.refused(word, strings));
    }
    let pressed = AKeyPressed {
        application: pressing.application().to_owned(),
        key: pressing.key(),
    };
    (
        Entry::ran(&pressing.into_authorised(), strings),
        Ok(pressed),
    )
}
