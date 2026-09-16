//! How the installer leaves this repository for the page a person downloads it
//! from, held to what has actually been decided about it.
//!
//! The installer plan's task 5: *a workflow builds `alo-installer` for Windows
//! on a tag, signs the executable, and publishes it as a Release asset with a
//! checksum beside it; the Release notes say which image digest it installs and
//! which Secure Boot state it accepts; the README gains a `Try it` section …
//! and nothing else; and a test in the repository refuses a Release whose notes
//! name a digest the pinned file does not.*
//!
//! One half of the first line is not a machine's, and that is a decision rather
//! than an omission:
//! [ADR 0046](../../../docs/decisions/0046-the-installer-is-signed-by-a-certificate-a-person-holds.md)
//! says a machine builds, a person signs, and the Release is a draft until they
//! have. So this file holds the workflow to building, checksumming and drafting
//! — and to never signing, and never reaching for a certificate — the way
//! `how_the_image_is_published.rs` holds the image's workflow to ADR 0036.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_image::{
    ACCEPTED, Image, THE_HARDWARE, THE_IMAGE, THE_NOTES, THE_README, THE_RELEASE, THE_SECURE_BOOT,
    TheNotes, TheRelease, Wrong, everything_wrong_with, everything_wrong_with_the_try_it,
};

/// The decision about what signs the installer, and who may.
const THE_DECISION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/decisions/0046-the-installer-is-signed-by-a-certificate-a-person-holds.md"
);

/// The plan whose fifth task releases the installer.
const THE_PLAN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/autonomy/v0-5-the-installer-plan.md"
);

/// A file this test reads, as text.
fn text(at: &str) -> String {
    std::fs::read_to_string(at).unwrap_or_else(|why| panic!("{at} did not read: {why}"))
}

/// The notes this repository ships, as text.
fn the_notes() -> String {
    text(&format!("{THE_IMAGE}/{THE_NOTES}"))
}

/// The image this repository ships, read.
fn the_image() -> Image {
    match Image::at(Path::new(THE_IMAGE)) {
        Ok(image) => image,
        Err(why) => panic!("the image this repository ships did not read: {why}"),
    }
}

/// The plan's fifth task, from its heading to the next one.
fn the_fifth_task() -> String {
    let plan = text(THE_PLAN);
    let Some((_, from)) = plan.split_once("### 5. ") else {
        panic!("the plan has no task 5");
    };
    let Some((task, _)) = from.split_once("### 6. ") else {
        panic!("the plan has no task 6 after task 5");
    };
    task.to_owned()
}

/// **The workflow builds on a tag, checksums, drafts — and never signs.**
///
/// Every one of these is the plan's acceptance as ADR 0046 settled it, and each
/// is one step away from being undone in the file in this repository nobody
/// reads twice.
#[test]
fn the_workflow_builds_checksums_and_drafts_and_never_signs() {
    let release = TheRelease::read(&text(THE_RELEASE));

    assert!(
        !release.signs(),
        "the release workflow signs, and only a person signs (ADR 0046)"
    );
    assert!(
        !release.reaches_for_a_secret(),
        "the release workflow reaches for a certificate, a key or a password"
    );
    assert!(
        release.runs_only_on_a_tag(),
        "the release workflow makes a Release without a person pushing a tag"
    );
    assert!(
        release.publishes_a_draft(),
        "the release workflow puts an unsigned installer on the download page"
    );
    assert!(
        release.takes_its_notes_from_the_committed_file(),
        "the release workflow types its own notes rather than publishing the committed ones"
    );
    assert!(
        release.writes_the_checksum_before_it_publishes(),
        "the release workflow publishes without the checksum of every asset beside it"
    );
    assert!(
        release.holds_the_build_to_the_environment(),
        "the release workflow builds an installer that would refuse every download it was given"
    );
    assert!(
        release.refuses_a_tag_the_pin_does_not_name(),
        "the release workflow builds before it holds the tag to the pinned release"
    );
    assert!(
        release.says_why_it_does_not_sign(),
        "the release workflow no longer says why signing is a person's step"
    );
}

/// **What signs the installer is a recorded decision**, with the three roads and
/// a recommendation, and it keeps the certificate away from the workflow.
#[test]
fn what_signs_the_installer_is_written_down_with_its_alternatives() {
    let decision = text(THE_DECISION);

    let Some(status) = decision
        .lines()
        .find(|line| line.starts_with("**Status:**"))
    else {
        panic!("ADR 0046 records no status");
    };
    assert!(status.contains("proposed"), "{status}");

    for named in [
        "### A. A certificate the owner holds; a machine builds, a person signs",
        "### B. The certificate is a repository secret; the workflow signs",
        "### C. A build attestation instead of a signature",
        "Authenticode",
        "image/pinned.toml",
        "ADR 0036",
    ] {
        assert!(decision.contains(named), "ADR 0046 does not name `{named}`");
    }

    let Some((_, recommended)) = decision.split_once("## Recommendation") else {
        panic!("ADR 0046 makes no recommendation");
    };
    // A sentence in Markdown wraps wherever its line ran out, and a phrase is
    // the same phrase across a line break.
    let recommended = recommended.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        recommended.trim_start().starts_with("**A**"),
        "{recommended}"
    );
    assert!(
        recommended.contains("a machine builds, a person signs"),
        "the recommendation does not keep signing in a person's hands"
    );
}

/// **The notes a person reads say which release they are getting**, and every
/// fact in them is the pinned one.
#[test]
fn the_notes_say_the_pinned_digest_and_the_state_the_installer_accepts() {
    let image = the_image();
    let notes = TheNotes::read(&the_notes());

    assert_eq!(notes.digests(), [image.pin().digest().to_owned()]);
    assert_eq!(notes.says(THE_SECURE_BOOT), Some(ACCEPTED));

    let wrong = everything_wrong_with(&image);
    assert!(wrong.is_empty(), "{wrong:?}");
}

/// **A digest the pin does not name is seen wherever it is written** — in the
/// stated fact, and in a sentence of prose beside it.
///
/// The refusal itself is `released.rs`'s own test, against a copy of this image
/// with the notes broken (`notes_naming_another_digest_are_refused`); what this
/// adds is that it is the *shipped* notes being rewritten, so a digest moved
/// into prose one day is a digest this reader still collects.
#[test]
fn a_digest_the_pin_does_not_name_is_seen_wherever_it_is_written() {
    let image = the_image();
    let other = format!("sha256:{}", "b".repeat(64));
    let notes = the_notes();
    assert!(
        notes.contains(image.pin().digest()),
        "the shipped notes do not name the pinned digest at all"
    );

    for rewritten in [
        notes.replace(image.pin().digest(), &other),
        format!("{notes}\n\nThe release before it was {other}.\n"),
    ] {
        let read = TheNotes::read(&rewritten);
        assert!(
            read.digests().contains(&other),
            "the rewritten notes name no second digest"
        );
    }
}

/// **The README offers the installer and says nothing `docs/hardware.md` does
/// not** — and its one Secure Boot sentence never asks anybody to change the
/// setting (ADR 0033 §4).
#[test]
fn the_readme_offers_the_installer_and_says_nothing_else() {
    let hardware = text(THE_HARDWARE);
    let wrong = everything_wrong_with_the_try_it(&text(THE_README), &hardware);

    assert!(wrong.is_empty(), "{wrong:?}");

    let helpful = text(THE_README).replace(
        "installer stops, says why, and changes nothing.",
        "installer stops; you can turn it off in the firmware menu.",
    );
    let wrong = everything_wrong_with_the_try_it(&helpful, &hardware);
    assert!(
        wrong
            .iter()
            .any(|it| matches!(it, Wrong::TheReadmeArguesWithSecureBoot { .. })),
        "a README telling somebody to switch Secure Boot off was not refused: {wrong:?}"
    );
}

/// **The plan marks the release done**, with the decision a person still holds:
/// a task finished and not marked is a task the loop selects again.
#[test]
fn the_plan_marks_the_release_done() {
    let task = the_fifth_task();

    assert!(
        task.starts_with("Released from GitHub, and the page a person downloads from"),
        "{task}"
    );
    let Some(status) = task.lines().find(|line| line.starts_with("**Status:**")) else {
        panic!("task 5 has no status");
    };
    assert!(status.contains("**Done, 2026-09-16**"), "{status}");
    assert!(
        task.contains("0046"),
        "task 5 does not name the decision that says who signs"
    );
}
