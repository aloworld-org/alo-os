//! Each of the six states says the design's words, and the refusals: no
//! button while a Stop is on its way, never *stopped* for a Stop nothing
//! confirmed, and the state is in the screen reader's name, not only in a
//! colour.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_models::InferenceSource;
use alo_put_aside::TaskNumber;
use alo_strings::{Strings, Vocabulary};
use alo_tasks::{ATask, Act, HowItStands, TheCardsButton, WhatChanged};

use super::said;

fn strings() -> Strings {
    let mut vocabulary = Vocabulary::default();
    alo_tasks::declare_into(&mut vocabulary).expect("the task words declare");
    Strings::of(vocabulary)
}

fn a_task() -> ATask {
    ATask::asked_for("Summarise the Q3 report", InferenceSource::ThisMachine)
}

#[test]
fn a_working_task_says_alo_is_working_and_offers_stop() {
    let mut task = a_task();
    task.and_then(Act::doing("Reading report.pdf"));

    let card = said(TaskNumber::numbered(0), &task, &strings());

    assert_eq!(card.state, "alo is working");
    assert_eq!(card.title, "Summarise the Q3 report");
    assert_eq!(card.how_far, "Reading report.pdf");
    assert_eq!(card.last, "Nothing approved yet");
    assert_eq!(card.button, Some((TheCardsButton::Stop, "Stop".to_owned())));
    assert_eq!(card.under_the_square, "Working");
}

#[test]
fn a_task_waiting_says_needs_your_ok_and_offers_review() {
    let mut task = a_task();
    task.is_asking("Move 14 files to Archive");

    let card = said(TaskNumber::numbered(0), &task, &strings());

    assert_eq!(card.state, "Needs your OK");
    assert_eq!(card.how_far, "Move 14 files to Archive");
    assert_eq!(card.button, Some((TheCardsButton::Review, "Review".to_owned())));
    assert_eq!(card.under_the_square, "Needs you");
}

#[test]
fn the_last_approved_change_is_named() {
    let mut task = a_task();
    task.is_asking("open report.pdf");
    task.allow_once();
    task.and_then(Act::doing("Reading report.pdf"));

    assert_eq!(
        said(TaskNumber::numbered(0), &task, &strings()).last,
        "Last approved: open report.pdf"
    );
}

#[test]
fn a_stop_on_its_way_has_no_button() {
    let mut task = a_task();
    task.stop();

    let card = said(TaskNumber::numbered(0), &task, &strings());

    assert_eq!(card.state, "Stopping\u{2026}");
    assert_eq!(card.how_far, "Waiting for alo to confirm");
    assert_eq!(card.button, None, "a second Stop was offered for one Stop");
}

#[test]
fn a_confirmed_stop_says_what_it_kept_and_where_to_undo_it() {
    let mut task = a_task();
    task.stop();
    let mut changed = WhatChanged::nothing();
    changed.and_then("Moved a.pdf");
    changed.and_then("Moved b.pdf");
    task.it_stopped(changed);

    let card = said(TaskNumber::numbered(0), &task, &strings());

    assert_eq!(card.state, "Stopped \u{b7} changes kept");
    assert_eq!(card.how_far, "Changes kept before Stop: 2");
    assert_eq!(card.last, "Undo is in History");
    assert_eq!(card.button, Some((TheCardsButton::Open, "Open".to_owned())));
}

#[test]
fn a_stop_that_kept_nothing_does_not_claim_changes() {
    let mut task = a_task();
    task.stop();
    task.it_stopped(WhatChanged::nothing());

    let card = said(TaskNumber::numbered(0), &task, &strings());

    assert_eq!(card.state, "Stopped \u{b7} nothing changed");
    assert!(!card.state.contains("kept"), "{}", card.state);
}

#[test]
fn an_undelivered_stop_is_never_said_to_have_stopped() {
    let mut task = a_task();
    task.stop();
    task.its_stop_could_not_be_confirmed();

    let card = said(TaskNumber::numbered(0), &task, &strings());

    assert_eq!(card.stands, HowItStands::StopNotConfirmed);
    assert_eq!(card.state, "Stop requested \u{b7} not confirmed");
    assert_eq!(card.how_far, "This machine cannot confirm it stopped");
    assert_eq!(card.button, Some((TheCardsButton::Open, "Open".to_owned())));
}

#[test]
fn a_finished_task_is_done() {
    let mut task = a_task();
    task.alo_says(alo_tasks::Turn::from_alo("Revenue rose."));

    let card = said(TaskNumber::numbered(0), &task, &strings());

    assert_eq!(card.state, "Done");
    assert_eq!(card.how_far, "Answer ready");
}

/// **Never colour alone**: the state is in what a screen reader says.
#[test]
fn the_screen_readers_name_says_the_state_in_words() {
    let mut task = a_task();
    task.and_then(Act::doing("Reading report.pdf"));

    assert_eq!(
        said(TaskNumber::numbered(0), &task, &strings()).name,
        "alo task, Working: Summarise the Q3 report"
    );
}
