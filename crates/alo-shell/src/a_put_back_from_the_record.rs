//! **Put back**, between the record window and the desktop: where this
//! person's places are, the plan made when a change's *Put back…* is pressed,
//! the request sent when it is confirmed, and the service's answer.
//!
//! The window decides nothing about the disk (`crate::record_window`), and
//! this server has no words of its own, so the steps that need either are
//! handed to the frame loop, which has both (`crate::direct_desktop`): it
//! makes the plan with the person's words, sends the request through the
//! desktop, and tells the answer back here.

use std::path::PathBuf;
use std::time::SystemTime;

use crate::{RecordOpened, RecordShows};

impl crate::Server {
    /// Where this person's places to return to are, and their home — told
    /// once by the desktop at sign-in, as where the record is.
    pub fn the_places_are_kept(&mut self, whose: PathBuf, home: PathBuf) {
        self.places_kept = Some((whose, home));
    }

    /// The names of this person's places, read again into the window.
    pub(crate) fn the_places_are_read_again(&mut self) {
        let names = self
            .places_kept
            .as_ref()
            .map(|(whose, _)| crate::putting_back_from_the_record::the_places_under(whose))
            .unwrap_or_default();
        self.record.the_places_are(names);
    }

    /// What the window answered a key or a press with.
    pub(crate) fn the_record_answered(&mut self, opened: Option<RecordOpened>) {
        match opened {
            None => {}
            Some(RecordOpened::NowhereToShow(why)) => {
                eprintln!("alo-shell: the record could not be shown again: {why:?}");
            }
            Some(RecordOpened::Shown | RecordOpened::RefusedInTheWindow) => {
                self.the_places_are_read_again();
            }
            Some(RecordOpened::AskedToPutBack(at)) => self.plan_asked = Some(at),
            Some(RecordOpened::PutBackConfirmed { place, folders }) => {
                self.put_back_asked = Some((place, folders));
            }
        }
    }

    /// The change whose *Put back…* was pressed, taken once.
    pub(crate) fn take_the_plan_asked(&mut self) -> Option<usize> {
        self.plan_asked.take()
    }

    /// Make the plan for the change at `at`, in `strings`' words, and open the
    /// confirmation with it. A session never told where the places are opens
    /// it with the sentence for a place that is not kept.
    pub(crate) fn the_plan_is_made(&mut self, at: usize, strings: &alo_strings::Strings) {
        let RecordShows::Account { account, .. } = self.record.shows() else {
            return;
        };
        let plan = match &self.places_kept {
            Some((whose, home)) => crate::putting_back_from_the_record::the_plan(
                account,
                at,
                (whose, home),
                strings,
                SystemTime::now(),
            ),
            None => Err(strings
                .say(
                    &alo_as_it_was::words::NOTHING_IS_KEPT.key(),
                    &alo_strings::Filling::nothing(),
                )
                .into_text()),
        };
        self.record.the_confirmation_opens(at, plan);
    }

    /// The folders the person confirmed, taken once, for the desktop to ask
    /// the service.
    pub(crate) fn take_the_put_back_asked(&mut self) -> Option<(String, Vec<String>)> {
        self.put_back_asked.take()
    }

    /// The service answered. Put back: the record is read again, so the
    /// putting back is the newest line, and the confirmation is gone. Nothing
    /// put back: the confirmation stays, saying why.
    pub(crate) fn that_put_back_answered(&mut self, answer: Result<Vec<String>, String>) {
        match answer {
            Ok(_) => {
                if let Some(Ok(recounting)) = self.recounting.clone() {
                    let read = self
                        .record
                        .pressed(crate::RecordKey::ReadAgain, &recounting);
                    self.the_record_answered(read);
                }
            }
            Err(why) => self.record.nothing_was_put_back(why),
        }
    }
}

#[cfg(test)]
#[path = "a_put_back_from_the_record_tests.rs"]
mod tests;
