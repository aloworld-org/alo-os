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
use crate::putting_back_from_the_record::{OnTheSheet, PutBackHere, ThePutBack, TheSheet};
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
        /// Where Put back stands: this person's places, the change the
        /// keyboard is on, and the confirmation while it is open.
        put_back: &'a ThePutBack,
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
    /// The person asked to put back the change at this place in the account
    /// (oldest first): the host works out what that would do, from the
    /// places on the disk, and opens the confirmation with it.
    AskedToPutBack(usize),
    /// The person confirmed: the host asks the service that writes the
    /// record to put these folders back from this place.
    PutBackConfirmed {
        /// The place, by name.
        place: String,
        /// Every folder, as absolute paths.
        folders: Vec<String>,
    },
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
    /// Where Put back stands.
    put_back: ThePutBack,
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
            put_back: ThePutBack::default(),
        }
    }

    /// The names of this person's places, as the host has just read them.
    pub fn the_places_are(&mut self, places: Vec<String>) {
        self.put_back.places = places;
    }

    /// Open the confirmation under the change at `at`, with what putting it
    /// back would do — or why it cannot be done.
    pub fn the_confirmation_opens(&mut self, at: usize, plan: Result<crate::ThePlan, String>) {
        let on = if plan.is_ok() {
            OnTheSheet::PutBack
        } else {
            OnTheSheet::Cancel
        };
        self.put_back.chosen = Some(at);
        self.put_back.sheet = Some(TheSheet {
            at,
            plan,
            on,
            waiting: false,
            refused: None,
        });
    }

    /// The service put nothing back, and this is why: said on the
    /// confirmation, which stays open with *Cancel*.
    pub fn nothing_was_put_back(&mut self, why: String) {
        if let Some(sheet) = self.put_back.sheet.as_mut() {
            sheet.waiting = false;
            sheet.refused = Some(why);
            sheet.on = OnTheSheet::Cancel;
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

    /// A person opened the window on a machine that could not say where it
    /// keeps its record.
    ///
    /// There is no `Recounting` to read, so neither door above can be walked,
    /// and a window that stayed shut would answer a person's key with nothing.
    /// It opens on the refusal instead, in `alo-recounting`'s own words — the
    /// same state a record that could not be read leaves it in.
    pub fn could_not_be_found(&mut self, why: NotRecounted) -> RecordOpened {
        self.shown.taken_down();
        self.moved_past = 0;
        self.refusal = Some(why);
        RecordOpened::RefusedInTheWindow
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
        if self.put_back.sheet.is_some() {
            return self.on_the_confirmation(key);
        }
        let last = self
            .shown
            .account()
            .map_or(0, |account| account.how_many().saturating_sub(1));
        match key {
            RecordKey::NextPutBack => self.the_keyboard_moves(true),
            RecordKey::PreviousPutBack => self.the_keyboard_moves(false),
            RecordKey::Press => return self.put_back.chosen.map(RecordOpened::AskedToPutBack),
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

    /// A key while the confirmation is open: Tab between its two buttons,
    /// Enter presses the one the keyboard is on, Escape is *Cancel*. Nothing
    /// moves the view behind it.
    fn on_the_confirmation(&mut self, key: RecordKey) -> Option<RecordOpened> {
        let sheet = self.put_back.sheet.as_mut()?;
        match key {
            RecordKey::NextPutBack | RecordKey::PreviousPutBack => {
                sheet.on = match sheet.on {
                    OnTheSheet::PutBack => OnTheSheet::Cancel,
                    OnTheSheet::Cancel if sheet.can_put_back() => OnTheSheet::PutBack,
                    OnTheSheet::Cancel => OnTheSheet::Cancel,
                };
                None
            }
            RecordKey::Press => self.the_confirmation_is_pressed(),
            RecordKey::Close => {
                self.put_back.sheet = None;
                None
            }
            RecordKey::Newer
            | RecordKey::Older
            | RecordKey::Newest
            | RecordKey::Oldest
            | RecordKey::ReadAgain
            | RecordKey::Nothing => None,
        }
    }

    /// The button the keyboard is on, pressed — or a button pressed by
    /// pointer, which moves the keyboard to it first.
    pub fn the_confirmation_is_pressed(&mut self) -> Option<RecordOpened> {
        let sheet = self.put_back.sheet.as_mut()?;
        match sheet.on {
            OnTheSheet::Cancel => {
                if !sheet.waiting {
                    self.put_back.sheet = None;
                }
                None
            }
            OnTheSheet::PutBack => {
                let Ok(plan) = &sheet.plan else {
                    return None;
                };
                if !sheet.can_put_back() {
                    return None;
                }
                sheet.waiting = true;
                Some(RecordOpened::PutBackConfirmed {
                    place: plan.place.clone(),
                    folders: plan.folders_as_asked(),
                })
            }
        }
    }

    /// Put the keyboard on one of the confirmation's buttons.
    pub fn the_keyboard_is_on(&mut self, on: OnTheSheet) {
        if let Some(sheet) = self.put_back.sheet.as_mut() {
            sheet.on = on;
        }
    }

    /// Move the keyboard to the next change that offers *Put back…*, newer
    /// to older (`forwards`) or back, and the view with it so it is the first
    /// in view.
    fn the_keyboard_moves(&mut self, forwards: bool) {
        let Some(account) = self.shown.account() else {
            return;
        };
        let told = account.told();
        let places = &self.put_back.places;
        // Positions newest first: the order the person reads.
        let mut offered: Vec<usize> = Vec::new();
        for (at, here) in told.iter().enumerate().rev() {
            if PutBackHere::of(here, places) == PutBackHere::Offered {
                offered.push(at);
            }
        }
        let now_at = self
            .put_back
            .chosen
            .and_then(|chosen| offered.iter().position(|at| *at == chosen));
        let next = match (now_at, forwards) {
            (None, true) => offered.first(),
            (None, false) => offered.last(),
            (Some(here), true) => offered.get(here + 1).or_else(|| offered.first()),
            (Some(here), false) => here
                .checked_sub(1)
                .and_then(|before| offered.get(before))
                .or_else(|| offered.last()),
        };
        if let Some(at) = next.copied() {
            self.put_back.chosen = Some(at);
            self.moved_past = told.len().saturating_sub(at + 1);
        }
    }

    /// Close the window. The record is untouched; the account is dropped.
    pub fn closed(&mut self) {
        self.shown.taken_down();
        self.refusal = None;
        self.moved_past = 0;
        self.put_back.chosen = None;
        self.put_back.sheet = None;
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
                put_back: &self.put_back,
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
        self.put_back.chosen = None;
        self.put_back.sheet = None;
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
