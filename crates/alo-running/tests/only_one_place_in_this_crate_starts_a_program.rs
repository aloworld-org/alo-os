//! This crate may start a program, and exactly one file in it may.
//!
//! `alo-bounding` has the opposite test — `a_turn_is_this_thread.rs` reads its
//! own source and fails on any spelling of `exec`, because law 2 is that the
//! crate serving the broker's closed verb list starts nothing.
//! [ADR 0117](../../../docs/decisions/0117-the-thing-that-starts-a-program-is-not-the-thing-that-bounds-a-turn.md)
//! moved the `exec` here instead, and promised this test in return.
//!
//! # What it is really asserting, after the first version of it was wrong
//!
//! Not *there is one `exec`*. **That the command a person approved is only ever
//! reached after the admission.**
//!
//! This test was first written as *exactly one file here starts a program*, and
//! it failed on its own crate: there are **two**, and both belong.
//!
//! | file | what it starts | why it is allowed |
//! |---|---|---|
//! | `a_run_inside_a_box.rs` | `alo-enter-the-box`, and nothing else | a fixed program of ours, named by a constant, never anything a caller passed as the command |
//! | `entering.rs` | the command | after writing itself into the box, in the same function |
//!
//! So the rule has two halves, one per file, and that is what is checked. The
//! failure being guarded against is a **third** site, or either of these two
//! starting the wrong thing — a road into a box that skipped joining it, which
//! would make everything else in this crate a decoration: a command running
//! outside the boundary while every type says it is inside one.
//!
//! # It is a source test, which is a weak kind, so it has a control
//!
//! A test that reads source and finds nothing has two indistinguishable
//! outcomes — nothing is there, or nothing was read. So this counts the files
//! it read and asserts it read them, and it runs its own matcher over lines
//! that **must** match. `alo-bounding`'s version of this had the same control
//! for the same reason, and this repository has written down twice what a null
//! result looks like when it is mistaken for a negative one.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{fs, path::Path};

/// The file that may become the command, after admitting itself.
const THE_ONE_THAT_EXECS: &str = "entering.rs";

/// What the admission writes, which has to be in the same file as the `exec`.
const THE_ADMISSION: &str = "THE_PROCESSES";

/// The file that may start the helper, and only the helper.
const THE_ONE_THAT_SPAWNS: &str = "a_run_inside_a_box.rs";

/// The only thing that file may start.
///
/// Spelled as the call, so that starting anything else — above all
/// `what.program()`, the command itself — is a different string and fails.
const ONLY_THE_HELPER: &str = "Command::new(helper)";

/// Which way of starting a program a line names, if it names one.
///
/// `alo-bounding`'s list, kept deliberately identical: the two tests are two
/// halves of one rule and a matcher that drifted would make them disagree about
/// what starting a program even is. The four letters of `exec` on their own are
/// in *execution*, which is what a turn does, so these are the spellings that
/// are a syscall.
fn starts_a_program(line: &str) -> Option<&'static str> {
    ["Command", "fork(", "exec(", "execv", "execl", "posix_spawn"]
        .into_iter()
        .find(|starting| line.contains(starting))
}

/// Whether a line says something to a reader rather than to the machine.
fn is_a_comment(line: &str) -> bool {
    line.trim_start().starts_with("//")
}

/// **The matcher matches what it is supposed to**, so that finding nothing
/// below means nothing is there.
#[test]
fn the_matcher_finds_a_program_being_started() {
    for named in [
        "let child = Command::new(\"sh\").spawn()?;",
        "std::process::Command::new(what).status()",
        "Command::new(program).args(arguments).exec()",
        "libc::fork()",
        "posix_spawn(&mut pid, path, ...)",
    ] {
        assert!(
            starts_a_program(named).is_some(),
            "the matcher does not recognise a program being started: {named}"
        );
    }

    // And does not call an ordinary line one, which is the other half: a
    // matcher that said yes to everything would pass the test above and fail
    // the whole crate.
    for ordinary in [
        "/// One approval, one execution, which is ADR 0001 §5.",
        "let executed = what_happened.the_command_was_happy();",
        "pub fn enter_the_box_and_become(",
    ] {
        assert!(
            starts_a_program(ordinary).is_none(),
            "an ordinary line was called a program being started: {ordinary}"
        );
    }
}

/// **Only `entering.rs` starts a program**, and it is the file that admits
/// first.
#[test]
fn only_the_file_that_admits_first_starts_a_program() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut read = 0_usize;
    let mut starting = Vec::new();

    for file in fs::read_dir(&source).expect("this crate has a src directory") {
        let file = file.expect("a directory entry can be read").path();
        if file.extension().is_none_or(|is| is != "rs") {
            continue;
        }
        let written = fs::read_to_string(&file).expect("a source file can be read");
        read += 1;

        let here: Vec<_> = written
            .lines()
            .filter(|line| !is_a_comment(line))
            .filter_map(starts_a_program)
            .collect();
        if !here.is_empty() {
            let name = file
                .file_name()
                .expect("a file that was read has a name")
                .to_string_lossy()
                .into_owned();

            if name == THE_ONE_THAT_EXECS {
                // The `exec` and the admission must be in the same file, or the
                // command is reachable without joining the box.
                assert!(
                    written.contains(THE_ADMISSION),
                    "{THE_ONE_THAT_EXECS} starts a program and does not name the admission, so \
                     the `exec` no longer has the control group write in front of it"
                );
            } else if name == THE_ONE_THAT_SPAWNS {
                // This one may start our own helper and nothing else. If it
                // ever started `what.program()` directly, the command would run
                // outside the box with no admission anywhere in front of it.
                assert!(
                    written.contains(ONLY_THE_HELPER),
                    "{THE_ONE_THAT_SPAWNS} starts a program and it is not {ONLY_THE_HELPER}"
                );
                let how_many = written.matches("Command::new(").count();
                assert_eq!(
                    how_many, 1,
                    "{THE_ONE_THAT_SPAWNS} starts {how_many} programs, and the only one it may \
                     start is the helper"
                );
            }
            starting.push(name);
        }
    }

    // The control: a run that read nothing would find nothing.
    assert!(
        read >= 5,
        "only {read} source files were read, and this crate has more than that — so finding \
         nothing would have meant nothing was looked at"
    );

    starting.sort();
    assert_eq!(
        starting,
        vec![
            THE_ONE_THAT_SPAWNS.to_owned(),
            THE_ONE_THAT_EXECS.to_owned()
        ],
        "two files in this crate may start a program — the one that starts the helper and the \
         one that becomes the command — and these do"
    );
}
