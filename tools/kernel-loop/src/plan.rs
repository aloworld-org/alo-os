//! The plan, read for one question: is this a task somebody planned?
//!
//! The loop does not choose work — a supervisor that invented tasks would be a
//! supervisor writing its own scope. What it does is refuse to publish a commit
//! under a name that appears nowhere in
//! `docs/autonomy/kernel-enforcement-plan.md`, which catches the two mistakes
//! that matter: a task nobody wrote down, and a typo in the name a report will
//! be filed under.

use std::path::Path;

/// Where this workstream's plan lives.
const THE_PLAN: &str = "docs/autonomy/kernel-enforcement-plan.md";

/// That the plan has a task by this name.
///
/// Matched against the plan's task headings, which are `### ` followed by a
/// number and the descriptive name.
///
/// # Errors
/// A sentence when the plan cannot be read, or when it does not name this task
/// — listing what it does name, because the likeliest cause is a name typed
/// slightly differently in the handoff.
pub fn names_this_task(at: &Path, task: &str) -> Result<(), String> {
    let plan = at.join(THE_PLAN);
    let written = std::fs::read_to_string(&plan)
        .map_err(|why| format!("{} could not be read: {why}", plan.display()))?;

    // `### 3. Socket attribution and default-deny` — a task is a numbered
    // heading, and the number is dropped so a handoff names the task rather
    // than its position in a list that will be reordered.
    //
    // **The number is what makes it a task.** The plan's audit sections are
    // headings at the same depth, and without this a handoff could name
    // *Implemented and verified* and be published as though somebody had
    // planned it — which is exactly the mistake this check exists to catch.
    let named: Vec<String> = written
        .lines()
        .filter_map(|line| line.strip_prefix("### "))
        .filter_map(|heading| heading.split_once(". "))
        .filter(|(number, _)| number.chars().all(|digit| digit.is_ascii_digit()))
        .map(|(_, name)| name.trim().to_owned())
        .collect();

    if named.iter().any(|known| known == task) {
        return Ok(());
    }
    Err(format!(
        "the plan does not name a task called `{task}`, so nothing was published. It names:\n  {}",
        named.join("\n  ")
    ))
}
