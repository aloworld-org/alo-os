//! The approval surface: one change in front of the person, two answers, and
//! the changes that arrived behind it.
//!
//! ADR 0001 §5: *any change to the machine is proposed with a sentence
//! describing it and waits for one approval. What a person approves is that
//! sentence.* `alo_approving` is that rule's model — the sentence, the one
//! answer, and every refusal worded — and this screen is where it meets a
//! person's eyes and keys.
//!
//! # It words nothing
//!
//! The sentence is `alo_approving::Asked::sentence`, which `alo-approving`
//! makes from a change that is really waiting and nothing else, and it reaches
//! this screen only through `alo_approving::Compositor` (`crate::approval_shown`).
//! The two answers are `Asked::approve_said` and `Asked::no_said`. A refusal is
//! `NotAsked::said` or `NotAnswered::said`, in whoever refused's own words.
//! Nothing here summarises, shortens, heads or rewraps the sentence into a
//! different claim — `tests/approval_source.rs` holds these files to writing no
//! words at all.
//!
//! # It decides nothing
//!
//! Every answer goes back through `alo_approving::Approving`, once per press,
//! and **a second answer to the same question is refused there rather than
//! here**: this screen does not guard against it, so the rule that one approval
//! is one execution is `alo-approving`'s and `alo-capability`'s to keep, and is
//! kept even when this screen is wrong.
//!
//! # Nothing is preselected, and nothing proceeds on silence
//!
//! A question goes up with neither answer selected, so Enter on a fresh
//! question — held down from the last dialogue, or pressed for something else —
//! does nothing. Tab selects; Enter gives the selected answer. There is no
//! timer, no default and no answer on close: a screen left alone answers
//! nothing, and a question left alone lapses in `alo-capability`, which carries
//! nothing out.
//!
//! # No is the whole answer
//!
//! Declining takes no reason and shows nothing afterwards: there are no letters
//! on this surface (`crate::ApprovalKey`) and no sentence after a *no*, because
//! a system that recorded a reason would be a system that asked for one.
//!
//! # One question at a time, in the order they arrived
//!
//! A change that arrives while another is open is kept behind it
//! (`crate::approval_queue`), never put up over it, so nobody answers a
//! question they did not read. The next goes up when the one in front is
//! answered — and when the answer was refused, only after the person has read
//! why.
//!
//! # And it is never a road to more than one change
//!
//! No *approve all*, no *remember this*, no timer. If a person wants an agent
//! to do a class of thing without asking, that is a grant, made in
//! `alo-picking`, and nothing here reaches one.
//!
//! # One screen per turn
//!
//! A question's number means something only inside the turn that asked it, so
//! a screen is made when a turn begins and dropped when it ends, and every call
//! is handed that turn.

use std::time::SystemTime;

use alo_approving::{Answered, Approving, Asked, Asks, NotAnswered, NotAsked};
use alo_capability::{Grants, ProposalId};
use alo_strings::Said;
use alo_turn::Turning;

use crate::ApprovalKey;
use crate::approval_queue::ApprovalQueue;
use crate::approval_shown::ApprovalShown;

/// One of the two answers a person may give. There is no third.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalAnswer {
    /// Say no. Nothing happens, and nobody is asked why.
    No,
    /// Approve: the change is carried out, once.
    Approve,
}

impl ApprovalAnswer {
    /// Both answers, in the order a person reads them: *no* first, and the
    /// answer that changes the machine last, so reaching it takes the longest
    /// road through the surface.
    pub const IN_READING_ORDER: [Self; 2] = [Self::No, Self::Approve];
}

/// What the approval surface draws now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalShows<'a> {
    /// No question and no sentence: the surface draws nothing at all.
    Nothing,
    /// One change waiting for this person's answer.
    Question {
        /// The question as `alo-approving` made it.
        asked: &'a Asked,
        /// The answer selected, or [`None`] — which is how every question
        /// goes up.
        selected: Option<ApprovalAnswer>,
    },
    /// Why an answer carried nothing out, or why a question that arrived could
    /// not be put up, in the words of whoever refused. Nothing to answer; Enter
    /// acknowledges it.
    Refusal(&'a Said),
}

/// What one call did that the host has to know about.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ApprovalOutcome {
    /// What an answer did, when this call carried one to `alo-approving`.
    ///
    /// A carried change is the verb's own answer handed on whole; a refused
    /// one is also on the screen, as [`ApprovalShows::Refusal`], unless it was
    /// [`NotAnswered::NothingToAnswer`] — an answer with nothing in front of
    /// the person, which there is nothing to show about.
    pub answered: Option<Answered>,
    /// Questions that could not be put to the person because there is nowhere
    /// to show them, for the host's log. They were not answered, and they
    /// lapse in the turn that asked them.
    pub nowhere_to_show: Vec<NotAsked>,
}

/// The approval surface for one turn.
#[derive(Debug)]
pub struct ApprovalScreen {
    /// The question in front of the person, as `alo-approving` keeps it.
    approving: Approving,
    /// What the compositor was handed.
    shown: ApprovalShown,
    /// The changes that arrived behind it.
    queue: ApprovalQueue,
    /// The answer selected, if the person has selected one.
    selected: Option<ApprovalAnswer>,
    /// A refusal the person has not yet acknowledged.
    refusal: Option<Said>,
}

impl ApprovalScreen {
    /// The surface for a turn, on a compositor drawing an output.
    #[must_use]
    pub fn on_an_output() -> Self {
        Self::with(ApprovalShown::on_an_output())
    }

    /// The surface for a turn on a compositor with nothing to show it on. Every
    /// question is refused in `alo-approving`'s words, and none is answered.
    #[must_use]
    pub fn with_no_output() -> Self {
        Self::with(ApprovalShown::with_no_output())
    }

    /// A surface with nothing in front of anybody.
    fn with(shown: ApprovalShown) -> Self {
        Self {
            approving: Approving::nothing_to_answer(),
            shown,
            queue: ApprovalQueue::default(),
            selected: None,
            refusal: None,
        }
    }

    /// A change the turn proposed has arrived for this person.
    ///
    /// Put up at once when nothing is open; kept behind the open question or
    /// refusal otherwise. A number already in front of the person or already
    /// kept is one question, and nothing happens.
    pub fn arrived(
        &mut self,
        turning: &Turning<'_, '_>,
        id: ProposalId,
        now: SystemTime,
    ) -> ApprovalOutcome {
        if self.approving.asking() == Some(id) || self.queue.holds(id) {
            return ApprovalOutcome::default();
        }
        self.queue.arrived(id);
        self.opened_next(turning, now)
    }

    /// One key press, and what it did.
    ///
    /// Tab and Shift+Tab select an answer on an open question. Enter gives the
    /// selected answer — and does nothing while none is selected — or
    /// acknowledges a refusal, which puts up the next question. Every other
    /// key does nothing.
    pub fn pressed(
        &mut self,
        key: ApprovalKey,
        turning: &mut Turning<'_, '_>,
        grants: &Grants,
        now: SystemTime,
    ) -> ApprovalOutcome {
        match key {
            ApprovalKey::NextAnswer | ApprovalKey::PreviousAnswer
                if self.shown.asked().is_some() =>
            {
                self.selected = Some(moved(self.selected, key == ApprovalKey::NextAnswer));
                ApprovalOutcome::default()
            }
            ApprovalKey::Choose if self.refusal.is_some() => {
                self.refusal = None;
                self.opened_next(turning, now)
            }
            ApprovalKey::Choose => match self.selected {
                Some(answer) if self.shown.asked().is_some() => {
                    self.answer(answer, turning, grants, now)
                }
                _ => ApprovalOutcome::default(),
            },
            // Escape answers no — never yes, and never nothing
            // (`alo_access::leaving`). A refusal being read is acknowledged by
            // it the way Enter acknowledges one, because there is no proposal
            // behind a refusal to decline.
            ApprovalKey::Decline if self.refusal.is_some() => {
                self.refusal = None;
                self.opened_next(turning, now)
            }
            ApprovalKey::Decline if self.shown.asked().is_some() => {
                self.answer(ApprovalAnswer::No, turning, grants, now)
            }
            ApprovalKey::NextAnswer
            | ApprovalKey::PreviousAnswer
            | ApprovalKey::Decline
            | ApprovalKey::Nothing => ApprovalOutcome::default(),
        }
    }

    /// Give one answer to whatever is in front of the person.
    ///
    /// The road a pointer takes, and the one [`ApprovalScreen::pressed`] takes
    /// once an answer is selected. It goes to `alo_approving::Approving` every
    /// time it is called — **it does not check first whether anything is
    /// open**, so a second answer to one question reaches `alo-approving` and
    /// is refused there, as [`NotAnswered::NothingToAnswer`].
    pub fn answer(
        &mut self,
        answer: ApprovalAnswer,
        turning: &mut Turning<'_, '_>,
        grants: &Grants,
        now: SystemTime,
    ) -> ApprovalOutcome {
        let answered = match answer {
            ApprovalAnswer::Approve => self.approving.approve(turning, grants, now),
            ApprovalAnswer::No => self.approving.decline(turning, now),
        };
        let mut outcome = match &answered {
            Answered::Refused(NotAnswered::NothingToAnswer) => ApprovalOutcome::default(),
            Answered::Refused(why) => {
                self.shown.taken_down();
                self.selected = None;
                if why.is_the_end_of_the_turn() {
                    self.queue.forgotten();
                }
                self.refusal = Some(why.said(turning.strings()));
                ApprovalOutcome::default()
            }
            Answered::Carried(_) | Answered::Declined => {
                self.shown.taken_down();
                self.selected = None;
                self.opened_next(turning, now)
            }
        };
        outcome.answered = Some(answered);
        outcome
    }

    /// Read the open question off the turn again, as a frame is about to be
    /// drawn.
    ///
    /// Answers nothing. A question still waiting stays exactly as it is, with
    /// the same answer selected; one that has lapsed, or was answered another
    /// way, comes down and `alo-approving`'s sentence for why goes up in its
    /// place — so nobody is offered an answer the machine will refuse.
    pub fn looked_again(&mut self, turning: &Turning<'_, '_>, now: SystemTime) -> ApprovalOutcome {
        let Some(id) = self.approving.asking() else {
            return ApprovalOutcome::default();
        };
        let mut outcome = ApprovalOutcome::default();
        match self.approving.ask(Some(&mut self.shown), turning, id, now) {
            Asks::Asked(_) => {}
            Asks::Refused(why) => {
                self.selected = None;
                self.shown.taken_down();
                self.refused_to_ask(why, turning, &mut outcome);
                if self.refusal.is_none() {
                    let next = self.opened_next(turning, now);
                    outcome.nowhere_to_show.extend(next.nowhere_to_show);
                }
            }
        }
        outcome
    }

    /// What the surface draws now.
    #[must_use]
    pub fn shows(&self) -> ApprovalShows<'_> {
        if let Some(said) = &self.refusal {
            return ApprovalShows::Refusal(said);
        }
        match self.shown.asked() {
            Some(asked) => ApprovalShows::Question {
                asked,
                selected: self.selected,
            },
            None => ApprovalShows::Nothing,
        }
    }

    /// Whether anything is in front of the person — a question or a refusal —
    /// so the host routes keys here rather than to an application.
    #[must_use]
    pub fn is_open(&self) -> bool {
        !matches!(self.shows(), ApprovalShows::Nothing)
    }

    /// How many changes arrived behind the one in front of the person.
    #[must_use]
    pub fn waiting_behind(&self) -> usize {
        self.queue.len()
    }

    /// Put up the change that arrived first, if nothing is open.
    ///
    /// A change that has lapsed or was answered since it arrived becomes the
    /// refusal on the screen and stops here, so the person reads it before the
    /// next question goes up. With nowhere to show anything, every kept change
    /// is refused into the outcome for the host's log.
    fn opened_next(&mut self, turning: &Turning<'_, '_>, now: SystemTime) -> ApprovalOutcome {
        let mut outcome = ApprovalOutcome::default();
        while self.refusal.is_none() && self.shown.asked().is_none() {
            let Some(id) = self.queue.next() else {
                break;
            };
            match self.approving.ask(Some(&mut self.shown), turning, id, now) {
                Asks::Asked(_) => self.selected = None,
                Asks::Refused(why) => self.refused_to_ask(why, turning, &mut outcome),
            }
        }
        outcome
    }

    /// A question that could not be put up: into the log when there is nowhere
    /// to show anything, onto the screen otherwise.
    fn refused_to_ask(
        &mut self,
        why: NotAsked,
        turning: &Turning<'_, '_>,
        outcome: &mut ApprovalOutcome,
    ) {
        if why.is_nowhere_to_show_it() {
            outcome.nowhere_to_show.push(why);
        } else {
            self.refusal = Some(why.said(turning.strings()));
        }
    }
}

/// The selection after one Tab or Shift+Tab: from nothing, the first answer in
/// reading order forwards and the last backwards; otherwise the other answer.
fn moved(selected: Option<ApprovalAnswer>, forwards: bool) -> ApprovalAnswer {
    let [first, last] = ApprovalAnswer::IN_READING_ORDER;
    match selected {
        None if forwards => first,
        None => last,
        Some(answer) if answer == first => last,
        Some(_) => first,
    }
}

#[cfg(test)]
#[path = "approval_screen_tests.rs"]
mod tests;
