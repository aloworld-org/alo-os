//! **A candidate is a thing to try, never a thing to ship** — and since
//! ADR 0096 that is carried by six controls rather than by keeping it off the
//! download page.
//!
//! # What changed on 2026-10-07, and why this file reversed its central rule
//!
//! This held the candidate to *never makes a Release*, which was ADR 0046's
//! rejected option. That clause leaned on **unsigned** meaning *not for the
//! download page* — and the owner then published release 0.0.6 unsigned,
//! deliberately, with notes explaining SmartScreen. Unsigned stopped
//! distinguishing a candidate from a release, so a control had to be replaced
//! rather than merely added to.
//!
//! ADR 0096 is that replacement, and it reframes the question: not *may a
//! candidate be a pre-release*, but **may a workflow publish at all** — what a
//! draft protects being that publication is a person's act rather than a
//! workflow's. The controls below are the price of letting one publish.
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

/// The marker a candidate's tag is built with.
const THE_TAG_MARKER: &str = "candidate-";

/// The sentence a candidate's notes must carry, word for word.
///
/// A named constant rather than a rule about saying *plainly what it is*,
/// because prose is what a test cannot hold to anything — the fault four of
/// this repository's `docs/misreadings/` entries are about, and the third PC's
/// own correction to their first proposal.
const THE_NOTES_MARKER: &str = "It is not a release.";

/// The variable the build date is compiled in through, for the programme's own
/// first line.
const THE_DATE: &str = "ALO_INSTALLER_CANDIDATE_BUILT";

/// What would pin a candidate, which nothing here may touch.
const THE_PIN: &str = "pinned.toml";

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

/// **Everything it publishes is a pre-release, and nothing is ever latest.**
///
/// # This test used to say the opposite, and the reversal is the decision
///
/// It read `it_never_publishes` until 2026-10-07, holding the candidate to
/// ADR 0046's rejected option *publishing a Release that is not a draft*. That
/// clause leaned on **unsigned** meaning *not for the download page*, and on
/// 2026-10-07 the owner published release 0.0.6 unsigned, deliberately, with
/// notes explaining SmartScreen. The property stopped distinguishing anything.
///
/// ADR 0096 replaced it: a workflow may publish, and only a candidate, and the
/// six controls are **the price of letting it publish at all** rather than
/// decoration on a decision already made. This file holds five of them; the
/// sixth is the programme's own first line, held in `alo-installer`.
///
/// **This control is the only one enforced by somebody other than us.** GitHub
/// defines `/releases/latest` as the most recent non-prerelease, non-draft
/// release and refuses to make a pre-release latest, so it holds whether or
/// not this test runs and whether or not a future reader understands why.
#[test]
fn everything_it_publishes_is_a_prerelease_and_never_latest() {
    let candidate = the_candidate();

    assert!(
        candidate.publishes_only_prereleases(),
        "the installer-candidate workflow makes a Release that is not marked `--prerelease`. \
         GitHub's `/releases/latest` is the most recent non-prerelease release, so an unmarked \
         one becomes what a stranger downloads believing it was meant for them (ADR 0096)"
    );
    assert!(
        !candidate.makes_something_latest(),
        "the installer-candidate workflow asks for a Release to be the latest one, which is the \
         one thing marking it a pre-release exists to prevent (ADR 0096)"
    );
}

/// **Exactly one candidate exists at a time, and the previous goes first.**
///
/// *A candidate that lives forever becomes a release by default* — the third
/// PC's sentence, and the control none of us had until they wrote it. Every
/// distribution deletes its dailies.
///
/// Before rather than merely somewhere, for `looks_before_it_pushes`'s reason:
/// a delete that ran afterwards would take the candidate just published.
///
/// **Seven days is intent and nothing enforces it.** What is enforced is this:
/// the candidate on that page is always the newest. There is deliberately no
/// scheduled deleter, because its failure mode is deleting something somebody
/// is mid-walk on.
#[test]
fn exactly_one_candidate_exists_at_a_time() {
    assert!(
        the_candidate().deletes_the_previous_before_publishing(),
        "the installer-candidate workflow does not take down the previous candidate before \
         publishing the next, so candidates would accumulate on the releases page until one of \
         them was old enough to be wrong and still look current (ADR 0096)"
    );
}

/// **Its tag is built with the candidate marker, and never like a release.**
#[test]
fn its_tag_says_what_it_is() {
    let candidate = the_candidate();

    assert!(
        candidate.runs_something_naming(THE_TAG_MARKER),
        "the installer-candidate workflow does not build its tag with `{THE_TAG_MARKER}`, so \
         the tag on the releases page would not say what it is (ADR 0096)"
    );
    assert!(
        !candidate.runs_something_naming("v0."),
        "the installer-candidate workflow writes a tag that looks like a release version. \
         ROADMAP.md's *two numbers that look alike* is about exactly this confusion, and \
         ADR 0046 reserves the release tag to the owner"
    );
}

/// **Its notes carry the sentence, and the programme is given the date.**
///
/// Both by named constant. *Says plainly what it is* was in the first draft of
/// this design and came out, because a test cannot hold prose to anything —
/// which is the fault four `docs/misreadings/` entries describe and which the
/// third PC corrected in their own proposal.
#[test]
fn it_says_what_it_is_where_a_person_will_read_it() {
    let candidate = the_candidate();

    assert!(
        candidate.runs_something_naming(THE_NOTES_MARKER),
        "the installer-candidate workflow publishes notes that do not contain \
         `{THE_NOTES_MARKER}` -- the one sentence a person reads before downloading (ADR 0096)"
    );
    assert!(
        candidate.runs_something_naming(THE_DATE),
        "the installer-candidate workflow does not set `{THE_DATE}`, so the programme cannot \
         say which day it was built on. That is ADR 0096's fifth control and the only one that \
         survives the file outliving the page it came from"
    );
}

/// **It never pins.**
///
/// There is exactly one pin in this repository, so a candidate that is never
/// written into it cannot be resolved to by anything — a stronger invariant
/// than any distribution has available, and the reason it is worth a test of
/// its own rather than a line in the notes.
#[test]
fn it_never_pins() {
    assert!(
        !the_candidate().runs_something_naming(THE_PIN),
        "the installer-candidate workflow touches `{THE_PIN}`. The pin is what an installer \
         resolves to, and a candidate in it is a candidate somebody installs believing it is \
         the release (ADR 0096)"
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
