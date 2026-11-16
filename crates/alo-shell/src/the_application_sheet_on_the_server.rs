//! **The sheet alo asks for an application on, as the server holds it**:
//! raised for what a task's turn wanted, answered by the person's keys,
//! presses and screen reader, and each answer kept for the frame loop to carry
//! out (`docs/design/when-alo-asks-for-an-application.md`).
//!
//! # Every road to an answer is a person's, or is *not now*
//!
//! A key and a click reach here from the seat
//! (`crate::a_key_reaches_the_application_sheet`), a reader's press from the
//! tree this machine published (`crate::a_reader_presses`), and nothing else
//! calls these: **nothing an agent sends reaches the sheet**. The other two
//! answers — five minutes with nobody answering, and a sheet that could not be
//! drawn — are *not now*, never a grant.

use std::time::Instant;

use crate::the_application_sheet::{
    AnApplicationWanted, ApplicationSheetKey, OnTheApplicationSheet, TheSheetAnswered,
};
use crate::the_tasks_a_person_opened::WhichTask;

impl crate::Server {
    /// **A task's turn wanted these applications and nothing grants them**:
    /// each goes up on the sheet, or waits behind the one up. Each is named
    /// as alo's own surfaces name it.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "raised once the agent pauses on a refused application verb, which needs \n                      the turn that goes on after an answer (#1310); until then nothing reaches \n                      this sheet on a real machine"
        )
    )]
    pub(crate) fn applications_were_wanted(
        &mut self,
        which: WhichTask,
        identifiers: &[String],
        now: Instant,
    ) {
        for identifier in identifiers {
            let called = self.what_an_application_is_called(identifier);
            let wanted = AnApplicationWanted::by(which, identifier, &called);
            let answered = self.application_sheet.wanted(wanted, now);
            self.the_application_sheet_answered(answered);
        }
    }

    /// A key while the sheet is up.
    pub(crate) fn the_application_sheet_key(&mut self, key: ApplicationSheetKey) {
        let answered = self.application_sheet.pressed(key, Instant::now());
        self.the_application_sheet_answered(answered);
    }

    /// A press of the pointer on the sheet.
    pub(crate) fn the_application_sheet_pressed(&mut self, on: OnTheApplicationSheet) {
        let answered = self.application_sheet.press(on, Instant::now());
        self.the_application_sheet_answered(answered);
    }

    /// **A screen reader pressed `on`, on the sheet numbered `sheet`.**
    /// `false` when that sheet is no longer up.
    pub(crate) fn a_reader_pressed_on_the_application_sheet(
        &mut self,
        sheet: u64,
        on: OnTheApplicationSheet,
    ) -> bool {
        match self
            .application_sheet
            .a_reader_pressed(sheet, on, Instant::now())
        {
            Some(answered) => {
                self.the_application_sheet_answered(answered);
                true
            }
            None => false,
        }
    }

    /// **The sheet could not be drawn on this display**: *not now*.
    pub(crate) fn the_application_sheet_could_not_be_drawn(&mut self) {
        let answered = self.application_sheet.could_not_be_drawn(Instant::now());
        self.the_application_sheet_answered(answered);
    }

    /// Once a frame: a sheet nobody answered in five minutes is *not now*.
    pub(crate) fn the_application_sheet_waited(&mut self, now: Instant) {
        for answered in self.application_sheet.waited(now) {
            self.the_application_sheet_answered(Some(answered));
        }
    }

    /// **The answers given since the frame loop last asked**, taken so each
    /// is carried out once.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "carried out by the frame loop in the change that makes the grant"
        )
    )]
    pub(crate) fn take_what_the_application_sheet_answered(&mut self) -> Vec<TheSheetAnswered> {
        std::mem::take(&mut self.application_sheet_answers)
    }

    /// Keep an answer for the frame loop.
    fn the_application_sheet_answered(&mut self, answered: Option<TheSheetAnswered>) {
        if let Some(answered) = answered {
            self.application_sheet_answers.push(answered);
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_application_sheet_on_the_server_tests.rs"]
mod tests;
