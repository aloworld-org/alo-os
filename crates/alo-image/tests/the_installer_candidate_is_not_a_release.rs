//! **A candidate is a thing to try, never a thing to ship**, and the workflow
//! that builds one must stay unable to become a release.
//!
//! # What it is for
//!
//! `release.yml` runs on a tag and only on a tag, which is right: a tag is the
//! owner's word that a version is a version. But between tags there was no way
//! to put a built installer in front of a person, and on 2026-10-05 that gap
//! cost an evening. `crt-static` landed on `main` as the fix for an installer
//! that **did not start at all** on a clean Windows — `0xC0000135` in the
//! loader, before `main`, printing nothing — and the only machine in this
//! project that could confirm the fix had nothing to run.
//!
//! **A fix nobody can try is a fix nobody has tried.**
//!
//! # Why this file exists rather than just the workflow
//!
//! A candidate road is one short step from a release road: add a sign, add a
//! `gh release create`, and the thing a person downloads from the website comes
//! out of a machine instead of out of the owner's hands. ADR 0046 reserves
//! signing the installer to a person holding a certificate and ADR 0036
//! reserves the image's signature the same way — and **neither ADR can stop a
//! YAML edit on its own.**
//!
//! Nothing in this repository enumerates its workflows, so a new one is
//! unguarded by default. This is that guard.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_image::TheWorkflow;

/// The workflow this file is about.
const THE_CANDIDATE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../.github/workflows/installer-candidate.yml"
);

/// What makes a Release, which this may never do.
const PUBLISH: &str = "gh release create";

/// What reads the built program before anybody is handed it.
const THE_READING: &str = "the_installer_starts_on_a_clean_windows";

/// What puts the program into the file somebody carries to a machine.
const THE_ZIP: &str = "Compress-Archive";

/// Its text, read the way every workflow in this repository is read.
///
/// **`TheWorkflow::read` takes the workflow's text and not its path**, and a
/// path handed to it parses as a one-line workflow with no triggers and no
/// steps — which is to say every rule here passes or fails for reasons that
/// have nothing to do with the file. The first draft of this test did exactly
/// that, and two of its four rules failed while the workflow was correct.
fn the_candidate() -> TheWorkflow {
    let text = std::fs::read_to_string(THE_CANDIDATE)
        .unwrap_or_else(|why| panic!("{THE_CANDIDATE} could not be read: {why}"));
    TheWorkflow::read(&text)
}

/// **It runs only when somebody asks it to.**
///
/// The same rule `image.yml` has, for the same reason: a candidate nobody asked
/// for is a red run nobody reads, and a build that happens on every push is a
/// build somebody will eventually mistake for a release because it was there.
#[test]
fn it_runs_only_when_somebody_asks() {
    assert!(
        the_candidate().runs_only_when_asked(),
        "the installer-candidate workflow runs on something other than a person asking for it"
    );
}

/// **It never signs.**
///
/// ADR 0046: the certificate is a person's and does not reach a workflow. This
/// is the cheaper half of that promise — the expensive half is that the
/// certificate is not in the repository at all — and it is the half that an
/// edit could undo without anybody noticing.
#[test]
fn it_never_signs() {
    assert!(
        !the_candidate().signs(),
        "the installer-candidate workflow signs something, and signing is a person's step \
         (ADR 0046); a candidate is unsigned by design and Windows saying so is correct"
    );
}

/// **It never makes a Release.**
///
/// The distinction this whole file protects: an artefact somebody fetches from
/// a workflow run is a thing to try, and an artefact on the Releases page is a
/// thing a stranger downloads believing it was meant for them.
#[test]
fn it_never_publishes() {
    let candidate = the_candidate();
    let publishes = candidate.live().iter().any(|line| line.contains(PUBLISH));

    assert!(
        !publishes,
        "the installer-candidate workflow makes a Release, which is `release.yml`'s to do on a \
         tag the owner pushed -- a candidate that lands on the Releases page is a release \
         nobody decided to make"
    );
}

/// **It reads the program before it hands it over**, and reads it before the
/// archive is made.
///
/// The failure this candidate road exists to let somebody confirm is an
/// executable that does not start. Handing over one that would die in the
/// loader is worse than handing over nothing: a person carries it to a machine
/// and learns nothing, which is exactly the evening this was written after.
///
/// Before the zip rather than merely somewhere, for `releasing.rs`'s reason: a
/// reading that happens after the archive is made is a reading of something
/// already on its way out.
#[test]
fn it_reads_the_program_before_it_hands_it_over() {
    let candidate = the_candidate();
    let read = candidate
        .live()
        .iter()
        .position(|line| line.contains(THE_READING));
    let zipped = candidate
        .live()
        .iter()
        .position(|line| line.contains(THE_ZIP));

    match (read, zipped) {
        (Some(read), Some(zipped)) => assert!(
            read < zipped,
            "the installer-candidate workflow archives the program before it reads what the \
             program asks Windows to load"
        ),
        (None, _) => panic!(
            "the installer-candidate workflow never runs {THE_READING}, so it can hand somebody \
             an executable that dies in the Windows loader before `main`"
        ),
        (Some(_), None) => panic!(
            "the installer-candidate workflow reads the program but never archives it, so there \
             is nothing to hand over"
        ),
    }
}
