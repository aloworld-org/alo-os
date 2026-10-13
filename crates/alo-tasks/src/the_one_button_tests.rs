//! That the composer's one button offers exactly one action at a time, that it
//! is the action the design draws, and that Enter can never reach Stop.
//!
//! The mapping these check is not invented here. It is read off the seven
//! variants of the component set `417:28422` and the interaction contract
//! `418:28202`, and written down in `docs/design/the-ask-alo-task-window.md`.

use alo_models::InferenceSource;

use super::TheOneButton;
use crate::ATask;
use crate::activity::Act;
use crate::changed::WhatChanged;
use crate::conversation::Turn;

/// A task somebody just asked for and nothing has answered.
fn just_asked() -> ATask {
    ATask::asked_for(
        "Prepare the launch presentation",
        InferenceSource::ThisMachine,
    )
}

/// A task alo has replied to and is now working in — the design's *Working*.
fn working() -> ATask {
    let mut task = just_asked();
    task.alo_says(Turn::from_alo("I'll read the two files."));
    task.and_then(Act::doing("alo is reading Product notes.md"));
    task
}

/// A task that has finished — the design's *Complete*.
fn complete() -> ATask {
    let mut task = just_asked();
    task.and_then(Act::done("Launch brief.pdf read"));
    task.alo_says(Turn::from_alo("Your presentation is ready."));
    task
}

/// **A request shows Stop before anything has answered it.**
///
/// The seconds between Enter and alo's first word are exactly when a person
/// needs to know the machine heard them and that they can call it back. A
/// reader that decided *working* from the activity alone would call this task
/// idle and offer Send.
#[test]
fn a_task_nothing_has_answered_yet_offers_stop() {
    let task = just_asked();

    assert!(
        task.activity().doing_now().is_none(),
        "nothing is doing yet"
    );
    assert_eq!(task.the_one_button(), TheOneButton::Stop);
}

/// **Working is Stop.** The design's *Working*, *Model picker*, *Permissions*
/// and *Dark* variants all draw Stop, and all four are a task alo is in.
#[test]
fn a_working_task_offers_stop() {
    assert_eq!(working().the_one_button(), TheOneButton::Stop);
}

/// **A task waiting for an approval offers Stop, not Send.**
///
/// The design's *Needs permission* variant draws Stop, which settles what the
/// written requirement left open: waiting for an answer is still being in the
/// middle of the work.
#[test]
fn a_task_waiting_for_permission_offers_stop() {
    let mut task = working();
    task.alo_says(Turn::alo_proposing(
        "The draft is ready. May I save it in your Presentations folder?",
    ));

    assert!(task.conversation().is_waiting_for_the_person());
    assert_eq!(task.the_one_button(), TheOneButton::Stop);
}

/// **Stopping is a greyed Stop, and never a Send.**
///
/// The state a boolean cannot hold. A window that went straight to Send would
/// say the work had ended at the moment somebody asked it to.
#[test]
fn a_stop_that_has_been_asked_for_cannot_be_asked_for_again() {
    let mut task = working();
    task.stop();

    assert_eq!(task.the_one_button(), TheOneButton::Stopping);
    assert!(
        task.the_one_button().is_stop(),
        "it is still the stop action"
    );
    assert!(
        !task.the_one_button().can_be_pressed(),
        "repeated cancellation is refused"
    );
}

/// **Stopping outranks working.** While stopping, the work is still running —
/// so every later test in the rule would still say Stop, and would say it as a
/// pressable button. The order of the tests is the rule.
#[test]
fn stopping_wins_over_a_task_that_is_still_working() {
    let mut task = working();
    task.stop();

    assert!(
        task.activity().doing_now().is_some(),
        "the work has not ended, which is the point"
    );
    assert_eq!(task.the_one_button(), TheOneButton::Stopping);
}

/// **Confirmation returns it to Send, and the draft is still there.**
#[test]
fn a_confirmed_stop_returns_to_send_with_the_draft_kept() {
    let mut task = working();
    task.the_person_is_typing("Actually, make it four slides");
    task.stop();
    assert_eq!(task.the_one_button(), TheOneButton::Stopping);

    task.it_stopped(WhatChanged::nothing());

    assert_eq!(task.the_one_button(), TheOneButton::Send);
    assert_eq!(
        task.the_draft(),
        "Actually, make it four slides",
        "a stop must not eat what somebody typed"
    );
}

/// **A finished task with an empty box offers a greyed Send.**
#[test]
fn a_finished_task_with_nothing_typed_cannot_send() {
    let task = complete();

    assert_eq!(task.the_one_button(), TheOneButton::NothingToSend);
    assert!(!task.the_one_button().can_be_pressed());
    assert!(!task.the_one_button().is_stop(), "it is the send action");
}

/// **Typing turns the greyed Send on, and does nothing else.**
#[test]
fn typing_enables_send_and_starts_nothing() {
    let mut task = complete();
    let before = task.activity().each().len();

    task.the_person_is_typing("Add a slide about pricing");

    assert_eq!(task.the_one_button(), TheOneButton::Send);
    assert_eq!(
        task.activity().each().len(),
        before,
        "typing neither submits nor interrupts"
    );
    assert!(matches!(task.stopping(), crate::Stopping::NobodyAsked));
}

/// **Deleting what was typed greys Send again.**
#[test]
fn emptying_the_box_greys_send_again() {
    let mut task = complete();
    task.the_person_is_typing("Add a slide");
    assert_eq!(task.the_one_button(), TheOneButton::Send);

    task.the_person_is_typing("");

    assert_eq!(task.the_one_button(), TheOneButton::NothingToSend);
}

/// **Enter never activates Stop.**
///
/// The refusal that matters most. A person replying to a working task presses
/// Enter while the button beside them says Stop, and Enter must send the reply
/// rather than cancel their work.
#[test]
fn enter_sends_a_reply_while_the_button_says_stop() {
    let mut task = working();
    task.the_person_is_typing("Keep it to four slides");

    assert_eq!(
        task.the_one_button(),
        TheOneButton::Stop,
        "the button is Stop, which is what makes this the dangerous case"
    );
    assert!(
        task.enter_sends_the_draft(),
        "Enter sends the reply and does not cancel the task"
    );
}

/// **Enter does nothing with an empty box**, in every state the button can be
/// in — so there is no state in which Enter falls through to the button.
#[test]
fn enter_does_nothing_with_an_empty_box_in_any_state() {
    let mut stopping = working();
    stopping.stop();
    let mut stopped = working();
    stopped.it_stopped(WhatChanged::nothing());

    let every = [just_asked(), working(), complete(), stopping, stopped];
    let mut seen = Vec::new();
    for task in &every {
        assert!(
            !task.enter_sends_the_draft(),
            "an empty box sends nothing, whatever the button says"
        );
        seen.push(task.the_one_button());
    }

    assert!(
        seen.contains(&TheOneButton::Stop)
            && seen.contains(&TheOneButton::Stopping)
            && seen.contains(&TheOneButton::NothingToSend),
        "the five fixtures must cover more than one button state, or this \
         proves nothing: {seen:?}"
    );
}

/// **The button is never both and never neither.** Every state is exactly one
/// of the four, and the two questions about it agree: a pressable button is
/// Send or Stop, and `is_stop` partitions the four.
#[test]
fn every_state_is_exactly_one_action() {
    let mut stopping = working();
    stopping.stop();
    let mut stopped = working();
    stopped.it_stopped(WhatChanged::nothing());
    let mut replied = complete();
    replied.the_person_is_typing("More");

    for task in [
        just_asked(),
        working(),
        complete(),
        stopping,
        stopped,
        replied,
    ] {
        let button = task.the_one_button();
        assert_eq!(
            button.is_stop(),
            matches!(button, TheOneButton::Stop | TheOneButton::Stopping),
            "{button:?}"
        );
        assert_eq!(
            button.can_be_pressed(),
            matches!(button, TheOneButton::Send | TheOneButton::Stop),
            "{button:?}"
        );
    }
}

/// **Sending takes the draft out of the box.** Clearing the box and adding the
/// turn are two acts, and a caller doing only one of them loses what somebody
/// typed or sends it twice.
#[test]
fn sending_empties_the_box_and_keeps_the_words() {
    let mut task = complete();
    task.the_person_is_typing("Add a slide about pricing");

    let sent = task.take_the_draft();
    task.the_person_says(sent.clone());

    assert_eq!(sent, "Add a slide about pricing");
    assert!(
        task.the_draft().is_empty(),
        "the box is empty after sending"
    );
    assert_eq!(
        task.conversation().each().last().map(Turn::said),
        Some("Add a slide about pricing")
    );
    assert_eq!(
        task.the_one_button(),
        TheOneButton::Stop,
        "and the task is alo's again, so the button is Stop"
    );
}

/// **A confirmed stop outranks an activity log that still reads *doing*.**
///
/// These two really can disagree: `it_stopped` is the runtime saying the work
/// has ended, while the activity is a record of what happened and may still
/// hold an act nobody closed out. This asserts the disagreement resolves the
/// runtime's way — and asserts the disagreement is *present*, so the test
/// cannot pass by the log happening to be tidy.
#[test]
fn a_confirmed_stop_outranks_an_activity_that_still_says_doing() {
    let mut task = working();
    task.it_stopped(WhatChanged::nothing());

    assert!(
        task.activity().doing_now().is_some(),
        "the fixture must still hold an unfinished act, or this proves nothing"
    );
    assert_eq!(
        task.the_one_button(),
        TheOneButton::NothingToSend,
        "nothing is running, so a person may reply rather than cancel again"
    );
    assert!(!task.the_one_button().is_stop());
}
