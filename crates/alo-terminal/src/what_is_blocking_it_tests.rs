//! That the three readers name the right process, measured against real ones.
//!
//! # The parser is tested on the two lines the kernel really writes
//!
//! Both were taken from `/proc/locks` on 2026-10-17 while two processes
//! contended for one file, and they are the reason `ALock::read` skips the
//! arrow instead of counting fields. A parser that counted would read a pid
//! from one line and a lock mode from the other — **a number that is not a
//! process, compared against a process, quietly never matching.**

use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use super::{ALock, WhatIsBlockingIt, a_child_of_its_own, a_pipe_fed_by, what_is_blocking};

/// Long enough for a process to start on a loaded machine.
const PATIENCE: Duration = Duration::from_secs(10);

/// **The two lines the kernel writes, and the pid read out of each.**
///
/// The holder and the process blocked behind it, sharing an index, with every
/// field after the arrow shifted by one.
#[test]
fn a_holder_and_a_waiter_are_read_from_the_lines_the_kernel_writes() {
    let holder = ALock::read("10: FLOCK  ADVISORY  WRITE 2142693 fd:01:4311 0 EOF")
        .expect("a holder's line reads");
    assert_eq!(holder.index, "10");
    assert!(!holder.blocked, "a holder was read as blocked");
    assert_eq!(holder.process, 2_142_693);

    let waiter = ALock::read("10: -> FLOCK  ADVISORY  WRITE 2142696 fd:01:4311 0 EOF")
        .expect("a waiter's line reads");
    assert_eq!(waiter.index, "10", "the two lines share an index");
    assert!(waiter.blocked, "a blocked waiter was read as a holder");
    assert_eq!(
        waiter.process, 2_142_696,
        "the arrow shifted the fields and the pid was read from the wrong one"
    );

    // **The control for the shift.** Were the arrow ignored, the waiter's pid
    // would read as the lock's kind — which does not parse as a number, so the
    // line would be dropped rather than misread. Both failures are silent in
    // different ways, and this says the right one happened.
    assert_ne!(
        holder.process, waiter.process,
        "both lines read as the same process, so the arrow is not being handled"
    );
}

/// **A POSIX line reads too**, because the kind is a token this skips rather
/// than a word it knows.
#[test]
fn the_kind_of_lock_is_skipped_rather_than_known() {
    let posix = ALock::read("3: POSIX  ADVISORY  WRITE 1938013 00:1d:7482 0 EOF")
        .expect("a POSIX line reads");
    assert_eq!(posix.process, 1_938_013);
    assert!(!posix.blocked);

    // And something that is not a lock line at all is refused rather than
    // guessed at.
    assert!(ALock::read("").is_none());
    assert!(ALock::read("not a lock line").is_none());
}

/// **A command reading a pipe is told who feeds it.**
///
/// `sleep 60 | cat` really started: the writer holds the pipe as its output and
/// the reader as its input, which is the pairing this walks `/proc` to find.
#[test]
fn a_pipe_is_traced_to_the_process_feeding_it() {
    let mut writer = Command::new("/bin/sh")
        .args(["-c", "sleep 60"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("a shell starts");
    let pipe = writer.stdout.take().expect("stdout was asked for");

    let mut reader = Command::new("/bin/cat")
        .stdin(Stdio::from(pipe))
        .stdout(Stdio::null())
        .spawn()
        .expect("cat starts");

    let found = until(|| a_pipe_fed_by(reader.id()));
    drop(stop(&mut reader));
    drop(stop(&mut writer));

    let who = found.expect("the process feeding the pipe is named");
    assert_eq!(
        who,
        writer.id(),
        "the pipe was traced to {who} and the process feeding it is {}",
        writer.id()
    );
}

/// **And `what_is_blocking` reaches the same answer**, which is what a caller
/// actually asks.
#[test]
fn what_is_blocking_names_the_pipe_and_who_feeds_it() {
    let mut writer = Command::new("/bin/sh")
        .args(["-c", "sleep 60"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("a shell starts");
    let pipe = writer.stdout.take().expect("stdout was asked for");
    let mut reader = Command::new("/bin/cat")
        .stdin(Stdio::from(pipe))
        .stdout(Stdio::null())
        .spawn()
        .expect("cat starts");

    let found = until(|| what_is_blocking(reader.id()));
    drop(stop(&mut reader));
    drop(stop(&mut writer));

    match found.expect("something is blocking a command reading an idle pipe") {
        WhatIsBlockingIt::APipeFedBy(who) => assert_eq!(who, writer.id()),
        other => panic!("a command reading an idle pipe was blamed on {other:?}"),
    }
}

/// **A process waiting for a child is told which child.**
#[test]
fn a_child_is_named_to_the_process_waiting_for_it() {
    // A shell that waits for a sleep it started: the shell is the parent and
    // has exactly one child to be told about.
    let mut parent = Command::new("/bin/sh")
        .args(["-c", "sleep 60"])
        .stdout(Stdio::null())
        .spawn()
        .expect("a shell starts");

    let found = until(|| a_child_of_its_own(parent.id()));
    drop(stop(&mut parent));

    let child = found.expect("a shell waiting for a sleep has a child to name");
    assert_ne!(child, parent.id(), "a process was named as its own child");
}

/// **Nothing is blocking a process that is simply working.**
///
/// The control for all of the above: a reader that answered *a child* about
/// anything would pass every test here except this one.
#[test]
fn nothing_is_blocking_a_process_that_is_working() {
    let mut busy = Command::new("/bin/sh")
        .args(["-c", "while :; do :; done"])
        .stdout(Stdio::null())
        .spawn()
        .expect("a shell starts");

    // It has no pipe on its input, holds no lock, and has no child: the `sh`
    // runs the loop itself.
    let answer = what_is_blocking(busy.id());
    drop(stop(&mut busy));

    assert_eq!(
        answer, None,
        "a process doing nothing but work was reported as blocked by {answer:?}"
    );
}

/// Keep asking until there is an answer, or give up loudly.
fn until<T>(mut ask: impl FnMut() -> Option<T>) -> Option<T> {
    let began = Instant::now();
    while began.elapsed() < PATIENCE {
        if let Some(answer) = ask() {
            return Some(answer);
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    None
}

/// Stop a child this test started, so nothing is left running on the machine.
fn stop(child: &mut Child) -> std::io::Result<()> {
    child.kill()?;
    child.wait().map(drop)
}
