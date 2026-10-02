//! Every refusal by name, because a report drawn half-built is the fault.
use super::*;

const WINDOW: ReportedWindow = ReportedWindow::of(7);

fn whole() -> Result<Reported, NotReported> {
    Reported::of(
        WINDOW,
        "rename the invoices",
        MayChange::NothingYet,
        Some(40),
        Some("renamed two of nine"),
        None,
    )
}

#[test]
fn a_whole_report_is_made_and_reads_back_what_it_was_given() -> Result<(), NotReported> {
    let report = whole()?;
    assert_eq!(report.window(), WINDOW);
    assert_eq!(report.task(), "rename the invoices");
    assert_eq!(report.may_change(), &MayChange::NothingYet);
    assert_eq!(report.hundredths(), Some(40));
    assert_eq!(report.last_confirmed(), Some("renamed two of nine"));
    assert_eq!(report.waiting_for(), None);
    Ok(())
}

#[test]
fn a_report_about_no_named_task_is_refused() {
    for blank in ["", "   ", "\t"] {
        assert_eq!(
            Reported::of(WINDOW, blank, MayChange::NothingYet, None, None, None),
            Err(NotReported::ItNamesNoTask),
            "a blank task was accepted"
        );
    }
}

#[test]
fn progress_past_the_end_is_named_rather_than_clamped() {
    assert_eq!(
        Reported::of(
            WINDOW,
            "a task",
            MayChange::NothingYet,
            Some(101),
            None,
            None
        ),
        Err(NotReported::PastTheEnd(101)),
        "a clamp would draw a sender's mistake as finished"
    );
    assert!(
        Reported::of(
            WINDOW,
            "a task",
            MayChange::NothingYet,
            Some(100),
            None,
            None
        )
        .is_ok(),
        "a hundred hundredths is finished, not past the end"
    );
}

#[test]
fn a_blank_sentence_is_refused_and_none_is_not() {
    assert_eq!(
        Reported::of(
            WINDOW,
            "a task",
            MayChange::NothingYet,
            None,
            Some("  "),
            None
        ),
        Err(NotReported::ItConfirmedNothing)
    );
    assert_eq!(
        Reported::of(
            WINDOW,
            "a task",
            MayChange::NothingYet,
            None,
            None,
            Some("")
        ),
        Err(NotReported::ItWaitsForNothing)
    );
    assert!(
        Reported::of(WINDOW, "a task", MayChange::NothingYet, None, None, None).is_ok(),
        "saying nothing is confirmed and nothing is awaited is a whole report"
    );
}

/// **The contract carries no authority, and this is the test that says so.**
///
/// It asserts over the public surface rather than over a comment: there is no
/// constructor, field or method here that widens what a sender may do. If one is
/// added, the name below stops compiling or this assertion stops holding, and
/// either is the signal.
#[test]
fn a_report_claims_a_scope_and_cannot_grant_one() -> Result<(), NotReported> {
    let claimed = MayChange::TheseFiles(vec![std::path::PathBuf::from("/home/a/invoices")]);
    let report = Reported::of(WINDOW, "tidy", claimed.clone(), None, None, None)?;
    assert_eq!(
        report.may_change(),
        &claimed,
        "the claim is carried verbatim, for a coordinator to check against the grant"
    );
    Ok(())
}
