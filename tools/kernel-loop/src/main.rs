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
//! # It chooses the task; a person does the writing
//!
//! Each iteration reads `docs/autonomy/kernel-enforcement-plan.md`, takes the
//! **next task that is not done and whose dependencies are**, and says which
//! one it is. Then it waits for that task's work to appear as
//! `.kernel-loop/handoff.toml` — the files it touched, the commit it becomes,
//! the report it published — gates it, publishes it, and goes round for the
//! next one. A handoff naming a *different* task than the one selected is
//! refused, because a supervisor that took whatever it was given would be one
//! whose plan is decoration.
//!
//! **The writing is a person's.** That step cannot be automated by this program
//! and pretending otherwise would put a placeholder at the centre of the thing
//! that publishes. What the loop removes is everything around it: choosing what
//! is next, the pull, the gates, the rebase, the re-gate, the bounded retry,
//! and the log.
//!
//! So a run is: *select, wait, gate, publish, select…* until the plan has no
//! executable task left, nobody produces the work for the one selected, or
//! somebody stops it.
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
use std::time::{Duration, Instant};

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
                    break "asked to stop after publishing".to_owned();
                }
            }
            Ok(journal::Went::PlanIsFinished) => {
                // **Not "the workstream is complete".** Every task being done is
                // a list being empty, and the plan's own last section says what
                // that is and is not: implementation, with hardware acceptance
                // still owed and the v0.5 items still unbuilt.
                break "the plan has no executable task left, which is a list being empty                        rather than a workstream being finished"
                    .to_owned();
            }
            Ok(journal::Went::NobodyWroteIt(task)) => {
                break format!("nobody handed over the work for `{task}`");
            }
            Ok(journal::Went::Stopped) => break "asked to stop".to_owned(),
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

/// How long the loop waits for a selected task's work before it stops.
///
/// Bounded on purpose. A supervisor that waited forever would sit on a lock
/// nobody could see the reason for; one that gave up in a minute would be
/// useless for work that takes an afternoon. When it runs out the loop stops
/// and says which task it was waiting for, which is a thing a person can act
/// on.
const WAITING_AT_MOST: Duration = Duration::from_secs(60 * 60);

/// How often it looks for the work while waiting, and how often it notices a
/// stop.
const LOOKING_EVERY: Duration = Duration::from_secs(10);

/// One task: choose it, wait for its work, check the checkout, gate, publish.
///
/// Every road out of this either published one task or explains why it did not.
/// Nothing here removes a file, resets a branch or discards a change: a
/// supervisor that tidied up would be one that can throw work away.
fn one_iteration(at: &Path, ours: &Path) -> Result<journal::Went, String> {
    if journal::was_asked_to_stop(ours) {
        return Ok(journal::Went::Stopped);
    }

    let Some(chosen) = plan::next_executable(at)? else {
        return Ok(journal::Went::PlanIsFinished);
    };
    journal::note(
        ours,
        &format!("next in the plan: {}. {}", chosen.number, chosen.named),
    );

    let Some(task) = waiting_for(ours, &chosen.named)? else {
        return Ok(journal::Went::NobodyWroteIt(chosen.named));
    };
    journal::note(ours, &format!("taking up: {}", task.task));

    repository::on_main_and_clean_but_for(at, &task.files)?;
    repository::pulled(at)?;

    publishing::gated_and_pushed(at, ours, &task)
        .map(|sha| journal::Went::Published(sha, task.task.clone()))
}

/// The handoff for the task that was chosen, once somebody writes it.
///
/// A handoff for a **different** task is refused rather than published: the
/// plan's order is the plan's, and a supervisor that published whatever it was
/// handed would be one whose choosing meant nothing.
///
/// # Errors
/// A sentence when a handoff is there and names another task, or is missing
/// something a commit needs.
fn waiting_for(ours: &Path, chosen: &str) -> Result<Option<handoff::Handed>, String> {
    let until = Instant::now() + WAITING_AT_MOST;
    let mut said_so = false;
    loop {
        if journal::was_asked_to_stop(ours) {
            return Ok(None);
        }
        if let Some(handed) = handoff::Handed::waiting(ours)? {
            if handed.task != chosen {
                return Err(format!(
                    "the handoff is for `{}` and the plan's next task is `{chosen}`. Nothing was \
                     published: either finish the chosen task, or mark it done in the plan if it \
                     is.",
                    handed.task
                ));
            }
            return Ok(Some(handed));
        }
        if !said_so {
            journal::note(ours, "waiting for its work to be handed over");
            said_so = true;
        }
        if Instant::now() >= until {
            return Ok(None);
        }
        std::thread::sleep(LOOKING_EVERY);
    }
}
