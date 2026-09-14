//! What an agent's question wants back: words, or its own next request.
//!
//! An agent asks a model two kinds of thing. One is about the world — *may the
//! tenant sublet?* — and is answered in prose. The other is the agent asking
//! what to ask this machine for next, and its answer must be a line of this
//! protocol. [ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md)
//! decided that the pinned runtime is asked the second **held to the envelope**
//! and never the first, so the daemon has to be told which one it is carrying —
//! and only the agent that composed the question knows.
//!
//! # Why this is not a place, and not a way in
//!
//! `ask` names no place, and still does not: where a question is answered is
//! the person's (ADR 0008), and nothing here reaches it. This says what shape
//! the answer should come back in, which is the agent's own business about its
//! own question. And it lets nothing through: the answer returns to the agent
//! as a model's words either way, and is a request only when the agent sends it
//! as one — read by this crate and validated by `alo-capability` like every
//! other line.
//!
//! # Additive on the wire
//!
//! Absent means [`Answered::InWords`], so every `ask` a client has ever written
//! still means what it meant, and one written in words is written exactly as
//! before — the field is left out rather than spelled.

use serde::{Deserialize, Serialize};

/// What an agent's question wants back.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Answered {
    /// An answer in words. What every `ask` meant before this existed, and what
    /// one without the field still means.
    #[default]
    InWords,
    /// The agent's next request, as a line of this protocol. The pinned runtime
    /// is asked for it held to the envelope; every other place, freely.
    AsTheNextRequest,
}

impl Answered {
    /// Whether this is the answer in words, which is left off the wire.
    #[must_use]
    pub fn is_in_words(&self) -> bool {
        matches!(self, Self::InWords)
    }
}
