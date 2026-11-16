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

use std::time::{Instant, SystemTime};

use crate::giving_alo_an_application_it_asked_for as giving;

use crate::the_application_sheet::{
    AnApplicationWanted, ApplicationSheetKey, HowLongForAnApplication, OnTheApplicationSheet,
    TheSheetAnswered,
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
            // **Said on the journal**, by identifier, so a walk can tell the
            // sheet was put in front of the person for this application.
            eprintln!(
                "{}",
                alo_acting::said_on_the_log::the_sheet_asks_for(identifier)
            );
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
            reason = "taken by the turn that asked, once the agent pauses on a refused application \n                      verb (#1310)"
        )
    )]
    pub(crate) fn take_what_the_application_sheet_answered(&mut self) -> Vec<TheSheetAnswered> {
        std::mem::take(&mut self.application_sheet_answers)
    }

    /// **Carry an answer out, and keep it for the turn that asked.** A press
    /// that granted is made into the grant now, through the one road Settings
    /// uses; one that could not be made is kept as *not now*, because the
    /// action would only be refused again.
    fn the_application_sheet_answered(&mut self, answered: Option<TheSheetAnswered>) {
        let Some(answered) = answered else {
            return;
        };
        let not_given = match &answered {
            TheSheetAnswered::Granted {
                wanted, how_long, ..
            } => !self.alo_is_given_what_it_asked_for(wanted, *how_long),
            TheSheetAnswered::NotNow { .. } => false,
        };
        let kept = match answered {
            TheSheetAnswered::Granted { wanted, .. } if not_given => {
                TheSheetAnswered::NotNow { wanted }
            }
            kept => kept,
        };
        self.application_sheet_answers.push(kept);
    }

    /// Make the grant the person pressed for. `false` where it was not made,
    /// and why is said on the desktop's log.
    fn alo_is_given_what_it_asked_for(
        &mut self,
        wanted: &AnApplicationWanted,
        how_long: HowLongForAnApplication,
    ) -> bool {
        let Some(places) = self.settings_places.as_ref() else {
            eprintln!("alo-shell: nothing was granted: this session was not told where grants are");
            return false;
        };
        let Some(agent) = places.agent() else {
            eprintln!("alo-shell: nothing was granted: this session names no agent");
            return false;
        };
        let door = alo_changing::TheDaemonsDoor::at(places.daemon());
        let now = SystemTime::now();
        match giving::alo_is_given(places.grants(), &door, agent, wanted, how_long, now) {
            Ok(id) => {
                if how_long == HowLongForAnApplication::ThisTask {
                    self.grants_for_a_task.push((wanted.task(), id));
                }
                self.the_grants_changed_here(now);
                true
            }
            Err(why) => {
                eprintln!(
                    "alo-shell: {} was not granted to alo: {why}",
                    wanted.identifier()
                );
                false
            }
        }
    }

    /// **A task ended — its window closed, or it was stopped — so every grant
    /// made for *This task* goes now**, and the daemon is told. A grant already
    /// gone (taken back in Settings) changes nothing.
    pub(crate) fn a_tasks_grants_end(&mut self, which: WhichTask) {
        let (ending, staying): (Vec<_>, Vec<_>) = self
            .grants_for_a_task
            .iter()
            .copied()
            .partition(|(task, _)| *task == which);
        if ending.is_empty() {
            return;
        }
        self.grants_for_a_task = staying;
        let Some(places) = self.settings_places.as_ref() else {
            return;
        };
        let door = alo_changing::TheDaemonsDoor::at(places.daemon());
        let now = SystemTime::now();
        for (_, id) in ending {
            match giving::a_tasks_grant_is_taken_back(places.grants(), &door, id, now) {
                // **Said on the journal**, so a walk can tell the grant went
                // with its task.
                Ok(alo_changing::Gone::Revoked { .. }) => {
                    eprintln!("{}", alo_acting::said_on_the_log::a_tasks_grant_went());
                }
                Ok(alo_changing::Gone::AlreadyGone) => {}
                Err(why) => {
                    eprintln!(
                        "alo-shell: a grant for a task that ended could not be taken back, and \
                         still stands until sign-out: {why}"
                    );
                }
            }
        }
        self.the_grants_changed_here(now);
    }

    /// Settings is told the grants changed, and which are a task's.
    fn the_grants_changed_here(&mut self, now: SystemTime) {
        let for_a_task = self.grants_for_a_task.iter().map(|(_, id)| *id).collect();
        self.settings.the_grants_changed_elsewhere(for_a_task, now);
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_application_sheet_on_the_server_tests.rs"]
mod tests;
