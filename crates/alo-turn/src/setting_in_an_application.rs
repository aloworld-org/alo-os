//! **`accessible.set_value`**: one slider or number box in an application's
//! window moved to a whole number — a change, so it runs only after one
//! approval of *move “{name}” in {application} to {value}*, or without a card
//! at full access (added 2026-11-17, the owner's *"Yes do all"*).
//!
//! The steps are `alo_adapters::Setting`'s: the grants asked again at this
//! moment, the application here, no adapter of its own, and then the control
//! looked for again in the window as it is now, its own range asked, and a
//! number outside it refused before anything moves. What this file adds is
//! the order around it: the tree is opened **after** every refusal that needs
//! no tree, for this one move, and dropped when it ends; and once the
//! application was asked, the control is **read back** for up to
//! [`READ_BACK_FOR`].
//!
//! A move the application was asked to make is recorded as run whether or not
//! it reads back as the number asked for, with the sentence saying so beside
//! it. Every way this ends before the application is asked is a refusal,
//! written down as one. Nothing is typed, and a terminal nobody allowed their
//! agent is refused by the grants at every level before any tree is opened
//! (ADR 0111).

use std::time::{Duration, Instant};

use alo_adapters::{NotWalked, Setting, TreeFault};
use alo_capability::{Authorised, Grants, Refused};
use alo_record::Entry;

use crate::machine::Machine;
use crate::refusing::NotDone;

/// How long the control is read back for, after the application was asked.
pub const READ_BACK_FOR: Duration = Duration::from_secs(1);

/// How long between two readings.
const BETWEEN_READS: Duration = Duration::from_millis(100);

/// One slider or number box alo moved — or asked to, of an application that
/// did not answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AValueSet {
    /// The application.
    application: String,
    /// The control, by its name.
    name: String,
    /// The number asked for.
    value: i64,
    /// Whether it read back as that number.
    seen: bool,
    /// Whether the application did not answer.
    unanswered: bool,
}

impl AValueSet {
    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The control, by its name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The number asked for.
    #[must_use]
    pub const fn value(&self) -> i64 {
        self.value
    }

    /// Whether it read back as that number.
    #[must_use]
    pub const fn seen(&self) -> bool {
        self.seen
    }

    /// Whether the application did not answer, so whether it moved is not
    /// known.
    #[must_use]
    pub const fn unanswered(&self) -> bool {
        self.unanswered
    }
}

/// Move one approved slider or number box, once.
pub(crate) fn setting_in_an_application(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<AValueSet, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refusing = |refused: Refused| {
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

    let setting = match Setting::of(
        authorised,
        machine.the_adapters(),
        grants,
        &machine.the_applications_installed_now(),
        strings,
    ) {
        Ok(setting) => setting,
        Err(refused) => return refusing(refused),
    };
    let application = setting.application().to_owned();

    // Opened now, for this one move, and dropped when it ends.
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
    let set = match setting.set(tree.as_ref(), strings) {
        Ok(set) => set,
        Err(refused) => return refusing(refused),
    };
    let seen = !set.unanswered() && read_back(&set, tree.as_ref());
    drop(tree);

    let entry = Entry::ran(set.authorised(), strings);
    let entry = match set.said(seen, strings) {
        Some(said) => entry.telling(&[said]),
        None => entry,
    };
    (
        entry,
        Ok(AValueSet {
            application,
            name: set.name().to_owned(),
            value: set.value(),
            seen,
            unanswered: set.unanswered(),
        }),
    )
}

/// Read the control back until it shows the number asked for, or
/// [`READ_BACK_FOR`] has passed.
fn read_back(set: &alo_adapters::Set, tree: &dyn alo_adapters::AccessibilityTree) -> bool {
    let until = Instant::now() + READ_BACK_FOR;
    loop {
        if set.reads_back(tree) {
            return true;
        }
        if Instant::now() >= until {
            return false;
        }
        std::thread::sleep(BETWEEN_READS);
    }
}
