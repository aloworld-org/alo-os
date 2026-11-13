//! **Every run and every refusal of a command leaves a record**, one approval is
//! one run, and a stale or repeated number runs nothing (ADR 0149).
//!
//! The command is run by a stand-in here, which counts what it was handed: this
//! crate starts nothing (`alo-running` does), and what is under test is the
//! turn's half — the record, the numbers, and that nothing runs unasked.

use std::cell::RefCell;

use alo_capability::{ACommand, AnswerError, AroundACommand, ItDidNotStart, WhatACommandDid};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_record::{Happened, Record};
use alo_strings::Filling;

use super::*;
use crate::machine::Machine;
use crate::testing::{NothingIsBounded, hour, in_english, noon};

/// A command these tests ask for.
fn making_a_folder() -> ACommand {
    ACommand::of("mkdir -p /home/anna/demo").unwrap()
}

/// A turn on a machine whose record is `record`, handed to `doing`.
fn on_a_turn<T>(record: &mut Record, doing: impl FnOnce(&mut Turning<'_, '_>) -> T) -> T {
    let strings = in_english();
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
    let mut grants = alo_capability::Grants::default();
    let mut turning = Turning::beginning(
        Context::at_invocation(noon()),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    doing(&mut turning)
}

/// A stand-in that runs nothing and remembers every command it was handed.
fn counting(
    ran: &RefCell<Vec<String>>,
) -> impl FnMut(&ACommand, &mut dyn AroundACommand) -> Result<WhatACommandDid, ItDidNotStart> + '_ {
    move |command: &ACommand, _: &mut dyn AroundACommand| {
        ran.borrow_mut().push(command.text().to_owned());
        Ok(WhatACommandDid::of(
            Some(0),
            "done".to_owned(),
            false,
            false,
        ))
    }
}

/// The one entry a record holds, and what happened in it.
fn the_one_entry(record: &Record) -> Happened {
    let entries: Vec<_> = record.everything().collect();
    assert_eq!(entries.len(), 1, "{entries:?}");
    entries.first().unwrap().happened().clone()
}

/// **At full access it runs, without a grant, and the run is written down**:
/// the shell, the command whole, no folders allowed, nothing boxed, and the
/// place kept before it.
#[test]
fn a_command_that_runs_is_written_down_with_the_place_kept_before_it() {
    let ran = RefCell::new(Vec::new());
    let mut record = Record::default();
    on_a_turn(&mut record, |turning| {
        turning.a_place_was_kept("1760000000-7-1");
        let did = turning
            .running_a_command(&making_a_folder(), &mut counting(&ran), noon())
            .unwrap();
        assert_eq!(did.returned(), Some(0));
    });
    assert_eq!(ran.borrow().as_slice(), ["mkdir -p /home/anna/demo"]);
    let entry = record.everything().next().unwrap();
    assert!(
        entry
            .kept_before_it()
            .is_some_and(|kept| kept.is("1760000000-7-1")),
        "Put back cannot be asked from this run"
    );
    // **It says it was at full access**, so the record a person reads back
    // says what let it run rather than claim a box (ADR 0149).
    assert!(entry.was_at_full_access(), "{entry:?}");
    let happened = the_one_entry(&record);
    assert!(
        matches!(happened, Happened::RanACommand { .. }),
        "{happened:?}"
    );
    if let Happened::RanACommand {
        agent,
        program,
        arguments,
        allowed,
        returned,
        the_kernel_was_holding_it,
    } = happened
    {
        assert!(agent.is("@files"));
        assert!(program.is("/bin/sh"));
        assert_eq!(
            arguments
                .iter()
                .map(alo_record::Line::as_str)
                .collect::<Vec<_>>(),
            ["-c", "mkdir -p /home/anna/demo"]
        );
        assert!(allowed.is_empty(), "a grant was named for a command");
        assert_eq!(returned, Some(0));
        assert!(!the_kernel_was_holding_it);
    }
}

/// **A shell that would not start is written down as a command not run**, and
/// the turn says so in the person's words.
#[test]
fn a_command_that_did_not_start_is_written_down_as_not_run() {
    let mut record = Record::default();
    on_a_turn(&mut record, |turning| {
        let refused = turning
            .running_a_command(
                &making_a_folder(),
                &mut |_: &ACommand, _: &mut dyn AroundACommand| {
                    Err(ItDidNotStart::because("no shell"))
                },
                noon(),
            )
            .unwrap_err();
        assert!(matches!(refused, NotDone::ACommandDidNotStart(_)));
        assert!(!refused.said(&in_english()).is_a_bug());
    });
    let happened = the_one_entry(&record);
    assert!(
        matches!(happened, Happened::ACommandWasNotRun { .. }),
        "{happened:?}"
    );
}

/// **At *ask me each time* nothing runs before the OK; one OK runs it exactly
/// once; the same number again runs nothing.**
#[test]
fn a_waiting_command_runs_once_after_one_approval_and_never_again() {
    let ran = RefCell::new(Vec::new());
    let mut record = Record::default();
    on_a_turn(&mut record, |turning| {
        let id = turning
            .proposing_a_command(making_a_folder(), hour(), noon())
            .unwrap();
        assert!(ran.borrow().is_empty(), "it ran before anybody said yes");
        assert_eq!(turning.commands_waiting_at(noon()).count(), 1);
        assert!(turning.command_proposed(id).is_some());
        assert_eq!(turning.became(id.as_u64(), noon()), Became::Waiting);

        turning
            .approving_a_command(id, &mut counting(&ran), noon())
            .unwrap();
        assert_eq!(ran.borrow().len(), 1);
        assert_eq!(turning.became(id.as_u64(), noon()), Became::Approved);

        let again = turning
            .approving_a_command(id, &mut counting(&ran), noon())
            .unwrap_err();
        assert_eq!(
            again,
            NotDone::NotAnswered(AnswerError::NothingWaiting {
                number: id.as_u64()
            })
        );
        assert_eq!(ran.borrow().len(), 1, "one approval ran it twice");
    });
    // The proposal wrote nothing; the run wrote one entry — and not as one at
    // full access, because what let it run was the person's yes.
    assert!(matches!(
        the_one_entry(&record),
        Happened::RanACommand { .. }
    ));
    assert!(!record.everything().next().unwrap().was_at_full_access());
}

/// **A question that stood too long runs nothing, and neither does the same
/// number afterwards.**
#[test]
fn a_command_that_waited_too_long_runs_nothing() {
    let ran = RefCell::new(Vec::new());
    let mut record = Record::default();
    on_a_turn(&mut record, |turning| {
        let id = turning
            .proposing_a_command(making_a_folder(), hour(), noon())
            .unwrap();
        let later = noon() + hour();
        assert_eq!(turning.became(id.as_u64(), later), Became::Lapsed);
        let lapsed = turning
            .approving_a_command(id, &mut counting(&ran), later)
            .unwrap_err();
        assert!(matches!(
            lapsed,
            NotDone::NotAnswered(AnswerError::ACommandLapsed { .. })
        ));
        let stale = turning
            .approving_a_command(id, &mut counting(&ran), noon())
            .unwrap_err();
        assert!(
            matches!(
                stale,
                NotDone::NotAnswered(AnswerError::NothingWaiting { .. })
            ),
            "{stale:?}"
        );
    });
    assert!(ran.borrow().is_empty(), "a stale number ran a command");
}

/// **A change's number runs no command, and a command's number approves no
/// change** — the two share numbers and never an answer.
#[test]
fn a_command_and_a_change_never_answer_for_each_other() {
    let ran = RefCell::new(Vec::<String>::new());
    let mut record = Record::default();
    on_a_turn(&mut record, |turning| {
        let id = turning
            .proposing_a_command(making_a_folder(), hour(), noon())
            .unwrap();
        let grants = alo_capability::Grants::default();
        assert!(matches!(
            turning.approving(id, &grants, noon()),
            Err(NotDone::NotAnswered(AnswerError::NothingWaiting { .. }))
        ));
        assert_eq!(turning.commands_waiting_at(noon()).count(), 1);
    });
    assert!(ran.borrow().is_empty());
}

/// **A no is written down, and nothing runs.**
#[test]
fn a_declined_command_is_written_down_and_never_runs() {
    let ran = RefCell::new(Vec::new());
    let mut record = Record::default();
    on_a_turn(&mut record, |turning| {
        let id = turning
            .proposing_a_command(making_a_folder(), hour(), noon())
            .unwrap();
        turning.declining(id, noon()).unwrap();
        assert_eq!(turning.became(id.as_u64(), noon()), Became::Declined);
        assert!(
            turning
                .approving_a_command(id, &mut counting(&ran), noon())
                .is_err()
        );
        assert!(
            turning.declining(id, noon()).is_err(),
            "a no was answered twice"
        );
    });
    assert!(ran.borrow().is_empty());
    let happened = the_one_entry(&record);
    assert!(
        matches!(&happened, Happened::ACommandWasNotRun { why, .. } if why.as_str().contains("you said no")),
        "{happened:?}"
    );
}

/// **A refusal the turn did not make is written down with its sentence**: the
/// sealed box's, here, as the service would hand it in.
#[test]
fn a_refused_command_is_written_down_with_the_sentence_the_person_reads() {
    let mut record = Record::default();
    let strings = in_english();
    let said = strings.say(
        &alo_capability::words::THE_SEALED_BOX_CANNOT_RUN_PROGRAMS_HERE_YET.key(),
        &Filling::of("program", "mkdir"),
    );
    on_a_turn(&mut record, |turning| {
        turning.a_command_was_not_run(&said, noon()).unwrap();
    });
    let happened = the_one_entry(&record);
    assert!(
        matches!(
            &happened,
            Happened::ACommandWasNotRun { why, program, .. }
                if program.is("/bin/sh") && why.as_str().starts_with("mkdir was not run.")
        ),
        "{happened:?}"
    );
}
