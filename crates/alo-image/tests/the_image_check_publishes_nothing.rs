//! **The image check builds the image on every pull request and publishes
//! nothing.**
//!
//! The owner decided on 2026-11-14 that the image is built on every pull
//! request, after #1245 broke main's image build and nobody saw it for hours.
//! That build is a check, like a walk image: CLAUDE.md allows it only because it
//! is *never pushed, published, pinned or handed to anybody*. On a runner that is
//! true only while `.github/workflows/image-check.yml` says nothing that would
//! make it otherwise, and a workflow is the file nobody reads twice — the edit
//! that turns a check into a publish is one step added to it.
//!
//! So this reads it with `alo_image::TheWorkflow`, the reader `image.yml` is held
//! by: the lines that are not comments, because the comment explaining that
//! nothing is pushed is exactly where the word belongs. Every rule is a function
//! of the text, so each one is shown to refuse a workflow that breaks it as well
//! as to accept the one that ships.

use alo_image::TheWorkflow;

/// The workflow, from this repository.
const THE_CHECK: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../.github/workflows/image-check.yml"
);

/// The events it runs on, in order: every pull request, and the merge queue's
/// commit, which is the tree that becomes main and the one a required check is
/// asked of.
const TRIGGERS: [&str; 2] = ["pull_request", "merge_group"];

/// What would make the build leave the runner as an image, or make the run
/// reach for anything it does not need.
///
/// Pushing and logging in, in every tool that could; saving or exporting the
/// image to a file, which is how an image is uploaded without a registry;
/// making a Release; a token allowed to write; any secret; and the event that
/// runs with the base repository's secrets on a stranger's branch.
const FORBIDDEN: [&str; 22] = [
    "podman push",
    "docker push",
    "buildah push",
    "skopeo copy",
    "podman login",
    "docker login",
    "buildah login",
    "skopeo login",
    "ghcr.io",
    "podman save",
    "podman image save",
    "docker save",
    "podman export",
    "oci-archive",
    "docker-archive",
    "gh release",
    "packages: write",
    "contents: write",
    "id-token: write",
    "secrets.",
    "pull_request_target",
    "continue-on-error",
];

/// The one file that may leave the runner: the build's log.
const THE_LOG: &str = "path: ${{ runner.temp }}/image-build.log";

/// What builds the recipe.
const THE_BUILD: &str = "sudo podman build -f image/Containerfile";

/// The longest a kept log may stay, in days.
const KEPT_AT_MOST: u32 = 7;

/// The shipped workflow, as text.
fn shipped() -> String {
    std::fs::read_to_string(THE_CHECK).unwrap_or_default()
}

/// Everything this workflow does that a check may not, one sentence each. An
/// empty list is a workflow that builds the image and keeps nothing of it.
fn what_is_wrong_with(text: &str) -> Vec<String> {
    let read = TheWorkflow::read(text);
    let mut wrong = Vec::new();

    if read.triggers() != TRIGGERS {
        wrong.push(format!(
            "it runs on {:?}, not on every pull request and the merge queue alone",
            read.triggers()
        ));
    }
    if read.signs() {
        wrong.push("it signs, and only the owner signs".to_owned());
    }
    for line in read.live() {
        for word in FORBIDDEN {
            if line.contains(word) {
                wrong.push(format!("{word:?} in: {line}"));
            }
        }
    }
    if !read.live().iter().any(|line| line.starts_with(THE_BUILD)) {
        wrong.push("it does not build image/Containerfile".to_owned());
    }
    if !read
        .live()
        .iter()
        .any(|line| line.starts_with(THE_BUILD) && line.ends_with("exit 1; }"))
    {
        wrong.push("a failed build does not fail the run".to_owned());
    }
    wrong.extend(what_leaves_the_runner(read.live()));
    wrong
}

/// Every upload that is not the build log, kept no longer than a week.
///
/// An upload's settings are the lines under it up to the next step, so a
/// second upload — of the image, say — is read on its own and refused on its
/// own.
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
        if paths.len() != 1 || paths.iter().any(|it| it.as_str() != THE_LOG) {
            wrong.push(format!("an upload that is not the build log: {settings:?}"));
        }
        let kept = settings
            .iter()
            .find_map(|it| it.strip_prefix("retention-days:"))
            .and_then(|days| days.trim().parse::<u32>().ok());
        if !kept.is_some_and(|days| days <= KEPT_AT_MOST) {
            wrong.push(format!(
                "an upload kept longer than {KEPT_AT_MOST} days, or for as long as GitHub keeps \
                 anything: {settings:?}"
            ));
        }
    }
    wrong
}

/// The shipped workflow with one thing changed, after checking the thing to
/// change is there — so a mutation that silently did nothing cannot read as a
/// refusal.
fn with(from: &str, to: &str) -> Option<String> {
    let text = shipped();
    text.contains(from).then(|| text.replacen(from, to, 1))
}

/// **The shipped check builds the image, fails when it does not build, and
/// publishes nothing** — so the refusals below are about the rules rather than
/// a reader that never worked.
#[test]
fn the_shipped_check_builds_the_image_and_keeps_only_its_log() {
    let text = shipped();
    assert!(!text.is_empty(), "{THE_CHECK} could not be read");
    let wrong = what_is_wrong_with(&text);
    assert!(wrong.is_empty(), "{wrong:?}");
}

/// **A push, a log-in, a saved image or a Release is refused**, each added as
/// the one step somebody might add.
#[test]
fn a_check_that_pushes_or_publishes_is_refused() {
    let after = "      - name: Throw the image away\n";
    for step in [
        "      - run: sudo podman push localhost/alo-os:check ghcr.io/aloworld-org/alo-os:pr\n",
        "      - run: echo \"$T\" | podman login ghcr.io -u me --password-stdin\n",
        "      - run: docker push example.org/alo-os:pr\n",
        "      - run: skopeo copy containers-storage:localhost/alo-os:check docker://example.org/x\n",
        "      - run: sudo podman save -o alo.tar localhost/alo-os:check\n",
        "      - run: sudo podman build -f image/Containerfile -t x . --output type=oci-archive,dest=x.tar\n",
        "      - run: gh release create pr-image alo.tar --prerelease\n",
        "      - run: cosign sign --key cosign.key localhost/alo-os:check\n",
        "      - env:\n          T: ${{ secrets.GITHUB_TOKEN }}\n        run: echo\n",
    ] {
        let changed = with(after, &format!("{step}{after}"));
        assert!(
            changed.is_some_and(|it| !what_is_wrong_with(&it).is_empty()),
            "a check that added this step before Throw the image away passed:\n{step}"
        );
    }
}

/// **A token allowed to write is refused**, which is what a push would need
/// first.
#[test]
fn a_check_whose_token_may_write_is_refused() {
    for written in [
        "permissions:\n  contents: read\n  packages: write\n",
        "permissions:\n  contents: write\n",
        "permissions:\n  contents: read\n  id-token: write\n",
    ] {
        let changed = with("permissions:\n  contents: read\n", written);
        assert!(
            changed.is_some_and(|it| !what_is_wrong_with(&it).is_empty()),
            "a token with {written:?} passed"
        );
    }
}

/// **Uploading the image is refused**, beside the log or instead of it, and so
/// is keeping the log for longer than a week.
#[test]
fn a_check_that_uploads_the_image_or_keeps_its_log_too_long_is_refused() {
    let second = "      - uses: actions/upload-artifact@v4\n        with:\n          name: the-image\n          path: alo.tar\n          retention-days: 1\n";
    let last = "          if-no-files-found: warn\n";
    for changed in [
        with(last, &format!("{last}{second}")),
        with(THE_LOG, "path: alo.tar"),
        with("retention-days: 3", "retention-days: 90"),
        with("          retention-days: 3\n", ""),
    ] {
        assert!(
            changed
                .as_deref()
                .is_some_and(|it| !what_is_wrong_with(it).is_empty()),
            "an upload beyond the build log passed: {changed:?}"
        );
    }
}

/// **A check that runs on anything but pull requests and the queue is refused**,
/// and so is one that would lose either of the two.
///
/// Without `merge_group` the queue's commit is never built and a required check
/// waits for ever; with `pull_request_target` a stranger's branch is built with
/// this repository's secrets; with `push` it is a build nobody asked for on a
/// branch that has no pull request.
#[test]
fn a_check_on_other_events_is_refused() {
    let on = "on:\n  pull_request:\n  merge_group:\n";
    for changed in [
        "on:\n  pull_request:\n",
        "on:\n  merge_group:\n",
        "on:\n  pull_request_target:\n  merge_group:\n",
        "on:\n  pull_request:\n  merge_group:\n  push:\n",
        "on:\n  pull_request:\n  merge_group:\n  workflow_dispatch:\n",
    ] {
        let text = with(on, changed);
        assert!(
            text.is_some_and(|it| !what_is_wrong_with(&it).is_empty()),
            "a check running on {changed:?} passed"
        );
    }
}

/// **A build whose failure does not fail the run is refused**: the whole point
/// is a red check on a change that breaks the image.
#[test]
fn a_build_that_cannot_fail_the_check_is_refused() {
    for changed in [
        with("exit 1; }", "true; }"),
        with(
            "    runs-on: ubuntu-24.04\n",
            "    runs-on: ubuntu-24.04\n    continue-on-error: true\n",
        ),
        with(THE_BUILD, "echo podman build -f image/Containerfile"),
    ] {
        assert!(
            changed
                .as_deref()
                .is_some_and(|it| !what_is_wrong_with(it).is_empty()),
            "a check whose build could not fail it passed: {changed:?}"
        );
    }
}

/// **The comment saying nothing is pushed is not a push.**
#[test]
fn a_comment_about_pushing_is_not_pushing() {
    let changed = with(
        "name: Image check\n",
        "# nothing here runs podman push to ghcr.io\nname: Image check\n",
    );
    assert!(changed.is_some(), "the check is not named Image check");
    let wrong = what_is_wrong_with(&changed.unwrap_or_default());
    assert!(wrong.is_empty(), "{wrong:?}");
}
