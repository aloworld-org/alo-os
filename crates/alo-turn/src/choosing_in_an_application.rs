//! **`accessible.choose_option` and `accessible.select_item`**: one option of
//! a drop-down, or one item of a list, chosen in an application's window — a
//! change, so it runs only after one approval of *choose “{option}” in the
//! drop-down “{name}” in {application}* or *select “{name}” in
//! {application}*, or without a card at full access (added 2026-11-17, the
//! owner's *"Yes do all"*).
//!
//! The steps are `alo_adapters::Choosing`'s: the grants asked again at this
//! moment, the application here, no adapter of its own, and then the
//! drop-down or item looked for again in the window as it is now. What this
//! file adds is the order around it: the tree is opened **after** every
//! refusal that needs no tree, for this one choice, and dropped when it ends;
//! and once the application was asked, the window is **read back** for up to
//! [`READ_BACK_FOR`], so what alo says happened is what the window shows.
//!
//! # Recorded as run once the application was asked
//!
//! A choice the application was asked to make is recorded as having run,
//! whether or not the window then shows it — with the sentence saying it is
//! not seen chosen, or that the application did not answer. Every way this
//! ends before the application is asked is a refusal, written down as one.
//!
//! # Nothing is typed, and a terminal is the person's choice
//!
//! Neither verb carries text to type, and **selecting never opens an item**
//! (`alo_adapters::choosing::SELECTING_AN_ITEM`). A terminal is reached only
//! where the person allowed their agent one (ADR 0111): until then the grants
//! refuse it at every level, full access included, before any tree is opened.

use std::time::{Duration, Instant};

use alo_adapters::{Choosing, NotWalked, ToChoose, TreeFault};
use alo_capability::{Authorised, Grants, Refused};
use alo_record::Entry;

use crate::machine::Machine;
use crate::refusing::NotDone;

/// How long the window is read back for, after the application was asked.
pub const READ_BACK_FOR: Duration = Duration::from_secs(1);

/// How long between two readings.
const BETWEEN_READS: Duration = Duration::from_millis(100);

/// One choice alo made — or asked for, of an application that did not answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AChoiceMade {
    /// The application.
    application: String,
    /// The drop-down or the item, by what it shows.
    name: String,
    /// The option chosen, for a drop-down; [`None`] for an item.
    option: Option<String>,
    /// Whether the window showed it chosen when it was read back.
    seen: bool,
    /// Whether the application did not answer.
    unanswered: bool,
}

impl AChoiceMade {
    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The drop-down or the item, by what it shows.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The option chosen, for a drop-down; [`None`] for an item selected.
    #[must_use]
    pub fn option(&self) -> Option<&str> {
        self.option.as_deref()
    }

    /// Whether the window showed it chosen when it was read back.
    #[must_use]
    pub const fn seen(&self) -> bool {
        self.seen
    }

    /// Whether the application did not answer, so whether it chose is not
    /// known.
    #[must_use]
    pub const fn unanswered(&self) -> bool {
        self.unanswered
    }
}

/// Choose one approved option or item, once.
pub(crate) fn choosing_in_an_application(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<AChoiceMade, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refusing = |refused: Refused| {
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

    let choosing = match Choosing::of(
        authorised,
        machine.the_adapters(),
        grants,
        &machine.the_applications_installed_now(),
        strings,
    ) {
        Ok(choosing) => choosing,
        Err(refused) => return refusing(refused),
    };
    let application = choosing.application().to_owned();

    // Opened now, for this one choice, and dropped when it ends.
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
    let chosen = match choosing.choose(tree.as_ref(), strings) {
        Ok(chosen) => chosen,
        Err(refused) => return refusing(refused),
    };
    let seen = !chosen.unanswered() && read_back(&chosen, tree.as_ref());
    drop(tree);

    let entry = Entry::ran(chosen.authorised(), strings);
    let entry = match chosen.said(seen, strings) {
        Some(said) => entry.telling(&[said]),
        None => entry,
    };
    let (name, option) = match chosen.to_choose() {
        ToChoose::AnOption { drop_down, option } => (drop_down.clone(), Some(option.clone())),
        ToChoose::AnItem { item } => (item.clone(), None),
    };
    (
        entry,
        Ok(AChoiceMade {
            application,
            name,
            option,
            seen,
            unanswered: chosen.unanswered(),
        }),
    )
}

/// Read the window back until it shows the choice made, or
/// [`READ_BACK_FOR`] has passed.
fn read_back(chosen: &alo_adapters::Chosen, tree: &dyn alo_adapters::AccessibilityTree) -> bool {
    let until = Instant::now() + READ_BACK_FOR;
    loop {
        if chosen.shows_chosen(tree) {
            return true;
        }
        if Instant::now() >= until {
            return false;
        }
        std::thread::sleep(BETWEEN_READS);
    }
}
