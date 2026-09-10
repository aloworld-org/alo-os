//! Every `git` this loop runs, in one file, so that what it can do to a
//! checkout is a list somebody can read.
//!
//! **What is not here is the point.** There is no `reset`, no `checkout --`, no
//! `clean`, no `push --force` and no `rebase --skip`. A supervisor with any of
//! those can throw away work that was never published anywhere, and no message
//! it writes afterwards brings the work back.
//!
//! # `main` is the only branch this pushes. Ever.
//!
//! Every `push` in this file names [`MAIN`], and any future one must too. The
//! repository has one branch and gets one branch: a reader looking at it sees
//! what alo OS is, not a shelf of everything that was ever attempted.
//!
//! Parking makes a **local** branch and stops there. Parked work is by
//! definition work that did not pass its gates, and pushing it put a permanent
//! branch on the shared repository for every failed draft — seven in a single
//! evening, each one a task that had already been redone properly and published
//! to `main`. The drafts outnumbered the deliveries.
//!
//! The cost is stated rather than hidden: a parked branch lives on one disk, so
//! losing the checkout loses it. That is the right trade for a draft. **Work
//! worth keeping is work that passed its gates**, and the way to keep it is to
//! finish the task — which is what the loop does next anyway.

use std::path::Path;
use std::process::Command;

/// The branch this workstream publishes to.
const MAIN: &str = "main";

/// Run one `git` and hand back what it said.
///
/// # Errors
/// A sentence naming the command and what it printed, so a failure reads as
/// *this is what git was asked and this is what it answered* rather than as an
/// exit code.
pub fn git(at: &Path, args: &[&str]) -> Result<String, String> {
    let said = Command::new("git")
        .current_dir(at)
        .args(args)
        .output()
        .map_err(|why| format!("git could not be run: {why}"))?;
    let out = String::from_utf8_lossy(&said.stdout).trim().to_owned();
    if said.status.success() {
        return Ok(out);
    }
    let err = String::from_utf8_lossy(&said.stderr).trim().to_owned();
    Err(format!("`git {}` refused: {err}{out}", args.join(" ")))
}

/// That this is `main`, and that nothing is changed except what a task named.
///
/// The second half is what stops a task publishing somebody else's work by
/// accident: a file changed in the tree and absent from the handoff is either a
/// second task half-done or a mistake, and both are worth stopping over.
///
/// # Errors
/// A sentence naming the branch, or the files nobody accounted for.
pub fn on_main_and_clean_but_for(at: &Path, named: &[String]) -> Result<(), String> {
    let branch = git(at, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if branch != MAIN {
        return Err(format!(
            "the checkout is on `{branch}` and this workstream publishes to `{MAIN}`"
        ));
    }
    // `--untracked-files=all`, and it is the difference between a task that
    // creates a crate publishing and being refused. Porcelain's default
    // collapses a brand-new directory into one entry — `?? crates/alo-overlay/`
    // — while a handoff names the files inside it, so the comparison could never
    // match and **every task that created a directory was refused as
    // unaccounted for**. The first task the v0.01 loop ever ran was parked by
    // exactly this, with a handoff that had correctly named all ten of its
    // files.
    let changed = git(at, &["status", "--porcelain", "--untracked-files=all"])?;
    // **Not `line[3..]`, and not a bare `split_once`.** Porcelain writes two
    // status columns then a space, so counting three characters looks right —
    // but the whole output is trimmed by the time it arrives here, which eats
    // the leading space of the first line and takes the first letter of its
    // path with it. Splitting at the first space fixes that and breaks the
    // other lines, because an unstaged one *starts* with a space and the split
    // lands at nothing. Trimming each line first and then dropping its status
    // field is the one reading that survives all four spellings — ` M`, `M `,
    // `MM` and `??`. Both mistakes were made here and both were found by
    // running the loop rather than by reading it.
    let unaccounted: Vec<&str> = changed
        .lines()
        .filter_map(|line| line.trim_start().split_once(' '))
        .map(|(_, path)| path.trim_start())
        .filter(|path| !accounted_for(path, named))
        .collect();
    if !unaccounted.is_empty() {
        return Err(format!(
            "these are changed and no task named them, so nothing was published: {}",
            unaccounted.join(", ")
        ));
    }
    Ok(())
}

/// Whether a task named this changed path.
///
/// # Renames are two paths, and a task has to name both
///
/// Porcelain writes a rename as `old -> new` on one line, so the path this sees
/// for a moved file is that whole pair and matches no single entry. Before this
/// existed, **a task that moved a file could not publish at all** — the pair was
/// reported as unaccounted for however carefully the task named its files, which
/// is how moving one fixture into a crate of its own first failed.
///
/// Both halves must be named, which is the honest requirement rather than a
/// convenience: a move deletes a path and creates another, a reader of the
/// commit needs to see both, and naming only the new one would let a file
/// disappear from a crate without the task that did it saying so.
fn accounted_for(path: &str, named: &[String]) -> bool {
    /// What porcelain puts between the two halves of a rename.
    const MOVED_TO: &str = " -> ";

    match path.split_once(MOVED_TO) {
        Some((from, to)) => {
            let named_it = |what: &str| named.iter().any(|named| named == what);
            named_it(from) && named_it(to)
        }
        None => named.iter().any(|named| named == path),
    }
}

/// Bring in whatever was published while the work was being done.
///
/// Fast-forward only. A merge commit made by a supervisor is a supervisor
/// deciding how two people's work fits together, which is a judgement it has no
/// way to make.
///
/// **Skipped when this checkout has commits of its own that are not published
/// yet**, because a fast-forward is exactly what that is not, and the publish
/// path rebases them onto whatever arrived anyway. Found by running the loop
/// with one task committed and the next being written: it refused to start on
/// the second because it could not fast-forward past the first.
///
/// # Errors
/// A sentence when a checkout with nothing of its own still cannot be
/// fast-forwarded, which is a state somebody should look at.
pub fn pulled(at: &Path) -> Result<(), String> {
    if has_unpublished(at)? {
        return Ok(());
    }
    git(at, &["pull", "--ff-only", "origin", MAIN]).map(|_| ())
}

/// Whether this checkout has commits `origin/main` does not.
///
/// # Errors
/// Whatever `git` said about fetching or counting.
pub fn has_unpublished(at: &Path) -> Result<bool, String> {
    git(at, &["fetch", "origin", MAIN, "--quiet"])?;
    let ahead = git(at, &["rev-list", "--count", "origin/main..HEAD"])?;
    Ok(ahead.trim() != "0")
}

/// Whether `origin/main` has anything this checkout does not.
///
/// # Errors
/// Whatever `git` said about fetching or counting.
pub fn advanced(at: &Path) -> Result<bool, String> {
    git(at, &["fetch", "origin", MAIN, "--quiet"])?;
    let behind = git(at, &["rev-list", "--count", "HEAD..origin/main"])?;
    Ok(behind.trim() != "0")
}

/// Put this checkout's unpublished commits on top of what arrived.
///
/// # Errors
/// A sentence when the rebase stops, which on this workstream means a real
/// conflict. **Nothing is aborted here**: the rebase is left exactly where it
/// stopped so that whoever looks at it sees what git saw. A supervisor that ran
/// `--abort` would be one that hides a conflict by undoing it.
pub fn rebased_onto_origin(at: &Path) -> Result<(), String> {
    git(at, &["rebase", "origin/main"])
        .map(|_| ())
        .map_err(|why| {
            format!(
                "{why}\nThe rebase has been left where it stopped, with both sides intact. \
             Resolve it by hand; nothing has been aborted or discarded."
            )
        })
}

/// Stage exactly these paths.
///
/// # Errors
/// Whatever `git` said. Named one at a time so a path that does not exist is a
/// failure that names the path.
pub fn staged(at: &Path, files: &[String]) -> Result<(), String> {
    for named in files {
        // `--all`, so that a named path which is now a **deletion** stages as
        // one. Still only the paths a task named: this is `--all` within one
        // pathspec, never across the tree.
        if let Err(why) = git(at, &["add", "--all", "--", named]) {
            // A task that moved a file names both ends of the move, and `git
            // mv` has **already recorded** the end it came from — so that path
            // is in neither the worktree nor the index under its own name, and
            // a pathspec matches nothing. That is staged, not missing.
            //
            // Asked of git rather than assumed from the error text, and only
            // the *from* end of an actual staged rename is forgiven: a path
            // nobody recorded is a real mistake and still stops the publish.
            if moved_away_already(at, named)? {
                continue;
            }
            return Err(why);
        }
    }
    Ok(())
}

/// Whether this path is the end a staged rename moved **from**.
///
/// # Errors
/// Whatever `git` said.
fn moved_away_already(at: &Path, named: &str) -> Result<bool, String> {
    // `--untracked-files=all` for the same reason as the cleanliness check:
    // one reading of the tree, not two that disagree about new directories.
    let changed = git(at, &["status", "--porcelain", "--untracked-files=all"])?;
    Ok(changed
        .lines()
        .filter_map(|line| line.trim_start().split_once(' '))
        .filter_map(|(_, path)| path.trim_start().split_once(" -> "))
        .any(|(from, _)| from == named))
}

/// Commit what is staged, with this message, as whoever the checkout is
/// configured as.
///
/// **No author override and no trailer.** The identity is the repository's own
/// configuration, which is the owner's; a supervisor that set an author would be
/// claiming somebody wrote something they did not.
///
/// # Errors
/// Whatever `git` said.
pub fn committed(at: &Path, message: &str) -> Result<String, String> {
    let mut wrote = tempting(at, message)?;
    let made = git(at, &["commit", "--quiet", "--file", &wrote]);
    drop(std::fs::remove_file(&wrote));
    wrote.clear();
    made?;
    git(at, &["rev-parse", "--short", "HEAD"])
}

/// Put the message somewhere `git` can read it, because a commit body has
/// newlines in it and an argument list is a poor place for those.
fn tempting(at: &Path, message: &str) -> Result<String, String> {
    let named = at.join(".kernel-loop").join("message.txt");
    std::fs::write(&named, message)
        .map_err(|why| format!("the commit message could not be written: {why}"))?;
    Ok(named.to_string_lossy().into_owned())
}

/// Publish. An ordinary push, and never a forced one.
///
/// # Errors
/// Whatever `git` said, which for a lost race is the fast-forward hint the
/// caller reads as *integrate and try again*.
pub fn pushed(at: &Path) -> Result<(), String> {
    git(at, &["push", "origin", MAIN]).map(|_| ())
}

/// Put a task's unfinished work on a branch of its own, push it, and leave
/// `main` clean.
///
/// **The alternative to this is a loop that stops for days.** A task whose gates
/// will not pass leaves its work in the tree; the next task cannot start,
/// because starting one on top of somebody else's uncommitted work is exactly
/// the ambiguous authorship this supervisor exists to prevent. So the run ended
/// — and unattended, it ended in the first hour.
///
/// Nothing is discarded and nothing is reset. The work is committed to a branch
/// named for the task, pushed so it exists somewhere other than this disk, and
/// `main` returns to the commit it was already on. Whoever picks that task up
/// finds every line of it.
///
/// Deliberately **not** a stash: a stash is invisible from any other machine and
/// is the first thing lost when somebody tidies a checkout.
///
/// # Errors
/// A sentence when git refuses a step, in which case the work is still in the
/// tree — and the loop stops, which is the right answer for a repository that
/// will not do as it is asked.
pub fn parked(at: &Path, task: u32, why: &str) -> Result<String, String> {
    let branch = format!("parked/task-{task}-{}", moment());
    git(at, &["switch", "--create", &branch])?;

    // **The handoff goes with it, and it has to be forced.** `.kernel-loop` is
    // ignored, so `--all` walks straight past the one file that says which task
    // this was, what evidence it claimed and which files it touched — the most
    // useful thing in the branch to whoever picks the work up. Ignored is right
    // for the loop's scratch directory and wrong for this.
    drop(git(
        at,
        &["add", "--force", "--", ".kernel-loop/handoff.toml"],
    ));

    let put_away = git(at, &["add", "--all"])
        .and_then(|_| {
            git(
                at,
                &[
                    "commit",
                    "--message",
                    &format!(
                        "wip(parked): task {task} did not pass its gates\n\n{}\n\nParked by the \
                         supervisor so the run could continue. Nothing here reached main, \
                         nothing was discarded, and the gates that refused it are the gates it \
                         still has to pass.",
                        why.trim()
                    ),
                ],
            )
        })
        .or_else(|why| {
            // **Nothing to park is not a failure to park.** A task can fail its
            // gates having changed nothing in the tree — the refusal was about
            // the handoff, or about the machine — and git calls an empty commit
            // an error. Treating that as *the work could not be put anywhere
            // safe* stopped a run dead with nothing whatsoever at risk.
            if why.contains("nothing to commit") || why.contains("nothing added to commit") {
                Ok(String::new())
            } else {
                Err(why)
            }
        })
        // **Not pushed.** `main` is the only branch this supervisor publishes,
        // and parked work is by definition work that did not pass its gates —
        // pushing it put a permanent branch on the shared repository for every
        // draft that failed, seven in one evening. The branch is local: nothing
        // is discarded, the work is a `git switch` away for whoever wants it,
        // and the repository stays one branch.
        //
        // What that costs is honest: parked work lives on this disk only, so a
        // lost checkout loses it. That is the right trade for a draft. Work
        // worth keeping is work that passes its gates, and the way to keep it
        // is to finish the task.
        ;

    // Back onto `main` whatever happened, so a push that failed does not also
    // leave the checkout on a branch nobody is expecting.
    git(at, &["switch", MAIN])?;
    put_away?;

    // **And the handoff is taken off `main`.** It named the task being parked,
    // and that task is no longer being pursued — leaving it behind poisons the
    // next iteration, which selects a different task, finds a handoff for this
    // one, and refuses because the two disagree. That happened: the run stopped
    // dead one task after a successful park, and the stop said the handoff and
    // the plan named different tasks. It is safe to remove because the branch
    // above now carries it.
    drop(std::fs::remove_file(
        at.join(".kernel-loop").join("handoff.toml"),
    ));

    Ok(branch)
}

/// A moment, as something that can be part of a branch name.
fn moment() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or_else(
            |_| "unknown".to_owned(),
            |since| since.as_secs().to_string(),
        )
}

#[cfg(test)]
mod tests {
    /// **`main` is the only branch this supervisor pushes**, and this reads the
    /// file to say so rather than trusting anybody to remember.
    ///
    /// Parking used to push its branch, and one evening's failed drafts left
    /// seven permanent branches on the shared repository — every one of them a
    /// task that had since been redone properly and published to `main`. The
    /// drafts outnumbered the deliveries.
    ///
    /// A comment saying *do not push anything else* would be a request. This is
    /// the rule: every `push` in this file names `MAIN`, and a new one that does
    /// not fails here.
    #[test]
    fn the_only_branch_this_pushes_is_main() {
        let source = include_str!("repository.rs");
        let pushes: Vec<&str> = source
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with("//") && !line.starts_with("///"))
            .filter(|line| line.contains("\"push\""))
            .collect();

        assert!(
            !pushes.is_empty(),
            "no push at all — this test has stopped watching anything"
        );
        for pushing in pushes {
            assert!(
                pushing.contains("MAIN"),
                "a push that does not name MAIN: {pushing}"
            );
            assert!(!pushing.contains("--force"), "a forced push: {pushing}");
        }
    }

    use super::accounted_for;

    /// What porcelain writes for a file that moved.
    const A_MOVE: &str = "crates/alo-secrets/tests/a_keyring_of_our_own/mod.rs -> crates/alo-keyring-fixture/src/lib.rs";

    /// **A task that names both ends of a move has accounted for it.**
    #[test]
    fn a_rename_is_accounted_for_when_both_ends_are_named() {
        let named = [
            "crates/alo-secrets/tests/a_keyring_of_our_own/mod.rs".to_owned(),
            "crates/alo-keyring-fixture/src/lib.rs".to_owned(),
        ];
        assert!(
            accounted_for(A_MOVE, &named),
            "a move whose two ends were both named was still reported as unaccounted for"
        );
    }

    /// **Naming only where it went is not enough**, which is the point.
    ///
    /// A move deletes one path and creates another. A reader of the commit needs
    /// to see both, and a task naming only the destination would let a file
    /// disappear from a crate without saying so.
    #[test]
    fn a_rename_naming_only_one_end_is_refused() {
        let only_the_new = ["crates/alo-keyring-fixture/src/lib.rs".to_owned()];
        assert!(
            !accounted_for(A_MOVE, &only_the_new),
            "a move was published while the task named only where the file went"
        );

        let only_the_old = ["crates/alo-secrets/tests/a_keyring_of_our_own/mod.rs".to_owned()];
        assert!(
            !accounted_for(A_MOVE, &only_the_old),
            "a move was published while the task named only where the file was"
        );
    }

    /// **A path that merely contains the arrow is still one path.**
    ///
    /// Nothing in this repository is named like that, which is exactly why it is
    /// worth pinning: the rule reads a rename out of porcelain's own spelling,
    /// and a file whose name happened to contain ` -> ` must not be split into
    /// two paths neither of which anybody named.
    #[test]
    fn an_ordinary_path_is_matched_whole() {
        let named = ["docs/autonomy/updates/one-keyring-fixture-two-crates.md".to_owned()];
        assert!(accounted_for(&named[0], &named));
        assert!(
            !accounted_for("docs/autonomy/updates/something-else.md", &named),
            "a path nobody named was accounted for"
        );
    }
}
