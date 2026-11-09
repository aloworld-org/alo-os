//! **At full access the verbs reach what the person's account can, and at every
//! other level they reach what was granted** (ADR 0149) — through the agent's
//! door and the person's, on real files, with the level read from a file of
//! each test's own.
//!
//! Every folder is this test's own and is removed afterwards; the file a verb
//! reaches "outside every grant" is in a folder no grant names, which is the
//! claim made true on the disk rather than about a stand-in.

use std::path::{Path, PathBuf};

use alo_capability::HowCodeRuns;
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_protocol::{ToAPerson, ToAnAgent};
use alo_record::{Entry, Record};
use alo_strings::Strings;
use alo_turn::{Machine, Turning};

use super::*;
use crate::answering::what_a_person_said;
use crate::doing::what_an_agent_said;
use crate::holding::Holding;
use crate::network::TheNetwork;
use crate::pairing::Nearby;
use crate::rereading::WhatIsGranted;
use crate::running_a_command::Commands;
use crate::testing::{
    NobodyIsNearby, NothingIsBounded, NothingIsRemembered, a_directory_of_our_own,
    a_folder_with_an_invoice, a_message, granting, hour, in_english, noon, nothing_has_been_chosen,
    the_studio,
};
use crate::the_bracket_this_turn_takes::TheTurnsBracket;

/// The agent every turn here is for, which is the one `granting` grants to.
const THE_AGENT: &str = "@files";

/// A folder no grant names, with an invoice in it, and where this test keeps
/// the person's level.
struct Elsewhere {
    /// The folder, resolved.
    folder: PathBuf,
    /// The invoice in it.
    invoice: PathBuf,
    /// The person's level, for this test.
    level_at: PathBuf,
}

impl Elsewhere {
    /// A fresh one, with the level as given — or no file at all.
    fn with_the_level(what: &str, level: Option<HowCodeRuns>) -> Self {
        let (folder, invoice) = a_folder_with_an_invoice(&format!("{what}-elsewhere"));
        let level_at = a_directory_of_our_own(&format!("{what}-level")).join("how-code-runs.toml");
        let elsewhere = Self {
            folder,
            invoice,
            level_at,
        };
        if let Some(level) = level {
            elsewhere.choosing(level);
        }
        elsewhere
    }

    /// The person chooses this level in Settings → Running code.
    fn choosing(&self, level: HowCodeRuns) {
        alo_remembering::the_level_kept(&self.level_at, level).unwrap();
    }

    /// Where the level is read, for this test.
    fn here(&self) -> CommandsHere {
        CommandsHere::at(&self.level_at, &self.folder)
    }

    /// An agent asking to read the invoice no grant names.
    fn reading_it(&self) -> String {
        a_message(&format!(
            r#"{{"read":{{"verb":"read_file","given":[{{"named":"file","is":"{}"}}]}}}}"#,
            self.invoice.display()
        ))
    }

    /// An agent proposing to rename it.
    fn renaming_it(&self) -> String {
        a_message(&format!(
            r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"march-final.pdf"}}]}}}}"#,
            self.invoice.display()
        ))
    }

    /// Where the invoice is once renamed.
    fn renamed(&self) -> PathBuf {
        self.folder.join("march-final.pdf")
    }
}

impl Drop for Elsewhere {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.folder);
        if let Some(level) = self.level_at.parent() {
            let _ = std::fs::remove_dir_all(level);
        }
    }
}

/// A turn for [`THE_AGENT`] whose grants reach one folder of its own, on a
/// machine that carries out the six file verbs and writes into `record`.
fn on_a_turn<T>(
    what: &str,
    record: &mut Record,
    doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants, &Strings, &Path) -> T,
) -> T {
    let strings = in_english();
    let (granted, _) = a_folder_with_an_invoice(what);
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        record,
    )
    .unwrap();
    let mut grants = granting(&granted, noon());
    let mut turning = Turning::beginning(
        alo_context::Context::at_invocation(noon()),
        THE_AGENT,
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    let done = doing(&mut turning, &mut grants, &strings, &granted);
    let _ = std::fs::remove_dir_all(&granted);
    done
}

/// What the agent's door answers this line with, at the level `here` reads.
fn the_agent_asked(
    line: &str,
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    strings: &Strings,
    here: &CommandsHere,
) -> ToAnAgent {
    let mut bracket = a_bracket();
    what_an_agent_said(
        line,
        turning,
        &mut nothing_has_been_chosen(),
        None,
        grants,
        strings,
        hour(),
        noon(),
        Commands::here(here, Some(&mut bracket)),
    )
}

/// The person approves the change waiting under `number`, at the level `here`
/// reads at that moment.
fn the_person_approved(
    number: u64,
    turning: &mut Turning<'_, '_>,
    grants: &mut Grants,
    strings: &Strings,
    here: &CommandsHere,
) -> ToAPerson {
    let network = TheNetwork::on(the_studio());
    let mut bracket = a_bracket();
    what_a_person_said(
        &a_message(&format!(r#"{{"approve":{{"number":{number}}}}}"#)),
        &mut Holding::ATurn {
            turning,
            questions: &mut nothing_has_been_chosen(),
            bracket: &mut bracket,
            commands: here,
        },
        &mut WhatIsGranted::of(grants, &NothingIsRemembered),
        &Nearby {
            network: &network,
            looking: &NobodyIsNearby,
            advertising: &crate::testing::nothing_is_advertised(),
        },
        strings,
        noon(),
    )
    .unwrap()
}

/// A bracket with nowhere to go.
fn a_bracket() -> TheTurnsBracket {
    TheTurnsBracket::none_yet(None, std::time::SystemTime::UNIX_EPOCH, 0, 0)
}

/// The number a change waits under, from what the agent was told.
fn the_number(said: &ToAnAgent) -> u64 {
    match said {
        ToAnAgent::Proposed(standing) => standing.number(),
        other => unreachable!("nothing is waiting: {other:?}"),
    }
}

/// Every entry the record holds.
fn what_was_written(record: &Record) -> Vec<Entry> {
    record.everything().cloned().collect()
}

/// **The level, read at the moment**: only an explicit full access gives a
/// copy at full access, and nothing chosen and an unreadable file are never it.
#[test]
fn only_full_access_chosen_gives_full_access() {
    let agent = Grantee::named(THE_AGENT);
    let grants = Grants::default();
    let elsewhere = Elsewhere::with_the_level("level-read", None);
    assert!(
        !the_grants_as_the_level_is_now(&grants, &elsewhere.here(), &agent).has_full_access(&agent),
        "nothing chosen gave full access"
    );

    std::fs::write(&elsewhere.level_at, "this is not a level file {").unwrap();
    assert!(
        !the_grants_as_the_level_is_now(&grants, &elsewhere.here(), &agent).has_full_access(&agent),
        "a level file that will not read gave full access"
    );

    for level in [HowCodeRuns::InASealedBox, HowCodeRuns::AskingEachTime] {
        elsewhere.choosing(level);
        assert!(
            !the_grants_as_the_level_is_now(&grants, &elsewhere.here(), &agent)
                .has_full_access(&agent),
            "{level:?} gave full access"
        );
    }

    elsewhere.choosing(HowCodeRuns::AtFullTrust);
    let now = the_grants_as_the_level_is_now(&grants, &elsewhere.here(), &agent);
    assert!(now.has_full_access(&agent));
    // **And the kept list is never touched**: the copy is for this moment.
    assert!(!grants.has_full_access(&agent));
    assert!(matches!(now, std::borrow::Cow::Owned(_)));
}

/// **At full access a read reaches a file outside every grant** — and the
/// record says full access allowed it, not a grant.
#[test]
fn at_full_access_a_read_reaches_a_file_no_grant_names() {
    let elsewhere = Elsewhere::with_the_level("read-full", Some(HowCodeRuns::AtFullTrust));
    let mut record = Record::default();
    let said = on_a_turn("read-full", &mut record, |turning, grants, strings, granted| {
        assert!(!elsewhere.invoice.starts_with(granted));
        the_agent_asked(&elsewhere.reading_it(), turning, grants, strings, &elsewhere.here())
    });
    assert!(said.done().is_some(), "{said:?}");
    let written = what_was_written(&record);
    let [entry] = written.as_slice() else {
        unreachable!("one read, one entry: {written:?}");
    };
    assert!(entry.happened().ran(), "{entry:?}");
    assert!(entry.happened().by_full_access(), "{entry:?}");
    assert!(entry.happened().against().is_empty(), "{entry:?}");
    assert!(entry.was_at_full_access(), "{entry:?}");
}

/// **At full access a change reaches a file outside every grant — after the
/// person's OK, exactly once** — and is written down as full access.
#[test]
fn at_full_access_a_change_reaches_a_file_no_grant_names_after_one_ok() {
    let elsewhere = Elsewhere::with_the_level("change-full", Some(HowCodeRuns::AtFullTrust));
    let mut record = Record::default();
    let (first, again) = on_a_turn("change-full", &mut record, |turning, grants, strings, _| {
        let here = elsewhere.here();
        let said = the_agent_asked(&elsewhere.renaming_it(), turning, grants, strings, &here);
        let number = the_number(&said);
        assert!(
            elsewhere.invoice.exists(),
            "a change ran before anybody said yes"
        );
        let first = the_person_approved(number, turning, grants, strings, &here);
        let again = the_person_approved(number, turning, grants, strings, &here);
        (first, again)
    });
    assert!(first.done().is_some(), "{first:?}");
    assert!(again.refusal().is_some(), "one OK ran twice: {again:?}");
    assert!(elsewhere.renamed().exists(), "the change did not happen");
    let written = what_was_written(&record);
    let ran: Vec<_> = written.iter().filter(|entry| entry.happened().ran()).collect();
    let [ran] = ran.as_slice() else {
        unreachable!("one approval, one execution: {written:?}");
    };
    assert!(ran.happened().by_full_access(), "{ran:?}");
    assert!(ran.was_at_full_access(), "{ran:?}");
}

/// **Switching full access off takes effect immediately**: a change proposed at
/// full access and approved after the person moved to *ask me each time*, with
/// no grant covering it, is refused at execution, written down, and changes
/// nothing.
#[test]
fn switching_full_access_off_before_the_ok_refuses_the_change() {
    let elsewhere = Elsewhere::with_the_level("switched-off", Some(HowCodeRuns::AtFullTrust));
    let mut record = Record::default();
    let approved = on_a_turn("switched-off", &mut record, |turning, grants, strings, _| {
        let here = elsewhere.here();
        let said = the_agent_asked(&elsewhere.renaming_it(), turning, grants, strings, &here);
        let number = the_number(&said);
        elsewhere.choosing(HowCodeRuns::AskingEachTime);
        the_person_approved(number, turning, grants, strings, &here)
    });
    assert!(approved.refusal().is_some(), "{approved:?}");
    assert!(approved.done().is_none(), "{approved:?}");
    assert!(elsewhere.invoice.exists(), "the change ran after full access was switched off");
    assert!(!elsewhere.renamed().exists());
    let written = what_was_written(&record);
    let [refused] = written.as_slice() else {
        unreachable!("the refusal is the one entry: {written:?}");
    };
    assert!(refused.happened().was_stopped(), "{refused:?}");
    assert!(
        !refused.was_at_full_access(),
        "a refusal decided after full access was switched off says it was at full access"
    );
}

/// **And for the next read**: a read at full access answers, the person
/// switches it off, and the same read is refused at once — and both are
/// written down, the first as full access and the second not.
#[test]
fn switching_full_access_off_refuses_the_next_read() {
    let elsewhere = Elsewhere::with_the_level("next-read", Some(HowCodeRuns::AtFullTrust));
    let mut record = Record::default();
    let (before, after) = on_a_turn("next-read", &mut record, |turning, grants, strings, _| {
        let here = elsewhere.here();
        let before = the_agent_asked(&elsewhere.reading_it(), turning, grants, strings, &here);
        elsewhere.choosing(HowCodeRuns::InASealedBox);
        let after = the_agent_asked(&elsewhere.reading_it(), turning, grants, strings, &here);
        (before, after)
    });
    assert!(before.done().is_some(), "{before:?}");
    assert!(after.refusal().is_some(), "{after:?}");
    let written = what_was_written(&record);
    let [ran, refused] = written.as_slice() else {
        unreachable!("two reads, two entries: {written:?}");
    };
    assert!(ran.happened().ran() && ran.was_at_full_access(), "{ran:?}");
    assert!(
        refused.happened().was_stopped() && !refused.was_at_full_access(),
        "{refused:?}"
    );
}

/// **At *ask me each time*, the sealed box, with nothing chosen and with a file
/// that will not read**, a read and a change outside the grants are refused and
/// written down — full access is the only level that widens anything.
#[test]
fn every_other_level_still_refuses_a_file_no_grant_names() {
    for (what, level) in [
        ("asking", Some(HowCodeRuns::AskingEachTime)),
        ("sealed", Some(HowCodeRuns::InASealedBox)),
        ("nothing-chosen", None),
        ("unreadable", None),
    ] {
        let elsewhere = Elsewhere::with_the_level(&format!("refused-{what}"), level);
        if what == "unreadable" {
            std::fs::write(&elsewhere.level_at, "chose = [\"everything\"").unwrap();
        }
        let mut record = Record::default();
        let (read, proposed) =
            on_a_turn(&format!("refused-{what}"), &mut record, |turning, grants, strings, _| {
                let here = elsewhere.here();
                (
                    the_agent_asked(&elsewhere.reading_it(), turning, grants, strings, &here),
                    the_agent_asked(&elsewhere.renaming_it(), turning, grants, strings, &here),
                )
            });
        assert!(read.refusal().is_some(), "{what}: {read:?}");
        assert!(proposed.refusal().is_some(), "{what}: {proposed:?}");
        assert!(!proposed.waits_for_a_person(), "{what}: {proposed:?}");
        assert!(elsewhere.invoice.exists(), "{what}");
        let written = what_was_written(&record);
        assert_eq!(written.len(), 2, "{what}: every refusal leaves a record: {written:?}");
        for entry in &written {
            assert!(entry.happened().was_stopped(), "{what}: {entry:?}");
            assert!(!entry.was_at_full_access(), "{what}: {entry:?}");
        }
    }
}

/// **At *ask me each time* a change inside the grants does not run without the
/// OK, and runs once with it** — grants work at every level, and full access
/// is not needed for what was granted.
#[test]
fn asking_each_time_a_granted_change_waits_for_the_ok() {
    let elsewhere = Elsewhere::with_the_level("asking-granted", Some(HowCodeRuns::AskingEachTime));
    let mut record = Record::default();
    let renamed = on_a_turn("asking-granted", &mut record, |turning, grants, strings, granted| {
        let here = elsewhere.here();
        let invoice = granted.join("march.pdf");
        let line = a_message(&format!(
            r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"march-final.pdf"}}]}}}}"#,
            invoice.display()
        ));
        let said = the_agent_asked(&line, turning, grants, strings, &here);
        let number = the_number(&said);
        assert!(invoice.exists(), "it ran before the OK");
        let approved = the_person_approved(number, turning, grants, strings, &here);
        assert!(approved.done().is_some(), "{approved:?}");
        granted.join("march-final.pdf").exists()
    });
    assert!(renamed, "the approved change did not run");
    let ran: Vec<_> = what_was_written(&record)
        .into_iter()
        .filter(|entry| entry.happened().ran())
        .collect();
    let [ran] = ran.as_slice() else {
        unreachable!("one approval, one execution: {ran:?}");
    };
    assert!(!ran.happened().by_full_access(), "{ran:?}");
    assert_eq!(ran.happened().against().len(), 1, "{ran:?}");
    assert!(!ran.was_at_full_access(), "{ran:?}");
}
