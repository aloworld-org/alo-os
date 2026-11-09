//! **At full access a change is carried out at once, and at every other level
//! it waits for one OK** (ADR 0149) — through the agent's door and the
//! person's, on real files, with the level read from a file of each test's own
//! at the moment of each change.

use std::path::{Path, PathBuf};

use alo_capability::HowCodeRuns;
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_protocol::{ToAPerson, ToAnAgent};
use alo_record::{Entry, Record};
use alo_turn::{Machine, Turning};

use super::*;
use crate::answering::what_a_person_said;
use crate::doing::what_an_agent_said;
use crate::holding::Holding;
use crate::network::TheNetwork;
use crate::pairing::Nearby;
use crate::rereading::WhatIsGranted;
use crate::running_a_command::CommandsHere;
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

/// An agent proposing to rename `file` to `march-final.pdf`.
fn renaming(file: &Path) -> String {
    a_message(&format!(
        r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"march-final.pdf"}}]}}}}"#,
        file.display()
    ))
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

/// What the agent's door answers this line with, at the level `here` reads,
/// with the turn's bracket.
fn the_agent_asked(
    line: &str,
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    strings: &Strings,
    here: &CommandsHere,
    bracket: &mut TheTurnsBracket,
) -> ToAnAgent {
    what_an_agent_said(
        line,
        turning,
        &mut nothing_has_been_chosen(),
        None,
        grants,
        strings,
        hour(),
        noon(),
        Commands::here(here, Some(bracket)),
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

/// A bracket with nowhere to go: a session with no home, which is still asked
/// for its snapshot before every change it runs, and keeps none.
fn a_bracket() -> TheTurnsBracket {
    TheTurnsBracket::none_yet(None, std::time::SystemTime::UNIX_EPOCH, 0, 0)
}

/// The number a change waits under, from what the agent was told — and
/// [`None`] when nothing waits.
fn waiting_under(said: &ToAnAgent) -> Option<u64> {
    match said {
        ToAnAgent::Proposed(standing) => Some(standing.number()),
        _ => None,
    }
}

/// What the person is told was done without asking, if it was.
fn told_it_was_done(said: &ToAnAgent) -> Option<String> {
    match said {
        ToAnAgent::DoneWithoutAsking { said, .. } => Some(said.text().to_owned()),
        _ => None,
    }
}

/// Every entry the record holds.
fn what_was_written(record: &Record) -> Vec<Entry> {
    record.everything().cloned().collect()
}

/// **At full access a change is carried out at once, without any OK and
/// exactly once**, on a file no grant names: the snapshot is asked for first,
/// nothing is left waiting for the person, the person is told it was done and
/// that it cannot be put back (this bracket keeps nothing), and the record says
/// it ran at full access from no approval.
#[test]
fn at_full_access_a_change_is_carried_out_at_once_without_an_ok() {
    let elsewhere = Elsewhere::with_the_level("done-at-once", Some(HowCodeRuns::AtFullTrust));
    let mut record = Record::default();
    let mut bracket = a_bracket();
    let (said, nothing_to_approve) = on_a_turn(
        "done-at-once",
        &mut record,
        |turning, grants, strings, granted| {
            assert!(!elsewhere.invoice.starts_with(granted));
            let here = elsewhere.here();
            let said = the_agent_asked(
                &renaming(&elsewhere.invoice),
                turning,
                grants,
                strings,
                &here,
                &mut bracket,
            );
            assert_eq!(
                turning.waiting_at(noon()).count(),
                0,
                "a change was left waiting"
            );
            // The person's door has nothing to approve: no number was handed out.
            let nothing = the_person_approved(0, turning, grants, strings, &here);
            (said, nothing)
        },
    );
    let told = told_it_was_done(&said).unwrap();
    assert!(told.contains("Done without asking you first"), "{told}");
    assert!(told.contains("full access"), "{told}");
    assert!(told.contains("march-final.pdf"), "{told}");
    assert!(told.contains("can't be put back"), "{told}");
    assert!(said.done().is_some(), "{said:?}");
    assert!(
        nothing_to_approve.refusal().is_some(),
        "{nothing_to_approve:?}"
    );

    assert!(
        elsewhere.renamed().is_file(),
        "the change was not carried out"
    );
    assert!(!elsewhere.invoice.exists());
    assert!(
        bracket
            .the_first_sentence_for_tests()
            .is_some_and(|sentence| sentence.contains("march-final.pdf")),
        "the snapshot was not asked for before the change: {:?}",
        bracket.the_first_sentence_for_tests()
    );

    let written = what_was_written(&record);
    let ran: Vec<_> = written
        .iter()
        .filter(|entry| entry.happened().ran())
        .collect();
    let [ran] = ran.as_slice() else {
        unreachable!("one change, one execution: {written:?}");
    };
    assert_eq!(ran.happened().from_approval(), None, "{ran:?}");
    assert!(ran.happened().by_full_access(), "{ran:?}");
    assert!(ran.was_at_full_access(), "{ran:?}");
}

/// **At *ask me each time*, the sealed box, with nothing chosen and with a
/// level file that will not read**, a granted change waits: nothing runs and no
/// snapshot is taken without the OK, one OK runs it exactly once, and the same
/// number again runs nothing.
#[test]
fn at_every_other_level_a_change_waits_for_one_ok() {
    for (what, level) in [
        ("asking", Some(HowCodeRuns::AskingEachTime)),
        ("sealed", Some(HowCodeRuns::InASealedBox)),
        ("nothing-chosen", None),
        ("unreadable", None),
    ] {
        let elsewhere = Elsewhere::with_the_level(&format!("waits-{what}"), level);
        if what == "unreadable" {
            std::fs::write(&elsewhere.level_at, "chose = [\"everything\"").unwrap();
        }
        let mut record = Record::default();
        let mut bracket = a_bracket();
        let (waited, first, again, renamed) = on_a_turn(
            &format!("waits-{what}"),
            &mut record,
            |turning, grants, strings, granted| {
                let here = elsewhere.here();
                let invoice = granted.join("march.pdf");
                let said = the_agent_asked(
                    &renaming(&invoice),
                    turning,
                    grants,
                    strings,
                    &here,
                    &mut bracket,
                );
                assert!(told_it_was_done(&said).is_none(), "{what}: {said:?}");
                assert!(invoice.is_file(), "{what}: it ran before the OK");
                let Some(number) = waiting_under(&said) else {
                    return (false, None, None, false);
                };
                let first = the_person_approved(number, turning, grants, strings, &here);
                let again = the_person_approved(number, turning, grants, strings, &here);
                (
                    true,
                    Some(first),
                    Some(again),
                    granted.join("march-final.pdf").is_file(),
                )
            },
        );
        assert!(waited, "{what}: the change did not wait for the OK");
        assert!(
            bracket.the_first_sentence_for_tests().is_none(),
            "{what}: a snapshot was taken on the agent's door for a change that waited"
        );
        assert!(first.is_some_and(|first| first.done().is_some()), "{what}");
        assert!(
            again.is_some_and(|again| again.refusal().is_some()),
            "{what}: one OK ran twice"
        );
        assert!(renamed, "{what}: the approved change did not run");
        let written = what_was_written(&record);
        let ran: Vec<_> = written
            .iter()
            .filter(|entry| entry.happened().ran())
            .collect();
        let [ran] = ran.as_slice() else {
            unreachable!("{what}: one approval, one execution: {written:?}");
        };
        assert!(ran.happened().from_approval().is_some(), "{what}: {ran:?}");
        assert!(!ran.was_at_full_access(), "{what}: {ran:?}");
    }
}

/// **Switching full access off between two changes**: the first is carried out
/// at once; the person moves to *ask me each time*; the second, granted, waits
/// for the OK, and one outside every grant is refused — the level is read at
/// the moment of each change.
#[test]
fn switching_full_access_off_between_two_changes_makes_the_second_wait() {
    let elsewhere = Elsewhere::with_the_level("switched", Some(HowCodeRuns::AtFullTrust));
    let mut record = Record::default();
    let mut bracket = a_bracket();
    let (first, second, third, still_there) = on_a_turn(
        "switched",
        &mut record,
        |turning, grants, strings, granted| {
            let here = elsewhere.here();
            let first = the_agent_asked(
                &renaming(&elsewhere.invoice),
                turning,
                grants,
                strings,
                &here,
                &mut bracket,
            );
            elsewhere.choosing(HowCodeRuns::AskingEachTime);
            let invoice = granted.join("march.pdf");
            let second = the_agent_asked(
                &renaming(&invoice),
                turning,
                grants,
                strings,
                &here,
                &mut bracket,
            );
            let april = elsewhere.folder.join("april.pdf");
            std::fs::write(&april, "April, 3920.00").unwrap();
            let third = the_agent_asked(
                &renaming(&april),
                turning,
                grants,
                strings,
                &here,
                &mut bracket,
            );
            (first, second, third, invoice.is_file() && april.is_file())
        },
    );
    assert!(told_it_was_done(&first).is_some(), "{first:?}");
    assert!(waiting_under(&second).is_some(), "{second:?}");
    assert!(third.refusal().is_some(), "{third:?}");
    assert!(
        still_there,
        "a change ran after full access was switched off"
    );
    let written = what_was_written(&record);
    let [ran, refused] = written.as_slice() else {
        unreachable!("one run and one refusal, and a question is not written down: {written:?}");
    };
    assert!(ran.happened().ran() && ran.was_at_full_access(), "{ran:?}");
    assert!(
        refused.happened().was_stopped() && !refused.was_at_full_access(),
        "{refused:?}"
    );
}

/// **A change the disk refuses at full access is answered as not done** — a
/// refusal in the machine's words, never *done without asking* — and written
/// down at full access with what the machine said beside it, so the record
/// does not read as a change that worked. Nothing is renamed.
#[test]
fn at_full_access_a_change_the_disk_refuses_is_not_told_as_done() {
    let elsewhere = Elsewhere::with_the_level("disk-says-no", Some(HowCodeRuns::AtFullTrust));
    std::fs::write(elsewhere.renamed(), "already here").unwrap();
    let mut record = Record::default();
    let mut bracket = a_bracket();
    let said = on_a_turn(
        "disk-says-no",
        &mut record,
        |turning, grants, strings, _| {
            the_agent_asked(
                &renaming(&elsewhere.invoice),
                turning,
                grants,
                strings,
                &elsewhere.here(),
                &mut bracket,
            )
        },
    );
    assert!(told_it_was_done(&said).is_none(), "{said:?}");
    assert!(said.refusal().is_some(), "{said:?}");
    assert!(elsewhere.invoice.is_file());
    assert_eq!(
        std::fs::read_to_string(elsewhere.renamed()).unwrap(),
        "already here"
    );
    let written = what_was_written(&record);
    let [entry] = written.as_slice() else {
        unreachable!("one attempt, one entry: {written:?}");
    };
    assert!(entry.was_at_full_access(), "{entry:?}");
    assert!(
        !entry.told().is_empty(),
        "an attempt the disk refused was written down with nothing saying so: {entry:?}"
    );
}
