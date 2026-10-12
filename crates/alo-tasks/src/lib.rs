//! What a task window holds: the six areas the owner specified, as types.
//!
//! A request through the composer opens a window of its own where a person and
//! the agent work together — `docs/features.md`, 2026-10-12. **It is a chat a
//! person can follow, not a log they have to read**, and that sentence decides
//! more here than it looks: the activity is beside the conversation rather than
//! being it, and the technical log is behind *Details* rather than in front of
//! somebody who only wanted to ask for something.
//!
//! # What is here and what is not
//!
//! Here: the rules that can be decided without a screen. A model change cannot
//! land midway through an action ([`WhichModel`]); Stop has a state in which a
//! person is waiting and nothing has stopped ([`Stopping`]); what a task
//! changed is carried beside the fact that it ended ([`WhatChanged`]).
//!
//! Not here: pixels, and the agent. `alo-shell` draws this and `alo-asking`
//! answers; a crate that did either would be a second place deciding something
//! that already has one.
#![forbid(unsafe_code)]

pub mod activity;
pub mod changed;
pub mod conversation;
pub mod layout;
pub mod measures;
pub mod permissions;
pub mod stopping;
pub mod task;
pub mod the_one_button;
pub mod which_model;

pub use activity::{Act, HowItWent, TheActivity};
pub use changed::WhatChanged;
pub use conversation::{TheConversation, Turn, WhoSaidIt};
pub use layout::{ARow, TheControls, TheRows, TooSmall};
pub use permissions::{Answered, ThePermissions};
pub use stopping::Stopping;
pub use task::{ATask, InThePanel};
pub use the_one_button::TheOneButton;
pub use which_model::{BeforeItChanges, WhichModel};
