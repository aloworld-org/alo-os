//! That a task holds its six areas, that minimising shows three things, and
//! that a follow-up lands a waiting model change.

use alo_models::{InferenceSource, Region};

use super::ATask;
use crate::activity::Act;
use crate::changed::WhatChanged;
use crate::conversation::Turn;
use crate::permissions::Answered;

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

/// **A stop asked for and not confirmed says *stopping*, never *stopped*.**
///
/// The one thing a Stop must not claim. The work is still running at this
/// moment, and every later branch of the rule would have said something else.
#[test]
fn a_stop_not_yet_confirmed_says_stopping() {
    let mut task = a_task();
    task.and_then(Act::doing("reading your documents"));
    task.stop();

    assert_eq!(task.what_is_happening(), crate::words::STOPPING);
    assert_ne!(task.what_is_happening(), crate::words::STOPPED);
}

/// **A confirmed stop says *stopped*, whatever the activity still says.**
///
/// The runtime's word outranks a log nobody closed out — the same rule
/// `the_one_button` follows, and asserted with the disagreement present so it
/// cannot pass by the log happening to be tidy.
#[test]
fn a_confirmed_stop_says_stopped_even_with_an_act_still_open() {
    let mut task = a_task();
    task.and_then(Act::doing("reading your documents"));
    task.it_stopped(WhatChanged::nothing());

    assert!(
        task.activity().doing_now().is_some(),
        "the fixture must still hold an open act, or this proves nothing"
    );
    assert_eq!(task.what_is_happening(), crate::words::STOPPED);
}

/// **Waiting on the person is its own word, and it is not the working one.**
#[test]
fn waiting_on_the_person_is_not_the_working_word() {
    let mut task = a_task();
    task.alo_says(Turn::alo_proposing(
        "May I save it in your Presentations folder?",
    ));

    assert_eq!(
        task.what_is_happening(),
        crate::words::WAITING_FOR_YOUR_PERMISSION
    );
    assert_ne!(task.what_is_happening(), crate::words::ALO_IS_WORKING);
}

/// **A request nothing has answered yet is already working.**
///
/// The seconds between Enter and alo's first word, where a person most needs to
/// see that the machine heard them.
#[test]
fn a_request_nothing_has_answered_is_already_working() {
    assert_eq!(a_task().what_is_happening(), crate::words::ALO_IS_WORKING);
}

/// **A task that finished says so.**
#[test]
fn a_finished_task_says_complete() {
    let mut task = a_task();
    task.and_then(Act::done("Launch brief.pdf read"));
    task.alo_says(Turn::from_alo("Your presentation is ready."));

    assert_eq!(task.what_is_happening(), crate::words::COMPLETE);
}

/// **A task carries the question and hands back the answer**, and the sentence
/// it shows is the one it was given, unchanged.
#[test]
fn a_task_can_be_asked_and_answers_once() {
    let mut task = a_task();
    assert_eq!(task.permissions().asking(), None);

    let sentence = "Rename march.pdf to invoice-march.pdf in /home/anna/Invoices";
    task.is_asking(sentence);
    assert_eq!(task.permissions().asking(), Some(sentence));

    assert_eq!(task.allow_once(), Answered::AllowedOnce);
    assert_eq!(
        task.permissions().asking(),
        None,
        "the question stayed up after it was answered"
    );
}

/// **A second press is not a refusal the person made.** A stale window and a
/// double press both arrive here, and answering them *Denied* would record a
/// no that nobody said.
#[test]
fn answering_twice_says_nothing_was_asked_rather_than_denied() {
    let mut task = a_task();
    task.is_asking("Rename march.pdf");
    assert_eq!(task.allow_once(), Answered::AllowedOnce);

    assert_eq!(task.allow_once(), Answered::NothingWasAsked);
    assert_eq!(task.deny(), Answered::NothingWasAsked);
}

/// Saying no is its own answer and leaves nothing behind.
#[test]
fn denying_answers_the_question_and_keeps_nothing() {
    let mut task = a_task();
    task.is_asking("Delete everything in /home/anna");
    assert_eq!(task.deny(), Answered::Denied);
    assert_eq!(task.permissions().asking(), None);
}

/// **A held reply stays out of the conversation**, a second one joins it, and
/// taking it leaves nothing held.
#[test]
fn a_held_reply_is_kept_apart_until_it_is_taken() {
    let mut task = a_task();
    let before = task.conversation().each().len();
    task.hold("and the April one");
    task.hold("and May");
    assert_eq!(task.held(), Some("and the April one\nand May"));
    assert_eq!(task.conversation().each().len(), before);
    assert_eq!(
        task.take_held().as_deref(),
        Some("and the April one\nand May")
    );
    assert_eq!(task.held(), None);
}

/// **The task remembers which sentence was answered and how**, so a follow-up
/// can say what became of each change — and a press on nothing is not kept.
#[test]
fn each_answer_is_kept_with_the_sentence_it_answered() {
    let mut task = a_task();
    task.is_asking("Move march.pdf to Paid");
    assert_eq!(task.allow_once(), Answered::AllowedOnce);
    assert_eq!(task.deny(), Answered::NothingWasAsked);
    task.is_asking("Delete april.pdf");
    assert_eq!(task.deny(), Answered::Denied);
    assert_eq!(
        task.answers_given().each(),
        [
            ("Move march.pdf to Paid".to_owned(), Answered::AllowedOnce),
            ("Delete april.pdf".to_owned(), Answered::Denied),
        ]
    );
}

/// **Allowing once widens nothing**, which is why `ThePermissions` can answer
/// a question without any way to grant.
///
/// The failure this refuses is the kind-sounding one — *they allowed it once,
/// so stop asking* — and it is how an approval becomes a session. So the next
/// question is asked exactly as the first was, and the standing grants the
/// task holds are no more numerous after a yes than before it.
#[test]
fn allowing_once_does_not_widen_what_the_task_holds() {
    let mut task = a_task();
    let held_before = task.permissions().how_many_grants();

    task.is_asking("Rename march.pdf");
    assert_eq!(task.allow_once(), Answered::AllowedOnce);
    assert_eq!(
        task.permissions().how_many_grants(),
        held_before,
        "saying yes once granted something standing"
    );

    // Asked again, it asks again: a yes did not become a session.
    task.is_asking("Rename april.pdf");
    assert_eq!(task.permissions().asking(), Some("Rename april.pdf"));
    assert_eq!(task.deny(), Answered::Denied);
    assert_eq!(task.permissions().how_many_grants(), held_before);
}

/// **A change still asked about keeps the task waiting**, whatever was said
/// after it. A person may write beneath a card; the card is still there.
#[test]
fn a_change_still_asked_about_keeps_the_task_waiting() {
    let mut task = a_task();
    task.is_asking("Rename march.pdf");
    task.alo_says(Turn::alo_proposing("Rename march.pdf"));
    task.the_person_says("which one?");
    task.alo_says(Turn::from_alo("That reply did not reach alo."));

    assert!(task.is_waiting_for_the_person());
    assert_eq!(
        task.what_is_happening(),
        crate::words::WAITING_FOR_YOUR_PERMISSION
    );
    assert!(task.in_the_panel().waiting_for_you);

    let _ = task.allow_once();
    assert!(
        !task.is_waiting_for_the_person(),
        "an answered change still kept the task waiting"
    );
}

/// **Only a task whose work is over may close without asking**: finished, or
/// confirmed stopped.
#[test]
fn a_finished_or_stopped_task_may_close_without_asking() {
    let mut finished = a_task();
    finished.alo_says(Turn::from_alo("Your presentation is ready."));
    assert!(finished.may_close_without_asking());

    let mut stopped = a_task();
    stopped.and_then(Act::doing("reading your documents"));
    stopped.it_stopped(WhatChanged::nothing());
    assert!(stopped.may_close_without_asking());
}

/// **A task still running may not**: working, stopping, or waiting on the
/// person — a change to approve or a question. Each is the refusal the close
/// promise turns on, so each is held separately.
#[test]
fn a_running_task_may_not_close_without_asking() {
    let working = a_task();
    assert!(!working.may_close_without_asking(), "working");

    let mut stopping = a_task();
    stopping.and_then(Act::doing("reading your documents"));
    stopping.stop();
    assert!(!stopping.may_close_without_asking(), "asked to stop");

    let mut a_change = a_task();
    a_change.alo_says(Turn::from_alo("Here is the plan."));
    a_change.is_asking("Rename march.pdf to invoice-march.pdf");
    assert!(!a_change.may_close_without_asking(), "a change waiting");

    let mut a_question = a_task();
    a_question.alo_says(Turn::alo_asking("Which folder?"));
    assert!(!a_question.may_close_without_asking(), "a question waiting");
}
