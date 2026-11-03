//! **Closing alo's terminal**: at once when the shell is idle, and after a
//! question when a command is still running.
//!
//! The owner's decision of 2026-11-01, in full: *Close ends the terminal at
//! once when the shell is idle; when a command is still running, it first asks
//! "A command is still running. Close anyway?" — Close anyway / Cancel, and
//! Escape is Cancel.* `docs/features.md` and
//! `docs/design/the-terminal-window.md` record it with its date.
//!
//! # What *running* is
//!
//! `alo_terminal::whether_a_command_is_running`: the terminal's foreground
//! process group is not the shell's own, asked of the kernel at the moment
//! Close is pressed. A terminal that will not say is **asked about**, never
//! ended blind.
//!
//! # What closing ends
//!
//! The shell, and with it whatever is running in it: the terminal is dropped,
//! and dropping it sends the shell `SIGHUP` and, if it does not go, `SIGKILL`
//! (`alo_terminal::AShellOnAPseudoterminal`'s `Drop`). That is what closing any
//! terminal has always meant, which is exactly why the question is asked first
//! when something is running.
//!
//! # Cancel changes nothing
//!
//! Cancel, or Escape, closes the question and nothing else: the terminal stays
//! where it is and the command keeps running. While the question is open it
//! takes the terminal's keys, so a key meant for it never reaches the command.
//!
//! # Where the keyboard starts
//!
//! **On Cancel**, the button that loses nothing. A person who pressed Close by
//! mistake and presses Enter has lost no work. *A reading, recorded so it can
//! be disagreed with*: the decision names the two buttons and Escape, and not
//! which one the keyboard starts on.

use alo_terminal::what_a_person_typed::{ANamedKey, WhatWasTyped};

use crate::which_window_is_in_front::AKindOfWindow;

/// Which of the question's two buttons the keyboard is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnTheQuestion {
    /// *Close anyway*: ends the terminal and what is running in it.
    CloseAnyway,
    /// *Cancel*: closes only the question.
    Cancel,
}

impl OnTheQuestion {
    /// The other button: there are two, so Tab and either arrow go round them.
    const fn the_other(self) -> Self {
        match self {
            Self::CloseAnyway => Self::Cancel,
            Self::Cancel => Self::CloseAnyway,
        }
    }
}

/// What a press on Close did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhatClosingDid {
    /// The shell was idle, and the terminal is closed.
    Closed,
    /// A command is running, or the terminal would not say, so it asks.
    Asked,
    /// There was no terminal to close.
    NothingWasOpen,
}

impl crate::Server {
    /// **The person pressed the terminal's Close.**
    ///
    /// Idle: it ends now. Running, or the terminal would not say: the question
    /// opens with the keyboard on Cancel, and the terminal comes in front so
    /// the keys — Escape among them — reach the question.
    pub(crate) fn the_person_closes_the_terminal(&mut self) -> WhatClosingDid {
        let Some(terminal) = self.alos_own_terminal.as_ref() else {
            return WhatClosingDid::NothingWasOpen;
        };
        if terminal.is_a_command_running().closing_asks_first() {
            self.closing_the_terminal = Some(OnTheQuestion::Cancel);
            self.the_composer.it_gave_the_keyboard_up();
            self.the_keyboard_is_now_in(None);
            self.the_person_went_to(AKindOfWindow::TheTerminal);
            return WhatClosingDid::Asked;
        }
        self.the_terminal_ends();
        WhatClosingDid::Closed
    }

    /// The terminal ends: its shell is sent away with whatever it ran, and its
    /// window leaves the canvas. The next Ctrl+Alt+T opens a new one.
    pub(crate) fn the_terminal_ends(&mut self) {
        self.alos_own_terminal = None;
        self.the_terminal_on_the_plane = None;
        self.where_the_terminal_is = None;
        self.closing_the_terminal = None;
        self.where_the_question_was_drawn = None;
    }

    /// Whether the question is open, and where the keyboard is on it.
    pub(crate) const fn the_question_closing_asks(&self) -> Option<OnTheQuestion> {
        self.closing_the_terminal
    }

    /// One of the question's buttons was pressed.
    pub(crate) fn the_question_is_answered(&mut self, answer: OnTheQuestion) {
        match answer {
            OnTheQuestion::CloseAnyway => self.the_terminal_ends(),
            OnTheQuestion::Cancel => {
                self.closing_the_terminal = None;
                self.where_the_question_was_drawn = None;
            }
        }
    }

    /// A key typed at the terminal while the question is open: **the
    /// question's, and never the shell's.**
    ///
    /// Escape is Cancel; Tab and the left and right arrows go between the two
    /// buttons; Enter and Space press the one the keyboard is on. Any other key
    /// is taken and does nothing, so typing at a question cannot reach the
    /// command it is asking about.
    ///
    /// Answers whether the question was open to take it.
    pub(crate) fn a_key_reached_the_question(&mut self, typed: WhatWasTyped) -> bool {
        let Some(on) = self.closing_the_terminal else {
            return false;
        };
        match typed {
            WhatWasTyped::Named(ANamedKey::Escape) => {
                self.the_question_is_answered(OnTheQuestion::Cancel);
            }
            WhatWasTyped::Named(ANamedKey::Tab | ANamedKey::Left | ANamedKey::Right) => {
                self.closing_the_terminal = Some(on.the_other());
            }
            WhatWasTyped::Named(ANamedKey::Enter) | WhatWasTyped::Character(' ') => {
                self.the_question_is_answered(on);
            }
            WhatWasTyped::Named(_) | WhatWasTyped::Character(_) => {}
        }
        true
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "closing_the_terminal_tests.rs"]
mod tests;
