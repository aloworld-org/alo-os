//! Gate, hold the evidence up, commit, integrate what arrived, gate the
//! combination, push.
//!
//! # It fails closed, and that is the whole of what this file is for
//!
//! Every road through [`gated_and_pushed`] either publishes a task that passed
//! **every** check, or publishes nothing. There is no step that runs after a
//! failed one, no result that is looked at and stepped over, and no push that
//! happens because the thing before it was never asked how it went.
//!
//! That is worth saying out loud because the failure it prevents is not
//! hypothetical. On 2026-09-07 this task's own author gated the combined tree
//! and pushed in one shell line, the two joined by a newline rather than by a
//! check of the first command's result. The gates had failed — the machine had
//! lost its BPF filesystem between runs — and the push went out anyway. The
//! change was sound and the sequence was not, and **the sequence is what a
//! supervisor is for.** Since then there is no hand-rolled publication path at
//! all: `publish` is a subcommand of this program, so a person recovering by
//! hand walks through the same checks in the same order as the loop.
//!
//! # Gates, and then the task's own evidence
//!
//! The gates say the repository still works. They cannot say this task was
//! done: the suite they pass is the one that was already there. So each gating
//! is followed by [`crate::evidence`], which runs the test behind each
//! acceptance criterion on its own and refuses a task whose evidence is not part
//! of the change it is publishing. **Passing existing tests is never taken as
//! completed implementation.**
//!
//! # The second gate is the one that matters
//!
//! Gating before the rebase says this task works. Gating **after** it says this
//! task works *beside whatever the other worker published while it was being
//! written*, and only the second is a claim worth making about `main`. Two
//! changes that each pass alone and fail together is not a hypothetical; it is
//! the ordinary way a shared branch breaks.
//!
//! # A lost race, and the thing that is not one
//!
//! A push that loses to somebody else's is a race: integrate again, gate the
//! combination, retry, three times. A push refused while `origin/main` has
//! **not** moved is not a race — it is a real push error, and retrying it would
//! be a supervisor waiting for a network to heal while holding a lock.
//! `docs/autonomy/SHARED_MAIN.md` says exactly that, and this stops instead,
//! with the commit sitting locally and intact.
//!
//! # Nothing is discarded, ever
//!
//! A failed gate leaves the tree as it was. A conflicted rebase is left where it
//! stopped. A refused push leaves the commit local. There is no `reset`, no
//! `--force` and no `--abort` anywhere in this program — [`crate::repository`]
//! documents that absence as its subject.

use std::path::Path;

use crate::{evidence, gates, handoff::Handed, journal, repository};

/// How many times a lost race is worth answering before somebody should look.
const TIMES: u8 = 3;

/// Every step publication is made of, so that the order they happen in is a
/// thing this crate's own tests can check.
///
/// The real implementation runs gates, cargo and git; a test's runs nothing and
/// records what it was asked for. **The algorithm in [`gated_and_pushed`] is the
/// same one in both cases**, which is the only arrangement under which a test
/// saying *a failed gate publishes nothing* is evidence about the program a
/// person actually runs.
pub trait Steps {
    /// Every gate, and then this task's acceptance evidence. `which` names the
    /// tree being checked, for the log.
    ///
    /// # Errors
    /// A sentence naming the gate or the piece of evidence that did not pass.
    fn check(&mut self, which: &str) -> Result<(), String>;

    /// Stage exactly the files the task named, and nothing else.
    ///
    /// # Errors
    /// Whatever `git` said.
    fn stage(&mut self) -> Result<(), String>;

    /// Commit what is staged; the short sha comes back.
    ///
    /// # Errors
    /// Whatever `git` said.
    fn commit(&mut self) -> Result<String, String>;

    /// Whether `origin/main` has anything this checkout does not.
    ///
    /// # Errors
    /// Whatever `git` said about fetching or counting.
    fn advanced(&mut self) -> Result<bool, String>;

    /// Put this checkout's unpublished commits on top of what arrived.
    ///
    /// # Errors
    /// A sentence when the rebase stops, which is a real conflict. It is left
    /// where it stopped.
    fn rebase(&mut self) -> Result<(), String>;

    /// Push. An ordinary push, and never a forced one; the published sha comes
    /// back.
    ///
    /// # Errors
    /// Whatever `git` said, which for a lost race is the fast-forward hint.
    fn push(&mut self) -> Result<String, String>;

    /// Put the handoff away, so a published task is never published twice.
    ///
    /// # Errors
    /// Whatever the machine said.
    fn put_away(&mut self, sha: &str) -> Result<(), String>;

    /// Write one line in the loop's own log.
    fn note(&mut self, said: &str);
}

/// One task, all the way onto `main` — or nowhere at all.
///
/// # Errors
/// A sentence naming what stopped it, at whichever step. **Work is never
/// discarded on any of these roads**: a failed gate leaves the tree as it was, a
/// conflicted rebase is left where it stopped, and a lost or refused push leaves
/// the commit sitting locally for the next attempt.
pub fn gated_and_pushed(steps: &mut dyn Steps) -> Result<String, String> {
    // Nothing is staged before this returns `Ok`. A failure here leaves a tree
    // the supervisor has not touched, which is what makes the work recoverable
    // by reading it rather than by finding it.
    steps.check("this task's tree")?;

    steps.stage()?;
    let sha = steps.commit()?;
    steps.note(&format!("committed {sha} locally"));

    for attempt in 1..=TIMES {
        if steps.advanced()? {
            steps.note(&format!(
                "main advanced; rebasing and gating the combined tree (attempt {attempt})"
            ));
            steps.rebase()?;
            // **The combination is checked before it is pushed, every time.** A
            // failure here leaves the rebased commit local and unpublished.
            steps.check("the combined tree")?;
        }

        match steps.push() {
            Ok(published) => {
                steps.put_away(&published)?;
                return Ok(published);
            }
            Err(why) => {
                if !steps.advanced()? {
                    return Err(format!(
                        "the push was refused and `origin/main` has not moved, so this is a real \
                         push error rather than a lost race. The commit is in this checkout, \
                         unpublished and intact; nothing was discarded: {why}"
                    ));
                }
                if attempt == TIMES {
                    return Err(format!(
                        "the push did not go through after {TIMES} attempts, and the commit is \
                         sitting in this checkout unpublished — nothing was discarded: {why}"
                    ));
                }
                steps.note(&format!(
                    "the push lost a race, integrating and trying again: {why}"
                ));
            }
        }
    }

    Err(format!(
        "publication was not reached in {TIMES} attempts; the work is committed locally and \
         intact"
    ))
}

/// The steps as this machine really performs them.
pub struct OnThisMachine<'a> {
    /// The checkout.
    at: &'a Path,

    /// The loop's own directory, where the log and the handoff live.
    ours: &'a Path,

    /// What is being published.
    task: &'a Handed,
}

impl<'a> OnThisMachine<'a> {
    /// One of these, for one task.
    #[must_use]
    pub const fn publishing(at: &'a Path, ours: &'a Path, task: &'a Handed) -> Self {
        Self { at, ours, task }
    }
}

impl Steps for OnThisMachine<'_> {
    fn check(&mut self, which: &str) -> Result<(), String> {
        journal::note(
            self.ours,
            &format!("checking {which}; the report is {}", self.task.report),
        );
        let passed = gates::all_of_them(self.at, &self.task.files)?;
        journal::note(self.ours, &format!("{which} passed: {}", passed.join("; ")));
        let stood = evidence::stands_up(self.at, &self.task.files, &self.task.evidence)?;
        journal::note(
            self.ours,
            &format!("the evidence stood up: {}", stood.join("; ")),
        );
        Ok(())
    }

    fn stage(&mut self) -> Result<(), String> {
        repository::staged(self.at, &self.task.files)
    }

    fn commit(&mut self) -> Result<String, String> {
        repository::committed(self.at, &self.task.message())
    }

    fn advanced(&mut self) -> Result<bool, String> {
        repository::advanced(self.at)
    }

    fn rebase(&mut self) -> Result<(), String> {
        repository::rebased_onto_origin(self.at)
    }

    fn push(&mut self) -> Result<String, String> {
        repository::pushed(self.at)?;
        repository::git(self.at, &["rev-parse", "--short", "HEAD"])
    }

    fn put_away(&mut self, sha: &str) -> Result<(), String> {
        Handed::put_away(self.ours, sha)
    }

    fn note(&mut self, said: &str) {
        journal::note(self.ours, said);
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Every step, recorded rather than done, with any one of them able to
    /// refuse.
    ///
    /// The point of it is the `did` list: a test about publication is a test
    /// about **what did not happen**, and a double that reported only its return
    /// values could not say whether `stage` was ever reached.
    #[derive(Debug, Default)]
    struct Recording {
        /// What was asked for, in order.
        did: Vec<String>,

        /// Which step refuses, if any, by the name it is recorded under.
        refusing: Option<&'static str>,

        /// What `advanced` answers, one call at a time, then `false`.
        advancing: Vec<bool>,

        /// How many pushes lose a race before one goes through.
        pushes_refused: u8,
    }

    impl Recording {
        /// Note the step and answer according to what this double was set up to
        /// do.
        fn doing(&mut self, named: &'static str) -> Result<(), String> {
            self.did.push(named.to_owned());
            if self.refusing == Some(named) {
                return Err(format!("`{named}` was set up to refuse"));
            }
            Ok(())
        }
    }

    impl Steps for Recording {
        fn check(&mut self, which: &str) -> Result<(), String> {
            self.did.push(format!("check {which}"));
            if self.refusing == Some("check") || self.refusing == Some(which) {
                return Err(format!("`{which}` did not pass"));
            }
            Ok(())
        }

        fn stage(&mut self) -> Result<(), String> {
            self.doing("stage")
        }

        fn commit(&mut self) -> Result<String, String> {
            self.doing("commit").map(|()| "abc1234".to_owned())
        }

        fn advanced(&mut self) -> Result<bool, String> {
            self.did.push("advanced".to_owned());
            Ok(if self.advancing.is_empty() {
                false
            } else {
                self.advancing.remove(0)
            })
        }

        fn rebase(&mut self) -> Result<(), String> {
            self.doing("rebase")
        }

        fn push(&mut self) -> Result<String, String> {
            self.did.push("push".to_owned());
            if self.pushes_refused > 0 {
                self.pushes_refused -= 1;
                return Err("Updates were rejected because the remote contains work".to_owned());
            }
            if self.refusing == Some("push") {
                return Err("`push` was set up to refuse".to_owned());
            }
            Ok("abc1234".to_owned())
        }

        fn put_away(&mut self, _sha: &str) -> Result<(), String> {
            self.doing("put_away")
        }

        fn note(&mut self, _said: &str) {}
    }

    /// **A failed gate publishes nothing, and stages nothing.**
    ///
    /// The first check comes before the first `git add` for this reason: a
    /// supervisor that staged first would leave a tree somebody has to unstage
    /// before they can even read what happened.
    #[test]
    fn a_failed_gate_stages_nothing_and_pushes_nothing() {
        let mut steps = Recording {
            refusing: Some("check"),
            ..Recording::default()
        };
        let went = gated_and_pushed(&mut steps);

        assert!(went.is_err(), "a failed gate published: {went:?}");
        assert_eq!(steps.did, ["check this task's tree"]);
    }

    /// **A failed check on the combined tree publishes nothing**, and the commit
    /// already made is left where it is.
    ///
    /// This is what the second gating exists for: the task passed on its own,
    /// `main` moved, and the two together do not work.
    #[test]
    fn a_failed_combined_tree_check_leaves_the_commit_unpublished() {
        let mut steps = Recording {
            refusing: Some("the combined tree"),
            advancing: vec![true],
            ..Recording::default()
        };
        let went = gated_and_pushed(&mut steps);

        assert!(went.is_err(), "a broken combination published: {went:?}");
        assert_eq!(
            steps.did,
            [
                "check this task's tree",
                "stage",
                "commit",
                "advanced",
                "rebase",
                "check the combined tree",
            ]
        );
    }

    /// **A conflicted rebase stops there**, unpushed and un-aborted, and the
    /// combination is not gated over a rebase that never finished.
    #[test]
    fn a_conflicted_rebase_publishes_nothing() {
        let mut steps = Recording {
            refusing: Some("rebase"),
            advancing: vec![true],
            ..Recording::default()
        };
        let went = gated_and_pushed(&mut steps);

        assert!(went.is_err());
        assert!(!steps.did.iter().any(|what| what == "push"));
        assert!(
            !steps
                .did
                .iter()
                .any(|what| what == "check the combined tree")
        );
    }

    /// **A lost race is answered by integrating and gating again**, not by
    /// pushing again at whatever happens to be in the tree.
    #[test]
    fn a_lost_race_is_re_gated_before_the_second_attempt() {
        let mut steps = Recording {
            advancing: vec![false, true, true],
            pushes_refused: 1,
            ..Recording::default()
        };
        let published = gated_and_pushed(&mut steps).unwrap();

        assert_eq!(published, "abc1234");
        assert_eq!(
            steps.did,
            [
                "check this task's tree",
                "stage",
                "commit",
                "advanced",
                "push",
                "advanced",
                "advanced",
                "rebase",
                "check the combined tree",
                "push",
                "put_away",
            ]
        );
    }

    /// **A refused push with an unmoved remote stops at once** rather than
    /// retrying, which is `SHARED_MAIN.md`'s rule about what a rejection means.
    #[test]
    fn a_push_error_that_is_not_a_race_stops_at_once() {
        let mut steps = Recording {
            refusing: Some("push"),
            ..Recording::default()
        };
        let went = gated_and_pushed(&mut steps);

        assert!(went.is_err_and(|why| why.contains("real push error")));
        assert_eq!(
            steps.did.iter().filter(|what| *what == "push").count(),
            1,
            "a push error that was not a race was retried"
        );
    }

    /// And the ordinary road: everything passes, once, in order.
    #[test]
    fn a_task_that_passes_everything_is_published_once() {
        let mut steps = Recording::default();
        let published = gated_and_pushed(&mut steps).unwrap();

        assert_eq!(published, "abc1234");
        assert_eq!(
            steps.did,
            [
                "check this task's tree",
                "stage",
                "commit",
                "advanced",
                "push",
                "put_away",
            ]
        );
    }
}
