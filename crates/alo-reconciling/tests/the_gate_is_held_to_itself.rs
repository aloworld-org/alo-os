//! **The v0.5 exit gate, checked rather than read.**
//!
//! `ROADMAP.md`'s reconciliation log records what reading it by hand cost: the
//! gate was read seven times over three months, the eighth reading took a day,
//! and four of the eight refusals that reading wrote were wrong. Every check
//! here was specified by something that pass found, and this is where each of
//! them meets the real documents on the disk this test is running on.
//!
//! # Why one test per check
//!
//! Because a failure has to name what to do. A single test reporting *the gate
//! disagrees with the definition* would be the reading again with a red bar
//! instead of an afternoon. Each test below fails with the promise, the line and
//! which of the things went wrong.
//!
//! # Known debt is listed by name, with a reason, and nothing else is
//!
//! Two of these checks find something this repository owes and cannot pay from
//! here: a hue another lane owns, and a v1 box whose contents the definition puts
//! elsewhere. Those are listed as constants with the reason and who owes them, so
//! the check stays armed for everything new. A list that grows without a reader
//! noticing is the failure this whole crate exists to prevent, so each list is
//! short enough to read and each entry says what would remove it.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use alo_reconciling::the_gate::{
    counts_that_drifted, denials_by_task_number, orphaned_boxes, promises_with_no_box,
    refusals_a_report_contradicts, statuses_their_own_section_contradicts, tiers_that_disagree,
    withdrawals_left_open,
};
use alo_reconciling::tier::Tier;
use alo_reconciling::{Finding, promises_at, reconcile_at};

/// This repository.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is two directories above this crate")
}

/// One of this repository's documents, read.
fn document(at: &str) -> String {
    let at = the_repository().join(at);
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// Every plan under `docs/autonomy/`, by filename.
///
/// Every plan and not only this release's: the status words that disagreed with
/// their own bodies were in six plans across two releases and the supervisor's,
/// and a check that looked at one release would have found nine of seventy-four.
fn the_plans() -> BTreeMap<String, String> {
    let mut plans = BTreeMap::new();
    let at = the_repository().join("docs/autonomy");
    for entry in fs::read_dir(&at)
        .expect("docs/autonomy is readable")
        .flatten()
    {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.contains("plan") && name.ends_with(".md") {
            plans.insert(
                name.to_owned(),
                fs::read_to_string(&path).unwrap_or_default(),
            );
        }
    }
    assert!(
        plans.len() > 15,
        "only {} plans were found, which is not this repository",
        plans.len()
    );
    plans
}

/// Every report under `docs/autonomy/updates/`, by filename.
fn the_reports() -> BTreeSet<String> {
    let at = the_repository().join("docs/autonomy/updates");
    let mut reports = BTreeSet::new();
    for entry in fs::read_dir(&at).expect("updates is readable").flatten() {
        if let Some(name) = entry.path().file_name().and_then(|name| name.to_str())
            && name.ends_with(".md")
        {
            reports.insert(name.to_owned());
        }
    }
    assert!(
        reports.len() > 100,
        "only {} reports were found, which is not this repository",
        reports.len()
    );
    reports
}

/// How a list of findings is reported.
fn say(findings: &[Finding]) -> String {
    findings
        .iter()
        .map(|finding| format!("\n  {finding}"))
        .collect()
}

/// The findings a list of known debt does not account for.
fn beyond<'a>(findings: &'a [Finding], known: &[(&str, &str)]) -> Vec<&'a Finding> {
    findings
        .iter()
        .filter(|finding| {
            let said = finding.to_string();
            !known.iter().any(|(what, _)| said.contains(what))
        })
        .collect()
}

/// How an unaccounted-for finding is reported.
fn say_each(findings: &[&Finding]) -> String {
    findings
        .iter()
        .map(|finding| format!("\n  {finding}"))
        .collect()
}

/// **No half is under a promise it is not about.**
///
/// The v0.5 cut moved nine promises to v1 by their first line alone and left four
/// boxes behind, two of them ticked. A promise carrying two code halves is that,
/// and so is a promise carrying two machine halves.
#[test]
fn no_half_is_under_the_wrong_promise() {
    let findings = orphaned_boxes(&document("ROADMAP.md"));
    assert!(
        findings.is_empty(),
        "{} half/halves are under a promise they are not about:{}",
        findings.len(),
        say(&findings)
    );
}

/// **A promise with no box, and a promise answered at the wrong tier, are found
/// — and neither can gate yet. This test says why, with the numbers.**
///
/// Both checks have to decide whether a box *covers* a promise, and the gate
/// groups: **31 boxes over 92 promises.** *Settings, as one place* is one box over
/// nine areas; *Making it yours* is one box over six promises about appearance.
/// So most promises are covered by a box that does not quote them, and coverage is
/// a judgement the gate expresses in prose.
///
/// Two settings were measured against the real documents, and neither is a check:
///
/// | how a box is taken to answer a promise | what it reported |
/// |---|---|
/// | half the promise's words over two characters appear anywhere in the section | **3** promises with no box — and it missed at least two real ones, because half the words of *Audio in and out, with device switching that works mid-call* are `and`, `out`, `with`, `that` |
/// | a run of four consecutive words of five characters or more | **69** of the 92, including *Multi-monitor, display scaling, hotplug*, which has a box titled *Multi-monitor, scaling, hotplug* |
///
/// **Tuning that number until the answer looks right would be fitting the check to
/// the documents**, which is the reading it replaces wearing a test's name. The
/// answer is not a better matcher: **it is for each box in the gate to name the
/// promises it answers.** Then this is exact — two sets, compared — and no
/// heuristic is involved. That is a change to the gate's own shape and is written
/// up in
/// `docs/autonomy/updates/the-gate-is-a-check-now-and-what-it-cannot-check-yet.md`
/// for whoever owns `ROADMAP.md`.
///
/// Until then both functions are kept, unit-tested against fixtures in
/// `the_gate`, and run here against the real documents so they cannot rot into
/// code nobody compiles — asserting only what is reliable: that each **does** find
/// the thing it was written for.
#[test]
fn a_promise_with_no_box_is_found_even_though_it_cannot_gate_yet() {
    let findings = promises_with_no_box(
        &document("docs/features.md"),
        &document("ROADMAP.md"),
        Tier::V0_5,
    );
    let said = say(&findings);
    for known in [
        "Input methods for non-Latin scripts",
        "Bluetooth: pairing, audio, keyboards, mice",
        "Full-disk encryption, enrolled at install",
    ] {
        assert!(
            said.contains(known),
            "the check no longer finds `{known}`, which has no box in the v0.5 gate and was found \
             by hand on 2026-09-27. Either a box was added for it — in which case remove it from \
             this list — or the check stopped working"
        );
    }

    let tiers = tiers_that_disagree(&document("docs/features.md"), &document("ROADMAP.md"));
    let tiers = say(&tiers);
    for known in EVERY_DISAGREEMENT_THAT_STANDS {
        assert!(
            tiers.contains(known),
            "the check no longer finds `{known}`, which it reported against the real documents on \
             2026-09-29. Either the documents were reconciled — in which case take it off that \
             list and say so — or the check stopped working"
        );
    }
}

/// **Every tier disagreement this check reports against the real documents**,
/// measured on 2026-09-29 rather than recalled.
///
/// # Why this list exists
///
/// The assertion above named **one** of these — *Camera and microphone* — and that
/// one was true. What it did not say is that the check reports **six findings
/// across five promises**, so four promises stood unmentioned in a repository
/// that runs this test on every gate. A reader of the old assertion would have
/// concluded the check had one finding and it was known about.
///
/// That is not the check failing to work. It is a reader learning less from a
/// passing test than the test's own function knew, and the remedy is cheap:
/// name them all.
///
/// # What this list is not
///
/// **It is not an approval, and it is deliberately not a count.** The test above
/// says at length why neither of these checks can gate yet, and asserting *and
/// nothing else* would pin the output of a heuristic — the same fitting-to-the-
/// documents that docstring refuses. **A new disagreement appearing will not fail
/// this test**, and that is a stated limit rather than an oversight.
///
/// What it does buy: each of the five is held **live**. If the documents are
/// reconciled, or the matcher drifts, the one that goes quiet is named.
///
/// # What each one is, on inspection
///
/// Three look like a promise **kept earlier than it was promised**, which is not a
/// contradiction: *Lock screen, suspend and resume* and *Drag and drop between
/// applications* are `[v0.5]` in the definition and answered inside v0.01's gate,
/// and *Serving more than one person from one workstation* is `[v1]` answered in
/// v0.01's. **Work done ahead of its tier reports the same as work at the wrong
/// tier**, because the check compares where a promise is answered with where it is
/// promised and cannot see which direction is a fault.
///
/// Three are the kind it was written for — *Camera and microphone*, *Drag and drop
/// between applications* also answered in v1's gate, and the self-hosted workspace
/// line: each `[v0.5]` in the definition and answered in v1's gate, which is a
/// promise the scope gate says is in this release and the roadmap puts in the next.
///
/// **Which of those is a fault and which is a stale sentence is the owner's, not
/// this test's.** It is recorded here so the question exists somewhere other than
/// in a passing check's silence.
const EVERY_DISAGREEMENT_THAT_STANDS: [&str; 5] = [
    // Most likely kept earlier than promised, these two and the next.
    "Lock screen, suspend and resume",
    "Serving more than one person from one workstation",
    // Answered in two gates at once, v0.01's and v1's — its own question, and the
    // reason five promises produce six findings.
    "Drag and drop between applications",
    // The kind the check was written for.
    "Camera and microphone",
    "A self-hosted workspace on the network is **discovered, not configured**",
];

/// **No box denies work that its own named task says is done.**
///
/// *Machines find each other* named task 12 of the local-network plan as *open*
/// for twelve days after it was finished, and *"Where is that file?"* named task
/// 11 of the machine-measured plan as *open* for thirteen — the second found by
/// this check on the day it was written. A clause precise enough to name a task
/// number reads as authoritative, which makes it worse than vagueness.
#[test]
fn no_box_denies_work_a_plan_says_is_done() {
    let findings = denials_by_task_number(&document("ROADMAP.md"), &the_plans());
    assert!(
        findings.is_empty(),
        "{} box(es) deny work a plan says is done:{}",
        findings.len(),
        say(&findings)
    );
}

/// **No task's status word is contradicted by its own body.**
///
/// Seventy-four were, across six plans, on 2026-09-27 — thirty-two of them in
/// `v0-01-delivery-plan.md`, a release that shipped. `SHARED_MAIN.md` had a rule
/// telling every reader to route around them; this is the check that rule was
/// standing in for.
#[test]
fn no_status_is_contradicted_by_its_own_section() {
    let mut findings = Vec::new();
    for (named, plan) in the_plans() {
        findings.extend(statuses_their_own_section_contradicts(&plan, &named));
    }
    assert!(
        findings.is_empty(),
        "{} task(s) state a status their own body contradicts. The `**Done,` marker is the one to \
         trust, so the status word is what to correct:{}",
        findings.len(),
        say(&findings)
    );
}

/// **A refusal in the gate has been shown the reports that look like it.**
///
/// The weakest of the checks and the one written for the worst mistake: four
/// refusals on 2026-09-26 were negative claims from searches narrower than the
/// claim, and reports had answered all four for days. It cannot know whether a
/// report is about the promise, so what it produces is a list to look at rather
/// than a verdict — and a refusal that survives being shown its reports is a
/// refusal somebody looked at.
#[test]
fn every_refusal_has_been_shown_the_reports_that_look_like_it() {
    /// A refusal, and why the report that looks like it does not contradict it.
    ///
    /// Removing an entry here means either withdrawing the refusal or finding
    /// that the report does contradict it. Adding one without looking is the
    /// mistake this check exists for.
    /// **Two entries were here on 2026-09-27 and were removed by looking.** *A
    /// web browser for the open web* refused because no browser was pinned, and
    /// *The ordinary desktop* because four applications did not exist and the
    /// decision between writing them and pinning upstream ones was owed. Both
    /// were wrong: `crates/alo-software/shipped.toml` has pinned all seven since
    /// 2026-09-15, with a test and a report. Those refusals are withdrawn in
    /// `ROADMAP.md` rather than listed here, which is what this check is for.
    const LOOKED_AT: [(&str, &str); 1] = [(
        "Undo what the agent did",
        "undo-what-the-agent-did.md is task 4, and \
         undo-what-the-agent-did-waits-on-a-snapshot-road.md is the finding this box \
         quotes: built all round, waiting on btrfs at install",
    )];

    let findings = refusals_a_report_contradicts(&document("ROADMAP.md"), &the_reports());
    let unlooked = beyond(&findings, &LOOKED_AT);
    assert!(
        unlooked.is_empty(),
        "{} refusal(s) have a report whose filename reads like the promise and are not listed as \
         looked at. Look, then either withdraw the refusal or add it to LOOKED_AT with the reason \
         the report does not contradict it:{}",
        unlooked.len(),
        say_each(&unlooked)
    );
}

/// **A number the definition states is the number the code has.**
///
/// *Making it yours* stayed ticked, claiming *the accent set as working code: five
/// hues*, after `Accent::ALL` became `[Self; 4]`. The change that falsified it
/// touched neither `ROADMAP.md` nor `docs/features.md` and passed review.
///
/// The counts are read here rather than in the crate, because reading Rust is not
/// that crate's business and a caller holding the value is the honest place for
/// it.
#[test]
fn every_count_the_definition_states_is_the_count_the_code_has() {
    /// A count that has drifted, and who owes it.
    ///
    /// **One entry, and it is a debt rather than an exemption.**
    /// `docs/features.md` promises five designed hues and ADR 0067 requires five;
    /// `Accent::ALL` has four, because #185 removed `Token::Terracotta` and never
    /// added `Accent::Terracotta`. The `ROADMAP.md` tick was withdrawn on
    /// 2026-09-26 rather than left standing, so nothing in this repository claims
    /// the promise is met. It is owed by whoever owns `alo-appearance`, with the
    /// contrast measurement the ADR records as *to be measured*, and this entry
    /// goes when the fifth hue lands.
    const OWED: [(&str, &str); 1] = [(
        "the designed accent hues",
        "ADR 0067 half-implemented by #185: the token was removed and the accent never added. \
         The ROADMAP tick is withdrawn, so no document claims otherwise",
    )];

    let accents = the_accents_the_code_has();
    let counted = [("the designed accent hues", 5, accents)];
    let findings = counts_that_drifted(&counted);
    let unaccounted = beyond(&findings, &OWED);
    assert!(
        unaccounted.is_empty(),
        "{} count(s) the definition states are not the count the code has:{}",
        unaccounted.len(),
        say_each(&unaccounted)
    );
    assert!(
        accents > 0,
        "no accents were counted at all, so this check is reading the wrong thing rather than \
         holding a number"
    );
}

/// How many accents `alo-appearance` actually has, read out of its source.
///
/// Read rather than linked, because this crate depends on nothing in the
/// workspace and a check that had to be given a dependency to hold a number would
/// have to be given one for every number.
fn the_accents_the_code_has() -> usize {
    let source = document("crates/alo-appearance/src/accent.rs");
    source
        .split_once("pub const ALL: [Self;")
        .and_then(|(_, rest)| rest.split_once(']'))
        .and_then(|(how_many, _)| how_many.trim().parse::<usize>().ok())
        .unwrap_or(0)
}

/// **Every v0.5 promise names the test or the report that shows it, or is named
/// as owed** — against this repository, on the disk it is checked out on.
///
/// The same audit `every_v0_01_promise_is_reconciled.rs` runs for the first
/// release, at the second's tier. 92 promises rather than 41, which is why its
/// ledger groups them with bold lines instead of headings: every `###` under the
/// one heading is an entry, and a group heading would have taken a section of
/// entries out of the audit without failing it.
#[test]
fn every_v0_5_promise_is_reconciled_against_evidence_that_runs() {
    let here = the_repository();
    let off_the_disk = move |named: &str| fs::read_to_string(here.join(named)).ok();
    let (ledger, _) = Tier::V0_5
        .ledger()
        .expect("v0.5 has a ledger, or nothing above is checking anything");

    match reconcile_at(
        &document("docs/features.md"),
        &document(ledger),
        Tier::V0_5,
        &off_the_disk,
    ) {
        Ok(reconciled) => {
            assert_eq!(
                reconciled.promises(),
                promises_at(&document("docs/features.md"), Tier::V0_5).len(),
                "the reconciliation counted a different number of promises than the definition \
                 makes"
            );
            assert!(
                reconciled.promises() > 50,
                "only {} v0.5 promises were read, which means this audit passed over a document \
                 it never found",
                reconciled.promises()
            );
            assert_eq!(
                reconciled.wholly_shown() + reconciled.partly_owed() + reconciled.wholly_owed(),
                reconciled.promises(),
                "every promise is wholly shown, partly owed or wholly owed, and these do not add \
                 up to the promises read"
            );
        }
        Err(findings) => panic!(
            "{} finding(s) between docs/features.md and {ledger}:{}",
            findings.len(),
            say(&findings)
        ),
    }
}

/// **No promise this repository withdrew is still open in the roadmap, and this
/// one gates.**
///
/// Its neighbours above cannot: whether a box *covers* a promise is a judgement,
/// and every threshold measured either missed real cases or flagged most of the
/// document. This asks a narrower question with a definite answer — a withdrawal
/// names the promise in the words the roadmap's own box was written from, so the
/// match is between two copies of one phrase, and **the right number of findings
/// is zero**. A withdrawn promise left open is never correct.
///
/// It exists because one was. ADR 0076 withdrew *per display, so the dock can sit
/// along the bottom of the laptop and down the side of the external screen*;
/// `docs/features.md` recorded it; `ROADMAP.md` kept the box open for three days
/// with its code half ticked, citing a function the same change had deleted. A
/// lane picking it up would have been asked to build a thing this repository had
/// decided not to have.
#[test]
fn no_withdrawn_promise_is_still_open() {
    let findings = withdrawals_left_open(&document("docs/features.md"), &document("ROADMAP.md"));
    assert!(
        findings.is_empty(),
        "{} withdrawn promise(s) are still open boxes in ROADMAP.md:\n\n- {}",
        findings.len(),
        say(&findings)
    );
}
