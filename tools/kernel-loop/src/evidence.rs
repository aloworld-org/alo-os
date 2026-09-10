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
    /// The workspace it belongs to, relative to the checkout: `.` for the
    /// product's own, and the directory for anything with a workspace of its
    /// own — `tools/kernel-loop` is one, which is why this is written down
    /// rather than assumed. A test in a workspace the root one does not contain
    /// cannot be run from the root at all, and a supervisor whose own tests
    /// could never be offered as evidence would be exempting itself.
    pub at: String,

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
    /// `. alo-agentd a_question_is_bounded_by_the_kernel a_question_is_answered`
    /// — four words, because four is what it takes to run exactly one test and
    /// no more than one, wherever it lives.
    ///
    /// # Errors
    /// A sentence when the line does not have all four.
    pub fn read(line: &str) -> Result<Self, String> {
        let words: Vec<&str> = line.split_whitespace().collect();
        let [at, within, target, named] = words.as_slice() else {
            return Err(format!(
                "`{line}` is not a piece of evidence. Each line is four words: the workspace it \
                 is in (`.` for the product's), the crate, the test target (a test file's stem, \
                 or `lib`/`bin` for a unit test), and the test's full name."
            ));
        };
        Ok(Self {
            at: (*at).to_owned(),
            within: (*within).to_owned(),
            target: (*target).to_owned(),
            named: (*named).to_owned(),
        })
    }

    /// The file this test lives in, as a path fragment a staged file must end
    /// with.
    ///
    /// `lib` and `bin` are the crate's own source and have no single file, so
    /// they are matched by its directory instead: a unit test offered as
    /// evidence has to come with a change somewhere in the crate that holds it.
    fn belongs_to(&self) -> String {
        match self.target.as_str() {
            "lib" | "bin" if self.at == "." => format!("crates/{}/src/", self.within),
            "lib" | "bin" => format!("{}/src/", self.at),
            target => format!("/{target}.rs"),
        }
    }

    /// What to run to see this one test, and only this one.
    fn as_arguments(&self) -> Vec<String> {
        let mut args = vec!["test".to_owned(), "-p".to_owned(), self.within.clone()];
        match self.target.as_str() {
            "lib" => args.push("--lib".to_owned()),
            "bin" => args.push("--bins".to_owned()),
            target => {
                args.push("--test".to_owned());
                args.push(target.to_owned());
            }
        }
        args.push("--".to_owned());
        args.push("--exact".to_owned());
        // **Including the ignored ones**, which is not the loophole it looks
        // like. This runs the one test `--exact` has already narrowed to, named
        // deliberately by a person in a handoff, while the workspace gate keeps
        // running *without* it — so an `#[ignore]`d test stays out of the suite
        // exactly as its author intended.
        //
        // Without this, no `#[ignore]`d test could ever be evidence: run by name
        // it selects nothing, reports zero passing, and is refused by the
        // safeguard below. That would put a whole class of work — the tests
        // needing a quiet machine, a real session, real hardware — permanently
        // beyond publishing, which is the opposite of what the safeguard is for.
        //
        // The cost is real and worth writing down: naming such a test as
        // evidence runs it, and some of them touch the machine. That is the
        // author's decision to make in the handoff, and the plan is where a task
        // says it needs a coordinated window.
        args.push("--include-ignored".to_owned());
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

        let said = gates::running(at, &one.at, "cargo", &one.as_arguments())?
            .output()
            .map_err(|why| format!("`{}` could not be run: {why}", one.named))?;
        let printed = format!(
            "{}{}",
            String::from_utf8_lossy(&said.stdout),
            String::from_utf8_lossy(&said.stderr)
        );
        one_test_passed(said.status.success(), &printed).map_err(|why| {
            let lines: Vec<&str> = printed.lines().collect();
            let from = lines.len().saturating_sub(20);
            format!(
                "the evidence `{}` did not stand up: {why}. Nothing was published.\n{}",
                one.named,
                lines.get(from..).unwrap_or_default().join("\n")
            )
        })?;
        stood.push(format!("{} ({})", one.named, one.within));
    }
    Ok(stood)
}

/// Whether what `cargo test` printed is **one** test, run, and passed.
///
/// Read strictly, from the result line rather than from anywhere the words
/// happen to appear, because the failure this has to catch prints *success*:
///
/// ```text
/// running 0 tests
/// test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out
/// ```
///
/// That is what a name with a typo in it produces, and cargo exits **zero** for
/// it — so an exit-code check waves it through, and so does the whole-workspace
/// run, which cannot tell a test that does not exist from one it never had to
/// look for. A substring search for `1 passed` is not enough either: the
/// `filtered out` count and the timings sit on the same line, and a run of two
/// targets prints two result lines.
///
/// So: exactly one result line, it must say `ok.`, and it must be one passed and
/// none failed.
///
/// # Errors
/// A sentence naming which of those it was.
fn one_test_passed(status_ok: bool, printed: &str) -> Result<(), String> {
    let results: Vec<&str> = printed
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("test result:"))
        .collect();
    let [result] = results.as_slice() else {
        return Err(format!(
            "cargo printed {} test results and exactly one was expected, so what ran is not \
             what was named",
            results.len()
        ));
    };
    if !result.starts_with("test result: ok.") {
        return Err(format!("the test did not pass — cargo said `{result}`"));
    }
    if !result.contains("1 passed") || !result.contains("0 failed") {
        return Err(format!(
            "running it on its own did not produce one passing test — cargo said `{result}`. A \
             name that matches nothing reports zero passed and exits successfully, which is why \
             this is read rather than the exit code"
        ));
    }
    if !status_ok {
        return Err("cargo reported one passing test and still exited unsuccessfully".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Four words, and a line that is not four is refused** rather than
    /// guessed at: a guess would run the wrong test and pass.
    #[test]
    fn a_piece_of_evidence_names_a_workspace_a_crate_a_target_and_a_name() {
        let read = Shown::read("  . alo-agentd a_question_is_bounded_by_the_kernel it_answers  ");
        assert_eq!(
            read,
            Ok(Shown {
                at: ".".to_owned(),
                within: "alo-agentd".to_owned(),
                target: "a_question_is_bounded_by_the_kernel".to_owned(),
                named: "it_answers".to_owned(),
            })
        );
        assert!(Shown::read("alo-agentd a_target it_answers").is_err());
        assert!(Shown::read(". alo-agentd a_target it_answers and_more").is_err());
        assert!(Shown::read("").is_err());
    }

    /// **A named test runs even when it is `#[ignore]`d**, and still only that
    /// one.
    ///
    /// Evidence names a test deliberately; the workspace gate is what decides
    /// whether the *suite* runs it. Without `--include-ignored` here, a test
    /// that needs a quiet machine — a real session, real hardware — selects
    /// nothing when run by name, reports zero passing, and is refused as
    /// missing evidence. `--exact` beside it is what keeps this one test rather
    /// than every ignored one.
    #[test]
    fn a_named_test_is_run_even_when_the_suite_skips_it() {
        let ran = Shown {
            at: ".".to_owned(),
            within: "alo-secrets".to_owned(),
            target: "a_session_that_really_ended".to_owned(),
            named: "one_session_ended_takes_the_bus_with_it".to_owned(),
        }
        .as_arguments();

        assert!(
            ran.contains(&"--include-ignored".to_owned()),
            "an ignored test named as evidence would select nothing: {ran:?}"
        );
        assert!(
            ran.contains(&"--exact".to_owned()),
            "without --exact this would run more than the test it names: {ran:?}"
        );
        // The name is the last word, so nothing is being matched by prefix.
        assert_eq!(
            ran.last().map(String::as_str),
            Some("one_session_ended_takes_the_bus_with_it")
        );
    }

    /// **A test target maps to the file a change has to contain**, which is
    /// what stops an existing green test being offered as evidence of new work.
    #[test]
    fn evidence_has_to_live_in_a_file_the_task_is_publishing() {
        let integration = Shown {
            at: ".".to_owned(),
            within: "alo-agentd".to_owned(),
            target: "a_question_is_bounded_by_the_kernel".to_owned(),
            named: "it_answers".to_owned(),
        };
        assert_eq!(
            integration.belongs_to(),
            "/a_question_is_bounded_by_the_kernel.rs"
        );

        let unit = Shown {
            at: ".".to_owned(),
            within: "alo-turn".to_owned(),
            target: "lib".to_owned(),
            named: "asking::tests::it_bounds".to_owned(),
        };
        assert_eq!(unit.belongs_to(), "crates/alo-turn/src/");

        // A workspace of its own — the supervisor's, which the root workspace
        // does not contain and whose tests would otherwise be unofferable.
        let elsewhere = Shown {
            at: "tools/kernel-loop".to_owned(),
            within: "alo-kernel-loop".to_owned(),
            target: "bin".to_owned(),
            named: "publishing::tests::nothing_is_published".to_owned(),
        };
        assert_eq!(elsewhere.belongs_to(), "tools/kernel-loop/src/");
    }

    /// **A task showing nothing is refused**, because the gates it passed are
    /// the repository's state and not its own.
    #[test]
    fn a_task_that_shows_nothing_is_not_published() {
        let refused = stands_up(Path::new("."), &[], &[]);
        assert!(refused.is_err_and(|why| why.contains("no acceptance evidence")));
    }

    /// **A name that matches no test is refused**, which is the failure that
    /// prints success: cargo reports zero passed and exits zero, so neither the
    /// exit code nor the whole-workspace run can tell it from evidence.
    #[test]
    fn a_name_that_matches_nothing_is_not_evidence() {
        let nothing_ran = "running 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; \
                           0 measured; 5 filtered out; finished in 0.00s\n";
        assert!(
            one_test_passed(true, nothing_ran).is_err_and(|why| why.contains("one passing test")),
            "a test name matching nothing was accepted as evidence"
        );

        let one_ran = "running 1 test\ntest it_answers ... ok\n\ntest result: ok. 1 passed; \
                       0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 2.06s\n";
        assert_eq!(one_test_passed(true, one_ran), Ok(()));
    }

    /// **A failing test is refused**, and so is a run that printed no result at
    /// all — a build error, a bridge that could not reach the machine, a
    /// process killed part way.
    #[test]
    fn a_test_that_failed_or_never_ran_is_not_evidence() {
        let failed = "running 1 test\ntest it_answers ... FAILED\n\ntest result: FAILED. \
                      0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\n";
        assert!(one_test_passed(false, failed).is_err());

        let nothing_at_all = "error[E0432]: unresolved import `alo_asking`\n";
        assert!(
            one_test_passed(false, nothing_at_all).is_err_and(|why| why.contains("0 test results")),
            "a run that never got as far as a test was accepted"
        );

        // Two targets is two result lines, and neither of them is *the* answer.
        let two = "test result: ok. 1 passed; 0 failed\ntest result: ok. 0 passed; 0 failed\n";
        assert!(one_test_passed(true, two).is_err());

        // And cargo agreeing with itself is required: one passing test beside a
        // non-zero exit is a machine saying two things at once.
        let disagreeing = "test result: ok. 1 passed; 0 failed; 0 ignored\n";
        assert!(one_test_passed(false, disagreeing).is_err());
    }

    /// And one whose evidence is somewhere the change does not go, before
    /// anything is run: the refusal is about the handoff, not about a test.
    #[test]
    fn evidence_outside_the_change_is_refused_before_anything_runs() {
        let shown = [Shown {
            at: ".".to_owned(),
            within: "alo-turn".to_owned(),
            target: "a_test_that_was_already_green".to_owned(),
            named: "it_passed_yesterday_too".to_owned(),
        }];
        let files = ["docs/autonomy/updates/a-report.md".to_owned()];
        let refused = stands_up(Path::new("."), &files, &shown);
        assert!(refused.is_err_and(|why| why.contains("is not publishing")));
    }
}
