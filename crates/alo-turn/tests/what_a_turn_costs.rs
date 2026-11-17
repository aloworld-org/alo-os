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
//! # Judged against the machine it runs on, not against a clock
//!
//! **Until 2026-11-17 the bound was 2 ms, wall-clock, at the median** — about
//! fifty times the 38 µs measured on the gating machine on 2026-10-13, so that a
//! fiftyfold regression failed here. It stopped meaning that: the suite runs in
//! debug, in parallel, on whatever else the runner is doing, and a turn there
//! measured 1.4 to 3.0 ms on `main` itself — so the test failed `main` one run
//! in three on a lane's machine, and failed merge-queue builds on CI, throwing
//! out whole groups of changes, for a reason that was the runner and not alo.
//!
//! So a turn is now timed **beside a reference act**, interleaved, in the same
//! process: the same file written and renamed, a line appended to a record, and
//! a fixed piece of the string work a sentence is made of — what any program
//! pays on this machine, at this moment, for the same kind of act. A loaded
//! runner slows both. What is asserted is **the ratio of their medians**
//! ([`AT_THE_MEDIAN`]), so alo OS getting slower fails and the machine getting
//! slower does not. The intent of the 2 ms bound is kept — **an order of
//! magnitude fails here** — and [`a_turn_made_slow_is_caught`] proves it on
//! every run, by making a turn slow and seeing the judgement refuse it.
//!
//! It is **not** the hardware measurement `CLAUDE.md` asks for, and says so:
//! that is a certified machine, and this is a gating virtual machine with two
//! cores.

// Counted: `unwrap` throughout and no `expect`, `panic!` or `unreachable!`, so
// only the one is listed — an unfulfilled expectation is an error here.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::io::Write;
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

/// How many turns are timed, each beside one reference act.
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

/// The most a turn may cost **in reference acts**, at the median: alo OS's
/// turn over the same kind of act any program performs, timed beside it.
///
/// **Measured first, then set**, on 2026-11-17, in a debug build on a lane's
/// machine (WSL on a shared Windows server, loaded by other lanes' builds); a
/// reference act took 208–243 µs, a turn 2.0–2.4 ms and a refusal 1.9–2.9 ms:
///
/// | | a turn, five runs | a refusal, five runs |
/// |---|---|---|
/// | this test alone, one thread | 9.6–10.2 | 8.0–9.5 |
/// | its three tests in parallel | 9.5–11.2 | 9.2–13.5 |
///
/// The bound is **100**: ten times the usual ratio and seven times the highest
/// seen — the 2 ms bound's own intent, *an order of magnitude fails here*, now
/// judged against the machine rather than a clock. On this machine it is a turn
/// of about 20 ms.
const AT_THE_MEDIAN: f64 = 100.0;

/// And the most it may add to the slowest turn of the fifty, wall-clock.
///
/// Separate from the median because they fail for different reasons: a median
/// that moves is work that got slower, and a tail that moves is work that
/// sometimes waits for something — a disk, a lock, another process. Kept as a
/// clock rather than a ratio because what it catches is a turn that *blocks* —
/// on a network, a lock or a process — and 100 ms is far beyond anything a
/// loaded runner adds to work that does not.
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

/// **The reference act**: what any program pays on this machine, now, for the
/// same kind of act a turn is — a file renamed, a line appended to a record,
/// and the string work of wording and validating a sentence. Nothing of alo
/// OS's is in it, so a machine that is slow makes it slow too.
///
/// The file is made before the clock starts, as a turn's is.
fn a_reference_act(folder: &Path, which: usize) -> Duration {
    let file = folder.join(format!("reference-{which}.pdf"));
    fs::write(&file, b"an invoice").unwrap();
    let journal = folder.join("reference-journal.txt");

    let began = Instant::now();
    fs::rename(&file, folder.join(format!("reference-{which}-final.pdf"))).unwrap();
    let mut words: Vec<String> = (0..200)
        .map(|n| format!("rename {n} “march-{which}.pdf” to “march-{which}-final.pdf”"))
        .collect();
    words.sort_unstable();
    let line = std::hint::black_box(words.join(" ")).len();
    let mut kept = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal)
        .unwrap();
    writeln!(kept, "{which} {line}").unwrap();
    began.elapsed()
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

/// One refused proposal — a real file in a folder nobody granted — timed.
fn one_refusal(
    granted: &Path,
    out_there: &Path,
    strings: &Strings,
    record: &mut Record,
    which: usize,
) -> Duration {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@files",
            Reach::Folder(granted.to_path_buf()),
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
    let took = began.elapsed();
    assert!(
        refused.is_err(),
        "a folder nobody granted was put to somebody"
    );
    took
}

/// The median of some durations.
fn median_of(mut took: Vec<Duration>) -> Duration {
    took.sort_unstable();
    took.get(took.len() / 2).copied().unwrap_or_default()
}

/// What one measurement found: the act's durations and the reference's, taken
/// in pairs, and the ratio of their medians.
struct Measured {
    /// The act's median.
    median: Duration,
    /// The act's slowest.
    slowest: Duration,
    /// The reference act's median.
    reference: Duration,
}

impl Measured {
    /// How many reference acts the act costs, at the median.
    fn ratio(&self) -> f64 {
        self.median.as_secs_f64() / self.reference.as_secs_f64().max(f64::MIN_POSITIVE)
    }

    /// **The judgement**: within the bound, or why not.
    fn judged(&self, what: &str) -> Result<(), String> {
        let ratio = self.ratio();
        if ratio > AT_THE_MEDIAN {
            return Err(format!(
                "{what} costs {ratio:.1} reference acts at the median ({:?} beside {:?}), over \
                 {AT_THE_MEDIAN}",
                self.median, self.reference
            ));
        }
        if self.slowest > AT_THE_TAIL {
            return Err(format!(
                "the slowest {what} of {HOW_MANY} took {:?}, over {AT_THE_TAIL:?}",
                self.slowest
            ));
        }
        Ok(())
    }
}

/// Time `act` and the reference act in pairs, interleaved — the order swapped
/// each time, so neither always runs on a warmer machine — and say what was
/// found. `slowed_by` makes each act slower by that many of its own pair's
/// reference acts, which is how [`a_turn_made_slow_is_caught`] proves the
/// judgement refuses a slow alo OS.
fn measured(
    folder: &Path,
    mut act: impl FnMut(usize) -> Duration,
    slowed_by: Option<u32>,
) -> Measured {
    for which in 0..THROWN_AWAY {
        let _ = act(which);
        let _ = a_reference_act(folder, which);
    }
    let mut took = Vec::with_capacity(HOW_MANY);
    let mut references = Vec::with_capacity(HOW_MANY);
    for which in THROWN_AWAY..THROWN_AWAY + HOW_MANY {
        let (act_took, reference) = if which % 2 == 0 {
            let act_took = act(which);
            (act_took, a_reference_act(folder, which))
        } else {
            let reference = a_reference_act(folder, which);
            (act(which), reference)
        };
        let act_took = match slowed_by {
            Some(times) => {
                let more = reference * times;
                std::thread::sleep(more);
                act_took + more
            }
            None => act_took,
        };
        took.push(act_took);
        references.push(reference);
    }
    let slowest = took.iter().max().copied().unwrap_or_default();
    Measured {
        median: median_of(took),
        slowest,
        reference: median_of(references),
    }
}

/// **What alo OS adds to one turn, judged against the machine it runs on.**
///
/// Measured at **37.8 µs** at the median on 2026-10-13 in a release build — so
/// essentially all of a turn a person waits through is the model thinking and
/// the person reading, and alo OS's own share of it is not something a person
/// can perceive.
///
/// Printed as well as asserted, so a person reading the suite's output can see
/// the numbers and not only that they were inside a bound. `cargo test -p
/// alo-turn --test what_a_turn_costs -- --nocapture` shows them.
#[test]
fn what_alo_os_adds_to_a_turn_is_microseconds() {
    let folder = a_folder_of_our_own("median");
    let strings = on_this_machine();
    let mut record = Record::default();

    let found = measured(
        &folder,
        |which| one_turn(&folder, &strings, &mut record, which),
        None,
    );
    println!(
        "alo OS adds, over {HOW_MANY} turns: median {:?} beside a reference act's {:?} — {:.1} \
         reference acts; slowest {:?}",
        found.median,
        found.reference,
        found.ratio(),
        found.slowest
    );
    println!(
        "  (validating, asking the grants, resolving, wording the sentence, spending the \
         approval, moving the file, writing the record)"
    );
    println!("  not measured: the model, the person, the socket, the kernel's boundary");

    found.judged("a turn").unwrap();
    let _ = fs::remove_dir_all(folder);
}

/// **A turn that is refused costs no more than one that runs**, which is what
/// stops a refusal from being a way to tell whether something exists — judged
/// the same way, against the reference act timed beside it.
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

    let found = measured(
        &granted,
        |which| one_refusal(&granted, &out_there, &strings, &mut record, which),
        None,
    );
    println!(
        "a refused proposal costs, at the median: {:?} beside a reference act's {:?} — {:.1} \
         reference acts",
        found.median,
        found.reference,
        found.ratio()
    );

    found
        .judged("a refused proposal")
        .map_err(|why| {
            format!("{why} — an expensive refusal is one an agent can cause a great many of")
        })
        .unwrap();

    let _ = fs::remove_dir_all(granted);
    let _ = fs::remove_dir_all(ungranted);
}

/// **The judgement refuses a slow alo OS** — proved on every run rather than
/// claimed. Each turn is made slower by twice the bound in reference acts, so
/// whatever this machine is doing, a turn costs more than [`AT_THE_MEDIAN`] of
/// them; a judgement that passed it would be one that cannot fail.
#[test]
fn a_turn_made_slow_is_caught() {
    let folder = a_folder_of_our_own("slowed");
    let strings = on_this_machine();
    let mut record = Record::default();

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the bound is a small positive whole-ish number, written above"
    )]
    let twice_the_bound = (AT_THE_MEDIAN * 2.0).ceil() as u32;
    let found = measured(
        &folder,
        |which| one_turn(&folder, &strings, &mut record, which),
        Some(twice_the_bound),
    );
    println!(
        "a turn slowed by {twice_the_bound} reference acts: {:.1} reference acts",
        found.ratio()
    );
    assert!(
        found.judged("a slowed turn").is_err(),
        "a turn slowed by {twice_the_bound} reference acts passed, at {:.1} of them — the \
         judgement cannot fail",
        found.ratio()
    );
    let _ = fs::remove_dir_all(folder);
}
