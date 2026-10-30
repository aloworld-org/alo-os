//! Keystrokes for the task window that holds the keyboard, through the seat this
//! compositor already has.
//!
//! **A task window had never received a key on a running machine.** Its reply
//! box was built and tested in `crate::a_key_reaches_a_task_window`, and
//! `Server::a_key_reached_a_task_window` had no production caller — the fault
//! `crate::the_bars_seat` records fixing for the Bar on 2026-10-23, in the same
//! words. Walking a booted machine on 2026-10-25, typing meant for a task's
//! reply box went into the Bar instead.
//!
//! # When a task window holds the keyboard
//!
//! **Ordinary focus**, settled by the owner on 2026-10-26: the window a person
//! just asked for takes the keyboard, the way any window they open does, and
//! choosing something else takes it away. **A question arriving never moves
//! it.** If a proposal could take the keyboard, a person typing in a document
//! could press *Tab* then *Enter* — two ordinary keys — and approve a change
//! they never read.
//!
//! # Intercepted, exactly as the Bar is
//!
//! While a task window holds the keyboard every key is intercepted at the seat
//! and never forwarded, so an application behind it hears nothing of a reply
//! being typed. One reading of the symbol yields both meanings it can have
//! here — as typing, and as an answer to a waiting change — and
//! `crate::a_key_reaches_the_task_in_front` decides which applies.
//!
//! **Except a shortcut**, which is carried out rather than typed:
//! `crate::a_task_window_hands_on_a_chord` says why, and which ones take the
//! keyboard with them.

use smithay::{backend::input::KeyState, input::keyboard::FilterResult, utils::SERIAL_COUNTER};

use crate::approval_keys::ApprovalKey;
use crate::composer_keys::{ComposerKey, Composing};
use crate::the_tasks_a_person_opened::WhichTask;
use crate::{InputError, Server};

/// What the seat took from one press while a task window held the keyboard.
enum TheTaskTook {
    /// A shortcut this person has bound, to be carried out rather than typed.
    AChord(alo_shortcuts::Chord),
    /// Page Up or Page Down, which scroll the conversation rather than type.
    Scrolling(crate::a_task_window_scrolls::APage),
    /// A key, as typing and as an answer to a waiting change.
    AKey(ComposerKey, ApprovalKey),
}

impl Server {
    /// What one key means to the task window holding the keyboard, as typing
    /// and as an answer, or [`None`] where it is not a press.
    ///
    /// **Built from `Server::the_bars_key`**, because the two answer the same
    /// question about the same seat. What differs is that this reads the symbol
    /// twice over, once in each vocabulary a task window has, and that a bound
    /// shortcut is carried out here and answers [`None`].
    ///
    /// # Errors
    /// [`InputError::Unavailable`] for a display bound without a keyboard, and
    /// [`InputError::InvalidKey`] for a code outside evdev's range.
    pub(crate) fn the_tasks_key(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
    ) -> Result<Option<(ComposerKey, ApprovalKey)>, InputError> {
        if !(1..=0x2ff).contains(&code) {
            return Err(InputError::InvalidKey);
        }
        let handle = self
            .surfaces
            .keyboard
            .as_ref()
            .ok_or(InputError::Unavailable)?
            .handle
            .clone();
        let code = (code + 8).into();
        // A release, and a repeat of a key already down: neither is a press.
        if handle.pressed_keys().contains(&code) == (state == KeyState::Pressed) {
            return Ok(None);
        }
        // **A shortcut is asked about first**, inside the same filter and for
        // `crate::keyboard`'s reason: this call is what advances XKB, so the
        // modifiers are this press's own. Taken out and put back so the filter
        // can read them while the seat holds `surfaces`.
        let shortcuts = self.shortcuts.take();
        let meant = handle.input::<Option<TheTaskTook>, _>(
            &mut self.surfaces,
            code,
            state,
            SERIAL_COUNTER.next_serial(),
            time,
            |_, held, symbol| {
                if state != KeyState::Pressed {
                    return FilterResult::Intercept(None);
                }
                if let Some(chord) = Self::the_chord_a_key_makes(shortcuts.as_ref(), &symbol, held)
                {
                    return FilterResult::Intercept(Some(TheTaskTook::AChord(chord)));
                }
                let symbol = symbol.modified_sym();
                // **The page keys scroll the conversation** — `crate::a_task_window_scrolls`.
                // The reply box has no use for them, and a person who cannot use a
                // wheel still has to be able to read the start of a long answer.
                match symbol {
                    smithay::input::keyboard::Keysym::Page_Up => {
                        return FilterResult::Intercept(Some(TheTaskTook::Scrolling(
                            crate::a_task_window_scrolls::APage::Back,
                        )));
                    }
                    smithay::input::keyboard::Keysym::Page_Down => {
                        return FilterResult::Intercept(Some(TheTaskTook::Scrolling(
                            crate::a_task_window_scrolls::APage::Forward,
                        )));
                    }
                    _ => {}
                }
                // `Composing::No` for `the_bars_key`'s reason: this
                // compositor runs no input method.
                FilterResult::Intercept(Some(TheTaskTook::AKey(
                    ComposerKey::of(symbol, held, Composing::No),
                    ApprovalKey::of(symbol, held),
                )))
            },
        );
        self.shortcuts = shortcuts;
        if let Some(keyboard) = self.surfaces.keyboard.as_mut() {
            keyboard.time = time;
        }
        match meant.flatten() {
            // **Carried out here and never typed**, so a shortcut does what it
            // names in a task window as it does everywhere else —
            // `crate::a_task_window_hands_on_a_chord` has the account.
            Some(TheTaskTook::AChord(chord)) => {
                let _ = self.a_task_window_hands_on_a_chord(chord);
                Ok(None)
            }
            Some(TheTaskTook::Scrolling(page)) => {
                if let Some(which) = self.the_task_with_the_keyboard {
                    let _ = self.a_page_in_the_task(which, page);
                }
                Ok(None)
            }
            Some(TheTaskTook::AKey(typed, answering)) => Ok(Some((typed, answering))),
            None => Ok(None),
        }
    }

    /// Whether a task window is the thing taking keys right now.
    pub fn a_task_is_taking_the_keys(&self) -> bool {
        self.the_task_with_the_keyboard.is_some()
    }

    /// This task window takes the keyboard — because the person just asked for
    /// it. **Never because a question arrived in it.**
    ///
    /// **And off the Dock**, if F6 had put it there: the Dock is asked for keys
    /// after a task window, so Enter would reach the task — but the Dock would
    /// still draw its focus ring and take the keyboard back the moment the task
    /// let go, which is a second place the keyboard is that nobody chose.
    pub const fn this_task_took_the_keyboard(&mut self, which: WhichTask) {
        self.the_task_with_the_keyboard = Some(which);
        self.which_area_has_the_keyboard = None;
        self.which_slot_on_the_bar = None;
    }

    /// No task window holds the keyboard any more.
    pub const fn the_task_gave_the_keyboard_up(&mut self) {
        self.the_task_with_the_keyboard = None;
    }
}
