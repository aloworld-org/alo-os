//! Where this machine says it is, and why it does not look it up.

use serde::{Deserialize, Serialize};

/// **The timezone a person chose**, as an IANA name — `Europe/Brussels`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timezone(String);

impl Timezone {
    /// The timezone a person named.
    ///
    /// # Errors
    /// Nothing here checks that the zone exists: the list of zones is the
    /// system's and changes with the world, and a machine that refused a zone
    /// its own database has would be wrong in the direction that strands
    /// somebody. An empty name is refused, because that is not a choice.
    pub fn named(named: &str) -> Result<Self, NotATimezone> {
        let named = named.trim();
        if named.is_empty() || !named.contains('/') {
            return Err(NotATimezone {
                asked_for: named.to_owned(),
            });
        }
        Ok(Self(named.to_owned()))
    }

    /// What it is called.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Not a timezone anybody could have meant.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("`{asked_for}` is not a timezone: they are written as Europe/Brussels")]
pub struct NotATimezone {
    /// What was asked for.
    pub asked_for: String,
}

/// **Whether this machine may ask the network where it is.**
///
/// Off until a person turns it on, and that is law 1 rather than a preference:
/// finding the timezone from the network is a request to somebody else's
/// service, carrying this machine's address, made by a machine that was sitting
/// still. A person who wants it says so; a person who does not is never asked
/// about it behind their back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FollowingTheNetwork {
    /// The timezone is what the person chose, and nothing is asked.
    #[default]
    No,
    /// The person turned it on, knowing it is a request that leaves.
    Yes,
}

impl FollowingTheNetwork {
    /// Whether anything would leave this machine for this.
    #[must_use]
    pub const fn would_leave_the_machine(self) -> bool {
        matches!(self, Self::Yes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Following the network is off until somebody says otherwise.**
    #[test]
    fn a_machine_nobody_has_asked_does_not_look_up_where_it_is() {
        assert_eq!(FollowingTheNetwork::default(), FollowingTheNetwork::No);
        assert!(!FollowingTheNetwork::default().would_leave_the_machine());
        assert!(FollowingTheNetwork::Yes.would_leave_the_machine());
    }

    #[test]
    fn a_timezone_is_a_zone_and_not_a_word() {
        assert_eq!(
            Timezone::named("Europe/Brussels").map(|zone| zone.name().to_owned()),
            Ok("Europe/Brussels".to_owned())
        );
        assert!(Timezone::named("").is_err());
        assert!(Timezone::named("Brussels").is_err());
    }
}
