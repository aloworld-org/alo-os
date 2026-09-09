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
//! [`THE_FORMAT`] is `2` and `1` is still read ([`ALSO_READ`]). A description
//! that says anything else is **refused rather than guessed at**, which is
//! `docs/contracts/record-file.md`'s rule about a record from a newer alo OS,
//! applied to the file that says what a machine is. A newer alo OS may add keys;
//! this one reading a file written for it would be a service running under a
//! description it understood some of, and a machine half-described is exactly
//! the thing nobody notices until it matters.
//!
//! **`2` since `[questions]`**, and that section is the one thing in this file
//! that could not be carried additively. Every other key an older service does
//! not know leaves it describing the same machine; a policy it ignored would
//! leave it sending an organisation's questions wherever the person chose, which
//! is not the same machine at all. So a description carrying a bound says `2`,
//! an alo OS that reads only `1` refuses it and does not start — a managed
//! machine too old to understand its policy does not run unmanaged — and
//! [`NotDescribed::APolicyNeedsANewerShape`] is the other direction: a `1` with
//! `[questions]` in it is a file claiming an older service could have read it
//! correctly, and it could not.
//!
//! A description **without** `[questions]` means the same thing under either
//! number, so `1` keeps working untouched and `2` is not a migration anybody has
//! to perform.
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
use alo_models::SourcePolicy;
use serde::Deserialize;

use crate::caller::{Gid, Uid};
use crate::described::Described;
use crate::lasting::Lasting;
use crate::questions::TheBound;
use crate::refusing::NotDescribed;
use crate::side::Sides;
use crate::trusting::WhoDescribedIt;

/// The shape of description this alo OS writes, and the newest it reads.
pub const THE_FORMAT: u32 = 2;

/// Every shape this alo OS still reads, oldest first.
///
/// Expand, migrate, contract — `CLAUDE.md`'s rule for a schema, and this is the
/// expand. Every description written before there was anywhere to state a
/// policy says `1`, has no `[questions]` in it, and means exactly what it always
/// meant; nothing rewrites it and nothing asks anybody to.
pub const ALSO_READ: [u32; 1] = [1];

/// Whether this alo OS reads a description that says it is this shape.
#[must_use]
pub const fn is_a_shape_we_read(format: u32) -> bool {
    format == THE_FORMAT || format == ALSO_READ[0]
}

/// The key the agent's name is written under.
pub const THE_NAME: &str = "agent.name";

/// The key a turn's length is written under.
pub const THE_TURN: &str = "agent.turn-seconds";

/// The key a proposal's length is written under.
pub const THE_PROPOSAL: &str = "agent.proposal-seconds";

/// The key the record's path is written under.
pub const THE_RECORD: &str = "record.path";

/// The key an organisation's bound is written under.
pub const THE_MAY_GO: &str = "questions.may-go";

/// The key the region that bound names is written under.
pub const THE_REGION: &str = "questions.region";

/// Anywhere the person chooses.
const ANYWHERE: &str = "anywhere";

/// Local and paired machines only.
const IN_THE_BUILDING: &str = "in-the-building";

/// Anything declared to run in the region [`THE_REGION`] names.
const IN_A_REGION: &str = "in-a-region";

/// This machine, and nowhere else at all.
const THIS_MACHINE_ONLY: &str = "this-machine-only";

/// Every `may-go` this service reads, for a refusal that can list them.
const EVERY_PLACE: [&str; 4] = [ANYWHERE, IN_THE_BUILDING, IN_A_REGION, THIS_MACHINE_ONLY];

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
    /// Where a question may be answered, or absent on a machine no
    /// organisation manages.
    ///
    /// **The one optional section in this file**, and it is optional because
    /// ADR 0016 says a machine no organisation manages has *no policy at all —
    /// not empty, not permissive by default, absent*. Every other key is
    /// required for item 23's reason; this one is not missing when it is
    /// absent, it is answered.
    questions: Option<TheQuestions>,
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

/// Where an organisation permits a question to be answered.
///
/// The **spelling is this file's**, not `alo_models::SourcePolicy`'s: that type
/// has no `Deserialize` and deliberately gains none here. A derive would name
/// the variants the way the fields happen to be named, and — worse — could not
/// refuse `region` written beside `may-go = "anywhere"`, because to serde that
/// is a key that parsed. A key somebody believes is bounding their questions
/// and is not, is the whole failure this section exists to avoid.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct TheQuestions {
    /// Which of [`EVERY_PLACE`] this organisation permits.
    may_go: String,
    /// Which region, and only where `may-go` is [`IN_A_REGION`].
    region: Option<String>,
}

impl TheQuestions {
    /// This section as the rule it states, or why it states none.
    ///
    /// **Nothing here is read as unrestricted.** Every way of not holding is a
    /// refusal, because an organisation that wrote a policy and got no policy —
    /// with nothing on the machine saying so — is the one failure worth
    /// building the whole section around.
    fn checked(self) -> Result<SourcePolicy, NotDescribed> {
        // Answered first, so that `may-go = "in-the-buiding"` written beside a
        // region is reported as the typo it is rather than as a region that
        // bounds nothing — which would send somebody to the wrong line.
        if !EVERY_PLACE.contains(&self.may_go.as_str()) {
            return Err(NotDescribed::NowhereNamedThat {
                said: self.may_go,
                every: EVERY_PLACE.join(", "),
            });
        }
        match (self.may_go.as_str(), self.region) {
            (ANYWHERE, None) => Ok(SourcePolicy::Anywhere),
            (IN_THE_BUILDING, None) => Ok(SourcePolicy::InTheBuilding),
            (THIS_MACHINE_ONLY, None) => Ok(SourcePolicy::ThisMachineOnly),
            (IN_A_REGION, Some(region)) if !region.trim().is_empty() => {
                Ok(SourcePolicy::InRegion(region))
            }
            // No region, or a region of nothing but spaces — which is the shape
            // the mistake really arrives in: somebody clearing a value rather
            // than deleting the line. `crate::described` refuses an agent's
            // name the same way and for the same reason.
            (IN_A_REGION, _) => Err(NotDescribed::NoRegionNamed),
            // Known, not `in-a-region`, and a region was written anyway. Only
            // reachable that way: the unknown spellings left above.
            (may_go, _) => Err(NotDescribed::ARegionThatBoundsNothing {
                may_go: may_go.to_owned(),
            }),
        }
    }
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
    ///
    /// `who` is which of the two permitted owners wrote the file, which
    /// `crate::trusting` established before any of this was parsed. It decides
    /// the **origin** of a bound and nothing else: root is an administrator, the
    /// person is themselves, and how strict the rule is never enters into it.
    fn checked(self, at: &std::path::Path, who: WhoDescribedIt) -> Result<Described, NotDescribed> {
        if !is_a_shape_we_read(self.format) {
            return Err(NotDescribed::AnotherFormat {
                at: at.to_owned(),
                format: self.format,
                reads: THE_FORMAT,
            });
        }
        // A description from before there was anywhere to state a policy cannot
        // state one. The section parses — it is the same shape either way — and
        // honouring it would be this service believing the half of a
        // disagreement it preferred, on a file that says an older alo OS could
        // have read it correctly when it could not.
        if self.format < THE_FORMAT && self.questions.is_some() {
            return Err(NotDescribed::APolicyNeedsANewerShape {
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
        let bound = match self.questions {
            // ADR 0016's *absent*: no rule, and nobody to name in a refusal.
            None => TheBound::Nobodys,
            Some(questions) => match who {
                WhoDescribedIt::AnAdministrator => TheBound::AnOrganisations(questions.checked()?),
                WhoDescribedIt::ThePerson => TheBound::ThePersons(questions.checked()?),
            },
        };
        Described::of(
            sides,
            &self.agent.name,
            Lasting::of_seconds(self.agent.turn_seconds, THE_TURN)?,
            Lasting::of_seconds(self.agent.proposal_seconds, THE_PROPOSAL)?,
            &self.record.path,
            self.record.keeping,
            bound,
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
pub(crate) fn read(
    said: &str,
    at: &std::path::Path,
    who: WhoDescribedIt,
) -> Result<Described, NotDescribed> {
    let not_understood = |why: toml::de::Error| NotDescribed::NotUnderstood {
        at: at.to_owned(),
        why: Box::new(why),
    };

    let which: WhichFormat = toml::from_str(said).map_err(not_understood)?;
    if !is_a_shape_we_read(which.format) {
        return Err(NotDescribed::AnotherFormat {
            at: at.to_owned(),
            format: which.format,
            reads: THE_FORMAT,
        });
    }

    let written: AsWritten = toml::from_str(said).map_err(not_understood)?;
    written.checked(at, who)
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

    /// This description, read as one an administrator wrote.
    ///
    /// Root owning `/etc/alo/agentd.toml` is the ordinary managed machine and
    /// it is what nearly every test here is about; the two that are about the
    /// **other** owner say so by calling [`the_person_wrote`], and nothing about
    /// the text decides which.
    fn an_administrator_wrote(said: &str) -> Result<Described, NotDescribed> {
        read(said, somewhere(), WhoDescribedIt::AnAdministrator)
    }

    /// The same description, read as one the person whose machine it is wrote.
    fn the_person_wrote(said: &str) -> Result<Described, NotDescribed> {
        read(said, somewhere(), WhoDescribedIt::ThePerson)
    }

    /// The same machine, in the shape that can carry a bound, with this one on
    /// it.
    fn a_machine_bounded_by(section: &str) -> String {
        format!(
            "{}\n[questions]\n{section}\n",
            an_ordinary_machine().replace("format = 1", "format = 2")
        )
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
        let machine = an_administrator_wrote(&an_ordinary_machine()).unwrap();
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
        assert_eq!(
            an_administrator_wrote(&said).unwrap().keeping().days(),
            Some(90)
        );
    }

    /// **A description from a newer alo OS is refused rather than guessed at**,
    /// and the refusal says both numbers so whoever is on call can see which
    /// way round it is.
    #[test]
    fn a_description_from_a_newer_alo_os_is_refused() {
        let said = an_ordinary_machine().replace("format = 1", "format = 3");
        let refused = an_administrator_wrote(&said).unwrap_err();
        assert!(matches!(
            refused,
            NotDescribed::AnotherFormat {
                format: 3,
                reads: 2,
                ..
            }
        ));
        assert!(
            refused.to_string().contains("/etc/alo/agentd.toml"),
            "{refused}"
        );
    }

    /// **And the shape before this one is still read**, unchanged and without
    /// anybody rewriting it. Every description that exists says `1`, and expand
    /// → migrate → contract means it goes on meaning what it meant.
    #[test]
    fn the_shape_before_this_one_is_still_read() {
        assert!(is_a_shape_we_read(1));
        assert!(is_a_shape_we_read(2));
        assert!(!is_a_shape_we_read(3));
        assert!(!is_a_shape_we_read(0));
        assert!(an_administrator_wrote(&an_ordinary_machine()).is_ok());
    }

    /// **A key nobody declared is refused**, because the only other thing to do
    /// with a typo is run under whatever the key it was meant to be says.
    #[test]
    fn a_key_nobody_declared_is_refused() {
        let said = an_ordinary_machine().replace("turn-seconds", "truns-seconds");
        let refused = an_administrator_wrote(&said).unwrap_err();
        assert!(matches!(refused, NotDescribed::NotUnderstood { .. }));
        assert!(refused.to_string().contains("truns-seconds"), "{refused}");
    }

    /// **A key left out is refused**, rather than defaulted. A machine whose
    /// description says nothing about how long its record is kept is a machine
    /// whose administrator left a line out, not one whose owner never decided.
    #[test]
    fn a_retention_left_out_is_refused_rather_than_defaulted() {
        let said = an_ordinary_machine().replace(r#"keeping = "forever""#, "");
        let refused = an_administrator_wrote(&said).unwrap_err();
        assert!(matches!(refused, NotDescribed::NotUnderstood { .. }));
        assert!(refused.to_string().contains("keeping"), "{refused}");
    }

    /// **A machine whose person and agent are one login is refused here**, which
    /// is `crate::side`'s refusal reaching the file it is really about.
    #[test]
    fn a_machine_with_one_login_is_refused() {
        let said = an_ordinary_machine().replace("agent = 989", "agent = 1000");
        let refused = an_administrator_wrote(&said).unwrap_err();
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
            an_administrator_wrote(&said).unwrap_err(),
            NotDescribed::NotTwoSides(crate::NotTwoSides::AgentAsRoot)
        ));
    }

    /// `-1` is what a Unix call answers with when there is no user, and it is
    /// what a shell script that could not look one up puts in a file.
    #[test]
    fn a_number_that_is_not_a_user_is_refused() {
        let said = an_ordinary_machine().replace("agent = 989", "agent = 4294967295");
        assert!(matches!(
            an_administrator_wrote(&said).unwrap_err(),
            NotDescribed::NotAUser(crate::NotAUser::NoSuchUser)
        ));
        let said = an_ordinary_machine().replace("group = 989", "group = 4294967295");
        assert!(matches!(
            an_administrator_wrote(&said).unwrap_err(),
            NotDescribed::NotAUser(crate::NotAUser::NoSuchGroup)
        ));
    }

    /// **A turn that lasts no time at all is refused**, and the refusal names
    /// the key rather than the concept.
    #[test]
    fn a_turn_that_lasts_no_time_is_refused_by_its_key() {
        let said = an_ordinary_machine().replace("turn-seconds = 900", "turn-seconds = 0");
        let refused = an_administrator_wrote(&said).unwrap_err();
        assert!(matches!(refused, NotDescribed::NoTimeAtAll { what } if what == THE_TURN));
    }

    /// And so is a proposal, by its own key — because *the turn is wrong* and
    /// *the proposal is wrong* send somebody to two different lines.
    #[test]
    fn a_proposal_that_stands_no_time_is_refused_by_its_own_key() {
        let said = an_ordinary_machine().replace("proposal-seconds = 300", "proposal-seconds = 0");
        let refused = an_administrator_wrote(&said).unwrap_err();
        assert!(matches!(refused, NotDescribed::NoTimeAtAll { what } if what == THE_PROPOSAL));
    }

    /// **A proposal that stands for a week is refused** — an approval is never a
    /// session, and a configuration file is not a way to make one.
    #[test]
    fn a_proposal_that_stands_for_a_week_is_refused() {
        let said =
            an_ordinary_machine().replace("proposal-seconds = 300", "proposal-seconds = 604800");
        assert!(matches!(
            an_administrator_wrote(&said).unwrap_err(),
            NotDescribed::TooLong { .. }
        ));
    }

    /// **A relative record path is refused**, carried up from the checked value
    /// with the key it was written under.
    #[test]
    fn a_relative_record_path_is_refused() {
        let said = an_ordinary_machine().replace(r#""/var/lib/alo/record""#, r#""record""#);
        assert!(matches!(
            an_administrator_wrote(&said).unwrap_err(),
            NotDescribed::NotAbsolute { what, .. } if what == THE_RECORD
        ));
    }

    /// Text that is not a description at all is refused as one, and the refusal
    /// names the file — whoever is reading it has several.
    #[test]
    fn something_that_is_not_a_description_is_refused_as_one() {
        let refused = an_administrator_wrote("this is not a machine").unwrap_err();
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
            an_administrator_wrote("").unwrap_err(),
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
            an_administrator_wrote(&said).unwrap_err(),
            NotDescribed::AnotherFormat { format: 7, .. }
        ));
    }

    // `[questions]` — ADR 0016's bound, arriving from the one file that can
    // state it.

    /// **A machine with no `[questions]` in it is unmanaged**, which is ADR
    /// 0016's *absent* rather than a permissive default: no rule, and nobody to
    /// name in a refusal. Every description that exists today is this one.
    #[test]
    fn a_machine_with_no_questions_section_is_unmanaged() {
        let machine = an_administrator_wrote(&an_ordinary_machine()).unwrap();
        assert_eq!(machine.questions(), &TheBound::Nobodys);
        assert!(!machine.questions().by_an_organisation());
    }

    /// **And so is one in the newer shape that simply has no policy in it.**
    /// Taking a bound off a machine is deleting a section, not also editing a
    /// number — the two shapes describe the same machine when neither carries
    /// one, which is the whole reason `1` is still read.
    #[test]
    fn the_newer_shape_without_a_policy_is_unmanaged_too() {
        let said = an_ordinary_machine().replace("format = 1", "format = 2");
        assert_eq!(
            an_administrator_wrote(&said).unwrap().questions(),
            &TheBound::Nobodys
        );
    }

    /// **A bound an administrator wrote reaches the daemon as an
    /// organisation's**, which is the whole point of the section: ADR 0016's
    /// rule, from the file ADR 0004 gives to whoever manages the machine.
    #[test]
    fn a_bound_an_administrator_wrote_is_an_organisations() {
        let machine =
            an_administrator_wrote(&a_machine_bounded_by(r#"may-go = "this-machine-only""#))
                .unwrap();
        assert_eq!(
            machine.questions(),
            &TheBound::AnOrganisations(SourcePolicy::ThisMachineOnly)
        );
        assert!(machine.questions().by_an_organisation());
    }

    /// **The identical rule, in a file the person owns, names no
    /// administrator** — and this is the test that says attribution is a fact
    /// about the file rather than a reading of the value. The same four words
    /// produce the same bound and a different origin, so nothing on this machine
    /// can tell somebody an administrator restricted them when nobody did.
    #[test]
    fn the_same_bound_the_person_wrote_names_no_administrator() {
        let said = a_machine_bounded_by(r#"may-go = "this-machine-only""#);
        let theirs = the_person_wrote(&said).unwrap();
        assert_eq!(
            theirs.questions(),
            &TheBound::ThePersons(SourcePolicy::ThisMachineOnly)
        );
        assert!(!theirs.questions().by_an_organisation());
        // The strictest rule there is, and it bounds identically either way.
        assert_eq!(
            theirs.questions().policy(),
            an_administrator_wrote(&said).unwrap().questions().policy()
        );
    }

    /// Every place an organisation may name is read as the rule it is.
    #[test]
    fn every_place_an_organisation_may_name_is_read() {
        for (written, rule) in [
            ("anywhere", SourcePolicy::Anywhere),
            ("in-the-building", SourcePolicy::InTheBuilding),
            ("this-machine-only", SourcePolicy::ThisMachineOnly),
        ] {
            let said = a_machine_bounded_by(&format!(r#"may-go = "{written}""#));
            assert_eq!(
                an_administrator_wrote(&said).unwrap().questions(),
                &TheBound::AnOrganisations(rule),
                "{written}"
            );
        }
    }

    /// A region is carried exactly as the organisation named it — "the EU" is
    /// theirs to spell, and nothing here normalises it.
    #[test]
    fn a_region_is_carried_exactly_as_it_was_named() {
        let said = a_machine_bounded_by("may-go = \"in-a-region\"\nregion = \"the EU\"");
        assert_eq!(
            an_administrator_wrote(&said).unwrap().questions(),
            &TheBound::AnOrganisations(SourcePolicy::InRegion("the EU".to_owned()))
        );
    }

    /// **A bound this alo OS cannot read is refused, never treated as
    /// unrestricted.** An organisation that wrote a policy and got no policy,
    /// with nothing saying so, is the one failure this section exists to
    /// prevent — so the service does not start.
    #[test]
    fn a_bound_this_alo_os_cannot_read_is_refused_rather_than_ignored() {
        let said = a_machine_bounded_by(r#"may-go = "in-the-buiding""#);
        let refused = an_administrator_wrote(&said).unwrap_err();
        assert!(matches!(refused, NotDescribed::NowhereNamedThat { .. }));
        // It says what was written and what it could have been.
        assert!(refused.to_string().contains("in-the-buiding"), "{refused}");
        assert!(refused.to_string().contains(IN_THE_BUILDING), "{refused}");
    }

    /// **A region that names none is refused**, rather than becoming a bound
    /// that holds nothing to anywhere.
    #[test]
    fn a_bound_by_region_with_no_region_is_refused() {
        let said = a_machine_bounded_by(r#"may-go = "in-a-region""#);
        assert!(matches!(
            an_administrator_wrote(&said).unwrap_err(),
            NotDescribed::NoRegionNamed
        ));
    }

    /// **And a region of nothing but spaces is no region**, which is the shape
    /// the mistake arrives in: somebody clearing a value rather than deleting
    /// the line.
    #[test]
    fn a_region_of_nothing_but_spaces_is_no_region() {
        let said = a_machine_bounded_by("may-go = \"in-a-region\"\nregion = \"   \"");
        assert!(matches!(
            an_administrator_wrote(&said).unwrap_err(),
            NotDescribed::NoRegionNamed
        ));
    }

    /// **A region beside a bound that would ignore it is refused**, because a
    /// key somebody believes is bounding their questions and is not is exactly
    /// the quiet failure the whole section is built to avoid.
    #[test]
    fn a_region_beside_a_bound_that_ignores_it_is_refused() {
        let said = a_machine_bounded_by("may-go = \"in-the-building\"\nregion = \"the EU\"");
        let refused = an_administrator_wrote(&said).unwrap_err();
        assert!(matches!(
            refused,
            NotDescribed::ARegionThatBoundsNothing { .. }
        ));
        assert!(refused.to_string().contains(IN_THE_BUILDING), "{refused}");
    }

    /// **The typo in `may-go` is answered before the region beside it**, so
    /// whoever is reading the log is sent to the line that is really wrong.
    #[test]
    fn an_unknown_bound_is_answered_before_the_region_written_beside_it() {
        let said = a_machine_bounded_by("may-go = \"in-a-regoin\"\nregion = \"the EU\"");
        assert!(matches!(
            an_administrator_wrote(&said).unwrap_err(),
            NotDescribed::NowhereNamedThat { .. }
        ));
    }

    /// **A `[questions]` with no bound in it is refused**, and the refusal names
    /// the key. An empty section is a mistake, not an absent policy — absent is
    /// no section.
    #[test]
    fn a_questions_section_with_no_bound_is_refused() {
        let said = a_machine_bounded_by("");
        let refused = an_administrator_wrote(&said).unwrap_err();
        assert!(matches!(refused, NotDescribed::NotUnderstood { .. }));
        assert!(refused.to_string().contains("may-go"), "{refused}");
    }

    /// **A key nobody declared inside `[questions]` is refused**, the same as
    /// everywhere else in this file.
    #[test]
    fn a_key_nobody_declared_inside_questions_is_refused() {
        let said = a_machine_bounded_by("may-go = \"anywhere\"\nmay-not-go = \"nowhere\"");
        let refused = an_administrator_wrote(&said).unwrap_err();
        assert!(matches!(refused, NotDescribed::NotUnderstood { .. }));
        assert!(refused.to_string().contains("may-not-go"), "{refused}");
    }

    /// **A policy written into the older shape is refused**, and this is the
    /// compatibility decision made enforceable: an alo OS reading format `1`
    /// would ignore the section and send this organisation's questions wherever
    /// the person chose. A file claiming otherwise is not half-honoured.
    #[test]
    fn a_policy_in_the_shape_that_could_not_carry_one_is_refused() {
        let said = format!(
            "{}\n[questions]\nmay-go = \"this-machine-only\"\n",
            an_ordinary_machine()
        );
        let refused = an_administrator_wrote(&said).unwrap_err();
        assert!(matches!(
            refused,
            NotDescribed::APolicyNeedsANewerShape {
                format: 1,
                reads: 2,
                ..
            }
        ));
        assert!(refused.to_string().contains("questions"), "{refused}");
    }

    /// The keys a bound is written under are named once, because they are in a
    /// refusal somebody reads and in the contract they wrote the file against.
    #[test]
    fn the_bounds_keys_are_named_once() {
        assert_eq!(THE_MAY_GO, "questions.may-go");
        assert_eq!(THE_REGION, "questions.region");
        assert_eq!(
            EVERY_PLACE,
            [
                "anywhere",
                "in-the-building",
                "in-a-region",
                "this-machine-only"
            ]
        );
    }
}
