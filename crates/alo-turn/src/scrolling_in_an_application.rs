//! **`accessible.scroll_to` and `accessible.scroll_page`**: one application's
//! window scrolled, to a named thing or by a page (2026-11-17) — a change, so
//! at *ask each time* it runs only after one approval of *scroll {application}
//! until “{name}” is on the screen* or *scroll {toward} by one page in
//! {application}*, and at full access it runs at once, recorded the same way.
//!
//! The steps are `alo_adapters::Scrolling`'s: the grants asked again at this
//! moment, the application here; then what to scroll to
//! found in the window as it is now, never a password field; then the
//! application asked once, and the thing's states asked again.
//!
//! What this file adds is the order around it: the tree is opened **after**
//! every refusal that needs no tree, for this scroll alone, and dropped when it
//! ends. A terminal is reached only where the person allowed their agent one
//! (ADR 0111): until then the grants refuse it before any tree is opened.

use alo_adapters::{Asked, NotWalked, Scrolling, TreeFault};
use alo_capability::{Authorised, Grants, Refused};
use alo_record::Entry;

use crate::machine::Machine;
use crate::refusing::NotDone;

/// One scroll alo made in an application's window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AWindowScrolled {
    /// The application.
    application: String,
    /// What was asked: the thing's name, or `down` or `up` for a page.
    to: String,
    /// Whether it was a page rather than a named thing.
    a_page: bool,
    /// Whether what was scrolled to reads as on the screen afterwards.
    now_showing: bool,
}

impl AWindowScrolled {
    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The thing's name — or, for a page, `down` or `up`.
    #[must_use]
    pub fn to(&self) -> &str {
        &self.to
    }

    /// Whether it was a page rather than a named thing.
    #[must_use]
    pub const fn a_page(&self) -> bool {
        self.a_page
    }

    /// Whether what was scrolled to reads as on the screen afterwards.
    #[must_use]
    pub const fn now_showing(&self) -> bool {
        self.now_showing
    }
}

/// Scroll once, as approved.
///
/// **A scroll the application said yes to is recorded as having run**, with
/// what the window said afterwards beside it where it was not what was asked;
/// every other way this ends without a scroll is a refusal, written down as
/// one.
pub(crate) fn scrolling_in_an_application(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<AWindowScrolled, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refusing = |refused: Refused| {
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

    let scrolling = match Scrolling::of(
        authorised,
        grants,
        &machine.the_applications_installed_now(),
        strings,
    ) {
        Ok(scrolling) => scrolling,
        Err(refused) => return refusing(refused),
    };
    let application = scrolling.application().to_owned();

    // Opened now, for this one scroll, and dropped when it ends.
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
    let scrolled = match scrolling.scroll(tree.as_ref(), strings) {
        Ok(scrolled) => scrolled,
        Err(refused) => return refusing(refused),
    };
    drop(tree);

    let entry = Entry::ran(scrolled.authorised(), strings);
    let entry = match scrolled.said() {
        Some(said) => entry.telling(std::slice::from_ref(said)),
        None => entry,
    };
    let (to, a_page) = match scrolled.asked() {
        Asked::To(name) => (name.clone(), false),
        Asked::APage(toward) => (toward.named().to_owned(), true),
    };
    (
        entry,
        Ok(AWindowScrolled {
            application,
            to,
            a_page,
            now_showing: scrolled.now_showing(),
        }),
    )
}
