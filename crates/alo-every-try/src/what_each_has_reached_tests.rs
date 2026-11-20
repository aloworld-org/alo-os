//! The record of what each request has reached: read and written as one
//! file, raised only by a measured run, and never lowered.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use super::{AS_COMMITTED, Measured, Reached, the_level_named, the_name_of};
use crate::a_road::ARequest;
use crate::a_try::{TheLevel, Tried, Went};
use crate::tallied::Tallied;
use crate::the_bar::THE_BAR;
use crate::went_wrong::WentWrong;

fn on_the_road() -> Tried {
    Tried::of(vec!["open_application".to_owned()], Went::TheRoad, 2)
}

fn off_the_road() -> Tried {
    Tried::of(
        vec!["run: firefox".to_owned()],
        Went::WentWrong(WentWrong::RanACommand {
            step: 1,
            expected: "open_application".to_owned(),
            command: "firefox".to_owned(),
        }),
        1,
    )
}

fn tried(named: &'static str, passes: usize, slips: usize) -> Tallied {
    Tallied::of(
        ARequest::asking(named, "Open Firefox.", Vec::new()),
        std::iter::repeat_with(on_the_road)
            .take(passes)
            .chain(std::iter::repeat_with(off_the_road).take(slips))
            .collect(),
    )
}

#[test]
fn the_levels_are_named_as_the_workflow_names_them() {
    for level in [TheLevel::FullAccess, TheLevel::AskEachTime] {
        assert_eq!(the_level_named(the_name_of(level)), Some(level));
    }
    assert_eq!(the_name_of(TheLevel::FullAccess), "full-access");
    assert_eq!(the_name_of(TheLevel::AskEachTime), "ask-each-time");
    assert_eq!(the_level_named("somewhere else"), None);
}

#[test]
fn the_committed_record_reads() {
    Reached::read(AS_COMMITTED).expect("the committed record reads");
}

#[test]
fn a_record_reads_back_as_itself() {
    let record = Reached::read("reached\tfull-access\topen it\nreached\task-each-time\tmove it\n")
        .expect("it reads");
    assert!(record.has(TheLevel::FullAccess, "open it"));
    assert!(!record.has(TheLevel::AskEachTime, "open it"));
    assert!(record.has(TheLevel::AskEachTime, "move it"));
    assert_eq!(Reached::read(&record.written()).expect("it reads"), record);
}

/// **The refusal cases**: a line of no kind, a level that is not one, and an
/// entry written twice.
#[test]
fn a_record_that_does_not_read_is_refused() {
    assert!(Reached::read("nonsense\n").is_err());
    assert!(Reached::read("reached\tsomewhere\topen it\n").is_err());
    assert!(
        Reached::read("reached\tfull-access\topen it\nreached\tfull-access\topen it\n").is_err()
    );
}

/// **A measured run raises an entry** when the request took the road on every
/// one of at least the bar's tries at that level, and on nothing less.
#[test]
fn a_measured_run_raises_only_what_took_every_try() {
    let all = THE_BAR.tries();
    let measured = Measured::of(
        TheLevel::FullAccess,
        &[
            tried("every time", all, 0),
            tried("once wrong", all - 1, 1),
            tried("too few to say", 3, 0),
        ],
    );
    let raised = Reached::default().raised_by(&measured);
    assert!(raised.has(TheLevel::FullAccess, "every time"));
    assert!(
        !raised.has(TheLevel::AskEachTime, "every time"),
        "at that level only"
    );
    assert!(!raised.has(TheLevel::FullAccess, "once wrong"));
    assert!(
        !raised.has(TheLevel::FullAccess, "too few to say"),
        "three of three is not a measurement at the bar"
    );
}

/// A measured run never lowers an entry: a slip after reaching it is the
/// check's to fail, not the record's to forget.
#[test]
fn a_measured_run_never_lowers_an_entry() {
    let record = Reached::read("reached\tfull-access\tonce wrong\n").expect("it reads");
    let measured = Measured::of(
        TheLevel::FullAccess,
        &[tried("once wrong", THE_BAR.tries() - 1, 1)],
    );
    assert!(
        record
            .raised_by(&measured)
            .has(TheLevel::FullAccess, "once wrong")
    );
}

#[test]
fn what_was_measured_reads_back_as_itself() {
    let measured = Measured::of(TheLevel::AskEachTime, &[tried("open it", 9, 1)]);
    assert_eq!(
        Measured::read(&measured.written()).expect("it reads"),
        measured
    );
    assert!(Measured::read("measured\task-each-time\topen it\tnine\t10\n").is_err());
}

/// **An entry may never be lowered**: one in the base and not in the change is
/// named, unless its request is gone from the set; one only in the change is a
/// raise and fine.
#[test]
fn an_entry_taken_out_is_lowering_unless_its_request_is_gone() {
    let base = Reached::read("reached\tfull-access\topen it\nreached\tfull-access\tgone\n")
        .expect("it reads");
    let raised = Reached::read(
        "reached\tfull-access\topen it\nreached\tfull-access\tgone\nreached\task-each-time\topen it\n",
    )
    .expect("it reads");
    let in_the_set = |named: &str| named != "gone";
    assert!(raised.lowered_from(&base, &in_the_set).is_empty());
    let lowered = Reached::default().lowered_from(&base, &in_the_set);
    assert_eq!(lowered, [("full-access".to_owned(), "open it".to_owned())]);
}
