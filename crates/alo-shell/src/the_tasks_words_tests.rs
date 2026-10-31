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
use crate::a_change_waiting_reaches_the_person::WhichAnswer;

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

    let both = TheTasksWords::said(
        &tasks,
        &[Some(somewhere()), Some(somewhere())],
        &strings,
        None,
    );
    assert_eq!(both.as_displayed().len(), 2);

    let one = TheTasksWords::said(&tasks, &[Some(somewhere()), None], &strings, None);
    assert_eq!(
        one.as_displayed().len(),
        1,
        "a task with nowhere to be was given a window anyway"
    );

    let neither = TheTasksWords::said(&tasks, &[None, None], &strings, None);
    assert!(neither.as_displayed().is_empty());
}

/// **The title is the person's own words, and the rest comes from the
/// vocabulary.**
#[test]
fn the_title_is_the_persons_words_and_the_rest_is_said() {
    let strings = words();
    let said = TheTasksWords::said(&[a_task()], &[Some(somewhere())], &strings, None);
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

    let said = TheTasksWords::said(&[task], &[Some(somewhere())], &strings, None);
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

    let said = TheTasksWords::said(&[a_task()], &[Some(somewhere())], &strings, None);
    let drawn = said.as_displayed();

    assert_eq!(
        drawn.first().expect("one window").words.destination,
        expected,
        "the window words where processing happens differently from the rest of \
         this machine"
    );
}

/// **The model's chip says the name the person chose, exactly as they wrote
/// it, and nothing where they chose none** — *what the person configured;
/// nothing is branded* (the owner, 2026-10-31).
///
/// Until that day this test asserted the chip was empty, because a task carried
/// no model's name, and it was written to fail on the day one did.
#[test]
fn the_models_chip_says_the_name_the_person_chose() {
    let strings = words();
    let named = a_task().asked_of_the_model_named(Some("gpt-4o"));
    let said = TheTasksWords::said(&[named, a_task()], &[Some(somewhere()), Some(somewhere())], &strings, None);
    let drawn = said.as_displayed();

    assert_eq!(drawn.first().expect("two windows").words.model, "gpt-4o");
    assert!(
        drawn.get(1).expect("two windows").words.model.is_empty(),
        "a task whose person named no model was given a name"
    );
}

/// **What was said keeps its order and its speaker.**
#[test]
fn what_was_said_keeps_its_order_and_its_speaker() {
    let strings = words();
    let said = TheTasksWords::said(&[a_task()], &[Some(somewhere())], &strings, None);
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
    let said = TheTasksWords::said(&[a_task()], &[Some(somewhere())], &strings, None);
    let drawn = said.as_displayed();
    let one = drawn.first().expect("one window");

    assert_eq!(one.how_many_files, 0);
    assert!(one.words.files.is_empty());
    assert!(one.words.how_many_files.is_empty());
}

/// The machine's vocabulary **with the shell's card words declared**, as the
/// shell declares them at boot. `alo-saying` does not collect the shell's own
/// words, so without this the answers would be a fallback, not *Allow once*.
fn words_with_the_card() -> alo_strings::Strings {
    let mut vocabulary =
        alo_saying::everything_this_machine_can_say().expect("the machine's vocabulary");
    crate::the_cards_words::declare_the_cards_words(&mut vocabulary)
        .expect("the card's words declare");
    alo_strings::Strings::of(vocabulary)
}

/// A task with a change waiting for the person to answer.
fn a_task_asking(sentence: &str) -> ATask {
    let mut task = a_task();
    task.is_asking(sentence);
    task.alo_says(Turn::alo_proposing(sentence));
    task
}

/// **The waiting proposal carries the two answers, worded, and which one the
/// keyboard is on.**
#[test]
fn the_waiting_proposal_carries_the_two_answers() {
    let strings = words_with_the_card();
    let sentence = "Rename march.pdf to invoice-march.pdf";
    let tasks = [a_task_asking(sentence)];

    let said = TheTasksWords::said(&tasks, &[Some(somewhere())], &strings, None);
    let drawn = said.as_displayed();
    let window = drawn.first().expect("one window");

    let carrying: Vec<_> = window
        .said
        .iter()
        .filter(|one| one.answers.is_some())
        .collect();
    assert_eq!(
        carrying.len(),
        1,
        "the answers were not under exactly one block"
    );
    let proposal = carrying[0];
    assert_eq!(
        proposal.words, sentence,
        "the answers sit under something else"
    );

    let answers = proposal.answers.expect("answers");
    assert_eq!(answers.allow_once, "Allow once");
    assert_eq!(answers.deny, "Deny");
    assert_eq!(answers.selected, None, "an answer was preselected");
}

/// The keyboard's place reaches the drawing.
#[test]
fn the_selected_answer_reaches_the_drawing() {
    let strings = words_with_the_card();
    let tasks = [a_task_asking("Rename march.pdf")];

    let said = TheTasksWords::said(
        &tasks,
        &[Some(somewhere())],
        &strings,
        Some(WhichAnswer::Deny),
    );
    let drawn = said.as_displayed();
    let answers = drawn
        .first()
        .expect("one window")
        .said
        .iter()
        .find_map(|one| one.answers)
        .expect("answers under the proposal");
    assert_eq!(answers.selected, Some(WhichAnswer::Deny));
}

/// **A proposal already answered carries no answers**, so a person cannot
/// answer the same change twice by finding its old card.
#[test]
fn an_answered_proposal_carries_no_answers() {
    let strings = words_with_the_card();
    let mut task = a_task_asking("Rename march.pdf");
    task.allow_once();

    let said = TheTasksWords::said(&[task], &[Some(somewhere())], &strings, None);
    let drawn = said.as_displayed();
    assert!(
        drawn
            .first()
            .expect("one window")
            .said
            .iter()
            .all(|one| one.answers.is_none()),
        "an answered proposal still offered its answers"
    );
}

/// **Only the latest proposal carries them**: the earlier one was answered,
/// and two cards with answers under them is two questions at once.
#[test]
fn only_the_latest_proposal_carries_the_answers() {
    let strings = words_with_the_card();
    let mut task = a_task_asking("Rename march.pdf");
    task.allow_once();
    task.is_asking("Archive Pictures");
    task.alo_says(Turn::alo_proposing("Archive Pictures"));

    let said = TheTasksWords::said(&[task], &[Some(somewhere())], &strings, None);
    let drawn = said.as_displayed();
    let window = drawn.first().expect("one window");
    let carrying: Vec<_> = window
        .said
        .iter()
        .filter(|one| one.answers.is_some())
        .collect();
    assert_eq!(carrying.len(), 1, "more than one proposal offered answers");
    assert_eq!(carrying[0].words, "Archive Pictures");
}

/// **A task that is not asking draws no answers at all**, whatever selection
/// the machine holds — the selection belongs to the one task asking.
#[test]
fn a_task_not_asking_draws_no_answers_whatever_is_selected() {
    let strings = words_with_the_card();
    let said = TheTasksWords::said(
        &[a_task()],
        &[Some(somewhere())],
        &strings,
        Some(WhichAnswer::AllowOnce),
    );
    let drawn = said.as_displayed();
    assert!(
        drawn
            .first()
            .expect("one window")
            .said
            .iter()
            .all(|one| one.answers.is_none()),
        "a task with nothing waiting drew answers"
    );
}

/// **alo's Markdown is read into marks; a person's words and a proposal are
/// drawn exactly as said, with nothing marked.** The control is the same `**`
/// in all three.
#[test]
fn only_alos_own_words_are_read_for_markdown() {
    let strings = words();
    let mut task = ATask::asked_for("rename **draft**.txt", InferenceSource::ThisMachine);
    task.alo_says(Turn::from_alo("1. **Header**: the word"));
    task.is_asking("Rename **draft**.txt");
    task.alo_says(Turn::alo_proposing("Rename **draft**.txt"));

    let said = TheTasksWords::said(&[task], &[Some(somewhere())], &strings, None);
    let drawn = said.as_displayed();
    let one = drawn.first().expect("one window");

    assert_eq!(
        one.said[0].words, "rename **draft**.txt",
        "the person's words were changed"
    );
    assert!(
        one.said[0].marks.is_empty(),
        "the person's words were marked"
    );
    assert_eq!(one.said[1].words, "1. Header: the word");
    assert!(
        one.said[1]
            .marks
            .iter()
            .any(|mark| mark.how == crate::alos_marked_words::HowItIsMarked::Strong),
        "alo's bold was not read"
    );
    assert_eq!(
        one.said[2].words, "Rename **draft**.txt",
        "the sentence to approve was changed"
    );
    assert!(
        one.said[2].marks.is_empty(),
        "the sentence to approve was marked"
    );
}
