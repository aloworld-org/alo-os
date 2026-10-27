//! **A model's answer to the person**, in the shape a turn's reply is held to.
//!
//! A turn's reply is held to a shape — by the pinned runtime's grammar and by a
//! provider's strict schema (ADR 0032, ADR 0107) — and until 2026-10-27 that
//! shape had `read` and `propose` in it and nothing else. A request that asked
//! for no change then had no way to end: the loop in `alo_agent::driving` took
//! a line that was not JSON as the answer, and the shape would not let one be
//! written. ADR 0133 adds this door, and
//! [`crate::EVERY_DOOR_AND_AN_ANSWER`] tells a model about it.
//!
//! # Not a line of the protocol, on purpose
//!
//! `alo-protocol` is what an agent says to the **machine**, and the machine has
//! nothing to do with an answer: it is said to the person, by the agent, after
//! the machine has answered what it was asked. So this is read here, by the one
//! crate that told the model about it, and the daemon and every client written
//! against its protocol are untouched.

use serde_json::Value;

/// The door a model answers the person through, beside `read` and `propose`
/// inside `asks`. `alo-models`' envelopes name the same word, and
/// `alo-driving`'s tests hold the two to each other.
pub const THE_ANSWER_DOOR: &str = "answer";

/// The protocol version an answer carries, as every line a model writes does.
const THE_FORMAT: u64 = 1;

/// **The answer in a line, if the line is one.**
///
/// `{"format":1,"asks":{"answer":{"text":"…"}}}` and nothing else: the right
/// version, `answer` as the only door, and `text` as words with something in
/// them. Anything else is [`None`], and is read by the protocol as before — so
/// a line that is not an answer is never mistaken for one.
#[must_use]
pub fn the_answer_in(line: &str) -> Option<String> {
    let value: Value = serde_json::from_str(line.trim()).ok()?;
    let whole = value.as_object()?;
    if whole.len() != 2 || whole.get("format")?.as_u64()? != THE_FORMAT {
        return None;
    }
    let asks = whole.get("asks")?.as_object()?;
    if asks.len() != 1 {
        return None;
    }
    let answer = asks.get(THE_ANSWER_DOOR)?.as_object()?;
    if answer.len() != 1 {
        return None;
    }
    let text = answer.get("text")?.as_str()?.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::the_answer_in;

    /// **The shape a model is told is the shape this reads**: the example line
    /// in the instructions a turn shows is itself an answer.
    #[test]
    fn the_example_a_model_is_shown_is_read_as_an_answer() {
        let shown = crate::Instructions::SHOWN_TO_A_TURN.text();
        let example = shown
            .lines()
            .find(|line| line.contains("\"answer\""))
            .expect("a turn is shown the answer's shape");
        assert_eq!(
            the_answer_in(example).as_deref(),
            Some("YOUR ANSWER, IN PLAIN WORDS")
        );
    }

    #[test]
    fn an_answer_is_read_as_its_words() {
        assert_eq!(
            the_answer_in(
                r#"{"format":1,"asks":{"answer":{"text":"Two files: march.pdf and notes.txt."}}}"#
            ),
            Some("Two files: march.pdf and notes.txt.".to_owned())
        );
    }

    /// **The refusal path**: nothing that is not exactly an answer is taken as
    /// one, so a request can never be swallowed as words and lost.
    #[test]
    fn nothing_else_is_taken_for_an_answer() {
        for not_one in [
            r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[]}}}"#,
            r#"{"format":1,"asks":{"propose":{"verb":"rename_file","given":[]}}}"#,
            r#"{"format":2,"asks":{"answer":{"text":"from a newer alo"}}}"#,
            r#"{"asks":{"answer":{"text":"no version"}}}"#,
            r#"{"format":1,"asks":{"answer":{"text":"   "}}}"#,
            r#"{"format":1,"asks":{"answer":{"text":"a"},"read":{}}}"#,
            r#"{"format":1,"asks":{"answer":{"text":"a","verb":"x"}}}"#,
            r#"{"format":1,"asks":{"answer":{"text":"a"}},"extra":1}"#,
            r#"{"format":1,"asks":{"answer":{"words":"a"}}}"#,
            "It is in your Documents folder.",
            "",
        ] {
            assert_eq!(the_answer_in(not_one), None, "{not_one}");
        }
    }
}
