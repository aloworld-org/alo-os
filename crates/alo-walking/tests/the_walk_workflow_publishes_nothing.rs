//! **The walk workflow builds an image and publishes nothing.**
//!
//! CLAUDE.md: *no real image is built while any branch but main exists*, and a
//! walk image is the one exception — *never pushed, published, pinned or
//! handed to anybody*. On a lane machine that is true because nothing there can
//! push. On a runner it is true only because `.github/workflows/walk.yml` says
//! nothing that would, and a workflow is the file nobody reads twice. So this
//! reads it, the way `alo-image`'s `workflow.rs` reads `image.yml`: by lines,
//! and only the lines that are not comments, because the comment explaining
//! that nothing is pushed is exactly where the word *push* belongs.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unreadable workflow is the failure being reported"
)]

/// The workflow, from this repository.
const THE_WORKFLOW: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../.github/workflows/walk.yml"
);

/// The workflow's lines that are not comments, trimmed.
fn said() -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(THE_WORKFLOW) else {
        panic!("{THE_WORKFLOW} could not be read");
    };
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// **Nothing pushes, logs in to a registry, signs or releases.** The signing
/// spellings are `alo-image`'s list, for the image's key and the installer's.
#[test]
fn nothing_is_pushed_signed_or_published() {
    let forbidden = [
        "podman push",
        "docker push",
        "buildah push",
        "skopeo copy",
        "podman login",
        "docker login",
        "registry",
        "ghcr.io",
        "gh release",
        "packages: write",
        "contents: write",
        "cosign sign",
        "COSIGN_",
        "signtool",
        "osslsigncode",
        "Set-AuthenticodeSignature",
        ".pfx",
    ];
    for line in said() {
        for word in forbidden {
            assert!(!line.contains(word), "{word:?} in: {line}");
        }
    }
}

/// **The image is main's**: the checkout it is built from is the merged
/// commit after a merge to main and `main` otherwise, and it is built from
/// that checkout.
#[test]
fn the_image_is_built_from_main() {
    let lines = said();
    let checkout = lines.windows(3).any(|three| {
        three
            .iter()
            .any(|it| it == "ref: ${{ github.event_name == 'push' && github.sha || 'main' }}")
            && three.iter().any(|it| it == "path: image-source")
    });
    assert!(checkout, "no checkout of main into image-source");
    assert!(
        lines.windows(2).any(|two| matches!(two, [step, place]
            if step.starts_with("- name: Build the walk image")
                && place == "working-directory: image-source")),
        "the image is not built from the checkout of main"
    );
}

/// **What leaves the runner is the walk's own directory, for seven days** —
/// the pictures, the report and the cleaned log, never the disk or the raw log
/// beside it.
#[test]
fn only_the_walk_is_kept_and_only_for_seven_days() {
    let lines = said();
    let uploads: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains("actions/upload-artifact"))
        .map(|(at, _)| at)
        .collect();
    assert_eq!(uploads.len(), 1, "one upload, not {}", uploads.len());
    let after: Vec<&String> = uploads
        .first()
        .map(|at| lines.iter().skip(*at).take(8).collect())
        .unwrap_or_default();
    assert!(
        after.iter().any(|it| *it == "path: the-walk"),
        "the upload is not the walk's own directory: {after:?}"
    );
    assert!(
        after.iter().any(|it| *it == "retention-days: 7"),
        "the upload is not kept for seven days: {after:?}"
    );
    assert!(
        lines.iter().any(|it| it == "--out the-walk \\")
            && lines.iter().any(|it| it == "--scratch walk"),
        "the walk does not keep its raw log apart from what is uploaded"
    );
}

/// **It runs after every merge to main, and when a person starts it** —
/// nothing else: a push to any other branch, a pull request or a schedule
/// would be a walk the owner did not decide on (2026-11-14).
#[test]
fn it_runs_after_every_merge_to_main_and_by_hand() {
    let lines = said();
    let Some(on) = lines.iter().position(|it| it == "on:") else {
        panic!("no triggers");
    };
    let Some(jobs) = lines.iter().position(|it| it == "permissions:") else {
        panic!("no permissions after the triggers");
    };
    let triggers: Vec<&String> = lines
        .iter()
        .take(jobs)
        .skip(on + 1)
        .filter(|it| it.ends_with(':') && !it.contains(' '))
        .collect();
    assert_eq!(
        triggers,
        vec![
            "push:",
            "workflow_dispatch:",
            "inputs:",
            "with_the_provider:"
        ],
        "a trigger beside a merge to main and a person"
    );
    assert!(
        lines.windows(2).any(|two| matches!(two, [push, branches]
            if push == "push:" && branches == "branches: [main]")),
        "the push trigger is not only main's"
    );
}

/// **Walks of main do not pile up**: one runs and only the newest waits, and
/// the one running is never cancelled, so every walk that starts says how it
/// ended.
#[test]
fn one_walk_runs_and_only_the_newest_waits() {
    let lines = said();
    assert!(
        lines
            .windows(3)
            .any(|three| matches!(three, [concurrency, group, cancel]
            if concurrency == "concurrency:"
                && group == "group: walk-${{ github.ref }}"
                && cancel == "cancel-in-progress: false")),
        "no concurrency group that lets a running walk finish"
    );
}

/// **A secret is reached only on a walk after a merge, or when a person asks
/// for it**: every line that names a secret also names the merge and the box
/// they tick.
#[test]
fn a_secret_is_reached_only_after_a_merge_or_when_asked_for() {
    let named: Vec<String> = said()
        .into_iter()
        .filter(|line| line.contains("secrets."))
        .collect();
    assert!(
        !named.is_empty(),
        "the provider's secret is not named at all"
    );
    for line in named {
        assert!(
            line.contains(
                "(github.event_name == 'push' || inputs.with_the_provider) && secrets.ALO_WALK_PROVIDER_KEY"
            ),
            "a secret reached without a merge or the box: {line}"
        );
    }
}
