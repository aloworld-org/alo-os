//! The kernel-enforcement workstream's supervisor: one completed task at a
//! time, gated, published, and written down.
//!
//! # What this does, and the thing it deliberately does not
//!
//! It **verifies and publishes**. It runs the gates the repository requires,
//! commits exactly the files a task names, integrates whatever arrived on
//! `main` while the work was being done, reruns the gates on the combined tree,
//! pushes, and records what happened.
//!
//! **It does not write code.** A supervisor that claimed to would be a
//! placeholder with a loop around it, and `CLAUDE.md` has a word for that. The
//! implementing is done by the contributor between iterations; what this
//! removes is the part that is mechanical, forgettable and dangerous to get
//! wrong — the pull, the rebase, the rerun, the bounded retry, the log.
//!
//! That is the same division `tools/dev-loop` has, and this is a second program
//! rather than a change to it: the compositor worker's supervisor must keep
//! running whatever happens here, and two workstreams sharing one supervisor
//! would be two workstreams sharing one lock.
//!
//! # How a task reaches it
//!
//! A task is handed over by writing `.kernel-loop/handoff.toml`: the files it
//! touched, the commit subject and body, and the report it published. An
//! iteration with no handoff has nothing to publish and says so rather than
//! inventing work.
//!
//! # Stopping
//!
//! `stop` writes a file. The loop finishes what it is doing — it never abandons
//! a half-published task — and does not begin another. Nothing is discarded and
//! nothing is rolled back, because a supervisor that tidied up after itself
//! would be a supervisor that can throw work away.

mod gates;
mod handoff;
mod journal;
mod lock;
mod plan;
mod publishing;
mod repository;

use std::path::Path;
use std::process::ExitCode;

/// Where the loop keeps its lock, its log and the file that stops it.
const ITS_OWN: &str = ".kernel-loop";

/// What the loop was asked to do.
enum Asked {
    /// Run iterations until there is nothing to publish or somebody stops it.
    Run,

    /// Say what is happening, and what happened last.
    Status,

    /// Ask the running loop to finish and not begin again.
    Stop,
}

impl Asked {
    /// What the arguments say, or [`None`] if they say nothing this understands.
    fn from(mut args: impl Iterator<Item = String>) -> Option<Self> {
        let _ = args.next();
        match args.next().as_deref() {
            Some("run") => Some(Self::Run),
            Some("status") => Some(Self::Status),
            Some("stop") => Some(Self::Stop),
            _ => None,
        }
    }
}

fn main() -> ExitCode {
    let Some(asked) = Asked::from(std::env::args()) else {
        eprintln!(
            "alo-kernel-loop — the kernel-enforcement workstream's supervisor\n\
             \n\
             usage:\n\
             \x20 alo-kernel-loop run      gate and publish ready tasks until told to stop\n\
             \x20 alo-kernel-loop status   what is happening, and what happened last\n\
             \x20 alo-kernel-loop stop     finish the current task and begin no other\n"
        );
        return ExitCode::FAILURE;
    };

    let at = match std::env::current_dir() {
        Ok(here) => here,
        Err(why) => {
            eprintln!("alo-kernel-loop: it cannot tell where it is being run: {why}");
            return ExitCode::FAILURE;
        }
    };
    let ours = at.join(ITS_OWN);

    match asked {
        Asked::Run => run(&at, &ours),
        Asked::Status => match journal::said(&ours) {
            Ok(said) => {
                println!("{said}");
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("alo-kernel-loop: {why}");
                ExitCode::FAILURE
            }
        },
        Asked::Stop => match journal::asked_to_stop(&ours) {
            Ok(()) => {
                println!(
                    "alo-kernel-loop: asked to stop. It finishes what it is doing and begins \
                     nothing else; nothing is discarded."
                );
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("alo-kernel-loop: it could not be asked to stop: {why}");
                ExitCode::FAILURE
            }
        },
    }
}

/// Iterations, until there is nothing to publish or somebody stops it.
fn run(at: &Path, ours: &Path) -> ExitCode {
    if let Err(why) = std::fs::create_dir_all(ours) {
        eprintln!("alo-kernel-loop: it has nowhere to keep its own files: {why}");
        return ExitCode::FAILURE;
    }

    // **One at a time, and the lock is the whole of that.** Two supervisors in
    // one checkout are two editors, which `CLAUDE.md` forbids outright: the
    // second would rebase over the first's half-made commit.
    let held = match lock::Held::taken(ours) {
        Ok(held) => held,
        Err(why) => {
            eprintln!("alo-kernel-loop: {why}");
            return ExitCode::FAILURE;
        }
    };

    // A stop asked for before this run began is not this run's to obey — it
    // belonged to the loop that has already finished. Cleared here so that
    // `stop` always means *the loop that is running now*.
    journal::the_stop_is_cleared(ours);
    journal::note(ours, "the loop began");

    let ending = loop {
        match one_iteration(at, ours) {
            Ok(journal::Went::Published(sha, task)) => {
                journal::note(ours, &format!("published {sha} — {task}"));
                if journal::was_asked_to_stop(ours) {
                    break "asked to stop after publishing";
                }
            }
            Ok(journal::Went::NothingReady) => break "nothing is handed over to publish",
            Err(why) => {
                journal::note(ours, &format!("STOPPED: {why}"));
                eprintln!("alo-kernel-loop: {why}");
                drop(held);
                return ExitCode::FAILURE;
            }
        }
    };

    journal::note(ours, &format!("the loop ended: {ending}"));
    println!("alo-kernel-loop: {ending}.");
    drop(held);
    ExitCode::SUCCESS
}

/// One task: check the checkout, take what was handed over, gate it, publish it.
///
/// Every road out of this either published one task or explains why it did not.
/// Nothing here removes a file, resets a branch or discards a change: a
/// supervisor that tidied up would be one that can throw work away.
fn one_iteration(at: &Path, ours: &Path) -> Result<journal::Went, String> {
    if journal::was_asked_to_stop(ours) {
        return Ok(journal::Went::NothingReady);
    }

    let Some(task) = handoff::Handed::waiting(ours)? else {
        return Ok(journal::Went::NothingReady);
    };
    journal::note(ours, &format!("taking up: {}", task.task));

    // The plan is read for its own sake: a task that names one this file does
    // not know about is a task nobody planned, and that is worth refusing over
    // rather than publishing quietly.
    plan::names_this_task(at, &task.task)?;

    repository::on_main_and_clean_but_for(at, &task.files)?;
    repository::pulled(at)?;

    publishing::gated_and_pushed(at, ours, &task)
        .map(|sha| journal::Went::Published(sha, task.task.clone()))
}
