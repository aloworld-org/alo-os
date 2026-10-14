//! **Two binaries read the keyboard this machine has. Something must write
//! it** — and on 2026-10-14 nothing did, for a day, with every test green.
//!
//! # What happened
//!
//! `THE_MACHINES_KEYBOARDS` is this crate's name for
//! `/etc/alo/keyboards.toml`. #668 taught the greeter to read it, #683 built
//! the language sheet that writes it, and **#687 deleted the language sheet** —
//! a pull request about a keyboard chord, merged from a branch cut before #683
//! landed and never rebased. It held older copies of files it had never
//! edited, so merging it removed them: six whole files and roughly 480 lines
//! inside twenty more, 2248 deletions in total, **and not one conflict.**
//!
//! Git merged it cleanly because a stale whole-file copy conflicts with
//! nothing. No test failed, because the tests for the deleted screen were
//! deleted with it. The readers survived and went on reading a file that
//! nothing on the machine could ever write again — so every machine fell back
//! to the layout named in its unit file, which is the fault #683 existed to
//! end.
//!
//! # Why the assertion is a pair
//!
//! A test asserting *the language sheet exists* would be a test about a
//! filename, and it would fail on the day somebody renames it for a good
//! reason. A test asserting *something writes this file* would pass before
//! #683 was ever written, when nothing read it either and nothing needed to.
//!
//! The fault is the **pair**: something reads this file and nothing writes it.
//! Either half alone is fine:
//!
//! - nothing reads it and nothing writes it — **a machine that has not grown
//!   the feature yet, correct**;
//! - something reads it and something writes it — **the goal, correct**;
//! - something reads it and nothing writes it — **refused**, and this is what
//!   main held on 2026-10-14;
//! - nothing reads it and something writes it — not this test's business.
//!
//! This test lives in `alo-keyboards` rather than in `alo-shell` on purpose.
//! The screen, its tests and the shell's module list were all deleted together
//! by one merge; a guard inside that blast radius would have gone with them.
//! This crate owns the constant and was not touched.
//!
//! # What this walk looks at
//!
//! Every `.rs` under `crates/*/src` that is not a `_tests.rs` module. **Not
//! `tools/`**, which is its own workspace and nothing in it reads this
//! file today — so a reader appearing there would not count toward the
//! control below, and the control is the half that would weaken. Said here
//! because a green run tells the next reader nothing about what was read.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The repository root, from this crate's own manifest directory.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("the repository root")
        .to_path_buf()
}

/// The call that answers *which keyboard does this machine have*.
///
/// A file that makes this call is a **reader**: it consumes the answer. Three
/// do today: the module that gives it, the shell's booting path, and the greeter
/// binary that fills in the machine it stands on.
const READS_IT: &str = "the_keyboard_this_machine_has";

/// The call that writes a keyboards file at all.
///
/// **The function name alone, not `keeping::keep(`.** A writer doing
/// `use alo_keyboards::keeping::keep;` and then `keep(&path, ..)` would match
/// the qualified spelling nowhere, and this test would then fail *while a
/// writer existed* — which is worse than not having the test, because the
/// sentence below would send somebody looking for a deleted file that was
/// never deleted.
///
/// Looser about spelling and no looser about intent: it is
/// [`THE_MACHINE_WIDE_FILE`] in the same file that makes a keeper a keeper of
/// *this* file. Measured rather than assumed — of the seven production
/// files naming that constant, exactly one contains a `keep(` call, and it is
/// the right one.
const WRITES_A_KEYBOARDS_FILE: &str = "keep(";

/// This crate's name for the machine-wide file, as source code spells it.
const THE_MACHINE_WIDE_FILE: &str = "THE_MACHINES_KEYBOARDS";

/// Every production source file in the workspace.
///
/// Test modules and `tests/` directories are left out: a test writing this
/// file is a test choosing its own fixture, which is the right thing for it to
/// do and no evidence that a machine can.
fn every_production_file(repository: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let crates = std::fs::read_dir(repository.join("crates")).expect("crates/");
    for entry in crates {
        walk(
            &entry.expect("a crate directory").path().join("src"),
            &mut found,
        );
    }
    found.sort();
    found
}

/// Every `.rs` file under a directory that is not a test module.
fn walk(at: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            walk(&path, into);
        } else if path.extension().is_some_and(|it| it == "rs")
            && !path
                .file_name()
                .is_some_and(|it| it.to_string_lossy().ends_with("_tests.rs"))
        {
            into.push(path);
        }
    }
}

#[test]
fn something_on_this_machine_writes_the_keyboard_its_readers_read() {
    let repository = the_repository();
    let mut readers: Vec<String> = Vec::new();
    let mut writers: Vec<String> = Vec::new();

    for file in every_production_file(&repository) {
        let text = std::fs::read_to_string(&file).expect("a file this workspace builds");
        let named = file
            .strip_prefix(&repository)
            .unwrap_or(&file)
            .display()
            .to_string();
        let reads = text.contains(READS_IT);
        if reads {
            readers.push(named.clone());
        }
        // **A writer is a file that keeps a keyboards file and names the
        // machine-wide one, and is not itself a reader.** The last clause is
        // what keeps `alo-desktop` out: it keeps this person's own keyboards
        // and it names the machine-wide file to read it, which makes it a
        // consumer twice over and a writer of this file not at all.
        if !reads && text.contains(WRITES_A_KEYBOARDS_FILE) && text.contains(THE_MACHINE_WIDE_FILE)
        {
            writers.push(named);
        }
    }

    // **The positive control.** A directory walk that finds nothing satisfies
    // every assertion below it. Three files read this answer today — the module
    // that gives it and the two binaries that take it — so a count under two
    // means this test has stopped reading the repository rather than started
    // approving of it.
    assert!(
        readers.len() >= 2,
        "this test found {} files reading `{READS_IT}`, and there are three. Either it was \
         renamed or this walk has stopped finding the workspace — and a walk that finds nothing \
         passes everything. Point it at them again before trusting a green run.\n  found: {:?}",
        readers.len(),
        readers
    );

    assert!(
        !writers.is_empty(),
        "{} files read the keyboard this machine has:\n  {}\n\n\
         and **nothing writes it**. So `{}` can never exist on a machine, every reader falls \
         back to the layout its unit file names, and a person who chooses a language at the \
         sign-in screen is handed somebody else's keyboard — which is the whole fault #683 \
         existed to end.\n\n\
         This is what `main` held for a day after #687 deleted the language sheet from a stale \
         branch, with no conflict and no failing test. If the screen was renamed or moved, point \
         this test at what writes the file now. **If it was deleted, it was deleted by accident: \
         run `git log --diff-filter=D — crates/alo-shell/src/the_keyboard_a_person_chose.rs` and \
         bring it back.**",
        readers.len(),
        readers.join("\n  "),
        alo_keyboards::THE_MACHINES_KEYBOARDS
    );
}
