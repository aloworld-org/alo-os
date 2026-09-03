//! What one line of the indicator is about.
//!
//! There are two kinds of egress on a machine running alo OS: one an agent
//! causes, under somebody's authority and against the organisation's policy
//! ([`Leaving`]), and one alo OS causes itself, with no agent behind it and for
//! one of an enumerated handful of reasons ([`OnItsOwn`]).
//!
//! **They are two decisions and one list.** They are decided differently on
//! purpose — [`crate::errand`] says why the policy an organisation stated about
//! where questions may be answered is not asked about a model download — but a
//! person looking at their machine to find out whether anything is leaving it
//! must have one place to look. A second indicator would be a second place to
//! forget, and the failure law 1 exists to prevent is not *the policy was
//! wrong*, it is *nobody could see it*.
//!
//! So this is the union, and it is small on purpose: what the shell needs of a
//! line is where it is going, what it says, and — only if somebody asks —
//! whose it was.

use alo_capability::Grantee;
use alo_strings::{Said, Strings};
use serde::Serialize;

use crate::destination::Destination;
use crate::itself::OnItsOwn;
use crate::leaving::Leaving;

/// What is behind one line on the indicator.
///
/// Serialises, as both halves of it do, so whatever draws the shell can be
/// handed the list. It does not deserialise: an egress read back off a disk
/// would be one nothing decided about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Showing {
    /// An agent caused this, under somebody's authority.
    Agent(Leaving),
    /// alo OS caused this itself, for one of the reasons on
    /// [`Errand`](crate::Errand).
    Itself(OnItsOwn),
}

impl Showing {
    /// Where it is going.
    #[must_use]
    pub fn destination(&self) -> &Destination {
        match self {
            Self::Agent(leaving) => leaving.destination(),
            Self::Itself(errand) => errand.destination(),
        }
    }

    /// Whose authority this is under, when it is under anybody's.
    ///
    /// `None` for an errand, and that is the honest answer rather than a
    /// placeholder: nobody granted alo OS permission to sign somebody in, so
    /// naming an authority would be inventing one. See [`crate::itself`].
    #[must_use]
    pub fn agent(&self) -> Option<&Grantee> {
        match self {
            Self::Agent(leaving) => Some(leaving.agent()),
            Self::Itself(_) => None,
        }
    }

    /// The agent's egress this is, if it is one.
    #[must_use]
    pub fn leaving(&self) -> Option<&Leaving> {
        match self {
            Self::Agent(leaving) => Some(leaving),
            Self::Itself(_) => None,
        }
    }

    /// The machine's own errand this is, if it is one.
    #[must_use]
    pub fn on_its_own(&self) -> Option<&OnItsOwn> {
        match self {
            Self::Agent(_) => None,
            Self::Itself(errand) => Some(errand),
        }
    }

    /// The line a person reads, in the language they read it in.
    ///
    /// One road to the words whichever kind it is, because a shell that had to
    /// branch to draw a line is a shell where one of the two branches is the
    /// one nobody translated.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::Agent(leaving) => leaving.said(strings),
            Self::Itself(errand) => errand.said(strings),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::errand::Errand;
    use crate::leaving::Why;
    use crate::testing::in_english;

    fn an_agents() -> Showing {
        Showing::Agent(Leaving::because(
            &Grantee::named("@files"),
            Why::Fetching,
            Destination::at("alo.example").unwrap(),
        ))
    }

    fn the_machines() -> Showing {
        Showing::Itself(OnItsOwn::for_(
            Errand::FetchingAModel,
            Destination::at("models.alo.example").unwrap(),
        ))
    }

    /// Both kinds answer the questions a shell asks of a line, by one road.
    #[test]
    fn either_kind_of_line_says_where_it_is_going_and_what_it_says() {
        let strings = in_english();
        assert_eq!(
            an_agents().said(&strings).text(),
            "@files is fetching something from alo.example"
        );
        assert_eq!(
            the_machines().said(&strings).text(),
            "alo OS is fetching a model from models.alo.example"
        );
        assert_eq!(
            an_agents().destination(),
            &Destination::at("alo.example").unwrap()
        );
        assert_eq!(
            the_machines().destination(),
            &Destination::at("models.alo.example").unwrap()
        );
    }

    /// **An errand is under nobody's authority, and says so.** A line that
    /// named one would be inventing a grant nobody made.
    #[test]
    fn only_an_agents_egress_is_under_anybodys_authority() {
        assert_eq!(an_agents().agent(), Some(&Grantee::named("@files")));
        assert_eq!(the_machines().agent(), None);
        assert!(an_agents().leaving().is_some());
        assert!(an_agents().on_its_own().is_none());
        assert!(the_machines().leaving().is_none());
        assert!(the_machines().on_its_own().is_some());
    }

    /// What the shell is handed can be written down, and the two kinds are
    /// distinguishable in it — an errand that read like an agent's egress would
    /// let a record of one be made from the other.
    #[test]
    fn the_two_kinds_are_told_apart_when_they_are_written_down() {
        let agents = serde_json::to_string(&an_agents()).unwrap();
        let machines = serde_json::to_string(&the_machines()).unwrap();
        assert!(agents.contains("agent"), "{agents}");
        assert!(agents.contains("@files"), "{agents}");
        assert!(machines.contains("itself"), "{machines}");
        assert!(!machines.contains("@files"), "{machines}");
    }
}
