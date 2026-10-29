//! **The conversation so far, as one line between the shell and the agent.**
//!
//! A person's shell hands their words to `alo-agent` over the socket ADR 0109
//! decided, a line at a time (`docs/contracts/asking-socket.md`). A follow-up
//! adds one line after the request:
//!
//! ```text
//! alo-earlier:{"turns":[{"person":"list my invoices"},{"alo":"march.pdf, april.pdf"}]}
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
//! # The one bound, and where it comes from
//!
//! **Not a limit on the conversation** — the owner decided alo shortens none
//! (ADR 0137). It is a limit on what a privileged reader will take off a
//! socket, and it is set from the only place a conversation can go: inside a
//! question to `alo-agentd`, which refuses any request over
//! [`THE_MOST_A_QUESTION_IS`] bytes (`docs/contracts/daemon-protocol.md`,
//! *Bounds*). That is about a quarter of a million tokens, more than the
//! context of any model alo OS can be pointed at today. A conversation that
//! could never be asked is not worth reading, and
//! [`TheConversationSoFar::fits_in_a_question`] lets the shell tell the person
//! so before it sends anything.
//!
//! # What a reader refuses
//!
//! Anything a current shell would not have written: a line that is not JSON, a
//! turn of a kind this does not know, an answer that is not one of the three,
//! and a line longer than [`THE_LONGEST_LINE`]. Each refusal is a sentence;
//! nothing is guessed.

use serde_json::{Map, Value, json};

use crate::the_conversation_so_far::{AnEarlierTurn, TheAnswer, TheConversationSoFar};

/// The marker the line begins with.
pub const THE_EARLIER_CONVERSATION: &str = "alo-earlier:";

/// **The most a question to `alo-agentd` may be**, in bytes: its own bound on a
/// request, `alo_protocol::LONGEST`, spelt here for the reason the
/// socket's path is spelt in two places — this crate depends on the verbs and
/// a digest and nothing else. A test in `alo-agent` holds the two equal.
pub const THE_MOST_A_QUESTION_IS: usize = 1024 * 1024;

/// What a question needs besides the conversation and the request: the
/// instructions, every verb, the folders and the envelope around them.
///
/// Generous on purpose — it is measured against a whole question of a few
/// tens of kilobytes — because a conversation the shell let through and the
/// daemon then refused would be refused in the daemon's words about envelopes
/// rather than in a sentence about the conversation.
pub const ROOM_FOR_THE_REST_OF_A_QUESTION: usize = 128 * 1024;

/// The most a line may be, in bytes, before it is read at all — a question's
/// bound, since the conversation can only travel inside one.
pub const THE_LONGEST_LINE: usize = THE_MOST_A_QUESTION_IS;

/// Why a line could not be read as the conversation so far.
///
/// Written out by hand rather than derived, because this crate carries the
/// verbs, a digest and JSON and nothing else (`tests/nothing_behind_these_words.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotTheConversation {
    /// The line does not begin with [`THE_EARLIER_CONVERSATION`].
    NotMarked,
    /// Longer than any question could carry.
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
                "the earlier conversation is {bytes} bytes long, which is more than a question \
                 to this machine may carry, so it was not read"
            ),
            Self::NotReadable(why) => {
                write!(f, "the earlier conversation could not be read: {why}")
            }
        }
    }
}

impl std::error::Error for NotTheConversation {}

impl TheConversationSoFar {
    /// **Whether this conversation and `request` fit in one question** to
    /// `alo-agentd`, counted as they will travel: written as JSON, with room
    /// for the instructions and the verbs.
    ///
    /// The shell asks before it sends, so a conversation too long for this
    /// machine to ask about is said to be too long rather than refused later
    /// in words about envelopes.
    #[must_use]
    pub fn fits_in_a_question(&self, request: &str) -> bool {
        let travels = |text: &str| Value::from(text).to_string().len();
        travels(&self.told()) + travels(request) + ROOM_FOR_THE_REST_OF_A_QUESTION
            <= THE_MOST_A_QUESTION_IS
    }

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
        format!("{THE_EARLIER_CONVERSATION}{}", json!({ "turns": turns }))
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
            .filter(|whole| whole.len() == 1)
            .ok_or_else(|| not("it is not an object holding the turns alone"))?;
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
        Ok(Self::of(turns))
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
