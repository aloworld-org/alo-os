//! How the image leaves this repository for the registry an installer pulls
//! from, held to what has actually been decided about it.
//!
//! ADR 0033 §3 publishes the image to `ghcr.io/aloworld-org/alo-os` and pins
//! the digest the installer pulls. Two things have to be true before that pin
//! can be written, and neither is a worker's to invent: the recipe names the
//! release it builds, which is a line in `image/Containerfile`, and somebody
//! holds the key the image is signed with, which is ADR 0036 — accepted by the
//! owner on 2026-09-15.
//!
//! So this file holds what the plan's first task accepts: the recipe names one
//! release; the decision about the key was made by the owner; only the public
//! half of that key is in the repository; the digest the owner signed is pinned
//! in `image/pinned.toml` and the image agrees with it; the workflow pushes a
//! candidate, never signs, and says why it is not yet the road; and the plan
//! marks the task done with the digest it pinned.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_image::{
    Image, THE_IMAGE, THE_REGISTRY, THE_VERSION_LABEL, THE_WORKFLOW, TheWorkflow,
    everything_wrong_with,
};

/// The decision about who holds the signing key.
const THE_DECISION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/decisions/0036-the-image-is-signed-by-a-key-a-person-holds.md"
);

/// The plan whose first task publishes the image.
const THE_PLAN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/autonomy/v0-5-the-installer-plan.md"
);

/// The recipe, read.
const THE_RECIPE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../image/Containerfile");

/// A file this test reads, as text.
fn text(at: &str) -> String {
    std::fs::read_to_string(at).unwrap_or_else(|why| panic!("{at} did not read: {why}"))
}

/// The plan's first task, from its heading to the next one.
fn the_first_task() -> String {
    let plan = text(THE_PLAN);
    let Some((_, from)) = plan.split_once("### 1. ") else {
        panic!("the plan has no task 1");
    };
    let Some((task, _)) = from.split_once("### 2. ") else {
        panic!("the plan has no task 2 after task 1");
    };
    task.to_owned()
}

/// **The recipe names the one release a published digest is pinned against**,
/// in the OCI specification's own label, as three numbers — so the image that
/// is pushed can say which release it is.
#[test]
fn the_recipe_names_the_one_release_a_pin_is_held_against() {
    let image = match Image::at(Path::new(THE_IMAGE)) {
        Ok(image) => image,
        Err(why) => panic!("the image this repository ships did not read: {why}"),
    };

    assert!(
        image.version().names_one_release(),
        "the recipe names `{}` as its release",
        image.version().everything_stated()
    );
    assert!(
        text(THE_RECIPE).contains(&format!("LABEL {THE_VERSION_LABEL}=")),
        "the release is not stated as the OCI label a registry reads"
    );
}

/// **Who holds the signing key is a recorded decision, and the owner made it.**
///
/// It names the registry, the tool and all three roads; it recommends the key a
/// person holds; it says no agent holds or publishes with it; and its status
/// says it was accepted **by the owner**, option A — the root of trust for
/// every installed machine is not a worker's to accept.
#[test]
fn the_decision_on_who_holds_the_signing_key_was_made_by_the_owner() {
    let decision = text(THE_DECISION);

    let Some(status) = decision
        .lines()
        .find(|line| line.starts_with("**Status:**"))
    else {
        panic!("ADR 0036 records no status");
    };
    assert!(status.contains("accepted by the owner"), "{status}");
    assert!(status.contains("**A**"), "{status}");

    for named in [
        "ghcr.io/aloworld-org/alo-os",
        "cosign",
        "### A. A key pair a person generates and holds",
        "### B. The private key is a GitHub Actions secret",
        "### C. Keyless signing through Sigstore",
        "image/signing/alo-os.pub",
        "org.opencontainers.image.version",
    ] {
        assert!(decision.contains(named), "ADR 0036 does not name `{named}`");
    }

    let Some((_, recommended)) = decision.split_once("## Recommendation") else {
        panic!("ADR 0036 makes no recommendation");
    };
    // A sentence in Markdown wraps wherever its line ran out, and a phrase is
    // the same phrase across a line break.
    let recommended = recommended.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        recommended.trim_start().starts_with("**A.**"),
        "{recommended}"
    );
    assert!(
        recommended.contains("no agent and no build loop ever holds the private key or publishes"),
        "the recommendation does not keep the key and the publish away from agents"
    );
    assert!(
        recommended.contains("by digest"),
        "the recommendation does not sign by digest"
    );
}

/// The public half of the owner's key, where ADR 0036 says it is.
const THE_PUBLIC_KEY: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../image/signing/alo-os.pub"
);

/// **Only the public half of the key is in the repository, and it is one.**
///
/// A PEM public key and nothing else — so a puller has something to verify
/// against — and, just as much, no private key anywhere under `image/signing/`:
/// a private half committed here would hand the root of trust for every
/// installed machine to anybody who can read the repository.
#[test]
fn the_public_half_is_committed_and_the_private_half_is_not() {
    let key = text(THE_PUBLIC_KEY);
    let key = key.trim();
    assert!(key.starts_with("-----BEGIN PUBLIC KEY-----"), "{key}");
    assert!(key.ends_with("-----END PUBLIC KEY-----"), "{key}");
    assert_eq!(
        key.matches("-----BEGIN").count(),
        1,
        "the public key file holds more than one block"
    );

    let Some(signing) = Path::new(THE_PUBLIC_KEY).parent() else {
        panic!("the public key has no folder");
    };
    let entries = std::fs::read_dir(signing)
        .unwrap_or_else(|why| panic!("{} did not list: {why}", signing.display()));
    for entry in entries.flatten() {
        let named = entry.file_name().to_string_lossy().into_owned();
        assert!(
            !named.ends_with(".key"),
            "`{named}` is in image/signing/ — a private key must never be in the repository"
        );
        let written = std::fs::read_to_string(entry.path()).unwrap_or_default();
        assert!(
            !written.contains("PRIVATE KEY"),
            "`{named}` holds a private key — it must never be in the repository"
        );
    }
}

/// The digest the owner signed on 2026-09-21, for release `0.0.5`.
///
/// **The one place the digest is spelled rather than read, and that is
/// deliberate.** Everywhere else it was spelled it was a *copy*, one of seven
/// that had to be edited in step; those read the pin now. This one is not a
/// copy, it is the checkpoint. A test that read the pin and then asserted the
/// pin would pass however the pin changed, and the point is that the digest an
/// installer pulls cannot move without a person writing it here too.
///
/// So when this fails it is asking a question rather than reporting a fault:
/// *was this pin changed by somebody who verified the signature?* Answer it by
/// verifying, then edit this line. The verification cannot live here — it needs
/// the registry, and nothing in this crate reaches the network.
const THE_SIGNED_DIGEST: &str =
    "sha256:6c9abbc5a6a0f5299991f4cca65152452b3cbae339b161059528d72f2aad3ba1";

/// **The digest an installer pulls is the one the owner signed, pinned in one
/// file the crate reads, and the image agrees with it.**
///
/// The pin names the registry ADR 0033 decided, the release the recipe names,
/// the signed digest, and the committed public key; and nothing the crate checks
/// — the recipe, the key, `docs/booting.md` — disagrees with it.
#[test]
fn the_signed_digest_is_pinned_and_the_image_agrees_with_it() {
    let image = match Image::at(Path::new(THE_IMAGE)) {
        Ok(image) => image,
        Err(why) => panic!("the image this repository ships did not read: {why}"),
    };
    let pin = image.pin();

    assert_eq!(pin.registry(), THE_REGISTRY);
    assert_eq!(pin.registry(), "ghcr.io/aloworld-org/alo-os");
    assert_eq!(pin.digest(), THE_SIGNED_DIGEST);
    // The recipe names the release being built: the declared candidate while
    // one is in flight, and the pinned release otherwise. Between releases
    // those are the same sentence; during one they are not, and holding the
    // recipe to the pinned release would make it impossible to build the next
    // one from a published commit — which is ADR 0036's first step, and the
    // whole reason `next` exists.
    assert_eq!(
        image.version().said(),
        Some(pin.next().unwrap_or(pin.version())),
        "the recipe builds neither what is pinned nor what is declared next"
    );
    if let Some(next) = pin.next() {
        assert_ne!(
            next,
            pin.version(),
            "a candidate is declared that is the release already pinned"
        );
    }
    assert_eq!(
        Path::new(THE_IMAGE).join(pin.key()),
        Path::new(THE_PUBLIC_KEY),
        "the pin names a key other than the one ADR 0036 committed"
    );

    let wrong = everything_wrong_with(&image);
    assert!(wrong.is_empty(), "{wrong:?}");
}

/// **The workflow is written, pushes a candidate, and never signs**, with the
/// reason it is not yet the road recorded in it (the plan's acceptance; ADR 0036
/// as the owner accepted it).
#[test]
fn the_workflow_pushes_a_candidate_never_signs_and_says_why_it_waits() {
    let workflow = TheWorkflow::read(&text(THE_WORKFLOW));

    assert!(
        !workflow.signs(),
        "the workflow signs, and only the owner signs"
    );
    assert!(
        workflow.runs_only_when_asked(),
        "the workflow runs on {:?}, and it runs only when a person asks",
        workflow.triggers()
    );
    assert!(workflow.builds_only_from_main());
    assert!(workflow.pushes_to(THE_REGISTRY));
    assert!(
        workflow.looks_before_it_pushes(),
        "the workflow could push over a published release"
    );
    assert!(workflow.says_why_it_is_not_yet_the_road());
}

/// **The plan marks the publish done, and still carries what was signed.**
///
/// A task finished and not marked is a task the loop selects again; one marked
/// without the digest it pinned is one nobody can check.
#[test]
fn the_plan_marks_the_publish_done_with_the_digest_it_pinned() {
    let task = the_first_task();

    assert!(
        task.starts_with("The image is published from GitHub, signed, and pinned"),
        "{task}"
    );
    let Some(status) = task.lines().find(|line| line.starts_with("**Status:**")) else {
        panic!("task 1 has no status");
    };
    assert!(status.contains("**Done, 2026-09-15**"), "{status}");
    assert!(
        task.contains(THE_SIGNED_DIGEST),
        "task 1 does not carry the digest the owner signed"
    );
    assert!(
        task.contains("image/pinned.toml"),
        "task 1 does not name the file the digest is pinned in"
    );
}
