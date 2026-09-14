//! The instructions a model is shown before it is asked anything, which of them
//! a grade was earned under, and which of them a turn shows —
//! [ADR 0034](../../../docs/decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md)
//! and
//! [ADR 0037](../../../docs/decisions/0037-the-words-a-turn-shows-a-model-are-the-products-own.md).
//!
//! Every grade in the catalogue until 2026-09-14 was earned under one set of
//! instructions, whose only example request goes through the read door; the
//! failures read in tasks 12 to 14 were changes sent through that door. ADR 0034
//! adds a second set that shows one example per door, and keeps the first
//! unchanged. A grade names the set it was earned under by the SHA-256 of its
//! text, so two grades under different instructions are never read as one
//! measurement.

/// What every model is told before it is asked anything.
///
/// It describes the message `alo_protocol::FromAnAgent` reads and nothing else.
/// The two keys and the two doors are the whole of the envelope; a model that
/// cannot reproduce those cannot reproduce a verb call either.
pub const HOW_TO_ANSWER: &str = "\
You are talking to a computer, not to a person. Answer with one line of JSON and
nothing else: no explanation, no code fence, no second line.

The line has this shape:

{\"format\":1,\"asks\":{\"read\":{\"verb\":\"NAME\",\"given\":[{\"named\":\"ARGUMENT\",\"is\":VALUE}]}}}

Use \"read\" for a verb that only answers a question, and \"propose\" for a verb
that changes something; each verb below says which it is. VALUE is text in
quotes, or a whole number with no quotes. Give every argument the verb takes and
no others. A path is always a full path.

These are the only verbs there are:";

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

    /// **The set an agent turn shows a model** (ADR 0037, decision 2).
    ///
    /// The choice is a measured one rather than a taste: on an Apple M3 with
    /// 8 GB the same weights drove the verbs 80 of 80 under these and 71 of 80
    /// under [`Self::AsFirstWritten`], through the pinned runtime, in the
    /// envelope — and 80 of 80 against 65 of 80 through `llama.cpp`'s own
    /// server. A turn shown any other text is a turn no grade is about, so what
    /// a turn shows is named here, once, and taken by
    /// [`crate::shown_to_a_turn`].
    pub const SHOWN_TO_A_TURN: Self = Self::OneExamplePerDoor;

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
    /// The text moved crate on 2026-09-14 (ADR 0037) and this is what says it
    /// moved without changing.
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

    /// **The set a turn shows is one this crate has**, and it is the one the
    /// measurements of 2026-09-14 named. A turn shown text with no digest in
    /// the catalogue is a turn no grade is about.
    #[test]
    fn what_a_turn_is_shown_is_named_and_measured() {
        assert_eq!(
            Instructions::SHOWN_TO_A_TURN,
            Instructions::OneExamplePerDoor
        );
        assert_eq!(
            Instructions::of_digest(&Instructions::SHOWN_TO_A_TURN.digest()),
            Some(Instructions::SHOWN_TO_A_TURN)
        );
    }

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
