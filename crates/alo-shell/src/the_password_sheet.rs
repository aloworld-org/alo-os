//! **The password sheet**: polkit asking the person for their password, on
//! the desktop alo draws
//! ([ADR 0150](../../../docs/decisions/0150-administrator-actions-ask-the-persons-password-each-time.md)).
//!
//! The owner, 2026-11-09: *"Yes, alo's desktop draws it"* — a sheet drawn by
//! alo's own desktop, never by the assistant or an application, and alo never
//! sees the password. So this is the desktop's half of polkit's authentication
//! agent: a question arrives from `alo-desktop`'s agent thread
//! ([`ToTheSheet::Ask`]), the person types and presses Enter, and what they
//! typed goes back on that question's own channel ([`FromTheSheet::Typed`]) to
//! be checked by polkit's helper. The agent, the model and the agent service
//! never see it.
//!
//! # What the person can do
//!
//! - **Type**, into one hidden field — the field the sign-in screen uses, which
//!   zeroes what it held when it lets go (`crate::sign_in_password`).
//! - **Enter** sends what is typed, once, and the field is forgotten at once.
//! - **Escape** closes the sheet and polkit is told they cancelled, which is
//!   what the agent service is told too.
//!
//! **A wrong password leaves the sheet up**, saying so, and they type again. A
//! password that cannot be checked here at all — the system's list was not
//! given it at sign-in, or the checker did not answer — says that instead,
//! because *not right* would be false.
//!
//! **While it is up every key is the sheet's.** A key that reached the
//! application behind it would be the person's password typed into somebody
//! else's window.

use std::collections::VecDeque;
use std::sync::mpsc::Sender;

use crate::SignInKey;
use crate::sign_in_entry::SignInEntry;

/// What polkit is asking the password for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatItIsFor {
    /// A command alo was asked to run as administrator: the command itself.
    ACommandAloWasAsked(String),
    /// Anything else on this machine that asked polkit: polkit's own sentence.
    SomethingElse(String),
}

/// One question polkit put to this desktop.
pub struct PasswordAsked {
    /// polkit's name for the question, so that taking it back names it.
    pub cookie: String,
    /// What the password is for, drawn on the sheet.
    pub what_for: WhatItIsFor,
    /// Where the person's answer goes.
    pub answer: Sender<FromTheSheet>,
}

/// Why a password could not be checked here at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyNotChecked {
    /// The system's list was not given the person's password when they signed
    /// in (`alo_administrator::in_step`).
    NotInStep,
    /// The part of the system that checks a password did not answer.
    NoChecker,
}

/// What the agent thread tells the sheet.
pub enum ToTheSheet {
    /// Put this question up, or queue it behind the one that is.
    Ask(PasswordAsked),
    /// What was typed for `cookie` was not the person's password.
    NotRight {
        /// The question it was for.
        cookie: String,
    },
    /// What was typed for `cookie` could not be checked here.
    CannotBeChecked {
        /// The question it was for.
        cookie: String,
        /// Why.
        why: WhyNotChecked,
    },
    /// The question `cookie` is over: answered, or taken back by polkit.
    Over {
        /// The question that is over.
        cookie: String,
    },
}

/// What the person did.
pub enum FromTheSheet {
    /// They typed this and pressed Enter.
    Typed(ThePasswordTyped),
    /// They closed the sheet.
    Cancelled,
}

/// **What the person typed, carried to polkit's helper and nowhere else.**
///
/// No `Debug`, no `Clone`, no `Display`, and its bytes are overwritten when it
/// is dropped — `crate::sign_in_password`'s rules, for the one copy that has
/// to cross to another thread.
pub struct ThePasswordTyped {
    /// The bytes, overwritten on drop.
    typed: String,
}

impl ThePasswordTyped {
    /// A copy of what is in the field.
    fn of(entry: &SignInEntry) -> Self {
        Self {
            typed: entry.password().to_owned(),
        }
    }

    /// What a caller already holding the text hands over — a test of the
    /// agent's side, which has no sheet to type into.
    #[must_use]
    pub fn for_a_test(typed: &str) -> Self {
        Self {
            typed: typed.to_owned(),
        }
    }

    /// Lent, for the one call that hands it to the helper.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.typed
    }
}

impl Drop for ThePasswordTyped {
    fn drop(&mut self) {
        let mut bytes = std::mem::take(&mut self.typed).into_bytes();
        bytes.fill(0);
        std::hint::black_box(&bytes);
    }
}

/// What the sheet says under the question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TheLineUnder {
    /// How to answer.
    HowToAnswer,
    /// What was typed is being checked.
    Checking,
    /// It was not the person's password.
    NotRight,
    /// It could not be checked here.
    CannotBeChecked(WhyNotChecked),
}

/// What one frame draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SheetShows<'a> {
    /// What the password is for.
    pub(crate) what_for: &'a WhatItIsFor,
    /// Whether anything is typed — the one fact about it that is drawn.
    pub(crate) typed_something: bool,
    /// The line under the question.
    pub(crate) under: TheLineUnder,
    /// Which question this is, counted from the first: what a screen reader
    /// is told a new question by (`crate::access_sheet_in_front`).
    pub(crate) asked: u64,
}

/// **The sheet's state**: the questions waiting, the one field, what it says.
pub(crate) struct ThePasswordSheet {
    /// The questions, the one on screen first.
    waiting: VecDeque<PasswordAsked>,
    /// The hidden field.
    entry: SignInEntry,
    /// What the sheet says under the question.
    under: TheLineUnder,
    /// How many times a question has come to the front.
    asked: u64,
}

impl ThePasswordSheet {
    /// No question, and nothing drawn.
    pub(crate) fn none() -> Self {
        Self {
            waiting: VecDeque::new(),
            entry: SignInEntry::waiting_for_a_password(),
            under: TheLineUnder::HowToAnswer,
            asked: 0,
        }
    }

    /// Whether a question is up, and so whether every key is the sheet's.
    pub(crate) fn is_up(&self) -> bool {
        !self.waiting.is_empty()
    }

    /// What this frame draws, while a question is up.
    pub(crate) fn shows(&self) -> Option<SheetShows<'_>> {
        self.waiting.front().map(|asked| SheetShows {
            what_for: &asked.what_for,
            typed_something: self.entry.has_password(),
            under: self.under,
            asked: self.asked,
        })
    }

    /// What the agent thread said.
    pub(crate) fn told(&mut self, told: ToTheSheet) {
        match told {
            ToTheSheet::Ask(asked) => {
                if self.waiting.is_empty() {
                    self.a_fresh_question();
                }
                self.waiting.push_back(asked);
            }
            ToTheSheet::NotRight { cookie } => {
                if self.is_in_front(&cookie) {
                    self.under = TheLineUnder::NotRight;
                }
            }
            ToTheSheet::CannotBeChecked { cookie, why } => {
                if self.is_in_front(&cookie) {
                    self.under = TheLineUnder::CannotBeChecked(why);
                }
            }
            ToTheSheet::Over { cookie } => {
                let was_in_front = self.is_in_front(&cookie);
                self.waiting.retain(|asked| asked.cookie != cookie);
                if was_in_front {
                    self.a_fresh_question();
                }
            }
        }
    }

    /// One key, while a question is up.
    pub(crate) fn pressed(&mut self, key: SignInKey) {
        if !self.is_up() {
            return;
        }
        match key {
            // **Escape closes it**, and polkit is told the person cancelled.
            SignInKey::Clear => self.answered(FromTheSheet::Cancelled),
            SignInKey::Enter => {
                if self.under != TheLineUnder::Checking && self.entry.has_password() {
                    let typed = ThePasswordTyped::of(&self.entry);
                    self.entry.forgotten();
                    self.under = TheLineUnder::Checking;
                    self.send(FromTheSheet::Typed(typed));
                }
            }
            SignInKey::Letter(_) | SignInKey::Erase => {
                if self.under != TheLineUnder::Checking {
                    let _ = self.entry.pressed(key);
                }
            }
            // One field: nothing to move to, and no list.
            SignInKey::OtherField | SignInKey::Earlier | SignInKey::Later | SignInKey::Nothing => {}
        }
    }

    /// The sheet could not be drawn on this display, so the question in front
    /// is answered as cancelled: a sheet nobody can see must not hold every key.
    pub(crate) fn could_not_be_drawn(&mut self) {
        if self.is_up() {
            self.answered(FromTheSheet::Cancelled);
        }
    }

    /// Whether `cookie` is the question on screen.
    fn is_in_front(&self, cookie: &str) -> bool {
        self.waiting
            .front()
            .is_some_and(|asked| asked.cookie == cookie)
    }

    /// The field empty and the first line back, for the question now in front.
    fn a_fresh_question(&mut self) {
        self.entry.forgotten();
        self.under = TheLineUnder::HowToAnswer;
        self.asked = self.asked.wrapping_add(1);
    }

    /// Answer the question in front and take it down.
    fn answered(&mut self, answer: FromTheSheet) {
        if let Some(asked) = self.waiting.pop_front() {
            let _ = asked.answer.send(answer);
        }
        self.a_fresh_question();
    }

    /// Send on the question in front's channel; a question whose asker has gone
    /// is taken down rather than left holding every key.
    fn send(&mut self, answer: FromTheSheet) {
        let gone = self
            .waiting
            .front()
            .is_some_and(|asked| asked.answer.send(answer).is_err());
        if gone {
            self.waiting.pop_front();
            self.a_fresh_question();
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_password_sheet_tests.rs"]
mod tests;
