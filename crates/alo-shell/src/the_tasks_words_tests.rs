//! That a task window's words come from the vocabulary, that a task with
//! nowhere to be is left out, and that nothing here words where a person's data
//! goes.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on a vocabulary that will not load, or on a window this file \
              just asked for, is the failure being reported"
)]

use alo_models::InferenceSource;
use alo_tasks::conversation::Turn;
use alo_tasks::{ATask, WhoseBlock};
use smithay::utils::Rectangle;

use super::TheTasksWords;

/// A machine's whole vocabulary, in English.
fn words() -> alo_strings::Strings {
    alo_strings::Strings::of(
        alo_saying::everything_this_machine_can_say().expect("the machine's vocabulary"),
    )
}

/// A task somebody asked for, with alo working in it.
fn a_task() -> ATask {
    let mut task = ATask::asked_for("Tidy my invoices folder", InferenceSource::ThisMachine);
    task.alo_says(Turn::from_alo("I'll look through them."));
    task
}

/// Somewhere for a window to be.
fn somewhere() -> Rectangle<i32, smithay::utils::Physical> {
    Rectangle::new((120, 80).into(), (1040, 800).into())
}

/// **A task with a position is worded; one with nowhere to be is left out.**
///
/// A window nothing has placed has nowhere to be drawn, and inventing a
/// position here would make this a second placer. The control is the same task
/// with no position, so the test cannot pass by wording everything.
#[test]
fn a_task_with_nowhere_to_be_is_left_out() {
    let strings = words();
    let tasks = [a_task(), a_task()];

    let both = TheTasksWords::said(&tasks, &[Some(somewhere()), Some(somewhere())], &strings);
    assert_eq!(both.as_displayed().len(), 2);

    let one = TheTasksWords::said(&tasks, &[Some(somewhere()), None], &strings);
    assert_eq!(
        one.as_displayed().len(),
        1,
        "a task with nowhere to be was given a window anyway"
    );

    let neither = TheTasksWords::said(&tasks, &[None, None], &strings);
    assert!(neither.as_displayed().is_empty());
}

/// **The title is the person's own words, and the rest comes from the
/// vocabulary.**
#[test]
fn the_title_is_the_persons_words_and_the_rest_is_said() {
    let strings = words();
    let said = TheTasksWords::said(&[a_task()], &[Some(somewhere())], &strings);
    let drawn = said.as_displayed();
    let one = drawn.first().expect("one window");

    assert_eq!(
        one.words.title, "Tidy my invoices folder",
        "the title is not what the person asked for"
    );
    for (what, got) in [
        ("kind", one.words.kind),
        ("status", one.words.status),
        ("invitation", one.words.invitation),
        ("scope note", one.words.scope_note),
        ("destination", one.words.destination),
    ] {
        assert!(
            !got.trim().is_empty(),
            "the {what} was not said: a window would draw nothing there"
        );
    }
}

/// **The status is the word the model chose**, not one this file worked out.
#[test]
fn the_status_is_the_word_the_task_chose() {
    let strings = words();
    let task = a_task();
    let expected = strings
        .say(
            &task.what_is_happening().key(),
            &alo_strings::Filling::nothing(),
        )
        .into_text();

    let said = TheTasksWords::said(&[task], &[Some(somewhere())], &strings);
    let drawn = said.as_displayed();

    assert_eq!(drawn.first().expect("one window").words.status, expected);
}

/// **Where the processing happens is `alo-models`' own sentence.**
///
/// Not this crate's: two sentences about where a person's data goes is the one
/// duplication law 1 cannot afford. A `tasks.where.this-computer` existed for
/// part of 2026-10-12 and was removed for exactly this.
#[test]
fn where_it_happens_is_the_sentence_alo_models_already_says() {
    let strings = words();
    let expected = InferenceSource::ThisMachine.said(&strings).into_text();

    let said = TheTasksWords::said(&[a_task()], &[Some(somewhere())], &strings);
    let drawn = said.as_displayed();

    assert_eq!(
        drawn.first().expect("one window").words.destination,
        expected,
        "the window words where processing happens differently from the rest of \
         this machine"
    );
}

/// **The model's name is empty, and that is the gap rather than a bug.**
///
/// `alo_tasks::WhichModel` holds an `InferenceSource`, which says *where* a
/// question is answered and not *which model* answers it. The design draws a
/// name. Asserted so that the day a task carries a model identity, this test
/// fails and somebody fills the slot — rather than the emptiness going
/// unnoticed because nobody looked at that part of the window.
#[test]
fn the_models_name_is_empty_because_a_task_does_not_carry_one() {
    let strings = words();
    let said = TheTasksWords::said(&[a_task()], &[Some(somewhere())], &strings);
    let drawn = said.as_displayed();

    assert!(
        drawn.first().expect("one window").words.model.is_empty(),
        "a task now carries a model name — fill the slot the design gives it and \
         delete this test"
    );
}

/// **What was said keeps its order and its speaker.**
#[test]
fn what_was_said_keeps_its_order_and_its_speaker() {
    let strings = words();
    let said = TheTasksWords::said(&[a_task()], &[Some(somewhere())], &strings);
    let drawn = said.as_displayed();
    let one = drawn.first().expect("one window");

    assert_eq!(one.said.len(), 2, "a request and a reply");
    assert_eq!(one.said[0].whose, WhoseBlock::ThePersons);
    assert_eq!(one.said[0].words, "Tidy my invoices folder");
    assert_eq!(one.said[1].whose, WhoseBlock::Alos);
}

/// **No files are shared, and the count says so rather than inventing one.**
///
/// Attachments are their own promise and are not built. An empty list is the
/// honest state; a chip drawn for a file nobody attached would be the window
/// claiming something about a person's disk.
#[test]
fn nothing_is_shared_with_a_task_yet_and_the_window_says_nothing() {
    let strings = words();
    let said = TheTasksWords::said(&[a_task()], &[Some(somewhere())], &strings);
    let drawn = said.as_displayed();
    let one = drawn.first().expect("one window");

    assert_eq!(one.how_many_files, 0);
    assert!(one.words.files.is_empty());
    assert!(one.words.how_many_files.is_empty());
}
