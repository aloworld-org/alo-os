//! **A request as the model is shown it and as its road is judged**: the text
//! its fingerprint is taken of (`crate::the_declaration`).
//!
//! What the model is shown: the person's words, the window offered with them
//! (in the words a turn tells it), and what the machine answers at each step.
//! How the road is judged: each step's verb, the values it expects, the places
//! for text, whether it is a look, taken as often as asked, the only control
//! of its kind or a one-line field, the reads aside of it, the other roads, and what is on the
//! machine.
//!
//! **A field left at its default adds nothing**, so a change that adds a field
//! every request leaves empty changes no fingerprint (the owner, 2026-11-20).
//! Not the name a summary calls the request by, which the model never sees.
//!
//! Both structs are taken apart **with no `..`**, so a field added to either
//! does not compile here until somebody decides how it is told.

use std::fmt::Write as _;

use super::{ARequest, Step};

/// One step, on one line.
fn a_step(step: &Step) -> String {
    let Step {
        verb,
        expecting,
        fields,
        answered,
        look,
        again,
        the_only_one,
        one_line,
    } = step;
    let mut line = (*verb).to_owned();
    if !expecting.is_empty() {
        let each: Vec<String> = expecting
            .iter()
            .map(|(argument, is)| format!("{argument}={is}"))
            .collect();
        let _ = write!(line, "; expecting {}", each.join(", "));
    }
    for (argument, place) in fields {
        let _ = write!(line, "; the field {argument} names {place:?}");
    }
    let _ = write!(line, "; answered {answered:?}");
    if *look {
        line.push_str("; a look");
    }
    if *again {
        line.push_str("; as often as asked");
    }
    if *the_only_one {
        line.push_str("; the only control of its kind");
    }
    if *one_line {
        line.push_str("; a one-line field");
    }
    line
}

/// Steps, one a line, under `heading`.
fn steps(heading: &str, steps: &[Step]) -> String {
    let mut said = format!("{heading}:\n");
    for step in steps {
        let _ = writeln!(said, "- {}", a_step(step));
    }
    said
}

impl ARequest {
    /// The request as the model is shown it and as its road is judged, with
    /// nothing for a field left at its default.
    #[must_use]
    pub fn as_shown_and_judged(&self) -> String {
        let Self {
            named: _,
            words,
            road,
            aside,
            other,
            window,
            there,
        } = self;
        let mut said = format!("words: {words}\n");
        if let Some(window) = window {
            let _ = writeln!(said, "window: {}", window.told().trim());
        }
        if !there.is_empty() {
            let each: Vec<String> = there.iter().map(|at| at.display().to_string()).collect();
            let _ = writeln!(said, "on its machine: {}", each.join(", "));
        }
        said.push_str(&steps("road", road));
        if !aside.is_empty() {
            said.push_str(&steps("aside", aside));
        }
        for (at, road) in other.iter().enumerate() {
            said.push_str(&steps(&format!("another road {}", at + 1), road));
        }
        said
    }
}
