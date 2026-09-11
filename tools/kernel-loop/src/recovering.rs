//! Picking a parked task back up, as a command rather than as a memory.
//!
//! # The defect this replaces
//!
//! Parking works: a task whose gates refuse it is committed to a branch of its
//! own, nothing is discarded, and the run carries on. Picking one back up was
//! left to whoever remembered how, and on 2026-09-11 what they remembered was
//!
//! ```text
//! git restore --source=parked/task-21-... -- .
//! ```
//!
//! which is not *bring that task's work back*. It is **revert the whole working
//! tree to that branch**, and the branch is a photograph of `main` as it was
//! before two other tasks were published — so both of them were silently undone.
//! It happened twice in one evening. Both times the supervisor's *these are
//! changed and no task named them* check stopped it before anything reached
//! `main`; neither time did anything in this repository stop it being done again
//! ten minutes later.
//!
//! # What the right recovery is
//!
//! Mechanical, and knowable from what the parked branch already carries. The
//! branch has the task's handoff on it, and a handoff names **every file the
//! task touched**. So:
//!
//! - a file the task named and nobody has published over since comes back
//!   **whole**, as the branch has it;
//! - a file both touched has the task's **own diff** applied over what is in the
//!   tree — never its version, which is the whole of the defect above;
//! - a conflict between them is **reported and left**, markers and all, because
//!   two people's work disagreeing is not a supervisor's to decide;
//! - the handoff goes back where the loop looks for one, so the recovered task
//!   is ready to be gated.
//!
//! Not one path here stands for more than one file. There is no `restore -- .`,
//! no `checkout`, no `reset` and no `clean`, and the test
//! `recovery_never_touches_more_than_one_named_path` reads this file to keep it
//! that way.
//!
//! # It restores; it never publishes
//!
//! Recovery ends with work in the tree and nothing else. The gates are where
//! they were, `publish` is what runs them, and a recovery that published would
//! be a road around the thing this program exists to be. It deletes no branch
//! either: a recovery that goes wrong can simply be done again.

use std::path::Path;

use crate::handoff::Handed;
use crate::repository::{self, Applied};

/// What every parked branch is called, from [`crate::repository::parked`].
const PARKED: &str = "parked/task-";

/// Where the patch being applied is written, inside the loop's own directory.
///
/// A file rather than a pipe: `git apply` reads one happily, the loop's
/// directory is ignored by git, and a patch left behind by a crash is a thing
/// somebody can read.
const THE_PATCH: &str = "recovering.patch";

/// What a recovery did, file by file, so that the account of it is a value
/// rather than something printed on the way past.
#[derive(Debug, Default)]
pub struct Recovered {
    /// The branch it came from. Still there: nothing here deletes one.
    pub branch: String,

    /// The task, as its handoff names it.
    pub task: String,

    /// Files nobody else touched, put back as the branch has them.
    pub whole: Vec<String>,

    /// Files both touched, merged by applying the task's own diff.
    pub merged: Vec<String>,

    /// Files where that merge found a conflict. Left with markers in them.
    pub conflicted: Vec<String>,

    /// Paths the branch changed that its handoff never named, left exactly
    /// where they are.
    ///
    /// Always at least the handoff itself, which parking force-adds. Anything
    /// else is a file that was in the tree when the task was parked and that
    /// the task never claimed, and restoring it would be this command making
    /// the same mistake in smaller print.
    pub left_alone: Vec<String>,
}

impl Recovered {
    /// Whether the work is in the tree with nothing left to decide.
    #[must_use]
    pub fn is_ready_to_gate(&self) -> bool {
        self.conflicted.is_empty()
    }
}

/// Bring a parked task's work back on top of today's `main`.
///
/// # Errors
/// A sentence, for every way this is the wrong thing to be asked: a branch that
/// was never parked, a branch that is not there, a handoff already waiting that
/// this would write over, a branch with nothing on it, and a handoff naming
/// files the task did not change.
pub fn recover(at: &Path, ours: &Path, branch: &str) -> Result<Recovered, String> {
    if !is_a_parked_name(branch) {
        return Err(format!(
            "`{branch}` is not a parked branch. Parking names its branches `{PARKED}<task>-\
             <moment>`, and this restores one of those; a branch that was never parked is \
             somebody's work by some other route and is not this command's to unpack."
        ));
    }
    if !repository::is_a_branch(at, branch) {
        return Err(format!(
            "there is no branch `{branch}` in this checkout. Parked branches are local — \
             `git branch --list '{PARKED}*'` is the list."
        ));
    }
    // **Before anything is written.** A handoff in place is a task somebody is
    // already holding, and recovery would write over the one file that says
    // what it was.
    if let Some(already) = Handed::waiting(ours)? {
        return Err(format!(
            "a handoff is already waiting, for `{}`. Nothing was restored: publish or put that \
             one aside first, because recovering writes the parked task's handoff into the same \
             place.",
            already.task
        ));
    }

    let from = repository::where_they_parted(at, branch)?;
    let changed = repository::what_a_branch_changed(at, &from, branch)?;
    let changed: Vec<&str> = what_it_changed(&changed).collect();
    if changed.is_empty() {
        return Err(format!(
            "`{branch}` changes nothing against the `main` it was parked from, so there is no \
             work on it to restore. A task can fail its gates having written nothing, and \
             parking records that honestly rather than inventing a change."
        ));
    }

    let written = repository::one_file_as(at, branch, &the_handoff_on_the_branch())?;
    let handed = Handed::read(&written)?;
    let missing = named_but_unchanged(&handed.files, &changed);
    if !missing.is_empty() {
        return Err(format!(
            "`{branch}` carries a handoff for `{}` naming files that branch never changed: {}. \
             Nothing was restored — a handoff that does not describe its own branch cannot say \
             what to bring back, and guessing would be this command restoring files nobody \
             claimed.",
            handed.task,
            missing.join(", ")
        ));
    }

    let mut recovered = Recovered {
        branch: branch.to_owned(),
        task: handed.task.clone(),
        left_alone: changed
            .iter()
            .filter(|path| !handed.files.iter().any(|named| named.as_str() == **path))
            .map(|path| (*path).to_owned())
            .collect(),
        ..Recovered::default()
    };

    for path in &handed.files {
        if repository::changed_here_since(at, &from, path)? {
            match merged_into_the_tree(at, ours, &from, branch, path)? {
                Applied::Cleanly => recovered.merged.push(path.clone()),
                Applied::WithAConflict => recovered.conflicted.push(path.clone()),
            }
        } else {
            repository::restored_one_file_from(at, branch, path)?;
            recovered.whole.push(path.clone());
        }
    }

    // **Last, and only once every file is back.** The handoff is what makes the
    // recovered work publishable, and writing it before the restoring could
    // leave a tree that looks ready and is half a task.
    std::fs::create_dir_all(ours)
        .map_err(|why| format!("{} could not be made: {why}", ours.display()))?;
    std::fs::write(Handed::where_one_waits(ours), &written).map_err(|why| {
        format!(
            "the work is back in the tree but its handoff could not be written: {why}. Nothing \
             was lost; `{branch}` still has it."
        )
    })?;

    Ok(recovered)
}

/// One file both the task and somebody else changed, merged by applying the
/// task's own diff over what is in the tree.
///
/// # Errors
/// Whatever `git` said about producing or applying the patch. A conflict is not
/// one of those: it is [`Applied::WithAConflict`], reported and left.
fn merged_into_the_tree(
    at: &Path,
    ours: &Path,
    from: &str,
    branch: &str,
    path: &str,
) -> Result<Applied, String> {
    let diff = repository::one_files_diff(at, from, branch, path)?;
    std::fs::create_dir_all(ours)
        .map_err(|why| format!("{} could not be made: {why}", ours.display()))?;
    let patching = ours.join(THE_PATCH);
    std::fs::write(&patching, &diff)
        .map_err(|why| format!("the patch for {path} could not be written: {why}"))?;
    let went = repository::applied_over(at, &patching, path);
    drop(std::fs::remove_file(&patching));
    went.map_err(|why| {
        format!(
            "{path} was changed both by this task and on `main` since, and the task's own diff \
             would not apply over it: {why}\nNothing about {path} was guessed at. Whatever ran \
             before this in the recovery is in the tree."
        )
    })
}

/// Where parking puts the handoff it force-adds.
fn the_handoff_on_the_branch() -> String {
    format!("{}/handoff.toml", crate::ITS_OWN)
}

/// Whether this is a name parking would have made.
///
/// `parked/task-<number>-<moment>`. Held to the shape rather than to the prefix
/// alone, so that a branch somebody named `parked/task-mine` by hand is refused:
/// this command reads a handoff out of a branch and writes files from it, and
/// the one thing it must be sure of is that the branch is one the supervisor
/// made.
fn is_a_parked_name(branch: &str) -> bool {
    let Some(rest) = branch.strip_prefix(PARKED) else {
        return false;
    };
    let Some((task, moment)) = rest.split_once('-') else {
        return false;
    };
    let numbered = !task.is_empty() && task.chars().all(|c| c.is_ascii_digit());
    // `moment` is seconds since the epoch, or the word parking uses when the
    // clock refused to answer.
    let timed =
        moment == "unknown" || (!moment.is_empty() && moment.chars().all(|c| c.is_ascii_digit()));
    numbered && timed
}

/// The paths out of `git diff --name-status`.
///
/// Each line is a status letter, a tab, and a path. The status itself is not
/// read: a file the task added, changed or deleted is a file the task changed,
/// and restoring each of those is [`repository::restored_one_file_from`]'s job
/// rather than a decision to take twice.
fn what_it_changed(lines: &str) -> impl Iterator<Item = &str> {
    lines
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(_, path)| path.trim())
        .filter(|path| !path.is_empty())
}

/// The files a handoff names that its own branch never changed.
///
/// Empty for a handoff that describes its branch, which is every handoff the
/// supervisor parked. Anything else is a handoff that cannot be trusted to say
/// what to bring back.
fn named_but_unchanged(named: &[String], changed: &[&str]) -> Vec<String> {
    named
        .iter()
        .filter(|name| !changed.iter().any(|path| path == &name.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A checkout with a `main`, a parked branch on it, and nothing shared with
    /// any other test.
    ///
    /// A real repository rather than a stand-in for one. What these tests have
    /// to prove is what `git` does to a working tree when the recovery asks it
    /// for something, and a fake `git` would prove only that the fake agrees
    /// with whoever wrote it — which is exactly the gap the defect lived in.
    struct ARepository {
        /// Where it is.
        at: PathBuf,
    }

    impl ARepository {
        /// A repository on `main`, with one commit and `.kernel-loop` ignored,
        /// exactly as this one is.
        fn made(called: &str) -> Self {
            let at = std::env::temp_dir()
                .join(format!("alo-recovering-{}-{called}", std::process::id()));
            drop(std::fs::remove_dir_all(&at));
            std::fs::create_dir_all(&at).expect("a directory to work in");
            let made = Self { at };
            made.git(&["init", "--quiet", "--initial-branch", "main"]);
            made.git(&["config", "user.email", "loop@example.invalid"]);
            made.git(&["config", "user.name", "a test"]);
            // The fixture writes `\n` and reads it back. A checkout configured
            // to translate line endings would make every patch in these tests
            // about Windows rather than about recovery.
            made.git(&["config", "core.autocrlf", "false"]);
            made.write(".gitignore", "/.kernel-loop\n");
            made.write("kept.txt", "a file no task in these tests touches\n");
            made.git(&["add", "--all"]);
            made.git(&["commit", "--quiet", "--message", "the main it parted from"]);
            made
        }

        /// Run one git here, and say what it printed.
        fn git(&self, args: &[&str]) -> String {
            repository::git(&self.at, args)
                .map_err(|why| format!("`git {}` in the fixture: {why}", args.join(" ")))
                .expect("the fixture's git to do as it is asked")
        }

        /// Where the loop keeps its own files here.
        fn ours(&self) -> PathBuf {
            self.at.join(crate::ITS_OWN)
        }

        /// Write a file, making whatever directories it needs.
        fn write(&self, path: &str, what: &str) {
            let full = self.at.join(path);
            if let Some(holding) = full.parent() {
                std::fs::create_dir_all(holding).expect("a directory for the file");
            }
            std::fs::write(full, what).expect("the file to be written");
        }

        /// What a file says now, or [`None`] when it is not there.
        fn reading(&self, path: &str) -> Option<String> {
            std::fs::read_to_string(self.at.join(path)).ok()
        }

        /// Write a handoff naming these files, as a worker would.
        fn handing_over(&self, task: &str, files: &[&str]) {
            let mut written = format!(
                "task = {task}\nreport = {A_REPORT}\nsubject = feat(x): {task}\nevidence =\n  . \
                 alo-x lib a_test\nfiles =\n"
            );
            for named in files {
                written.push_str("  ");
                written.push_str(named);
                written.push('\n');
            }
            written.push_str("body =\n  What it says.\n");
            self.write(&format!("{}/handoff.toml", crate::ITS_OWN), &written);
        }

        /// Park whatever is in the tree, as the supervisor does.
        fn parking(&self, task: u32) -> String {
            repository::parked(&self.at, task, "the gates said no").expect("the work to be parked")
        }

        /// Commit something on `main`, as another task publishing would.
        fn published(&self, path: &str, what: &str) {
            self.write(path, what);
            self.git(&["add", "--all", "--", path]);
            self.git(&["commit", "--quiet", "--message", "another task"]);
        }

        /// Recover from a branch here.
        fn recovering(&self, branch: &str) -> Result<Recovered, String> {
            recover(&self.at, &self.ours(), branch)
        }
    }

    impl Drop for ARepository {
        fn drop(&mut self) {
            drop(std::fs::remove_dir_all(&self.at));
        }
    }

    /// The report every fixture handoff publishes, because a handoff that does
    /// not list its own report is refused before anything here is reached.
    const A_REPORT: &str = "docs/autonomy/updates/a-report.md";

    /// **A parked task comes back on top of today's `main`, ready to gate.**
    ///
    /// The whole command in one test: the task's work is in the tree, the work
    /// another task published while it was parked is **still there**, and the
    /// handoff is back where the loop looks for one. The middle of those three
    /// is the defect — reverting the tree to the branch passes the first and
    /// the third and silently undoes the second.
    #[test]
    fn a_parked_task_comes_back_on_top_of_todays_main_with_its_handoff() {
        let it = ARepository::made("ontop");
        it.write("src/work.rs", "what the task wrote\n");
        it.write(A_REPORT, "the task's report\n");
        it.handing_over("Doing the thing", &["src/work.rs", A_REPORT]);
        let branch = it.parking(28);

        it.published("src/somebody-else.rs", "published while it was parked\n");

        let back = it.recovering(&branch).expect("the task to come back");
        assert_eq!(back.task, "Doing the thing");
        assert!(back.is_ready_to_gate(), "an unexpected conflict: {back:?}");
        assert_eq!(
            it.reading("src/work.rs").as_deref(),
            Some("what the task wrote\n"),
            "the task's own work did not come back"
        );
        assert_eq!(
            it.reading("src/somebody-else.rs").as_deref(),
            Some("published while it was parked\n"),
            "recovery undid a task published while this one was parked — the whole defect"
        );
        assert_eq!(
            it.reading("kept.txt").as_deref(),
            Some("a file no task in these tests touches\n"),
            "recovery reverted a file no task named"
        );
        let waiting = Handed::waiting(&it.ours())
            .expect("the handoff to read")
            .expect("a handoff to be waiting");
        assert_eq!(waiting.task, "Doing the thing");
        assert!(
            repository::is_a_branch(&it.at, &branch),
            "the branch was deleted, so a recovery that went wrong could not be done again"
        );
    }

    /// **A file the task named and nobody else has touched comes back whole** —
    /// including one the task deleted, which comes back deleted.
    #[test]
    fn a_file_nobody_else_touched_comes_back_whole() {
        let it = ARepository::made("whole");
        it.published("src/old.rs", "to be deleted by the task\n");

        it.write("src/added.rs", "added by the task\n");
        std::fs::remove_file(it.at.join("src/old.rs")).expect("the task's own deletion");
        it.write(A_REPORT, "the task's report\n");
        it.handing_over(
            "Adding and removing",
            &["src/added.rs", "src/old.rs", A_REPORT],
        );
        let branch = it.parking(28);
        assert_eq!(
            it.reading("src/old.rs").as_deref(),
            Some("to be deleted by the task\n"),
            "the fixture is wrong: parking left the task's deletion in the tree"
        );

        let back = it.recovering(&branch).expect("the task to come back");
        assert!(
            back.merged.is_empty(),
            "a file nobody else touched was merged rather than put back: {back:?}"
        );
        assert_eq!(
            back.whole.len(),
            3,
            "not every file came back whole: {back:?}"
        );
        assert_eq!(
            it.reading("src/added.rs").as_deref(),
            Some("added by the task\n")
        );
        assert_eq!(
            it.reading("src/old.rs"),
            None,
            "a file the task deleted came back"
        );
    }

    /// **A file both touched is merged by applying the task's own diff**, never
    /// by restoring the task's version of it.
    ///
    /// The published line has to survive. Restoring the branch's version of the
    /// file would pass every other assertion here and lose it, which is the
    /// small-print form of the defect.
    /// A plan with room in it, so that two tasks marking themselves done are
    /// two changes rather than one line each side of the other.
    ///
    /// The distance is the realistic case and not a convenience: a plan is
    /// hundreds of lines, and two tasks editing it are ordinarily nowhere near
    /// each other. Adjacent lines are what the conflict test below is for.
    fn a_plan(done: &[u32]) -> String {
        (1..=20_u32)
            .map(|task| {
                let how = if done.contains(&task) {
                    "done"
                } else {
                    "ready"
                };
                format!("task {task} - {how}\n")
            })
            .collect()
    }

    #[test]
    fn a_file_both_touched_is_merged_by_the_tasks_own_diff() {
        let it = ARepository::made("merged");
        it.published("plan.md", &a_plan(&[]));

        it.write("plan.md", &a_plan(&[18]));
        it.write(A_REPORT, "the task's report\n");
        it.handing_over("Marking itself done", &["plan.md", A_REPORT]);
        let branch = it.parking(28);

        it.published("plan.md", &a_plan(&[2]));

        let back = it.recovering(&branch).expect("the task to come back");
        assert!(back.is_ready_to_gate(), "an unexpected conflict: {back:?}");
        assert_eq!(back.merged, vec!["plan.md".to_owned()]);
        let plan = it.reading("plan.md").unwrap_or_default();
        assert!(
            plan.contains("task 18 - done"),
            "the task's own change is not in the file: {plan}"
        );
        assert!(
            plan.contains("task 2 - done"),
            "recovery restored the task's version and undid what was published: {plan}"
        );
    }

    /// **A conflict is reported and left**, markers and all.
    ///
    /// Nothing is resolved by preference, and the recovery says so rather than
    /// handing back a tree that looks ready to publish.
    #[test]
    fn a_conflict_is_reported_and_left_unresolved() {
        let it = ARepository::made("conflict");
        it.published("plan.md", "one\ntask 28 - ready\nthree\n");

        it.write("plan.md", "one\ntask 28 - done by me\nthree\n");
        it.write(A_REPORT, "the task's report\n");
        it.handing_over("Both on one line", &["plan.md", A_REPORT]);
        let branch = it.parking(28);

        it.published("plan.md", "one\ntask 28 - done by somebody else\nthree\n");

        let back = it.recovering(&branch).expect("a conflict is not a refusal");
        assert!(
            !back.is_ready_to_gate(),
            "a conflicted recovery said it was ready to gate: {back:?}"
        );
        assert_eq!(back.conflicted, vec!["plan.md".to_owned()]);
        let plan = it.reading("plan.md").unwrap_or_default();
        assert!(
            plan.contains("<<<<<<<") && plan.contains(">>>>>>>"),
            "the conflict was resolved rather than left: {plan}"
        );
        assert!(
            plan.contains("done by me") && plan.contains("done by somebody else"),
            "a side was chosen for somebody: {plan}"
        );
    }

    /// **A handoff naming files its branch never changed is refused in words.**
    ///
    /// The branch's own handoff is the only thing recovery reads to know what
    /// to bring back, so one that does not describe the branch is one this
    /// cannot act on. Restoring the files it *can* find and staying quiet about
    /// the rest would hand back a tree that is a task minus whatever was wrong.
    #[test]
    fn a_handoff_naming_files_the_task_did_not_change_is_refused() {
        let it = ARepository::made("mismatch");
        it.write("src/work.rs", "what the task wrote\n");
        it.write(A_REPORT, "the task's report\n");
        it.handing_over(
            "Naming what it did not write",
            &["src/work.rs", "src/never-written.rs", A_REPORT],
        );
        let branch = it.parking(28);

        let refused = it.recovering(&branch);
        assert!(
            refused
                .as_ref()
                .is_err_and(|why| why.contains("src/never-written.rs")),
            "a handoff that does not describe its branch was acted on: {refused:?}"
        );
        assert_eq!(
            it.reading("src/work.rs"),
            None,
            "a refused recovery still put files in the tree"
        );
        assert!(
            Handed::waiting(&it.ours())
                .expect("the handoff to read")
                .is_none(),
            "a refused recovery still left a handoff waiting"
        );
    }

    /// **A branch that is not a parked branch is refused**, whatever is on it.
    ///
    /// Recovery reads a handoff out of a branch and writes files from it. The
    /// one thing it has to be sure of is that the branch is one the supervisor
    /// parked, and a name that merely begins with the word is not that.
    #[test]
    fn a_branch_that_was_never_parked_is_refused() {
        let it = ARepository::made("notparked");
        for name in ["main", "parked/task-mine", "parked/task-", "some/branch"] {
            let refused = recover(&it.at, &it.ours(), name);
            assert!(
                refused.is_err_and(|why| why.contains("not a parked branch")),
                "`{name}` was taken for a parked branch"
            );
        }
    }

    /// **A parked name for a branch nobody has is refused by saying so**, which
    /// is a different sentence from the one above and a different mistake.
    #[test]
    fn a_parked_branch_that_is_not_here_is_refused() {
        let it = ARepository::made("missing");
        let refused = it.recovering("parked/task-28-1757000000");
        assert!(
            refused.is_err_and(|why| why.contains("no branch")),
            "a branch nobody has was recovered from"
        );
    }

    /// **A handoff already waiting is not written over.**
    ///
    /// Somebody holding a task has one file that says what it is. Recovery
    /// writes that file, so a recovery run over an occupied checkout would
    /// replace it with a different task's — and the loop would then gate one
    /// task's tree under another task's name.
    #[test]
    fn a_recovery_will_not_write_over_a_handoff_that_is_waiting() {
        let it = ARepository::made("occupied");
        it.write("src/work.rs", "what the task wrote\n");
        it.write(A_REPORT, "the task's report\n");
        it.handing_over("The parked one", &["src/work.rs", A_REPORT]);
        let branch = it.parking(28);

        it.handing_over("Somebody's work in hand", &["src/other.rs", A_REPORT]);
        let refused = it.recovering(&branch);
        assert!(
            refused.is_err_and(|why| why.contains("already waiting")),
            "a recovery wrote over a handoff somebody was holding"
        );
        let still = Handed::waiting(&it.ours())
            .expect("the handoff to read")
            .expect("a handoff to be waiting");
        assert_eq!(still.task, "Somebody's work in hand");
    }

    /// **Nothing in the recovery touches more than one named path.**
    ///
    /// The defect was a pathspec — a source-restore with `.` on the end of it —
    /// and the repair is that no operation here takes one that stands for a
    /// tree. Held on the source because that is where the mistake is made: a
    /// whole-tree command added for convenience would pass every behavioural
    /// test above on the day it was written and revert somebody's published
    /// work on the day it was used.
    #[test]
    fn recovery_never_touches_more_than_one_named_path() {
        let whole = include_str!("recovering.rs");
        let source = whole.split("#[cfg(test)]").next().unwrap_or_default();
        assert!(
            source.contains("pub fn recover"),
            "this test is no longer reading the recovery"
        );
        for line in source
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with("//"))
        {
            // Quoted, because these are git's words rather than English ones:
            // a refusal that mentions the checkout a branch is missing from is
            // prose, and `&["checkout", …]` is the defect.
            for forbidden in [
                "\"checkout\"",
                "\"reset\"",
                "\"clean\"",
                "\"restore\"",
                "Command::new",
            ] {
                assert!(
                    !line.contains(forbidden),
                    "a git that can touch a whole tree found its way into the recovery: {line}"
                );
            }
        }

        // And the one operation that puts a file back reads exactly one path.
        let git = include_str!("repository.rs");
        let restoring = git
            .split("pub fn restored_one_file_from")
            .nth(1)
            .and_then(|rest| rest.split("\n}\n").next())
            .unwrap_or_default();
        assert!(
            restoring.contains("\"--\", path"),
            "the one restore in this loop no longer names exactly one path"
        );
        for forbidden in ["\".\"", "\"--all\"", "\"-A\"", "\"*\""] {
            assert!(
                !restoring.contains(forbidden),
                "the restore took a pathspec standing for many files: {forbidden}"
            );
        }
    }

    /// **The shape of a parked name**, read the way parking writes it.
    #[test]
    fn only_the_names_parking_writes_are_parked_names() {
        assert!(is_a_parked_name("parked/task-28-1757600000"));
        assert!(is_a_parked_name("parked/task-3-unknown"));
        for not in [
            "parked/task-28",
            "parked/task-abc-1757600000",
            "parked/task--1757600000",
            "main",
            "parked/something-else",
        ] {
            assert!(
                !is_a_parked_name(not),
                "`{not}` was taken for a parked name"
            );
        }
    }

    /// **Name-status is read as paths**, whatever letter stands in front of
    /// them: added, changed and deleted are all *the task touched this*.
    #[test]
    fn every_kind_of_change_is_a_path_to_bring_back() {
        let lines = "M\tsrc/a.rs\nA\tsrc/b.rs\nD\tsrc/c.rs\n";
        let paths: Vec<&str> = what_it_changed(lines).collect();
        assert_eq!(paths, vec!["src/a.rs", "src/b.rs", "src/c.rs"]);
        assert!(what_it_changed("").next().is_none());
    }
}
