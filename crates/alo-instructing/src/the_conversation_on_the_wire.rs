//! **The conversation so far, as one line between the shell and the agent.**
//!
//! A person's shell hands their words to `alo-agent` over the socket ADR 0109
//! decided, a line at a time (`docs/contracts/asking-socket.md`). A follow-up
//! adds one line after the request:
//!
//! ```text
//! alo-earlier:{"turns":[{"person":"list my invoices"},{"alo":"march.pdf, april.pdf"}],"left-out":0,"cut":false}
//! ```
//!
//! and a first request sends the marker with nothing after it, so an agent
//! that reads the line always finds one where a current shell sent it.
//!
//! # Why here
//!
//! The shell writes it and the agent reads it, and the type both handle is
//! [`TheConversationSoFar`]: one crate holding the shape and both of its ends
//! is what keeps the writer and the reader from agreeing only by accident.
//! This crate opens nothing; it turns a value into a string and back.
//!
//! # What a reader refuses
//!
//! Anything a current shell would not have written: a line that is not JSON, a
//! turn of a kind this does not know, an answer that is not one of the three,
//! and **a conversation over the bound** — the shell holds it to
//! [`TheConversationSoFar::THE_MOST_SENT`] before sending, so one longer did
//! not come from a shell and is not shown to a model. Each refusal is a
//! sentence; nothing is guessed.

use serde_json::{Map, Value, json};

use crate::the_conversation_so_far::{AnEarlierTurn, TheAnswer, TheConversationSoFar};

/// The marker the line begins with.
pub const THE_EARLIER_CONVERSATION: &str = "alo-earlier:";

/// The most a line may be, in bytes, before it is read at all.
///
/// The bound on what was said, six times over for JSON writing a control
/// character as six bytes, and a little for the names around it.
pub const THE_LONGEST_LINE: usize = TheConversationSoFar::THE_MOST_SENT * 6 + 4096;

/// Why a line could not be read as the conversation so far.
///
/// Written out by hand rather than derived, because this crate carries the
/// verbs, a digest and JSON and nothing else (`tests/nothing_behind_these_words.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotTheConversation {
    /// The line does not begin with [`THE_EARLIER_CONVERSATION`].
    NotMarked,
    /// Longer than any shell sends.
    TooLong(usize),
    /// Not the JSON a shell writes.
    NotReadable(String),
}

impl std::fmt::Display for NotTheConversation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotMarked => {
                write!(
                    f,
                    "the line after the request is not the earlier conversation"
                )
            }
            Self::TooLong(bytes) => write!(
                f,
                "the earlier conversation is {bytes} bytes long, which is more than a shell \
                 sends, so it was not read"
            ),
            Self::NotReadable(why) => {
                write!(f, "the earlier conversation could not be read: {why}")
            }
        }
    }
}

impl std::error::Error for NotTheConversation {}

impl TheConversationSoFar {
    /// **The line a shell writes**, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        if self.is_empty() {
            return THE_EARLIER_CONVERSATION.to_owned();
        }
        let turns: Vec<Value> = self
            .turns()
            .iter()
            .map(|turn| match turn {
                AnEarlierTurn::ThePersonSaid(said) => json!({ "person": said }),
                AnEarlierTurn::AloAnswered(said) => json!({ "alo": said }),
                AnEarlierTurn::AloProposed { sentence, answer } => json!({
                    "proposed": sentence,
                    "answer": match answer {
                        TheAnswer::Allowed => "allowed",
                        TheAnswer::Denied => "denied",
                        TheAnswer::NotAnswered => "not-answered",
                    },
                }),
            })
            .collect();
        let line = json!({
            "turns": turns,
            "left-out": self.left_out(),
            "cut": self.was_cut(),
        });
        format!("{THE_EARLIER_CONVERSATION}{line}")
    }

    /// **The line read back**, with or without its newline.
    ///
    /// # Errors
    /// [`NotTheConversation`], for anything a current shell would not write.
    pub fn read(line: &str) -> Result<Self, NotTheConversation> {
        let line = line.trim_end_matches(['\n', '\r']);
        if line.len() > THE_LONGEST_LINE {
            return Err(NotTheConversation::TooLong(line.len()));
        }
        let payload = line
            .strip_prefix(THE_EARLIER_CONVERSATION)
            .ok_or(NotTheConversation::NotMarked)?;
        if payload.trim().is_empty() {
            return Ok(Self::nothing());
        }
        let not = |why: &str| NotTheConversation::NotReadable(why.to_owned());
        let value: Value = serde_json::from_str(payload).map_err(|why| not(&why.to_string()))?;
        let whole = value
            .as_object()
            .ok_or_else(|| not("it is not an object"))?;
        let left_out = whole
            .get("left-out")
            .and_then(Value::as_u64)
            .and_then(|count| usize::try_from(count).ok())
            .ok_or_else(|| not("it does not say how much was left out"))?;
        let cut = whole
            .get("cut")
            .and_then(Value::as_bool)
            .ok_or_else(|| not("it does not say whether a turn was cut"))?;
        let mut turns = Vec::new();
        for turn in whole
            .get("turns")
            .and_then(Value::as_array)
            .ok_or_else(|| not("it has no turns"))?
        {
            turns.push(
                turn.as_object()
                    .and_then(a_turn)
                    .ok_or_else(|| not("a turn is not one a shell writes"))?,
            );
        }
        let said: usize = turns.iter().map(|turn| turn.said().len()).sum();
        if said > Self::THE_MOST_SENT {
            return Err(NotTheConversation::TooLong(said));
        }
        Ok(Self::from_parts(turns, left_out, cut))
    }
}

/// One turn, exactly as a shell writes one, or [`None`].
fn a_turn(turn: &Map<String, Value>) -> Option<AnEarlierTurn> {
    let text = |name: &str| turn.get(name).and_then(Value::as_str).map(str::to_owned);
    match turn.len() {
        1 => text("person")
            .map(AnEarlierTurn::ThePersonSaid)
            .or_else(|| text("alo").map(AnEarlierTurn::AloAnswered)),
        2 => {
            let answer = match turn.get("answer").and_then(Value::as_str)? {
                "allowed" => TheAnswer::Allowed,
                "denied" => TheAnswer::Denied,
                "not-answered" => TheAnswer::NotAnswered,
                _ => return None,
            };
            Some(AnEarlierTurn::AloProposed {
                sentence: text("proposed")?,
                answer,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "the_conversation_on_the_wire_tests.rs"]
mod tests;
