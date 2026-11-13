//! **`accessible.read_window`**: what an application's windows show, read
//! inside the turn — a read, so it answers at once (ADR 0001 §5).
//!
//! The fourth executor family on this machine, and the first whose steps are
//! all somebody else's: `alo_adapters::ReadingWindows` asks the grants, then
//! whether the application is here, then whether it has an adapter of its own,
//! and only then is the tree opened; the walk is the one a person's own
//! `alo-app … read` takes, with the same bounds and a password field's contents
//! never asked for. What this file adds is the order around it — the tree is
//! opened **after** every refusal that needs no tree, and dropped when the read
//! ends — and the entry.
//!
//! # What the record keeps, and what it does not
//!
//! The entry is the call's own sentence, *read what {application} shows in its
//! windows*, exactly as every read's is — and **nothing that was read**. A
//! record of what was on a person's screen is a record of the person, which is
//! the owner's answer for `alo-app`'s `read` (*never written in the record*)
//! and holds here for the same reason: the gate asks that the read is written
//! down, not what it saw.

use std::sync::Arc;

use alo_adapters::{NotWalked, ReadingWindows, Shown, WorkingFully, nothing_to_read};
use alo_capability::{Authorised, Grants, Refused};
use alo_record::Entry;

use crate::machine::Machine;
use crate::refusing::NotDone;

/// What an application's windows showed, and what alo is told beside it.
///
/// **Shared, never copied**: `alo_adapters::Shown` is deliberately not `Clone`,
/// so that what was read is one value handed back once. The `Arc` lets this
/// travel inside [`crate::WhatAVerbDid`], which is `Clone`, without making a
/// second copy of anything a window showed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheWindowsShowed {
    /// What was read.
    shown: Arc<Shown>,
    /// What alo is told beside it, in the person's language: that not all of
    /// it was read, and for a read that found no text, which switch is the
    /// likeliest reason.
    told: Vec<String>,
}

impl TheWindowsShowed {
    /// What was read.
    #[must_use]
    pub fn shown(&self) -> &Shown {
        &self.shown
    }

    /// What alo is told beside it, in the person's language.
    #[must_use]
    pub fn told(&self) -> &[String] {
        &self.told
    }
}

/// Read one application's windows, once, now.
///
/// Answers the record's entry and what happened, as every executor does: a
/// refusal is written down exactly as a read is.
///
/// # The refusals
///
/// The grants' own words for an application nobody granted;
/// `alo-applications`' for one this machine does not have; the fallback's for
/// one with an adapter of its own, for no window on the screen, for nothing on
/// this machine to read windows through, and for an application that stopped
/// answering. **Every one of them means nothing was read.**
pub(crate) fn reading_an_applications_windows(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<TheWindowsShowed, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refusing = |refused: Refused| {
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

    // Everything that needs no tree, first: granted, here, and no adapter of
    // its own. **The tree has not been opened yet**, so an ungranted
    // application's windows are never asked anything.
    let reading = match ReadingWindows::of(
        authorised,
        machine.the_adapters(),
        grants,
        machine.the_applications_installed(),
        strings,
    ) {
        Ok(reading) => reading,
        Err(refused) => return refusing(refused),
    };
    let application = reading.application().to_owned();

    // Opened now, for this read, and dropped when it ends.
    let (tree, fully) = match machine.the_windows_on_the_screen() {
        Some(windows) => (windows.opening(), windows.working_fully(&application)),
        None => (
            Err(alo_adapters::TreeFault::NotThere),
            WorkingFully::NothingMoreToTurnOn,
        ),
    };
    let tree = match tree {
        Ok(tree) => tree,
        Err(fault) => {
            let said = NotWalked::Tree(fault).said(&application, None, strings);
            return refusing(Refused::worded_elsewhere(call, said));
        }
    };
    let read = match reading.read(tree.as_ref(), strings) {
        Ok(read) => read,
        Err(refused) => return refusing(refused),
    };
    drop(tree);

    let (authorised, shown) = read.into_parts();
    let mut told: Vec<String> = shown
        .not_all_read_said(strings)
        .map(alo_strings::Said::into_text)
        .into_iter()
        .collect();
    // **A read that found nothing says why**, window by window — never an
    // empty answer a model would take for an empty document — naming the one
    // switch that would help (`alo_adapters::nothing_to_read`).
    if !shown.shows_any_text() {
        told.extend(
            shown
                .windows()
                .iter()
                .map(|window| nothing_to_read(window.title(), &application, fully, strings))
                .map(alo_strings::Said::into_text),
        );
    }
    (
        Entry::ran(&authorised, strings),
        Ok(TheWindowsShowed {
            shown: Arc::new(shown),
            told,
        }),
    )
}
