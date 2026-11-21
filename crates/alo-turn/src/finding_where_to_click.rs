//! **A click by description, in two halves under one approval**
//! ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md),
//! amended 2026-11-21: *"Yes, click by description"*).
//!
//! The first half runs where every approved change runs
//! ([`Turning::running`]) and ends, once every check has passed and the window
//! is pictured, with [`WhatAVerbDid::AThingIsToBeFound`]: what the reader the
//! person chose is to find, and the picture to find it in. **The approval
//! waits inside the turn**, so the daemon that asks the reader holds nothing it
//! could spend twice.
//!
//! The second half is [`Turning::clicking_where_it_was_found`], handed the
//! reader's answer. Every road out of either half writes exactly one entry, and
//! **a click whose answer never comes is written down as refused** — when the
//! next click is readied, or when the turn ends
//! ([`Turning::a_click_never_found`]).

use alo_capability::{Authorised, Grants};
use alo_strings::Said;

use crate::clicking_in_an_applications_window::{
    clicking_where_it_was_found, never_found, readying_a_click,
};
use crate::refusing::NotDone;
use crate::turning::Turning;
use crate::what_a_verb_did::WhatAVerbDid;

impl Turning<'_, '_> {
    /// **The first half**: every check and the picture before, or the refusal
    /// written down. Nothing is clicked.
    ///
    /// # Errors
    /// [`NotDone::Refused`], written down; [`NotDone::NotRecorded`] where the
    /// record would not take the entry.
    pub(crate) fn readying_a_click(
        &mut self,
        authorised: Authorised,
        grants: &Grants,
    ) -> Result<WhatAVerbDid, NotDone> {
        // **One waits at a time**: one left from before is written down as
        // never made, first.
        self.a_click_never_found()?;
        let at_full_access = grants.has_full_access(self.grantee());
        let origin = self.origin().cloned();
        match readying_a_click(self.machine(), authorised, grants, origin.as_ref()) {
            Ok((ready, thing)) => {
                *self.the_click_to_find() = Some(ready.decided_at_full_access(at_full_access));
                Ok(WhatAVerbDid::AThingIsToBeFound(Box::new(thing)))
            }
            Err(refused) => {
                let (entry, not_done) = *refused;
                let entry = self.as_the_level_was(entry, grants);
                self.writing_down(entry)?;
                Err(not_done)
            }
        }
    }

    /// **The second half**: `read` is the reader's one line about the picture
    /// [`WhatAVerbDid::AThingIsToBeFound`] carried, or the sentence saying why
    /// no reader could be asked. Exactly one thing found is clicked, once;
    /// anything else is refused, and nothing is clicked.
    ///
    /// # Errors
    /// [`NotDone::NotAnswered`] where no click waits to be found (with no
    /// number, since it is not a proposal);
    /// [`NotDone::Refused`] for every way it was not clicked, written down;
    /// [`NotDone::NotRecorded`] where the record would not take the entry.
    pub fn clicking_where_it_was_found(
        &mut self,
        read: Result<String, Said>,
    ) -> Result<WhatAVerbDid, NotDone> {
        let Some(ready) = self.the_click_to_find().take() else {
            // **No number stands for it**: a click waiting to be found is not a
            // proposal, and a caller with nothing waiting has nothing to answer.
            return Err(NotDone::NotAnswered(
                alo_capability::AnswerError::NothingWaiting { number: 0 },
            ));
        };
        let at_full_access = ready.at_full_access();
        let (entry, made) = clicking_where_it_was_found(self.machine(), ready, read);
        let kept_as = self.kept_as().map(str::to_owned);
        let entry = match kept_as {
            Some(named) if entry.happened().ran() => entry.kept_as(&named),
            _ => entry,
        };
        let entry = if at_full_access {
            entry.at_full_access()
        } else {
            entry
        };
        self.writing_down(entry)?;
        made.map(WhatAVerbDid::from)
    }

    /// **A click still waiting to be found is written down as never made**,
    /// and is gone. Nothing, where none waits.
    ///
    /// # Errors
    /// [`NotDone::NotRecorded`] where the record would not take the entry.
    pub fn a_click_never_found(&mut self) -> Result<(), NotDone> {
        let Some(ready) = self.the_click_to_find().take() else {
            return Ok(());
        };
        let at_full_access = ready.at_full_access();
        let entry = never_found(self.machine(), ready);
        let entry = if at_full_access {
            entry.at_full_access()
        } else {
            entry
        };
        self.writing_down(entry)
    }
}
