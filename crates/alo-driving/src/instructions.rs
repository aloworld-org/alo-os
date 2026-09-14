//! The instructions a model is shown before it is asked anything, and which of
//! them a grade was earned under —
//! [ADR 0034](../../../docs/decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md).
//!
//! Every grade in the catalogue until 2026-09-14 was earned under one set of
//! instructions, whose only example request goes through the read door; the
//! failures read in tasks 12 to 14 were changes sent through that door. ADR 0034
//! adds a second set that shows one example per door, and keeps the first
//! unchanged. A grade names the set it was earned under by the SHA-256 of its
//! text, so two grades under different instructions are never read as one
//! measurement.

use crate::exercise::HOW_TO_ANSWER;

/// The instructions shown one example per door (ADR 0034, decision 1).
pub const ONE_EXAMPLE_PER_DOOR: &str = "\
You are talking to a computer, not to a person. Answer with one line of JSON and
nothing else: no explanation, no code fence, no second line.

For a verb that only answers a question, the line has this shape:

{\"format\":1,\"asks\":{\"read\":{\"verb\":\"NAME\",\"given\":[{\"named\":\"ARGUMENT\",\"is\":VALUE}]}}}

For a verb that changes something, the line has this shape:

{\"format\":1,\"asks\":{\"propose\":{\"verb\":\"NAME\",\"given\":[{\"named\":\"ARGUMENT\",\"is\":VALUE}]}}}

Each verb below says which it is: (read) or (propose). VALUE is text in quotes,
or a whole number with no quotes. Give every argument the verb takes and no
others. A path is always a full path.

These are the only verbs there are:";

/// **Which instructions a model was shown.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instructions {
    /// [`HOW_TO_ANSWER`], unchanged: one example, through the read door. Every
    /// grade earned before ADR 0034 was earned under these.
    AsFirstWritten,
    /// [`ONE_EXAMPLE_PER_DOOR`]: an example through each door.
    OneExamplePerDoor,
}

impl Instructions {
    /// Both, in the order they were written.
    pub const ALL: [Self; 2] = [Self::AsFirstWritten, Self::OneExamplePerDoor];

    /// The text a model is shown.
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            Self::AsFirstWritten => HOW_TO_ANSWER,
            Self::OneExamplePerDoor => ONE_EXAMPLE_PER_DOOR,
        }
    }

    /// How a harness and a report name them.
    #[must_use]
    pub fn named(self) -> &'static str {
        match self {
            Self::AsFirstWritten => "as-first-written",
            Self::OneExamplePerDoor => "one-example-per-door",
        }
    }

    /// **The SHA-256 of their text**, in sixty-four lowercase hexadecimal
    /// characters — what a grade records beside itself.
    #[must_use]
    pub fn digest(self) -> String {
        ring::digest::digest(&ring::digest::SHA256, self.text().as_bytes())
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    /// The instructions a digest names, if it names any this crate has.
    #[must_use]
    pub fn of_digest(digest: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|instructions| instructions.digest() == digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The instructions every existing grade was earned under are unchanged**:
    /// held to the digest they had when ADR 0034 was taken, so an edit to
    /// `HOW_TO_ANSWER` fails here rather than silently re-labelling a grade.
    #[test]
    fn the_first_instructions_are_the_ones_every_existing_grade_was_earned_under() {
        assert_eq!(
            Instructions::AsFirstWritten.digest(),
            FIRST_WRITTEN_DIGEST,
            "HOW_TO_ANSWER changed; a grade earned under it now names text that no longer exists"
        );
    }

    /// The digest the first instructions had on 2026-09-14.
    const FIRST_WRITTEN_DIGEST: &str = include_str!("instructions_first_written.sha256");

    /// **The second set shows each door once, and the first shows only `read`.**
    #[test]
    fn one_example_per_door_names_each_door_once_and_the_first_names_one() {
        let count =
            |text: &str, door: &str| text.matches(&format!("\"asks\":{{\"{door}\"")).count();
        assert_eq!(count(Instructions::AsFirstWritten.text(), "read"), 1);
        assert_eq!(count(Instructions::AsFirstWritten.text(), "propose"), 0);
        assert_eq!(count(Instructions::OneExamplePerDoor.text(), "read"), 1);
        assert_eq!(count(Instructions::OneExamplePerDoor.text(), "propose"), 1);
    }

    /// Two sets, two digests, and each digest finds its own set.
    #[test]
    fn a_digest_names_one_set_of_instructions() {
        let [first, second] = Instructions::ALL.map(Instructions::digest);
        assert_ne!(first, second);
        assert_eq!(first.len(), 64);
        assert!(
            first
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
        assert_eq!(
            Instructions::of_digest(&first),
            Some(Instructions::AsFirstWritten)
        );
        assert_eq!(
            Instructions::of_digest(&second),
            Some(Instructions::OneExamplePerDoor)
        );
        assert_eq!(Instructions::of_digest(&"0".repeat(64)), None);
        assert!(
            Instructions::OneExamplePerDoor
                .text()
                .ends_with("These are the only verbs there are:")
        );
    }
}
