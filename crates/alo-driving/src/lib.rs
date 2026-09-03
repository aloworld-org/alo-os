//! **Whether a model can drive alo OS's verbs, measured rather than claimed.**
//!
//! `docs/features.md` promises at v0.01: *★ The catalogue says whether a model
//! can drive the verbs, not just whether it will run*, and *it is measured by
//! us, not claimed by the publisher*.
//! [ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md), in its
//! *since it was accepted* section, is why: an agent turn asks a model to emit
//! a typed verb call with valid arguments several times over, and that is
//! exactly what small models are worst at. Sentences they manage. Structure
//! they lose. A model that runs beautifully on a laptop and cannot emit a valid
//! call is useless as an agent, and a catalogue that knew only about memory
//! would recommend it.
//!
//! This crate is the measurement. The grade it produces is
//! [`alo_models::Driving`], which lives with the catalogue because that is
//! where a machine reads it — a machine deciding what to offer somebody must be
//! able to answer with no model loaded, no socket open and no verbs declared.
//!
//! # The journey
//!
//! | | |
//! |---|---|
//! | [`Exercise`] | One request put to a model, and the verb a correct answer calls |
//! | [`Exercises`] | The fixed ten, bound to one machine's verbs — and the only road to a score |
//! | [`Attempt`] | What a model produced for one exercise, put through the daemon's door |
//! | [`Outcome`] | What became of it: five ways to fail, and one to drive |
//! | [`Measured`] | A whole run, and the grade it earns |
//!
//! # Three decisions, and each is in the file that makes it
//!
//! **An answer is scored through the daemon's own door**, by
//! [`alo_protocol::FromAnAgent`] and [`alo_capability::Verbs::call`], because
//! anything else would be a second parser for one syntax and a score about a
//! format nothing uses. [`attempt`] has it, and has what that costs.
//!
//! **An exercise names the verb a correct answer calls**, not only *a* real
//! verb, because a set scored on well-formedness alone is cleared by a model
//! that answers `list_folder` ten times. [`exercise`] has it.
//!
//! **A run that skipped an exercise is not a measurement.** [`measured`] has
//! it, together with the bar — nine attempts in ten.
//!
//! # What this crate does not do
//!
//! **It asks nothing.** No client, no socket, no runtime, not even behind a
//! feature: it hands out a prompt and scores what comes back. Whoever runs the
//! measurement puts the prompt to a model through `alo-asking`, which is the
//! one crate in this workspace that speaks to anything, and brings the text
//! here. `alo-answering`'s argument, one crate on: a promise about the absence
//! of code is worth what the code around it is small enough to prove.
//!
//! **It is not run on a machine.** Nothing reads this at runtime and nothing
//! depends on it. A measurement is made by whoever is adding a catalogue entry,
//! and what ships is the grade they wrote down — which is why
//! `data/catalogue.toml` says how, and why every entry in it today says
//! `not-measured`.
//!
//! **It says nothing to anybody.** No vocabulary, no `words.rs`, and nothing
//! here has a `Display` except the two refusals of our own set and our own run.
//! The sentence a person reads about all of this is
//! `alo_models::NoAgentHere`, in their own language, made where the refusal is
//! made.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod attempt;
pub mod exercise;
pub mod exercises;
pub mod measured;

#[cfg(test)]
mod testing;

pub use attempt::{Attempt, Outcome};
pub use exercise::{Exercise, HOW_TO_ANSWER, prompt};
pub use exercises::{Exercises, NotComparable, THE_SET};
pub use measured::{Measured, NotMeasurable, RELIABLY, SOMETIMES};
