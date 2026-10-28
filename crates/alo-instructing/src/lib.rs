//! **The words a model is shown before it is asked for a request.**
//!
//! How to answer, every verb the machine has in the verb's own words, and the
//! request last. One text, built in one place, for the two things that must
//! never be shown different words: the measurement that grades a model
//! (`alo-driving`) and the agent turn that asks one for its next request.
//!
//! # Why this is a crate of its own
//!
//! Until 2026-09-14 this text lived inside `alo-driving`, the measurement
//! harness, and **nothing in the product composed any text at all**:
//! `alo_turn::Turning::asking_for_the_next_request` takes what a model is shown
//! as a string from whoever calls it, so the words reaching a model on a shipped
//! machine were the caller's and no grade was about them.
//! [ADR 0034](../../../docs/decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md)'s
//! second cost names that exactly — *if a turn is shown different instructions
//! from the ones its model was graded under, the grade says nothing about that
//! turn* — and
//! [ADR 0037](../../../docs/decisions/0037-the-words-a-turn-shows-a-model-are-the-products-own.md)
//! is the decision that the words are the product's, kept here, where the
//! daemon can take them without the catalogue, a runtime client or a network
//! behind them.
//!
//! | | |
//! |---|---|
//! | [`Instructions`] | Which set of how-to-answer a model was shown, and the digest a grade names it by |
//! | [`shown_to_a_model`] | The whole text: the instructions, the verbs, the request |
//! | [`shown_to_a_turn`] | The same, under the set a turn shows ([`Instructions::SHOWN_TO_A_TURN`]) |
//! | [`TheConversationSoFar`] | What was said before a follow-up, held to its bound, how a model is told it, and the line it travels on (ADR 0136) |
//! | [`TheConversationSoFar`] | What was said before a follow-up, held to its bound, and how a model is told it (ADR 0136) |
//!
//! # What this crate does not do
//!
//! **It asks nothing and reads nothing.** It depends on the verb registry and
//! on SHA-256, and on nothing else — no catalogue, no runtime, no socket, no
//! file. `tests/crates_this_crate_carries.rs` holds that to the manifest rather
//! than to this sentence.
//!
//! **It scores nothing.** What becomes of an answer is `alo-driving`'s, through
//! the daemon's own door.
//!
//! **It is in English**, and that is a limit rather than a decision. The text is
//! read by a model and not by a person, so it is not an `alo_strings::Word` and
//! this crate declares no vocabulary — but it follows that a grade says how a
//! model drives the verbs *when it is asked in English*, and that a person whose
//! machine runs in Latvian is served by a turn asking in English.
//! `docs/quirks.md` records it. Measuring, and asking, in twenty-four languages
//! is a real question and it is not this one.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod a_turn_in_progress;
pub mod an_answer;
pub mod answering_in;
pub mod instructions;
pub mod shown;
pub mod the_conversation_on_the_wire;
pub mod the_conversation_so_far;
pub mod the_language_asked_in;
pub mod verb_as_told;
pub mod what_it_may_reach;

#[cfg(test)]
mod testing;

pub use a_turn_in_progress::{
    AStep, ATurnInProgress, shown_in_progress, shown_to_a_turn_in_progress,
};
pub use an_answer::{THE_ANSWER_DOOR, the_answer_in};
pub use answering_in::{answering_in, shown_to_a_turn_answering_in_the_language_asked_in};
pub use instructions::{
    AND_AN_ANSWER, AND_YOU_MAY_LOOK_FIRST, EVERY_DOOR_AND_AN_ANSWER, HOW_TO_ANSWER, Instructions,
    ONE_EXAMPLE_PER_DOOR, SEVERAL_STEPS,
};
pub use shown::{shown_to_a_model, shown_to_a_model_after, shown_to_a_turn};
pub use the_conversation_on_the_wire::{
    NotTheConversation, THE_EARLIER_CONVERSATION, THE_LONGEST_LINE, THE_MOST_A_QUESTION_IS,
};
pub use the_conversation_so_far::{AnEarlierTurn, TheAnswer, TheConversationSoFar};
pub use the_language_asked_in::the_language_of;
pub use what_it_may_reach::WhatItMayReach;
