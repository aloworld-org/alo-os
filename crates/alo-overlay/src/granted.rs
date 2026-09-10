//! What the agent may reach, as a number a person can read.
//!
//! The second of the three readings the overlay shows at rest. `CLAUDE.md`:
//! *no path, window, application or device is reachable that a person has not
//! granted.* `alo-capability` holds the grants and `alo-agentd` enforces them,
//! and until now nothing has ever put in front of a person the one number that
//! follows from both — **how much the agent can reach right now**.
//!
//! # Right now, and only for this agent
//!
//! Both halves matter and neither is free.
//!
//! *Right now*: [`Granted::of`] counts what is active at the moment it is
//! asked, through `alo_capability::Grants::held_by`, so an expired grant is
//! not counted whether or not anything has swept it up. An overlay that showed
//! the length of the list would tell a person their agent can reach three
//! folders when every one of them lapsed at lunchtime.
//!
//! *This agent*: the count is for the grantee the machine's description names
//! (`alo-agentd`'s `Described::agent`), never for the list as a whole. One
//! agent's grant is not another agent's, and a number that added them up would
//! be the one line in this system that treats them as interchangeable.
//!
//! # Nothing here can widen anything
//!
//! [`Granted::of`] takes `&Grants` and counts. There is no method on it that
//! makes a grant, and there could not be: making one is a person's act in a
//! file chooser (ADR 0001 §3, and the plan's task 6). Reading the number a
//! hundred times leaves the grants exactly as they were.

use std::num::NonZeroUsize;
use std::time::SystemTime;

use alo_capability::{Grantee, Grants};
use alo_strings::{Counting, Filling, Said, Strings};

use crate::words::{self, Counted, Word};

/// How much the agent can reach at this moment.
///
/// Two cases, because *nothing* is not a number a person reads as a number:
/// **nothing is granted** is the state every machine starts in and the thing
/// the overlay has something to say about, while any other count is a fact
/// counted in the reader's own language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Granted {
    /// The agent can reach nothing at all.
    Nothing,
    /// It holds this many active grants.
    ThisMany(NonZeroUsize),
}

impl Granted {
    /// What this agent holds on this machine, at this moment.
    ///
    /// `now` is passed in rather than read from the clock, as everywhere else
    /// in this repository: what the overlay shows and what a record says must
    /// not be able to disagree about when something was true.
    #[must_use]
    pub fn of(agent: &Grantee, grants: &Grants, now: SystemTime) -> Self {
        Self::of_how_many(grants.held_by(agent, now).count())
    }

    /// The same, from a count somebody has already made.
    #[must_use]
    pub fn of_how_many(how_many: usize) -> Self {
        NonZeroUsize::new(how_many).map_or(Self::Nothing, Self::ThisMany)
    }

    /// How many, as a number.
    #[must_use]
    pub const fn how_many(self) -> usize {
        match self {
            Self::Nothing => 0,
            Self::ThisMany(how_many) => how_many.get(),
        }
    }

    /// Whether the agent can reach anything at all.
    #[must_use]
    pub const fn is_nothing(self) -> bool {
        matches!(self, Self::Nothing)
    }

    /// The plain string this crate declares for it, where it has one.
    ///
    /// [`None`] for a count, which is [`Self::counted`] instead: a countable
    /// string is declared and looked up differently, and a caller that could
    /// reach one through here would get whichever English form somebody
    /// happened to pick.
    #[must_use]
    pub const fn word(self) -> Option<Word> {
        match self {
            Self::Nothing => Some(words::GRANTED_NOTHING),
            Self::ThisMany(_) => None,
        }
    }

    /// The countable string this crate declares for it, where it has one.
    #[must_use]
    pub const fn counted(self) -> Option<Counted> {
        match self {
            Self::Nothing => None,
            Self::ThisMany(_) => Some(words::GRANTED_HOW_MANY),
        }
    }

    /// The line a person reads, in the language they read it in.
    ///
    /// A count goes through `alo_strings::Strings::count`, so the sentence
    /// arrives in the form **the reader's own language** uses for **that**
    /// number — three forms in Polish, five in Irish, and one in Latvian for
    /// nothing at all.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        match self {
            Self::Nothing => strings.say(&words::GRANTED_NOTHING.key(), &Filling::nothing()),
            Self::ThisMany(how_many) => strings.count(
                &words::GRANTED_HOW_MANY.key(),
                // A machine with more grants than a `u64` can hold does not
                // exist; saturating rather than casting is how that stays a
                // fact about arithmetic instead of a `-D warnings` exemption.
                &Counting::of(u64::try_from(how_many.get()).unwrap_or(u64::MAX)),
                &Filling::nothing(),
            ),
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
    use crate::testing::in_english;
    use alo_capability::{Grant, Reach};
    use std::path::PathBuf;
    use std::time::Duration;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    fn the_agent() -> Grantee {
        Grantee::named("@files")
    }

    /// A machine where this agent has been granted one folder.
    fn one_folder() -> Grants {
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from("/home/anna/Invoices")),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        grants
    }

    /// **A machine where nothing has been granted says so**, which is the
    /// state every machine starts in.
    #[test]
    fn a_machine_where_nothing_is_granted_says_nothing_is_granted() {
        let granted = Granted::of(&the_agent(), &Grants::default(), noon());
        assert_eq!(granted, Granted::Nothing);
        assert!(granted.is_nothing());
        assert_eq!(granted.how_many(), 0);

        let said = granted.said(&in_english());
        assert!(!said.is_a_bug(), "the reading is not declared");
        assert_eq!(said.text(), "nothing is granted");
    }

    /// One grant is one grant, in the singular form the reader's language
    /// uses for it.
    #[test]
    fn one_grant_is_counted_as_one() {
        let granted = Granted::of(&the_agent(), &one_folder(), noon());
        assert_eq!(granted.how_many(), 1);
        assert_eq!(granted.said(&in_english()).text(), "one thing is granted");
    }

    /// And more than one is counted rather than written with a number stuck
    /// into an English sentence.
    #[test]
    fn more_than_one_is_counted_in_the_readers_own_language() {
        let mut grants = one_folder();
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Application("org.blender.Blender".to_owned()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        let granted = Granted::of(&the_agent(), &grants, noon());
        assert_eq!(granted.how_many(), 2);
        assert_eq!(granted.said(&in_english()).text(), "2 things are granted");
    }

    /// **An expired grant is not counted.** This is the refusal path that
    /// matters most here: an overlay counting the length of the list would
    /// tell a person their agent can reach a folder whose grant ran out at
    /// lunchtime, and the daemon would refuse the very next verb.
    #[test]
    fn a_grant_that_has_expired_is_not_something_the_agent_can_reach() {
        let grants = one_folder();
        let after = noon() + hour();
        assert!(
            !grants.permits(
                &the_agent(),
                &alo_capability::Ask::path("/home/anna/Invoices/march.pdf"),
                after
            ),
            "the daemon would still permit this, so the overlay is not the thing under test"
        );
        assert_eq!(Granted::of(&the_agent(), &grants, after), Granted::Nothing);
        // Still on the list until somebody sweeps it, and still counted as
        // nothing — the count is of what is active, never of what is stored.
        assert_eq!(grants.len(), 1);
    }

    /// **A revoked grant is gone from the overlay at once**, on the next
    /// reading rather than at the next sign-in.
    #[test]
    fn a_revoked_grant_stops_being_counted_immediately() {
        let mut grants = one_folder();
        let id = grants.active_at(noon()).next().unwrap().id;
        assert_eq!(Granted::of(&the_agent(), &grants, noon()).how_many(), 1);
        assert!(grants.revoke(id));
        assert_eq!(Granted::of(&the_agent(), &grants, noon()), Granted::Nothing);
    }

    /// **Another agent's grants are not this agent's.** A count that added
    /// every grant on the machine together would be the one line in this
    /// system that treats two agents as interchangeable.
    #[test]
    fn another_agents_grants_are_not_counted_as_this_ones() {
        let grants = one_folder();
        assert_eq!(
            Granted::of(&Grantee::named("@mail"), &grants, noon()),
            Granted::Nothing
        );
    }

    /// **Reading the number never grants anything.** Asking a hundred times
    /// leaves the grants exactly as they were, which is `alo-capability`'s
    /// *never widened by use* seen from the screen that reads it.
    #[test]
    fn reading_the_number_leaves_the_grants_where_they_were() {
        let grants = one_folder();
        let before = serde_json::to_string(&grants).unwrap();
        for _ in 0..100 {
            assert_eq!(Granted::of(&the_agent(), &grants, noon()).how_many(), 1);
        }
        assert_eq!(serde_json::to_string(&grants).unwrap(), before);
    }

    /// Each case reaches the kind of string it is declared as, and neither
    /// reaches the other's — a count answered from a plain phrase would show
    /// whichever English form somebody happened to write.
    #[test]
    fn each_case_names_the_kind_of_string_it_is() {
        assert_eq!(Granted::Nothing.word(), Some(words::GRANTED_NOTHING));
        assert_eq!(Granted::Nothing.counted(), None);
        let one = Granted::of_how_many(1);
        assert_eq!(one.word(), None);
        assert_eq!(one.counted(), Some(words::GRANTED_HOW_MANY));
    }
}
