//! What is said beside an answer, and in what order.
//!
//! Two provenance lines, composed in one place so that a process cannot print
//! one and forget the other:
//!
//! 1. **where the answer came from** — the answer's own
//!    `alo_models::InferenceSource`, which no answer can be built without;
//! 2. **that this is not where the person chose**, when it is not —
//!    `alo_choosing::Picked::not_the_place_you_chose`, which answers nothing
//!    when they chose a model on this machine.
//!
//! # Why this is a function and not two lines in `main.rs`
//!
//! **Because the answered path cannot be reached by a test on a machine with
//! no model runtime**, which is every machine in CI.
//! `tests/what_a_person_who_typed_it_sees.rs` runs the real binary and can
//! reach every refusal; it cannot reach the line after an answer arrives. So
//! the second line would have been a caller nothing checked, in a crate whose
//! own header says the decisions live in modules and `main.rs` holds the order
//! — which is the arrangement that makes this testable at all.
//!
//! *The first line had exactly that coverage until 2026-10-18, and nobody had
//! said so.* It is in here now too, so the order of the two is a thing a test
//! reads rather than a thing a reader hopes.
//!
//! # Nothing is offered and nothing is refused
//!
//! [ADR 0008](../../../docs/decisions/0008-where-inference-happens.md) forbids
//! a silent fallback, and an offer to ask somewhere else — from a command named
//! for the place it asks — is that same substitution arriving as a suggestion.
//! So this says what happened and stops. The owner decided on 2026-10-18 that
//! whether alo may use another model or API when the chosen one is unavailable
//! is **a setting the person sets**; that setting does not exist yet, and both
//! sentences here are true either way.

use alo_choosing::Picked;
use alo_models::InferenceSource;
use alo_strings::{Said, Strings};

/// What a person is told beside an answer, in the order they read it.
///
/// Always at least one line: where the answer came from. A second when the
/// person chose somewhere else, and that second line is the one
/// [ADR 0008](../../../docs/decisions/0008-where-inference-happens.md) asks
/// for.
///
/// **Where first, then that where is not what they asked for.** The order is
/// the point rather than a preference: *on this machine* alone reads as
/// confirmation, which is how the fault of 2026-10-18 was invisible, and the
/// second line is what turns it into a statement about their choice.
/// # Why it takes the source rather than the answer
///
/// `alo_asking::Answer::new` is `pub(crate)`, deliberately — an answer is
/// evidence that something answered, so nothing outside that crate may make
/// one. Taking `alo_asking::Answer::source`, which is public, keeps the first
/// line **derived here** rather than handed in by a caller who might derive it
/// differently, and leaves this testable without a model runtime.
#[must_use]
pub fn beside_an_answer(
    source: &InferenceSource,
    chosen: Option<&Picked>,
    strings: &Strings,
) -> Vec<Said> {
    let mut lines = vec![source.said(strings)];
    if let Some(not_there) =
        chosen.and_then(|chosen| chosen.not_the_place_you_chose(source, strings))
    {
        lines.push(not_there);
    }
    lines
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, or an index past a length the test just \
              asserted, is the failure being reported"
)]
#[path = "beside_an_answer_tests.rs"]
mod tests;
