//! The record window: what the machine did, in front of the person.
//!
//! `docs/features.md`: *afterwards, ask what it did.* `alo_recounting` reads the
//! record off the disk and composes the account — every clause, every remark
//! the record makes about itself, every refusal — and this window is where that
//! account meets a person's eyes.
//!
//! # Two roads, one account
//!
//! ADR 0009 says the plain way to reach anything the agent can reach must exist
//! beside the agent's. So there are two doors, and they are the same door:
//!
//! - [`RecordWindow::opened_by_hand`] — a person opening the window, which
//!   takes nothing but where the record is. No turn, no model, no overlay and
//!   no agent is involved or reachable from these files, so the window opens on
//!   a machine where the agent was declined, never set up, or cannot be paid
//!   for;
//! - [`RecordWindow::asked_what_it_did`] — a person asking the agent *what did
//!   you do?*, which reaches **this same window and this same account** rather
//!   than a model's retelling of it. A model able to word what happened would
//!   be a machine with two accounts of one moment.
//!
//! Both read the record through one private call to
//! `alo_recounting::Recounting::show`, so there is nothing either road could
//! do differently.
//!
//! # It reads the whole record and writes nothing
//!
//! What is asked is always `alo_record::Asking::anything()` at
//! `alo_recounting::AtMost::ONE_SITTING`: no filter, and so no filter that could
//! hide a refusal by default, and no search that changes what *today* means.
//! The bound is `alo-recounting`'s and says itself when it left something out.
//! Nothing here opens the record to write, and `tests/record_source.rs` holds
//! these files to that.
//!
//! # It words nothing
//!
//! Every clause, remark and refusal is `alo-recounting`'s or the record's own,
//! drawn by `crate::record_raster`. There is no summary, because
//! `alo-recounting`'s own rule is that a surface able to word what happened is
//! a machine with two accounts of one moment.
//!
//! # A refusal is not an empty window
//!
//! A record that is not there, not believed, or not readable is drawn as the
//! refuser's sentence in the window — never as a list with nothing in it,
//! because a machine that did nothing and a machine whose record was deleted
//! look the same on an empty screen.

use alo_record::Asking;
use alo_recounting::{Account, AtMost, NotRecounted, Recounting, Recounts};

use crate::RecordKey;
use crate::record_shown::RecordShown;

/// What the record window draws now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordShows<'a> {
    /// The window is closed: nothing at all is drawn.
    Nothing,
    /// The account, as `alo-recounting` handed it to the compositor.
    Account {
        /// The account, read off the disk when the window opened or last read
        /// again.
        account: &'a Account,
        /// How many of the most recent entries the view has moved past.
        /// Zero shows the most recent first.
        moved_past: usize,
    },
    /// Why the record could not be shown, in the words of whoever refused.
    Refusal(&'a NotRecounted),
}

/// What opening the window, or reading the record again, did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordOpened {
    /// The account is in the window.
    Shown,
    /// The record could not be read, and the window shows why.
    RefusedInTheWindow,
    /// There is nowhere to show anything; the sentence is for the host's log.
    /// The window stays closed.
    NowhereToShow(NotRecounted),
}

/// The record window.
#[derive(Debug)]
pub struct RecordWindow {
    /// The account, as the compositor was handed it.
    shown: RecordShown,
    /// Why the record could not be shown, while the window says so.
    refusal: Option<NotRecounted>,
    /// How many of the most recent entries the view has moved past.
    moved_past: usize,
}

impl RecordWindow {
    /// The window, closed, on a compositor drawing an output.
    #[must_use]
    pub fn on_an_output() -> Self {
        Self::with(RecordShown::on_an_output())
    }

    /// The window, closed, on a compositor with nothing to show it on. Opening
    /// it reads the record and answers with `alo-recounting`'s refusal.
    #[must_use]
    pub fn with_no_output() -> Self {
        Self::with(RecordShown::with_no_output())
    }

    /// A closed window.
    fn with(shown: RecordShown) -> Self {
        Self {
            shown,
            refusal: None,
            moved_past: 0,
        }
    }

    /// A person opened the window, without asking any agent anything.
    ///
    /// ADR 0009's road: it takes where the record is and nothing else.
    pub fn opened_by_hand(&mut self, recounting: &Recounting) -> RecordOpened {
        self.read(recounting)
    }

    /// A person asked the agent *what did you do?*
    ///
    /// The same account the plain road opens, read the same way: nothing a
    /// model says reaches this window, and nothing in it is a second telling.
    pub fn asked_what_it_did(&mut self, recounting: &Recounting) -> RecordOpened {
        self.read(recounting)
    }

    /// One key press, and what it did when it read the record again.
    ///
    /// Up, Down, Home and End move the view through the account; F5 reads the
    /// record off the disk again; Escape closes the window. Every key does
    /// nothing while the window is closed.
    pub fn pressed(&mut self, key: RecordKey, recounting: &Recounting) -> Option<RecordOpened> {
        if !self.is_open() {
            return None;
        }
        let last = self
            .shown
            .account()
            .map_or(0, |account| account.how_many().saturating_sub(1));
        match key {
            RecordKey::Newer => self.moved_past = self.moved_past.saturating_sub(1),
            RecordKey::Older => self.moved_past = (self.moved_past + 1).min(last),
            RecordKey::Newest => self.moved_past = 0,
            RecordKey::Oldest => self.moved_past = last,
            RecordKey::ReadAgain => return Some(self.read(recounting)),
            RecordKey::Close => self.closed(),
            RecordKey::Nothing => {}
        }
        None
    }

    /// Close the window. The record is untouched; the account is dropped.
    pub fn closed(&mut self) {
        self.shown.taken_down();
        self.refusal = None;
        self.moved_past = 0;
    }

    /// What the window draws now.
    #[must_use]
    pub fn shows(&self) -> RecordShows<'_> {
        if let Some(why) = &self.refusal {
            return RecordShows::Refusal(why);
        }
        match self.shown.account() {
            Some(account) => RecordShows::Account {
                account,
                moved_past: self.moved_past,
            },
            None => RecordShows::Nothing,
        }
    }

    /// Whether the window is open — an account or a refusal — so the host
    /// routes keys here rather than to an application.
    #[must_use]
    pub fn is_open(&self) -> bool {
        !matches!(self.shows(), RecordShows::Nothing)
    }

    /// Read the whole record off the disk and hand it to the compositor.
    ///
    /// The one call to `Recounting::show` in this crate, for both roads.
    fn read(&mut self, recounting: &Recounting) -> RecordOpened {
        self.refusal = None;
        self.moved_past = 0;
        match recounting.show(
            Some(&mut self.shown),
            &Asking::anything(),
            AtMost::ONE_SITTING,
        ) {
            Recounts::Shown(_) => RecordOpened::Shown,
            Recounts::Refused(why) if why.is_nowhere_to_show_it() => {
                self.shown.taken_down();
                RecordOpened::NowhereToShow(why)
            }
            Recounts::Refused(why) => {
                self.shown.taken_down();
                self.refusal = Some(why);
                RecordOpened::RefusedInTheWindow
            }
        }
    }
}

#[cfg(test)]
#[path = "record_window_tests.rs"]
mod tests;
