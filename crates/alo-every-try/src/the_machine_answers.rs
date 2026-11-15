//! **What the machine answers when a step is taken**, as a booted machine at
//! full access would — the script a try's later choices are made against.
//!
//! Every answer is built from the protocol's own types and constructors, the
//! ones `alo-agentd` builds its answers from
//! (`what_a_client_is_told_a_verb_did.rs`, `saying_it_was_done_without_asking.rs`),
//! so the model reads a line in the shape it would read on a machine. **The
//! contents are the script's**: the window the editor shows is the one
//! `docs/quirks/vscodium-is-pressed-and-read-only-when-described-completely.md`
//! measured VSCodium showing — a text entry named after the open file — and
//! not one read off a machine during the try.

use alo_capability::Call;
use alo_protocol::ToAnAgent;
use alo_strings::Strings;

use crate::a_road::Answered;

/// The machine's answer to `call`, taken as a step answered `answered`.
#[must_use]
pub fn answering(answered: &Answered, call: &Call, strings: &Strings) -> ToAnAgent {
    let _ = (answered, call);
    ToAnAgent::Refused(alo_protocol::Wording::of(&strings.say(
        &alo_capability::words::DONE_AT_FULL_ACCESS.key(),
        &alo_strings::Filling::nothing(),
    )))
}

#[cfg(test)]
#[path = "the_machine_answers_tests.rs"]
mod tests;
