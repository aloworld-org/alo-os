//! That a task holds its six areas, that minimising shows three things, and
//! that a follow-up lands a waiting model change.

use alo_models::{InferenceSource, Region};

use super::ATask;
use crate::activity::Act;
use crate::changed::WhatChanged;
use crate::conversation::Turn;

/// A provider, as a person would name it.
fn a_provider() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "alo".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// A task somebody just asked for.
fn a_task() -> ATask {
    ATask::asked_for("Tidy up my invoices folder", InferenceSource::ThisMachine)
}

/// **A task begins with the request and nothing else.**
#[test]
fn a_task_begins_with_what_was_asked() {
    let task = a_task();

    assert_eq!(
        task.conversation().the_request().map(|turn| turn.said()),
        Some("Tidy up my invoices folder")
    );
    assert!(task.activity().each().is_empty());
    assert_eq!(task.model().answering(), &InferenceSource::ThisMachine);
    assert_eq!(task.permissions().asking(), None);
    assert!(!task.stopping().has_stopped());
}

/// **A follow-up lands the waiting model change.**
///
/// A person changes the model and then asks for something. If the follow-up did
/// not begin the next request, their choice would wait for ever in the one
/// window where they made it — and *applies to the next request* would be a
/// sentence rather than a behaviour.
#[test]
fn a_follow_up_begins_the_next_request_and_lands_the_change() {
    let mut task = a_task();
    task.model_mut().will_answer_next(a_provider());
    assert!(task.model().a_change_is_waiting(), "the premise");

    task.the_person_says("Actually, just list them");

    assert_eq!(
        task.model().answering(),
        &a_provider(),
        "the model a person chose did not take effect on the next request"
    );
    assert!(!task.model().a_change_is_waiting());
}

/// **And a change made with nothing following it does not land.**
///
/// The other half: a person who changes the model and says nothing has changed
/// nothing yet, and the request in flight keeps the model it began with.
#[test]
fn a_change_with_no_follow_up_does_not_take_effect() {
    let mut task = a_task();

    task.model_mut().will_answer_next(a_provider());

    assert_eq!(
        task.model().answering(),
        &InferenceSource::ThisMachine,
        "a model change landed without a request to land on"
    );
}

/// **Minimising shows the request, what it is doing, and whether it waits.**
///
/// Three things and no more. A preview carrying the conversation would be a
/// second window in a column six icons wide.
#[test]
fn the_panel_shows_what_it_is_doing_and_whether_it_waits() {
    let mut task = a_task();
    task.and_then(Act::done("Read 14 files"));
    task.and_then(Act::doing("Comparing duplicates"));

    let preview = task.in_the_panel();

    assert_eq!(preview.request, "Tidy up my invoices folder");
    assert_eq!(preview.doing.as_deref(), Some("Comparing duplicates"));
    assert!(!preview.waiting_for_you);
    assert!(!preview.something_went_wrong);
}

/// **A task waiting on the person says so from the panel.**
///
/// *Is this waiting on me* is what somebody asks of a minimised task from
/// across the room, and it is the one answer a preview must not get wrong.
#[test]
fn a_task_waiting_on_the_person_says_so_in_the_panel() {
    let mut task = a_task();
    task.alo_says(Turn::alo_proposing(
        "Delete 3 duplicate files in Documents/invoices",
    ));

    assert!(task.in_the_panel().waiting_for_you);

    task.the_person_says("Yes");

    assert!(
        !task.in_the_panel().waiting_for_you,
        "a task the person answered still says it is waiting on them"
    );
}

/// **Something going wrong reaches the panel too**, so a minimised task cannot
/// fail quietly in a corner.
#[test]
fn a_task_that_went_wrong_says_so_in_the_panel() {
    let mut task = a_task();
    task.and_then(Act::went_wrong("Could not delete invoice-2019-03.pdf"));

    assert!(task.in_the_panel().something_went_wrong);
}

/// **Stop asks, and the ending is a separate event that names what changed.**
#[test]
fn stopping_a_task_asks_and_then_reports() {
    let mut task = a_task();

    task.stop();
    assert!(task.stopping().is_waiting());
    assert!(
        !task.stopping().has_stopped(),
        "pressing Stop reported the work as over while it was running"
    );

    let mut changed = WhatChanged::nothing();
    changed.and_then("Deleted 1 duplicate file");
    task.it_stopped(changed);

    assert!(task.stopping().has_stopped());
    assert_eq!(
        task.stopping().what_changed().map(WhatChanged::how_many),
        Some(1),
        "a stopped task did not name what it had already changed"
    );
}
