//! **Follow-ups, measured on their own**, apart from the fixed ten.
//!
//! [ADR 0136](../../../docs/decisions/0136-a-follow-up-is-sent-with-the-conversation-so-far.md):
//! a reply typed in a task window is sent to a model with the conversation so
//! far, and a grade earned on a first request is not proof about one. The
//! owner chose to measure follow-ups separately for each model, and keep the
//! result apart from the first-request grade, so that every grade already in
//! the catalogue stays exactly what it was.
//!
//! # What a follow-up exercise is
//!
//! An earlier conversation, a follow-up that **only makes sense with it** —
//! *and read the April one* — and the call a correct answer makes. Each one
//! needs the conversation to be read: the verb alone is not enough, so a
//! follow-up also names what the call must be **about** (`refers_to`), and an
//! answer that calls the right verb about the wrong file is scored as
//! [`FollowedUp::TheWrongOne`] rather than as having driven.
//!
//! **The text is the product's.** A follow-up is shown
//! `alo_instructing::shown_to_a_model_after`, the one function a real turn's
//! question is built by, with the same instructions a first request is — so a
//! follow-up grade names the same digest and is about the same words.

use alo_capability::Verbs;
use alo_instructing::{
    AnEarlierTurn, Instructions, TheAnswer, TheConversationSoFar, WhatItMayReach,
    shown_to_a_model_after,
};

use crate::attempt::{Attempt, Outcome};
use crate::exercise::Exercise;

/// One thing said before a follow-up, written in the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Said {
    /// Anna said this.
    ByThePerson(&'static str),
    /// alo answered this.
    ByAlo(&'static str),
    /// alo proposed this change, and this is what became of it.
    Proposed(&'static str, TheAnswer),
}

/// One follow-up: what came before, what was asked next, and what a correct
/// answer calls and is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AFollowUp {
    /// The request, its name and the verb it calls.
    exercise: Exercise,
    /// What was said before it, oldest first.
    earlier: &'static [Said],
    /// What a correct call must name — a path, an application, a choice —
    /// each of which only the earlier conversation supplies.
    refers_to: &'static [&'static str],
}

impl AFollowUp {
    /// Declare one.
    #[must_use]
    pub const fn after(
        earlier: &'static [Said],
        exercise: Exercise,
        refers_to: &'static [&'static str],
    ) -> Self {
        Self {
            exercise,
            earlier,
            refers_to,
        }
    }

    /// Its short stable name.
    #[must_use]
    pub fn named(&self) -> &'static str {
        self.exercise.named()
    }

    /// The conversation before it, as a turn would carry it.
    #[must_use]
    pub fn the_conversation_so_far(&self) -> TheConversationSoFar {
        TheConversationSoFar::of(
            self.earlier
                .iter()
                .map(|said| match *said {
                    Said::ByThePerson(said) => AnEarlierTurn::ThePersonSaid(said.to_owned()),
                    Said::ByAlo(said) => AnEarlierTurn::AloAnswered(said.to_owned()),
                    Said::Proposed(sentence, answer) => AnEarlierTurn::AloProposed {
                        sentence: sentence.to_owned(),
                        answer,
                    },
                })
                .collect(),
        )
    }

    /// **The whole prompt**, under these instructions: the text a real
    /// follow-up's question is built from.
    ///
    /// `not_said` for what it may reach, for the fixed ten's reason: an
    /// exercise has no person and no grants, and the conversation is the only
    /// thing that should tell the model where to look.
    #[must_use]
    pub fn prompt_under(&self, instructions: Instructions, verbs: &Verbs) -> String {
        shown_to_a_model_after(
            instructions,
            verbs,
            &WhatItMayReach::not_said(),
            &self.the_conversation_so_far(),
            self.exercise.asked(),
        )
    }

    /// **What became of an answer**: scored through the daemon's own door as
    /// any exercise is, and then held to what it had to be about.
    #[must_use]
    pub fn scored(&self, verbs: &Verbs, produced: &str) -> FollowedUp {
        let attempt = Attempt::at(&self.exercise, verbs, produced);
        match attempt.outcome() {
            Outcome::Drove if self.refers_to.iter().all(|what| produced.contains(what)) => {
                FollowedUp::Drove
            }
            Outcome::Drove => FollowedUp::TheWrongOne,
            other => FollowedUp::DidNotDrive(other.clone()),
        }
    }
}

/// What became of one follow-up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FollowedUp {
    /// The call alo OS would act on, about what the conversation meant.
    Drove,
    /// The right verb, about something the conversation did not mean — the
    /// failure a follow-up exists to catch.
    TheWrongOne,
    /// Not the right call at all, for the fixed set's reasons.
    DidNotDrive(Outcome),
}

impl FollowedUp {
    /// Whether this one drove.
    #[must_use]
    pub const fn drove(&self) -> bool {
        matches!(self, Self::Drove)
    }
}

/// **The five**, each needing the conversation before it.
///
/// Anna, `/home/anna/…` and `org.alo.Writer` as in the fixed ten, so a model is
/// measured on following a conversation and not on tracking new names.
pub const THE_FOLLOW_UPS: [AFollowUp; 5] = [
    AFollowUp::after(
        &[
            Said::ByThePerson("list what is in /home/anna/Invoices"),
            Said::ByAlo("There are two files in /home/anna/Invoices: march.pdf and april.pdf."),
        ],
        Exercise::asking("the-april-one", "and read the April one", "read_file"),
        &["/home/anna/Invoices/april.pdf"],
    ),
    AFollowUp::after(
        &[
            Said::ByThePerson("rename /home/anna/Invoices/scan001.pdf to march.pdf"),
            Said::Proposed(
                "Rename scan001.pdf in /home/anna/Invoices to march.pdf",
                TheAnswer::Allowed,
            ),
        ],
        Exercise::asking(
            "the-same-again",
            "now do the same with scan002.pdf, calling it april.pdf",
            "rename_file",
        ),
        &["/home/anna/Invoices/scan002.pdf", "april.pdf"],
    ),
    AFollowUp::after(
        &[
            Said::ByThePerson("move /home/anna/Invoices/march.pdf to /home/anna/Archive"),
            Said::Proposed(
                "Move march.pdf from /home/anna/Invoices to /home/anna/Archive",
                TheAnswer::Denied,
            ),
        ],
        Exercise::asking(
            "somewhere-else",
            "no, put it in /home/anna/Paid instead",
            "move_file",
        ),
        &["/home/anna/Invoices/march.pdf", "/home/anna/Paid"],
    ),
    AFollowUp::after(
        &[
            Said::ByThePerson("start org.alo.Writer"),
            Said::Proposed("Open Writer (org.alo.Writer)", TheAnswer::Allowed),
        ],
        Exercise::asking(
            "that-window",
            "now put it on the left half of the screen",
            "arrange_application",
        ),
        &["org.alo.Writer", "left_half"],
    ),
    AFollowUp::after(
        &[
            Said::ByThePerson("what is in /home/anna/Archive"),
            Said::ByAlo("Three folders: 2024, 2025 and taxes."),
        ],
        Exercise::asking(
            "in-that-one",
            "look for anything called october in the taxes one, twenty at most",
            "find_in_folder",
        ),
        &["/home/anna/Archive/taxes", "october"],
    ),
];

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on a follow-up that is not there is the failure being reported"
)]
#[path = "follow_ups_tests.rs"]
mod tests;
