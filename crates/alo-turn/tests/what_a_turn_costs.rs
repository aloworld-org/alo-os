//! **What alo OS itself adds to a turn**, measured on every build rather than
//! claimed once.
//!
//! `ROADMAP.md`'s first launch item is *a person asks the agent in plain words,
//! and the agent does it*, and the question anybody comparing this product with
//! another will ask is how long that takes. Almost all of it is the model
//! thinking and the person reading — neither of which alo OS controls. **What
//! alo OS controls is everything else**, and this is that number.
//!
//! # What is measured, and what is deliberately not
//!
//! A turn is five things, and only two of them are ours:
//!
//! | | |
//! |---|---|
//! | the model deciding what to ask for | **not measured** — the provider's or the runtime's |
//! | the person reading a sentence and deciding | **not measured** — a person's, and the one part that should be slow |
//! | validating the request and putting it to the person | **measured** |
//! | spending the approval and carrying the verb out | **measured** |
//! | the socket between an agent and the daemon | **not measured here** — `alo-agentd`'s, and a different crate's to answer for |
//!
//! So this is **propose plus approve**, on a real file, on a real disk, with a
//! real record written. It is the floor under every turn and the only part a
//! change in this repository can make slower.
//!
//! # Why a test with a bound rather than a number in a document
//!
//! `CLAUDE.md`: *say whether a measurement was performed once or runs forever*
//! — and *a report cannot tell them apart unless the writer says which it was.*
//! A figure in a report is true the day it is written. This runs on every
//! build, so a change that makes a turn ten times slower fails here rather than
//! being discovered by somebody using the product.
//!
//! **The bound is generous on purpose.** It runs on whatever the tests run on,
//! shared with whatever else is running, so a tight bound would fail for
//! reasons that have nothing to do with the work — and a test that fails for
//! unrelated reasons is one somebody turns off. What it is built to catch is an
//! order of magnitude, not a percentage.
//!
//! It is **not** the hardware measurement `CLAUDE.md` asks for, and says so:
//! that is a certified machine, and this is a gating virtual machine with two
//! cores.

// Counted: 16 `unwrap` and no `expect`, `panic!` or `unreachable!`, so only the
// one is listed — an unfulfilled expectation is an error in this workspace.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant, SystemTime};

use alo_capability::{Given, Grant, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Resolving, file_words};
use alo_record::Record;
use alo_strings::Strings;
use alo_turn::{Machine, Turning};

/// How many turns are timed.
///
/// Enough that one slow one cannot set the median, and few enough that this
/// stays a test rather than a benchmark: the suite runs it on every build.
const HOW_MANY: usize = 50;

/// How many are run and thrown away before anything is timed.
///
/// The first turn on a fresh process pays for things no later turn pays for —
/// the vocabulary being built, the allocator warming, the record file being
/// created. Including them would measure *starting a daemon*, which is a
/// different question with a different answer.
const THROWN_AWAY: usize = 5;

/// The most alo OS may add to one turn, at the median.
///
/// **Measured first, then set — and the first bound written here was useless.**
/// It was 50 ms, chosen before measuring, on the assumption that a turn costs
/// milliseconds. Measured on the gating virtual machine (aarch64, two cores)
/// on 2026-10-13, over 50 turns:
///
/// | | |
/// |---|---|
/// | | first run | second run |
/// |---|---|---|
/// | median | **37.8 µs** | **38.2 µs** |
/// | slowest of fifty | 56.9 µs | **356.5 µs** |
/// | fastest | 29.1 µs | 27.9 µs |
/// | a refused proposal, median | 23.9 µs | 28.1 µs |
///
/// **Two runs rather than one, and the reason is in the third row:** the median
/// moved by 1% and the slowest by sixfold. A single run would have made the
/// tail look like a property of the work. It is not — it is this machine — and
/// that is why the median is the assertion that means something and the tail's
/// bound is loose.
///
/// A 50 ms bound is **1,300 times** the median, which is a bound that cannot
/// fail and therefore says nothing. This is 2 ms — still about fifty times the
/// measured median, which is the headroom a shared two-core machine needs, and
/// tight enough that a fiftyfold regression fails here rather than being found
/// by somebody using the product.
///
/// A bound set before the measurement is a guess about what the answer should
/// be. This one is set from the answer.
const AT_THE_MEDIAN: Duration = Duration::from_millis(2);

/// And the most it may add to the slowest turn of the fifty.
///
/// Separate from the median because they fail for different reasons: a median
/// that moves is work that got slower, and a tail that moves is work that
/// sometimes waits for something — a disk, a lock, another process.
///
/// **Deliberately much looser than the median's bound**, because this one
/// really can be a shared machine rather than the work: the slowest measured
/// was 56.9 µs on one run and 356.5 µs on the next, and one turn of fifty being
/// preempted, or waiting on a disk
/// that is busy with somebody else's build, is a thing that happens and says
/// nothing about alo OS. What it still catches is a turn that *blocks* —
/// anything that waits on a network, a lock or a process would land here and
/// nowhere else.
const AT_THE_TAIL: Duration = Duration::from_millis(100);

/// The moment every turn here begins.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a turn, a grant and a proposal stand.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A folder of this test's own, resolved — a grant is over a place.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-turn-costs-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// Every crate's words in one vocabulary, which is the arrangement a real
/// machine is in — and the arrangement whose cost is being measured.
fn on_this_machine() -> Strings {
    let mut vocabulary = file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_keeping::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A boundary that is not one.
///
/// **This is why the number is a floor and not the whole truth, and it is said
/// here rather than left to be noticed.** On a real machine
/// `alo_turn::Bounding` puts the work in a cgroup the kernel imposes (ADR
/// 0013), which costs something this cannot measure — a library cannot impose
/// one, and `alo-turn`'s own `bounding` module refuses to ship an
/// implementation that does nothing precisely so no host gets ADR 0015's
/// guarantee switched off by accident.
///
/// So what is measured is **alo OS's own decisions**: validating, asking the
/// grants, resolving a path, wording a sentence, spending an approval, moving a
/// file and writing the record. The kernel's share is `alo-boundaryd`'s to
/// answer for.
struct NothingIsBounded;

impl alo_turn::Bounding for NothingIsBounded {
    fn carrying_out(
        &mut self,
        _reaching: &alo_files::Reaching,
        doing: alo_turn::Doing<'_>,
    ) -> Result<alo_turn::Done, alo_turn::NoBoundary> {
        Ok(doing.done())
    }

    fn carrying_out_a_departure(
        &mut self,
        _to: &[std::net::SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), alo_turn::NoBoundary> {
        doing();
        Ok(())
    }
}

/// One whole turn over a file that is really there, timed.
///
/// Proposed, put to a person, approved, carried out, recorded. The file is made
/// **before** the clock starts and the name it becomes is unique, so what is
/// timed is the turn and not this test's own housekeeping.
fn one_turn(folder: &Path, strings: &Strings, record: &mut Record, which: usize) -> Duration {
    let file = folder.join(format!("march-{which}.pdf"));
    fs::write(&file, b"an invoice").unwrap();

    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@files",
            Reach::Folder(folder.to_path_buf()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_verbs(
        strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        record,
    )
    .unwrap();
    let mut turning = Turning::beginning(
        Context::at_invocation(noon()),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();

    // **The clock starts here**: everything above is a daemon that is already
    // running, and everything below is what a turn costs.
    let began = Instant::now();
    let id = turning
        .proposing(
            "rename_file",
            &[
                ("file", Given::text(file.to_string_lossy().into_owned())),
                ("name", Given::text(format!("march-{which}-final.pdf"))),
            ],
            &grants,
            hour(),
            noon(),
        )
        .unwrap();
    let did = turning.approving(id, &grants, noon()).unwrap();
    let took = began.elapsed();

    // The work really happened, so a turn that refused early cannot be timed as
    // a fast one. **The cheapest way to make this number small is to do less**,
    // and this is what stops that from passing.
    assert!(did.to_files().is_some(), "the turn did not rename anything");
    assert!(
        folder.join(format!("march-{which}-final.pdf")).exists(),
        "the file was not renamed, so what was timed was not a turn"
    );
    took
}

/// **What alo OS adds to one turn, at the median and at the tail.**
///
/// Measured at **37.8 µs** at the median on 2026-10-13 — so essentially all of
/// a turn a person waits through is the model thinking and the person reading,
/// and alo OS's own share of it is not something a person can perceive. That is
/// the honest version of *faster than the others*: not that this is fast, but
/// that there is almost nothing here to be slow.
///
/// Printed as well as asserted, so a person reading the suite's output can see
/// the number and not only that it was inside a bound. `cargo test -p alo-turn
/// --test what_a_turn_costs -- --nocapture` shows it.
#[test]
fn what_alo_os_adds_to_a_turn_is_microseconds() {
    let folder = a_folder_of_our_own("median");
    let strings = on_this_machine();
    let mut record = Record::default();

    for which in 0..THROWN_AWAY {
        let _ = one_turn(&folder, &strings, &mut record, which);
    }
    let mut took: Vec<Duration> = (THROWN_AWAY..THROWN_AWAY + HOW_MANY)
        .map(|which| one_turn(&folder, &strings, &mut record, which))
        .collect();
    took.sort_unstable();

    let median = took.get(took.len() / 2).copied().unwrap_or_default();
    let tail = took.last().copied().unwrap_or_default();
    let fastest = took.first().copied().unwrap_or_default();
    println!(
        "alo OS adds, over {HOW_MANY} turns: median {median:?}, slowest {tail:?}, fastest \
         {fastest:?}"
    );
    println!(
        "  (validating, asking the grants, resolving, wording the sentence, spending the \
         approval, moving the file, writing the record)"
    );
    println!("  not measured: the model, the person, the socket, the kernel's boundary");

    assert!(
        median <= AT_THE_MEDIAN,
        "alo OS adds {median:?} to a turn at the median, over {AT_THE_MEDIAN:?}"
    );
    assert!(
        tail <= AT_THE_TAIL,
        "alo OS adds {tail:?} to the slowest of {HOW_MANY} turns, over {AT_THE_TAIL:?}"
    );

    let _ = fs::remove_dir_all(folder);
}

/// **A turn that is refused costs no more than one that runs**, which is what
/// stops a refusal from being a way to tell whether something exists.
///
/// Not a timing-attack defence and it is not claimed as one: the sentences
/// differ, the record differs, and a caller who can read either knows more than
/// a clock would tell them. What this catches is a refusal path that got
/// *expensive* — one that walked a directory, or asked a provider, or retried —
/// because the cheapest thing for an agent to do with an expensive refusal is
/// to cause a great many of them.
#[test]
fn a_refused_turn_is_not_dearer_than_one_that_runs() {
    let granted = a_folder_of_our_own("granted");
    let ungranted = a_folder_of_our_own("ungranted");
    let strings = on_this_machine();
    let mut record = Record::default();

    // A real file in a folder nobody granted.
    let out_there = ungranted.join("secret.pdf");
    fs::write(&out_there, b"not yours").unwrap();

    let mut refusals = Vec::new();
    for which in 0..HOW_MANY {
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked("@files", Reach::Folder(granted.clone()), noon(), hour()).unwrap(),
        );
        let mut indicator = Indicator::default();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();

        let began = Instant::now();
        let refused = turning.proposing(
            "rename_file",
            &[
                (
                    "file",
                    Given::text(out_there.to_string_lossy().into_owned()),
                ),
                ("name", Given::text(format!("taken-{which}.pdf"))),
            ],
            &grants,
            hour(),
            noon(),
        );
        refusals.push(began.elapsed());
        assert!(
            refused.is_err(),
            "a folder nobody granted was put to somebody"
        );
    }
    refusals.sort_unstable();
    let median = refusals
        .get(refusals.len() / 2)
        .copied()
        .unwrap_or_default();
    println!("a refused proposal costs, at the median: {median:?}");

    assert!(
        median <= AT_THE_MEDIAN,
        "a refusal costs {median:?}, over {AT_THE_MEDIAN:?} — an expensive refusal is one an \
         agent can cause a great many of"
    );

    let _ = fs::remove_dir_all(granted);
    let _ = fs::remove_dir_all(ungranted);
}
