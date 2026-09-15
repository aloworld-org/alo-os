//! Who a grant is for: an agent, or an application.
//!
//! [ADR 0040](../../../docs/decisions/0040-what-an-applications-grant-is-over.md),
//! part 2. ADR 0005 says a portal request is ADR 0001's sentence with
//! *application* in place of *agent*, and that a person keeps one list rather
//! than two. So both kinds of grantee hold grants on the one
//! [`crate::Grants`], and a grantee **says which kind it is**, because the two
//! are refused in different words and treated differently when the person
//! declines the agent (ADR 0009).
//!
//! # Two kinds that never answer for each other
//!
//! An agent called `org.gnome.Cheese` and the application `org.gnome.Cheese`
//! are two grantees. The kind is part of the identity, compared exactly like
//! the name, so a grant to one is never a grant to the other: letting a name
//! stand for both would be a widening nobody asked for.
//!
//! # And two doors
//!
//! An agent asks through [`crate::Grants::permitting`] and is refused with
//! [`crate::NotGranted`], whose words speak of agents and of picking a folder.
//! An application asks through [`crate::Grants::allowing`] with an
//! [`Applicant`], and is refused with [`crate::NotAllowed`], whose words never
//! call it an agent. The search behind both doors is one search.
//!
//! # Written down the way it always was
//!
//! An agent serialises as its name — the shape every record and every kept
//! value already has. An application serialises as `{ "application": … }`, so
//! nothing that reads an agent's name can mistake an application's for one.

use serde::{Deserialize, Serialize};

/// Who a grant is for: one agent or one application, by the name the system
/// knows it by.
///
/// Compared exactly, like every other identity in this crate. Two agents whose
/// names differ only in case are two agents, and an agent and an application
/// that share a name are two grantees.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Grantee(Who);

/// The two kinds, private so that nothing builds one without trimming its name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
enum Who {
    /// An agent, written as its bare name.
    Agent(String),
    /// An application, written under its own key.
    Application {
        /// The application's identifier.
        application: String,
    },
}

impl Grantee {
    /// The agent known by this name.
    #[must_use]
    pub fn named(name: &str) -> Self {
        Self(Who::Agent(name.trim().to_owned()))
    }

    /// The name, as the system knows it — an agent's name or an application's
    /// identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match &self.0 {
            Who::Agent(name) | Who::Application { application: name } => name,
        }
    }

    /// Whether this grantee is an application rather than an agent.
    #[must_use]
    pub const fn is_an_application(&self) -> bool {
        matches!(self.0, Who::Application { .. })
    }

    /// The application this grantee is, or [`None`] for an agent.
    #[must_use]
    pub fn applicant(&self) -> Option<Applicant> {
        match &self.0 {
            Who::Application { application } => Some(Applicant(application.clone())),
            Who::Agent(_) => None,
        }
    }
}

/// An application asking for something through a portal.
///
/// Its own type rather than a [`Grantee`] with a flag, so that the door an
/// application asks through ([`crate::Grants::allowing`]) cannot be handed an
/// agent, and the refusal that comes back cannot speak of one.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Applicant(String);

impl Applicant {
    /// The application known by this identifier, like `org.gnome.Cheese`.
    ///
    /// Trimmed, and otherwise exactly as given. Whether the identifier is one
    /// at all is checked where it arrives from outside — a grant made to an
    /// empty one is refused by [`crate::Grant::checked_for`].
    #[must_use]
    pub fn named(id: &str) -> Self {
        Self(id.trim().to_owned())
    }

    /// The identifier, as the system knows it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// This application as the grantee a grant to it names.
    #[must_use]
    pub fn grantee(&self) -> Grantee {
        Grantee(Who::Application {
            application: self.0.clone(),
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **An agent and an application that share a name are two grantees.**
    #[test]
    fn an_agent_and_an_application_never_answer_for_each_other() {
        let agent = Grantee::named("org.gnome.Cheese");
        let application = Applicant::named("org.gnome.Cheese").grantee();
        assert_ne!(agent, application);
        assert_eq!(agent.as_str(), application.as_str());
        assert!(!agent.is_an_application());
        assert!(application.is_an_application());
        assert_eq!(agent.applicant(), None);
        assert_eq!(
            application.applicant(),
            Some(Applicant::named("org.gnome.Cheese"))
        );
    }

    /// Names are trimmed and otherwise matched exactly.
    #[test]
    fn a_name_is_trimmed_and_otherwise_exact() {
        assert_eq!(
            Applicant::named("  org.gnome.Cheese "),
            Applicant::named("org.gnome.Cheese")
        );
        assert_ne!(
            Applicant::named("org.gnome.cheese"),
            Applicant::named("org.gnome.Cheese")
        );
    }

    /// **An agent is written down as it always was**, and an application
    /// under a key of its own, so neither can be read back as the other.
    #[test]
    fn each_kind_is_written_down_as_itself() {
        let agent = Grantee::named("@files");
        assert_eq!(serde_json::to_string(&agent).unwrap(), r#""@files""#);
        let application = Applicant::named("org.gnome.Cheese").grantee();
        let written = serde_json::to_string(&application).unwrap();
        assert_eq!(written, r#"{"application":"org.gnome.Cheese"}"#);

        let read: Grantee = serde_json::from_str(&written).unwrap();
        assert_eq!(read, application);
        let read: Grantee = serde_json::from_str(r#""@files""#).unwrap();
        assert_eq!(read, agent);
    }
}
