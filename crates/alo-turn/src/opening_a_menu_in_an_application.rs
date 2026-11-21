//! **`accessible.open_menu`**: one thing's right-click menu opened in an
//! application's window, by the application's own action (2026-11-17) — a
//! change, so at *ask each time* it runs only after one approval of *open the
//! right-click menu of “{name}” in {application}*, and at full access it runs
//! at once, recorded the same way.
//!
//! The steps are `alo_adapters::OpeningAMenu`'s: the grants asked again now,
//! the application here; then the thing found on the
//! screen by its exact name, never a password field, and its own menu action
//! done once. The tree is opened after every refusal that needs no tree, for
//! this one act, and dropped when it ends. A terminal is reached only where the
//! person allowed their agent one (ADR 0111).

use alo_adapters::{NotWalked, OpeningAMenu, TreeFault};
use alo_capability::{Authorised, Grants, Refused};
use alo_record::Entry;

use crate::machine::Machine;
use crate::refusing::NotDone;

/// One thing's menu alo opened — or asked to, of an application that did not
/// answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AMenuOpened {
    /// The application.
    application: String,
    /// The thing's name, as the application shows it.
    name: String,
    /// Whether the application did not answer.
    unanswered: bool,
}

impl AMenuOpened {
    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The thing's name, as the application shows it.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the application did not answer, so whether the menu is open is
    /// not known.
    #[must_use]
    pub const fn unanswered(&self) -> bool {
        self.unanswered
    }
}

/// Open one approved menu, once.
///
/// **A menu the application did not answer about is recorded as having run**,
/// with the sentence saying so beside it; every other way this ends without an
/// action is a refusal, written down as one.
pub(crate) fn opening_a_menu_in_an_application(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<AMenuOpened, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refusing = |refused: Refused| {
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

    let opening = match OpeningAMenu::of(
        authorised,
        grants,
        &machine.the_applications_installed_now(),
        strings,
    ) {
        Ok(opening) => opening,
        Err(refused) => return refusing(refused),
    };
    let application = opening.application().to_owned();

    // Opened now, for this one act, and dropped when it ends.
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
    let entry = match opened.said() {
        Some(said) => entry.telling(std::slice::from_ref(said)),
        None => entry,
    };
    (
        entry,
        Ok(AMenuOpened {
            application,
            name: opened.name().to_owned(),
            unanswered: opened.unanswered(),
        }),
    )
}
