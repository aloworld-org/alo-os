//! What a task put aside shows on its card and its square in the right-hand
//! panel: **one of six states**, and the four lines and one button the design
//! draws for each.
//!
//! The design is `07 / A running task in the put-aside panel` (`23:2`),
//! approved by the owner on 2026-11-11 with *"Approve as drawn"*, and measured
//! in `docs/design/the-windows-put-aside.md`. This file decides **what** each
//! line says; `alo-shell` decides where it goes and how it looks.
//!
//! # Read from the task, never kept beside it
//!
//! A card is worked out from the task every time it is drawn, so the card and
//! the task window cannot disagree about whether alo is still working — the
//! same reason [`ATask::what_is_happening`] is the one place the window's
//! status is decided. And the order of the questions below is that method's:
//! a Stop asked for outranks anything the activity says.

use crate::ATask;
use crate::stopping::Stopping;

/// Which of the six states a task is in, as its card and square show it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowItStands {
    /// alo is acting. The only state drawn in teal.
    Working,
    /// alo waits for the person to approve a change, or to answer.
    NeedsYourOk,
    /// The person pressed Stop and alo has not yet confirmed.
    Stopping,
    /// Stop was confirmed, and this is how many changes it kept.
    Stopped {
        /// How many changes the person approved before Stop. Possibly none,
        /// which is said rather than left blank.
        changes_kept: usize,
    },
    /// The person pressed Stop and this machine could not deliver it.
    StopNotConfirmed,
    /// The task finished on its own.
    Done,
}

impl HowItStands {
    /// Whether the work may still be running, which is when Stop is offered
    /// on the card and Delete on a focused square stops it.
    #[must_use]
    pub const fn may_be_stopped(self) -> bool {
        matches!(self, Self::Working | Self::NeedsYourOk)
    }
}

/// What the card's third line says: how far the task has got.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HowFar {
    /// What alo is doing now, as its activity says it.
    Doing(String),
    /// What alo is waiting on the person for, as the change was put to them.
    Wants(String),
    /// Nothing to say yet: alo is thinking, with no act to report.
    NothingToSayYet,
    /// A Stop is on its way.
    WaitingForAloToConfirm,
    /// Stop was confirmed after this many changes.
    ChangesKept(usize),
    /// The Stop could not be delivered.
    CannotConfirmItStopped,
    /// The task finished, with its answer in the window.
    AnswerReady,
}

/// What the card's fourth line says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LastApproved {
    /// The last change the person approved, as it was put to them.
    This(String),
    /// The person has approved nothing in this task.
    Nothing,
    /// Changes were kept by a Stop, and taking them back is in History.
    UndoIsInHistory,
}

/// The one button on the card, or none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheCardsButton {
    /// Stop the work. Filled navy, while alo works.
    Stop,
    /// Open the task window at the change waiting. Outlined.
    Review,
    /// Open the task window. Outlined.
    Open,
    /// No button: a Stop is on its way, and the card says so.
    None,
}

/// Everything a task's card shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnItsCard {
    /// Which state, which decides the first line, its colour and the square.
    pub stands: HowItStands,
    /// The task in the person's own words: what they asked for.
    pub task: String,
    /// How far it has got.
    pub how_far: HowFar,
    /// The last thing it did that the person approved.
    pub last_approved: LastApproved,
    /// The button at the card's bottom right.
    pub button: TheCardsButton,
}

impl ATask {
    /// Which of the six states this task is in.
    #[must_use]
    pub fn how_it_stands(&self) -> HowItStands {
        match self.stopping() {
            Stopping::Asked => HowItStands::Stopping,
            Stopping::CouldNotBeConfirmed => HowItStands::StopNotConfirmed,
            Stopping::Stopped { changed } => HowItStands::Stopped {
                changes_kept: changed.how_many(),
            },
            Stopping::NobodyAsked => {
                if self.is_waiting_for_the_person() {
                    HowItStands::NeedsYourOk
                } else if self.activity().doing_now().is_some() || self.conversation().is_alos_turn()
                {
                    HowItStands::Working
                } else {
                    HowItStands::Done
                }
            }
        }
    }

    /// What this task's card in the right-hand panel shows.
    #[must_use]
    pub fn on_its_card(&self) -> OnItsCard {
        let stands = self.how_it_stands();
        let task = self
            .conversation()
            .the_request()
            .map(|turn| turn.said().to_owned())
            .unwrap_or_default();
        let last = self.what_it_changed().each().last().cloned();
        let approved = last.map_or(LastApproved::Nothing, LastApproved::This);
        let (how_far, last_approved, button) = match stands {
            HowItStands::Working => (
                self.activity()
                    .doing_now()
                    .map_or(HowFar::NothingToSayYet, |act| HowFar::Doing(act.said().to_owned())),
                approved,
                TheCardsButton::Stop,
            ),
            HowItStands::NeedsYourOk => (
                self.permissions()
                    .asking()
                    .map_or(HowFar::NothingToSayYet, |what| HowFar::Wants(what.to_owned())),
                approved,
                TheCardsButton::Review,
            ),
            HowItStands::Stopping => (HowFar::WaitingForAloToConfirm, approved, TheCardsButton::None),
            HowItStands::Stopped { changes_kept } => (
                HowFar::ChangesKept(changes_kept),
                if changes_kept > 0 {
                    LastApproved::UndoIsInHistory
                } else {
                    LastApproved::Nothing
                },
                TheCardsButton::Open,
            ),
            HowItStands::StopNotConfirmed => {
                (HowFar::CannotConfirmItStopped, approved, TheCardsButton::Open)
            }
            HowItStands::Done => (HowFar::AnswerReady, approved, TheCardsButton::Open),
        };
        OnItsCard {
            stands,
            task,
            how_far,
            last_approved,
            button,
        }
    }
}

#[cfg(test)]
#[path = "on_its_card_tests.rs"]
mod tests;
