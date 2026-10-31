//! **What a boxed command says reaches the person** — the carrying half, run on
//! any Linux machine.
//!
//! A booted machine on 2026-10-31 ran `try id` inside the person's session and
//! printed nothing at all: the box refuses the person's terminal, so a command
//! that wrote to it wrote to nothing. `alo_running::carried_until_it_ends` is
//! what carries the box's output out through pipes instead, and these tests
//! hold it to arriving whole. The kernel half — that a real box's output
//! arrives — is in `the_kernel_refuses_what_a_run_was_not_given.rs`, which
//! needs a loaded boundary.
//!
//! In `tests/` rather than beside the code, because a child process is started
//! here, and `only_one_place_in_this_crate_starts_a_program.rs` holds `src/` to
//! starting nothing but the helper.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::process::{Child, Command, Stdio};

use alo_running::{Carrying, carried_until_it_ends};

/// A shell running `script`, with all three streams piped as a box's are.
fn piped(script: &str) -> Child {
    Command::new("/bin/sh")
        .arg("-c")
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("a shell to start")
}

/// Run `script` and say what came out of each stream, and how it ended.
fn what_it_said(script: &str) -> (Option<i32>, Vec<u8>, Vec<u8>) {
    let mut child = piped(script);
    let mut out = Vec::new();
    let mut err = Vec::new();
    let ended = carried_until_it_ends(
        &mut child,
        Carrying::To {
            out: &mut out,
            err: &mut err,
        },
    )
    .expect("the child to be waited for");
    (ended.code(), out, err)
}

/// **What a command says on both streams arrives, whole and apart.**
#[test]
fn both_streams_arrive_whole_and_apart() {
    let (code, out, err) = what_it_said("echo said; echo complained >&2; exit 3");

    assert_eq!(code, Some(3));
    assert_eq!(String::from_utf8_lossy(&out), "said\n");
    assert_eq!(String::from_utf8_lossy(&err), "complained\n");
}

/// **A refusal arrives**, which is the half of law 3 the booted machine broke:
/// the helper's own sentence about a command that would not start.
#[test]
fn a_refusal_on_standard_error_alone_arrives() {
    let (code, out, err) =
        what_it_said("echo 'alo-enter-the-box: /usr/bin/id did not start' >&2; exit 1");

    assert_eq!(code, Some(1));
    assert!(out.is_empty());
    assert!(
        String::from_utf8_lossy(&err).contains("did not start"),
        "a refusal written to standard error did not reach the person"
    );
}

/// **More than a pipe holds arrives too**, so a command that says a lot is not
/// stuck waiting for somebody to read.
#[test]
fn more_than_a_pipe_holds_arrives() {
    let (code, out, err) = what_it_said("head -c 1000000 /dev/zero; head -c 300000 /dev/zero >&2");

    assert_eq!(code, Some(0));
    assert_eq!(out.len(), 1_000_000);
    assert_eq!(err.len(), 300_000);
}

/// **The helper ends as a refusal even when it cannot say why**, never as a
/// panic.
///
/// The booted machine's 101: inside the box its stderr was refused, so
/// `eprintln!` panicked and the panic's message was refused too. `/dev/full`
/// refuses every write the same way (`ENOSPC` rather than the boundary's
/// `EACCES`, and `eprintln!` panics on both). Measured once on the shipped
/// helper before this change: exit 101, nothing printed.
#[test]
fn the_helper_ends_as_a_refusal_when_its_refusal_cannot_be_written() {
    let refusing = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("/dev/full to open");
    let ended = Command::new(env!("CARGO_BIN_EXE_alo-enter-the-box"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(refusing)
        .status()
        .expect("the helper to start");

    assert_eq!(
        ended.code(),
        Some(1),
        "the helper did not end as its own refusal; 101 is a panic, which says nothing"
    );
}

/// **And with somewhere to write, it says what was missing.**
#[test]
fn the_helper_says_why_when_it_can() {
    let said = Command::new(env!("CARGO_BIN_EXE_alo-enter-the-box"))
        .stdin(Stdio::null())
        .output()
        .expect("the helper to start");

    assert_eq!(said.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&said.stderr).contains("nothing said"),
        "{:?}",
        String::from_utf8_lossy(&said.stderr)
    );
}

/// **A command that reads its input is given an end of it**, rather than
/// waiting for ever on a caller with no terminal.
#[test]
fn a_command_that_reads_is_given_an_end() {
    let (code, out, _) = what_it_said("cat; echo done");

    assert_eq!(code, Some(0));
    assert_eq!(String::from_utf8_lossy(&out), "done\n");
}
