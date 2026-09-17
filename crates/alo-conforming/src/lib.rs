//! **EN 301 549's clauses that apply to this shell, each held to the evidence
//! that meets it.**
//!
//! Task 4 of `docs/autonomy/v0-5-access-and-language-plan.md`.
//! `docs/features.md` sets the bar for v1: *procurement asks for the report, not
//! the intention.* This is what the report will be written from.
//!
//! | | |
//! |---|---|
//! | [`THE_CLAUSES`] | every clause that applies, and where it stands |
//! | [`Standing`], [`Evidence`], [`Waiting`] | met by a test, not yet because of a task, or not applicable and why |
//! | [`Checked`] | whether a person has read the clause against the standard's own text |
//! | [`held`], [`Held`], [`Finding`] | the list checked against this repository, not against another list |
//!
//! # No clause may be met by a sentence
//!
//! [`Standing::Met`] has nowhere to put one: it carries a crate, a file and the
//! name of a test, and `tests/every_clause_that_is_met_names_a_test_that_exists.rs`
//! reads this repository and refuses any that names a test nobody wrote. *Not
//! yet* carries a task number and a plan, both checked the same way. A clause
//! that is not applicable carries the reason it is not.
//!
//! That is the acceptance this crate exists for, and it is the difference
//! between a conformance file and a wish: **every green row here is a test that
//! ran in the gate this morning.**
//!
//! # And it does not claim conformance
//!
//! It cannot: `crate::standard` says what this list is not — not the standard's
//! text, not a report, and **not yet read against the published document by
//! anybody**. Every clause carries that admission and [`Held`] counts it.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod clause;
pub mod every_clause;
pub mod holding;
pub mod standard;
pub mod standing;

pub use clause::Clause;
pub use every_clause::THE_CLAUSES;
pub use holding::{Finding, Held, held};
pub use standard::{THE_PLAN, THE_STANDARD, THE_VERSION};
pub use standing::{Checked, Evidence, Standing, Waiting};
