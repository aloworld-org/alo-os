//! **The install check is started by hand, runs the virtual-machine test by
//! name on the free runner, and publishes nothing but logs.**
//!
//! `.github/workflows/install-check.yml` installs alo OS in virtual machines
//! from the pinned release. That is safe only because the workflow says nothing
//! that would push, sign or upload an image, and a workflow is the file nobody
//! reads twice. So this reads it, by its lines that are not comments — the
//! comment explaining that nothing is pushed is where the word belongs — and
//! each rule is shown to catch the workflow that breaks it.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unreadable workflow is the failure being reported"
)]

/// The workflow, from this repository.
const THE_WORKFLOW: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../.github/workflows/install-check.yml"
);

/// The workflow as it is.
fn the_workflow() -> String {
    std::fs::read_to_string(THE_WORKFLOW)
        .unwrap_or_else(|why| panic!("{THE_WORKFLOW} could not be read: {why}"))
}

/// A workflow's lines that are not comments, trimmed.
fn said(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// What in these lines would push, log in, sign, release or write.
fn what_publishes(lines: &[String]) -> Vec<String> {
    let forbidden = [
        "podman push",
        "docker push",
        "buildah push",
        "skopeo copy",
        "podman login",
        "docker login",
        "gh release",
        "packages: write",
        "contents: write",
        "cosign sign",
        "COSIGN_",
        "secrets.",
    ];
    lines
        .iter()
        .flat_map(|line| {
            forbidden
                .iter()
                .filter(|word| line.contains(*word))
                .map(move |word| format!("{word:?} in: {line}"))
        })
        .collect()
}

/// Whether the workflow starts only when a person starts it: its `on:` holds
/// `workflow_dispatch:` and nothing else.
fn started_by_hand_only(lines: &[String]) -> bool {
    let Some(at) = lines.iter().position(|line| line == "on:") else {
        return false;
    };
    let triggers: Vec<&String> = lines
        .iter()
        .skip(at + 1)
        .take_while(|line| !line.starts_with("permissions:") && !line.starts_with("jobs:"))
        .collect();
    triggers == ["workflow_dispatch:"]
}

/// Whether it runs on the free hosted runner, and runs the virtual-machine test
/// by name with its ignored tests included, one machine at a time.
fn runs_the_test_by_name(lines: &[String]) -> bool {
    lines.iter().any(|line| line == "runs-on: ubuntu-24.04")
        && !lines
            .iter()
            .any(|line| line.starts_with("runs-on:") && line != "runs-on: ubuntu-24.04")
        && lines.iter().any(|line| {
            line.contains(
                "cargo test --locked -p alo-installing --test installed_in_a_virtual_machine",
            ) && !line.contains("--no-run")
        })
        && lines
            .iter()
            .any(|line| line.contains("-- --include-ignored --test-threads 1"))
}

/// Whether what leaves the runner is the logs, and only the logs: one upload,
/// of `*.log` files.
fn keeps_only_logs(lines: &[String]) -> bool {
    let uploads: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains("actions/upload-artifact"))
        .map(|(at, _)| at)
        .collect();
    let [upload] = uploads.as_slice() else {
        return false;
    };
    let paths: Vec<&String> = lines
        .iter()
        .skip(*upload)
        .take(8)
        .filter(|line| line.starts_with("path:"))
        .collect();
    paths == ["path: install-check/*.log"]
}

/// **Nothing pushes, logs in to a registry, signs, releases or reads a
/// secret.**
#[test]
fn nothing_is_pushed_signed_or_published() {
    assert_eq!(what_publishes(&said(&the_workflow())), Vec::<String>::new());
}

/// **It is started by hand, and by nothing else.**
#[test]
fn it_is_started_by_hand_only() {
    assert!(started_by_hand_only(&said(&the_workflow())));
}

/// **It runs the virtual-machine test by name, on the free runner.**
#[test]
fn it_runs_the_virtual_machine_test_by_name() {
    assert!(runs_the_test_by_name(&said(&the_workflow())));
}

/// **What leaves the runner is the logs**, never a disk.
#[test]
fn only_logs_leave_the_runner() {
    assert!(keeps_only_logs(&said(&the_workflow())));
}

/// **Each rule catches the workflow that breaks it**: a push, a trigger on
/// every pull request, another runner, the test not run, and a disk uploaded.
#[test]
fn a_workflow_breaking_each_rule_is_caught() {
    let workflow = the_workflow();
    let changed = |from: &str, to: &str| {
        assert!(
            workflow.contains(from),
            "the workflow no longer says {from}"
        );
        said(&workflow.replacen(from, to, 1))
    };
    assert!(
        !what_publishes(&changed(
            "df -h /\n",
            "df -h /\n          sudo podman push localhost/alo-os\n"
        ))
        .is_empty()
    );
    assert!(!started_by_hand_only(&changed(
        "  workflow_dispatch:\n",
        "  workflow_dispatch:\n  pull_request:\n"
    )));
    assert!(!runs_the_test_by_name(&changed(
        "runs-on: ubuntu-24.04",
        "runs-on: alo-16-core"
    )));
    assert!(!runs_the_test_by_name(&changed(
        "-- --include-ignored --test-threads 1",
        "-- --test-threads 1"
    )));
    assert!(!keeps_only_logs(&changed(
        "path: install-check/*.log",
        "path: target/tmp/alo-installing-vm"
    )));
}
