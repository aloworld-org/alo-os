//! Bringing a parked branch across from the other checkout, by name.
//!
//! Two checkouts share this repository (`docs/autonomy/SHARED_MAIN.md`) and a
//! parked branch is local by decision, so a branch parked in one is invisible
//! from the other. Until now the way across was
//! `git fetch <path> <branch>:<branch>` typed by hand: **a step with no test,
//! no refusal, and a refspec a person can get wrong in the direction that
//! overwrites a branch already here.**
//!
//! That direction is the whole reason this is a module and not a line in a
//! runbook. `<branch>:<branch>` with a local branch of the same name already
//! present is how a person loses work they had not published — and it fails
//! silently in the sense that matters, because the fetch succeeds and the
//! branch that was here is simply no longer what it was.
//!
//! # It fetches and never pushes
//!
//! The other checkout is **read and not written.** Nothing here pushes, and
//! `repository`'s own `the_only_branch_this_pushes_is_main` stays true because
//! there is no push in this file to find. Nothing here deletes a branch either:
//! a recovery that goes wrong is one that can be done again.
//!
//! # What it refuses, and why each one
//!
//! | | |
//! |---|---|
//! | `--from` is not a git repository | a typo in a path would otherwise reach `git fetch` and fail in git's words about a stranger's argument, not ours |
//! | the other checkout does not have that branch | the likeliest mistake: the branch is parked somewhere else, or spelled differently |
//! | this checkout already has that branch | **the dangerous one.** Never overwritten, even when the name is the same |
//!
//! The third is not a convenience. A person who already has `task/x` here and
//! fetches `task/x` from elsewhere means one of two things and the command
//! cannot tell which, so it does neither and says so.

use std::path::Path;

use crate::repository::git;

/// Bring `branch` across from the checkout at `from`, into the checkout at
/// `into`.
///
/// Fetches exactly that branch, by name, and creates it here as an ordinary
/// local branch — the same kind a parked task already makes. What happens
/// after is the existing recovery, unchanged.
///
/// # Errors
/// A sentence for each of the three refusals above, and for a fetch git itself
/// would not do. Every one of them leaves both checkouts exactly as they were.
pub fn bring_across(from: &Path, branch: &str, into: &Path) -> Result<String, String> {
    if branch.is_empty() {
        return Err("no branch was named, so there is nothing to bring across".to_owned());
    }

    // Asked of the other checkout before anything else, so that a mistyped
    // path is answered in this program's words rather than in git's words
    // about an argument the person did not write.
    let theirs = git(from, &["rev-parse", "--git-dir"]).map_err(|_| {
        format!(
            "{} is not a git repository, so there is no checkout there to bring \
             a branch across from",
            from.display()
        )
    })?;
    let _ = theirs;

    if git(
        from,
        &["rev-parse", "--verify", "--quiet", &refs_heads(branch)],
    )
    .is_err()
    {
        return Err(format!(
            "the checkout at {} has no branch called `{branch}`. A parked branch \
             is local to the checkout it was parked in, so it may be in a third \
             one — or spelled differently there",
            from.display()
        ));
    }

    // **The refusal this module exists for.** Asked before the fetch, because
    // `<branch>:<branch>` would overwrite and then succeed.
    if git(
        into,
        &["rev-parse", "--verify", "--quiet", &refs_heads(branch)],
    )
    .is_ok()
    {
        return Err(format!(
            "this checkout already has a branch called `{branch}`, and bringing \
             one across would write over it. Nothing was fetched. Rename or \
             finish the one here first — a branch that is overwritten is work \
             nobody can get back, and the two may not be the same task"
        ));
    }

    let said = git(
        into,
        &[
            "fetch",
            "--quiet",
            "--no-tags",
            &from.display().to_string(),
            &format!("{branch}:{branch}"),
        ],
    )?;
    let _ = said;

    Ok(format!(
        "`{branch}` was brought across from {}",
        from.display()
    ))
}

/// A branch's full ref, so that `rev-parse --verify` answers about a branch
/// rather than about anything else of that name.
///
/// `rev-parse --verify task/x` would also answer for a tag called `task/x`, or
/// a file, and the question here is only ever about a branch.
fn refs_heads(branch: &str) -> String {
    format!("refs/heads/{branch}")
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Two real checkouts of one repository, the way the two on this machine
    /// are, in a directory of this test's own.
    ///
    /// Real rather than mocked because the thing under test is what `git
    /// fetch` does with a refspec, and a mock of git would be a mock of the
    /// exact behaviour the refusals exist to prevent.
    fn two_checkouts(named: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "alo-kernel-loop-bring-across-{named}-{}",
            std::process::id()
        ));
        drop(std::fs::remove_dir_all(&root));
        std::fs::create_dir_all(&root).unwrap();

        // The bare repository both checkouts clone from; named by the clone
        // below rather than by a binding.
        git(
            &root,
            &["init", "--quiet", "--bare", "-b", "main", "shared.git"],
        )
        .unwrap();
        for one in ["theirs", "ours"] {
            git(&root, &["clone", "--quiet", "shared.git", one]).unwrap();
        }
        let theirs = root.join("theirs");
        let ours = root.join("ours");
        std::fs::write(theirs.join("a.txt"), "one\n").unwrap();
        commit(&theirs, "the first");
        git(&theirs, &["push", "--quiet", "origin", "main"]).unwrap();
        git(&ours, &["pull", "--quiet", "origin", "main"]).unwrap();
        (theirs, ours)
    }

    /// Commit everything, as a fixture author only a test repository has.
    fn commit(at: &Path, message: &str) {
        git(at, &["add", "--all"]).unwrap();
        git(
            at,
            &[
                "-c",
                "user.name=fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--quiet",
                "-m",
                message,
            ],
        )
        .unwrap();
    }

    /// Park a branch in `at` with one file on it, and go back to main.
    fn park(at: &Path, branch: &str, wrote: &str) {
        git(at, &["checkout", "--quiet", "-b", branch, "main"]).unwrap();
        std::fs::write(at.join("parked.txt"), wrote).unwrap();
        commit(at, "parked work");
        git(at, &["checkout", "--quiet", "main"]).unwrap();
    }

    /// **A parked branch crosses, and arrives as an ordinary local branch.**
    #[test]
    fn a_parked_branch_is_brought_across() {
        let (theirs, ours) = two_checkouts("crosses");
        park(&theirs, "task/parked", "their work\n");

        assert!(
            git(
                &ours,
                &["rev-parse", "--verify", "--quiet", "refs/heads/task/parked"]
            )
            .is_err()
        );
        let said = bring_across(&theirs, "task/parked", &ours).unwrap();
        assert!(said.contains("task/parked"), "{said}");

        git(
            &ours,
            &["rev-parse", "--verify", "--quiet", "refs/heads/task/parked"],
        )
        .unwrap();
        git(&ours, &["checkout", "--quiet", "task/parked"]).unwrap();
        assert_eq!(
            std::fs::read_to_string(ours.join("parked.txt")).unwrap(),
            "their work\n"
        );
    }

    /// **A path that is not a git repository is refused**, in this program's
    /// words rather than git's about an argument nobody typed.
    #[test]
    fn a_from_that_is_not_a_repository_is_refused() {
        let (_theirs, ours) = two_checkouts("not-a-repo");
        let nowhere = std::env::temp_dir().join("alo-kernel-loop-not-a-repository");
        drop(std::fs::create_dir_all(&nowhere));

        let why = bring_across(&nowhere, "task/parked", &ours).unwrap_err();
        assert!(why.contains("is not a git repository"), "{why}");
    }

    /// **A branch the other checkout does not have is refused**, and says
    /// where a parked branch might actually be.
    #[test]
    fn a_branch_the_other_checkout_does_not_have_is_refused() {
        let (theirs, ours) = two_checkouts("no-such-branch");

        let why = bring_across(&theirs, "task/never-parked", &ours).unwrap_err();
        assert!(why.contains("has no branch called"), "{why}");
        assert!(why.contains("task/never-parked"), "{why}");
    }

    /// **A branch this checkout already has is never written over**, which is
    /// the refusal this module exists for.
    ///
    /// The assertion that matters is the second one: not merely that it
    /// refused, but that **the branch that was here is untouched**. A refusal
    /// that had already fetched would be the fault wearing a refusal's words.
    #[test]
    fn a_branch_already_here_is_never_written_over() {
        let (theirs, ours) = two_checkouts("already-here");
        park(&theirs, "task/same-name", "their work\n");
        park(&ours, "task/same-name", "our work\n");

        let before = git(&ours, &["rev-parse", "refs/heads/task/same-name"]).unwrap();
        let why = bring_across(&theirs, "task/same-name", &ours).unwrap_err();
        assert!(why.contains("already has a branch called"), "{why}");
        assert!(why.contains("Nothing was fetched"), "{why}");

        let after = git(&ours, &["rev-parse", "refs/heads/task/same-name"]).unwrap();
        assert_eq!(before, after, "the branch that was here changed");

        git(&ours, &["checkout", "--quiet", "task/same-name"]).unwrap();
        assert_eq!(
            std::fs::read_to_string(ours.join("parked.txt")).unwrap(),
            "our work\n",
            "our parked work was written over by theirs"
        );
    }

    /// **The other checkout is read and not written.** Its branches, its head
    /// and its working tree are the same after a crossing as before one.
    #[test]
    fn the_other_checkout_is_left_exactly_as_it_was() {
        let (theirs, ours) = two_checkouts("read-only");
        park(&theirs, "task/parked", "their work\n");

        let branches_before = git(&theirs, &["branch", "--list"]).unwrap();
        let head_before = git(&theirs, &["rev-parse", "HEAD"]).unwrap();
        let dirty_before = git(&theirs, &["status", "--porcelain"]).unwrap();

        bring_across(&theirs, "task/parked", &ours).unwrap();

        assert_eq!(
            branches_before,
            git(&theirs, &["branch", "--list"]).unwrap()
        );
        assert_eq!(head_before, git(&theirs, &["rev-parse", "HEAD"]).unwrap());
        assert_eq!(
            dirty_before,
            git(&theirs, &["status", "--porcelain"]).unwrap()
        );
    }

    /// **Nothing in this module pushes**, held the way `repository.rs` holds
    /// its own: by reading the source rather than by trusting the reading.
    ///
    /// It reads the code **above `#[cfg(test)]` only**, which is the idiom
    /// `crates/alo-shell/tests/desktop_source.rs` already uses — the tests are
    /// the last thing in the file, and a fixture legitimately pushes to build
    /// two checkouts to fetch between.
    ///
    /// *The first version read the whole file and refused its own fixture's
    /// `git push origin main`. It was right that there was a push and wrong
    /// about whose: **a check that read more than it was about**, caught by
    /// running it.*
    #[test]
    fn this_module_never_pushes_and_never_deletes_a_branch() {
        let source = include_str!("fetching.rs");
        let code = source
            .split_once("#[cfg(test)]")
            .map_or(source, |(above, _)| above);
        assert!(
            code.contains("pub fn bring_across"),
            "the split took the code away with the tests"
        );

        let mut looked = 0_usize;
        for line in code
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with("//") && !line.starts_with("///"))
            .filter(|line| !line.starts_with('|'))
        {
            looked += 1;
            assert!(!line.contains("\"push\""), "a push: {line}");
            assert!(!line.contains("--delete"), "a branch deletion: {line}");
            assert!(!line.contains("--force"), "a forced anything: {line}");
        }
        assert!(
            looked > 20,
            "only {looked} lines were read, so this has stopped watching anything"
        );
    }
}
