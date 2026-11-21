//! **Whether alo's model takes the expected road on every try, not once.**
//!
//! Law 9 of `CLAUDE.md` ([ADR 0157](../../../docs/decisions/0157-a-bug-is-caught-by-a-check-its-author-did-not-write.md)):
//! *what the model chooses is tested many times, not once. Any change to what
//! a model is told or offered — its instructions, its verbs, their
//! descriptions — is tried against the real model on a fixed set of requests,
//! several times each, and passes only if every try takes the road expected.*
//! This crate is that check, and `.github/workflows/every-try.yml` runs it.
//!
//! # What a try is
//!
//! A try is **the agent's own loop**, `alo_agent::driving_about`, the function
//! a person's turn runs, with the real instructions
//! (`Instructions::SHOWN_TO_A_TURN`) and the real verbs
//! (`alo_turn::the_verbs_a_whole_machine_offers`). Its question goes to the
//! real model through `alo-asking`'s provider road, held to the same shape a
//! turn's question is (`alo_turn::the_shape_a_provider_is_held_to`). What is
//! different is the **door**: where the daemon would carry a verb out, the
//! door in [`a_try`] compares it with the step the road expects
//! ([`taking_a_step`]) and answers with what a booted machine at full access
//! would say ([`the_machine_answers`]). **Nothing is carried out on any
//! machine**: no file is opened, nothing is typed, nothing is published.
//!
//! The first step that is not the expected one ends the try, and the try is
//! recorded as having gone wrong there ([`WentWrong`]). A request passes only
//! when every one of its tries took the whole road and then answered in words
//! ([`tallied`]).
//!
//! # What it cannot show
//!
//! **The answers are a script.** The model's second choice depends on what the
//! machine said about its first, and here that is what [`the_machine_answers`]
//! writes, through the protocol's own types, rather than what a real VSCodium
//! window showed. A road that passes here can still fail on a machine whose
//! answers differ — the walk (`.github/workflows/walk.yml`) is what checks
//! that. What this shows is narrower and cheaper: **whether the model chooses
//! the expected verb, with the expected arguments, at each step, every time.**
//!
//! The set is [`the_set()`]: versioned, and every verb, argument and application
//! name in it read from the crate that declares it.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod a_road;
pub mod a_try;
pub mod answering_the_file_chooser;
pub mod as_the_machine_names;
pub mod at_the_provider;
pub mod compared_with_its_base;
pub mod counted;
pub mod every_other_verb;
pub mod its_own;
pub mod no_such_control;
pub mod nothing_is_there;
pub mod taking_a_step;
pub mod tallied;
pub mod the_bar;
pub mod the_declaration;
pub mod the_first_fingerprints;
pub mod the_machine_answers;
pub mod the_set;
pub mod went_wrong;
pub mod what_each_has_reached;
pub mod words_for_the_answers;

#[cfg(test)]
mod testing;

pub use a_road::{ARequest, Answered, Step};
pub use a_try::{TheLevel, TheModel, Tried, Went, one_try, one_try_at};
pub use at_the_provider::AtAProvider;
pub use tallied::{Tallied, Verdict, summary, tallying, tallying_at, verdict};
pub use the_set::{NotTheSet, THE_HOME, THE_VERSION, TheSet, the_set};
pub use went_wrong::WentWrong;
