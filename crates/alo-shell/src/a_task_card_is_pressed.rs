//! **A press on a running task in the right-hand panel**: on its square, on
//! its card, or on the card's button.
//!
//! The owner's approved design `23:2`: *pressing a square opens its card with
//! Stop first*; *selecting the card (not its button) opens the task window*;
//! the button is **Stop** while alo works, **Review** when it needs the OK and
//! **Open** once it is over. Stop is one press from the rail's card and asks
//! nothing first — it cancels the work and not the application, and stopping is
//! not undoing (`docs/features.md`, *Alo working in a window you put aside*).
//!
//! # Which task, checked against the panel as it is now
//!
//! The press was classified against the panel as it was last drawn, and the
//! panel may have changed since. A task that is no longer in the panel is not
//! acted on — the same rule `crate::which_preview_the_pointer_is_on` keeps for
//! windows, because acting on the wrong task is worse than doing nothing.

use alo_put_aside::{Panel, TaskNumber};
use alo_tasks::TheCardsButton;
use smithay::utils::{Physical, Point, Rectangle};

use crate::the_tasks_a_person_opened::WhichTask;
use crate::which_preview_the_pointer_is_on::ThePanelAsDrawn;

/// What part of a running task in the panel a press landed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnATask {
    /// Its square in the rail.
    ItsSquare(TaskNumber),
    /// Its open card, away from the button.
    ItsCard(TaskNumber),
    /// The open card's button.
    ItsButton(TaskNumber, TheCardsButton),
}

/// What a press at `at` landed on, if it was a task: the button first, then
/// the card around it, then the squares.
pub(crate) fn what_was_pressed(drawn: &ThePanelAsDrawn, at: Point<i32, Physical>) -> Option<OnATask> {
    if let Some(card) = drawn.open_card() {
        if let Some((which, button)) = card.button
            && holds(button, at)
        {
            return Some(OnATask::ItsButton(card.task, which));
        }
        if holds(card.area, at) {
            return Some(OnATask::ItsCard(card.task));
        }
    }
    drawn
        .task_squares()
        .iter()
        .find(|(_, square)| holds(*square, at))
        .map(|(task, _)| OnATask::ItsSquare(*task))
}

/// Whether `at` is inside `rectangle`.
fn holds(rectangle: Rectangle<i32, Physical>, at: Point<i32, Physical>) -> bool {
    rectangle.size.w > 0
        && rectangle.size.h > 0
        && at.x >= rectangle.loc.x
        && at.x < rectangle.loc.x.saturating_add(rectangle.size.w)
        && at.y >= rectangle.loc.y
        && at.y < rectangle.loc.y.saturating_add(rectangle.size.h)
}

/// What a press on a task did, for a caller and a test to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhatThePressDid {
    /// The task's card opened.
    OpenedItsCard,
    /// The task's card closed: its square was pressed again.
    ClosedItsCard,
    /// Stop was asked for.
    Stopped,
    /// The task window came back from the panel.
    BroughtItBack,
    /// Nothing: the task is no longer in the panel, or Stop no longer applies.
    Nothing,
}

impl crate::Server {
    /// **Act on a press on a running task in the panel.**
    pub(crate) fn a_press_on_a_task(&mut self, panel: &mut Panel, on: OnATask) -> WhatThePressDid {
        let task = match on {
            OnATask::ItsSquare(task) | OnATask::ItsCard(task) | OnATask::ItsButton(task, _) => task,
        };
        if !panel.holds_the_task(task) {
            return WhatThePressDid::Nothing;
        }
        match on {
            OnATask::ItsSquare(_) => {
                if self.the_task_card_open == Some(task) {
                    self.the_task_card_open = None;
                    WhatThePressDid::ClosedItsCard
                } else {
                    self.the_task_card_open = Some(task);
                    WhatThePressDid::OpenedItsCard
                }
            }
            OnATask::ItsButton(_, TheCardsButton::Stop) => {
                if self.the_person_stops_a_task(WhichTask::numbered(task.at())) {
                    WhatThePressDid::Stopped
                } else {
                    WhatThePressDid::Nothing
                }
            }
            OnATask::ItsCard(_)
            | OnATask::ItsButton(_, TheCardsButton::Review | TheCardsButton::Open) => {
                if self.bring_a_task_back(panel, task) {
                    WhatThePressDid::BroughtItBack
                } else {
                    WhatThePressDid::Nothing
                }
            }
            OnATask::ItsButton(_, TheCardsButton::None) => WhatThePressDid::Nothing,
        }
    }

}

#[cfg(test)]
#[path = "a_task_card_is_pressed_tests.rs"]
mod tests;
