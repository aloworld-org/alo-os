//! What comes back when a verb crossed and the door answered: what the
//! machine did, or the number a change waits under.
//!
//! One object, one of two shapes:
//!
//! ```text
//! {"did":{"listed":{"things":[…],"could-not-be-named":0,"cut-short":false}}}
//! {"waits":{"number":7}}
//! ```
//!
//! [`alo_protocol::Done`] is what one of the six file verbs answered, as the
//! local protocol already spells it for an agent on the same machine — every
//! path through the protocol's naming, every bound still saying it was
//! reached — and an agent on another machine is handed exactly the same
//! thing. A change does not answer: it waits for the receiving machine's
//! person, and what crosses back is the number it waits under, so the asking
//! agent can be told that and nothing more.
//!
//! What does *not* come back this way is a refusal. The door's refusals are
//! one word each from [`crate::AtTheDoor`]'s closed list, with a status that
//! says so, because a sentence worded on the receiving machine would be
//! worded in the receiving person's language and the asking person reads
//! their own.

use alo_nearby::NotNearby;
use alo_protocol::Done;
use serde::{Deserialize, Serialize};

/// The most bytes an answer on the wire may be.
///
/// The local protocol's bound on an answer, because it is the same answer: a
/// folder's listing or a file's text, spelt the same way.
pub const AT_MOST_AN_ANSWER: usize = alo_protocol::LONGEST_ANSWER;

/// What the door answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Answered {
    /// A read ran, and this is what the machine found or did.
    Did(Done),
    /// A change was put to the receiving machine's person and waits for them.
    Waits {
        /// The number it waits under, on that machine.
        number: u64,
    },
}

impl Answered {
    /// The bytes that cross back: the object, compactly.
    #[must_use]
    pub fn said(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// An answer the other machine sent, read back and checked for shape.
    ///
    /// # Errors
    ///
    /// [`NotNearby::NotAMessage`] for anything that is not exactly one of the
    /// two objects [`said`](Self::said) writes.
    pub fn read(said: &str) -> Result<Self, NotNearby> {
        serde_json::from_str(said)
            .map_err(|why| NotNearby::NotAMessage(format!("not an answer: {why}")))
    }

    /// What the machine did, if a read ran.
    #[must_use]
    pub const fn did(&self) -> Option<&Done> {
        match self {
            Self::Did(done) => Some(done),
            Self::Waits { .. } => None,
        }
    }

    /// The number a change waits under, if one does.
    #[must_use]
    pub const fn waits(&self) -> Option<u64> {
        match self {
            Self::Did(_) => None,
            Self::Waits { number } => Some(*number),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use alo_protocol::Done;

    use super::Answered;

    /// Both shapes are read back as they were written, and each answers only
    /// the question it is.
    #[test]
    fn both_shapes_are_read_back_as_written() {
        let did = Answered::Did(Done::Read {
            text: "March, 4180.00".to_owned(),
        });
        let read = Answered::read(&did.said()).unwrap();
        assert_eq!(read, did);
        assert!(read.did().is_some());
        assert_eq!(read.waits(), None);

        let waits = Answered::Waits { number: 7 };
        assert_eq!(waits.said(), r#"{"waits":{"number":7}}"#);
        let read = Answered::read(&waits.said()).unwrap();
        assert_eq!(read.waits(), Some(7));
        assert!(read.did().is_none());
    }

    /// Anything that is not one of the two objects is refused.
    #[test]
    fn what_is_not_an_answer_is_refused() {
        for not_one in [
            "",
            "{}",
            "not-granted-there",
            r#"{"waits":{"number":7},"did":{"read":{"text":""}}}"#,
            r#"{"waits":{"number":7,"where":"/a"}}"#,
            r#"{"ran":true}"#,
        ] {
            assert!(
                Answered::read(not_one).is_err(),
                "`{not_one}` was read as an answer"
            );
        }
    }
}
