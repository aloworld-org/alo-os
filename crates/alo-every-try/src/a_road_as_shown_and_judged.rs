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

use super::{ARequest, Answered, Step};

/// **How a step's answer is written** into the text a fingerprint is taken of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheAnswerWritten {
    /// As Rust prints it (`{:?}`): every field, an empty one included. The
    /// second scheme of fingerprints (`crate::the_declaration::Scheme::WhatIsShown`).
    AsPrinted,
    /// **As the model is sent it** (since 2026-11-21): what the machine did,
    /// as the protocol writes it on the wire, so a field the protocol leaves
    /// out when it is empty is not in it. Found on #1412, where three empty
    /// fields on a thing shown moved the line of every request reading a
    /// window though the model read the same bytes.
    AsSent,
}

/// One step's answer, written `how`.
fn the_answer(answered: &Answered, how: TheAnswerWritten) -> String {
    match (how, answered) {
        (TheAnswerWritten::AsSent, Answered::With(done)) => {
            serde_json::to_string(done).unwrap_or_else(|_| format!("{done:?}"))
        }
        // **The catalogue as the model may be sent it**: what it holds under
        // each kind of work a call may choose, and under everything.
        (TheAnswerWritten::AsSent, Answered::TheCatalogue(catalogue)) => std::iter::once(None)
            .chain(alo_software::Category::EVERY.into_iter().map(Some))
            .map(|kind| {
                let sent = alo_protocol::Done::of_a_catalogue_read(
                    &alo_software::WhatTheCatalogueHolds::under(catalogue, kind),
                );
                serde_json::to_string(&sent).unwrap_or_else(|_| format!("{sent:?}"))
            })
            .collect::<Vec<_>>()
            .join("; "),
        _ => format!("{answered:?}"),
    }
}

/// One step, on one line.
fn a_step(step: &Step, how: TheAnswerWritten) -> String {
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
    let _ = write!(line, "; answered {}", the_answer(answered, how));
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
fn steps(heading: &str, steps: &[Step], how: TheAnswerWritten) -> String {
    let mut said = format!("{heading}:\n");
    for step in steps {
        let _ = writeln!(said, "- {}", a_step(step, how));
    }
    said
}

impl ARequest {
    /// The request as the model is shown it and as its road is judged, with
    /// nothing for a field left at its default, and each answer as sent.
    #[must_use]
    pub fn as_shown_and_judged(&self) -> String {
        self.as_shown_and_judged_with(TheAnswerWritten::AsSent)
    }

    /// The same, each answer written `how`.
    #[must_use]
    pub fn as_shown_and_judged_with(&self, how: TheAnswerWritten) -> String {
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
        said.push_str(&steps("road", road, how));
        if !aside.is_empty() {
            said.push_str(&steps("aside", aside, how));
        }
        for (at, road) in other.iter().enumerate() {
            said.push_str(&steps(&format!("another road {}", at + 1), road, how));
        }
        said
    }
}
