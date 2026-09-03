//! The description as it arrives off the disk, and the one number that decides
//! whether it is read at all.
//!
//! `crate::described` is the checked value; this is the shape somebody typed.
//! They are two files for `alo-strings`' reason since item 9 — a translation as
//! it arrives and a translation that may be shown are two types, and the
//! checking is the door between them — and for a sharper one here: everything in
//! this file derives `Deserialize`, and nothing that does may also be the thing
//! a service is run from.
//!
//! # The format number, and what additively means
//!
//! [`THE_FORMAT`] is `1`. A description that says anything else is **refused
//! rather than guessed at**, which is `docs/contracts/record-file.md`'s rule
//! about a record from a newer alo OS, applied to the file that says what a
//! machine is. A newer alo OS may add keys; this one reading a file written for
//! it would be a service running under a description it understood some of, and
//! a machine half-described is exactly the thing nobody notices until it
//! matters.
//!
//! # A key nobody declared is refused
//!
//! `deny_unknown_fields`, and it is worth the sentence. The only other thing to
//! do with `truns-seconds = 900` is ignore it, and then the machine runs under
//! whatever the missing key defaults to while the file on the disk says
//! otherwise — which is the same failure as the format number, arriving one
//! typo at a time. Additive change is carried by [`THE_FORMAT`]; a key this
//! service does not know is a mistake, and it is answered with the line it is
//! on.
//!
//! # Nothing here has a default
//!
//! Not the retention, which ADR 0004 gives to the organisation that manages the
//! machine; not the two lengths of time; not the record's path. It is item 23's
//! decision about `alo_models::Driving::NotMeasured` — a required field with no
//! serde default, so a description that says nothing fails to load and *nobody
//! decided this* is never read as *probably fine*.
//!
//! `alo_keeping::Keeping` has a `Default` of its own, and it is deliberately not
//! reached for here: *forever unless somebody says otherwise* is the right
//! answer for a record whose owner never opened a settings panel, and the wrong
//! one for a machine whose description an administrator wrote and left a line
//! out of.

use std::path::PathBuf;

use alo_keeping::Keeping;
use serde::Deserialize;

use crate::caller::{Gid, Uid};
use crate::described::Described;
use crate::lasting::Lasting;
use crate::refusing::NotDescribed;
use crate::side::Sides;

/// The shape of description this `alo-agentd` reads.
pub const THE_FORMAT: u32 = 1;

/// The key the agent's name is written under.
pub const THE_NAME: &str = "agent.name";

/// The key a turn's length is written under.
pub const THE_TURN: &str = "agent.turn-seconds";

/// The key a proposal's length is written under.
pub const THE_PROPOSAL: &str = "agent.proposal-seconds";

/// The key the record's path is written under.
pub const THE_RECORD: &str = "record.path";

/// Which shape of description this is, and nothing else.
///
/// Read first, from the same text, and it is the one type here that does **not**
/// deny a key it does not know — because a description written for a newer alo
/// OS is exactly a file with keys this service has never heard of, and refusing
/// it as a typo would send whoever is on call looking for one. Reading the
/// number twice costs a parse of a file read once at startup, and buys a
/// refusal that says what is really wrong.
#[derive(Debug, Deserialize)]
struct WhichFormat {
    /// Which shape of description this is.
    format: u32,
}

/// A machine description exactly as it was typed.
///
/// Every field is required and nothing here is checked; this is the shape, and
/// [`checked`](AsWritten::checked) is where it becomes a machine.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AsWritten {
    /// Which shape of description this is.
    format: u32,
    /// The two logins this machine has.
    logins: TheLogins,
    /// The agent, and the two lengths of time a turn is served under.
    agent: TheAgent,
    /// Where what happened is written down, and for how long.
    record: TheRecord,
}

/// The two logins and the group they meet in, as numbers.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TheLogins {
    /// The signed-in person, who `alo-agentd` runs as.
    person: u32,
    /// The agent, which is a login of its own.
    agent: u32,
    /// The group that may reach the socket at all.
    group: u32,
}

/// The agent this machine has, and how long it is served for.
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TheRecord {
    /// The file `alo-keeping` appends to.
    path: PathBuf,
    /// The retention rule this machine is under (ADR 0004).
    keeping: Keeping,
}

impl AsWritten {
    /// This description as a machine, or the first reason it is not one.
    ///
    /// The order is the order somebody reading a log wants: the format number,
    /// then the two logins, because a machine whose person and agent are one
    /// login has no socket at all and the rest of the file does not matter.
    ///
    /// The format is answered **here** as well as in [`read`], and that is not
    /// belt and braces: this is the only road from a file to a
    /// [`Described`], so a number this service does not read cannot become a
    /// machine by any route. What [`read`] adds is the good refusal for a file
    /// whose keys this service has never heard of.
    fn checked(self, at: &std::path::Path) -> Result<Described, NotDescribed> {
        if self.format != THE_FORMAT {
            return Err(NotDescribed::AnotherFormat {
                at: at.to_owned(),
                format: self.format,
                reads: THE_FORMAT,
            });
        }
        let sides = Sides::of(
            Uid::of(self.logins.person)?,
            Uid::of(self.logins.agent)?,
            Gid::of(self.logins.group)?,
        )?;
        Described::of(
            sides,
            &self.agent.name,
            Lasting::of_seconds(self.agent.turn_seconds, THE_TURN)?,
            Lasting::of_seconds(self.agent.proposal_seconds, THE_PROPOSAL)?,
            &self.record.path,
            self.record.keeping,
        )
    }
}

/// The machine this description describes.
///
/// `at` is where the text came from, and it is carried only so that every
/// refusal names the file somebody has to go and edit — nothing here reads a
/// disk, which is what makes each of these refusals a test rather than a
/// fixture.
///
/// # Errors
///
/// [`NotDescribed::AnotherFormat`] for a description this service does not
/// read, [`NotDescribed::NotUnderstood`] for text that is not one, and whatever
/// the values themselves refuse.
pub(crate) fn read(said: &str, at: &std::path::Path) -> Result<Described, NotDescribed> {
    let not_understood = |why: toml::de::Error| NotDescribed::NotUnderstood {
        at: at.to_owned(),
        why: Box::new(why),
    };

    let which: WhichFormat = toml::from_str(said).map_err(not_understood)?;
    if which.format != THE_FORMAT {
        return Err(NotDescribed::AnotherFormat {
            at: at.to_owned(),
            format: which.format,
            reads: THE_FORMAT,
        });
    }

    let written: AsWritten = toml::from_str(said).map_err(not_understood)?;
    written.checked(at)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::path::Path;

    /// The path every refusal in these tests names.
    fn somewhere() -> &'static Path {
        Path::new("/etc/alo/agentd.toml")
    }

    /// A description of an ordinary machine, exactly as
    /// `docs/contracts/machine-description.md` writes it.
    fn an_ordinary_machine() -> String {
        r#"
format = 1

[logins]
person = 1000
agent = 989
group = 989

[agent]
name = "alo"
turn-seconds = 900
proposal-seconds = 300

[record]
path = "/var/lib/alo/record"
keeping = "forever"
"#
        .to_owned()
    }

    /// The example in the contract is a machine, and every value in it arrives
    /// where the service reads it.
    #[test]
    fn the_description_in_the_contract_is_a_machine() {
        let machine = read(&an_ordinary_machine(), somewhere()).unwrap();
        assert_eq!(machine.sides().person().raw(), 1000);
        assert_eq!(machine.sides().agent().raw(), 989);
        assert_eq!(machine.sides().shared().raw(), 989);
        assert_eq!(machine.agent(), "alo");
        assert_eq!(machine.turn().seconds(), 900);
        assert_eq!(machine.proposal().seconds(), 300);
        assert_eq!(machine.record(), Path::new("/var/lib/alo/record"));
        assert_eq!(machine.keeping(), Keeping::Forever);
    }

    /// A retention an organisation set is read as the rule it is, in the shape
    /// `alo-keeping` already writes it into the record's first line.
    #[test]
    fn a_retention_in_days_is_read_as_the_rule_it_is() {
        let said =
            an_ordinary_machine().replace(r#"keeping = "forever""#, "keeping = { for-days = 90 }");
        assert_eq!(read(&said, somewhere()).unwrap().keeping().days(), Some(90));
    }

    /// **A description from a newer alo OS is refused rather than guessed at**,
    /// and the refusal says both numbers so whoever is on call can see which
    /// way round it is.
    #[test]
    fn a_description_from_a_newer_alo_os_is_refused() {
        let said = an_ordinary_machine().replace("format = 1", "format = 2");
        let refused = read(&said, somewhere()).unwrap_err();
        assert!(matches!(
            refused,
            NotDescribed::AnotherFormat {
                format: 2,
                reads: 1,
                ..
            }
        ));
        assert!(
            refused.to_string().contains("/etc/alo/agentd.toml"),
            "{refused}"
        );
    }

    /// **A key nobody declared is refused**, because the only other thing to do
    /// with a typo is run under whatever the key it was meant to be says.
    #[test]
    fn a_key_nobody_declared_is_refused() {
        let said = an_ordinary_machine().replace("turn-seconds", "truns-seconds");
        let refused = read(&said, somewhere()).unwrap_err();
        assert!(matches!(refused, NotDescribed::NotUnderstood { .. }));
        assert!(refused.to_string().contains("truns-seconds"), "{refused}");
    }

    /// **A key left out is refused**, rather than defaulted. A machine whose
    /// description says nothing about how long its record is kept is a machine
    /// whose administrator left a line out, not one whose owner never decided.
    #[test]
    fn a_retention_left_out_is_refused_rather_than_defaulted() {
        let said = an_ordinary_machine().replace(r#"keeping = "forever""#, "");
        let refused = read(&said, somewhere()).unwrap_err();
        assert!(matches!(refused, NotDescribed::NotUnderstood { .. }));
        assert!(refused.to_string().contains("keeping"), "{refused}");
    }

    /// **A machine whose person and agent are one login is refused here**, which
    /// is `crate::side`'s refusal reaching the file it is really about.
    #[test]
    fn a_machine_with_one_login_is_refused() {
        let said = an_ordinary_machine().replace("agent = 989", "agent = 1000");
        let refused = read(&said, somewhere()).unwrap_err();
        assert!(matches!(
            refused,
            NotDescribed::NotTwoSides(crate::NotTwoSides::OneUser { uid: 1000 })
        ));
    }

    /// **An agent named as root is refused** (ADR 0001 §2), in the file where
    /// somebody wrote the number.
    #[test]
    fn an_agent_named_as_root_is_refused() {
        let said = an_ordinary_machine().replace("agent = 989", "agent = 0");
        assert!(matches!(
            read(&said, somewhere()).unwrap_err(),
            NotDescribed::NotTwoSides(crate::NotTwoSides::AgentAsRoot)
        ));
    }

    /// `-1` is what a Unix call answers with when there is no user, and it is
    /// what a shell script that could not look one up puts in a file.
    #[test]
    fn a_number_that_is_not_a_user_is_refused() {
        let said = an_ordinary_machine().replace("agent = 989", "agent = 4294967295");
        assert!(matches!(
            read(&said, somewhere()).unwrap_err(),
            NotDescribed::NotAUser(crate::NotAUser::NoSuchUser)
        ));
        let said = an_ordinary_machine().replace("group = 989", "group = 4294967295");
        assert!(matches!(
            read(&said, somewhere()).unwrap_err(),
            NotDescribed::NotAUser(crate::NotAUser::NoSuchGroup)
        ));
    }

    /// **A turn that lasts no time at all is refused**, and the refusal names
    /// the key rather than the concept.
    #[test]
    fn a_turn_that_lasts_no_time_is_refused_by_its_key() {
        let said = an_ordinary_machine().replace("turn-seconds = 900", "turn-seconds = 0");
        let refused = read(&said, somewhere()).unwrap_err();
        assert!(matches!(refused, NotDescribed::NoTimeAtAll { what } if what == THE_TURN));
    }

    /// And so is a proposal, by its own key — because *the turn is wrong* and
    /// *the proposal is wrong* send somebody to two different lines.
    #[test]
    fn a_proposal_that_stands_no_time_is_refused_by_its_own_key() {
        let said = an_ordinary_machine().replace("proposal-seconds = 300", "proposal-seconds = 0");
        let refused = read(&said, somewhere()).unwrap_err();
        assert!(matches!(refused, NotDescribed::NoTimeAtAll { what } if what == THE_PROPOSAL));
    }

    /// **A proposal that stands for a week is refused** — an approval is never a
    /// session, and a configuration file is not a way to make one.
    #[test]
    fn a_proposal_that_stands_for_a_week_is_refused() {
        let said =
            an_ordinary_machine().replace("proposal-seconds = 300", "proposal-seconds = 604800");
        assert!(matches!(
            read(&said, somewhere()).unwrap_err(),
            NotDescribed::TooLong { .. }
        ));
    }

    /// **A relative record path is refused**, carried up from the checked value
    /// with the key it was written under.
    #[test]
    fn a_relative_record_path_is_refused() {
        let said = an_ordinary_machine().replace(r#""/var/lib/alo/record""#, r#""record""#);
        assert!(matches!(
            read(&said, somewhere()).unwrap_err(),
            NotDescribed::NotAbsolute { what, .. } if what == THE_RECORD
        ));
    }

    /// Text that is not a description at all is refused as one, and the refusal
    /// names the file — whoever is reading it has several.
    #[test]
    fn something_that_is_not_a_description_is_refused_as_one() {
        let refused = read("this is not a machine", somewhere()).unwrap_err();
        assert!(matches!(refused, NotDescribed::NotUnderstood { .. }));
        assert!(
            refused.to_string().contains("/etc/alo/agentd.toml"),
            "{refused}"
        );
    }

    /// **An empty file is not an empty description.** It is a machine that has
    /// not been described, and it is refused with the keys it is missing.
    #[test]
    fn an_empty_file_is_not_an_empty_description() {
        assert!(matches!(
            read("", somewhere()).unwrap_err(),
            NotDescribed::NotUnderstood { .. }
        ));
    }

    /// **The format number is checked before anything in the file is used**, so
    /// a description written for a newer alo OS is refused as one rather than as
    /// whichever of its keys this service happened not to know.
    #[test]
    fn the_format_is_answered_before_the_values() {
        let said = an_ordinary_machine()
            .replace("format = 1", "format = 7")
            .replace("agent = 989", "agent = 1000");
        assert!(matches!(
            read(&said, somewhere()).unwrap_err(),
            NotDescribed::AnotherFormat { format: 7, .. }
        ));
    }
}
