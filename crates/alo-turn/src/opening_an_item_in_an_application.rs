//! **`accessible.open_item`**: one item of a list in an application's window
//! opened, as a double-click would — a change, so at *ask each time* it runs
//! only after one approval of *open “{name}” in {application}*, and at full
//! access it runs with no card (added 2026-11-17; the owner: *"Yes but it
//! should be without okay on full access"*).
//!
//! The steps are `alo_adapters::Opening`'s: the grants asked again at this
//! moment — a terminal nobody allowed their agent is refused there, at every
//! level (ADR 0111) — the application here, no adapter of its own, and then
//! the item looked for again in the window as it is now. What this file adds
//! is the order around it: the tree is opened **after** every refusal that
//! needs no tree, for this one opening, and dropped when it ends.
//!
//! An opening the application was asked to make is recorded as run, with
//! *did not answer* beside it where it did not. Every way this ends before the
//! application is asked is a refusal, written down as one. Nothing reads back
//! that it opened: what opens may be another application's window.

use alo_adapters::{NotWalked, Opening, TreeFault};
use alo_capability::{Authorised, Grants, Refused};
use alo_record::Entry;

use crate::machine::Machine;
use crate::refusing::NotDone;

/// One item alo opened — or asked to, of an application that did not answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnItemOpened {
    /// The application.
    application: String,
    /// The item, by what it shows.
    name: String,
    /// Whether the application did not answer.
    unanswered: bool,
}

impl AnItemOpened {
    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The item, by what it shows.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the application did not answer, so whether it opened is not
    /// known.
    #[must_use]
    pub const fn unanswered(&self) -> bool {
        self.unanswered
    }
}

/// Open one approved item, once.
pub(crate) fn opening_an_item_in_an_application(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<AnItemOpened, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refusing = |refused: Refused| {
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

    let opening = match Opening::of(
        authorised,
        machine.the_adapters(),
        grants,
        &machine.the_applications_installed_now(),
        strings,
    ) {
        Ok(opening) => opening,
        Err(refused) => return refusing(refused),
    };
    let application = opening.application().to_owned();

    // Opened now, for this one opening, and dropped when it ends.
    let tree = match machine.the_windows_on_the_screen() {
        Some(windows) => windows.opening(),
        None => Err(TreeFault::NotThere),
    };
    let tree = match tree {
        Ok(tree) => tree,
        Err(fault) => {
            let said = NotWalked::Tree(fault).said(&application, None, strings);
            return refusing(Refused::worded_elsewhere(call, said));
        }
    };
    let opened = match opening.open(tree.as_ref(), strings) {
        Ok(opened) => opened,
        Err(refused) => return refusing(refused),
    };
    drop(tree);

    let entry = Entry::ran(opened.authorised(), strings);
    let entry = match opened.said(strings) {
        Some(said) => entry.telling(&[said]),
        None => entry,
    };
    (
        entry,
        Ok(AnItemOpened {
            application,
            name: opened.name().to_owned(),
            unanswered: opened.unanswered(),
        }),
    )
}
