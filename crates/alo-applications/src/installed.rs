//! What this machine has, by identifier — the list a call is checked against.
//!
//! Whatever builds this list reads it off the machine: desktop entries, the
//! sandbox's own list of what is installed. That is Linux's, and this crate
//! holds no opinion about where the entries come from — exactly as
//! `alo_capability::Grants` holds none about where grants are stored. What is
//! *here* is the part a test can reach on any machine: what an entry has to be,
//! how one is matched, and what happens when two claim the same identifier.
//!
//! # Matched exactly, like a grant
//!
//! `alo_capability::Reach` matches an application identifier exactly, with no
//! case folding, because matching loosely matches more than the person picked.
//! The same list has to be matched the same way or the two questions — *is it
//! granted* and *is it here* — could be answered about different applications.
//!
//! # The first entry wins, and that is a security decision
//!
//! Two entries claiming one identifier is a packaging fault, and the list has
//! to do something about it. Taking the later one would mean something
//! installed after a grant was made could take the identifier that grant is
//! over, which turns *I allowed the agent to open Blender* into *I allowed the
//! agent to open whatever now answers to that name*. So the first is kept, the
//! second is refused, and [`Installed::add`] says which happened rather than
//! swallowing it.
//!
//! # This list is never how an agent finds out what is installed
//!
//! There is no verb that reads it. Enumerating somebody's installed
//! applications is a fingerprint of who they are and what they do, and
//! `docs/features.md` promises the agent learns what is open from the context
//! offered at invocation rather than by asking. [`crate::Reaching`] consults
//! this list **after** the grants have said yes, so a refusal never carries the
//! answer either.

use std::collections::BTreeMap;

use crate::application::Application;

/// The applications this machine has.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Installed {
    /// Each application, by the identifier it is matched on.
    applications: BTreeMap<String, Application>,
}

impl Installed {
    /// A machine with nothing installed on it.
    ///
    /// The honest starting state: something has to go and look, and until it
    /// has, nothing is reachable.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// A machine with these applications on it.
    ///
    /// Where two claim one identifier the first is kept, for the reason at the
    /// top of this file. Use [`Installed::add`] where knowing that happened
    /// matters.
    pub fn holding(applications: impl IntoIterator<Item = Application>) -> Self {
        let mut installed = Self::nothing();
        for application in applications {
            installed.add(application);
        }
        installed
    }

    /// Put an application on the list.
    ///
    /// Answers `false` if this identifier is already there — the entry already
    /// on the list is kept, and nothing about it changes.
    pub fn add(&mut self, application: Application) -> bool {
        if self.applications.contains_key(application.identifier()) {
            return false;
        }
        self.applications
            .insert(application.identifier().to_owned(), application);
        true
    }

    /// The application with this identifier, matched exactly.
    #[must_use]
    pub fn knows(&self, identifier: &str) -> Option<&Application> {
        self.applications.get(identifier)
    }

    /// Whether this machine has it.
    #[must_use]
    pub fn has(&self, identifier: &str) -> bool {
        self.applications.contains_key(identifier)
    }

    /// Everything installed, by identifier.
    ///
    /// For a settings panel showing what a grant could be made over. Nothing an
    /// agent asks for reaches this — see the top of this file.
    pub fn all(&self) -> impl Iterator<Item = &Application> {
        self.applications.values()
    }

    /// How many there are.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.applications.len()
    }

    /// Whether this machine has nothing on it.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.applications.is_empty()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn blender() -> Application {
        Application::called("org.blender.Blender", "Blender").unwrap()
    }

    #[test]
    fn a_machine_with_nothing_installed_knows_nothing() {
        let installed = Installed::nothing();
        assert!(installed.is_empty());
        assert_eq!(installed.how_many(), 0);
        assert!(!installed.has("org.blender.Blender"));
        assert!(installed.knows("org.blender.Blender").is_none());
    }

    /// **Matched exactly, as a grant is.** A list that matched kindly would
    /// answer *yes, it is here* about an application no grant covers.
    #[test]
    fn an_identifier_is_matched_exactly() {
        let installed = Installed::holding([blender()]);
        assert!(installed.has("org.blender.Blender"));
        assert!(!installed.has("org.blender.blender"));
        assert!(!installed.has("org.blender.Blender2"));
        assert!(!installed.has(" org.blender.Blender "));
        assert_eq!(
            installed
                .knows("org.blender.Blender")
                .and_then(Application::name),
            Some("Blender")
        );
    }

    /// **The first entry keeps the identifier.** Otherwise something installed
    /// after a grant was made could take the name that grant is over.
    #[test]
    fn a_second_entry_claiming_one_identifier_does_not_take_it() {
        let mut installed = Installed::holding([blender()]);
        let impostor =
            Application::called("org.blender.Blender", "Blender (free edition)").unwrap();
        assert!(!installed.add(impostor));
        assert_eq!(installed.how_many(), 1);
        assert_eq!(
            installed
                .knows("org.blender.Blender")
                .and_then(Application::name),
            Some("Blender"),
            "the second entry took the identifier the first one holds"
        );

        // And the same through the constructor a daemon would use.
        let both = Installed::holding([
            blender(),
            Application::called("org.blender.Blender", "something else").unwrap(),
        ]);
        assert_eq!(both.how_many(), 1);
        assert_eq!(
            both.knows("org.blender.Blender")
                .and_then(Application::name),
            Some("Blender")
        );
    }

    #[test]
    fn what_is_installed_can_be_read_for_a_settings_panel() {
        let mut installed =
            Installed::holding([blender(), Application::identified("org.gimp.GIMP").unwrap()]);
        let identifiers: Vec<_> = installed.all().map(Application::identifier).collect();
        assert_eq!(identifiers, ["org.blender.Blender", "org.gimp.GIMP"]);
        assert!(installed.add(Application::identified("org.kde.okular").unwrap()));
        assert_eq!(installed.how_many(), 3);
    }
}
