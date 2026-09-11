//! The audit itself, run against this repository.
//!
//! Everything in the crate is the method. This is the measurement: the real
//! `docs/features.md`, the real `docs/autonomy/v0-01-evidence.md`, and the real
//! files the ledger names — read off the disk this test is running on, so a test
//! that was renamed and a report that was never written both fail here rather
//! than in somebody's reading six months from now.
//!
//! # And the refusals, beside it
//!
//! A green audit says the two documents agree. It cannot say the audit would
//! have noticed if they did not, and that is the half a reader has to believe
//! rather than see — so every finding this crate can produce is put in front of
//! it here against a fixture, including the one that matters most: **a promise
//! nobody reconciled at all.**

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use alo_reconciling::{
    Finding, ledger::THE_ENTRIES, promises_in, reconcile, reconciling::Reconciled,
};

/// The only list of what gets built, which is the definition being audited.
const THE_DEFINITION: &str = "docs/features.md";

/// The other half of it: the evidence for every v0.01 promise, or what is owed.
const THE_LEDGER: &str = "docs/autonomy/v0-01-evidence.md";

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// A file of this repository, read.
fn reading(named: &str) -> String {
    let at = the_repository().join(named);
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// **Every v0.01 promise names the test or the report that shows it, or is named
/// as owed** — against this repository, on the disk it is checked out on.
#[test]
fn every_v0_01_promise_is_reconciled_against_evidence_that_runs() {
    let here = the_repository();
    let off_the_disk = move |named: &str| fs::read_to_string(here.join(named)).ok();

    match reconcile(
        &reading(THE_DEFINITION),
        &reading(THE_LEDGER),
        &off_the_disk,
    ) {
        Ok(reconciled) => {
            assert_eq!(
                reconciled.promises(),
                promises_in(&reading(THE_DEFINITION)).len(),
                "the reconciliation counted a different number of promises than \
                 the definition makes"
            );
            assert!(
                reconciled.promises() > 0,
                "there are no v0.01 promises in {THE_DEFINITION} at all, which \
                 means this audit passed over a document it never found"
            );
            assert_eq!(
                reconciled.wholly_shown() + reconciled.partly_owed() + reconciled.wholly_owed(),
                reconciled.promises(),
                "a promise was reconciled without being counted as shown, \
                 partly owed or owed, so the shape of the release does not add \
                 up to the release"
            );
        }
        Err(findings) => {
            let listed: Vec<String> = findings.iter().map(ToString::to_string).collect();
            panic!(
                "{} thing(s) do not add up between {THE_DEFINITION} and {THE_LEDGER}:\n\n- {}",
                listed.len(),
                listed.join("\n\n- ")
            );
        }
    }
}

/// Where the settled arguments live.
const THE_DECISIONS: &str = "docs/decisions/";

/// How a markdown document ends the name of one.
const A_DOCUMENT: &str = ".md";

/// Every decision a piece of text names, in the order it names them.
///
/// A ledger entry that cannot answer *shown by* sometimes answers *waiting on*,
/// and what it waits on is an ADR. That is the one kind of pointer this crate
/// deliberately refuses as **evidence** — a decision is where an argument was
/// settled, not proof that anything was built — which is exactly why nothing
/// checked it and why a promise could come to rest on a document nobody wrote.
fn decisions_named_in(text: &str) -> Vec<String> {
    text.split(|it: char| it.is_whitespace() || "`(),;:*".contains(it))
        .filter(|word| word.starts_with(THE_DECISIONS) && word.ends_with(A_DOCUMENT))
        .map(str::to_owned)
        .collect()
}

/// The ones among them that are not on this disk.
fn decisions_missing_from(named: &[String]) -> Vec<String> {
    let here = the_repository();
    named
        .iter()
        .filter(|at| !here.join(at).is_file())
        .cloned()
        .collect()
}

/// **A promise that waits on a decision names one that exists** — against this
/// repository, on the disk it is checked out on.
///
/// The promise this was written for is *the local model is what the machine
/// arrives ready to run* — before ADR 0025 was accepted on 2026-09-11 it read
/// *the agents point at the local model by default*, and its whole answer was
/// an argument rather than a test. The decision is taken now, so what the
/// entry points at is no longer a question waiting on the owner but the record
/// of the answer — and a pointer at a file nobody wrote would still read
/// exactly like an answer, which is this crate's own first sentence about why
/// the audit exists.
#[test]
fn a_promise_that_waits_on_a_decision_names_one_that_is_there() {
    let ledger = reading(THE_LEDGER);
    let named = decisions_named_in(&ledger);

    assert!(
        !named.is_empty(),
        "{THE_LEDGER} names no decision at all, so this check would pass over a \
         ledger that had stopped saying what its unanswerable promises wait on"
    );
    assert_eq!(
        decisions_missing_from(&named),
        Vec::<String>::new(),
        "the ledger sends a reader to a decision that is not in this repository"
    );

    let default = alo_reconciling::entries_in(&ledger)
        .into_iter()
        .find(|entry| entry.promise().contains("arrives ready to run"))
        .expect("the ledger still has an entry about the local model the machine arrives with");
    let owed = default
        .owed()
        .expect("the machine-arrives-ready promise is owed rather than shown")
        .sentence()
        .to_owned();
    assert!(
        default.names().is_empty(),
        "the machine-arrives-ready promise was reconciled as shown by something, \
         and no image this repository builds carries a model runtime or weights yet"
    );
    assert!(
        decisions_named_in(&owed).contains(&THE_REAL_DECISION.to_owned()),
        "the entry no longer names the accepted decision the reworded promise \
         rests on, so a reader cannot get from the ledger to why the wording \
         changed: {owed}"
    );
}

/// And the refusal beside it: a ledger sending a reader to a decision nobody
/// wrote.
///
/// The fixture is the shape the real entry has — an answer that is entirely
/// *what this waits on* — because that is the entry where a dead pointer would
/// be the whole of the answer rather than a detail beside one.
#[test]
fn a_ledger_naming_a_decision_nobody_wrote_is_a_finding() {
    // Assembled rather than written out, because `alo-citing` reads this file
    // too: a pointer at a decision nobody wrote, spelled out here, would be a
    // dead pointer this repository really carries and a reader could follow.
    let nowhere = format!("docs/decisions/{}-a-decision-nobody-wrote.md", "0099");
    let dead = format!("**Still owed:** the owner decides, in `{nowhere}`.");
    assert_eq!(
        decisions_named_in(&dead),
        std::slice::from_ref(&nowhere),
        "a decision named in an entry was not read as one, so the check below \
         would pass by finding nothing to check"
    );
    assert_eq!(
        decisions_missing_from(&decisions_named_in(&dead)),
        [nowhere],
        "a decision that is not in this repository was accepted as somewhere a \
         reader could go"
    );

    let alive = format!("**Still owed:** the owner decides, in `{THE_REAL_DECISION}`.");
    assert_eq!(
        decisions_missing_from(&decisions_named_in(&alive)),
        Vec::<String>::new(),
        "a decision that really is on this disk was reported missing, so the \
         refusal above is about the checker rather than about the ledger"
    );
}

/// The decision this promise waits on, named once so both tests above break
/// together if it is ever renamed.
const THE_REAL_DECISION: &str =
    "docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md";

/// **Every promise with no evidence at all says where the work is, and the
/// pointer lands** — against this repository, on the disk it is checked out on.
///
/// This is the measurement task 19 of the delivery plan was written to take.
/// Three v0.01 promises have nothing behind them, and for those the sentence in
/// the ledger is the whole of what the next person inherits: a promise recorded
/// as missing and pointing nowhere is one whose reasoning gets derived again
/// from scratch, which is the seven-times-over reading this ledger exists to
/// end.
///
/// The count is asserted rather than described. If a fourth promise falls to
/// nothing, or one of the three is closed, this fails and whoever moved it
/// writes down which — a ledger whose own summary drifts is a ledger that reads
/// as an answer.
///
/// **It was four until 2026-09-11**, when task 20 of the delivery plan closed
/// *copy, cut and paste* — the one task 19 found in the wrong pile, sorted into
/// *needs a machine* by an audit that was finding six things at once.
/// `crates/alo-clipboard` is the work and the ledger's entry for it says what is
/// still owed underneath, which is the compositor wiring rather than the
/// selection.
#[test]
fn each_promise_with_no_evidence_names_where_the_work_is() {
    let here = the_repository();
    let off_the_disk = move |named: &str| fs::read_to_string(here.join(named)).ok();
    let ledger = reading(THE_LEDGER);

    let reconciled = reconcile(&reading(THE_DEFINITION), &ledger, &off_the_disk)
        .unwrap_or_else(|findings| panic!("the ledger does not add up: {findings:?}"));
    assert_eq!(
        reconciled.wholly_owed(),
        3,
        "the ledger's own account of itself says three v0.01 promises have no \
         evidence at all; the audit counted {}. Whichever moved, say so under \
         the promise it is about",
        reconciled.wholly_owed()
    );

    let owed_and_pointing: Vec<(String, Vec<String>)> = alo_reconciling::entries_in(&ledger)
        .into_iter()
        .filter(|entry| entry.names().is_empty())
        .map(|entry| {
            let said = entry.owed().map(|owed| owed.sentence().to_owned());
            let waits = said.map(|said| {
                alo_reconciling::Waiting::named_in(&said)
                    .iter()
                    .map(alo_reconciling::Waiting::said)
                    .collect()
            });
            (entry.promise().to_owned(), waits.unwrap_or_default())
        })
        .collect();

    assert_eq!(
        owed_and_pointing.len(),
        3,
        "the entries with no evidence are not the three the count says: \
         {owed_and_pointing:?}"
    );
    for (promise, waits) in &owed_and_pointing {
        assert!(
            !waits.is_empty(),
            "`{promise}` has no evidence and says nothing about where the work \
             is, so whoever reads it next starts the reading again"
        );
    }
}

/// And the refusal beside it: a promise with nothing behind it and nowhere to
/// send anybody.
#[test]
fn a_promise_with_no_evidence_and_nowhere_to_go_is_refused() {
    let nowhere = a_sound_ledger().replace(
        "; task 20 of `docs/autonomy/a-plan.md` is the increment",
        ", and nobody has scheduled anything about it",
    );
    let findings = auditing(&nowhere).expect_err("a promise owed with nowhere to go was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::APromiseOwedWithNowhereToGo { promise } if promise.contains("Copy, cut and paste")
        )),
        "a promise with no evidence and no decision or task behind it was \
         reconciled: {findings:?}"
    );

    // And a promise shown *in part* is not held to the same rule: the code it
    // already has is where the next reader goes. The sound ledger's file-verbs
    // entry is owed a certified machine and names no task, and it reconciles.
    let file_verbs = alo_reconciling::entries_in(&a_sound_ledger())
        .into_iter()
        .find(|entry| entry.promise().contains("File verbs"))
        .expect("the fixture still has an entry about the file verbs");
    assert!(!file_verbs.names().is_empty() && file_verbs.owed().is_some());
    assert!(
        auditing(&a_sound_ledger()).is_ok(),
        "a promise shown by a test was required to name a task as well"
    );
}

/// A promise waiting on a task nobody wrote, and on a plan nobody wrote.
///
/// The number is the pointer a reader believes without opening, because it looks
/// like a fact — and a plan's tasks are numbered from one, so a number that
/// matches nothing here matches something in every other plan.
#[test]
fn a_promise_waiting_on_a_task_nobody_wrote_is_refused() {
    let unwritten = a_sound_ledger().replace("task 20 of", "task 44 of");
    assert!(
        auditing(&unwritten)
            .expect_err("a promise sent to a task number the plan does not have was accepted")
            .iter()
            .any(|finding| matches!(
                finding,
                Finding::AWaitNobodyCanFollow {
                    why: alo_reconciling::waiting::NoSuchWait::NoSuchTask,
                    ..
                }
            )),
        "a task number that is in no plan was accepted as where the work is"
    );

    let elsewhere = a_sound_ledger().replace(
        "`docs/autonomy/a-plan.md`",
        "`docs/autonomy/a-plan-nobody-wrote.md`",
    );
    assert!(
        auditing(&elsewhere)
            .expect_err("a promise sent to a plan that is not there was accepted")
            .iter()
            .any(|finding| matches!(
                finding,
                Finding::AWaitNobodyCanFollow {
                    why: alo_reconciling::waiting::NoSuchWait::NoSuchPlan,
                    ..
                }
            )),
        "a plan nobody wrote was accepted as where the work is"
    );
}

/// A definition with three promises in it, written the way `docs/features.md`
/// writes them.
const A_DEFINITION: &str = "\
## The shell

- [v0.01] ★ **File verbs**: list, read, find, rename, move, archive
- [v0.5] Lock screen, suspend and resume
- [v0.01] Copy, cut and paste — text, images and files, across applications
- [v0.01] ★ **No telemetry.** Not \"anonymised telemetry\". None
";

/// A ledger about that definition, sound in every way this crate can check.
fn a_sound_ledger() -> String {
    format!(
        "\
# The evidence

{THE_ENTRIES}

### File verbs

**Shown by:** `crates/alo-files/tests/doing.rs`

**Still owed:** the verbs have never run on a certified machine, which is the
half of law 3 only hardware gives.

### Copy, cut and paste

**Still owed:** nothing in this repository implements a clipboard — no crate, no
protocol and no test; task 20 of `docs/autonomy/a-plan.md` is the increment.

### No telemetry

**Shown by:** `docs/autonomy/updates/network-egress-enforcement.md`
"
    )
}

/// The plan those entries send a reader to, in the shape both of this
/// repository's plans are written in.
const A_PLAN: &str = "\
# A plan

## Tasks

### 19. Something finished

### 20. The clipboard, before any screen
";

/// The repository those entries are about.
fn a_repository(named: &str) -> Option<String> {
    match named {
        "crates/alo-files/tests/doing.rs" => Some("#[test]\nfn a_file_is_read() {}".to_owned()),
        "crates/alo-files/src/lib.rs" => Some("//! Nothing is tested in here.".to_owned()),
        "docs/autonomy/updates/network-egress-enforcement.md" => {
            Some("# What left the machine".to_owned())
        }
        "docs/autonomy/a-plan.md" => Some(A_PLAN.to_owned()),
        _ => None,
    }
}

/// The audit over a fixture, as the findings it produced.
fn auditing(ledger: &str) -> Result<Reconciled, Vec<Finding>> {
    reconcile(A_DEFINITION, ledger, &a_repository)
}

/// A sound ledger is reconciled, and counted the way the release actually sits:
/// one shown, one shown-and-owed, one wholly owed.
///
/// It runs first for the reason every refusal below depends on: an audit that
/// refused everything would pass all of them and mean nothing.
#[test]
fn a_ledger_that_adds_up_is_reconciled_and_counted() {
    let reconciled = auditing(&a_sound_ledger())
        .unwrap_or_else(|findings| panic!("a sound ledger was refused: {findings:?}"));
    assert_eq!(reconciled.promises(), 3);
    assert_eq!(reconciled.wholly_shown(), 1);
    assert_eq!(reconciled.partly_owed(), 1);
    assert_eq!(reconciled.wholly_owed(), 1);
}

/// **A promise nobody reconciled** — the finding this whole crate exists for,
/// and the one that really happened six times over seven readings.
#[test]
fn a_promise_the_ledger_says_nothing_about_is_the_finding() {
    let silent = a_sound_ledger().replace("### Copy, cut and paste", "### Something else entirely");
    let findings = auditing(&silent).expect_err("a promise nobody reconciled was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::APromiseNobodyReconciled { promise } if promise.contains("Copy, cut and paste")
        )),
        "the audit did not name the promise nothing in the ledger is about: {findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| matches!(finding, Finding::AnEntryAboutNothing { .. })),
        "the entry left quoting a promise the definition does not make was \
         accepted: {findings:?}"
    );
}

/// **A promise with neither** — no test, no report, and nothing admitted to.
#[test]
fn a_promise_with_no_evidence_and_nothing_owed_is_refused() {
    let empty = a_sound_ledger().replace(
        "**Still owed:** nothing in this repository implements a clipboard — no crate, no\nprotocol and no test; task 20 of `docs/autonomy/a-plan.md` is the increment.",
        "It is being thought about.",
    );
    let findings = auditing(&empty).expect_err("a promise with neither was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::APromiseWithNeither { promise } if promise.contains("Copy, cut and paste")
        )),
        "{findings:?}"
    );
}

/// Evidence that is not there, and a source file that tests nothing offered as
/// though it were a test.
#[test]
fn evidence_that_cannot_be_run_is_refused() {
    let moved = a_sound_ledger().replace(
        "`crates/alo-files/tests/doing.rs`",
        "`crates/alo-files/tests/a_test_nobody_wrote.rs`",
    );
    assert!(
        auditing(&moved)
            .expect_err("a promise shown by a test that is not there was accepted")
            .iter()
            .any(|finding| matches!(finding, Finding::EvidenceThatIsNotThere { .. })),
        "a named test that does not exist was reconciled"
    );

    let untested = a_sound_ledger().replace(
        "`crates/alo-files/tests/doing.rs`",
        "`crates/alo-files/src/lib.rs`",
    );
    assert!(
        auditing(&untested)
            .expect_err("a source file with nothing tested in it was accepted")
            .iter()
            .any(|finding| matches!(finding, Finding::EvidenceThatIsNotThere { .. })),
        "a file that tests nothing was reconciled as showing a promise"
    );
}

/// A decision, a roadmap or another repository's file, offered as evidence.
#[test]
fn something_that_is_not_evidence_is_refused() {
    let settled = a_sound_ledger().replace(
        "`crates/alo-files/tests/doing.rs`",
        "`docs/decisions/0001-the-capability-model.md`",
    );
    assert!(
        auditing(&settled)
            .expect_err("an ADR was accepted as evidence that something was built")
            .iter()
            .any(|finding| matches!(finding, Finding::SomethingThatIsNotEvidence { .. })),
        "where an argument was settled was taken for evidence that the work \
         exists"
    );
}

/// A quotation that fits two promises, which would reconcile whichever it
/// reached first and leave the other reading as done.
///
/// The fixture is the real hazard rather than an invented one: `docs/features.md`
/// says *neither is the other's fallback* twice, once about a CPU and a GPU and
/// once about this machine and a provider, and they are different promises with
/// different evidence.
#[test]
fn an_entry_that_fits_two_promises_is_refused() {
    let twice_over = "\
- [v0.01] ★ **It works well on a CPU and it works well on a GPU**. Neither is the other's fallback
- [v0.01] ★ **Neither is the other's fallback.** A model on this machine and a provider you added
";
    let one_entry = format!(
        "{THE_ENTRIES}\n\n### Neither is the other's fallback\n\n**Shown by:** \
         `crates/alo-files/tests/doing.rs`\n"
    );
    assert!(
        reconcile(twice_over, &one_entry, &a_repository)
            .expect_err("an entry matching two promises was accepted")
            .iter()
            .any(|finding| matches!(
                finding,
                Finding::AnEntryAboutTwoPromises { promises, .. } if *promises == 2
            )),
        "an entry quoting words that appear in more than one promise \
         reconciled one of them by accident"
    );
}

/// Two entries about one promise, which is two people's knowledge of it with the
/// shorter answer on top.
#[test]
fn a_promise_answered_twice_is_refused() {
    let twice = format!(
        "{}\n\n### File verbs\n\n**Shown by:** `crates/alo-files/tests/doing.rs`\n",
        a_sound_ledger()
    );
    assert!(
        auditing(&twice)
            .expect_err("a promise answered by two entries was accepted")
            .iter()
            .any(|finding| matches!(finding, Finding::APromiseAnsweredTwice { .. })),
        "two entries about one promise were both reconciled"
    );
}

/// What is owed, said as a shrug.
#[test]
fn a_shrug_is_not_what_is_still_owed() {
    let shrug = a_sound_ledger().replace(
        "nothing in this repository implements a clipboard — no crate, no\nprotocol and no test; task 20 of `docs/autonomy/a-plan.md` is the increment.",
        "not yet.",
    );
    assert!(
        auditing(&shrug)
            .expect_err("`not yet` was accepted as what is owed on a release promise")
            .iter()
            .any(|finding| matches!(finding, Finding::AShrugRatherThanAnAnswer { .. })),
        "a promise was reconciled by a shrug"
    );
}

/// And the one that would otherwise be silent: a ledger this crate cannot find,
/// with a definition full of promises.
#[test]
fn a_ledger_with_no_entries_is_checking_nothing() {
    let renamed = a_sound_ledger().replace(THE_ENTRIES, "## The promises");
    let findings = auditing(&renamed).expect_err("an empty ledger was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::NothingToReconcile { promises, .. } if *promises == 3
        )),
        "a ledger with no entries in it passed as an audit of three promises: \
         {findings:?}"
    );
}
