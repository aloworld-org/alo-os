//! **A command run at full access: as the person, with no box around it**
//! ([ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! The owner, 2026-11-08: *at full access alo can do everything the person's
//! account can*, and *"We should give it always full access and see all the
//! folders if the users gives it full access in the settings".* So when the
//! person chose full access — or said yes to this one command at *ask me each
//! time* — nothing alo runs is boxed: the command runs as the account the
//! service runs as, sees every folder that account sees, and is reached by no
//! grant.
//!
//! # Why this file may start a program at all
//!
//! ADR 0117 put the one `exec` in this crate, and
//! `tests/only_one_place_in_this_crate_starts_a_program.rs` holds every file
//! here to it. This is the third file that test names, and the only one that
//! starts a program outside a box — which is the point of it, and why it takes
//! nothing but an [`alo_capability::ACommand`]: it is reached only once the person's level has
//! been read and said yes, by the caller that read it. Nothing here reads the
//! level, and nothing here decides it.
//!
//! # What the three always-on protections are, here
//!
//! - **The record** is written by whoever called this (`alo_turn`, through the
//!   turn's own record), before the answer is given.
//! - **The snapshot before each run** is taken by the caller before calling
//!   this, because one taken from in here would be taken after the command had
//!   started.
//! - **The egress indicator** is reached through what the caller hands in
//!   ([`run_as_the_person_around`], since 2026-11-13, ADR 0148 as amended):
//!   the command is started through [`AroundACommand::starting`], so a caller
//!   that holds a control group for it has it born inside that group, and
//!   [`AroundACommand::looking`] is called on every look while it runs, so the
//!   count is read and the line put up while it is still sending. Nothing here
//!   counts or shows anything itself; this file only gives the caller the two
//!   moments it needs, and [`run_as_the_person`] hands in nothing.
//!
//! # Bounded twice
//!
//! What it writes is kept up to [`THE_MOST_IT_SAYS`] bytes and said to have
//! been cut short beyond that, and it runs for at most
//! [`THE_LONGEST_A_COMMAND_RUNS`]: the service answers one thing at a time, so
//! a command that never ended would be a machine that stopped answering
//! anybody. A command stopped for running too long is said to have been.

use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use alo_capability::{
    ACommand, AroundACommand, ItDidNotStart, NothingAround, THE_SHELL, WhatACommandDid,
};

/// The longest a command alo runs is let run before it is stopped.
pub const THE_LONGEST_A_COMMAND_RUNS: Duration = Duration::from_secs(120);

/// The most of what a command writes that is kept, in bytes.
pub const THE_MOST_IT_SAYS: usize = 64 * 1024;

/// How long one look at the command waits before looking again.
const ONE_LOOK: Duration = Duration::from_millis(50);

/// Run `command` as the person, in `folder`, for at most
/// [`THE_LONGEST_A_COMMAND_RUNS`].
///
/// # Errors
/// [`ItDidNotStart`] when the shell could not be started, or the machine lost
/// track of it — in English, for the service log.
pub fn run_as_the_person(
    command: &ACommand,
    folder: &Path,
) -> Result<WhatACommandDid, ItDidNotStart> {
    run_as_the_person_within(command, folder, THE_LONGEST_A_COMMAND_RUNS)
}

/// [`run_as_the_person`], stopped after `within` rather than the usual bound.
///
/// # Errors
/// As [`run_as_the_person`].
pub fn run_as_the_person_within(
    command: &ACommand,
    folder: &Path,
    within: Duration,
) -> Result<WhatACommandDid, ItDidNotStart> {
    run_as_the_person_around_within(command, folder, within, &mut NothingAround)
}

/// [`run_as_the_person`], started through `around` and looked at through it
/// while it runs — what a command alo runs in a turn is run with, so the
/// kernel can count it and the indicator show it (ADR 0148, as amended
/// 2026-11-13).
///
/// # Errors
/// As [`run_as_the_person`], and also when `around` could not start it where
/// it was to begin — then nothing ran.
pub fn run_as_the_person_around(
    command: &ACommand,
    folder: &Path,
    around: &mut dyn AroundACommand,
) -> Result<WhatACommandDid, ItDidNotStart> {
    run_as_the_person_around_within(command, folder, THE_LONGEST_A_COMMAND_RUNS, around)
}

/// [`run_as_the_person_around`], stopped after `within` rather than the usual
/// bound.
///
/// # Errors
/// As [`run_as_the_person_around`].
pub fn run_as_the_person_around_within(
    command: &ACommand,
    folder: &Path,
    within: Duration,
    around: &mut dyn AroundACommand,
) -> Result<WhatACommandDid, ItDidNotStart> {
    let (mut reading, writing) = std::io::pipe()
        .map_err(|why| ItDidNotStart::because(format!("no pipe for what it says: {why}")))?;
    let also_writing = writing
        .try_clone()
        .map_err(|why| ItDidNotStart::because(format!("no second end of the pipe: {why}")))?;
    let mut starting = Command::new(THE_SHELL);
    starting
        .args(command.arguments())
        .current_dir(folder)
        .stdin(Stdio::null())
        .stdout(writing)
        .stderr(also_writing)
        // **A group of its own**, so that stopping it for running too long
        // stops what it started as well, rather than only the shell.
        .process_group(0);
    // **Started from where the caller says it begins** — inside the control
    // group alo counts it in, when there is one — so it is inside from its
    // first instruction rather than moved in after it may have sent something.
    let started = around.starting(&mut || starting.spawn());
    // The command now holds the writing ends; ours must go, or the end of
    // what it says would never arrive.
    drop(starting);
    let mut child = started
        .map_err(|why| ItDidNotStart::because(format!("{THE_SHELL} would not start: {why}")))?;

    let began = Instant::now();
    let mut said = Vec::new();
    let mut cut_short = false;
    let mut ended_what_it_says = false;
    loop {
        if !ended_what_it_says {
            ended_what_it_says =
                read_what_is_there(&mut reading, &mut said, &mut cut_short, ONE_LOOK);
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                // **Whatever is already there is read; nothing more is waited
                // for.** Something it left running in the background may hold
                // the pipe open for ever, and that is not this command.
                while !ended_what_it_says && began.elapsed() < within && has_more(&reading) {
                    ended_what_it_says =
                        read_what_is_there(&mut reading, &mut said, &mut cut_short, Duration::ZERO);
                }
                return Ok(WhatACommandDid::of(
                    status.code(),
                    String::from_utf8_lossy(&said).into_owned(),
                    cut_short,
                    false,
                ));
            }
            Ok(None) => {}
            Err(why) => {
                stop_it(&mut child);
                return Err(ItDidNotStart::because(format!("lost track of it: {why}")));
            }
        }
        // **A look at it while it is still running**, and only then: the count
        // is read here and the indicator goes up the first time it moves.
        around.looking();
        if began.elapsed() >= within {
            stop_it(&mut child);
            return Ok(WhatACommandDid::of(
                None,
                String::from_utf8_lossy(&said).into_owned(),
                cut_short,
                true,
            ));
        }
        if ended_what_it_says {
            std::thread::sleep(ONE_LOOK);
        }
    }
}

/// Read whatever the command has written, waiting at most `waiting` for
/// something to arrive. Answers whether what it says has ended.
fn read_what_is_there(
    from: &mut std::io::PipeReader,
    into: &mut Vec<u8>,
    cut_short: &mut bool,
    waiting: Duration,
) -> bool {
    use rustix::event::{PollFd, PollFlags, Timespec, poll};
    let timeout = Timespec::try_from(waiting).ok();
    let mut asking = [PollFd::new(&*from, PollFlags::IN)];
    match poll(&mut asking, timeout.as_ref()) {
        Ok(0) => return false,
        Ok(_) => {}
        Err(rustix::io::Errno::INTR) => return false,
        Err(_) => return true,
    }
    let mut arrived = [0_u8; 8192];
    match from.read(&mut arrived) {
        Ok(0) | Err(_) => true,
        Ok(length) => {
            let room = THE_MOST_IT_SAYS.saturating_sub(into.len());
            let these = arrived.get(..length).unwrap_or_default();
            if these.len() > room {
                *cut_short = true;
            }
            into.extend_from_slice(these.get(..room.min(these.len())).unwrap_or_default());
            false
        }
    }
}

/// Whether more of what the command says — or its end — is there to read at
/// this moment.
fn has_more(from: &std::io::PipeReader) -> bool {
    use rustix::event::{PollFd, PollFlags, Timespec, poll};
    let mut asking = [PollFd::new(from, PollFlags::IN)];
    let at_once = Timespec::try_from(Duration::ZERO).ok();
    matches!(poll(&mut asking, at_once.as_ref()), Ok(count) if count > 0)
}

/// Stop the command and everything it started, and wait for it.
fn stop_it(child: &mut Child) {
    if let Some(group) = i32::try_from(child.id())
        .ok()
        .and_then(rustix::process::Pid::from_raw)
    {
        let _ = rustix::process::kill_process_group(group, rustix::process::Signal::KILL);
    }
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
#[path = "at_full_access_tests.rs"]
mod tests;
