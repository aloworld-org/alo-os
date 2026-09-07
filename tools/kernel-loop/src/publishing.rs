//! Gate, commit, integrate what arrived, gate the combination, push.
//!
//! # The second gate is the one that matters
//!
//! Gating before the rebase says this task works. Gating **after** it says this
//! task works *beside whatever the other worker published while it was being
//! written*, and only the second is a claim worth making about `main`. Two
//! changes that each pass alone and fail together is not a hypothetical; it is
//! the ordinary way a shared branch breaks.
//!
//! # Gates, and then the task's own evidence
//!
//! The gates say the repository still works. They cannot say this task was
//! done: the suite they pass is the one that was already there. So each gating
//! is followed by `crate::evidence`, which runs the test behind each acceptance
//! criterion on its own and refuses a task whose evidence is not part of the
//! change it is publishing. **Passing existing tests is never taken as
//! completed implementation.**
//!
//! # Bounded retries, and what a lost race is
//!
//! A push that loses to somebody else's is not a failure, it is a race, and the
//! answer is to integrate again and retry. Three times, and then it stops and
//! says so: a loop that retried forever against a branch somebody is pushing to
//! every minute would never publish and never complain.

use std::path::Path;

use crate::{evidence, gates, handoff::Handed, journal, repository};

/// How many times a lost race is worth answering before somebody should look.
const TIMES: u8 = 3;

/// One task, all the way onto `main`.
///
/// # Errors
/// A sentence naming what stopped it, at whichever step. **Work is never
/// discarded on any of these roads**: a failed gate leaves the tree as it was,
/// a conflicted rebase is left where it stopped, and a lost race leaves the
/// commit sitting locally for the next attempt.
pub fn gated_and_pushed(at: &Path, ours: &Path, task: &Handed) -> Result<String, String> {
    journal::note(
        ours,
        &format!("running the gates; the report is {}", task.report),
    );
    checked(at, ours, task, "this task's tree")?;

    repository::staged(at, &task.files)?;
    let sha = repository::committed(at, &task.message())?;
    journal::note(ours, &format!("committed {sha} locally"));

    for attempt in 1..=TIMES {
        if repository::advanced(at)? {
            journal::note(
                ours,
                &format!(
                    "main advanced; rebasing and gating the combined tree (attempt {attempt})"
                ),
            );
            repository::rebased_onto_origin(at)?;
            checked(at, ours, task, "the combined tree")?;
        }

        match repository::pushed(at) {
            Ok(()) => {
                let published = repository::git(at, &["rev-parse", "--short", "HEAD"])?;
                Handed::put_away(ours, &published)?;
                return Ok(published);
            }
            Err(why) if attempt < TIMES => {
                journal::note(
                    ours,
                    &format!("the push did not go through, integrating and trying again: {why}"),
                );
            }
            Err(why) => {
                return Err(format!(
                    "the push did not go through after {TIMES} attempts, and the commit is \
                     sitting in this checkout unpublished — nothing was discarded: {why}"
                ));
            }
        }
    }

    Err(format!(
        "publication was not reached in {TIMES} attempts; the work is committed locally and \
         intact"
    ))
}

/// Every gate, and then the task's own acceptance evidence.
///
/// In that order and never one without the other. The gates are cheap to fail
/// and are about the repository; the evidence is about the task, and running it
/// on a tree that had not passed the gates would say nothing either way.
///
/// # Errors
/// Whatever stopped it, named. Nothing is staged, committed or pushed on any of
/// these roads.
fn checked(at: &Path, ours: &Path, task: &Handed, which: &str) -> Result<(), String> {
    let passed = gates::all_of_them(at)?;
    journal::note(ours, &format!("{which} passed: {}", passed.join("; ")));
    let stood = evidence::stands_up(at, &task.files, &task.evidence)?;
    journal::note(
        ours,
        &format!("the evidence stood up: {}", stood.join("; ")),
    );
    Ok(())
}
