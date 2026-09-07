//! Whether a task's acceptance criteria have a test that actually ran.
//!
//! # The gates say the repository still works, not that the task was done
//!
//! `cargo test --workspace` passing is the state of everything **except** the
//! thing just written: a worker that produced a report, a rename and no
//! enforcement passes every gate in this tool, because the suite it passes is
//! the one that was already there. **Existing tests passing is not acceptance
//! evidence**, and a supervisor that treated it as such would publish
//! documentation of work that was not done — which is exactly the failure the
//! workstream's own rules name.
//!
//! So a handoff names its evidence: for each acceptance criterion, one test,
//! by crate, by test target and by name. This module holds it to two things
//! that together are hard to fake.
//!
//! **The test has to be this task's.** Its file must be among the files the
//! handoff is publishing. A worker cannot point at a test that was already
//! green and call it evidence of what it just wrote — that test's file is not
//! in the change.
//!
//! **The test has to have run, by that name.** It is run again on its own with
//! `--exact`, and the result must be *one* test passing. A name with a typo in
//! it selects nothing and reports zero tests passing, which the whole-workspace
//! run cannot tell apart from success; this can, and refuses it.
//!
//! # What it is not
//!
//! It is not a judge of whether the test is any good. Nothing mechanical can be,
//! and pretending otherwise would be worse than the gap: the report is read by a
//! person and the plan names what each task must show. What this removes is the
//! failure that needs no bad faith at all — a task published on a green suite
//! that never contained a test of it.

use std::path::Path;

use crate::gates;

/// One acceptance criterion's test, as a handoff names it.
#[derive(Debug, PartialEq, Eq)]
pub struct Shown {
    /// The crate it lives in, as `cargo -p` names it.
    pub within: String,

    /// The test target: an integration test's file stem, or `lib` for a unit
    /// test inside the crate's own source.
    pub target: String,

    /// The test's full path, as `cargo test -- --exact` takes it.
    pub named: String,
}

impl Shown {
    /// One line of a handoff's `evidence` block, read.
    ///
    /// `alo-agentd a_question_is_bounded_by_the_kernel a_question_is_answered`
    /// — three words, because three is what it takes to run exactly one test
    /// and no more than one.
    ///
    /// # Errors
    /// A sentence when the line does not have all three.
    pub fn read(line: &str) -> Result<Self, String> {
        let mut words = line.split_whitespace();
        match (words.next(), words.next(), words.next(), words.next()) {
            (Some(within), Some(target), Some(named), None) => Ok(Self {
                within: within.to_owned(),
                target: target.to_owned(),
                named: named.to_owned(),
            }),
            _ => Err(format!(
                "`{line}` is not a piece of evidence. Each line is three words: the crate, the \
                 test target (a test file's stem, or `lib` for a unit test), and the test's full \
                 name."
            )),
        }
    }

    /// The file this test lives in, as a path fragment a staged file must end
    /// with.
    ///
    /// `lib` is the crate's own source and has no single file, so it is matched
    /// by the crate's directory instead: a unit test offered as evidence has to
    /// come with a change somewhere in the crate that holds it.
    fn belongs_to(&self) -> String {
        if self.target == "lib" {
            format!("crates/{}/src/", self.within)
        } else {
            format!("/{}.rs", self.target)
        }
    }

    /// What to run to see this one test, and only this one.
    fn as_arguments(&self) -> Vec<String> {
        let mut args = vec!["test".to_owned(), "-p".to_owned(), self.within.clone()];
        if self.target == "lib" {
            args.push("--lib".to_owned());
        } else {
            args.push("--test".to_owned());
            args.push(self.target.clone());
        }
        args.push("--".to_owned());
        args.push("--exact".to_owned());
        args.push(self.named.clone());
        args
    }
}

/// Hold a handoff's evidence up, and say what stood.
///
/// Run after the gates, on the same tree they passed on, so a failure here is
/// about the task rather than about the repository.
///
/// # Errors
/// A sentence when a task shows no evidence, when a test it names is not in the
/// change it is publishing, or when running that test by name does not produce
/// exactly one passing test.
pub fn stands_up(at: &Path, files: &[String], shown: &[Shown]) -> Result<Vec<String>, String> {
    if shown.is_empty() {
        return Err(
            "the handoff shows no acceptance evidence, and the gates passing is the state of              everything except the thing just written. Name one test per acceptance              criterion in an `evidence` block: crate, test target, test name."
                .to_owned(),
        );
    }

    let mut stood = Vec::new();
    for one in shown {
        let belongs = one.belongs_to();
        if !files.iter().any(|named| {
            let named = named.replace('\\', "/");
            named.ends_with(&belongs) || named.contains(&belongs)
        }) {
            return Err(format!(
                "`{}` is offered as evidence and its test lives in a file this task is not \
                 publishing. A test that was already green is the state of the repository, not \
                 evidence of the work — the test that shows an acceptance criterion has to be \
                 part of the change that claims it.",
                one.named
            ));
        }

        let said = gates::running(at, ".", "cargo", &one.as_arguments())?
            .output()
            .map_err(|why| format!("`{}` could not be run: {why}", one.named))?;
        let printed = format!(
            "{}{}",
            String::from_utf8_lossy(&said.stdout),
            String::from_utf8_lossy(&said.stderr)
        );
        if !said.status.success() || !printed.contains("1 passed") {
            let lines: Vec<&str> = printed.lines().collect();
            let from = lines.len().saturating_sub(20);
            return Err(format!(
                "the evidence `{}` did not stand up: running it on its own did not produce one \
                 passing test, so nothing was published.\n{}",
                one.named,
                lines.get(from..).unwrap_or_default().join("\n")
            ));
        }
        stood.push(format!("{} ({})", one.named, one.within));
    }
    Ok(stood)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Three words, and a line that is not three is refused** rather than
    /// guessed at: a guess would run the wrong test and pass.
    #[test]
    fn a_piece_of_evidence_is_a_crate_a_target_and_a_name() {
        let read = Shown::read("  alo-agentd a_question_is_bounded_by_the_kernel it_answers  ");
        assert_eq!(
            read,
            Ok(Shown {
                within: "alo-agentd".to_owned(),
                target: "a_question_is_bounded_by_the_kernel".to_owned(),
                named: "it_answers".to_owned(),
            })
        );
        assert!(Shown::read("alo-agentd it_answers").is_err());
        assert!(Shown::read("alo-agentd a_target it_answers and_more").is_err());
        assert!(Shown::read("").is_err());
    }

    /// **A test target maps to the file a change has to contain**, which is
    /// what stops an existing green test being offered as evidence of new work.
    #[test]
    fn evidence_has_to_live_in_a_file_the_task_is_publishing() {
        let integration = Shown {
            within: "alo-agentd".to_owned(),
            target: "a_question_is_bounded_by_the_kernel".to_owned(),
            named: "it_answers".to_owned(),
        };
        assert_eq!(
            integration.belongs_to(),
            "/a_question_is_bounded_by_the_kernel.rs"
        );

        let unit = Shown {
            within: "alo-turn".to_owned(),
            target: "lib".to_owned(),
            named: "asking::tests::it_bounds".to_owned(),
        };
        assert_eq!(unit.belongs_to(), "crates/alo-turn/src/");
    }

    /// **A task showing nothing is refused**, because the gates it passed are
    /// the repository's state and not its own.
    #[test]
    fn a_task_that_shows_nothing_is_not_published() {
        let refused = stands_up(Path::new("."), &[], &[]);
        assert!(refused.is_err_and(|why| why.contains("no acceptance evidence")));
    }

    /// And one whose evidence is somewhere the change does not go, before
    /// anything is run: the refusal is about the handoff, not about a test.
    #[test]
    fn evidence_outside_the_change_is_refused_before_anything_runs() {
        let shown = [Shown {
            within: "alo-turn".to_owned(),
            target: "a_test_that_was_already_green".to_owned(),
            named: "it_passed_yesterday_too".to_owned(),
        }];
        let files = ["docs/autonomy/updates/a-report.md".to_owned()];
        let refused = stands_up(Path::new("."), &files, &shown);
        assert!(refused.is_err_and(|why| why.contains("is not publishing")));
    }
}
