//! One grant: who may reach what, and until when.
//!
//! A grant is the durable half of the capability model (ADR 0001 §3). Verbs
//! are what may be *done*; a grant is what they may be done *to*, and it is the
//! only thing in alo OS that decides reach.
//!
//! The properties a grant has to have are enforced here, at construction,
//! because a check somewhere else is a check somebody can forget to call:
//!
//! - it names one agent — a grant to nobody grants nothing;
//! - its path is a full path with no `..` in it, so it means the same thing
//!   wherever it is read;
//! - it is not the whole machine ([`GrantError::TheWholeMachine`]);
//! - **it ends.** [`Grant::checked`] takes how long it lasts and refuses zero,
//!   and there is no variant meaning "for ever". A grant that outlives the
//!   reason it was made is the failure this crate exists to make impossible,
//!   and the reliable way to prevent it is to leave the type no way to express
//!   it.
//!
//! Nothing here reads the clock: the moment a grant starts is passed in, and so
//! is the moment any later question is asked about it.

use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::path::{is_a_root, is_usable};
use crate::reach::{Ask, Reach};

/// Who a grant is for: one agent, by the name the system knows it by.
///
/// Compared exactly, like every other identity in this crate. Two agents whose
/// names differ only in case are two agents, and letting one answer for the
/// other would be a widening nobody asked for.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Grantee(String);

impl Grantee {
    /// The agent known by this name.
    #[must_use]
    pub fn named(name: &str) -> Self {
        Self(name.trim().to_owned())
    }

    /// The name, as the system knows it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Why a grant could not be made.
///
/// The messages say what to do about it. Somebody reads these having just
/// picked the wrong thing in a dialog, not having just read this file.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum GrantError {
    /// No agent was named.
    #[error("say which agent this grant is for — a grant to nobody reaches nothing")]
    Anonymous,
    /// The folder, file or application was empty.
    #[error("choose the folder, file or application this grant is over")]
    NothingNamed,
    /// A grant to `/`, or to any other spelling of the whole machine.
    #[error("there is no grant to the whole machine — pick the folder you actually mean")]
    TheWholeMachine,
    /// A relative path, which means something different depending on where it
    /// is read from.
    #[error(
        "grant a folder by its full path, so it means the same thing wherever it is asked about"
    )]
    NotAFullPath,
    /// A path containing `..`, which can leave the folder it appears to be in.
    #[error(
        "a path with .. in it can lead somewhere else — grant the folder you mean by its own path"
    )]
    CouldLeadElsewhere,
    /// A grant lasting no time at all.
    #[error("say how long the grant should last — a grant for no time reaches nothing")]
    NoTime,
    /// A grant so long it has no end this machine can represent.
    #[error("a grant has to end — choose how long this one should last")]
    NoEnd,
}

/// A grant a person made.
///
/// `Serialize` because the list of grants outlives the session that made it,
/// and a person who granted a folder on Monday should find it in the list on
/// Tuesday — still expiring at the moment it was always going to expire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grant {
    /// The agent this grant is for.
    pub grantee: Grantee,
    /// What it covers.
    pub reach: Reach,
    /// When it was made — shown in the list, so a person can see what they did
    /// and when.
    pub granted_at: SystemTime,
    /// When it stops. Not optional, on purpose.
    pub expires: SystemTime,
}

impl Grant {
    /// Make a grant, checking everything that has to be true of one.
    ///
    /// `from` is the moment it starts, normally now; `lasting` is how long it
    /// runs for. The pair is the argument rather than an end time because
    /// every caller has a duration in hand — an hour, the working day, the
    /// turn — and a subtraction is one more place to get the arithmetic wrong.
    ///
    /// # Errors
    /// [`GrantError`], saying what to fix.
    pub fn checked(
        grantee: &str,
        reach: Reach,
        from: SystemTime,
        lasting: Duration,
    ) -> Result<Self, GrantError> {
        let grantee = Grantee::named(grantee);
        if grantee.as_str().is_empty() {
            return Err(GrantError::Anonymous);
        }
        let reach = checked_reach(reach)?;
        if lasting.is_zero() {
            return Err(GrantError::NoTime);
        }
        let expires = from.checked_add(lasting).ok_or(GrantError::NoEnd)?;
        Ok(Self {
            grantee,
            reach,
            granted_at: from,
            expires,
        })
    }

    /// Whether this grant is still running at this moment.
    ///
    /// A grant that expires at five o'clock does not include five o'clock. The
    /// boundary has to fall somewhere, and it falls on the side that reaches
    /// less.
    #[must_use]
    pub fn is_active_at(&self, now: SystemTime) -> bool {
        self.expires > now
    }

    /// How much of the grant is left, or `None` once it has expired.
    #[must_use]
    pub fn expires_in(&self, now: SystemTime) -> Option<Duration> {
        self.expires
            .duration_since(now)
            .ok()
            .filter(|d| !d.is_zero())
    }

    /// Whether this grant is for that agent.
    #[must_use]
    pub fn is_for(&self, grantee: &Grantee) -> bool {
        &self.grantee == grantee
    }

    /// Whether this grant, at this moment, permits that ask.
    ///
    /// Both halves matter and both are checked here so that no caller can hold
    /// one without the other: an expired grant covering the right folder
    /// permits nothing at all.
    #[must_use]
    pub fn permits(&self, ask: &Ask, now: SystemTime) -> bool {
        self.is_active_at(now) && self.reach.covers(ask)
    }
}

/// Check what a grant is over, before it becomes one.
fn checked_reach(reach: Reach) -> Result<Reach, GrantError> {
    match &reach {
        Reach::Folder(path) | Reach::File(path) => {
            if path.as_os_str().is_empty() {
                return Err(GrantError::NothingNamed);
            }
            // The whole-machine check comes before the shape checks so that a
            // grant to `/` is refused for the reason a person needs to hear,
            // rather than for a technicality about its spelling.
            if is_a_root(path) {
                return Err(GrantError::TheWholeMachine);
            }
            if !path.has_root() {
                return Err(GrantError::NotAFullPath);
            }
            if !is_usable(path) {
                return Err(GrantError::CouldLeadElsewhere);
            }
        }
        Reach::Application(id) => {
            if id.trim().is_empty() {
                return Err(GrantError::NothingNamed);
            }
            return Ok(Reach::Application(id.trim().to_owned()));
        }
    }
    Ok(reach)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    fn invoices() -> Reach {
        Reach::Folder(PathBuf::from("/home/anna/Invoices"))
    }

    fn granted() -> Grant {
        Grant::checked("@files", invoices(), noon(), hour()).unwrap()
    }

    /// ADR 0001 §3, stated as a test because it is the sentence somebody will
    /// one day want an exception to.
    #[test]
    fn there_is_no_grant_to_the_whole_machine() {
        for root in ["/", "//", "/."] {
            let err =
                Grant::checked("@files", Reach::Folder(root.into()), noon(), hour()).unwrap_err();
            assert_eq!(err, GrantError::TheWholeMachine, "{root}");
        }
        assert!(
            GrantError::TheWholeMachine
                .to_string()
                .contains("pick the folder you actually mean")
        );
    }

    /// An expired grant grants nothing. Not less, not read-only — nothing.
    #[test]
    fn an_expired_grant_permits_nothing() {
        let grant = granted();
        let ask = Ask::path("/home/anna/Invoices/march.pdf");
        assert!(grant.permits(&ask, noon()));
        assert!(grant.permits(&ask, noon() + Duration::from_secs(3_599)));
        assert!(!grant.permits(&ask, noon() + hour()));
        assert!(!grant.permits(&ask, noon() + hour() + Duration::from_secs(1)));
        assert!(!grant.is_active_at(noon() + hour()));
        assert!(grant.expires_in(noon() + hour()).is_none());
        assert_eq!(grant.expires_in(noon()), Some(hour()));
    }

    /// A grant cannot be made without an end. There is no variant for "for
    /// ever", and zero is refused rather than treated as "until revoked".
    #[test]
    fn a_grant_has_to_end() {
        assert_eq!(
            Grant::checked("@files", invoices(), noon(), Duration::ZERO).unwrap_err(),
            GrantError::NoTime
        );
        assert_eq!(
            Grant::checked("@files", invoices(), noon(), Duration::MAX).unwrap_err(),
            GrantError::NoEnd
        );
    }

    /// A path that could lead somewhere else is refused when the grant is
    /// made, not argued about when it is used.
    #[test]
    fn a_grant_is_refused_before_it_can_lead_elsewhere() {
        assert_eq!(
            Grant::checked(
                "@files",
                Reach::Folder("/home/anna/../root".into()),
                noon(),
                hour()
            )
            .unwrap_err(),
            GrantError::CouldLeadElsewhere
        );
        assert_eq!(
            Grant::checked("@files", Reach::Folder("Invoices".into()), noon(), hour()).unwrap_err(),
            GrantError::NotAFullPath
        );
        assert_eq!(
            Grant::checked("@files", Reach::Folder("".into()), noon(), hour()).unwrap_err(),
            GrantError::NothingNamed
        );
    }

    #[test]
    fn a_grant_names_the_agent_it_is_for() {
        assert_eq!(
            Grant::checked("   ", invoices(), noon(), hour()).unwrap_err(),
            GrantError::Anonymous
        );
        let grant = granted();
        assert!(grant.is_for(&Grantee::named("@files")));
        assert!(!grant.is_for(&Grantee::named("@Files")));
        assert!(!grant.is_for(&Grantee::named("@mail")));
    }

    /// The grant a person made on Monday is the same grant on Tuesday,
    /// expiring when it was always going to expire.
    #[test]
    fn a_grant_survives_being_written_down_and_read_back() {
        let grant = granted();
        let written = serde_json::to_string(&grant).unwrap();
        let read: Grant = serde_json::from_str(&written).unwrap();
        assert_eq!(read, grant);
        assert!(!read.is_active_at(noon() + hour()));
    }

    /// The errors say what to do, not what went wrong.
    #[test]
    fn the_errors_say_what_to_do() {
        assert!(
            GrantError::Anonymous
                .to_string()
                .contains("say which agent")
        );
        assert!(GrantError::NoTime.to_string().contains("how long"));
        assert!(GrantError::NotAFullPath.to_string().contains("full path"));
    }
}
