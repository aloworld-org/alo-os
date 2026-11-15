//! **The mutation check breaks the code a pull request changes, and says out
//! loud when no test noticed.**
//!
//! Law 8, which pull request #1270 adds to `CLAUDE.md` with the owner's
//! decision of 2026-11-15: *a test must be able to fail.* The code
//! a change touches is broken on purpose with `cargo-mutants`, and a break no
//! test notices is reported with the change. `.github/workflows/mutants.yml` is
//! what does that on every pull request, and a workflow is the file nobody reads
//! twice: the edit that makes it test nothing, or test everything and never
//! finish, or go green whatever happened, is one line.
//!
//! So this reads it with [`TheWorkflow`], the reader `image.yml` is held by — the
//! lines that are not comments, because the comment that explains why a word is
//! absent is exactly where the word belongs — and holds it to what it promises.
//! Every rule is a function of the text, so each is shown to refuse a workflow
//! that breaks it as well as to accept the one that ships. A rule that could not
//! refuse anything would be the fault law 8 is about, in the test written for it.

use alo_image::TheWorkflow;

/// The workflow, from this repository.
const THE_CHECK: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../.github/workflows/mutants.yml"
);

/// The gate's workflow, whose list of libraries this one must install too.
const THE_GATE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../.github/workflows/gate.yml"
);

/// The gates themselves, whose test filter says which tests a hosted runner
/// runs.
const THE_GATES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tools/kernel-loop/src/gates.rs"
);

/// The events it runs on: every pull request, and a person asking for one pull
/// request by number.
///
/// **Not the merge queue.** Breaking code one change at a time takes tens of
/// minutes; whether a merge waits for it is the owner's setting, and a queue
/// check nobody required would only make every landing slower.
const TRIGGERS: [&str; 2] = ["pull_request", "workflow_dispatch"];

/// What the run must never do.
///
/// Write with its token, read a secret, run with the base repository's secrets
/// on a stranger's branch, or turn a failure into a pass. And **skip the
/// baseline**: cargo-mutants first runs the tests on the unbroken code, and
/// without that a test that already fails would "notice" every break, so a
/// broken suite would read as a perfect one.
const FORBIDDEN: [&str; 12] = [
    "contents: write",
    "statuses: write",
    "checks: write",
    "pull-requests: write",
    "packages: write",
    "id-token: write",
    "secrets.",
    "pull_request_target",
    "continue-on-error",
    "--baseline=skip",
    "--baseline skip",
    "cargo install cargo-mutants",
];

/// The files the diff leaves out, as `git diff` pathspecs, and why.
///
/// Test code is what notices a break, not what is broken: `tests/`, the
/// `*_tests.rs` files beside the code they test, and `tests.rs` modules. A build
/// script is code that writes code. `alo-bounding-kernel` is compiled for the
/// kernel's BPF machine and has no tests a runner can run. Documentation is not
/// a `.rs` file and never reaches the diff.
const LEFT_OUT: [&str; 7] = [
    "':(exclude,glob)**/tests/**'",
    "':(exclude,glob)**/*_tests.rs'",
    "':(exclude,glob)**/tests.rs'",
    "':(exclude,glob)**/build.rs'",
    "':(exclude,glob)**/benches/**'",
    "':(exclude,glob)**/examples/**'",
    "':(exclude)crates/alo-bounding-kernel'",
];

/// The pathspec that keeps Rust and nothing else.
const ONLY_RUST: &str = "':(glob)**/*.rs'";

/// What reads the change: the merge GitHub makes of the pull request, against
/// its first parent, which is main as the merge found it — so the line numbers
/// in the diff are the line numbers of the tree that is built.
const THE_DIFF: &str = "git diff HEAD^1 HEAD --";

/// The file that names the tests a hosted runner cannot run.
const CANNOT_RUN: &str = ".github/the-tests-a-hosted-runner-cannot-run.txt";

/// What fails the job when a break went unnoticed.
const SURVIVORS_FAIL: &str = "[ \"$missed\" -eq 0 ] || exit 1";

/// How each survivor is written into the summary: one row per line of
/// cargo-mutants' own list of them, ending in the words a person reads.
///
/// The whole row and not the words alone: *no test noticed* is also a heading
/// and a count, and a rule satisfied by the heading passed while the rows were
/// gone — this file's own first run showed it.
const NOBODY_NOTICED: &str = "| no test noticed |/' mutants.out/missed.txt";

/// The output that says the pull request changed Rust this check can break.
const CHANGED: &str = "steps.changed.outputs.rust == 'true'";

/// The step that breaks the code.
const THE_RUN: &str = "- name: Break the changed code, one change at a time";

/// The one directory that may leave the runner.
const THE_RESULTS: &str = "path: mutants.out";

/// The longest the results may be kept, in days.
const KEPT_AT_MOST: u32 = 7;

/// A file from this repository, as text; empty if it cannot be read, which the
/// first test refuses.
fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// The shipped workflow.
fn shipped() -> String {
    read(THE_CHECK)
}

/// The gate's test filter, joined the way Rust joins a string continued with
/// `\` at the end of a line.
fn the_gates_filter() -> Option<String> {
    let gates = read(THE_GATES);
    let after = gates.split_once("\"-E\",")?.1;
    let start = after.find('"')? + 1;
    let end = start + after[start..].find("\",")?;
    let mut joined = String::new();
    let mut lines = after[start..end].split("\\\n");
    if let Some(first) = lines.next() {
        joined.push_str(first);
    }
    for rest in lines {
        joined.push_str(rest.trim_start());
    }
    Some(joined)
}

/// Every package the gate leaves out of its tests, by name.
///
/// A package whose tests are never run would report every break in it as
/// unnoticed, so the check must not break code there either.
fn left_out_of_the_gate(filter: &str) -> Vec<String> {
    filter
        .split("package(")
        .skip(1)
        .filter_map(|it| it.split_once(')').map(|(name, _)| name.to_owned()))
        .collect()
}

/// The lines between `packages=(` and `)`, trimmed: the libraries a workflow
/// installs.
fn packages(text: &str) -> Vec<String> {
    let read = TheWorkflow::read(text);
    read.live()
        .iter()
        .skip_while(|it| it.as_str() != "packages=(")
        .skip(1)
        .take_while(|it| it.as_str() != ")")
        .flat_map(|it| it.split_whitespace().map(str::to_owned).collect::<Vec<_>>())
        .collect()
}

/// The lines of one step, from its `- name:` up to the next step.
fn step<'a>(live: &'a [String], named: &str) -> Vec<&'a String> {
    live.iter()
        .skip_while(|it| it.as_str() != named)
        .skip(1)
        .take_while(|it| !it.starts_with("- "))
        .collect()
}

/// Everything this workflow does that the check may not, one sentence each. An
/// empty list is a workflow that breaks the changed code, under a time limit,
/// and fails when no test noticed.
fn what_is_wrong_with(text: &str, gate: &str) -> Vec<String> {
    let read = TheWorkflow::read(text);
    let live = read.live();
    let mut wrong = Vec::new();

    if read.triggers() != TRIGGERS {
        wrong.push(format!(
            "it runs on {:?}, not on pull requests and a person asking alone",
            read.triggers()
        ));
    }
    for line in live {
        for word in FORBIDDEN {
            if line.contains(word) {
                wrong.push(format!("{word:?} in: {line}"));
            }
        }
    }
    wrong.extend(what_is_wrong_with_the_pin(live));
    wrong.extend(what_is_wrong_with_the_diff(live));
    wrong.extend(what_is_wrong_with_the_run(live));
    wrong.extend(what_leaves_the_runner(live));

    if !live.iter().any(|it| it.contains(SURVIVORS_FAIL)) {
        wrong.push("a break no test noticed does not fail the job".to_owned());
    }
    if !live.iter().any(|it| it.contains(NOBODY_NOTICED)) {
        wrong.push(format!("a survivor is not reported as {NOBODY_NOTICED:?}"));
    }
    if !live.iter().any(|it| it.contains("rust=false")) {
        wrong.push("a pull request with no Rust to break is not skipped by name".to_owned());
    }
    let gate_packages = packages(gate);
    if gate_packages.is_empty() || packages(text) != gate_packages {
        wrong.push(format!(
            "it installs {:?}, not the gate's libraries {gate_packages:?}",
            packages(text)
        ));
    }
    wrong
}

/// One cargo-mutants, by exact version: a new release may break code in new
/// ways, and a check that changes its mind on its own is not one anybody can
/// read twice.
fn what_is_wrong_with_the_pin(live: &[String]) -> Vec<String> {
    // Every line that could install it: an action's `tool:`, or any install.
    let pins: Vec<&String> = live
        .iter()
        .filter(|it| {
            it.contains("cargo-mutants") && (it.contains("tool:") || it.contains("install"))
        })
        .collect();
    let exact = |line: &str| {
        line.strip_prefix("tool: cargo-mutants@")
            .is_some_and(|version| {
                let parts: Vec<&str> = version.split('.').collect();
                parts.len() == 3
                    && parts
                        .iter()
                        .all(|it| !it.is_empty() && it.bytes().all(|b| b.is_ascii_digit()))
            })
    };
    if pins.len() == 1 && pins.iter().all(|it| exact(it)) {
        Vec::new()
    } else {
        vec![format!(
            "cargo-mutants is not installed once at an exact version: {pins:?}"
        )]
    }
}

/// The diff is the pull request's Rust code under test, and nothing else.
fn what_is_wrong_with_the_diff(live: &[String]) -> Vec<String> {
    let mut wrong = Vec::new();
    if !live.iter().any(|it| it.starts_with(THE_DIFF)) {
        wrong.push(format!("the change is not read with {THE_DIFF:?}"));
    }
    if !live.iter().any(|it| it == "fetch-depth: 2") {
        wrong.push("the checkout does not hold the merge's first parent".to_owned());
    }
    if !live.iter().any(|it| it == ONLY_RUST) {
        wrong.push("the diff is not limited to Rust files".to_owned());
    }
    for spec in LEFT_OUT {
        if !live.iter().any(|it| it == spec) {
            wrong.push(format!("the diff does not leave out {spec}"));
        }
    }
    match the_gates_filter() {
        None => wrong.push(format!(
            "the gate's test filter could not be read from {THE_GATES}"
        )),
        Some(filter) => {
            for package in left_out_of_the_gate(&filter) {
                let spec = format!("':(exclude)crates/{package}'");
                if !live.contains(&spec) {
                    wrong.push(format!(
                        "{package} is left out of the gate's tests and not out of the diff, so \
                         every break in it would read as unnoticed"
                    ));
                }
            }
        }
    }
    wrong
}

/// The run breaks only the changed code, under a time limit, with the tests the
/// gate runs, and only when there is something to break.
fn what_is_wrong_with_the_run(live: &[String]) -> Vec<String> {
    let mut wrong = Vec::new();
    let runs: Vec<&String> = live
        .iter()
        .filter(|it| it.contains("cargo mutants "))
        .collect();
    if runs.is_empty() {
        wrong.push("it never runs cargo mutants".to_owned());
    }
    for run in &runs {
        if !run.contains("--in-diff") {
            wrong.push(format!("a run that is not limited to the change: {run}"));
        }
    }
    if !runs
        .iter()
        .any(|it| !it.contains("--list") && it.contains("timeout ") && it.contains("$budget"))
    {
        wrong.push("the run has no time limit of its own".to_owned());
    }
    match the_gates_filter() {
        Some(filter) if live.iter().any(|it| *it == format!("filter=\"{filter}\"")) => {}
        _ => wrong.push("the tests are not chosen by the gate's own filter".to_owned()),
    }
    if !live.iter().any(|it| it.contains(CANNOT_RUN)) {
        wrong.push(format!("the tests named in {CANNOT_RUN} are not left out"));
    }
    if !step(live, THE_RUN)
        .iter()
        .any(|it| it.starts_with("if:") && it.contains(CHANGED))
    {
        wrong.push("the run does not wait for a change it can break".to_owned());
    }
    if !live
        .iter()
        .any(|it| it.starts_with("timeout-minutes:") && it != "timeout-minutes: 0")
    {
        wrong.push("the job has no time limit".to_owned());
    }
    wrong
}

/// Only the results leave the runner, kept no longer than a week.
fn what_leaves_the_runner(live: &[String]) -> Vec<String> {
    let mut wrong = Vec::new();
    for (at, line) in live.iter().enumerate() {
        if !line.contains("actions/upload-artifact") {
            continue;
        }
        let settings: Vec<&String> = live
            .iter()
            .skip(at + 1)
            .take_while(|it| !it.starts_with("- "))
            .collect();
        let paths: Vec<&&String> = settings
            .iter()
            .filter(|it| it.starts_with("path:"))
            .collect();
        if paths.len() != 1 || paths.iter().any(|it| it.as_str() != THE_RESULTS) {
            wrong.push(format!("an upload that is not the results: {settings:?}"));
        }
        let kept = settings
            .iter()
            .find_map(|it| it.strip_prefix("retention-days:"))
            .and_then(|days| days.trim().parse::<u32>().ok());
        if !kept.is_some_and(|days| days <= KEPT_AT_MOST) {
            wrong.push(format!(
                "results kept longer than {KEPT_AT_MOST} days: {settings:?}"
            ));
        }
    }
    wrong
}

/// The shipped workflow with one thing changed, after checking the thing to
/// change is there — so a change that silently did nothing cannot read as a
/// refusal.
///
/// **In a line that is not a comment**, for a change within one line. The
/// header explains most of what the rules hold, in the same words, so the
/// first occurrence of a phrase is usually the comment; changing that would
/// leave the workflow as it was, and the test asserting a refusal would fail
/// for a reason that has nothing to do with the rule. That happened, twice,
/// on this file's first run.
fn with(from: &str, to: &str) -> Option<String> {
    let text = shipped();
    if from.contains('\n') {
        return text.contains(from).then(|| text.replacen(from, to, 1));
    }
    let mut changed = false;
    let lines: Vec<String> = text
        .split_inclusive('\n')
        .map(|line| {
            if !changed && !line.trim_start().starts_with('#') && line.contains(from) {
                changed = true;
                line.replacen(from, to, 1)
            } else {
                line.to_owned()
            }
        })
        .collect();
    changed.then(|| lines.concat())
}

/// Whether a changed workflow is refused.
fn refused(changed: Option<&str>) -> bool {
    changed.is_some_and(|it| !what_is_wrong_with(it, &read(THE_GATE)).is_empty())
}

/// **The shipped check breaks the changed code under a time limit and fails
/// when no test noticed** — so the refusals below are about the rules rather
/// than a reader that never worked. An absent workflow is refused here.
#[test]
fn the_shipped_check_breaks_the_changed_code_and_fails_on_a_survivor() {
    let text = shipped();
    assert!(!text.is_empty(), "{THE_CHECK} could not be read");
    let wrong = what_is_wrong_with(&text, &read(THE_GATE));
    assert!(wrong.is_empty(), "{wrong:?}");
}

/// **The rules read real things**: the gate's filter leaves packages out, and
/// the gate installs libraries. If either reading came back empty, every rule
/// built on it would pass by having nothing to check.
#[test]
fn what_the_rules_are_read_from_is_there() {
    let filter = the_gates_filter().unwrap_or_default();
    assert!(
        left_out_of_the_gate(&filter).contains(&"alo-agentd".to_owned()),
        "the gate's filter was read as {filter:?}"
    );
    assert!(
        packages(&read(THE_GATE)).contains(&"libwayland-dev".to_owned()),
        "the gate's libraries were read as {:?}",
        packages(&read(THE_GATE))
    );
}

/// **A survivor that does not fail the job, or is not reported, is refused.**
#[test]
fn a_check_that_stays_green_on_a_survivor_is_refused() {
    for changed in [
        with(SURVIVORS_FAIL, "true"),
        with(SURVIVORS_FAIL, "[ \"$missed\" -eq 0 ] || exit 0"),
        with(
            "    runs-on: ubuntu-24.04\n",
            "    runs-on: ubuntu-24.04\n    continue-on-error: true\n",
        ),
        with(NOBODY_NOTICED, "|/' mutants.out/caught.txt"),
        with(
            "cargo mutants --in-diff",
            "cargo mutants --baseline=skip --in-diff",
        ),
    ] {
        assert!(refused(changed.as_deref()), "{changed:?}");
    }
}

/// **A run over the whole workspace, or with no time limit, is refused**: it
/// would take hours and test code nobody changed.
#[test]
fn a_check_that_breaks_more_than_the_change_or_never_stops_is_refused() {
    let run = shipped()
        .lines()
        .map(str::trim)
        .find(|it| it.contains("cargo mutants ") && !it.contains("--list"))
        .map(str::to_owned)
        .unwrap_or_default();
    assert!(!run.is_empty(), "no run of cargo mutants in {THE_CHECK}");
    for changed in [
        with(
            &run,
            &run.replace("--in-diff \"$RUNNER_TEMP/changed.diff\"", "--workspace"),
        ),
        with(&run, &run.replacen("timeout ", "", 1)),
        with("timeout-minutes:", "timeout-minutes: 0\n    x:"),
    ] {
        assert!(refused(changed.as_deref()), "{changed:?}");
    }
}

/// **A diff that keeps test code, or reads the wrong pair of trees, is
/// refused.** Breaking a test is not breaking the code; diffing against the
/// pull request's own head reads line numbers from a tree that is not built.
#[test]
fn a_diff_that_reaches_tests_or_the_wrong_tree_is_refused() {
    for spec in LEFT_OUT {
        assert!(refused(with(&format!("{spec}\n"), "").as_deref()), "{spec}");
    }
    for changed in [
        with(ONLY_RUST, "'*'"),
        with(THE_DIFF, "git diff origin/main --"),
        with("fetch-depth: 2", "fetch-depth: 1"),
        with("':(exclude)crates/alo-agentd'\n", ""),
    ] {
        assert!(refused(changed.as_deref()), "{changed:?}");
    }
}

/// **Tests chosen by anything but the gate's filter are refused**, and so is
/// forgetting the tests this runner cannot run: either would make a break look
/// noticed, or unnoticed, for a reason that has nothing to do with the code.
#[test]
fn tests_not_chosen_as_the_gate_chooses_them_are_refused() {
    let filter = the_gates_filter().unwrap_or_default();
    for changed in [
        with(&format!("filter=\"{filter}\""), "filter=\"all()\""),
        with(CANNOT_RUN, ".github/nothing.txt"),
    ] {
        assert!(refused(changed.as_deref()), "{changed:?}");
    }
}

/// **An unpinned or twice-installed cargo-mutants is refused.**
#[test]
fn a_cargo_mutants_that_is_not_pinned_is_refused() {
    let pin = shipped()
        .lines()
        .map(str::trim)
        .find(|it| it.starts_with("tool: cargo-mutants@"))
        .map(str::to_owned)
        .unwrap_or_default();
    assert!(!pin.is_empty(), "no pin in {THE_CHECK}");
    for changed in [
        with(&pin, "tool: cargo-mutants"),
        with(&pin, "tool: cargo-mutants@27"),
        with(&pin, "tool: cargo-mutants@latest"),
        with(
            &pin,
            &format!("{pin}\n        run: cargo install cargo-mutants"),
        ),
    ] {
        assert!(refused(changed.as_deref()), "{changed:?}");
    }
}

/// **A check on other events, or one that may write, is refused.**
#[test]
fn a_check_on_other_events_or_with_a_writing_token_is_refused() {
    for changed in [
        with("  pull_request:\n", "  pull_request_target:\n"),
        with("  pull_request:\n", "  pull_request:\n  merge_group:\n"),
        with("  pull_request:\n", "  pull_request:\n  push:\n"),
        with("  contents: read\n", "  contents: write\n"),
        with(
            "  contents: read\n",
            "  contents: read\n  pull-requests: write\n",
        ),
    ] {
        assert!(refused(changed.as_deref()), "{changed:?}");
    }
}

/// **A run that does not wait for Rust to break, or skips without saying so,
/// is refused**, and so is uploading more than the results.
#[test]
fn a_run_with_nothing_to_break_or_an_upload_beyond_the_results_is_refused() {
    let run_if = step(TheWorkflow::read(&shipped()).live(), THE_RUN)
        .into_iter()
        .find(|it| it.starts_with("if:"))
        .cloned()
        .unwrap_or_default();
    assert!(!run_if.is_empty(), "the run has no condition");
    // The run's own condition, by the step's name: the same condition guards
    // the steps before it, and changing one of those would prove nothing.
    for changed in [
        with(
            &format!("{THE_RUN}\n        {run_if}"),
            &format!("{THE_RUN}\n        if: always()"),
        ),
        with("rust=false", "rust=true"),
        with(THE_RESULTS, "path: target"),
        with("retention-days: 7", "retention-days: 90"),
        with("libwayland-dev", "libwayland-client0"),
    ] {
        assert!(refused(changed.as_deref()), "{changed:?}");
    }
}

/// **A comment is not a command.** The sentence explaining why the baseline is
/// never skipped names the flag, and must not be read as using it.
#[test]
fn a_comment_about_skipping_the_baseline_is_not_skipping_it() {
    let changed = with(
        "name: Mutants\n",
        "# nothing here passes --baseline=skip or reads secrets.\nname: Mutants\n",
    );
    assert!(changed.is_some(), "the check is not named Mutants");
    let wrong = what_is_wrong_with(&changed.unwrap_or_default(), &read(THE_GATE));
    assert!(wrong.is_empty(), "{wrong:?}");
}
