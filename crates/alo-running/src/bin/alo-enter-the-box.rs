//! Puts itself inside a box, and then becomes the command.
//!
//! The whole of this program is [`alo_running::enter_the_box_and_become`], and
//! it is a program at all because `pre_exec` is `unsafe` and
//! `CommandExt::exec` is not —
//! [ADR 0117](../../../../docs/decisions/0117-the-thing-that-starts-a-program-is-not-the-thing-that-bounds-a-turn.md)
//! has the argument.
//!
//! ```text
//! alo-enter-the-box <the service's boxes> <this box> <command> [arguments...]
//! ```
//!
//! It is started by `alo-running` and by nothing else. It is not a tool, it
//! takes no options, and it never reads a configuration file: everything it
//! does is in its four arguments, and the one that decides where it puts itself
//! is checked against the one before it.
//!
//! # It says nothing on success, because there is no success to say
//!
//! After the `exec` this process **is** the command, and whatever the command
//! prints is the command's. So the only output this program can ever produce is
//! a refusal, and a reader seeing a line from it is reading about a run that
//! did not start.

#![cfg(target_os = "linux")]

use std::{path::PathBuf, process::ExitCode};

/// Reads the four arguments, checks them, and hands over.
fn main() -> ExitCode {
    let mut asked = std::env::args_os().skip(1);
    let Some(inside) = asked.next().map(PathBuf::from) else {
        return said("the folder this service's boxes live under");
    };
    let Some(the_box) = asked.next().map(PathBuf::from) else {
        return said("the box to enter");
    };
    let Some(program) = asked.next().map(PathBuf::from) else {
        return said("the command to become");
    };
    let arguments: Vec<_> = asked.collect();

    // Returns only on failure. On success this process is the command and
    // nothing below here runs.
    let why = alo_running::enter_the_box_and_become(&inside, &the_box, &program, &arguments);
    refused(&format!(
        "{} did not start inside {}: {why}",
        program.display(),
        the_box.display()
    ))
}

/// Says which argument was missing, and stops.
///
/// Spelled out rather than a usage line, because the only thing that starts
/// this program is `alo-running` and a missing argument there is a bug in that
/// crate rather than a person holding it wrongly.
fn said(missing: &str) -> ExitCode {
    refused(&format!("nothing said {missing}"))
}

/// Writes a refusal and stops, **without panicking if the write is refused.**
///
/// This process is inside the box, and the box may refuse the very stream this
/// writes to. `eprintln!` panics when it is refused, and the panic's own
/// message is refused the same way — so on a booted machine on 2026-10-31 a
/// command that would not start ended with Rust's panic code, 101, and no word
/// to anybody. A refusal that cannot be written still ends as a refusal, with
/// this program's own code.
fn refused(why: &str) -> ExitCode {
    use std::io::Write;
    // Let go of on purpose: there is nowhere else to say it.
    writeln!(std::io::stderr(), "alo-enter-the-box: {why}").ok();
    ExitCode::FAILURE
}
