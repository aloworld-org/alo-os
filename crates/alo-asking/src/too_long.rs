//! The identifiers a service says a question was longer than its model can
//! read with.
//!
//! The owner decided on 2026-10-28 that alo OS puts no limit of its own on a
//! conversation (ADR 0138): the only limit is what the chosen model can read,
//! and when a conversation passes it the person is told so in plain words —
//! never answered from a conversation with its beginning quietly dropped. So
//! the refusal a service sends for it has to be told apart from every other
//! `400`, which until now all read as *something answered, and not with an
//! answer* and would have sent somebody to ask again into the same wall.
//!
//! # Identifiers only, as `crate::ran_out` reads them
//!
//! An identifier a service publishes is compared against the list below and
//! dropped; the message beside it — which names token counts and models in the
//! provider's own words — is never kept. What crosses out of here is a `bool`.
//! A service that says it only in a sentence, with no identifier, is read as
//! *nothing usable*, as before: matching a sentence somebody else composed
//! would be guessing in another vendor's language.

use serde_json::Value;

use crate::ran_out::{named_in, plainly};

/// The identifiers that mean the question was longer than the model reads.
///
/// Flattened as `crate::ran_out` flattens them. OpenAI's, which the services
/// copying its error shape send as well.
const LONGER_THAN_IT_READS: [&str; 1] = ["contextlengthexceeded"];

/// Whether this refusal is a service saying the question was longer than its
/// model can read.
pub(crate) fn said_in(body: &str) -> bool {
    let Ok(refusal) = serde_json::from_str::<Value>(body) else {
        return false;
    };
    named_in(&refusal).any(|name| LONGER_THAN_IT_READS.contains(&plainly(name).as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The refusal OpenAI really sends** for a question too long for the
    /// model, in its shape.
    #[test]
    fn a_question_longer_than_the_model_reads_is_read_as_that() {
        let body = r#"{"error":{"message":"This model's maximum context length is 128000 tokens. However, your messages resulted in 130000 tokens.","type":"invalid_request_error","param":"messages","code":"context_length_exceeded"}}"#;
        assert!(said_in(body));
    }

    /// **Every other refusal is not**, including one that only says it in a
    /// sentence: nothing here reads a sentence somebody else wrote.
    #[test]
    fn other_refusals_and_sentences_are_not_read_as_too_long() {
        for body in [
            r#"{"error":{"type":"invalid_request_error","code":"invalid_value"}}"#,
            r#"{"error":{"message":"maximum context length exceeded","type":"BadRequestError"}}"#,
            "not json at all",
        ] {
            assert!(!said_in(body), "{body}");
        }
    }
}
