//! **Every administrator action leaves a record saying it was one and what the
//! person did**, one approval is one run, and a refusal runs nothing (ADR 0150).
//!
//! The command is run by a stand-in that counts what it was handed: this crate
//! starts nothing, and what is under test is the turn's half.

use std::cell::RefCell;

use alo_capability::{ACommand, WhatACommandDid};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_record::{AsAnAdministrator, Happened, Record};
use alo_strings::Filling;

use super::*;
use crate::machine::Machine;
use crate::testing::{NothingIsBounded, hour, in_english, noon};

/// A command these tests ask for as administrator.
fn installing() -> ACommand {
    ACommand::of("sudo dnf install -y htop").unwrap()
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

/// What the stand-in answers.
#[derive(Clone, Copy)]
enum Answering {
    /// The person gave their password.
    Yes,
    /// They did this instead.
    No(AsAnAdministrator),
}

/// A stand-in that runs nothing, remembers every command, and answers `with`.
fn counting(
    ran: &RefCell<Vec<String>>,
    with: Answering,
) -> impl FnMut(&ACommand) -> Result<WhatACommandDid, (Said, AsAnAdministrator)> + '_ {
    move |command| {
        ran.borrow_mut().push(command.text().to_owned());
        match with {
            Answering::Yes => Ok(WhatACommandDid::of(
                Some(0),
                "installed".to_owned(),
                false,
                false,
            )),
            Answering::No(how) => Err((
                in_english().say(
                    &alo_capability::words::NOTHING_RAN_THEY_CANCELLED.key(),
                    &Filling::nothing(),
                ),
                how,
            )),
        }
    }
}

/// **A command run as administrator is written down as one, and that the
/// person gave their password.**
#[test]
fn a_run_is_written_down_as_an_administrators() {
    let ran = RefCell::new(Vec::new());
    let mut record = Record::default();
    let came = on_a_turn(&mut record, |turning| {
        turning
            .running_as_administrator(&installing(), &mut counting(&ran, Answering::Yes), noon())
            .unwrap()
    });
    assert_eq!(came.unwrap().said(), "installed");
    assert_eq!(*ran.borrow(), ["sudo dnf install -y htop"]);
    let entries: Vec<_> = record.everything().collect();
    assert_eq!(entries.len(), 1);
    let entry = entries.first().unwrap();
    assert!(matches!(entry.happened(), Happened::RanACommand { .. }));
    assert_eq!(
        entry.asked_as_an_administrator(),
        Some(AsAnAdministrator::TheyGaveTheirPassword)
    );
    assert!(
        entry.was_at_full_access(),
        "a run at full access was written down as one at another level"
    );
}

/// **Every refusal is written down with the sentence and what the person
/// did**, and is answered as a refusal, never as a run.
#[test]
fn every_refusal_is_written_down_with_what_the_person_did() {
    for how in [
        AsAnAdministrator::TheyCancelled,
        AsAnAdministrator::NotAccepted,
        AsAnAdministrator::NothingCouldAsk,
        AsAnAdministrator::NotAnAdministrator,
        AsAnAdministrator::NoAnswerInTime,
        AsAnAdministrator::CouldNotAsk,
    ] {
        let ran = RefCell::new(Vec::new());
        let mut record = Record::default();
        let came = on_a_turn(&mut record, |turning| {
            turning
                .running_as_administrator(
                    &installing(),
                    &mut counting(&ran, Answering::No(how)),
                    noon(),
                )
                .unwrap()
        });
        assert!(came.is_err(), "{how:?}");
        let entries: Vec<_> = record.everything().collect();
        assert_eq!(entries.len(), 1, "{how:?}");
        let entry = entries.first().unwrap();
        assert!(
            matches!(entry.happened(), Happened::ACommandWasNotRun { .. }),
            "{how:?}"
        );
        assert_eq!(entry.asked_as_an_administrator(), Some(how));
    }
}

/// **One OK at *ask me each time* is one password asked for and one run**, and
/// the same number a second time runs nothing.
#[test]
fn one_approval_is_one_run_as_administrator() {
    let ran = RefCell::new(Vec::new());
    let mut record = Record::default();
    on_a_turn(&mut record, |turning| {
        let id = turning
            .proposing_a_command(installing(), hour(), noon())
            .unwrap();
        assert!(
            turning
                .approving_a_command_as_administrator(
                    id,
                    &mut counting(&ran, Answering::Yes),
                    noon()
                )
                .unwrap()
                .is_ok()
        );
        assert!(
            turning
                .approving_a_command_as_administrator(
                    id,
                    &mut counting(&ran, Answering::Yes),
                    noon()
                )
                .is_err()
        );
    });
    assert_eq!(ran.borrow().len(), 1);
    let entries: Vec<_> = record.everything().collect();
    assert!(
        entries.iter().all(|entry| !entry.was_at_full_access()),
        "a run the person said yes to was written down as one at full access"
    );
}
