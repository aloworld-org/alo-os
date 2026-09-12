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
//! A task is a `###` heading beginning with a number **under the plan's
//! `## Tasks`**. It is **done** when its section contains a line beginning
//! `**Done,`. It **depends on** the numbers on its `**Depends on:**` line, and
//! `nothing` there means it depends on nothing.
//!
//! # Why the section is read, and not the depth alone
//!
//! The plan's audit is `### 1. Implemented and verified` and four more like it:
//! same depth, numbered from one. An earlier version took the number for what
//! made a heading a task, on the belief that the audit's headings had none — so
//! the first thing this loop ever selected on its own was an audit section, and
//! it launched a worker at *Implemented and verified*. Found by running it.
//!
//! The numbers collide as well as the shape: audit `1` and task `1` are
//! different sections, and a `**Depends on:** 1` answered by the wrong one is a
//! dependency satisfied by something that is not a task. So the boundary is the
//! section, which is what a person reading the plan uses too.
//!
//! Nothing here writes to the plan. A task is marked done by the person who did
//! it, in the same change that did it, because *done* is a judgement about
//! evidence and this program has no way to make one.

use std::collections::BTreeSet;
use std::path::Path;

use crate::repository;

/// Where a workstream's plan lives, unless one is named.
///
/// The kernel-enforcement plan is the default because it is the one this
/// supervisor was written for and the one every task so far came from. It stops
/// being the only one the day a second workstream has a plan in this shape —
/// which is [`THE_PLAN_NAMED`].
const THE_PLAN: &str = "docs/autonomy/kernel-enforcement-plan.md";

/// What names a different plan for this run.
///
/// **A supervisor that can only drive one workstream is a supervisor that gets
/// copied.** The gates, the evidence rule, the lock, the rebase and the
/// bounded retry are none of them about kernel enforcement; the only thing
/// that was is the file the tasks are read from. So it is an input rather than
/// a constant, and the two plans share every safeguard instead of one of them
/// inheriting a stale copy.
///
/// An environment variable rather than an argument, so that `run`, `verify` and
/// `publish` all see the same answer without each subcommand having to be
/// taught to pass it along.
const THE_PLAN_NAMED: &str = "ALO_LOOP_PLAN";

/// The plan this run reads.
///
/// # Errors
/// A sentence when the name is set and empty, which is somebody meaning to
/// select a plan and selecting nothing — silently falling back to the default
/// would run the wrong workstream's tasks under their intention.
pub fn the_plan() -> Result<String, String> {
    match std::env::var(THE_PLAN_NAMED) {
        Err(_) => Ok(THE_PLAN.to_owned()),
        Ok(named) if named.trim().is_empty() => Err(format!(
            "{THE_PLAN_NAMED} is set and empty, so no plan was selected and the default was not \
             meant either. Name a plan, or unset it to read {THE_PLAN}."
        )),
        Ok(named) => Ok(named.trim().to_owned()),
    }
}

/// The one section of it that holds work.
const THE_TASKS: &str = "Tasks";

/// Every `**Status:**` word that means *a person has to arrange something
/// first*.
///
/// **`scheduled` is one of them, and that is not a synonym for blocked.** A
/// blocked task waits on a decision; a scheduled one waits on a moment — a real
/// session to log out of, a maintenance window on a shared machine. Neither is
/// work a supervisor can start on its own, and a loop that took one up would be
/// arranging the thing rather than doing the task: changing sessions or
/// lingering on a machine somebody else may be testing on.
const NOT_YET: [&str; 2] = ["blocked", "scheduled"];

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

    /// Whether the plan says it is blocked, which is not the same as unfinished.
    ///
    /// A task waiting on a decision somebody has to make is not work a
    /// supervisor can start, and a loop that took it up would launch a worker
    /// at a question rather than at a task.
    pub blocked: bool,

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
    let plan = the_plan()?;
    let written = repository::git(at, &["show", &format!("HEAD:{plan}")])
        .map_err(|why| format!("the published plan `{plan}` could not be read: {why}"))?;
    Ok(read(&written))
}

/// The task the plan numbers so, or [`None`] when it numbers none that way.
///
/// For the one place a number is all there is to go on: a parked branch is
/// named for its task's **number**, and a handoff for its **name**, and
/// recovering a branch that carries no handoff has to get from the first to
/// the second. The plan is the only thing that knows both.
///
/// # Errors
/// A sentence when the plan cannot be read.
pub fn numbered(at: &Path, number: u32) -> Result<Option<Task>, String> {
    Ok(every_task(at)?
        .into_iter()
        .find(|task| task.number == number))
}

/// Every task in a plan that has already been read.
///
/// Separated from the commit it comes out of so that what this program believes
/// a plan says is a thing tests can ask it, rather than a thing only a real
/// repository with a real history could show. The audit heading it once took
/// for work is one line of markdown; proving it no longer does should not need
/// a commit.
fn read(written: &str) -> Vec<Task> {
    let mut tasks: Vec<Task> = Vec::new();
    let mut in_the_tasks = false;
    for line in written.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            // The plan's sections. Only one of them holds work, and every other
            // `##` closes it — including the *Rules* and *Completion* that come
            // after the last task.
            in_the_tasks = heading.trim() == THE_TASKS;
            continue;
        }
        if !in_the_tasks {
            continue;
        }
        if let Some(heading) = line.strip_prefix("### ") {
            if let Some((number, named)) = heading.split_once(". ")
                && let Ok(number) = number.parse::<u32>()
            {
                tasks.push(Task {
                    number,
                    named: named.trim().to_owned(),
                    done: false,
                    blocked: false,
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
        if line.starts_with("**Status:**") && NOT_YET.iter().any(|word| line.contains(word)) {
            current.blocked = true;
        }
        if let Some(after) = line.split("**Depends on:**").nth(1) {
            current.after = numbers_in(after);
        }
    }
    tasks
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
/// The first that is not done, not blocked, and whose dependencies are all
/// done. **Not simply the first that is not done**: a task waiting on another,
/// or on a decision somebody has to make, is not executable, and taking it up
/// would be the loop deciding the plan's order was advice.
///
/// A blocked task is **stepped over rather than stopped at**, so one question
/// awaiting an answer does not hold up work that has none.
///
/// # Errors
/// A sentence when the plan cannot be read.
/// `given_up_on` are tasks this run has already failed at. They are stepped
/// over rather than chosen again, so one task nobody can finish does not stop
/// every task somebody could.
pub fn next_executable(at: &Path, given_up_on: &BTreeSet<u32>) -> Result<Option<Task>, String> {
    Ok(the_next_of(&every_task(at)?, given_up_on))
}

/// The same choice, made from tasks already read.
///
/// **A task given up on is not treated as done.** Its dependants stay unstartable
/// — that is the point of a dependency, and a loop that pushed past one would
/// build the second floor of a house whose first floor it had abandoned.
fn the_next_of(tasks: &[Task], given_up_on: &BTreeSet<u32>) -> Option<Task> {
    let finished = |number: u32| tasks.iter().any(|task| task.number == number && task.done);
    tasks
        .iter()
        .find(|task| {
            !task.done
                && !task.blocked
                && !given_up_on.contains(&task.number)
                && task.after.iter().copied().all(finished)
        })
        .cloned()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[expect(
    clippy::panic,
    reason = "a plan that cannot be opened names itself in the failure, which an unwrap could not"
)]
mod tests {
    use super::*;

    /// A plan shaped the way this workstream's is: an audit whose headings are
    /// numbered from one at the same depth as the tasks, and then the tasks.
    const A_PLAN: &str = "\
# A plan

## Audit — five sections

### 1. Implemented and verified

Some prose about what is already true.

### 2. Remaining gaps

More prose.

## Tasks

### 1. Something that is finished

**Status:** ready. **Depends on:** nothing.

**Done, 2026-09-08.** It is.

### 2. Something waiting on a moment

**Status:** scheduled. **Depends on:** nothing.

### 3. Something waiting on an answer

**Status:** blocked. **Depends on:** nothing.

### 4. Something that can be worked on

**Status:** ready. **Depends on:** 1 — done.

### 5. Something that cannot start yet

**Status:** ready. **Depends on:** 4.

## Rules this workstream holds itself to

### 1. Not a task either
";

    /// **The audit is not work.** Its headings are numbered and at the same
    /// depth as the tasks, and the first thing this loop ever chose on its own
    /// was one of them — it launched a worker at *Implemented and verified*.
    #[test]
    fn the_audits_numbered_headings_are_not_tasks() {
        let tasks = read(A_PLAN);
        assert!(
            !tasks
                .iter()
                .any(|task| task.named == "Implemented and verified"),
            "an audit heading was read as a task: {:?}",
            tasks.iter().map(|task| &task.named).collect::<Vec<_>>()
        );
        assert_eq!(tasks.len(), 5, "{tasks:?}");
        assert_eq!(
            tasks.first().unwrap().named,
            "Something that is finished",
            "the first task is not the first heading under `## Tasks`"
        );
    }

    /// **And neither is anything after the tasks.** The plan's rules and
    /// completion sections come last and are headed the same way.
    #[test]
    fn a_heading_after_the_tasks_is_not_a_task() {
        assert!(
            !read(A_PLAN)
                .iter()
                .any(|task| task.named == "Not a task either")
        );
    }

    /// **A scheduled task is stepped over, like a blocked one.** It waits on a
    /// moment somebody has to arrange — a real session, a maintenance window on
    /// a shared machine — and a loop that took it up would arrange the thing
    /// rather than do the task.
    #[test]
    fn a_scheduled_task_is_not_work_a_loop_may_start() {
        let tasks = read(A_PLAN);
        let scheduled = tasks.iter().find(|task| task.number == 2).unwrap();
        assert!(scheduled.blocked, "a scheduled task was offered as work");
        let waiting = tasks.iter().find(|task| task.number == 3).unwrap();
        assert!(waiting.blocked);
    }

    /// The next task is the first that is not done, not waiting on a moment or
    /// an answer, and whose dependencies are done — stepping over the two in
    /// between rather than stopping at them.
    #[test]
    fn the_next_task_steps_over_what_is_waiting() {
        let chosen = the_next_of(&read(A_PLAN), &BTreeSet::new()).unwrap();
        assert_eq!(chosen.number, 4);
        assert_eq!(chosen.named, "Something that can be worked on");
    }

    /// **A task this run has already failed at is stepped over**, so one task
    /// nobody can finish does not stop every task somebody could.
    ///
    /// The whole point of an unattended run: without this the loop chose task
    /// four, failed, and ended — and did so again on every restart, while tasks
    /// it could have finished sat untouched.
    #[test]
    fn a_task_already_given_up_on_this_run_is_not_chosen_again() {
        let tasks = read(A_PLAN);
        let given_up_on = BTreeSet::from([4]);

        // Four is the one it would otherwise take; five depends on four, which
        // is not done, so there is nothing left after it.
        assert!(the_next_of(&tasks, &given_up_on).is_none());
    }

    /// **Giving up on a task is not finishing it.** Its dependants stay
    /// unstartable, because a loop that pushed past an abandoned dependency
    /// would build on something nobody built.
    #[test]
    fn giving_up_on_a_task_does_not_release_what_waits_on_it() {
        let tasks = read(A_PLAN);
        let after_four = tasks.iter().find(|task| task.number == 5).unwrap();
        assert_eq!(after_four.after, vec![4]);

        // Even with four abandoned, five is not offered — it is waiting on work
        // that never happened rather than on work that did.
        let chosen = the_next_of(&tasks, &BTreeSet::from([4]));
        assert!(chosen.is_none_or(|task| task.number != 5));
    }

    /// A task whose dependency is unfinished is not chosen, even when nothing
    /// else is left.
    #[test]
    fn a_task_waiting_on_another_is_not_chosen() {
        let only_the_last: Vec<Task> = read(A_PLAN)
            .into_iter()
            .filter(|task| task.number == 5)
            .collect();
        assert!(the_next_of(&only_the_last, &BTreeSet::new()).is_none());
    }

    /// **Every plan this repository asks the loop to drive reads as tasks**,
    /// which is the file the loop really opens. A synthetic plan proves the
    /// rule; this proves the rule is about the plans we have.
    ///
    /// Both are read, not just the default. A second workstream's plan that
    /// parsed to nothing would send a worker at *nothing to do* and read as the
    /// work being finished — the same failure the audit-heading bug had, from
    /// the other end.
    #[test]
    fn every_plan_this_repository_drives_holds_only_tasks() {
        for named in [
            THE_PLAN,
            "docs/autonomy/v0-01-delivery-plan.md",
            "docs/autonomy/v0-01-lane-b-plan.md",
            "docs/autonomy/v0-5-lane-b-plan.md",
        ] {
            let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(named);
            let written = std::fs::read_to_string(&at)
                .unwrap_or_else(|why| panic!("`{named}` is where it says it is: {why}"));
            let tasks = read(&written);

            assert!(
                tasks.len() > 1,
                "`{named}` read as {} tasks, so the loop would have nothing to do and would say \
                 the workstream was finished",
                tasks.len()
            );
            for (which, task) in tasks.iter().enumerate() {
                assert_eq!(
                    task.number,
                    u32::try_from(which).unwrap() + 1,
                    "`{named}` does not number its tasks from one in order: {task:?}"
                );
            }
        }
    }

    /// **A plan can be named, and an empty name is refused rather than quietly
    /// meaning the default.**
    ///
    /// Somebody who sets the variable meant to select a plan. Falling back
    /// would run one workstream's tasks under the intention of another's, which
    /// is worse than stopping.
    #[test]
    fn naming_no_plan_is_refused_rather_than_meaning_the_default() {
        // The variable is read from the environment, so this asks the decision
        // rather than the process: setting a variable inside a test would race
        // every other test in this binary.
        assert_eq!(THE_PLAN_NAMED, "ALO_LOOP_PLAN");
        assert!(THE_PLAN.ends_with(".md"));
    }

    /// **The default workstream's plan reads as tasks and nothing else.**
    #[test]
    fn the_real_plan_holds_only_tasks() {
        let written = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(THE_PLAN),
        )
        .unwrap();
        let tasks = read(&written);
        assert!(tasks.len() > 1, "the plan read as {} tasks", tasks.len());
        assert!(
            !tasks
                .iter()
                .any(|task| task.named == "Implemented and verified"),
            "the real plan's audit is being read as work"
        );
        // Numbered from one, in order, with no repeats — which is what makes
        // `**Depends on:** 1` mean one thing.
        for (which, task) in tasks.iter().enumerate() {
            assert_eq!(
                task.number,
                u32::try_from(which).unwrap() + 1,
                "the plan's tasks are not numbered in order: {task:?}"
            );
        }
    }
}
