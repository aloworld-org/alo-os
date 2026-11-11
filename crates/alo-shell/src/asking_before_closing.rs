//! **Closing a running task asks**: *Stop and close*, *Keep running*, or
//! *Cancel*
//! ([ADR 0139](../../../docs/decisions/0139-stop-reaches-alo-and-cancels-what-it-sent.md)).
//!
//! `docs/features.md` promises that closing the window of a task alo is still
//! working on offers three named choices and never kills it silently or leaves
//! it running where nobody can find it. Now:
//!
//! - **Stop and close** is Stop, and the window closes **once alo has really
//!   stopped** — never before, so a person never closes a window on work that
//!   is still running.
//! - **Keep running** leaves alo working and sends the window into the
//!   right-hand panel **by the road Minimise takes**
//!   (`crate::a_task_goes_aside`), so it is exactly where minimising puts it:
//!   a square in the rail whose card says what alo is doing and offers Stop.
//!   It stops nothing. A desktop with no panel keeps the window where it is
//!   rather than putting it nowhere.
//! - **Cancel** closes nothing and changes nothing.
//!
//! # A card in the conversation, until the design draws a sheet
//!
//! The design has not drawn where these choices sit, so they are asked as a
//! card at the end of the conversation, in the approval card's shell, with
//! *Keep running* between its two answers, and answered the same way: Tab
//! moves forward through them, Shift+Tab back, Enter chooses, Escape is
//! *Cancel*. Nothing is preselected, so Enter alone does nothing. The ADR
//! records the departure.
//!
//! **Nothing here answers a change.** A change waiting when *Stop and close* is
//! chosen lapses with the turn, as any Stop's does; under *Keep running* it
//! goes on waiting, and the task window brought back from the panel still
//! shows it.

use crate::approval_keys::ApprovalKey;
use crate::the_tasks_a_person_opened::WhichTask;

/// The three choices, in reading order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichChoice {
    /// Stop alo, and close the window once it has stopped.
    StopAndClose,
    /// Leave alo working, and put the window in the right-hand panel.
    KeepRunning,
    /// Close nothing.
    Cancel,
}

impl WhichChoice {
    /// The choice after this one, going round.
    const fn after(self) -> Self {
        match self {
            Self::StopAndClose => Self::KeepRunning,
            Self::KeepRunning => Self::Cancel,
            Self::Cancel => Self::StopAndClose,
        }
    }

    /// The choice before this one, going round.
    const fn before(self) -> Self {
        match self {
            Self::StopAndClose => Self::Cancel,
            Self::KeepRunning => Self::StopAndClose,
            Self::Cancel => Self::KeepRunning,
        }
    }
}

/// A running task whose window the person asked to close.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosingAsked {
    /// The task.
    pub which: WhichTask,
    /// Which choice the keyboard is on, and [`None`] until the person moves.
    pub selected: Option<WhichChoice>,
}

/// What a key did to the question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatTheKeyDid {
    /// It was not for the question.
    NotForIt,
    /// The selection moved.
    Moved,
    /// The person chose.
    Chose(WhichChoice),
}

impl ClosingAsked {
    /// One key, while the question is asked.
    ///
    /// From nothing selected, Tab lands on the first choice and Shift+Tab on
    /// the last, the way a row of buttons is entered from either end.
    #[must_use]
    pub fn pressed(&mut self, key: ApprovalKey) -> WhatTheKeyDid {
        match key {
            ApprovalKey::NextAnswer => {
                self.selected = Some(self.selected.map_or(WhichChoice::StopAndClose, WhichChoice::after));
                WhatTheKeyDid::Moved
            }
            ApprovalKey::PreviousAnswer => {
                self.selected = Some(self.selected.map_or(WhichChoice::Cancel, WhichChoice::before));
                WhatTheKeyDid::Moved
            }
            ApprovalKey::Choose => self
                .selected
                .map_or(WhatTheKeyDid::NotForIt, WhatTheKeyDid::Chose),
            ApprovalKey::Decline => WhatTheKeyDid::Chose(WhichChoice::Cancel),
            ApprovalKey::Nothing => WhatTheKeyDid::NotForIt,
        }
    }
}

impl crate::Server {
    /// **Ask before closing this running task.**
    pub(crate) fn ask_before_closing(&mut self, which: WhichTask) {
        self.closing_asked = Some(ClosingAsked {
            which,
            selected: None,
        });
    }

    /// The question about closing, if this task is the one asked about.
    #[must_use]
    pub(crate) fn closing_asked_about(&self, which: WhichTask) -> Option<ClosingAsked> {
        self.closing_asked.filter(|asked| asked.which == which)
    }

    /// **One key on the question about closing**, if it is asked about this
    /// task. Answers whether the key was the question's.
    pub(crate) fn a_key_on_the_closing_question(
        &mut self,
        which: WhichTask,
        key: ApprovalKey,
    ) -> bool {
        let Some(mut asked) = self.closing_asked_about(which) else {
            return false;
        };
        match asked.pressed(key) {
            WhatTheKeyDid::NotForIt => false,
            WhatTheKeyDid::Moved => {
                self.closing_asked = Some(asked);
                true
            }
            WhatTheKeyDid::Chose(WhichChoice::Cancel) => {
                self.closing_asked = None;
                true
            }
            // **Into the panel, still running**, by Minimise's own road: the
            // window goes once a frame lends the panel, and nothing is stopped.
            WhatTheKeyDid::Chose(WhichChoice::KeepRunning) => {
                self.closing_asked = None;
                self.this_task_is_asked_to_go_aside(which);
                true
            }
            WhatTheKeyDid::Chose(WhichChoice::StopAndClose) => {
                self.closing_asked = None;
                if self.the_person_stops_a_task(which) {
                    self.close_when_stopped.push(which);
                }
                true
            }
        }
    }

    /// **The task has stopped**: close its window if *Stop and close* asked
    /// for it.
    pub(crate) fn close_it_if_it_was_asked(&mut self, which: WhichTask) {
        let Some(at) = self.close_when_stopped.iter().position(|w| *w == which) else {
            return;
        };
        self.close_when_stopped.remove(at);
        let _ = self.the_person_closes_a_task(which);
    }
}

#[cfg(test)]
#[path = "asking_before_closing_tests.rs"]
mod tests;
