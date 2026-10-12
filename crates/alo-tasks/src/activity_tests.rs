//! That Details is shut, stays the person's once opened, and is never offered
//! over nothing.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::{Act, HowItWent, TheActivity};

/// **Details is shut when a task begins.**
///
/// *The logs support the conversation; they are not the whole interface.* A
/// window that opened on a log is a console, and the person who asked for their
/// invoices tidied did not ask to watch a machine work.
#[test]
fn details_are_shut_until_the_person_opens_them() {
    let mut activity = TheActivity::nothing_yet();
    assert!(!activity.details_are_open());

    activity.and_then(Act::doing("Reading Documents/invoices").and_the_details("stat(2) x14"));

    assert!(
        !activity.details_are_open(),
        "an act with a technical log behind it opened the log"
    );
}

/// **Once open they stay open**, including as new acts arrive.
///
/// Somebody who opened the log is watching something. A surface that shut it
/// under them each time a line arrived would be unusable at exactly the moment
/// it was wanted.
#[test]
fn details_stay_open_as_the_task_goes_on() {
    let mut activity = TheActivity::nothing_yet();
    activity.details_are_now(true);

    activity.and_then(Act::doing("Reading Documents/invoices"));
    activity.and_then(Act::done("Read 14 files").and_the_details("14 entries, 3 duplicates"));

    assert!(
        activity.details_are_open(),
        "a new act shut the log the person had opened"
    );
}

/// **An act with nothing behind it offers no Details**, so a person is never
/// given a control that opens on nothing.
#[test]
fn an_act_with_no_log_offers_no_details() {
    let plain = Act::done("Installed Blender");
    let with_log =
        Act::done("Installed Blender").and_the_details("flatpak install --noninteractive");

    assert!(!plain.has_details());
    assert_eq!(plain.details(), None);
    assert!(with_log.has_details());
    assert_eq!(
        with_log.details(),
        Some("flatpak install --noninteractive"),
        "the machine's own account is not behind Details"
    );
}

/// **What it is doing now is the last unfinished act**, which is what a
/// minimised task shows in the panel.
///
/// A person glancing at it from across the room wants *what is it doing*, not a
/// history — and the **last**, because an earlier act still marked as going
/// while a later one started would otherwise be what they were shown.
#[test]
fn what_it_is_doing_now_is_the_latest_unfinished_act() {
    let mut activity = TheActivity::nothing_yet();
    activity.and_then(Act::done("Read 14 files"));
    activity.and_then(Act::doing("Comparing duplicates"));
    activity.and_then(Act::doing("Hashing contents"));

    assert_eq!(
        activity.doing_now().map(Act::said),
        Some("Hashing contents"),
        "a minimised task would show an older act than the one it is on"
    );
}

/// **A task that has finished everything is doing nothing**, and says so
/// rather than showing the last thing it did as though it were still going.
#[test]
fn a_finished_task_is_doing_nothing() {
    let mut activity = TheActivity::nothing_yet();
    activity.and_then(Act::done("Read 14 files"));
    activity.and_then(Act::done("Deleted 3 duplicates"));

    assert_eq!(activity.doing_now(), None);
}

/// **An error is activity, in order, and findable.**
///
/// One list — *actions, progress, results and errors*. An error raised on some
/// other surface is one a person has to go and find, and the moment it happened
/// is the thing that explains it.
#[test]
fn an_error_is_activity_like_anything_else() {
    let mut activity = TheActivity::nothing_yet();
    activity.and_then(Act::done("Read 14 files"));
    activity.and_then(
        Act::went_wrong("Could not delete invoice-2019-03.pdf")
            .and_the_details("unlink: Permission denied (os error 13)"),
    );

    assert!(activity.anything_went_wrong());
    let last = activity.each().last().expect("the error is in the list");
    assert_eq!(last.went(), HowItWent::WentWrong);
    assert!(
        last.has_details(),
        "the one act whose technical account most matters has none"
    );
}

/// **A task where nothing went wrong says so.**
#[test]
fn a_task_that_went_well_says_nothing_went_wrong() {
    let mut activity = TheActivity::nothing_yet();
    activity.and_then(Act::done("Read 14 files"));

    assert!(!activity.anything_went_wrong());
}
