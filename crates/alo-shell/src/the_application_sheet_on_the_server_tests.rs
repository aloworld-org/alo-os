//! The sheet as the server holds it, measured on the grants file the daemon
//! reads: what was wanted goes up, **only a press grants**, *Not now* changes
//! nothing, a *This task* grant goes the instant its task ends, and a session
//! grant goes at sign-out and at the next sign-in after a power cut — the four
//! refusal tests the owner asked for on 2026-11-16.
//!
//! Every grant is checked with the daemon's own question, `Grants::permits`,
//! asked of the file as read back, never of a value in memory.

#![cfg(target_os = "linux")]

use std::os::unix::fs::PermissionsExt as _;
use std::time::{Instant, SystemTime};

use alo_capability::{Ask, Grantee};
use alo_models::InferenceSource;
use alo_tasks::ATask;
use alo_tasks::conversation::Turn;

use crate::the_application_sheet::{
    AnApplicationWanted, ApplicationSheetKey, HowLongForAnApplication, OnTheApplicationSheet,
    THE_LONGEST, TheSheetAnswered,
};
use crate::the_tasks_a_person_opened::WhichTask;

/// VSCodium's identifier.
const CODIUM: &str = "com.vscodium.codium";

/// A person's session: a server told where this person's grants are, and a
/// task alo is working in.
struct ASession {
    /// The runtime directory the server is bound in.
    _runtime: tempfile::TempDir,
    /// The person's machine: their grants file, and a daemon's socket.
    machine: crate::settings_testing::Machine,
    /// The server.
    server: crate::Server,
    /// The task alo is working in.
    task: WhichTask,
}

/// A session on `machine`, beginning — which is a sign-in.
fn a_session_on(machine: crate::settings_testing::Machine, named: &str) -> ASession {
    let runtime = tempfile::tempdir().unwrap();
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut server = crate::Server::bind(runtime.path(), named).unwrap();
    server.the_settings_places_are(machine.places.clone());
    let task = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "open my project in VSCodium",
        InferenceSource::ThisMachine,
    ));
    ASession {
        _runtime: runtime,
        machine,
        server,
        task,
    }
}

/// A fresh session on a fresh machine.
fn a_session(named: &str) -> ASession {
    a_session_on(crate::settings_testing::a_persons_machine(named), named)
}

impl ASession {
    /// This session's task wanted VSCodium.
    fn codium_is_wanted(&mut self) {
        self.server
            .applications_were_wanted(self.task, &[CODIUM.to_owned()], Instant::now());
    }

    /// Whether the daemon, reading the file now, lets alo use `identifier`.
    fn the_daemon_permits(&self, identifier: &str) -> bool {
        alo_remembering::remembered(self.machine.places.grants(), SystemTime::now())
            .unwrap()
            .permits(
                &Grantee::named("alo"),
                &Ask::application(identifier),
                SystemTime::now(),
            )
    }

    /// The grants file, byte for byte.
    fn the_file(&self) -> Option<Vec<u8>> {
        crate::settings_testing::bytes(self.machine.places.grants())
    }

    /// VSCodium wanted, and granted for `how_long` by the person's keys.
    fn granted_for(&mut self, how_long: HowLongForAnApplication) {
        self.codium_is_wanted();
        self.server
            .the_application_sheet_pressed(OnTheApplicationSheet::For(how_long));
        self.server
            .the_application_sheet_key(ApplicationSheetKey::Grant);
        assert!(
            self.the_daemon_permits(CODIUM),
            "the control: it was granted"
        );
    }

    /// The task's work is over, so its window may close without asking.
    fn the_work_is_over(&mut self) {
        self.server
            .tasks_mut()
            .the_mut(self.task)
            .unwrap()
            .alo_says(Turn::from_alo("It is open."));
    }
}

/// **What was wanted goes up, and nothing is granted until somebody
/// answers**: wanting is not granting.
#[test]
fn what_was_wanted_goes_up_and_nothing_is_granted_yet() {
    let mut session = a_session("application-sheet-up");
    session.codium_is_wanted();
    let shows = session.server.application_sheet.shows().unwrap();
    assert_eq!(
        shows.wanted,
        &AnApplicationWanted::by(session.task, CODIUM, CODIUM)
    );
    assert!(!session.the_daemon_permits(CODIUM));
    assert!(
        session
            .server
            .take_what_the_application_sheet_answered()
            .is_empty()
    );
}

/// **A key grants the length the keyboard is on, once**, on the disk; the
/// answer is kept for the turn and taken once; a second Enter grants nothing.
#[test]
fn a_key_grants_once_on_the_disk() {
    let mut session = a_session("application-sheet-key");
    session.codium_is_wanted();
    session
        .server
        .the_application_sheet_key(ApplicationSheetKey::Last);
    session
        .server
        .the_application_sheet_key(ApplicationSheetKey::Grant);
    assert!(session.the_daemon_permits(CODIUM));
    let answered = session.server.take_what_the_application_sheet_answered();
    assert!(
        matches!(
            answered.as_slice(),
            [TheSheetAnswered::Granted {
                wanted,
                how_long: HowLongForAnApplication::Remember,
                ..
            }] if *wanted == AnApplicationWanted::by(session.task, CODIUM, CODIUM)
        ),
        "{answered:?}"
    );
    let after = session.the_file();
    session
        .server
        .the_application_sheet_key(ApplicationSheetKey::Grant);
    assert_eq!(session.the_file(), after, "a second Enter changed the file");
    assert!(
        session
            .server
            .take_what_the_application_sheet_answered()
            .is_empty()
    );
}

/// **Not now grants nothing**: the file is byte for byte as it was, the
/// daemon is not knocked on, and the answer kept for the turn is *not now* —
/// so the turn goes on with the refusal it already has, which the daemon
/// recorded when it refused.
#[test]
fn not_now_grants_nothing_and_tells_the_daemon_nothing() {
    let mut session = a_session("application-sheet-not-now");
    let daemon = std::os::unix::net::UnixListener::bind(session.machine.places.daemon()).unwrap();
    daemon.set_nonblocking(true).unwrap();
    let before = session.the_file();
    session.codium_is_wanted();
    session
        .server
        .the_application_sheet_pressed(OnTheApplicationSheet::NotNow);
    assert_eq!(session.the_file(), before);
    assert!(!session.the_daemon_permits(CODIUM));
    assert!(daemon.accept().is_err(), "the daemon was knocked on");
    assert_eq!(
        session.server.take_what_the_application_sheet_answered(),
        [TheSheetAnswered::NotNow {
            wanted: AnApplicationWanted::by(session.task, CODIUM, CODIUM)
        }]
    );
}

/// **And Escape is *Not now***, by the same measure.
#[test]
fn escape_grants_nothing() {
    let mut session = a_session("application-sheet-escape");
    let daemon = std::os::unix::net::UnixListener::bind(session.machine.places.daemon()).unwrap();
    daemon.set_nonblocking(true).unwrap();
    let before = session.the_file();
    session.codium_is_wanted();
    session
        .server
        .the_application_sheet_key(ApplicationSheetKey::NotNow);
    assert_eq!(session.the_file(), before);
    assert!(!session.the_daemon_permits(CODIUM));
    assert!(daemon.accept().is_err(), "the daemon was knocked on");
}

/// **A grant the person pressed for, and the daemon is knocked on**, after
/// the file says so — the control for the test above.
#[test]
fn a_grant_knocks_on_the_daemon() {
    let mut session = a_session("application-sheet-knocks");
    let daemon = std::os::unix::net::UnixListener::bind(session.machine.places.daemon()).unwrap();
    // **A daemon that hears the knock and hangs up**, so the knock is not
    // left waiting for an answer.
    let heard = std::thread::spawn(move || daemon.accept().is_ok());
    session.granted_for(HowLongForAnApplication::Remember);
    assert!(heard.join().unwrap(), "the daemon was not told");
}

/// **Silence and a sheet nobody can see grant nothing**: the two answers no
/// person gave leave the file as it was.
#[test]
fn silence_and_a_sheet_nobody_can_see_grant_nothing() {
    let mut session = a_session("application-sheet-silence");
    let daemon = std::os::unix::net::UnixListener::bind(session.machine.places.daemon()).unwrap();
    daemon.set_nonblocking(true).unwrap();
    let before = session.the_file();
    let now = Instant::now();
    session.server.applications_were_wanted(
        session.task,
        &[CODIUM.to_owned(), "org.mozilla.firefox".to_owned()],
        now,
    );
    session
        .server
        .the_application_sheet_waited(now + THE_LONGEST);
    session.server.the_application_sheet_could_not_be_drawn();
    assert_eq!(session.the_file(), before);
    assert!(daemon.accept().is_err(), "the daemon was knocked on");
    let answered = session.server.take_what_the_application_sheet_answered();
    assert_eq!(answered.len(), 2);
    assert!(
        answered
            .iter()
            .all(|it| matches!(it, TheSheetAnswered::NotNow { .. })),
        "{answered:?}"
    );
}

/// **A reader's press grants as a click does, and only on its own sheet.**
#[test]
fn a_readers_press_is_held_to_its_sheet() {
    let mut session = a_session("application-sheet-reader");
    session.codium_is_wanted();
    let number = session.server.application_sheet.shows().unwrap().number;
    assert!(
        !session.server.a_reader_pressed_on_the_application_sheet(
            number + 1,
            OnTheApplicationSheet::LetAloUseIt
        )
    );
    assert!(!session.the_daemon_permits(CODIUM));
    assert!(
        session
            .server
            .a_reader_pressed_on_the_application_sheet(number, OnTheApplicationSheet::LetAloUseIt)
    );
    assert!(session.the_daemon_permits(CODIUM));
}

/// **A press with nowhere to write grants nothing, and is kept as *not now***:
/// a session never told where its grants are makes no grant, so the turn is
/// not sent back to be refused again.
#[test]
fn a_press_with_nowhere_to_write_is_kept_as_not_now() {
    let runtime = tempfile::tempdir().unwrap();
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut server = crate::Server::bind(runtime.path(), "application-sheet-nowhere").unwrap();
    let task = WhichTask::numbered(0);
    server.applications_were_wanted(task, &[CODIUM.to_owned()], Instant::now());
    server.the_application_sheet_key(ApplicationSheetKey::Grant);
    assert_eq!(
        server.take_what_the_application_sheet_answered(),
        [TheSheetAnswered::NotNow {
            wanted: AnApplicationWanted::by(task, CODIUM, CODIUM)
        }]
    );
}

/// **A *This task* grant is gone the instant the task's window closes** —
/// by the daemon's own question on the file — and the Settings row for it
/// said *until this task ends* while it lasted.
#[test]
fn a_this_task_grant_is_gone_the_instant_its_window_closes() {
    let mut session = a_session("application-sheet-task-closes");
    session.server.settings = session.machine.opened();
    session.granted_for(HowLongForAnApplication::ThisTask);
    assert_eq!(session.server.grants_for_a_task.len(), 1);
    assert_eq!(
        the_notes_in_settings(&session),
        [crate::settings_type::as_a_name("until this task ends")],
        "Settings did not say the grant ends with its task"
    );
    session.the_work_is_over();
    assert_eq!(
        session.server.the_person_closes_a_task(session.task),
        crate::closing_a_task_window::WhatClosingCameTo::Closed
    );
    assert!(!session.the_daemon_permits(CODIUM));
    assert!(session.server.grants_for_a_task.is_empty());
    assert!(
        the_notes_in_settings(&session).is_empty(),
        "Settings still lists the grant"
    );
}

/// **A *This session* grant says *until you sign out* in Settings.**
#[test]
fn a_session_grant_says_until_you_sign_out_in_settings() {
    let mut session = a_session("application-sheet-session-row");
    session.server.settings = session.machine.opened();
    session.granted_for(HowLongForAnApplication::ThisSession);
    assert_eq!(
        the_notes_in_settings(&session),
        [crate::settings_type::as_a_name("until you sign out")]
    );
}

/// **A press heard on a sheet whose task has since ended grants nothing**: the
/// sheet came down with the task, and a reader's press for it lands on
/// nothing.
#[test]
fn a_press_after_its_task_ended_grants_nothing() {
    let mut session = a_session("application-sheet-late-press");
    session.codium_is_wanted();
    let number = session.server.application_sheet.shows().unwrap().number;
    assert!(session.server.the_person_stops_a_task(session.task));
    let before = session.the_file();
    assert!(
        !session
            .server
            .a_reader_pressed_on_the_application_sheet(number, OnTheApplicationSheet::LetAloUseIt)
    );
    session
        .server
        .the_application_sheet_key(ApplicationSheetKey::Grant);
    assert_eq!(session.the_file(), before);
    assert!(!session.the_daemon_permits(CODIUM));
}

/// The notes beside alo's application grants on Settings' folders page, as
/// drawn — the words a person reads about how long each lasts.
fn the_notes_in_settings(session: &ASession) -> Vec<String> {
    let strings = crate::settings_testing::settings_words();
    let page = crate::settings_page::the_page(
        session.server.settings.open().unwrap(),
        crate::SettingsCategory::FoldersAloCanUse,
        &strings,
        SystemTime::now(),
    );
    page.blocks
        .iter()
        .flat_map(|block| match block {
            crate::settings_page::ABlock::Section(section) => section.rows.iter().collect(),
            _ => Vec::new(),
        })
        .filter(|row| row.title == CODIUM || row.title == "VSCodium")
        .filter_map(|row| row.note.as_ref().map(|note| note.words.clone()))
        .collect()
}

/// **A *This task* grant is gone the instant the task is stopped.**
#[test]
fn a_this_task_grant_is_gone_the_instant_its_task_is_stopped() {
    let mut session = a_session("application-sheet-task-stops");
    session.granted_for(HowLongForAnApplication::ThisTask);
    assert!(session.server.the_person_stops_a_task(session.task));
    assert!(!session.the_daemon_permits(CODIUM));
}

/// **Another task ending takes nothing**: only the task that asked ends its
/// grant, and a *This session* grant outlives every task.
#[test]
fn another_task_ending_takes_nothing_and_a_session_grant_outlives_tasks() {
    let mut session = a_session("application-sheet-other-task");
    session.granted_for(HowLongForAnApplication::ThisTask);
    let other = session
        .server
        .tasks_mut()
        .and_now_this_one(ATask::asked_for(
            "something else",
            InferenceSource::ThisMachine,
        ));
    assert!(session.server.the_person_stops_a_task(other));
    assert!(session.the_daemon_permits(CODIUM), "another task took it");

    let mut lasting = a_session("application-sheet-session-outlives");
    lasting.granted_for(HowLongForAnApplication::ThisSession);
    assert!(lasting.server.the_person_stops_a_task(lasting.task));
    assert!(
        lasting.the_daemon_permits(CODIUM),
        "a session grant went with a task"
    );
}

/// **A *This session* grant is gone after sign-out**, and a *Remember* grant
/// beside it is not.
#[test]
fn a_session_grant_is_gone_after_sign_out() {
    let mut session = a_session("application-sheet-sign-out");
    session.granted_for(HowLongForAnApplication::ThisSession);
    session.server.applications_were_wanted(
        session.task,
        &["org.mozilla.firefox".to_owned()],
        Instant::now(),
    );
    session
        .server
        .the_application_sheet_pressed(OnTheApplicationSheet::For(
            HowLongForAnApplication::Remember,
        ));
    session
        .server
        .the_application_sheet_key(ApplicationSheetKey::Grant);
    assert!(session.the_daemon_permits("org.mozilla.firefox"));
    // **The desktop signing out**, as `alo-desktop` does once its display
    // ends without a fault.
    assert_eq!(
        crate::the_grants_for_the_session_go(&session.machine.places),
        1
    );
    assert!(!session.the_daemon_permits(CODIUM));
    assert!(session.the_daemon_permits("org.mozilla.firefox"));
}

/// **After a power cut, the next sign-in takes a session grant — and a task
/// grant whose task never ended — away**, before anything is asked of the new
/// session.
#[test]
fn after_a_power_cut_the_next_sign_in_takes_them_away() {
    let mut session = a_session("application-sheet-power-cut");
    session.granted_for(HowLongForAnApplication::ThisSession);
    session.server.applications_were_wanted(
        session.task,
        &["org.mozilla.firefox".to_owned()],
        Instant::now(),
    );
    session
        .server
        .the_application_sheet_key(ApplicationSheetKey::Grant);
    assert!(session.the_daemon_permits("org.mozilla.firefox"));
    // **And one to remember**, which a sign-in must leave alone.
    session.server.applications_were_wanted(
        session.task,
        &["org.gnome.Calculator".to_owned()],
        Instant::now(),
    );
    session
        .server
        .the_application_sheet_key(ApplicationSheetKey::Last);
    session
        .server
        .the_application_sheet_key(ApplicationSheetKey::Grant);
    assert!(session.the_daemon_permits("org.gnome.Calculator"));
    // **The power goes**: nothing signs out, nothing ends a task. The file
    // still holds both.
    let ASession { machine, .. } = session;
    let after_the_power_cut = a_session_on(machine, "application-sheet-power-cut-2");
    assert!(!after_the_power_cut.the_daemon_permits(CODIUM));
    assert!(!after_the_power_cut.the_daemon_permits("org.mozilla.firefox"));
    assert!(
        after_the_power_cut.the_daemon_permits("org.gnome.Calculator"),
        "a sign-in took away a grant the person asked to be remembered"
    );
}
