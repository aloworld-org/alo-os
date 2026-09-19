//! Recognising a `.pages`, a `.heic` and a `.dwg` waits on a decision, and
//! this file is what holds it there.
//!
//! Task 6 of `docs/autonomy/v0-5-documents-and-paper-plan.md` asks for each of
//! the three to be recognised from its content and measured against **a real
//! file with its provenance**. No machine this team has can make one of any of
//! them, and the three ways out — wait for a real file, build one here, or
//! borrow somebody else's — decide between an unmet promise in
//! `docs/features.md` and a rule whose only evidence is that we agree with
//! ourselves. Neither is a worker's to pick, so the task was the decision:
//! [ADR 0057](../../../docs/decisions/0057-a-format-is-recognised-on-the-evidence-of-a-real-file.md).
//!
//! # What this file holds
//!
//! The shape `converting_waits_on_its_decision.rs` gives ADR 0039, for the same
//! reason: a decision nobody points at is a task the next worker takes up by
//! choosing an option themselves.
//!
//! - **The decision exists, once, under its number**, and still stands.
//! - **The plan points at it from task 6, and steps over task 6** while it is
//!   proposed.
//! - **It sets out what an owner answers by choosing**: three options, what
//!   each costs, one recommendation, what no option may do, and what must
//!   happen before the task is ready again.
//! - **While it says *proposed*, none of the three is recognised**: no kind is
//!   named for one, no name claims one, and a file of each reads as what this
//!   machine honestly does not recognise.
//!
//! It reads files and runs nothing, so it holds on any machine.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use alo_opening::{Cannot, Decided, EVERY_WORD, Kind, Named, Outcome, ThisMachine, Word, decide};
use alo_strings::Strings;

/// The decision's number, as `docs/decisions/README.md` says one is cited.
const THE_NUMBER: &str = "0057";

/// The decision's file, relative to the repository.
const THE_DECISION: &str =
    "docs/decisions/0057-a-format-is-recognised-on-the-evidence-of-a-real-file.md";

/// Where every decision lives.
const THE_DECISIONS: &str = "docs/decisions";

/// The plan that asked for the decision.
const THE_PLAN: &str = "docs/autonomy/v0-5-documents-and-paper-plan.md";

/// The heading of the task that produced it.
const THE_TASK: &str =
    "### 6. A `.pages`, a `.heic` and a `.dwg` — recognised, and converted or explained";

/// The line a decision records its status on.
const STATUS: &str = "**Status:**";

/// The statuses under which the decision still stands.
const STANDING: [&str; 2] = ["proposed", "accepted"];

/// The one status under which the three formats are held to being unrecognised.
const WAITING: &str = "proposed";

/// The headings an owner answers by choosing among.
const WHAT_IT_SETS_OUT: [&str; 4] = ["### A — ", "### B — ", "### C — ", "## The recommendation"];

/// What each option must say, as a phrase the decision carries.
const WHAT_EACH_COSTS: &str = "**What it costs:**";

/// What no option may do, whichever is chosen.
const WHAT_NO_OPTION_MAY_DO: &str = "nothing is uploaded";

/// The three things the decision says must happen before the task is ready,
/// each as a phrase it carries.
const WHAT_MUST_HAPPEN_FIRST: [&str; 3] = [
    "accepts, amends or rejects",
    "real files with their provenance",
    "walk and its table",
];

/// The words no sentence of this crate may carry while the decision waits:
/// each would be this machine naming one of the three formats.
/// The words no kind may carry while the decision is still proposed.
///
/// **The photograph is no longer among them, from 2026-09-19.** This list was
/// written when none of the three could be recognised, because no file of any
/// of them existed with provenance on any machine this team has. While that was
/// being written down, the Mac saved a real `.heic` from its own ImageIO and
/// took that third of the task, so a photograph is recognised and named.
///
/// **And the Pages document is no longer among them either, later the same
/// day.** The owner installed Pages on that Mac, a real document was saved from
/// it with this repository's own words and its own picture, and its provenance
/// and digest are in `tests/files/README.md`. The same sentence applies twice:
/// what this decision holds back is a format nobody here can produce a real file
/// of, and the moment somebody can, it stops being held back.
///
/// **One remains**, and it is the whole of what keeps the decision proposed: a
/// drawing. No machine this team has can write one.
const NOT_NAMED_YET: [&str; 2] = ["autocad", "drawing"];

/// The endings a name would claim one of the three by.
const NOT_CLAIMED_YET: [&str; 1] = ["a.dwg"];

/// A file of each of the three, as its own bytes begin.
const THE_THREE: [(&str, &[u8]); 3] = [
    (
        "a photograph",
        b"\0\0\0\x28ftypheic\0\0\0\0mif1MiHEMiPrmiafMiHBheic",
    ),
    ("a drawing", b"AC1032\0\0\x0b\0\0\0\0\0\0\0\0\0\0\0\0\0"),
    // A zip whose first bytes are a zip's, standing for a Pages document: what
    // would tell it apart is its list of contents, which is the rule that waits.
    ("a Pages document", b"PK\x03\x04\x14\0\0\0\0\0"),
];

/// Where the repository is, from this crate.
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

/// Every entry's name in a folder of this repository.
fn names_in(folder: &str) -> Vec<String> {
    let at = the_repository().join(folder);
    fs::read_dir(&at)
        .unwrap_or_else(|why| panic!("{} could not be listed: {why}", at.display()))
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect()
}

/// The decision's status, read off its status line the way `alo-citing` reads
/// one: the first line beginning with the marker, and the first known status in
/// it.
fn status_of(decision: &str) -> Result<String, String> {
    let line = decision
        .lines()
        .find(|line| line.starts_with(STATUS))
        .ok_or_else(|| format!("the decision has no line beginning `{STATUS}`"))?
        .to_lowercase();
    [
        "accepted",
        "proposed",
        "superseded",
        "rejected",
        "withdrawn",
    ]
    .into_iter()
    .find(|status| line.contains(status))
    .map(str::to_owned)
    .ok_or_else(|| format!("the status line says neither what it is nor what it was: `{line}`"))
}

/// Task 6's own section of the plan: from its heading to the next task's.
fn the_tasks_section(plan: &str) -> &str {
    let after = plan
        .split_once(THE_TASK)
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| panic!("the plan has no task headed `{THE_TASK}`"));
    after.split("\n### ").next().unwrap_or(after)
}

/// The decision's file name, without its folder.
fn the_decisions_name() -> &'static str {
    Path::new(THE_DECISION)
        .file_name()
        .and_then(|name| name.to_str())
        .expect("the decision's path has a file name")
}

/// **The decision exists, once, under its number, and still stands.**
#[test]
fn the_decision_exists_once_under_its_number() {
    let names = names_in(THE_DECISIONS);
    let carrying: Vec<&String> = names
        .iter()
        .filter(|name| name.starts_with(&format!("{THE_NUMBER}-")))
        .collect();
    match carrying.as_slice() {
        [one] => assert_eq!(
            one.as_str(),
            the_decisions_name(),
            "the decision numbered {THE_NUMBER} is `{one}`, not the file the plan names"
        ),
        [] => panic!("no decision under {THE_DECISIONS} carries the number {THE_NUMBER}"),
        many => panic!(
            "{} decisions carry the number {THE_NUMBER}: {many:?}",
            many.len()
        ),
    }

    let status = status_of(&reading(THE_DECISION)).expect("the decision has a readable status");
    assert!(
        STANDING.contains(&status.as_str()),
        "the decision's status is `{status}`, so task 6 waits on an answer that no longer stands"
    );
}

/// **The plan points at the decision from task 6, and while the decision is
/// proposed the plan steps over the task.**
///
/// The refusal this guards is the loop's: a task left *ready* beside an
/// unanswered decision sends the next worker at the same wall with no more
/// authority to climb it than the last one had.
#[test]
fn the_plan_points_at_the_decision_and_steps_over_the_task_while_it_waits() {
    let plan = reading(THE_PLAN);
    let section = the_tasks_section(&plan);
    assert!(
        section.contains(the_decisions_name()),
        "task 6 does not name the decision it waits on, so a worker lands on a task rather than \
         on the argument"
    );

    let status = status_of(&reading(THE_DECISION)).expect("the decision has a readable status");
    if status != WAITING {
        return;
    }
    let task_status = section
        .lines()
        .find(|line| line.starts_with(STATUS))
        .expect("task 6 has a status line");
    assert!(
        task_status.contains("blocked"),
        "task 6 says `{task_status}` while {THE_DECISION} is still proposed, so the loop would \
         offer it as work"
    );
    assert!(
        !section.lines().any(|line| line
            .strip_prefix(STATUS)
            .map_or(line, str::trim_start)
            .starts_with("**Done,")),
        "task 6 is marked done while the decision it waits on is still proposed"
    );
}

/// **It sets out what an owner answers by choosing**: three options, what each
/// costs, one recommendation, what no option may do, and what must happen
/// before the task is ready again.
#[test]
fn the_decision_sets_out_the_options_their_costs_and_a_recommendation() {
    let decision = reading(THE_DECISION);
    for heading in WHAT_IT_SETS_OUT {
        assert!(
            decision.contains(heading),
            "the decision is missing `{heading}`, so it is not yet something an owner can answer \
             by choosing"
        );
    }
    assert_eq!(
        decision.matches(WHAT_EACH_COSTS).count(),
        3,
        "each of the three options says what it costs, once — an option without its cost is a \
         preference"
    );
    assert!(
        decision.to_lowercase().contains(WHAT_NO_OPTION_MAY_DO),
        "the decision does not rule out uploading a file to find out what it is, which the plan \
         forbids whichever option is chosen"
    );
    for first in WHAT_MUST_HAPPEN_FIRST {
        assert!(
            decision.contains(first),
            "the decision does not say `{first}`, so the next worker cannot tell what the task is \
             still waiting on"
        );
    }
}

/// **While the decision says *proposed*, none of the three is recognised.**
///
/// The refusal path of an ADR. The day a kind is named for one of them, or a
/// name claims one, with this status line unchanged, the code has run ahead of
/// its owner, and this is the test that says so.
#[test]
fn none_of_the_three_is_recognised_while_the_decision_is_proposed() {
    let status = status_of(&reading(THE_DECISION)).expect("the decision has a readable status");
    if status != WAITING {
        return;
    }

    let strings = Strings::of(
        alo_saying::everything_this_machine_can_say()
            .expect("alo OS's own words are collected into one vocabulary"),
    );
    for kind in Kind::EVERY {
        let named = kind.said(&strings).text().to_lowercase();
        for not_yet in NOT_NAMED_YET {
            assert!(
                !named.contains(not_yet),
                "a kind is named `{named}` while {THE_DECISION} is still proposed"
            );
        }
    }
    for word in EVERY_WORD {
        let says = Word::says(&word).to_lowercase();
        for not_yet in NOT_NAMED_YET {
            assert!(
                !says.contains(not_yet),
                "`{}` says `{not_yet}` while {THE_DECISION} is still proposed",
                word.named()
            );
        }
    }
    for name in NOT_CLAIMED_YET {
        assert_eq!(
            Named::from_the_name(OsStr::new(name)),
            None,
            "`{name}` claims a kind while {THE_DECISION} is still proposed"
        );
    }

    // And what the machine actually answers about one of each: not a kind, and
    // not a film either.
    //
    // The photograph and the Pages document are skipped. Each was one of the
    // three when this was written, and each stopped being one the day a real
    // file of it arrived with its provenance and `alo-opening` learned to
    // recognise it — the `.heic` in the morning, the `.pages` the same evening,
    // once Pages was installed on the machine that could save one. What the
    // decision still holds back is the one nobody can yet produce a real file
    // of.
    for (what, bytes) in THE_THREE {
        if what.contains("photograph") || what.contains("Pages") {
            continue;
        }
        let mut file = Cursor::new(bytes.to_vec());
        let decided = decide(
            &mut file,
            OsStr::new("sent-to-me"),
            &ThisMachine::with_nothing(),
        )
        .expect("the file to be read");
        assert!(
            matches!(
                decided,
                Decided::AsItIs(Outcome::CannotOpen(
                    Cannot::Unrecognised | Cannot::Damaged(_)
                )) | Decided::AsItIs(Outcome::CannotOpen(Cannot::NothingHereOpens(
                    Kind::ZipArchive
                )))
            ),
            "{what} is answered as `{decided:?}` while {THE_DECISION} is still proposed"
        );
    }
}

/// And the check that would notice this file going quiet: everything it reads
/// by is still where it looks, so a rename that made the assertions above
/// vacuous fails here instead of passing.
#[test]
fn the_checks_would_notice_what_they_read_going_missing() {
    assert!(
        reading(THE_PLAN).contains(THE_TASK),
        "the task heading this file reads by is no longer in the plan"
    );
    assert!(
        names_in(THE_DECISIONS)
            .iter()
            .any(|name| name == the_decisions_name()),
        "the decision this file reads is not under {THE_DECISIONS}"
    );
    assert_eq!(
        status_of("# A decision\n\n**Status:** withdrawn, by nobody\n"),
        Ok("withdrawn".to_owned())
    );
    assert!(status_of("# A decision with no status\n").is_err());
}
