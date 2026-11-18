//! **alo answering the file chooser that is up**, after the person approved
//! *choose this file on the file chooser* (`choose_on_the_file_chooser`, the
//! coordinator's approval of 2026-11-17).
//!
//! The daemon writes `alo-chooses-file <folder> <name>` at the door once the
//! turn has judged the folder and the file against the grants and the person
//! has approved the sentence (`docs/contracts/application-door.md`). The door
//! hands the line here with its connection, and the desktop answers it on its
//! next frame:
//!
//! - **no sheet is up**: `no-sheet`, and nothing else happens;
//! - **the sheet could not have been answered with that file** — not there to
//!   open, a folder's name to save under: `not-believed`, and the sheet stays
//!   up for the person, as it was;
//! - otherwise **the sheet is answered with it, exactly as the person's own
//!   choice would be** — the portal hears `chosen <path>` and records and hands
//!   the file over as for any choice — and alo hears `chosen <application>`,
//!   naming who the file was chosen for.
//!
//! **Nothing here judges a grant.** Which folders alo may answer with is the
//! turn's question, asked before the line is written; a rule here would be a
//! second opinion, and the person's own use of the sheet is never narrowed by
//! one.

use std::io::Write as _;
use std::os::unix::net::UnixStream;
use std::time::Instant;

use alo_acting::{AnsweredTheChooser, ToAnswerAFileChooser};
use alo_file_chooser::Folders;

use crate::the_file_chooser::TheFileChooser;

/// alo's answer, with the daemon's connection held.
#[derive(Debug)]
pub struct AloAnswering {
    /// What alo answers with.
    asked: ToAnswerAFileChooser,
    /// The daemon's connection.
    talking: UnixStream,
}

impl AloAnswering {
    /// The door read `asked` on `talking`.
    pub(crate) const fn of(asked: ToAnswerAFileChooser, talking: UnixStream) -> Self {
        Self { asked, talking }
    }

    /// Tell the daemon, and let it go. A daemon that has gone is not told.
    fn told(mut self, answered: &AnsweredTheChooser) {
        let _ = self.talking.write_all(answered.written().as_bytes());
        let _ = self.talking.write_all(b"\n");
        let _ = self.talking.flush();
    }
}

impl TheFileChooser {
    /// **alo's answers since the last frame**, each answered as this module
    /// says, in the order they came.
    pub(crate) fn alo_answered(
        &mut self,
        answers: Vec<AloAnswering>,
        now: Instant,
        folders: &dyn Folders,
    ) {
        for answer in answers {
            let told = match self.shows() {
                None => AnsweredTheChooser::NoSheet,
                Some(shows) => {
                    let asking = shows.asking.clone();
                    match shows.chooser.alo_chose(
                        answer.asked.folder(),
                        answer.asked.name(),
                        folders,
                    ) {
                        Ok(file) => {
                            self.chosen_for_the_person(file, now, folders);
                            AnsweredTheChooser::Chosen(asking)
                        }
                        Err(_) => AnsweredTheChooser::NotBelieved,
                    }
                }
            };
            answer.told(&told);
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "alo_answers_the_file_chooser_tests.rs"]
mod tests;
