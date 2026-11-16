//! A turn paused for an application, told what the person answered: the agent
//! reads `alo-granted:` only after the grant is on the disk, `alo-not-granted:`
//! for every other answer — and at once when nothing was wanted — and a Stop
//! while it is paused travels on the turn the sheet was holding.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader};
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixStream;
use std::time::{Duration, SystemTime};

use alo_capability::{Ask, Grantee};
use alo_models::InferenceSource;
use alo_tasks::ATask;

use crate::the_application_sheet::{ApplicationSheetKey, OnTheApplicationSheet};
use crate::the_tasks_a_person_opened::WhichTask;

/// VSCodium's identifier.
const CODIUM: &str = "com.vscodium.codium";

/// A session told where its grants are, a task alo is working in, and the two
/// ends of that task's turn: the shell's, and the agent's.
fn a_paused_turn(
    named: &str,
) -> (
    tempfile::TempDir,
    crate::settings_testing::Machine,
    crate::Server,
    WhichTask,
    UnixStream,
    UnixStream,
) {
    let runtime = tempfile::tempdir().unwrap();
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut server = crate::Server::bind(runtime.path(), named).unwrap();
    let machine = crate::settings_testing::a_persons_machine(named);
    server.the_settings_places_are(machine.places.clone());
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "open my project in VSCodium",
        InferenceSource::ThisMachine,
    ));
    let (shell, agent) = UnixStream::pair().unwrap();
    agent
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    (runtime, machine, server, which, shell, agent)
}

/// The one line the agent reads, or none.
fn what_the_agent_reads(agent: &UnixStream) -> Option<String> {
    let mut line = String::new();
    match BufReader::new(agent).read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line),
    }
}

/// Whether the daemon, reading the file now, lets alo use VSCodium.
fn the_daemon_permits(machine: &crate::settings_testing::Machine) -> bool {
    alo_remembering::remembered(machine.places.grants(), SystemTime::now())
        .unwrap()
        .permits(
            &Grantee::named("alo"),
            &Ask::application(CODIUM),
            SystemTime::now(),
        )
}

/// **Nothing wanted: the turn goes on at once with the refusal**, no sheet
/// goes up, and the turn is handed back to be heard on.
#[test]
fn nothing_wanted_tells_the_turn_not_granted_at_once() {
    let (_runtime, _machine, mut server, which, shell, agent) = a_paused_turn("paused-nothing");
    server.a_call_for_an_application_was_refused(which, shell, &[]);
    assert!(!server.application_sheet.is_up());
    assert_eq!(
        what_the_agent_reads(&agent).as_deref(),
        Some("alo-not-granted:\n")
    );
    assert_eq!(server.take_the_turns_to_hear_again().len(), 1);
}

/// **Wanted: the sheet goes up and the turn waits, told nothing** until the
/// person answers.
#[test]
fn wanted_raises_the_sheet_and_tells_the_turn_nothing_yet() {
    let (_runtime, _machine, mut server, which, shell, agent) = a_paused_turn("paused-wanted");
    agent
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    server.a_call_for_an_application_was_refused(which, shell, &[CODIUM.to_owned()]);
    assert!(server.application_sheet.is_up());
    server.the_paused_turns_are_told();
    assert_eq!(
        what_the_agent_reads(&agent),
        None,
        "the turn was told before an answer"
    );
    assert!(server.take_the_turns_to_hear_again().is_empty());
}

/// **Granted: the agent reads `alo-granted:` only once the grant is on the
/// disk**, and the turn goes back to be heard on.
#[test]
fn granted_is_told_after_the_grant_is_on_the_disk() {
    let (_runtime, machine, mut server, which, shell, agent) = a_paused_turn("paused-granted");
    server.a_call_for_an_application_was_refused(which, shell, &[CODIUM.to_owned()]);
    server.the_application_sheet_key(ApplicationSheetKey::Grant);
    assert!(the_daemon_permits(&machine), "the grant was not made");
    server.the_paused_turns_are_told();
    assert_eq!(
        what_the_agent_reads(&agent).as_deref(),
        Some("alo-granted:\n")
    );
    assert_eq!(server.take_the_turns_to_hear_again().len(), 1);
}

/// **Not now: the agent reads `alo-not-granted:`**, and nothing is granted.
#[test]
fn not_now_is_told_as_not_granted_and_grants_nothing() {
    let (_runtime, machine, mut server, which, shell, agent) = a_paused_turn("paused-not-now");
    server.a_call_for_an_application_was_refused(which, shell, &[CODIUM.to_owned()]);
    server.the_application_sheet_pressed(OnTheApplicationSheet::NotNow);
    server.the_paused_turns_are_told();
    assert_eq!(
        what_the_agent_reads(&agent).as_deref(),
        Some("alo-not-granted:\n")
    );
    assert!(!the_daemon_permits(&machine));
}

/// **A Stop while it is paused takes the turn the sheet was holding**, so the
/// Stop travels on it, and the sheet comes down granting nothing.
#[test]
fn a_stop_while_paused_travels_on_the_held_turn() {
    let (_runtime, machine, mut server, which, shell, agent) = a_paused_turn("paused-stop");
    agent
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    server.a_call_for_an_application_was_refused(which, shell, &[CODIUM.to_owned()]);
    assert!(server.the_person_stops_a_task(which));
    assert!(!server.application_sheet.is_up());
    let stops = server.take_the_stops();
    assert!(
        stops
            .iter()
            .any(|stop| stop.which == which && stop.held_turn.is_some()),
        "the Stop did not take the held turn"
    );
    server.the_paused_turns_are_told();
    assert!(server.take_the_turns_to_hear_again().is_empty());
    assert!(!the_daemon_permits(&machine));
    assert_eq!(
        what_the_agent_reads(&agent),
        None,
        "the agent was told something"
    );
}

/// **A grant that could not be written is told as not granted**, never as
/// granted: the call would only be refused again.
#[test]
fn a_grant_that_could_not_be_written_is_told_as_not_granted() {
    let (_runtime, machine, mut server, which, shell, agent) = a_paused_turn("paused-unwritable");
    // **The grants file is not a file**, so it will not read or take a write.
    let grants = machine.places.grants().to_path_buf();
    let _ = std::fs::remove_file(&grants);
    std::fs::create_dir_all(&grants).unwrap();
    server.a_call_for_an_application_was_refused(which, shell, &[CODIUM.to_owned()]);
    server.the_application_sheet_key(ApplicationSheetKey::Grant);
    server.the_paused_turns_are_told();
    assert_eq!(
        what_the_agent_reads(&agent).as_deref(),
        Some("alo-not-granted:\n")
    );
}

/// **Nobody answering is told as not granted**, once the sheet's time is up.
#[test]
fn silence_is_told_as_not_granted() {
    let (_runtime, machine, mut server, which, shell, agent) = a_paused_turn("paused-silence");
    server.a_call_for_an_application_was_refused(which, shell, &[CODIUM.to_owned()]);
    server.the_application_sheet_waited(
        std::time::Instant::now() + crate::the_application_sheet::THE_LONGEST,
    );
    server.the_paused_turns_are_told();
    assert_eq!(
        what_the_agent_reads(&agent).as_deref(),
        Some("alo-not-granted:\n")
    );
    assert!(!the_daemon_permits(&machine));
}
