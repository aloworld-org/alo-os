//! **A turn paused because a call for an application was refused**, and what
//! becomes of it: the sheet goes up for what the daemon says was wanted, and
//! the person's answer is told to the agent, which asks again or goes on (the
//! owner's decision of 2026-11-16, `docs/design/when-alo-asks-for-an-application.md`;
//! `docs/contracts/asking-socket.md`, *alo asks again once it is let use an
//! application*).
//!
//! # What is told, and what is not
//!
//! The agent hears one line: `alo-granted:` once the grant is on the disk and
//! the daemon told, or `alo-not-granted:` — for *Not now*, for silence, for a
//! sheet nobody could see, and at once when nothing was wanted. **It never
//! hears what was wanted or why the call was refused**, and nothing it says
//! reaches the sheet: the list is the daemon's, asked on the person's door.
//!
//! # The turn is held, and handed back
//!
//! The socket keeping the agent's turn is held here while the sheet is up, and
//! once the line is written it goes back to be heard on, because the agent
//! goes on and says more on it. A Stop pressed while it is held travels on it
//! (`crate::the_person_stops_a_task`).

use std::io::Write as _;
use std::os::unix::net::UnixStream;
use std::time::Instant;

use alo_protocol::what_the_agent_says_back::{IT_WAS_NOT_GIVEN, THE_APPLICATION_WAS_GIVEN};

use crate::the_application_sheet::TheSheetAnswered;
use crate::the_tasks_a_person_opened::WhichTask;

impl crate::Server {
    /// **The agent paused this task's turn after a call for an application was
    /// refused**, and the daemon says these were `wanted`. With nothing wanted
    /// — or no task to ask in — the turn goes on at once with the refusal and
    /// no sheet goes up.
    pub(crate) fn a_call_for_an_application_was_refused(
        &mut self,
        which: WhichTask,
        the_turn: UnixStream,
        wanted: &[String],
    ) {
        if wanted.is_empty() || !self.the_tasks.is_open(which) {
            self.the_paused_turn_is_told(which, the_turn, false);
            return;
        }
        self.turns_paused_for_an_application.push((which, the_turn));
        self.applications_were_wanted(which, wanted, Instant::now());
    }

    /// **Each answer the sheet kept, told to the turn that paused for it** —
    /// once a frame. An answer for a task whose turn is no longer paused (a
    /// second application it wanted, answered after the first) is a grant or
    /// not on the disk all the same, and tells nobody.
    pub(crate) fn the_paused_turns_are_told(&mut self) {
        for answered in self.take_what_the_application_sheet_answered() {
            let (which, given) = match &answered {
                TheSheetAnswered::Granted { wanted, .. } => (wanted.task(), true),
                TheSheetAnswered::NotNow { wanted } => (wanted.task(), false),
            };
            if let Some(at) = self
                .turns_paused_for_an_application
                .iter()
                .position(|(paused, _)| *paused == which)
            {
                let (_, the_turn) = self.turns_paused_for_an_application.remove(at);
                self.the_paused_turn_is_told(which, the_turn, given);
            }
        }
    }

    /// The turn this task paused, handed to a Stop: the sheet it asked comes
    /// down and nothing is told on it but the Stop.
    pub(crate) fn the_paused_turn_of(&mut self, which: WhichTask) -> Option<UnixStream> {
        let at = self
            .turns_paused_for_an_application
            .iter()
            .position(|(paused, _)| *paused == which)?;
        Some(self.turns_paused_for_an_application.remove(at).1)
    }

    /// The turns told since the frame loop last asked, to be heard on again.
    pub(crate) fn take_the_turns_to_hear_again(&mut self) -> Vec<(WhichTask, UnixStream)> {
        std::mem::take(&mut self.turns_to_hear_again)
    }

    /// Write the one line, and hand the turn back to be heard on. A turn that
    /// will not take the line has gone, and is let go.
    fn the_paused_turn_is_told(&mut self, which: WhichTask, the_turn: UnixStream, given: bool) {
        let line = if given {
            THE_APPLICATION_WAS_GIVEN
        } else {
            IT_WAS_NOT_GIVEN
        };
        if (&the_turn)
            .write_all(format!("{line}\n").as_bytes())
            .is_ok()
        {
            self.turns_to_hear_again.push((which, the_turn));
        }
    }
}

#[cfg(test)]
#[path = "a_paused_turn_is_told_tests.rs"]
mod tests;
