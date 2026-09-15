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
//! So this file holds four things: the recipe names one release; the decision
//! about the key was made by the owner; only the public half of that key is in
//! the repository; and the plan does not call the publish done while the first
//! signed push is still to come. The last is the one that matters most to a
//! loop that selects from the plan: a task marked done over a missing signature
//! is a pin nobody can verify.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_image::{Image, THE_IMAGE, THE_VERSION_LABEL};

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

/// **The plan does not call the publish done while it waits**, and says what
/// it waits on — so the loop that selects from the plan neither selects it
/// again as ready nor reads it as finished.
#[test]
fn the_plan_does_not_mark_the_publish_done_while_it_waits() {
    let task = the_first_task();

    assert!(
        task.starts_with("The image is published from GitHub, signed, and pinned"),
        "{task}"
    );
    assert!(
        !task.contains("**Done"),
        "task 1 is marked done while nothing has been pushed or signed"
    );
    let Some(status) = task.lines().find(|line| line.starts_with("**Status:**")) else {
        panic!("task 1 has no status");
    };
    assert!(status.contains("blocked"), "{status}");
    assert!(
        task.contains("ADR 0036"),
        "task 1 does not name what it waits on"
    );
    assert!(
        task.contains("launches nothing and says so"),
        "task 1 does not tell the next worker what to do while it waits"
    );
}
