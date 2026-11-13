//! **Publishing a folder to the person's GitHub Pages site** — Demo 2's second
//! road: the person asks alo, and alo runs `git` and GitHub's own `gh` itself,
//! on its own command road (ADR 0149).
//!
//! The owner, 2026-11-13: *"We can add it but we need no shortcuts."* The
//! road through the code editor (`docs/autonomy/demo-two-alo-inside-applications-plan.md`)
//! stays; this is beside it. The owner's four decisions that day are what this
//! crate is:
//!
//! 1. **alo runs git on its own command road**, never typing into the person's
//!    terminal window — [`Runs`] is that road, handed in.
//! 2. **At full access it does not ask first; at *ask me each time* the
//!    sentence carries the warning**, and the line afterwards says it went to
//!    the internet and Put back cannot recall it — [`words::SENTENCE`] and
//!    [`what_the_person_is_told`]. The level is read by the daemon, as for
//!    every change; at the sealed box it is refused in
//!    [`words::THE_SEALED_BOX_CANNOT_PUBLISH`].
//! 3. **The person signs in with GitHub's own `gh`**, themselves — this crate
//!    never asks for or reads a password; it asks `gh` who is signed in, and
//!    refuses when it is somebody other than the account the sentence named.
//! 4. **A typed action whose sentence the machine writes** —
//!    [`verbs::PUBLISH_TO_GITHUB_PAGES`], on the list a model is offered.
//!
//! | | |
//! |---|---|
//! | [`verbs`] | the verb, what is checked before the person is asked, and what an authority carries |
//! | [`Account`] | a GitHub account's name, by GitHub's own rule |
//! | [`Publication`], [`Published`] | what was asked, and what came of it |
//! | [`the_line()`] | the one command a publication runs |
//! | [`publish()`] | that command run once, while the indicator shows it |
//! | [`what_the_person_is_told`] | the line afterwards, on every surface |
//! | [`NotPublished`] | why not, in words a person can act on |

mod account;
mod not_published;
mod publication;
mod publishing;
mod runs;
pub mod the_line;
pub mod verbs;
mod what_came_of_it;
pub mod what_the_person_is_told;
pub mod words;

pub use account::Account;
pub use not_published::NotPublished;
pub use publication::{Publication, Published};
pub use publishing::{WHERE_IT_GOES, publish, the_departure};
pub use runs::{GH, GITHUB, Runs, Through};
pub use the_line::{quoted, the_line};
pub use verbs::{
    PUBLISH_TO_GITHUB_PAGES, approved, changes_nothing_on_this_computer, checked_before_asking,
};
pub use what_came_of_it::what_came_of_it;
pub use what_the_person_is_told::{
    WhatWasPublished, published_after_your_ok, published_without_asking,
};
pub use words::publishing_words;
