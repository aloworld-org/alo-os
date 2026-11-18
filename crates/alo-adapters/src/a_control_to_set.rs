//! **Whether the slider or number box a move names is in the window, before
//! anybody is asked** (2026-11-17).
//!
//! A move is looked for again when it runs ([`crate::Setting::set`]), and that
//! stays: the window can change between the card and the approval. What this
//! adds is one read **before the card**, so a person is never asked to approve
//! *move “Zoom slider” in Loupe to 200* when the window has no control called
//! that. Asked so, the approval was spent on a move that could not happen, and
//! the refusal arrived only after it.
//!
//! **It refuses only what it saw cannot happen**: no control of that name in a
//! window it read whole. A window it could not read, or read only in part, or
//! with two of that name, is left to the move itself, which says exactly why
//! when it runs. Nothing is moved, pressed or typed here — the walk reads.

use alo_strings::{Said, Strings};

use crate::accessibility_tree::AccessibilityTree;
use crate::not_set::NotSet;
use crate::walking::{Whose, walk};

/// What the move would say for `name` in `application` — [`NotSet::NoSuchControl`],
/// in its own words — where `tree`, read whole, shows no slider or number box
/// of that exact name; [`None`] wherever that is not known.
#[must_use]
pub fn not_there_to_set(
    tree: &dyn AccessibilityTree,
    application: &str,
    name: &str,
    value: i64,
    strings: &Strings,
) -> Option<Said> {
    let walked = walk(tree, Whose::Application(application), None).ok()?;
    if walked.shown.not_all_read() {
        return None;
    }
    let there = walked.numbered.iter().any(|found| found.name == name);
    (!there).then(|| NotSet::NoSuchControl.said(application, name, value, strings))
}
