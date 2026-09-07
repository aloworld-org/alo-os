//! The plan, read for the two questions the loop asks it: **which task is next**
//! and **is this a task somebody planned?**
//!
//! The loop does not invent work — a supervisor writing its own scope is a
//! supervisor with no scope. What it does is read
//! `docs/autonomy/kernel-enforcement-plan.md`, which a person wrote, and take
//! the next task that is not done and whose dependencies are.
//!
//! # What the plan has to say for this to work
//!
//! ```text
//! ### 3. Socket attribution and default-deny for a bound turn
//!
//! **Status:** ready. **Depends on:** 1, 2 — both done.
//! ```
//!
//! A task is a `###` heading beginning with a number. It is **done** when its
//! section contains a line beginning `**Done,`. It **depends on** the numbers on
//! its `**Depends on:**` line, and `nothing` there means it depends on nothing.
//!
//! Nothing here writes to the plan. A task is marked done by the person who did
//! it, in the same change that did it, because *done* is a judgement about
//! evidence and this program has no way to make one.

use std::path::Path;

use crate::repository;

/// Where this workstream's plan lives.
const THE_PLAN: &str = "docs/autonomy/kernel-enforcement-plan.md";

/// One task, as the plan has it.
#[derive(Debug, Clone)]
pub struct Task {
    /// Its number, which is what other tasks depend on.
    pub number: u32,

    /// Its descriptive name, which is what a handoff names and a report is
    /// filed under.
    pub named: String,

    /// Whether the plan says it is finished.
    pub done: bool,

    /// The numbers of the tasks it cannot start before.
    pub after: Vec<u32>,
}

/// Every task the plan names, in the order it names them.
///
/// # Why the **published** plan and not the one on the disk
///
/// A task is marked done in the change that does it, which is right — a plan
/// that lagged its own work would be a plan nobody could trust. But it means
/// the copy in the working tree already says *done* while the task is still
/// being handed over, so a loop reading that copy would skip straight past the
/// task it is about to publish and choose the one after.
///
/// So this reads the plan as `HEAD` has it: what the *last published* state
/// says is next. Found by running the loop, which chose task 4 while task 3 sat
/// in the working tree waiting to be published.
///
/// # Errors
/// A sentence when the plan cannot be read out of the commit.
pub fn every_task(at: &Path) -> Result<Vec<Task>, String> {
    let written = repository::git(at, &["show", &format!("HEAD:{THE_PLAN}")])
        .map_err(|why| format!("the published plan could not be read: {why}"))?;

    let mut tasks: Vec<Task> = Vec::new();
    for line in written.lines() {
        if let Some(heading) = line.strip_prefix("### ") {
            // The number is what makes it a task. The plan's audit sections are
            // headings at the same depth, and without this the loop would take
            // one of them for work.
            if let Some((number, named)) = heading.split_once(". ")
                && let Ok(number) = number.parse::<u32>()
            {
                tasks.push(Task {
                    number,
                    named: named.trim().to_owned(),
                    done: false,
                    after: Vec::new(),
                });
            }
            continue;
        }
        let Some(current) = tasks.last_mut() else {
            continue;
        };
        if line.starts_with("**Done,") {
            current.done = true;
        }
        if let Some(after) = line.split("**Depends on:**").nth(1) {
            current.after = numbers_in(after);
        }
    }
    Ok(tasks)
}

/// The task numbers written in a fragment of a line.
///
/// `1, 2 — both done.` is one and two; `nothing.` is none. Read by taking runs
/// of digits, so the prose around them does not have to be parsed.
fn numbers_in(written: &str) -> Vec<u32> {
    let mut found = Vec::new();
    let mut digits = String::new();
    for letter in written.chars().chain(std::iter::once(' ')) {
        if letter.is_ascii_digit() {
            digits.push(letter);
            continue;
        }
        if !digits.is_empty()
            && let Ok(number) = digits.parse()
        {
            found.push(number);
        }
        digits.clear();
    }
    found
}

/// The next task that can be worked on, or [`None`] when none can.
///
/// The first that is not done and whose dependencies are all done. **Not the
/// first that is not done**: a task waiting on one that is blocked is not
/// executable, and taking it up would be the loop deciding the plan's order was
/// advice.
///
/// # Errors
/// A sentence when the plan cannot be read.
pub fn next_executable(at: &Path) -> Result<Option<Task>, String> {
    let tasks = every_task(at)?;
    let finished = |number: u32| tasks.iter().any(|task| task.number == number && task.done);
    Ok(tasks
        .iter()
        .find(|task| !task.done && task.after.iter().copied().all(finished))
        .cloned())
}
