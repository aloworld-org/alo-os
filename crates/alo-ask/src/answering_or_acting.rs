//! Whether a sentence a person typed wants an answer or wants something done.
//!
//! [ADR 0124](../../../docs/decisions/0124-one-command-asks-alo-from-the-terminal-and-asking-means-acting.md)
//! term 4, in the owner's words: *it should be smart enough to know if the
//! question needs answering or acting.* So there is no flag, no subcommand and
//! no quoting convention — a person types a sentence at their own prompt and
//! working out which it is belongs here.
//!
//! # Nothing in this file protects anybody, and that is the design
//!
//! The ADR is explicit that the classifier is **not** the guard: *the classifier
//! is not what protects the person — the approval is.* A question read as a
//! request becomes a proposal nobody approved, which costs reading one sentence;
//! a request read as a question costs asking again. **Nothing in either row
//! runs.** So this may be wrong, and the promise *reads answer, changes wait*
//! does not rest on it being right.
//!
//! What the ADR forbids, and what nothing here may ever grow into: *acting
//! without a proposal because the classification was confident. Confidence is
//! not approval, and there is no threshold above which it becomes one.*
//!
//! # Why the verdict is parsed here and asked elsewhere
//!
//! This module holds the question and reads the answer; it does not reach a
//! model. That keeps the part with a rule in it testable without one — a model
//! is the thing a test cannot pin, and a verdict that only a machine with a
//! runtime can check is a rule nobody can hold.

/// What a sentence turned out to want.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatItWants {
    /// An answer. The existing road, unchanged.
    Answering,
    /// Something done to this machine, which means a proposal and an approval.
    Acting,
    /// Neither reading is clear.
    ///
    /// **Asked rather than guessed.** The standing rule is *never a silent
    /// fallback*, and choosing between two readings without saying so is a
    /// silent choice between them. ADR 0124 records this as the lane's reading
    /// of what it left open, and marks it as the owner's to overturn.
    CannotTell,
}

/// What this word means when a model writes it alone on a line.
const ACTING: &str = "ACT";
/// The same, for a sentence that wants an answer.
const ANSWERING: &str = "ANSWER";

/// What to put to a model about a sentence somebody typed.
///
/// One word back, because a verdict wrapped in prose is a verdict somebody has
/// to parse out of prose, and the parsing is where a classifier starts agreeing
/// with itself.
#[must_use]
pub fn the_question_about(typed: &str) -> String {
    let one_line: String = typed
        .chars()
        .map(|it| if it == '\n' || it == '\r' { ' ' } else { it })
        .collect();
    format!(
        "Somebody typed this at a shell prompt, to an assistant that can both answer questions \
         and change things on this machine. Answer with exactly one word and nothing else: \
         `{ACTING}` if they are asking for something to be done to the machine, or `{ANSWERING}` \
         if they are asking to be told something. What they typed is: {one_line}"
    )
}

/// What a model's reply says the sentence wants.
///
/// **A reply that is not one of the two words is [`WhatItWants::CannotTell`]**,
/// including a reply that contains both and one that explains itself at length.
/// A model asked for one word and answering with a paragraph has not answered
/// the question that was put, and reading a verdict out of its prose would be
/// this file guessing on the model's behalf.
#[must_use]
pub fn what_the_reply_says(reply: &str) -> WhatItWants {
    let said = reply.trim().trim_matches('`').trim().to_uppercase();
    let acting = said == ACTING;
    let answering = said == ANSWERING;
    match (acting, answering) {
        (true, false) => WhatItWants::Acting,
        (false, true) => WhatItWants::Answering,
        _ => WhatItWants::CannotTell,
    }
}

#[cfg(test)]
#[path = "answering_or_acting_tests.rs"]
mod tests;
