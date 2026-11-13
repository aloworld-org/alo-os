//! **Where no verb fits, the person's level decides — and every run and every
//! refusal is written down** (ADR 0149).
//!
//! These run real commands, through `alo_running::run_as_the_person`, into
//! folders of each test's own that no grant names — so *a command reaches a
//! file outside every grant* is a thing that happened on the disk, not a
//! statement about a stand-in. Every folder is this test's own and is removed
//! afterwards; nothing assumes it is alone on the machine.

use std::path::{Path, PathBuf};

use alo_capability::HowCodeRuns;
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_protocol::{Done, FromAPerson, ToAPerson};
use alo_record::{Happened, Record};
use alo_turn::Machine;

use super::*;
use crate::answering::what_a_person_said;
use crate::doing::what_an_agent_said;
use crate::holding::Holding;
use crate::network::TheNetwork;
use crate::pairing::Nearby;
use crate::rereading::WhatIsGranted;
use crate::testing::{
    NobodyIsNearby, NothingIsBounded, NothingIsRemembered, a_directory_of_our_own,
    a_folder_with_an_invoice, a_message, granting, hour, in_english, noon, nothing_has_been_chosen,
    the_studio,
};

/// A folder of this test's own that no grant names, and where its level file
/// is kept.
struct Elsewhere {
    /// A folder the turn's grants do not reach.
    folder: PathBuf,
    /// Where the person's level is kept, for this test.
    level_at: PathBuf,
}

impl Elsewhere {
    /// A fresh one, with the level as given — or no file at all.
    fn with_the_level(what: &str, level: Option<HowCodeRuns>) -> Self {
        let folder = a_directory_of_our_own(what);
        let level_at = a_directory_of_our_own(&format!("{what}-level")).join("how-code-runs.toml");
        if let Some(level) = level {
            alo_remembering::the_level_kept(&level_at, level).unwrap();
        }
        Self { folder, level_at }
    }

    /// Where commands run and the level is read, for this test.
    fn here(&self) -> CommandsHere {
        CommandsHere::at(&self.level_at, &self.folder)
    }

    /// The folder a command is asked to make, which nothing granted.
    fn the_folder_it_makes(&self) -> PathBuf {
        self.folder.join("demo")
    }

    /// The line an agent sends to make it.
    fn asking_to_make_it(&self) -> String {
        a_message(&format!(
            r#"{{"run":{{"command":"mkdir -p {}"}}}}"#,
            self.the_folder_it_makes().display()
        ))
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

/// A turn whose grants reach one folder of its own, on a machine whose record
/// is `record`.
fn on_a_turn<T>(
    what: &str,
    record: &mut Record,
    doing: impl FnOnce(&mut Turning<'_, '_>, &mut alo_capability::Grants, &Strings, &Path) -> T,
) -> T {
    on_a_watched_turn(
        what,
        record,
        &mut NothingIsBounded,
        Indicator::default(),
        doing,
    )
}

/// The same, on a machine whose boundary is `bounding` and whose indicator is
/// `indicator` — for a test that watches what the indicator was told.
fn on_a_watched_turn<T>(
    what: &str,
    record: &mut Record,
    bounding: &mut dyn alo_turn::Bounding,
    mut indicator: Indicator,
    doing: impl FnOnce(&mut Turning<'_, '_>, &mut alo_capability::Grants, &Strings, &Path) -> T,
) -> T {
    let strings = in_english();
    let (granted, _) = a_folder_with_an_invoice(what);
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        bounding,
        &mut indicator,
        record,
    )
    .unwrap();
    let mut grants = granting(&granted, noon());
    let mut turning = Turning::beginning(
        alo_context::Context::at_invocation(noon()),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    let done = doing(&mut turning, &mut grants, &strings, &granted);
    let _ = std::fs::remove_dir_all(&granted);
    done
}

/// What an agent's line is answered with, on this turn and this level.
fn the_agent_asked(
    line: &str,
    turning: &mut Turning<'_, '_>,
    grants: &alo_capability::Grants,
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

/// What the person's door answers this line with, on this turn and level.
fn the_person_said(
    line: &str,
    turning: &mut Turning<'_, '_>,
    grants: &mut alo_capability::Grants,
    strings: &Strings,
    here: &CommandsHere,
    bracket: &mut TheTurnsBracket,
) -> ToAPerson {
    let network = TheNetwork::on(the_studio());
    what_a_person_said(
        line,
        &mut Holding::ATurn {
            turning,
            questions: &mut nothing_has_been_chosen(),
            bracket,
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
/// for its snapshot before every run.
fn a_bracket() -> TheTurnsBracket {
    TheTurnsBracket::none_yet(None, std::time::SystemTime::UNIX_EPOCH, 0, 0)
}

/// Every entry the record holds, by what happened.
fn what_was_written(record: &Record) -> Vec<Happened> {
    record
        .everything()
        .map(|entry| entry.happened().clone())
        .collect()
}

/// The plain sentence for a command the sealed box does not run (#1093).
const NOT_IN_THE_BOX: &str = "mkdir was not run. The sealed box can't run programs on this computer yet. Choose 'ask \
     each time' in Settings → Running code to run commands after you approve them.";

/// **At full access a command runs now, without any grant, and reaches a
/// folder outside every grant** — after the snapshot was asked for, and
/// written down as a run with no grants and no box, and the person told it
/// was done without asking.
#[test]
fn at_full_access_a_command_runs_and_reaches_what_no_grant_names() {
    let elsewhere = Elsewhere::with_the_level("full-access", Some(HowCodeRuns::AtFullTrust));
    let mut record = Record::default();
    let mut bracket = a_bracket();
    on_a_turn(
        "full-access",
        &mut record,
        |turning, grants, strings, granted| {
            assert!(!elsewhere.folder.starts_with(granted));
            let said = the_agent_asked(
                &elsewhere.asking_to_make_it(),
                turning,
                grants,
                strings,
                &elsewhere.here(),
                &mut bracket,
            );
            assert!(
                matches!(
                    said.done(),
                    Some(Done::RanACommand {
                        returned: Some(0),
                        ..
                    })
                ),
                "{said:?}"
            );
            // **The person is told, in the machine's own words**, that it was
            // done without asking and why — and, since this bracket keeps
            // nothing, that it cannot be put back. Until 2026-11-09 a command
            // was answered `did`, and the person read only the model's own
            // sentence about it.
            let told = match &said {
                ToAnAgent::DoneWithoutAsking { said, .. } => Some(said.text().to_owned()),
                _ => None,
            };
            assert_eq!(
                told,
                Some(format!(
                    "Done without asking you first, because you chose full access in Settings → \
                     Running code: run this command: mkdir -p {}. This one can't be put back: a \
                     copy of your files from before it could not be kept.",
                    elsewhere.the_folder_it_makes().display()
                )),
                "a command run at full access was not answered as done without asking: {said:?}"
            );
        },
    );
    assert!(
        elsewhere.the_folder_it_makes().is_dir(),
        "the command did not reach a folder no grant names"
    );
    assert_eq!(
        bracket.the_first_sentence_for_tests(),
        Some(
            format!(
                "run this command: mkdir -p {}",
                elsewhere.the_folder_it_makes().display()
            )
            .as_str()
        ),
        "the snapshot was not asked for before the run"
    );
    let written = what_was_written(&record);
    assert!(
        matches!(
            written.as_slice(),
            [Happened::RanACommand { allowed, the_kernel_was_holding_it: false, returned: Some(0), .. }]
                if allowed.is_empty()
        ),
        "{written:?}"
    );
}

/// **At full access a command that fails is never answered *Done***: the
/// person is told what was tried, that it did not work and what it ended
/// with — and the record keeps the status it ended with.
///
/// The boot walk of `f028d5ec`: *make a folder called demo in my Documents*
/// ran `mkdir /Users/username/Documents/demo`, which ended with status 1, and
/// the machine's own line said *Done without asking you first* above the
/// model's *the directory does not exist*.
#[test]
fn at_full_access_a_command_that_fails_is_not_answered_done() {
    let elsewhere = Elsewhere::with_the_level("full-access-fails", Some(HowCodeRuns::AtFullTrust));
    let nowhere = elsewhere.folder.join("not-there").join("demo");
    let line = a_message(&format!(
        r#"{{"run":{{"command":"mkdir {}"}}}}"#,
        nowhere.display()
    ));
    let mut record = Record::default();
    let mut bracket = a_bracket();
    on_a_turn(
        "full-access-fails",
        &mut record,
        |turning, grants, strings, _| {
            let said = the_agent_asked(
                &line,
                turning,
                grants,
                strings,
                &elsewhere.here(),
                &mut bracket,
            );
            // **The model is told the run whole**, status and all.
            assert!(
                matches!(
                    said.done(),
                    Some(Done::RanACommand {
                        returned: Some(1),
                        ..
                    })
                ),
                "{said:?}"
            );
            let told = match &said {
                ToAnAgent::DoneWithoutAsking { said, .. } => Some(said.text().to_owned()),
                _ => None,
            };
            assert_eq!(
                told,
                Some(format!(
                    "Tried without asking you first, because you chose full access in Settings → \
                 Running code: run this command: mkdir {}. It did not work: the command ended \
                 with exit status 1. If it changed anything, that can't be put back: a copy of \
                 your files from before it could not be kept.",
                    nowhere.display()
                )),
                "a command that failed at full access was not told as one that did not work: {said:?}"
            );
        },
    );
    assert!(!nowhere.exists(), "the command made the folder after all");
    let written = what_was_written(&record);
    assert!(
        matches!(
            written.as_slice(),
            [Happened::RanACommand {
                returned: Some(1),
                ..
            }]
        ),
        "the record does not keep the status the command failed with: {written:?}"
    );
}

/// **A push nobody had signed in for is told as that, with how to sign in** —
/// never as a bare exit status — and the record still keeps the status.
///
/// The command prints what `git` prints when it has an `https` remote on
/// GitHub and nobody to type a name, which is what alo's own runs now get
/// (`alo_running::nobody_is_there_to_answer`); nothing here reaches a network.
#[test]
fn at_full_access_a_push_nobody_signed_in_for_says_how_to_sign_in() {
    let elsewhere = Elsewhere::with_the_level("not-signed-in", Some(HowCodeRuns::AtFullTrust));
    let line = a_message(
        r#"{"run":{"command":"echo 'fatal: could not read Username for https://github.com: terminal prompts disabled' >&2; exit 128"}}"#,
    );
    let mut record = Record::default();
    let mut bracket = a_bracket();
    on_a_turn(
        "not-signed-in",
        &mut record,
        |turning, grants, strings, _| {
            let said = the_agent_asked(
                &line,
                turning,
                grants,
                strings,
                &elsewhere.here(),
                &mut bracket,
            );
            let told = match &said {
                ToAnAgent::DoneWithoutAsking { said, .. } => said.text().to_owned(),
                _ => String::new(),
            };
            assert!(
                told.contains("It did not work: you are not signed in to GitHub.")
                    && told.contains("gh auth login --web"),
                "a push nobody signed in for was not told as that: {said:?}"
            );
            assert!(!told.contains("exit status"), "{told:?}");
        },
    );
    let written = what_was_written(&record);
    assert!(
        matches!(
            written.as_slice(),
            [Happened::RanACommand {
                returned: Some(128),
                ..
            }]
        ),
        "{written:?}"
    );
}

/// **At *ask me each time* nothing runs before the OK; one OK runs it exactly
/// once; the same number again is refused** — on the person's own door, with
/// the command in the list of what waits.
#[test]
fn asking_each_time_runs_a_command_once_after_one_ok() {
    let elsewhere = Elsewhere::with_the_level("asking", Some(HowCodeRuns::AskingEachTime));
    let counted = elsewhere.folder.join("ran");
    let line = a_message(&format!(
        r#"{{"run":{{"command":"echo once >> {}"}}}}"#,
        counted.display()
    ));
    let mut record = Record::default();
    let mut bracket = a_bracket();
    on_a_turn("asking", &mut record, |turning, grants, strings, _| {
        let here = elsewhere.here();
        let said = the_agent_asked(&line, turning, grants, strings, &here, &mut bracket);
        assert!(said.waits_for_a_person(), "{said:?}");
        assert!(!counted.exists(), "it ran before anybody said yes");
        let standing = match said {
            ToAnAgent::Proposed(standing) => standing,
            other => unreachable!("not waiting: {other:?}"),
        };
        assert_eq!(
            standing.sentence().text(),
            format!("run this command: echo once >> {}", counted.display())
        );

        let waiting = the_person_said(
            &a_message(r#"{"waiting":{}}"#),
            turning,
            grants,
            strings,
            &here,
            &mut bracket,
        );
        assert!(
            waiting.changes().is_some_and(|changes| changes
                .iter()
                .any(|change| change.number() == standing.number()
                    && change.sentence() == standing.sentence())),
            "the person is not shown the command waiting: {waiting:?}"
        );

        let approve = a_message(&format!(
            r#"{{"approve":{{"number":{}}}}}"#,
            standing.number()
        ));
        let first = the_person_said(&approve, turning, grants, strings, &here, &mut bracket);
        assert!(first.done().is_some(), "{first:?}");
        assert!(
            first.cannot_be_undone(),
            "a session with no home kept no place, and the person must be told"
        );
        let again = the_person_said(&approve, turning, grants, strings, &here, &mut bracket);
        assert!(again.refusal().is_some(), "{again:?}");
    });
    assert_eq!(
        std::fs::read_to_string(&counted).unwrap(),
        "once\n",
        "one approval did not run it exactly once"
    );
    assert!(bracket.the_first_sentence_for_tests().is_some());
    let written = what_was_written(&record);
    assert!(
        matches!(written.as_slice(), [Happened::RanACommand { .. }]),
        "{written:?}"
    );
}

/// **A no is written down, and nothing runs.**
#[test]
fn a_declined_command_is_written_down_and_never_runs() {
    let elsewhere = Elsewhere::with_the_level("declined", Some(HowCodeRuns::AskingEachTime));
    let mut record = Record::default();
    let mut bracket = a_bracket();
    on_a_turn("declined", &mut record, |turning, grants, strings, _| {
        let here = elsewhere.here();
        let said = the_agent_asked(
            &elsewhere.asking_to_make_it(),
            turning,
            grants,
            strings,
            &here,
            &mut bracket,
        );
        let standing = match said {
            ToAnAgent::Proposed(standing) => standing,
            other => unreachable!("not waiting: {other:?}"),
        };
        let no = the_person_said(
            &a_message(&format!(
                r#"{{"decline":{{"number":{}}}}}"#,
                standing.number()
            )),
            turning,
            grants,
            strings,
            &here,
            &mut bracket,
        );
        assert_eq!(no, ToAPerson::Declined);
        let yes_after = the_person_said(
            &a_message(&format!(
                r#"{{"approve":{{"number":{}}}}}"#,
                standing.number()
            )),
            turning,
            grants,
            strings,
            &here,
            &mut bracket,
        );
        assert!(yes_after.refusal().is_some(), "{yes_after:?}");
    });
    assert!(
        !elsewhere.the_folder_it_makes().exists(),
        "a declined command ran"
    );
    let written = what_was_written(&record);
    assert!(
        matches!(written.as_slice(), [Happened::ACommandWasNotRun { .. }]),
        "{written:?}"
    );
}

/// **The sealed box, nothing chosen, and a file that will not read all refuse
/// in the plain sentence, write the refusal down, and run nothing.**
#[test]
fn the_sealed_box_nothing_chosen_and_an_unreadable_level_refuse_and_run_nothing() {
    let sealed = Elsewhere::with_the_level("sealed", Some(HowCodeRuns::InASealedBox));
    let not_chosen = Elsewhere::with_the_level("not-chosen", None);
    let unreadable = Elsewhere::with_the_level("unreadable", None);
    std::fs::write(&unreadable.level_at, "format = 1\nchose = \"full-trust\"\n").unwrap();
    assert!(alo_remembering::the_level_remembered(&unreadable.level_at).is_err());

    for elsewhere in [&sealed, &not_chosen, &unreadable] {
        let mut record = Record::default();
        let mut bracket = a_bracket();
        on_a_turn("refused", &mut record, |turning, grants, strings, _| {
            let said = the_agent_asked(
                &elsewhere.asking_to_make_it(),
                turning,
                grants,
                strings,
                &elsewhere.here(),
                &mut bracket,
            );
            assert_eq!(
                said.refusal().map(alo_protocol::Wording::text),
                Some(NOT_IN_THE_BOX),
                "{:?}",
                elsewhere.level_at
            );
            assert_eq!(turning.commands_waiting_at(noon()).count(), 0);
        });
        assert!(
            !elsewhere.the_folder_it_makes().exists(),
            "{:?} ran a command",
            elsewhere.level_at
        );
        assert!(
            bracket.the_first_sentence_for_tests().is_none(),
            "a snapshot was taken for a command that did not run"
        );
        let written = what_was_written(&record);
        assert!(
            matches!(
                written.as_slice(),
                [Happened::ACommandWasNotRun { why, .. }] if why.is(NOT_IN_THE_BOX)
            ),
            "{written:?}"
        );
    }
}

/// **A level taken back while the question stood is the person's no**: moved
/// to the sealed box before saying yes, the command is refused, written down,
/// and does not run.
#[test]
fn a_command_approved_after_the_level_was_taken_back_does_not_run() {
    let elsewhere = Elsewhere::with_the_level("taken-back", Some(HowCodeRuns::AskingEachTime));
    let mut record = Record::default();
    let mut bracket = a_bracket();
    on_a_turn("taken-back", &mut record, |turning, grants, strings, _| {
        let here = elsewhere.here();
        let said = the_agent_asked(
            &elsewhere.asking_to_make_it(),
            turning,
            grants,
            strings,
            &here,
            &mut bracket,
        );
        let standing = match said {
            ToAnAgent::Proposed(standing) => standing,
            other => unreachable!("not waiting: {other:?}"),
        };
        alo_remembering::the_level_kept(&elsewhere.level_at, HowCodeRuns::InASealedBox).unwrap();
        let yes = the_person_said(
            &a_message(&format!(
                r#"{{"approve":{{"number":{}}}}}"#,
                standing.number()
            )),
            turning,
            grants,
            strings,
            &here,
            &mut bracket,
        );
        assert_eq!(
            yes.refusal().map(alo_protocol::Wording::text),
            Some(NOT_IN_THE_BOX)
        );
        assert_eq!(turning.commands_waiting_at(noon()).count(), 0);
    });
    assert!(!elsewhere.the_folder_it_makes().exists());
    assert!(bracket.the_first_sentence_for_tests().is_none());
    let written = what_was_written(&record);
    assert!(
        matches!(written.as_slice(), [Happened::ACommandWasNotRun { why, .. }] if why.is(NOT_IN_THE_BOX)),
        "{written:?}"
    );
}

/// **A line that is not a command is refused and written down**, at every
/// level — even full access runs nothing that is not one line.
#[test]
fn a_line_that_is_not_a_command_is_refused_and_written_down() {
    let elsewhere = Elsewhere::with_the_level("not-a-command", Some(HowCodeRuns::AtFullTrust));
    let mut record = Record::default();
    let mut bracket = a_bracket();
    on_a_turn(
        "not-a-command",
        &mut record,
        |turning, grants, strings, _| {
            let said = the_agent_asked(
                &a_message(r#"{"run":{"command":"   "}}"#),
                turning,
                grants,
                strings,
                &elsewhere.here(),
                &mut bracket,
            );
            assert_eq!(
                said.refusal().map(alo_protocol::Wording::text),
                Some("Nothing ran: the command was empty.")
            );
        },
    );
    assert!(bracket.the_first_sentence_for_tests().is_none());
    assert!(matches!(
        what_was_written(&record).as_slice(),
        [Happened::ACommandWasNotRun { .. }]
    ));
}

/// **Every other door's road refuses a command as the sealed box does**: a
/// caller with no person's file to read is never a higher level.
#[test]
fn where_nothing_was_chosen_a_command_is_refused() {
    let mut record = Record::default();
    on_a_turn("nowhere", &mut record, |turning, grants, strings, _| {
        let said = what_an_agent_said(
            &a_message(r#"{"run":{"command":"mkdir -p /tmp/alo-never-made"}}"#),
            turning,
            &mut nothing_has_been_chosen(),
            None,
            grants,
            strings,
            hour(),
            noon(),
            Commands::where_nothing_was_chosen(),
        );
        assert_eq!(
            said.refusal().map(alo_protocol::Wording::text),
            Some(NOT_IN_THE_BOX)
        );
    });
    assert_eq!(
        CommandsHere::where_nothing_was_chosen().the_level_now(),
        HowCodeRuns::InASealedBox
    );
    assert!(matches!(
        what_was_written(&record).as_slice(),
        [Happened::ACommandWasNotRun { .. }]
    ));
}

/// The person's door still refuses a command of its own: theirs is typed at
/// their own prompt.
#[test]
fn the_persons_door_does_not_take_a_command() {
    assert!(FromAPerson::read(&a_message(r#"{"run":{"command":"ls"}}"#)).is_err());
}

/// A runner played by the test (ADR 0150): it hears one line and answers
/// `answer`, and hands back the line it heard.
fn a_runner(
    at: &Path,
    answer: alo_administrator::answering_as_administrator::Answered,
) -> std::thread::JoinHandle<String> {
    use std::io::{BufRead, Write};
    let listener = std::os::unix::net::UnixListener::bind(at).unwrap();
    std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut heard = String::new();
        std::io::BufReader::new(stream.try_clone().unwrap())
            .read_line(&mut heard)
            .unwrap();
        (&stream).write_all(answer.as_line().as_bytes()).unwrap();
        heard
    })
}

/// The line an agent sends to run `command`.
fn asking_to_run(command: &str) -> String {
    a_message(&format!(r#"{{"run":{{"command":"{command}"}}}}"#))
}

/// **At full access, `sudo` asks the person's password and runs nothing as
/// the person** (ADR 0150): what follows the word goes to the runner, and a
/// cancelled password window is a refusal in words, written down as an
/// administrator's action the person cancelled.
#[test]
fn at_full_access_sudo_asks_and_a_cancel_runs_nothing() {
    use alo_administrator::answering_as_administrator::{Answered, NotAsAdministrator};
    let elsewhere = Elsewhere::with_the_level("sudo-cancelled", Some(HowCodeRuns::AtFullTrust));
    let runner_at = elsewhere.folder.join("runner.socket");
    let runner = a_runner(
        &runner_at,
        Answered::Refused(NotAsAdministrator::TheyCancelled),
    );
    let here = elsewhere.here().running_as_administrator_at(&runner_at);
    let mut record = Record::default();
    let mut bracket = a_bracket();
    let line = format!(
        "sudo mkdir -p {}",
        elsewhere.the_folder_it_makes().display()
    );
    on_a_turn(
        "sudo-cancelled",
        &mut record,
        |turning, grants, strings, _| {
            let said = the_agent_asked(
                &asking_to_run(&line),
                turning,
                grants,
                strings,
                &here,
                &mut bracket,
            );
            assert_eq!(
                said.refusal().map(alo_protocol::Wording::text),
                Some("nothing ran, because you closed the window that asked for your password"),
                "{said:?}"
            );
        },
    );
    assert_eq!(
        runner.join().unwrap(),
        format!("mkdir -p {}\n", elsewhere.the_folder_it_makes().display())
    );
    assert!(
        !elsewhere.the_folder_it_makes().exists(),
        "it ran as the person"
    );
    let entries: Vec<_> = record.everything().collect();
    assert_eq!(entries.len(), 1, "{entries:?}");
    let entry = entries.first().unwrap();
    assert!(matches!(
        entry.happened(),
        Happened::ACommandWasNotRun { .. }
    ));
    assert_eq!(
        entry.asked_as_an_administrator(),
        Some(alo_record::AsAnAdministrator::TheyCancelled)
    );
}

/// **The person's password given, the runner's run is the answer**, and the
/// record says it ran as administrator after they authenticated.
#[test]
fn at_full_access_sudo_runs_as_administrator_after_the_password() {
    use alo_administrator::answering_as_administrator::Answered;
    let elsewhere = Elsewhere::with_the_level("sudo-given", Some(HowCodeRuns::AtFullTrust));
    let runner_at = elsewhere.folder.join("runner.socket");
    let _runner = a_runner(
        &runner_at,
        Answered::Ran {
            returned: Some(0),
            said: "Complete!".to_owned(),
            cut_short: false,
            took_too_long: false,
        },
    );
    let here = elsewhere.here().running_as_administrator_at(&runner_at);
    let mut record = Record::default();
    let mut bracket = a_bracket();
    on_a_turn("sudo-given", &mut record, |turning, grants, strings, _| {
        let said = the_agent_asked(
            &asking_to_run("sudo dnf install -y htop"),
            turning,
            grants,
            strings,
            &here,
            &mut bracket,
        );
        assert!(
            matches!(
                said.done(),
                Some(Done::RanACommand {
                    returned: Some(0),
                    ..
                })
            ),
            "{said:?}"
        );
        // **The person was asked, for their password**, so it is not told
        // as done without asking.
        assert!(
            !matches!(said, ToAnAgent::DoneWithoutAsking { .. }),
            "{said:?}"
        );
    });
    let entries: Vec<_> = record.everything().collect();
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(
        entries.first().unwrap().asked_as_an_administrator(),
        Some(alo_record::AsAnAdministrator::TheyGaveTheirPassword)
    );
    assert_eq!(
        bracket.the_first_sentence_for_tests(),
        Some("run this command: sudo dnf install -y htop"),
        "the snapshot was not asked for before the run"
    );
}

/// **No runner at all is said, and nothing runs as the person** — a machine
/// where the administrator's road is missing refuses truthfully rather than
/// falling back.
#[test]
fn no_runner_is_said_and_nothing_runs_as_the_person() {
    let elsewhere = Elsewhere::with_the_level("sudo-no-runner", Some(HowCodeRuns::AtFullTrust));
    let mut record = Record::default();
    let mut bracket = a_bracket();
    let line = format!(
        "sudo mkdir -p {}",
        elsewhere.the_folder_it_makes().display()
    );
    on_a_turn(
        "sudo-no-runner",
        &mut record,
        |turning, grants, strings, _| {
            let said = the_agent_asked(
                &asking_to_run(&line),
                turning,
                grants,
                strings,
                &elsewhere.here(),
                &mut bracket,
            );
            assert_eq!(
                said.refusal().map(alo_protocol::Wording::text),
                Some(
                    "nothing ran, because this computer could not ask for administrator rights just now"
                ),
                "{said:?}"
            );
        },
    );
    assert!(!elsewhere.the_folder_it_makes().exists());
    let entries: Vec<_> = record.everything().collect();
    assert_eq!(
        entries.first().unwrap().asked_as_an_administrator(),
        Some(alo_record::AsAnAdministrator::CouldNotAsk)
    );
}

/// **At the sealed box `sudo` is refused as every command is**, and the runner
/// is never asked.
#[test]
fn at_the_sealed_box_sudo_is_refused_and_nothing_is_asked() {
    let elsewhere = Elsewhere::with_the_level("sudo-sealed", Some(HowCodeRuns::InASealedBox));
    let mut record = Record::default();
    let mut bracket = a_bracket();
    on_a_turn("sudo-sealed", &mut record, |turning, grants, strings, _| {
        let said = the_agent_asked(
            &asking_to_run("sudo true"),
            turning,
            grants,
            strings,
            &elsewhere.here(),
            &mut bracket,
        );
        assert!(said.refusal().is_some(), "{said:?}");
    });
    let entries: Vec<_> = record.everything().collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries.first().unwrap().asked_as_an_administrator(), None);
}

/// Every list the indicator told, as its sentences — what a desktop sent
/// `watch-what-leaves` would have drawn, one list per change.
#[derive(Debug, Clone, Default)]
struct Watched(std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>);

impl alo_egress::Telling for Watched {
    fn now_showing(&self, showing: &[alo_egress::Shown]) {
        let strings = in_english();
        if let Ok(mut lists) = self.0.lock() {
            lists.push(
                showing
                    .iter()
                    .map(|shown| shown.said(&strings).text().to_owned())
                    .collect(),
            );
        }
    }
}

impl Watched {
    /// Every list told so far.
    fn lists(&self) -> Vec<Vec<String>> {
        self.0.lock().unwrap().clone()
    }
}

/// A boundary that bounds nothing and counts every command alo runs as having
/// sent `sends` bytes from its first look — the kernel's half stood in for, so
/// the daemon's half can be watched end to end with a real command.
struct EveryCommandSends {
    /// What each count reads.
    sends: u64,
    /// How many counts were begun, and how many ended.
    begun_and_ended: std::rc::Rc<std::cell::Cell<(u32, u32)>>,
}

/// One command's count, from [`EveryCommandSends`].
struct ItSends {
    /// What it reads.
    sends: u64,
    /// Where its ending is told.
    begun_and_ended: std::rc::Rc<std::cell::Cell<(u32, u32)>>,
}

impl alo_turn::ACommandCounted for ItSends {
    fn starting(
        &mut self,
        start: &mut dyn FnMut() -> std::io::Result<std::process::Child>,
    ) -> std::io::Result<std::process::Child> {
        start()
    }

    fn so_far(&self) -> Option<u64> {
        Some(self.sends)
    }

    fn ended(self: Box<Self>) -> Option<u64> {
        let (begun, ended) = self.begun_and_ended.get();
        self.begun_and_ended.set((begun, ended + 1));
        Some(self.sends)
    }
}

impl alo_turn::Bounding for EveryCommandSends {
    fn carrying_out(
        &mut self,
        reaching: &alo_files::Reaching,
        doing: alo_turn::Doing<'_>,
    ) -> Result<alo_turn::Done, alo_turn::NoBoundary> {
        alo_turn::Bounding::carrying_out(&mut NothingIsBounded, reaching, doing)
    }

    fn a_command_counted(&mut self) -> Option<Box<dyn alo_turn::ACommandCounted + '_>> {
        let (begun, ended) = self.begun_and_ended.get();
        self.begun_and_ended.set((begun + 1, ended));
        Some(Box::new(ItSends {
            sends: self.sends,
            begun_and_ended: std::rc::Rc::clone(&self.begun_and_ended),
        }))
    }
}

/// **A command alo runs is on the indicator, and the desktop is told, while it
/// runs** — here on a machine that cannot count it, which is every machine
/// these tests run on: the line says so for the whole run, comes off when it
/// ends, and the record keeps no number, because none was counted.
#[test]
fn a_command_alo_runs_is_on_the_indicator_where_nothing_can_count_it() {
    let elsewhere = Elsewhere::with_the_level("command-uncounted", Some(HowCodeRuns::AtFullTrust));
    let mut record = Record::default();
    let mut bracket = a_bracket();
    let watched = Watched::default();
    on_a_watched_turn(
        "command-uncounted",
        &mut record,
        &mut NothingIsBounded,
        Indicator::default().told_to(Box::new(watched.clone())),
        |turning, grants, strings, _| {
            let said = the_agent_asked(
                &asking_to_run("true"),
                turning,
                grants,
                strings,
                &elsewhere.here(),
                &mut bracket,
            );
            assert!(said.refusal().is_none(), "{said:?}");
        },
    );
    assert_eq!(
        watched.lists(),
        [
            vec!["alo is running true, and this machine cannot count what it sends".to_owned()],
            Vec::new()
        ],
        "the desktop was not told of the command while it ran, or not told it ended"
    );
    let entries: Vec<_> = record.everything().collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries.first().unwrap().sent_bytes(), None);
}

/// **A command that sends is shown and its bytes are in the record**, and its
/// count is begun once and ended once; **a refusal is still written down and
/// counts nothing** — at the sealed box, nothing is begun at all.
#[test]
fn a_command_that_sends_is_shown_and_counted_and_a_refusal_counts_nothing() {
    for (level, line) in [
        (
            HowCodeRuns::AtFullTrust,
            Some("true, a command alo is running, is sending"),
        ),
        (HowCodeRuns::InASealedBox, None),
    ] {
        let elsewhere = Elsewhere::with_the_level("command-counted", Some(level));
        let mut record = Record::default();
        let mut bracket = a_bracket();
        let watched = Watched::default();
        let begun_and_ended = std::rc::Rc::new(std::cell::Cell::new((0, 0)));
        let mut bounding = EveryCommandSends {
            sends: 4180,
            begun_and_ended: std::rc::Rc::clone(&begun_and_ended),
        };
        on_a_watched_turn(
            "command-counted",
            &mut record,
            &mut bounding,
            Indicator::default().told_to(Box::new(watched.clone())),
            |turning, grants, strings, _| {
                let said = the_agent_asked(
                    &asking_to_run("true"),
                    turning,
                    grants,
                    strings,
                    &elsewhere.here(),
                    &mut bracket,
                );
                assert_eq!(said.refusal().is_some(), line.is_none(), "{said:?}");
            },
        );
        let entries: Vec<_> = record.everything().collect();
        assert_eq!(entries.len(), 1, "{level:?}: {entries:?}");
        let entry = entries.first().unwrap();
        match line {
            Some(line) => {
                assert_eq!(
                    watched.lists(),
                    [vec![line.to_owned()], Vec::new()],
                    "{level:?}"
                );
                assert!(matches!(entry.happened(), Happened::RanACommand { .. }));
                assert_eq!(entry.sent_bytes(), Some(4180));
                assert_eq!(begun_and_ended.get(), (1, 1), "{level:?}");
            }
            None => {
                assert!(
                    watched.lists().is_empty(),
                    "{level:?}: {:?}",
                    watched.lists()
                );
                assert!(matches!(
                    entry.happened(),
                    Happened::ACommandWasNotRun { .. }
                ));
                assert_eq!(entry.sent_bytes(), None);
                assert_eq!(begun_and_ended.get(), (0, 0), "{level:?}");
            }
        }
    }
}
