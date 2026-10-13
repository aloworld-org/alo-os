//! **The agent.** A person's words, a model asked what to do about them, and
//! the machine doing it.
//!
//! Every part of this existed before this crate and nothing joined them:
//! `alo-agentd` answers an agent and decides every step correctly,
//! `alo_turn::Turning` is those decisions, `alo-instructing` writes what a
//! model is shown, `alo-protocol` carries both halves, `alo-capability` holds
//! the verbs. **Nothing was the agent.** A person could type a request into the
//! task window and it opened with nothing behind it —
//! `ROADMAP.md`'s launch list says so in its own words: *the composer is drawn
//! and takes no typing; nothing joins it to the agent*.
//!
//! This is the joining, and it is deliberately small: a loop, a bound on it,
//! five ways it can end, and one trait for the door. Everything it could decide
//! is decided somewhere else.
//!
//! # It is its own process, and that is not an implementation detail
//!
//! `docs/contracts/daemon-protocol.md` has two doors and says why a message
//! cannot cross them: *if one door took both, the side that proposed a change
//! could approve it*, and ADR 0001 §5 — one approval, one execution, given by a
//! person — would be true of the capability model and false of the socket in
//! front of it. So the loop does not live in the compositor beside the task
//! window: the window is the person's surface and reaches the person's door,
//! and this reaches the agent's. [`TheDoor`] is typed so that there is no
//! method here an approval could be sent through.
//!
//! # What it still cannot do
//!
//! [`AtTheDaemonsSocket`] is the real door; the loop is tested against one that
//! answers from a list.
//!
//! **Nothing here has driven a real model through more than one step.** Every
//! grade in `data/catalogue.toml` was earned on single-step exercises — ask,
//! answer, done — and `gpt-4o-mini` scoring ten of ten on 2026-10-13 was ten of
//! those. Whether any model finishes a job of five rounds is unmeasured, and
//! `alo_instructing::a_turn_in_progress` is what makes measuring it possible.
//! The loop is tested against a door that answers from a list, which proves the
//! order of the steps and nothing about a model.
#![forbid(unsafe_code)]

pub mod at_the_daemons_socket;
pub mod driving;
pub mod how_far;
pub mod the_door;
pub mod what_happened;

pub use at_the_daemons_socket::AtTheDaemonsSocket;
pub use driving::driving;
pub use how_far::HowFar;
pub use the_door::{NotAnswered, TheDoor};
pub use what_happened::WhatHappened;
