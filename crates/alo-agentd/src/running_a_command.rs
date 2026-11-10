//! **One command, where no verb fits: decided by the person's level, carried
//! out, and answered**
//! ([ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! The owner, 2026-11-08: *where no verb exists, alo does it by running a
//! command at full access (without asking) and at* ask me each time *(after the
//! OK). "There is no verb for that" is never an answer a person sees at either
//! level.* And, of what the command sees: *"We should give it always full
//! access and see all the folders if the users gives it full access in the
//! settings".*
//!
//! # The level is read here, at the moment, from the person's own file
//!
//! `/var/lib/alo/how-code-runs.toml`, through
//! `alo_remembering::the_level_remembered`, which answers three things:
//!
//! | the file says | what happens to the command |
//! |---|---|
//! | full access (`at-full-trust`) | it runs now, as the person, with no folder grants — after the snapshot, written down, and answered `done-without-asking` with the machine's own sentence for the person (`crate::saying_it_was_done_without_asking`); a `sudo` line asks the password instead, and is answered `did` |
//! | *ask me each time* | it waits for the person's OK under the same numbers every change waits under; nothing runs |
//! | the sealed box, **nothing chosen**, or **a file that will not read** | refused in the plain sentence, written down; nothing runs |
//!
//! **Not chosen and unreadable are the sealed box, never a higher level.** The
//! sealed box is ADR 0064's default, and a file that will not read is not a
//! person who chose anything. At the sealed box alo does not run the command at
//! all — the box is not widened for it — and says so in #1093's sentence, which
//! names the other levels and where to choose them.
//!
//! **And again at the OK.** A command approved at *ask me each time* is run
//! only if the level still allows running one when the person says yes: a
//! person who moved to the sealed box while the question stood has taken that
//! back, as a revoked grant is (`CLAUDE.md`: *a revoked grant takes effect
//! immediately*).
//!
//! # What stays on, and what is not yet true
//!
//! The record and the snapshot are kept for every run, as ADR 0064 and ADR 0149
//! require: the snapshot is asked for before the command starts, through the
//! turn's bracket (ADR 0045 — one that cannot be taken does not stop the run,
//! and the person's answer says it cannot be undone), and the run or refusal is
//! written down before anything is answered.
//!
//! **The egress indicator is not reached by a command.** What it sends leaves
//! from its own process, and the only counter this repository has for a
//! process's bytes is ADR 0148's, which counts an application's control group
//! at the desktop's request. Nothing here pretends otherwise; the gap is named
//! in `docs/contracts/daemon-protocol.md`, under `run`.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::{Duration, SystemTime};

use alo_capability::{ACommand, HowCodeRuns, ProposalId};
use alo_protocol::{Done, ToAPerson, ToAnAgent};
use alo_strings::{Filling, Said, Strings};
use alo_turn::{NotDone, Turning};

use crate::saying_it_was_done_without_asking::done_without_asking;
use crate::the_bracket_this_turn_takes::TheTurnsBracket;

/// Where the person's level for running code is kept, and the folder a command
/// runs in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandsHere {
    /// The person's own file, `/var/lib/alo/how-code-runs.toml` on a machine.
    level_at: PathBuf,
    /// Where a command starts: the person's home on a machine.
    in_folder: PathBuf,
    /// Where a command written as `sudo ...` is asked for as administrator
    /// (ADR 0150): `alo-running-as-administrator`'s socket on a machine.
    administrator_at: PathBuf,
}

/// A file nobody can have made, because nothing can create a folder under
/// `/proc` — so a level read from it is always *nothing chosen*.
const WHERE_NOTHING_IS_CHOSEN: &str = "/proc/alo-nothing-was-chosen/how-code-runs.toml";

/// A socket nobody can have made, for the same reason: a command asked for as
/// administrator through it is refused as *could not ask*, never run.
const WHERE_NO_RUNNER_IS: &str = "/proc/alo-no-runner-is-here/socket";

/// [`CommandsHere::where_nothing_was_chosen`], made once.
static NOTHING_CHOSEN: LazyLock<CommandsHere> =
    LazyLock::new(|| CommandsHere::at(Path::new(WHERE_NOTHING_IS_CHOSEN), Path::new("/")));

impl CommandsHere {
    /// The person's own file and their home, for the service on a machine.
    ///
    /// A session with no home starts a command at `/`: what it reaches is the
    /// account's permissions either way, and the folder is only where it
    /// begins.
    #[must_use]
    pub fn on_this_machine() -> Self {
        let home = std::env::var_os(alo_choosing::HOME)
            .map(PathBuf::from)
            .filter(|home| home.is_absolute())
            .unwrap_or_else(|| PathBuf::from("/"));
        Self::at(Path::new(alo_remembering::THE_LEVEL), &home).running_as_administrator_at(
            Path::new(alo_administrator::answering_as_administrator::THE_RUNNERS_SOCKET),
        )
    }

    /// The level kept at `level_at`, and commands started in `in_folder`.
    #[must_use]
    pub fn at(level_at: &Path, in_folder: &Path) -> Self {
        Self {
            level_at: level_at.to_owned(),
            in_folder: in_folder.to_owned(),
            administrator_at: PathBuf::from(WHERE_NO_RUNNER_IS),
        }
    }

    /// These, with a command written as `sudo ...` asked for as administrator
    /// at `socket` (ADR 0150). Without this, every such command is refused as
    /// *could not ask*, which is what a test of another door wants.
    #[must_use]
    pub fn running_as_administrator_at(mut self, socket: &Path) -> Self {
        self.administrator_at = socket.to_owned();
        self
    }

    /// Where nobody has chosen a level, so every command is refused as the
    /// sealed box refuses it — the default, for a caller with no person's file
    /// to read (the tests of every other door).
    #[must_use]
    pub fn where_nothing_was_chosen() -> &'static Self {
        &NOTHING_CHOSEN
    }

    /// The level the person chose, at this moment — and the sealed box for a
    /// person who has not chosen and for a file that will not read.
    #[must_use]
    pub fn the_level_now(&self) -> HowCodeRuns {
        match alo_remembering::the_level_remembered(&self.level_at) {
            Ok(Some(level)) => level,
            Ok(None) => HowCodeRuns::InASealedBox,
            Err(why) => {
                eprintln!(
                    "alo-agentd: the level for running code at {} would not read, so a command \
                     is refused as the sealed box refuses it: {why:?}",
                    self.level_at.display()
                );
                HowCodeRuns::InASealedBox
            }
        }
    }
}

/// What a command may reach while it is asked for: where the level is, and the
/// turn's bracket.
#[derive(Debug)]
pub struct Commands<'c> {
    /// Where the level is, and where a command starts.
    here: &'c CommandsHere,
    /// The turn's bracket, taken before a command runs — absent on a road that
    /// has none, where a run is then said to be one that cannot be undone.
    bracket: Option<&'c mut TheTurnsBracket>,
}

impl<'c> Commands<'c> {
    /// What a command reaches on this road.
    #[must_use]
    pub fn here(here: &'c CommandsHere, bracket: Option<&'c mut TheTurnsBracket>) -> Self {
        Self { here, bracket }
    }

    /// Where nothing was chosen and there is no bracket: every command is
    /// refused, as the sealed box refuses it.
    #[must_use]
    pub fn where_nothing_was_chosen() -> Commands<'static> {
        Commands {
            here: CommandsHere::where_nothing_was_chosen(),
            bracket: None,
        }
    }

    /// Where the person's level is read from, for a door that asks the grants
    /// as the level is now (`crate::the_grants_as_the_level_is_now`).
    #[must_use]
    pub fn level_at(&self) -> &'c CommandsHere {
        self.here
    }

    /// Where the level is, and the turn's bracket — for a change at full
    /// access, which takes the bracket as a command does
    /// (`crate::a_change_asked_for`).
    pub(crate) fn into_parts(self) -> (&'c CommandsHere, Option<&'c mut TheTurnsBracket>) {
        (self.here, self.bracket)
    }
}

/// **A command an agent asked for**, decided by the person's level and
/// answered — run, waiting, or refused, and written down in every case.
pub fn a_command_asked_for(
    line: &str,
    turning: &mut Turning<'_, '_>,
    commands: Commands<'_>,
    strings: &Strings,
    standing: Duration,
    now: SystemTime,
) -> ToAnAgent {
    let command = match ACommand::of(line) {
        Ok(command) => command,
        Err(why) => return refused(turning, &why.said(strings), strings, now),
    };
    match commands.here.the_level_now() {
        HowCodeRuns::AtFullTrust => {
            let Commands { here, mut bracket } = commands;
            let as_administrator =
                crate::running_as_administrator::the_administrators_part(&command).is_some();
            match ran(
                turning,
                &command,
                here,
                bracket.as_deref_mut(),
                strings,
                now,
            ) {
                // **Asked, for their password**, so not done without asking.
                Ok(did) if as_administrator => ToAnAgent::did(did),
                // **Done without asking**, and the person is told so in the
                // machine's own words, as a verb's change is
                // (`crate::saying_it_was_done_without_asking`).
                Ok(did) => done_without_asking(
                    did,
                    command.sentence(strings),
                    bracket_kept(bracket.as_deref()),
                    strings,
                ),
                Err(why) => ToAnAgent::refused(&why),
            }
        }
        HowCodeRuns::AskingEachTime => match turning.proposing_a_command(command, standing, now) {
            Ok(number) => match turning.command_proposed(number) {
                Some(waiting) => ToAnAgent::a_command_proposed(waiting, strings, now),
                None => ToAnAgent::refused(
                    &alo_capability::AnswerError::NothingWaiting {
                        number: number.as_u64(),
                    }
                    .said(strings),
                ),
            },
            Err(why) => ToAnAgent::refused(&why.said(strings)),
        },
        HowCodeRuns::InASealedBox => refused(
            turning,
            &not_in_the_sealed_box(&command, strings),
            strings,
            now,
        ),
    }
}

/// **The person said yes to a waiting command**, so it runs once — if their
/// level still lets a command run.
pub fn a_command_approved(
    turning: &mut Turning<'_, '_>,
    number: ProposalId,
    commands: Commands<'_>,
    strings: &Strings,
    now: SystemTime,
) -> ToAPerson {
    let Some(command) = turning
        .command_proposed(number)
        .map(|waiting| waiting.command().clone())
    else {
        return ToAPerson::refused(
            &alo_capability::AnswerError::NothingWaiting {
                number: number.as_u64(),
            }
            .said(strings),
        );
    };
    if commands.here.the_level_now() == HowCodeRuns::InASealedBox {
        // Taken back while the question stood: the person's level now says
        // no, so the command is taken off the list and written down with the
        // sentence, and nothing runs.
        let said = not_in_the_sealed_box(&command, strings);
        return match turning.a_command_taken_back(number, &said, now) {
            Ok(()) => ToAPerson::refused(&said),
            Err(why) => ToAPerson::refused(&why.said(strings)),
        };
    }
    let Commands { here, mut bracket } = commands;
    take_the_bracket(turning, &command, bracket.as_deref_mut(), strings);
    let kept = bracket_kept(bracket.as_deref());
    // **A command written as `sudo ...` asks the person's password now**, after
    // the OK, and runs as administrator only on their yes (ADR 0150).
    if crate::running_as_administrator::the_administrators_part(&command).is_some() {
        let mut run = |command: &ACommand| {
            crate::running_as_administrator::through_the_runner(
                &here.administrator_at,
                command,
                strings,
            )
        };
        return match turning.approving_a_command_as_administrator(number, &mut run, now) {
            Ok(Ok(did)) => crate::answering::the_change_did(Done::of_a_command(&did), kept),
            Ok(Err(said)) => ToAPerson::refused(&said),
            Err(why) => ToAPerson::refused(&said_about(&why, strings)),
        };
    }
    let mut run = |command: &ACommand| alo_running::run_as_the_person(command, &here.in_folder);
    match turning.approving_a_command(number, &mut run, now) {
        Ok(did) => crate::answering::the_change_did(Done::of_a_command(&did), kept),
        Err(why) => ToAPerson::refused(&said_about(&why, strings)),
    }
}

/// Run it at full access: the bracket first, then the run, then the record.
fn ran(
    turning: &mut Turning<'_, '_>,
    command: &ACommand,
    here: &CommandsHere,
    bracket: Option<&mut TheTurnsBracket>,
    strings: &Strings,
    now: SystemTime,
) -> Result<Done, Said> {
    take_the_bracket(turning, command, bracket, strings);
    // **A command written as `sudo ...` asks the person's password**, even at
    // full access, and runs as administrator only on their yes (ADR 0150).
    if crate::running_as_administrator::the_administrators_part(command).is_some() {
        let mut run = |command: &ACommand| {
            crate::running_as_administrator::through_the_runner(
                &here.administrator_at,
                command,
                strings,
            )
        };
        return match turning.running_as_administrator(command, &mut run, now) {
            Ok(Ok(did)) => Ok(Done::of_a_command(&did)),
            Ok(Err(said)) => Err(said),
            Err(why) => Err(said_about(&why, strings)),
        };
    }
    let mut run = |command: &ACommand| alo_running::run_as_the_person(command, &here.in_folder);
    match turning.running_a_command(command, &mut run, now) {
        Ok(did) => Ok(Done::of_a_command(&did)),
        Err(why) => Err(said_about(&why, strings)),
    }
}

/// **The snapshot, before the command starts** (ADR 0045, ADR 0149). A bracket
/// that cannot be taken does not stop the run; the turn is told the place's
/// name when one was kept, so the run is written down with it.
fn take_the_bracket(
    turning: &mut Turning<'_, '_>,
    command: &ACommand,
    bracket: Option<&mut TheTurnsBracket>,
    strings: &Strings,
) {
    let Some(bracket) = bracket else {
        return;
    };
    bracket.before_a_change(command.sentence(strings).text());
    if let Some(named) = bracket
        .taken_at()
        .and_then(Path::file_name)
        .and_then(std::ffi::OsStr::to_str)
    {
        turning.a_place_was_kept(named);
    }
}

/// Whether a place to return to was kept, which decides whether the person is
/// told the change can be put back.
fn bracket_kept(bracket: Option<&TheTurnsBracket>) -> bool {
    bracket.is_some_and(|bracket| bracket.taken_at().is_some())
}

/// The sealed box's plain sentence: the command was not run, and where to
/// choose a level that runs one (#1093, #1126).
fn not_in_the_sealed_box(command: &ACommand, strings: &Strings) -> Said {
    strings.say(
        &alo_capability::words::THE_SEALED_BOX_CANNOT_RUN_PROGRAMS_HERE_YET.key(),
        &Filling::of("program", command.program()),
    )
}

/// A refusal written down and answered — or, if it could not be written down,
/// that, because a refusal nobody kept is not the whole truth.
fn refused(
    turning: &mut Turning<'_, '_>,
    said: &Said,
    strings: &Strings,
    now: SystemTime,
) -> ToAnAgent {
    match turning.a_command_was_not_run(said, now) {
        Ok(()) => ToAnAgent::refused(said),
        Err(why) => ToAnAgent::refused(&why.said(strings)),
    }
}

/// What a turn's refusal of a command says — and, for a shell that would not
/// start, the English for whoever looks after the machine.
fn said_about(why: &NotDone, strings: &Strings) -> Said {
    if let NotDone::ACommandDidNotStart(not_started) = why {
        eprintln!(
            "alo-agentd: a command alo was to run did not start: {}",
            not_started.why()
        );
    }
    why.said(strings)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "running_a_command_tests.rs"]
mod tests;
