//! **A running task's card in the right-hand panel, in words**: what each line
//! of the approved design `23:2` says for one task, already said in the
//! person's language.
//!
//! `alo_tasks::OnItsCard` decides *which* of six states a task is in and what
//! each line is about; this file turns that into sentences with the person's
//! [`Strings`], and `crate::task_card_raster` draws them. The lane builds these
//! once a frame and the frame borrows them — the same shape the task windows'
//! own words take (`crate::the_tasks_words`), for the same reason: the raster
//! holds `&str`, and the strings have to live somewhere the lane keeps.

use alo_put_aside::TaskNumber;
use alo_strings::{Filling, Strings, Word};
use alo_tasks::card_words as words;
use alo_tasks::{ATask, HowFar, HowItStands, LastApproved, TheCardsButton};

/// One task's card and square, said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ATaskAsideOnTheDisplay {
    /// Which task.
    pub(crate) task: TaskNumber,
    /// Which of the six states, which decides colours and the square's ring.
    pub(crate) stands: HowItStands,
    /// Line one: the state, beside alo's mark.
    pub(crate) state: String,
    /// The word under the square, on focus or hover.
    pub(crate) under_the_square: String,
    /// What a screen reader says for the square.
    pub(crate) name: String,
    /// Line two: the task in the person's own words.
    pub(crate) title: String,
    /// Line three: how far it has got. Empty when there is nothing to say yet.
    pub(crate) how_far: String,
    /// Line four: the last thing approved.
    pub(crate) last: String,
    /// The button and its word, or [`None`] while a Stop is on its way.
    pub(crate) button: Option<(TheCardsButton, String)>,
}

/// Say every task's card.
///
/// **Every task that is still open**, whether or not it is in the panel: the
/// panel decides which it shows, and asking it here would be a second place
/// deciding. A closed task has no card.
pub(crate) fn the_cards(
    tasks: &crate::the_tasks_a_person_opened::TheTasks,
    strings: &Strings,
) -> Vec<ATaskAsideOnTheDisplay> {
    tasks
        .each()
        .iter()
        .enumerate()
        .filter(|(at, _)| {
            tasks
                .the_task_numbered(*at)
                .is_some_and(|which| tasks.is_open(which))
        })
        .map(|(at, task)| said(TaskNumber::numbered(at), task, strings))
        .collect()
}

/// One task's card, said.
pub(crate) fn said(task: TaskNumber, of: &ATask, strings: &Strings) -> ATaskAsideOnTheDisplay {
    let say = |word: Word| strings.say(&word.key(), &Filling::nothing()).into_text();
    let card = of.on_its_card();
    let state = match card.stands {
        HowItStands::Working => say(alo_tasks::words::ALO_IS_WORKING),
        HowItStands::NeedsYourOk => say(words::NEEDS_YOUR_OK),
        HowItStands::Stopping => say(alo_tasks::words::STOPPING),
        HowItStands::Stopped { changes_kept: 0 } => say(words::STOPPED_NOTHING_CHANGED),
        HowItStands::Stopped { .. } => say(words::STOPPED_CHANGES_KEPT),
        HowItStands::StopNotConfirmed => say(alo_tasks::words::STOP_NOT_CONFIRMED),
        HowItStands::Done => say(words::DONE),
    };
    let under_the_square = match card.stands {
        HowItStands::Working => say(words::RAIL_WORKING),
        HowItStands::NeedsYourOk => say(words::RAIL_NEEDS_YOU),
        HowItStands::Stopping => say(alo_tasks::words::STOPPING),
        HowItStands::Stopped { .. } => say(words::RAIL_STOPPED),
        HowItStands::StopNotConfirmed => say(alo_tasks::words::STOP_NOT_CONFIRMED),
        HowItStands::Done => say(words::DONE),
    };
    let how_far = match &card.how_far {
        HowFar::Doing(what) | HowFar::Wants(what) => what.clone(),
        HowFar::NothingToSayYet => String::new(),
        HowFar::WaitingForAloToConfirm => say(words::WAITING_FOR_ALO_TO_CONFIRM),
        HowFar::ChangesKept(0) => say(words::NOTHING_CHANGED_BEFORE_STOP),
        HowFar::ChangesKept(count) => strings
            .say(
                &words::CHANGES_KEPT_BEFORE_STOP.key(),
                &Filling::of("count", count.to_string()),
            )
            .into_text(),
        HowFar::CannotConfirmItStopped => say(words::CANNOT_CONFIRM_IT_STOPPED),
        HowFar::AnswerReady => say(words::ANSWER_READY),
    };
    let last = match &card.last_approved {
        LastApproved::This(what) => strings
            .say(
                &words::LAST_APPROVED.key(),
                &Filling::of("what", what.clone()),
            )
            .into_text(),
        LastApproved::Nothing => say(words::NOTHING_APPROVED_YET),
        LastApproved::UndoIsInHistory => say(words::UNDO_IS_IN_HISTORY),
    };
    let button = match card.button {
        TheCardsButton::Stop => Some((TheCardsButton::Stop, say(words::STOP))),
        TheCardsButton::Review => Some((TheCardsButton::Review, say(words::REVIEW))),
        TheCardsButton::Open => Some((TheCardsButton::Open, say(words::OPEN))),
        TheCardsButton::None => None,
    };
    let name = strings
        .say(
            &words::RAIL_NAME.key(),
            &Filling::of("state", under_the_square.clone()).and("task", card.task.clone()),
        )
        .into_text();
    ATaskAsideOnTheDisplay {
        task,
        stands: card.stands,
        state,
        under_the_square,
        name,
        title: card.task,
        how_far,
        last,
        button,
    }
}

#[cfg(test)]
#[path = "a_task_in_the_panel_tests.rs"]
mod tests;

/// **The tasks a frame can draw in the panel**: every task's card, said, and
/// which one the person opened.
///
/// Carried on `crate::DesktopFrame` beside `tasks`, and for its reason: the
/// raster borrows the words, and the lane keeps them alive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheTasksAside<'a> {
    /// Every open task's card, said.
    pub(crate) cards: &'a [ATaskAsideOnTheDisplay],
    /// The task whose card the person opened from its square, if any.
    pub(crate) open: Option<TaskNumber>,
}

impl TheTasksAside<'_> {
    /// No task in the panel, and no card open — every desktop until a person
    /// puts a running task aside.
    pub const NONE: TheTasksAside<'static> = TheTasksAside {
        cards: &[],
        open: None,
    };
}

impl<'a> TheTasksAside<'a> {
    /// These cards, with `open` the one the person opened.
    #[must_use]
    pub const fn of(cards: &'a [ATaskAsideOnTheDisplay], open: Option<TaskNumber>) -> Self {
        Self { cards, open }
    }
}
