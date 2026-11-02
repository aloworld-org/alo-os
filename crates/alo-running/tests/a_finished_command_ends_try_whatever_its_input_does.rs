//! **A finished command ends `try`, whatever its input is doing.**
//!
//! On a booted machine on 2026-11-02 (main `8263ddcb`, qemu, measured once)
//! every `alo-try` run under `timeout 60` ended `exit=124` with no output —
//! `/usr/bin/true` included, and a statically linked program on the home
//! subvolume included, so it was not the boundary. `try` carries the person's
//! input into the box (`alo_running::carried_until_it_ends`), and it **read the
//! terminal straight away**, whether or not the command ever asked for input.
//! `timeout` puts what it runs in a process group of its own, which is a
//! background group to the terminal, and a background group that reads its
//! terminal is stopped by it (`SIGTTIN`). So `try` was stopped before its
//! command could end, and the same happens to `try true &` at a prompt.
//!
//! The person's input is real only here, so these tests start this same test
//! binary again as the carrier ([`the_carrier_when_one_is_asked_for`]) with
//! the input arranged: a terminal it is in the background of, a pipe nobody
//! closes, and a pipe that says something.
//!
//! In `tests/` rather than beside the code, because a child process is started
//! here, and `only_one_place_in_this_crate_starts_a_program.rs` holds `src/` to
//! starting nothing but the helper.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use alo_running::{Carrying, carried_until_it_ends};

/// Set in the environment of the copy of this binary that is the carrier: the
/// shell script its command is.
const BE_THE_CARRIER: &str = "ALO_RUNNING_BE_THE_CARRIER";

/// How long a carrier is given before `timeout` ends it. Every command here
/// ends at once, so anything near this is the fault, never a slow machine.
const GIVEN: &str = "20";

/// What `timeout` answers when it had to end what it ran.
const TIMED_OUT: i32 = 124;

/// What every command here exits with, so the carrier's own code is told apart
/// from a timeout and from a crash.
const THE_COMMANDS_OWN: i32 = 7;

/// **The carrier, when a test asks for one** — and nothing at all otherwise.
///
/// Started by the tests below as this binary, `--exact` this test, with
/// [`BE_THE_CARRIER`] set. It does what `try` does with a box: starts the
/// command with all three streams piped and carries them to and from its own,
/// then ends with the command's code. Run in the ordinary way it is a test
/// that returns at once.
#[test]
fn the_carrier_when_one_is_asked_for() {
    let Ok(script) = std::env::var(BE_THE_CARRIER) else {
        return;
    };
    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("a shell to start");
    let ended = carried_until_it_ends(&mut child, Carrying::ToThePerson)
        .expect("the child to be waited for");
    std::process::exit(ended.code().unwrap_or(-1));
}

/// The carrier, under `timeout` exactly as the booted machine's measurement
/// ran it, from a shell whose input is `input`. Says the shell's report of how
/// `timeout` ended, what the carrier printed, and how long it took.
///
/// `a_terminal` says the input is one, and then it is made the shell's
/// controlling terminal, as the person's shell has; `setsid` refuses to make
/// anything else one, so a pipe is given a session with no terminal at all.
fn carried_under_a_timeout(
    script: &str,
    input: Stdio,
    a_terminal: bool,
) -> (Option<i32>, String, Duration) {
    let this = std::env::current_exe().expect("this test binary to have a path");
    let began = Instant::now();
    let mut setsid = Command::new("setsid");
    if a_terminal {
        setsid.arg("--ctty");
    }
    let finished = setsid
        .args(["--wait", "sh", "-c"])
        // The report on a line of its own: the test harness inside does not
        // end its own last line before the carrier exits.
        .arg(
            "timeout \"$1\" \"$2\" --exact the_carrier_when_one_is_asked_for --nocapture \
             --test-threads=1; ended=$?; echo; echo \"timeout-ended=$ended\"",
        )
        .args(["sh", GIVEN])
        .arg(this)
        .env(BE_THE_CARRIER, script)
        .stdin(input)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .expect("setsid and timeout to start");
    let said = String::from_utf8_lossy(&finished.stdout).into_owned();
    let code = said
        .lines()
        .find_map(|line| line.strip_prefix("timeout-ended="))
        .and_then(|code| code.trim().parse().ok());
    (code, said, began.elapsed())
}

/// A terminal for the carrier to be in the background of, and its far end,
/// which is held open and never written to — a person who typed nothing.
fn a_terminal_nobody_types_at() -> (std::fs::File, Stdio) {
    use rustix::pty::{OpenptFlags, grantpt, openpt, ptsname, unlockpt};
    let far_end = openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).expect("a terminal to open");
    grantpt(&far_end).expect("the terminal to be granted");
    unlockpt(&far_end).expect("the terminal to be unlocked");
    let near_end = ptsname(&far_end, Vec::new()).expect("the terminal to have a name");
    let near_end = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(near_end.to_string_lossy().as_ref())
        .expect("the terminal's near end to open");
    (std::fs::File::from(far_end), Stdio::from(near_end))
}

/// **The machine's case: a command that never reads, from a terminal `try` is
/// in the background of.** The command ends at once, so `try` must too.
///
/// This is the test that failed before the fix: the carrier read the terminal
/// at once, the terminal stopped it, and `timeout` ended it.
#[test]
fn a_command_that_never_reads_ends_try_in_the_background_of_a_terminal() {
    let (far_end, near_end) = a_terminal_nobody_types_at();
    let (code, said, took) =
        carried_under_a_timeout(&format!("exit {THE_COMMANDS_OWN}"), near_end, true);
    drop(far_end);

    assert_ne!(
        code,
        Some(TIMED_OUT),
        "try was still running {took:?} after its command ended, until timeout stopped it: {said}"
    );
    assert_eq!(code, Some(THE_COMMANDS_OWN), "{said}");
}

/// **The same command, with input that is open and never ends**: a pipe whose
/// writer is held for as long as this test runs. Waiting for its end would be
/// waiting for ever.
#[test]
fn a_command_that_never_reads_ends_try_while_its_input_stays_open() {
    let (reader, writer) = std::io::pipe().expect("a pipe");
    let (code, said, took) = carried_under_a_timeout(
        &format!("exit {THE_COMMANDS_OWN}"),
        Stdio::from(reader),
        false,
    );
    drop(writer);

    assert_ne!(
        code,
        Some(TIMED_OUT),
        "try waited {took:?} for an input that was never going to end: {said}"
    );
    assert_eq!(code, Some(THE_COMMANDS_OWN), "{said}");
}

/// **And what the person does send still arrives**, which is the half the fix
/// must not break: a command that reads its input is given it, whole.
#[test]
fn input_that_is_sent_still_reaches_the_command() {
    let (reader, mut writer) = std::io::pipe().expect("a pipe");
    writer
        .write_all(b"said to the box\n")
        .expect("the pipe to take a line");
    drop(writer);
    let (code, said, _) = carried_under_a_timeout(
        &format!("read line; echo \"heard: $line\"; exit {THE_COMMANDS_OWN}"),
        Stdio::from(reader),
        false,
    );

    assert_eq!(code, Some(THE_COMMANDS_OWN), "{said}");
    assert!(
        said.contains("heard: said to the box"),
        "what the person sent did not reach the command: {said}"
    );
}
