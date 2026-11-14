//! **The applications this machine has, as they are when a verb asks** — not
//! as they were when the daemon started.
//!
//! # What was wrong, read off the code on 2026-11-14
//!
//! `crate::Machine` held an `alo_applications::Installed` read once, when
//! `alo-agentd` started, and nothing ever read it again. `alo-agentd` starts
//! with the person's session (`BindsTo=user@1000.service`), and on a fresh
//! machine `alo-shipping` installs the first eight applications from the
//! network at the lowest priority, *after* the network is up — so a person who
//! signs in before it finishes has a daemon that will tell them, for the whole
//! session, that the applications the desktop has just announced are not
//! installed. The same was true of every application alo itself installed:
//! `install_application` succeeded, and `open_application` for it refused until
//! the next sign-in.
//!
//! Whether that is what happened on the walk of 2026-11-14 is **not
//! confirmed** — it fits, and the walk did not record when each service
//! started. What is confirmed is that the list could not change, and this is
//! what lets it.
//!
//! # Read when asked, and nothing watches
//!
//! [`SaysWhatIsInstalled`] is asked each time a verb needs the list, which is
//! a walk of a few directories of desktop entries — the same walk the daemon
//! did once. Nothing reads it between verbs, so there is no background reader
//! here (`CLAUDE.md`'s *context is offered, never watched*): the list is read
//! because an agent's call, which a person caused, needs it.

use alo_applications::Installed;

/// What says which applications this machine has, right now.
///
/// `alo-agentd` reads this machine's desktop entries; a test hands in a list.
/// This crate touches no disk, so it asks.
pub trait SaysWhatIsInstalled {
    /// The applications installed now.
    ///
    /// A directory that cannot be read is not an error and a file that does
    /// not parse is skipped — `alo_installed::read`'s own decisions — so this
    /// always answers a list.
    fn what_is_installed(&self) -> Installed;
}

/// The applications a machine's verbs are held to: a list fixed when the
/// machine was made, or one asked for each time.
pub enum TheApplicationsHere<'a> {
    /// A list that does not change: a machine that offers no application verb
    /// (where it is empty and nothing asks), and a test.
    AsTheyWere(Installed),
    /// Asked each time a verb needs it, which is how a running machine holds
    /// them.
    AskedEachTime(&'a dyn SaysWhatIsInstalled),
}

impl TheApplicationsHere<'_> {
    /// The applications installed now.
    #[must_use]
    pub fn now(&self) -> Installed {
        match self {
            Self::AsTheyWere(installed) => installed.clone(),
            Self::AskedEachTime(saying) => saying.what_is_installed(),
        }
    }
}

impl Default for TheApplicationsHere<'_> {
    /// Nothing installed, and nothing to ask.
    fn default() -> Self {
        Self::AsTheyWere(Installed::nothing())
    }
}

impl From<Installed> for TheApplicationsHere<'_> {
    fn from(installed: Installed) -> Self {
        Self::AsTheyWere(installed)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_applications::Application;
    use std::cell::RefCell;

    /// A machine whose list changes while it runs, as a fresh one's does while
    /// `alo-shipping` installs.
    struct AMachineStillInstalling(RefCell<Installed>);

    impl SaysWhatIsInstalled for AMachineStillInstalling {
        fn what_is_installed(&self) -> Installed {
            self.0.borrow().clone()
        }
    }

    /// **An application that arrives after the machine was made is seen.**
    /// The fault this file exists for: a list read once never would.
    #[test]
    fn an_application_installed_after_the_machine_was_made_is_seen() {
        let machine = AMachineStillInstalling(RefCell::new(Installed::nothing()));
        let here = TheApplicationsHere::AskedEachTime(&machine);
        assert!(here.now().is_empty());
        machine
            .0
            .borrow_mut()
            .add(Application::called("com.vscodium.codium", "VSCodium").unwrap());
        assert!(here.now().has("com.vscodium.codium"));
    }

    /// A fixed list is the list it was given, every time.
    #[test]
    fn a_list_fixed_when_the_machine_was_made_stays_that_list() {
        let here = TheApplicationsHere::from(Installed::holding([Application::identified(
            "org.gnome.TextEditor",
        )
        .unwrap()]));
        assert!(here.now().has("org.gnome.TextEditor"));
        assert!(TheApplicationsHere::default().now().is_empty());
    }
}
