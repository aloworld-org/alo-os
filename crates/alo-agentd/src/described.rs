//! What a machine says about itself, once it has been believed.
//!
//! Everything `crate::serving` is handed and does not decide: which two logins
//! this machine has, what its agent is called where the grants name it, how long
//! a turn lasts and a change waits for an answer, where what happened is written
//! down and how long it is kept. One value, made one way, so that a service
//! cannot be running under half a description.
//!
//! # It is a checked value, and there is one door into it
//!
//! [`Described::of`] is the only constructor and it takes types that have
//! already refused what they refuse: `crate::Sides` has refused a machine whose
//! person and agent are one login and an agent running as root, and
//! `crate::Lasting` has refused a length of time that is no time at all. What is
//! left for this file is the two things neither of them could see — an agent
//! with no name, and a path that is relative — and after that a `Described` is a
//! machine that can be served.
//!
//! There is deliberately **no `Default`**, for `alo_capability::Agent`'s reason
//! one crate on: a default would be alo OS answering, on somebody's behalf,
//! questions about their machine that only they can answer. A machine with no
//! description does not start.
//!
//! # Where the file is, and why that is a public surface
//!
//! [`THE_DESCRIPTION`] is where `alo-agentd` looks. It is in
//! `docs/contracts/machine-description.md` because the moment anything else
//! writes it — an installer, a management tool, an organisation's configuration
//! system (ADR 0004) — its shape and its path are things other people build
//! against, and `CLAUDE.md` says a contract changes additively or not at all.
//!
//! # What it deliberately does not hold
//!
//! **The directory the socket goes in**, which is the session's rather than the
//! machine's; `crate::session` is the whole of that argument.
//!
//! **Which model or provider answers a question, and under which policy.** That
//! is the rest of queue item 21e and it arrives with the process that asks one:
//! `crate::doing` refuses a question in words today, and the sentence stops
//! being the only answer when there is something behind it to ask.

use std::path::{Path, PathBuf};

use alo_keeping::Keeping;

use crate::caller::Uid;
use crate::describing::{THE_RECORD, read};
use crate::lasting::Lasting;
use crate::refusing::NotDescribed;
use crate::side::Sides;
use crate::trusting::as_written;

/// Where `alo-agentd` reads its description from.
///
/// Part of `docs/contracts/machine-description.md`, because whoever installs or
/// manages a machine writes this file.
pub const THE_DESCRIPTION: &str = "/etc/alo/agentd.toml";

/// What this machine says about itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Described {
    /// The two logins and the group they meet in.
    sides: Sides,
    /// The agent this machine has, as the grants name it.
    agent: String,
    /// How long a turn's own grant lasts.
    turn: Lasting,
    /// How long a change waits for an answer.
    proposal: Lasting,
    /// Where what happened is written down.
    record: PathBuf,
    /// How long it is kept.
    keeping: Keeping,
}

impl Described {
    /// A machine described this way.
    ///
    /// # Errors
    ///
    /// [`NotDescribed::Anonymous`] for an agent with no name, because
    /// `alo_capability::Grant` matches a grantee exactly and a grant could
    /// never name it — so every turn on such a machine would be refused, one
    /// caller at a time, for something that was wrong before it started.
    ///
    /// [`NotDescribed::NotAbsolute`] for a record path that is relative, which
    /// would put the evidence of what an agent did wherever the service happened
    /// to be started from — and somewhere different the next time.
    pub fn of(
        sides: Sides,
        agent: &str,
        turn: Lasting,
        proposal: Lasting,
        record: &Path,
        keeping: Keeping,
    ) -> Result<Self, NotDescribed> {
        if agent.trim().is_empty() {
            return Err(NotDescribed::Anonymous);
        }
        if !record.is_absolute() {
            return Err(NotDescribed::NotAbsolute {
                what: THE_RECORD,
                at: record.to_owned(),
            });
        }
        Ok(Self {
            sides,
            agent: agent.to_owned(),
            turn,
            proposal,
            record: record.to_owned(),
            keeping,
        })
    }

    /// The machine described by the file at this path.
    ///
    /// `us` is the user this process is running as, which `crate::unix::us`
    /// answers. It is passed in rather than asked for here so that the file's
    /// own rules are testable as rules; `crate::trusting` says why that matters
    /// more than it sounds.
    ///
    /// # Errors
    ///
    /// [`NotDescribed`], which is every way a description is not believed: the
    /// file itself (`crate::trusting`), the format number, the shape, and each
    /// of the values. Nothing is started and nothing is written in any of them.
    pub fn at(path: &Path, us: Uid) -> Result<Self, NotDescribed> {
        read(&as_written(path, us)?, path)
    }

    /// The two logins this machine has, and the group they meet in.
    #[must_use]
    pub const fn sides(&self) -> Sides {
        self.sides
    }

    /// The agent this machine has, as the grants name it.
    #[must_use]
    pub fn agent(&self) -> &str {
        &self.agent
    }

    /// How long a turn's own grant lasts.
    #[must_use]
    pub const fn turn(&self) -> Lasting {
        self.turn
    }

    /// How long a change waits for an answer.
    #[must_use]
    pub const fn proposal(&self) -> Lasting {
        self.proposal
    }

    /// Where what happened on this machine is written down.
    #[must_use]
    pub fn record(&self) -> &Path {
        &self.record
    }

    /// How long the record is kept.
    ///
    /// Read by whatever shortens it — queue item 20 — and by nothing else here.
    /// `alo_keeping::Keeping` is the rule; this is only where the machine says
    /// which one it is under.
    #[must_use]
    pub const fn keeping(&self) -> Keeping {
        self.keeping
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::caller::Gid;
    use crate::describing::THE_NAME;
    use std::num::NonZeroU32;

    /// The two logins an ordinary machine has.
    fn two_logins() -> Sides {
        Sides::of(
            Uid::of(1000).unwrap(),
            Uid::of(989).unwrap(),
            Gid::of(989).unwrap(),
        )
        .unwrap()
    }

    /// A quarter of an hour, as a turn.
    fn a_turn() -> Lasting {
        Lasting::of_seconds(900, "agent.turn-seconds").unwrap()
    }

    /// Five minutes, as a proposal.
    fn a_proposal() -> Lasting {
        Lasting::of_seconds(300, "agent.proposal-seconds").unwrap()
    }

    /// A machine described the ordinary way.
    fn an_ordinary_machine() -> Described {
        Described::of(
            two_logins(),
            "alo",
            a_turn(),
            a_proposal(),
            Path::new("/var/lib/alo/record"),
            Keeping::Forever,
        )
        .unwrap()
    }

    /// Everything a machine was described with comes back as it went in, which
    /// is what a service is handed.
    #[test]
    fn a_machine_says_back_what_it_was_told() {
        let machine = an_ordinary_machine();
        assert_eq!(machine.sides().person().raw(), 1000);
        assert_eq!(machine.sides().agent().raw(), 989);
        assert_eq!(machine.agent(), "alo");
        assert_eq!(machine.turn().duration().as_secs(), 900);
        assert_eq!(machine.proposal().duration().as_secs(), 300);
        assert_eq!(machine.record(), Path::new("/var/lib/alo/record"));
        assert_eq!(machine.keeping(), Keeping::Forever);
    }

    /// **An agent with no name is refused**, because a grant matches a grantee
    /// exactly and could never name it — so the machine would refuse every turn,
    /// one caller at a time, for something wrong before it started.
    #[test]
    fn an_agent_with_no_name_is_refused() {
        let refused = Described::of(
            two_logins(),
            "",
            a_turn(),
            a_proposal(),
            Path::new("/var/lib/alo/record"),
            Keeping::Forever,
        )
        .unwrap_err();
        assert!(matches!(refused, NotDescribed::Anonymous));
    }

    /// **And a name of nothing but spaces is no name**, which is the shape the
    /// mistake really arrives in: somebody clearing a value in a file rather
    /// than deleting the line.
    #[test]
    fn a_name_of_nothing_but_spaces_is_no_name() {
        assert!(matches!(
            Described::of(
                two_logins(),
                "   ",
                a_turn(),
                a_proposal(),
                Path::new("/var/lib/alo/record"),
                Keeping::Forever,
            )
            .unwrap_err(),
            NotDescribed::Anonymous
        ));
    }

    /// **A relative record path is refused**, and the refusal names the key —
    /// evidence of what an agent did does not go wherever the service was
    /// started from.
    #[test]
    fn a_relative_record_path_is_refused_and_names_the_key() {
        let refused = Described::of(
            two_logins(),
            "alo",
            a_turn(),
            a_proposal(),
            Path::new("record"),
            Keeping::Forever,
        )
        .unwrap_err();
        assert!(matches!(refused, NotDescribed::NotAbsolute { what, .. } if what == THE_RECORD));
        assert!(refused.to_string().contains("record.path"), "{refused}");
    }

    /// A retention an organisation set comes back as the rule it is, which is
    /// the whole of what queue item 20 was waiting on.
    #[test]
    fn a_retention_an_organisation_set_comes_back_as_the_rule() {
        let ninety = Keeping::ForDays(NonZeroU32::new(90).unwrap());
        let machine = Described::of(
            two_logins(),
            "alo",
            a_turn(),
            a_proposal(),
            Path::new("/var/lib/alo/record"),
            ninety,
        )
        .unwrap();
        assert_eq!(machine.keeping(), ninety);
        assert_eq!(machine.keeping().days(), Some(90));
    }

    /// The agent's name is the one place a description holds something that is
    /// not a number or a path, and it is used unchanged: the grants match a
    /// grantee exactly (item 1), so trimming or lowercasing it here would be
    /// this file quietly matching something else.
    #[test]
    fn the_agents_name_is_used_exactly_as_it_was_written() {
        let machine = Described::of(
            two_logins(),
            "Alo Assistant",
            a_turn(),
            a_proposal(),
            Path::new("/var/lib/alo/record"),
            Keeping::Forever,
        )
        .unwrap();
        assert_eq!(machine.agent(), "Alo Assistant");
    }

    /// The name of the key a record path is written under is one string,
    /// because it is in a refusal somebody reads and in the contract they wrote
    /// the file against.
    #[test]
    fn the_keys_are_named_once() {
        assert_eq!(THE_RECORD, "record.path");
        assert_eq!(THE_NAME, "agent.name");
        assert_eq!(THE_DESCRIPTION, "/etc/alo/agentd.toml");
    }
}
