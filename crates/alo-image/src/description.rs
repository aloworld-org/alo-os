//! What the image says the machine is, as far as the image is answerable for it.
//!
//! `docs/contracts/machine-description.md` says this file is written by whoever
//! installs a machine. On this image that is the image: it is the smallest thing
//! that boots and starts `alo-agentd`, so it stands a machine up, and
//! `image/etc/alo/agentd.toml` is what it says about the one it stood up.
//!
//! # This is a second reader, and it is one on purpose
//!
//! `alo-agentd`'s own `describing.rs` is what decides whether a description is
//! believed — the file's owner, its mode, the format number, every value, and
//! the refusals. **This is not that**, and it does not try to be: it reads the
//! same file for the handful of things the *image* is answerable for, so that
//! `crate::checking` can hold the description against the accounts and the
//! directories the image itself makes.
//!
//! There are two readers because there cannot be one. `alo-agentd` is a Unix
//! socket and the credentials a kernel keeps for one, so on any host that is not
//! Linux it compiles to nothing at all — and a check that only ran on Linux
//! would be a check the build loop's own machine never runs. What keeps the two
//! honest about the same file is that both refuse a key nobody declared and both
//! refuse a format number they do not know; the rest of the daemon's rules are
//! the daemon's, and are tested there.

use std::path::{Path, PathBuf};

use alo_keeping::Keeping;
use serde::Deserialize;

use crate::refusing::NotDescribed;

/// The shape of description this crate reads, which is `alo-agentd`'s.
pub const THE_FORMAT: u32 = 1;

/// Where the description goes on the machine.
pub const THE_DESCRIPTION: &str = "/etc/alo/agentd.toml";

/// What the machine description says, as the image needs to read it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Description {
    /// Which shape this description is in.
    format: u32,
    /// The two logins this machine has, and the group they meet in.
    logins: TheLogins,
    /// The agent, and how long it is served for.
    agent: TheAgent,
    /// Where what happened is written down, and for how long.
    record: TheRecord,
}

/// The two logins and the group they meet in, as numbers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct TheLogins {
    /// The signed-in person, whom `alo-agentd` runs as.
    person: u32,
    /// The agent, which is a login of its own.
    agent: u32,
    /// The group both are in.
    group: u32,
}

/// The agent this machine has, and how long it is served for.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct TheAgent {
    /// What the grants call it.
    name: String,
    /// How long a turn's own grant lasts, in whole seconds.
    turn_seconds: u64,
    /// How long a change waits for an answer, in whole seconds.
    proposal_seconds: u64,
}

/// Where what happened is written down, and how long it is kept.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct TheRecord {
    /// The file `alo-keeping` appends to.
    path: PathBuf,
    /// The retention rule this machine is under (ADR 0004).
    keeping: Keeping,
}

impl Description {
    /// The machine this text describes.
    ///
    /// # Errors
    ///
    /// [`NotDescribed::AnotherFormat`] for a description written for an alo OS
    /// this is not, answered before anything else in the file, and
    /// [`NotDescribed::NotTheShape`] for everything else — a missing key, a key
    /// nobody declared, a number where a string belongs.
    pub fn read(text: &str) -> Result<Self, NotDescribed> {
        let described: Self = toml::from_str(text).map_err(|why| NotDescribed::NotTheShape {
            why: why.to_string(),
        })?;
        if described.format != THE_FORMAT {
            return Err(NotDescribed::AnotherFormat {
                format: described.format,
            });
        }
        Ok(described)
    }

    /// The signed-in person, whom `alo-agentd` runs as.
    #[must_use]
    pub const fn person(&self) -> u32 {
        self.logins.person
    }

    /// The agent, which is a login of its own.
    #[must_use]
    pub const fn agent(&self) -> u32 {
        self.logins.agent
    }

    /// The group both are in, which is how the agent reaches the socket.
    #[must_use]
    pub const fn group(&self) -> u32 {
        self.logins.group
    }

    /// What this machine's agent is called, exactly as its grants name it.
    #[must_use]
    pub fn called(&self) -> &str {
        &self.agent.name
    }

    /// How long a turn's own grant lasts, in whole seconds.
    #[must_use]
    pub const fn turn_seconds(&self) -> u64 {
        self.agent.turn_seconds
    }

    /// How long a change waits for an answer, in whole seconds.
    #[must_use]
    pub const fn proposal_seconds(&self) -> u64 {
        self.agent.proposal_seconds
    }

    /// The file what happened is written into.
    #[must_use]
    pub fn record(&self) -> &Path {
        &self.record.path
    }

    /// The folder that file goes in, which the image makes and the daemon
    /// refuses to.
    #[must_use]
    pub fn record_folder(&self) -> Option<&Path> {
        self.record.path.parent()
    }

    /// The retention rule this machine is under.
    #[must_use]
    pub const fn keeping(&self) -> Keeping {
        self.record.keeping
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A description of the shape the image ships.
    const SHIPPED: &str = "\
format = 1

[logins]
person = 1000
agent = 989
group = 989

[agent]
name = \"alo\"
turn-seconds = 900
proposal-seconds = 300

[record]
path = \"/var/lib/alo/record.jsonl\"
keeping = \"forever\"
";

    /// Everything the image is answerable for comes back as it was written.
    #[test]
    fn a_description_reads_back_what_it_says() {
        let described = Description::read(SHIPPED).unwrap();

        assert_eq!(described.person(), 1000);
        assert_eq!(described.agent(), 989);
        assert_eq!(described.group(), 989);
        assert_eq!(described.called(), "alo");
        assert_eq!(described.turn_seconds(), 900);
        assert_eq!(described.proposal_seconds(), 300);
        assert_eq!(described.record(), Path::new("/var/lib/alo/record.jsonl"));
        assert_eq!(
            described.record_folder(),
            Some(Path::new("/var/lib/alo")),
            "which is the directory the image has to make"
        );
        assert_eq!(described.keeping(), Keeping::Forever);
    }

    /// **A description written for another alo OS is refused as one**, before
    /// any of its values are looked at — the rule `alo-agentd` keeps, kept here
    /// too so that a check does not pass on a file the daemon would refuse.
    #[test]
    fn a_description_for_another_alo_os_is_refused_as_one() {
        let refused = Description::read(&SHIPPED.replace("format = 1", "format = 2")).unwrap_err();
        assert!(
            matches!(refused, NotDescribed::AnotherFormat { format: 2 }),
            "{refused}"
        );
    }

    /// **A key nobody declared is refused**, which is the mistake that really
    /// happens — a typo — and which read any other way is a machine running
    /// under a value nobody chose.
    #[test]
    fn a_key_nobody_declared_is_refused() {
        let refused =
            Description::read(&SHIPPED.replace("turn-seconds", "truns-seconds")).unwrap_err();
        assert!(
            matches!(refused, NotDescribed::NotTheShape { .. }),
            "{refused}"
        );
    }

    /// **And a key left out is refused too.** There are no defaults here, for
    /// the reason there are none in the contract: *nobody decided this* must
    /// never be read as *probably fine*.
    #[test]
    fn a_key_left_out_is_refused() {
        let refused = Description::read(&SHIPPED.replace("agent = 989\n", "")).unwrap_err();
        assert!(
            matches!(refused, NotDescribed::NotTheShape { .. }),
            "{refused}"
        );
    }

    /// A retention an organisation set reads as the rule it is, so a check above
    /// can tell it apart from the one an image is allowed to ship.
    #[test]
    fn a_retention_an_organisation_set_reads_as_the_rule() {
        let described = Description::read(
            &SHIPPED.replace("keeping = \"forever\"", "keeping = { for-days = 90 }"),
        )
        .unwrap();
        assert_eq!(described.keeping().days(), Some(90));
    }

    /// The path this crate looks for the description at is the one the contract
    /// names, said once here and in `alo-agentd` rather than agreed by accident.
    #[test]
    fn the_description_is_where_the_contract_says() {
        assert_eq!(THE_DESCRIPTION, "/etc/alo/agentd.toml");
        assert_eq!(THE_FORMAT, 1);
    }
}
