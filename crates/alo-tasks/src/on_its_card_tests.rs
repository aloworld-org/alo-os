//! The six states a task's card shows, each with the lines and button the
//! approved design draws for it — and the refusals: no Stop offered once the
//! work is over, and never *stopped* for a Stop nothing confirmed.

use alo_models::InferenceSource;

use super::{HowFar, HowItStands, LastApproved, TheCardsButton};
use crate::ATask;
use crate::activity::Act;
use crate::changed::WhatChanged;
use crate::conversation::Turn;

fn a_task() -> ATask {
    ATask::asked_for("Summarise the Q3 report", InferenceSource::ThisMachine)
}

/// **Working**: what alo is doing, the last thing approved, and Stop.
#[test]
fn a_working_task_offers_stop_and_says_what_it_is_doing() {
    let mut task = a_task();
    task.and_then(Act::doing("Reading report.pdf"));

    let card = task.on_its_card();

    assert_eq!(card.stands, HowItStands::Working);
    assert_eq!(card.task, "Summarise the Q3 report");
    assert_eq!(card.how_far, HowFar::Doing("Reading report.pdf".to_owned()));
    assert_eq!(card.last_approved, LastApproved::Nothing);
    assert_eq!(card.button, TheCardsButton::Stop);
    assert!(card.stands.may_be_stopped());
}

/// **A request nothing has answered yet is already working**, and says nothing
/// it does not know about how far it has got.
#[test]
fn a_task_nothing_has_answered_is_working_with_nothing_to_say_yet() {
    let card = a_task().on_its_card();

    assert_eq!(card.stands, HowItStands::Working);
    assert_eq!(card.how_far, HowFar::NothingToSayYet);
    assert_eq!(card.button, TheCardsButton::Stop);
}

/// **Needs your OK**: what alo wants, and Review — never Stop's navy fill,
/// because alo is not acting.
#[test]
fn a_task_waiting_for_an_ok_offers_review_and_says_what_it_wants() {
    let mut task = a_task();
    task.is_asking("Move 14 files to Archive");

    let card = task.on_its_card();

    assert_eq!(card.stands, HowItStands::NeedsYourOk);
    assert_eq!(
        card.how_far,
        HowFar::Wants("Move 14 files to Archive".to_owned())
    );
    assert_eq!(card.button, TheCardsButton::Review);
    assert!(
        card.stands.may_be_stopped(),
        "a task waiting on the person is still running"
    );
}

/// **The last approved change is the last one the person said yes to.**
#[test]
fn the_fourth_line_is_the_last_change_the_person_approved() {
    let mut task = a_task();
    task.is_asking("Open report.pdf");
    task.allow_once();
    task.is_asking("Open notes.txt");
    task.deny();
    task.and_then(Act::doing("Reading report.pdf"));

    assert_eq!(
        task.on_its_card().last_approved,
        LastApproved::This("Open report.pdf".to_owned()),
        "a denied change was shown as approved"
    );
}

/// **Stopping…**: no button, and the card says alo has not confirmed.
#[test]
fn a_stop_on_its_way_is_stopping_with_no_button() {
    let mut task = a_task();
    task.and_then(Act::doing("Reading report.pdf"));
    task.stop();

    let card = task.on_its_card();

    assert_eq!(card.stands, HowItStands::Stopping);
    assert_eq!(card.how_far, HowFar::WaitingForAloToConfirm);
    assert_eq!(card.button, TheCardsButton::None);
    assert!(
        !card.stands.may_be_stopped(),
        "Stop offered twice for one Stop"
    );
}

/// **Stopped · changes kept**: how many, where to undo them, and Open.
#[test]
fn a_confirmed_stop_counts_what_it_kept_and_points_at_history() {
    let mut task = a_task();
    task.stop();
    let mut changed = WhatChanged::nothing();
    changed.and_then("Moved a.pdf");
    changed.and_then("Moved b.pdf");
    task.it_stopped(changed);

    let card = task.on_its_card();

    assert_eq!(card.stands, HowItStands::Stopped { changes_kept: 2 });
    assert_eq!(card.how_far, HowFar::ChangesKept(2));
    assert_eq!(card.last_approved, LastApproved::UndoIsInHistory);
    assert_eq!(card.button, TheCardsButton::Open);
    assert!(!card.stands.may_be_stopped());
}

/// **A Stop that kept nothing does not point at History**, because there is
/// nothing there to undo.
#[test]
fn a_confirmed_stop_that_kept_nothing_says_so() {
    let mut task = a_task();
    task.stop();
    task.it_stopped(WhatChanged::nothing());

    let card = task.on_its_card();

    assert_eq!(card.stands, HowItStands::Stopped { changes_kept: 0 });
    assert_eq!(card.last_approved, LastApproved::Nothing);
}

/// **Stop requested · not confirmed — never *stopped*.** The refusal that
/// matters most on this card: a Stop this machine could not deliver may have
/// stopped nothing.
#[test]
fn a_stop_that_could_not_be_delivered_is_never_shown_as_stopped() {
    let mut task = a_task();
    task.and_then(Act::doing("Renaming IMG_0012"));
    task.stop();
    task.its_stop_could_not_be_confirmed();

    let card = task.on_its_card();

    assert_eq!(card.stands, HowItStands::StopNotConfirmed);
    assert_eq!(card.how_far, HowFar::CannotConfirmItStopped);
    assert_eq!(card.button, TheCardsButton::Open);
    assert!(
        !task.may_close_without_asking(),
        "a task whose Stop was not confirmed may still be running, so closing it must ask"
    );
    assert_eq!(task.what_is_happening(), crate::words::STOP_NOT_CONFIRMED);
}

/// **And the task window still offers Stop**, so the person can try again.
#[test]
fn an_unconfirmed_stop_leaves_stop_on_offer_in_the_window() {
    let mut task = a_task();
    task.and_then(Act::doing("Renaming IMG_0012"));
    task.stop();
    task.its_stop_could_not_be_confirmed();

    assert_eq!(task.the_one_button(), crate::TheOneButton::Stop);
    task.stop();
    assert_eq!(task.on_its_card().stands, HowItStands::Stopping);
}

/// **Done**: the answer is ready, and Open.
#[test]
fn a_finished_task_is_done_with_its_answer_ready() {
    let mut task = a_task();
    task.alo_says(Turn::from_alo("Q3 revenue rose 4 per cent."));

    let card = task.on_its_card();

    assert_eq!(card.stands, HowItStands::Done);
    assert_eq!(card.how_far, HowFar::AnswerReady);
    assert_eq!(card.button, TheCardsButton::Open);
    assert!(
        !card.stands.may_be_stopped(),
        "Stop offered for work that is over"
    );
}
