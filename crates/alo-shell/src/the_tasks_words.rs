//! Everything in a person's task windows that they read, said once a frame.
//!
//! `crate::the_docks_words`' shape and for its reason: **the drawing path may
//! not word a sentence.** Every sentence a person meets comes from
//! `alo-saying`, and a raster that looked one up would be a second place
//! deciding what a person reads. So the lane says them and hands the results
//! over.
//!
//! # Owned here, borrowed by the frame
//!
//! `crate::task_raster::ATaskOnTheDisplay` holds `&str`, because a raster
//! should not own what it draws. So this holds the `String`s and
//! [`TheTasksWords::as_displayed`] lends them — which is why a caller keeps
//! this alive as a local for as long as the frame it built.
//!
//! # Which word, and who chose it
//!
//! **The model chose it.** `alo_tasks::ATask::what_is_happening` answers which
//! of the five status words applies, because that is a reading of the task; this
//! only says it. Two places working it out is how two surfaces come to disagree
//! about whether alo is still working.

use alo_strings::{Filling, Strings};
use alo_tasks::ATask;
use smithay::utils::{Physical, Rectangle};

use crate::alos_marked_words::MarkedWords;
use crate::task_raster::ATaskOnTheDisplay;
use crate::task_text::{TheWordsInATaskWindow, WhatWasSaid};

/// One task window's words, owned.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ATasksWords {
    /// Where its window is on this display.
    at: Rectangle<i32, Physical>,
    /// Its place in the list of tasks.
    place: usize,
    /// How far its conversation is scrolled back.
    scroll: alo_tasks::TheScroll,
    /// What its one button shows, Send or Stop.
    the_one_button: alo_tasks::TheOneButton,
    /// Whether a provider answers it, which puts the cloud before where it happens.
    a_provider_answers: bool,
    /// What the person asked for, which names it.
    title: String,
    /// What kind of thing it is.
    kind: String,
    /// What is happening in it.
    status: String,
    /// Which model answers.
    model: String,
    /// Where the processing happens.
    destination: String,
    /// The invitation in the empty reply field.
    invitation: String,
    /// How far the choices in it reach.
    scope_note: String,
    /// Each shared file's label.
    files: Vec<String>,
    /// How many files are shared, as a sentence.
    how_many_files: String,
    /// What was said, oldest first.
    said: Vec<(alo_tasks::WhoseBlock, MarkedWords)>,
    /// Which of `said` carries the two answers: the latest proposal, while
    /// this task is still asking. [`None`] when nothing in it is waiting.
    answers_after: Option<usize>,
    /// *Allow once*, said once for this task.
    allow_once: String,
    /// *Deny*, said once for this task.
    deny: String,
    /// Which answer the keyboard is on, and [`None`] until the person moves.
    selected: Option<crate::a_change_waiting_reaches_the_person::WhichAnswer>,
    /// Which of `said` carries *Stop and close* and *Cancel*: the question
    /// about closing a running task, while it is asked (ADR 0139).
    closing_after: Option<usize>,
    /// *Stop and close*, said once for this task.
    stop_and_close: String,
    /// *Cancel*, said once for this task.
    cancel: String,
    /// Which of those two the keyboard is on.
    closing_selected: Option<crate::a_change_waiting_reaches_the_person::WhichAnswer>,
}

/// Every task window's words, said once for this frame.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TheTasksWords {
    /// One per open task window, in the order they opened.
    each: Vec<ATasksWords>,
}

impl TheTasksWords {
    /// Say everything the person's task windows show.
    ///
    /// `placed` is where each task's window is, in the same order as `tasks`. A
    /// task with no position yet is **left out**: a window nothing has placed is
    /// a window with nowhere to be drawn, and inventing a position here would be
    /// a second placer.
    ///
    /// `selected` is which answer the person's keyboard is on, for the one
    /// question that stands on this machine. It reaches only the task that is
    /// asking, because only that task draws the two answers.
    pub(crate) fn said(
        tasks: &[ATask],
        placed: &[Option<Rectangle<i32, Physical>>],
        strings: &Strings,
        selected: Option<crate::a_change_waiting_reaches_the_person::WhichAnswer>,
    ) -> Self {
        let say =
            |word: alo_strings::Word| strings.say(&word.key(), &Filling::nothing()).into_text();
        let each = tasks
            .iter()
            .zip(placed.iter())
            .enumerate()
            .filter_map(|(place, (task, at))| {
                let at = (*at)?;
                Some(ATasksWords {
                    at,
                    place,
                    scroll: task.scroll(),
                    the_one_button: task.the_one_button(),
                    a_provider_answers: task.model().a_provider_answers(),
                    // **The person's own words, not a sentence this machine
                    // owns.** A task is named by what somebody asked for.
                    title: task
                        .conversation()
                        .the_request()
                        .map(|turn| turn.said().to_owned())
                        .unwrap_or_default(),
                    kind: say(alo_tasks::words::A_TASK),
                    status: say(task.what_is_happening()),
                    // **The name of the model the person chose, as they
                    // wrote it** — *what the person configured; nothing is
                    // branded*, the owner's decision of 2026-10-31. Recorded on
                    // the task when it was asked (`ATask::the_models_name`),
                    // and empty where the person named none, as for a machine
                    // down the corridor: printing the place's name in the slot
                    // the design gives to a model would be a window telling a
                    // person something it does not know.
                    model: task.the_models_name().unwrap_or_default().to_owned(),
                    // **`alo-models`' own sentence**, which words all four
                    // sources and fills in a provider and a region. Not this
                    // crate's to say: two sentences about where a person's data
                    // goes is the one duplication law 1 cannot afford.
                    destination: task.model().answering().said(strings).into_text(),
                    invitation: say(alo_tasks::words::REPLY_TO_ALO),
                    scope_note: say(alo_tasks::words::WHAT_THIS_TASK_REACHES),
                    // Nothing shares a file with a task yet: attachments are
                    // their own promise and are not built, so this is empty
                    // rather than invented.
                    files: Vec::new(),
                    how_many_files: String::new(),
                    said: task
                        .conversation()
                        .each()
                        .iter()
                        .map(|turn| (whose(turn), as_drawn(turn)))
                        .chain(held(task, strings))
                        .collect(),
                    // **The latest proposal, and only while the task is
                    // asking.** Every earlier proposal was answered and draws
                    // no answers, and a task that is not asking draws none at
                    // all — whatever its conversation still holds.
                    answers_after: task.permissions().asking().and_then(|_| {
                        task.conversation().each().iter().rposition(|turn| {
                            matches!(turn.who(), alo_tasks::WhoSaidIt::AloProposing)
                        })
                    }),
                    allow_once: say(crate::the_cards_words::ALLOW_ONCE),
                    deny: say(crate::the_cards_words::DENY),
                    selected,
                    closing_after: None,
                    stop_and_close: String::new(),
                    cancel: String::new(),
                    closing_selected: None,
                })
            })
            .collect();
        Self { each }
    }

    /// **And the question about closing a running task**, on a card at the end
    /// of that task's conversation, with *Stop and close* and *Cancel* under it
    /// (`crate::asking_before_closing`; ADR 0139). Nothing when nothing is
    /// asked.
    pub(crate) fn with_the_closing_question(
        mut self,
        asked: Option<crate::asking_before_closing::ClosingAsked>,
        strings: &Strings,
    ) -> Self {
        use crate::a_change_waiting_reaches_the_person::WhichAnswer;
        use crate::asking_before_closing::WhichChoice;

        let Some(asked) = asked else {
            return self;
        };
        let say =
            |word: alo_strings::Word| strings.say(&word.key(), &Filling::nothing()).into_text();
        if let Some(one) = self
            .each
            .iter_mut()
            .find(|one| one.place == asked.which.at())
        {
            one.said.push((
                alo_tasks::WhoseBlock::AnAccount,
                MarkedWords::as_said(&say(crate::the_closing_words::ALO_IS_STILL_WORKING_ON_THIS)),
            ));
            one.closing_after = Some(one.said.len().saturating_sub(1));
            one.stop_and_close = say(crate::the_closing_words::STOP_AND_CLOSE);
            one.cancel = say(crate::the_closing_words::CANCEL);
            one.closing_selected = asked.selected.map(|choice| match choice {
                WhichChoice::StopAndClose => WhichAnswer::AllowOnce,
                WhichChoice::Cancel => WhichAnswer::Deny,
            });
        }
        self
    }

    /// The task windows a frame draws, borrowing these words.
    pub(crate) fn as_displayed(&self) -> Vec<ATaskOnTheDisplay<'_>> {
        self.each
            .iter()
            .map(|one| ATaskOnTheDisplay {
                at: one.at,
                place: one.place,
                scroll: one.scroll,
                the_one_button: one.the_one_button,
                a_provider_answers: one.a_provider_answers,
                how_many_files: one.files.len(),
                words: TheWordsInATaskWindow {
                    title: &one.title,
                    kind: &one.kind,
                    status: &one.status,
                    model: &one.model,
                    destination: &one.destination,
                    invitation: &one.invitation,
                    scope_note: &one.scope_note,
                    files: &one.files,
                    how_many_files: &one.how_many_files,
                },
                // Built here so the frame borrows these strings rather than
                // copying them, which is why the bundle has to outlive it.
                said: one
                    .said
                    .iter()
                    .enumerate()
                    .map(|(at, (whose, words))| WhatWasSaid {
                        whose: *whose,
                        words: words.text(),
                        marks: words.marks(),
                        answers: if one.answers_after == Some(at) {
                            Some(crate::task_text::TheTwoAnswers {
                                allow_once: &one.allow_once,
                                deny: &one.deny,
                                selected: one.selected,
                            })
                        } else if one.closing_after == Some(at) {
                            // **The approval card's two answers, with the
                            // closing question's words** (ADR 0139).
                            Some(crate::task_text::TheTwoAnswers {
                                allow_once: &one.stop_and_close,
                                deny: &one.cancel,
                                selected: one.closing_selected,
                            })
                        } else {
                            None
                        },
                    })
                    .collect(),
            })
            .collect()
    }
}

/// **A reply waiting to be sent**, drawn under the conversation: the person's
/// words, and beneath them a card saying it has not gone yet, when it will,
/// and how to take it back (ADR 0138). Nothing when nothing is held.
///
/// Two blocks the window already draws, rather than a new kind: the words are
/// the person's, and the note is the machine's account of them — the card an
/// activity is drawn in.
fn held(task: &ATask, strings: &Strings) -> Vec<(alo_tasks::WhoseBlock, MarkedWords)> {
    let Some(held) = task.held() else {
        return Vec::new();
    };
    let note = if task.permissions().asking().is_some() {
        crate::the_reply_boxs_words::HELD_UNTIL_YOU_ANSWER
    } else {
        crate::the_reply_boxs_words::HELD_UNTIL_ALO_ANSWERS
    };
    vec![
        (
            alo_tasks::WhoseBlock::ThePersons,
            MarkedWords::as_said(held),
        ),
        (
            alo_tasks::WhoseBlock::AnAccount,
            MarkedWords::as_said(&strings.say(&note.key(), &Filling::nothing()).into_text()),
        ),
    ]
}

/// The words one turn is drawn with.
///
/// **alo's replies and questions with their Markdown read into bold,
/// headings and code**, and everything else exactly as it was said: a
/// person's words are theirs, and a proposal is the sentence they approve.
/// `crate::alos_marked_words` says what is drawn and what is only taken out.
fn as_drawn(turn: &alo_tasks::Turn) -> MarkedWords {
    match turn.who() {
        alo_tasks::WhoSaidIt::Alo | alo_tasks::WhoSaidIt::AloAsking => {
            MarkedWords::read_from(turn.said())
        }
        alo_tasks::WhoSaidIt::ThePerson | alo_tasks::WhoSaidIt::AloProposing => {
            MarkedWords::as_said(turn.said())
        }
    }
}

/// Which kind of block one turn is drawn as.
///
/// **A proposal is drawn as the approval card**, which the design calls *the
/// same shell as an activity block* — built 2026-10-26, and the waiting one
/// carries Allow once and Deny under it.
///
/// **A question is still drawn as alo's words.** It is a person's to answer
/// too, and its card is not built, so until it is it reads as what alo said.
/// Named here rather than silently mapped: a reader should be able to see that
/// card is missing rather than wonder why a question looks like a reply.
fn whose(turn: &alo_tasks::Turn) -> alo_tasks::WhoseBlock {
    match turn.who() {
        alo_tasks::WhoSaidIt::ThePerson => alo_tasks::WhoseBlock::ThePersons,
        alo_tasks::WhoSaidIt::Alo | alo_tasks::WhoSaidIt::AloAsking => alo_tasks::WhoseBlock::Alos,
        // **A proposal is a card** — the design's approval card is *the same
        // shell as an activity block*, which is this. Every proposal is one,
        // answered or not; only the one still waiting has the two answers
        // under it.
        alo_tasks::WhoSaidIt::AloProposing => alo_tasks::WhoseBlock::AnAccount,
    }
}

#[cfg(test)]
#[path = "the_tasks_words_tests.rs"]
mod tests;
