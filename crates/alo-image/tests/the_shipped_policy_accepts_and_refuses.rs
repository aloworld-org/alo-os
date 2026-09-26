//! **What the shipped policy accepts, and what it refuses** — measured through
//! the library that will read it, not through the tool that wrote the signature.
//!
//! `cosign verify` said yes to five releases in a row while **no alo OS machine
//! could read a thing**, so this asks `containers/image` instead: the same code
//! `bootc --enforce-container-sigpolicy` runs. `LOOP.md`'s rule — *measure from
//! the consumer, never from the producer* — is why this file exists at all.
//!
//! # Accepted alone is a file that might be inert
//!
//! Every clause here comes in a pair. A policy that accepts the signed release
//! and refuses nothing would pass an acceptance-only test and enforce nothing,
//! which is exactly what `--enforce-container-sigpolicy` looks like over a
//! permissive default.
//!
//! **And the tool has to be the right one.** `skopeo inspect` does **not** apply
//! the signature policy: run against a refused image it prints the config and
//! exits zero. Four of these clauses were first written with `inspect`, all four
//! passed, and all four were meaningless. Only `copy` verifies. That mistake is
//! recorded here because the next person will reach for `inspect` too — it is
//! the obvious tool and it silently proves nothing.
//!
//! # The negative case worth keeping forever
//!
//! `containers/image` fetches a sigstore signature from a registry **only** when
//! that registry is configured with `use-sigstore-attachments`. Without it, a
//! correctly signed image is refused with *A signature was required, but no
//! signature exists* — **the same sentence as a genuinely unsigned image.** So a
//! machine shipping `policy.json` without `registries.d` refuses every update
//! while telling the person the release is unsigned, and whoever debugs it checks
//! the signature, the key and the registry, all of which are fine.
//!
//! That is held below as its own clause, because it is invisible in every other
//! direction.
//!
//! # A machine with no network, or no skopeo, says so
//!
//! ADR 0063: a machine that cannot run the engine says so rather than failing.
//! These clauses reach the real registry, so a build host without either skips
//! and prints what it skipped — a skip nobody can see is the same colour as a
//! pass.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// This repository.
fn here() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("this crate is two directories below the repository")
        .to_path_buf()
}

/// The digest `image/pinned.toml` pins, which is the one a machine will pull.
fn the_pinned_digest() -> String {
    let pinned = std::fs::read_to_string(here().join("image/pinned.toml"))
        .expect("image/pinned.toml is readable");
    pinned
        .lines()
        .filter_map(|line| line.strip_prefix("digest = "))
        .next_back()
        .map(|digest| digest.trim().trim_matches('"').to_owned())
        .expect("image/pinned.toml pins a digest")
}

/// Where the image is published.
fn the_registry() -> String {
    let pinned = std::fs::read_to_string(here().join("image/pinned.toml"))
        .expect("image/pinned.toml is readable");
    pinned
        .lines()
        .filter_map(|line| line.strip_prefix("registry = "))
        .next()
        .map(|it| it.trim().trim_matches('"').to_owned())
        .expect("image/pinned.toml names a registry")
}

/// The shipped policy, with the key path pointed at this checkout.
///
/// The file names `/etc/containers/alo-os.pub`, which is where it lands **in the
/// image**. Nothing else about it is changed: the default, the scope and the
/// signature requirement are the shipped ones, because those are what is being
/// measured.
fn the_policy_here(at: &Path) -> PathBuf {
    let shipped = std::fs::read_to_string(here().join("image/policy.json"))
        .expect("image/policy.json is readable");
    let key = here().join("image/signing/alo-os.pub");
    let pointed = shipped.replace(
        "/etc/containers/alo-os.pub",
        key.to_str().expect("a path that is text"),
    );
    let written = at.join("policy.json");
    std::fs::write(&written, pointed).expect("a policy to measure with");
    written
}

/// Whether a `skopeo` that takes the flags this needs is on the path.
fn skopeo_is_here() -> bool {
    Command::new("skopeo")
        .arg("--version")
        .output()
        .is_ok_and(|it| it.status.success())
}

/// How long a copy is given before it is killed.
///
/// The policy is applied **before the first blob**, so acceptance is visible in
/// seconds and nothing here needs the image. The published release carries
/// several gigabytes of model weights, and a check that fetched them on every
/// gate run would cost every machine in the fleet that bandwidth to learn
/// something it knew twenty seconds in.
const SOON: &str = "20";

/// What `skopeo copy` said, with the policy and optionally the configuration.
///
/// **`copy`, never `inspect`.** Only `copy` applies the policy; `inspect` prints
/// the config of an image the policy would refuse and exits zero.
///
/// The copy is killed as soon as it starts fetching blobs: the published image
/// carries several gigabytes of model weights, and the policy is applied
/// **before** the first blob. So *Copying blob* is the acceptance and a fatal
/// rejection is the refusal, and neither needs the bytes.
fn copied(policy: &Path, configuration: Option<&Path>, image: &str) -> String {
    let out = std::env::temp_dir().join(format!("alo-policy-{}", std::process::id()));
    drop(std::fs::remove_dir_all(&out));
    let mut skopeo = Command::new("timeout");
    skopeo.arg(SOON).arg("skopeo");
    if let Some(configuration) = configuration {
        skopeo.arg("--registries.d").arg(configuration);
    }
    skopeo
        .arg("--policy")
        .arg(policy)
        .arg("copy")
        .arg(format!("docker://{image}"))
        .arg(format!("dir:{}", out.display()));
    let said = skopeo.output().expect("skopeo runs");
    drop(std::fs::remove_dir_all(&out));
    format!(
        "{}{}",
        String::from_utf8_lossy(&said.stdout),
        String::from_utf8_lossy(&said.stderr)
    )
}

/// **The shipped policy accepts the signed release and refuses everything else.**
///
/// Four clauses, and each is worthless without the others.
#[test]
fn the_shipped_policy_accepts_the_signed_release_and_refuses_the_rest() {
    if !skopeo_is_here() {
        println!(
            "skipped: this machine has no `skopeo`, so the policy cannot be measured \
             through the library that reads it"
        );
        return;
    }
    let held = tempfile::tempdir().expect("somewhere to write a policy");
    let policy = the_policy_here(held.path());
    let configuration = here().join("image/registries.d");
    let registry = the_registry();
    let signed = format!("{registry}@{}", the_pinned_digest());

    // 1. The pinned, signed release is accepted — it gets past the policy and
    //    starts fetching. Anything the network does after that is not the
    //    policy's business.
    let accepted = copied(&policy, Some(&configuration), &signed);
    if accepted.contains("no such host") || accepted.contains("dial tcp") {
        println!("skipped: this machine cannot reach {registry} — {accepted}");
        return;
    }
    assert!(
        accepted.contains("Copying blob") || accepted.contains("Getting image source signatures"),
        "the shipped policy refused the release it is written for:\n{accepted}"
    );

    // 2. **The negative case that is invisible in every other direction.** The
    //    same signed image, the same policy, without the registry configuration:
    //    refused, and refused with the sentence an *unsigned* image gets. This is
    //    why the configuration ships beside the policy rather than after it.
    let unconfigured = copied(&policy, None, &signed);
    assert!(
        unconfigured.contains("A signature was required, but no signature exists"),
        "without `registries.d` a signed release should be refused for want of \
         configuration, and was not:\n{unconfigured}"
    );

    // 3. The default is not permissive. An image outside the scope is refused by
    //    the policy itself, and says so in different words — which is how a
    //    reader tells a missing signature from a rejected image.
    let outside = copied(
        &policy,
        Some(&configuration),
        "docker.io/library/registry:2",
    );
    assert!(
        outside.contains("rejected by policy"),
        "the shipped policy's default accepted an image it never heard of, which \
         makes --enforce-container-sigpolicy a no-op:\n{outside}"
    );
}

/// **The policy's default rejects, read from the file rather than from a run.**
///
/// Parsed rather than grepped, and asserted about the shipped file itself: a
/// default of `insecureAcceptAnything` would make every clause above pass for the
/// wrong reason, because everything would be accepted.
#[test]
fn the_shipped_policy_rejects_by_default() {
    let shipped: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(here().join("image/policy.json"))
            .expect("image/policy.json is readable"),
    )
    .expect("image/policy.json is JSON");

    let default = shipped
        .get("default")
        .and_then(|it| it.as_array())
        .expect("a policy has a default");
    assert_eq!(
        default.len(),
        1,
        "the default is one requirement: {default:?}"
    );
    let only = default.first().expect("just counted one");
    assert_eq!(
        only.get("type").and_then(|it| it.as_str()),
        Some("reject"),
        "the shipped policy's default is not `reject`, so \
         --enforce-container-sigpolicy enforces nothing: {default:?}"
    );
}

/// **The registry is configured for sigstore attachments, or nothing is ever
/// fetched.**
///
/// Held separately from the run above so that a machine with no network still
/// catches the file going missing — which is the failure that looks like an
/// unsigned release.
#[test]
fn the_registry_is_configured_to_fetch_signatures() {
    let configured = std::fs::read_to_string(here().join("image/registries.d/alo-os.yaml"))
        .expect("image/registries.d/alo-os.yaml is readable");
    let registry = the_registry();
    assert!(
        configured.contains(&registry),
        "the configuration does not name {registry}, so no signature is fetched \
         for it:\n{configured}"
    );
    assert!(
        configured.contains("use-sigstore-attachments: true"),
        "without `use-sigstore-attachments` a signed release is refused with the \
         same words as an unsigned one:\n{configured}"
    );
}

/// **The image carries all three, and at the paths the policy names.**
#[test]
fn the_recipe_ships_the_policy_the_configuration_and_the_key() {
    let recipe = std::fs::read_to_string(here().join("image/Containerfile"))
        .expect("image/Containerfile is readable");
    for shipped in [
        "image/policy.json /etc/containers/policy.json",
        "image/registries.d/ /etc/containers/registries.d/",
        "image/signing/alo-os.pub /etc/containers/alo-os.pub",
    ] {
        assert!(
            recipe.contains(shipped),
            "the recipe does not ship `{shipped}`, so the machine enforces the \
             base's default, which accepts anything"
        );
    }
}
