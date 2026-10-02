//! Every refusal by name, and the two states a person must be able to tell apart.
use super::*;

const HANDLE: ReportedWindow = ReportedWindow::of(9);
const WINDOW: WindowId = WindowId::numbered(9);
const TASK: &str = "rename the invoices";

fn invoices() -> std::path::PathBuf {
    std::path::PathBuf::from("/home/a/invoices")
}

fn elsewhere() -> std::path::PathBuf {
    std::path::PathBuf::from("/home/a/.ssh")
}

fn reading_only() -> Reports {
    let mut reports = Reports::new();
    reports.handed_over(HANDLE, WINDOW, TASK, Granted::ReadingOnly);
    reports
}

fn report_of(
    task: &str,
    may_change: MayChange,
    waiting_for: Option<&str>,
) -> Result<Reported, alo_reported::NotReported> {
    Reported::of(
        HANDLE,
        task,
        may_change,
        Some(40),
        Some("renamed two"),
        waiting_for,
    )
}

#[test]
fn a_report_about_a_window_the_person_handed_over_is_admitted()
-> Result<(), Box<dyn std::error::Error>> {
    let reports = reading_only();
    let (window, showing) = reports.arrived(&report_of(TASK, MayChange::NothingYet, None)?)?;
    assert_eq!(window, WINDOW);
    let WhatAloIsDoing::AtWork(work) = &showing else {
        return Err("an admitted report must show as work".into());
    };
    assert_eq!(work.task(), TASK);
    Ok(())
}

/// **A report cannot create the association it needs.** This is the test that
/// says authority comes from the person: nothing was handed over, so a
/// well-formed report about a real-looking window is refused.
#[test]
fn a_report_about_a_window_nobody_handed_over_is_refused() -> Result<(), alo_reported::NotReported>
{
    let reports = Reports::new();
    assert_eq!(
        reports.arrived(&report_of(TASK, MayChange::NothingYet, None)?),
        Err(NotAdmitted::NobodyHandedThatOver(9)),
        "a report conjured its own association"
    );
    Ok(())
}

/// A second sender cannot take a window over by reporting against it, and a
/// stale one cannot overwrite the task that replaced its own.
#[test]
fn a_report_naming_another_task_is_refused() -> Result<(), alo_reported::NotReported> {
    let reports = reading_only();
    assert_eq!(
        reports.arrived(&report_of("read my mail", MayChange::NothingYet, None)?),
        Err(NotAdmitted::ADifferentTask)
    );
    Ok(())
}

#[test]
fn a_report_claiming_more_than_was_granted_is_refused_and_not_narrowed()
-> Result<(), alo_reported::NotReported> {
    let reports = reading_only();
    assert_eq!(
        reports.arrived(&report_of(
            TASK,
            MayChange::TheseFiles(vec![invoices()]),
            None
        )?),
        Err(NotAdmitted::MoreThanWasGranted),
        "a claim beyond a reading-only grant was allowed"
    );

    let mut narrower = Reports::new();
    narrower.handed_over(HANDLE, WINDOW, TASK, Granted::TheseFiles(vec![invoices()]));
    assert_eq!(
        narrower.arrived(&report_of(
            TASK,
            MayChange::TheseFiles(vec![invoices(), elsewhere()]),
            None
        )?),
        Err(NotAdmitted::MoreThanWasGranted),
        "a claim naming one granted path and one other was allowed"
    );
    assert!(
        narrower
            .arrived(&report_of(
                TASK,
                MayChange::TheseFiles(vec![invoices()]),
                None
            )?)
            .is_ok(),
        "a claim inside the grant was refused"
    );
    Ok(())
}

#[test]
fn reading_is_within_every_grant() -> Result<(), alo_reported::NotReported> {
    for granted in [Granted::ReadingOnly, Granted::TheseFiles(vec![invoices()])] {
        let mut reports = Reports::new();
        reports.handed_over(HANDLE, WINDOW, TASK, granted.clone());
        assert!(
            reports
                .arrived(&report_of(TASK, MayChange::NothingYet, None)?)
                .is_ok(),
            "reading only was refused against {granted:?}"
        );
    }
    Ok(())
}

#[test]
fn a_report_that_has_stopped_carries_what_it_is_waiting_for()
-> Result<(), Box<dyn std::error::Error>> {
    let reports = reading_only();
    let (_, showing) = reports.arrived(&report_of(
        TASK,
        MayChange::NothingYet,
        Some("two names collide — which should it keep?"),
    )?)?;
    let WhatAloIsDoing::AtWork(work) = &showing else {
        return Err("an admitted report must show as work".into());
    };
    assert!(
        work.requires_you().is_waiting(),
        "a stopped report must require the person"
    );
    Ok(())
}

/// **The owner's ruling, as a test: a lost connection becomes status
/// unavailable and never remains working.**
///
/// It asserts the state is neither of the two it could wrongly be. `AtWork`
/// would leave the screen claiming the task is running; `Nothing` would erase
/// the agent section and read as the task having finished.
#[test]
fn a_lost_reporter_becomes_status_unavailable_and_not_working_or_nothing()
-> Result<(), Box<dyn std::error::Error>> {
    let reports = reading_only();
    let Some((window, showing)) = reports.the_reporter_is_gone(WINDOW) else {
        return Err("a window handed to alo must have a status to lose".into());
    };
    assert_eq!(window, WINDOW);
    assert_eq!(showing, WhatAloIsDoing::StatusUnavailable);
    assert_ne!(
        showing,
        WhatAloIsDoing::Nothing,
        "a dropped connection must not erase the agent section"
    );
    Ok(())
}

/// A window nobody handed over has no claim on screen to go stale.
#[test]
fn a_lost_reporter_for_a_window_that_is_not_alos_says_nothing() {
    assert!(Reports::new().the_reporter_is_gone(WINDOW).is_none());
}

/// **Ending and being lost are different, and a person must be able to tell.**
#[test]
fn a_window_taken_back_shows_nothing_rather_than_unavailable() {
    let mut reports = reading_only();
    assert_eq!(
        reports.taken_back(WINDOW),
        Some((WINDOW, WhatAloIsDoing::Nothing)),
        "work that ended is not work whose status is unknown"
    );
    assert!(
        reports.taken_back(WINDOW).is_none(),
        "a window already taken back is not alo's to take back twice"
    );
    assert!(
        reports.the_reporter_is_gone(WINDOW).is_none(),
        "a window taken back has no status left to lose"
    );
}

/// Handing the same window over again is a person changing their mind, and it
/// replaces the terms rather than adding a second set.
#[test]
fn handing_the_same_window_over_again_replaces_the_terms() -> Result<(), alo_reported::NotReported>
{
    let mut reports = reading_only();
    reports.handed_over(HANDLE, WINDOW, "something else", Granted::ReadingOnly);
    assert_eq!(
        reports.arrived(&report_of(TASK, MayChange::NothingYet, None)?),
        Err(NotAdmitted::ADifferentTask),
        "the replaced task still admitted its old report"
    );
    assert!(
        reports
            .arrived(&report_of("something else", MayChange::NothingYet, None)?)
            .is_ok()
    );
    Ok(())
}
