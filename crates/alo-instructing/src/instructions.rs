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

/// **The one paragraph [`SEVERAL_STEPS`] adds to [`ONE_EXAMPLE_PER_DOOR`].**
///
/// Held separately so the difference between the two sets is a value rather
/// than a diff somebody reads: a test below asserts that taking this out of
/// `SEVERAL_STEPS` gives `ONE_EXAMPLE_PER_DOOR` back, which is what makes a
/// measured difference between them attributable to these words and nothing
/// else.
pub const AND_YOU_MAY_LOOK_FIRST: &str = "\
You may send more than one line, one at a time. The word after \"asks\" is
always read or propose and never the name of a verb. When you do not know what
is there, send a read line first; you will be shown its answer, and then you
send the next line. Send a propose line only once you know.

Never name a file you have not seen in an answer. A verb reaches what is there,
so a name you guessed is refused and the turn is wasted.

";

/// **Everything [`ONE_EXAMPLE_PER_DOOR`] says, and that a turn may take several
/// steps.**
///
/// Written out in full rather than composed, because what a model is shown
/// should be readable in one place — and held to
/// [`ONE_EXAMPLE_PER_DOOR`] plus [`AND_YOU_MAY_LOOK_FIRST`] by a test, so the
/// two cannot drift and a difference in a grade is attributable.
///
/// # What it fixes, measured
///
/// On 2026-10-13, under [`Instructions::OneExamplePerDoor`], a request that needed a
/// look first — *find whichever invoice is for March and add `-final` to its
/// name*, over a folder holding `march.pdf` and `april.pdf` — was answered by
/// **both** `gpt-4o-mini` and `gpt-4o` with a proposal to rename a file that
/// did not exist: `march_invoice.pdf` and `invoice_march.pdf`. Neither listed
/// the folder; neither read anything at all. The machine refused both in its
/// own words — *there is nothing at … — a verb reaches what is there, so name
/// something that exists* — so nothing was damaged and nothing was done.
///
/// Under these instructions both models searched the folder first, were shown
/// `march.pdf`, proposed renaming **that**, and the file moved.
/// `crates/alo-agentd/tests/the_agent_really_does_the_work.rs` is the run.
///
/// # And what it costs, also measured
///
/// **It is worse at a single step, which is why it is not what a turn shows.**
/// The fixed ten, three rounds, `gpt-4o-mini` at OpenAI on 2026-10-13:
///
/// | instructions | drove | grade |
/// |---|---|---|
/// | `one-example-per-door` | 30 of 30 | `reliably` |
/// | `several-steps`, as first written | 24 of 30 | `sometimes` |
/// | `several-steps`, reworded | 26 of 30 | `sometimes` |
///
/// Every failure was the same kind — `NotAMessage(NotReadable)`, the verb's
/// name put where the door belongs, as in
/// `{"format":1,"asks":{"find_in_folder":{…}}}`. The reword added *the word
/// after "asks" is always read or propose and never the name of a verb*, which
/// halved them and did not end them.
///
/// So **two wordings have now cost a grade** that
/// [`Instructions::OneExamplePerDoor`] holds, and this set is kept for what it
/// measured rather than because it is ready: a paragraph about taking several
/// turns, placed among the rules about shape, competes with them. Whether the
/// right answer is a third wording, the guidance moved out of the first round
/// entirely, or a change somewhere else altogether is open — and the
/// alternative worth weighing is that a proposal naming a path that is not
/// there could be refused when it is **proposed** rather than when it is
/// approved, which would let a model correct itself with no instruction
/// change at all and no grade touched.
pub const SEVERAL_STEPS: &str = "\
You are talking to a computer, not to a person. Answer with one line of JSON and
nothing else: no explanation, no code fence, no second line.

For a verb that only answers a question, the line has this shape:

{\"format\":1,\"asks\":{\"read\":{\"verb\":\"NAME\",\"given\":[{\"named\":\"ARGUMENT\",\"is\":VALUE}]}}}

For a verb that changes something, the line has this shape:

{\"format\":1,\"asks\":{\"propose\":{\"verb\":\"NAME\",\"given\":[{\"named\":\"ARGUMENT\",\"is\":VALUE}]}}}

Each verb below says which it is: (read) or (propose). VALUE is text in quotes,
or a whole number with no quotes. Give every argument the verb takes and no
others. A path is always a full path.

You may send more than one line, one at a time. The word after \"asks\" is
always read or propose and never the name of a verb. When you do not know what
is there, send a read line first; you will be shown its answer, and then you
send the next line. Send a propose line only once you know.

Never name a file you have not seen in an answer. A verb reaches what is there,
so a name you guessed is refused and the turn is wasted.

These are the only verbs there are:";

/// **The one paragraph [`ANSWERS_IN_WORDS`] adds to [`ONE_EXAMPLE_PER_DOOR`].**
///
/// Held separately for [`AND_YOU_MAY_LOOK_FIRST`]'s reason: a test asserts that
/// taking it out gives [`ONE_EXAMPLE_PER_DOOR`] back, so a measured difference
/// between the two is these words and nothing else.
pub const AND_ANSWER_WHEN_YOU_KNOW: &str = "\
When what you have been shown already answers the request, and the request
asks for no change, do not ask for anything more: answer the person instead,
in plain words, as one line that is not JSON. Never propose a change the
request did not ask for.

";

/// **Everything [`ONE_EXAMPLE_PER_DOOR`] says, and that a question is answered
/// in words.**
///
/// # What it fixes, measured
///
/// A turn ends with an answer only when the model writes a line that is not
/// JSON (`alo_agent::driving`), and [`ONE_EXAMPLE_PER_DOOR`] tells it to write
/// nothing but JSON — so a request that asks for no change has no way to end.
/// On booted machines on 2026-10-25 and 2026-10-27, *list my documents folder*
/// ended in proposals to install Krita and GIMP. On 2026-10-27,
/// `crates/alo-agentd/tests/a_question_is_answered_in_words.rs` put six such
/// requests to `gpt-4o` under [`ONE_EXAMPLE_PER_DOOR`] and **none** was
/// answered: the model read what it needed, then read it again until the turn
/// ran out, or proposed a rename or an archive nobody asked for.
///
/// Written out in full rather than composed, for [`SEVERAL_STEPS`]' reason, and
/// held to [`ONE_EXAMPLE_PER_DOOR`] plus [`AND_ANSWER_WHEN_YOU_KNOW`] by a test.
///
/// **Not yet what a turn shows.** Every local model's grade for a turn was
/// earned under [`ONE_EXAMPLE_PER_DOOR`]'s digest (`alo_models`'
/// `THE_WORDS_A_TURN_SHOWS`), so showing this instead takes the agent from each
/// of them until it is graded again.
pub const ANSWERS_IN_WORDS: &str = "\
You are talking to a computer, not to a person. Answer with one line of JSON and
nothing else: no explanation, no code fence, no second line.

For a verb that only answers a question, the line has this shape:

{\"format\":1,\"asks\":{\"read\":{\"verb\":\"NAME\",\"given\":[{\"named\":\"ARGUMENT\",\"is\":VALUE}]}}}

For a verb that changes something, the line has this shape:

{\"format\":1,\"asks\":{\"propose\":{\"verb\":\"NAME\",\"given\":[{\"named\":\"ARGUMENT\",\"is\":VALUE}]}}}

Each verb below says which it is: (read) or (propose). VALUE is text in quotes,
or a whole number with no quotes. Give every argument the verb takes and no
others. A path is always a full path.

When what you have been shown already answers the request, and the request
asks for no change, do not ask for anything more: answer the person instead,
in plain words, as one line that is not JSON. Never propose a change the
request did not ask for.

These are the only verbs there are:";

/// **Which instructions a model was shown.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instructions {
    /// [`HOW_TO_ANSWER`], unchanged: one example, through the read door. Every
    /// grade earned before ADR 0034 was earned under these.
    AsFirstWritten,
    /// [`ONE_EXAMPLE_PER_DOOR`]: an example through each door.
    OneExamplePerDoor,
    /// [`SEVERAL_STEPS`]: the same, and that a turn may take several of them.
    ///
    /// **Not yet what a turn shows.** `Self::SHOWN_TO_A_TURN` is a measured
    /// choice and this has not been measured against the fixed ten, so naming
    /// it there before it has been would make every grade a statement about
    /// text nobody has graded under.
    SeveralSteps,
    /// [`ANSWERS_IN_WORDS`]: the second set, and that a question is answered
    /// in words.
    ///
    /// **Not yet what a turn shows**, for the reason [`ANSWERS_IN_WORDS`]
    /// gives: every local model's grade for a turn is under another digest.
    AnswersInWords,
}

impl Instructions {
    /// Every set, in the order they were written.
    pub const ALL: [Self; 4] = [
        Self::AsFirstWritten,
        Self::OneExamplePerDoor,
        Self::SeveralSteps,
        Self::AnswersInWords,
    ];

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
            Self::SeveralSteps => SEVERAL_STEPS,
            Self::AnswersInWords => ANSWERS_IN_WORDS,
        }
    }

    /// How a harness and a report name them.
    #[must_use]
    pub fn named(self) -> &'static str {
        match self {
            Self::AsFirstWritten => "as-first-written",
            Self::OneExamplePerDoor => "one-example-per-door",
            Self::SeveralSteps => "several-steps",
            Self::AnswersInWords => "answers-in-words",
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

    /// **The third set is the second plus one paragraph, and nothing else.**
    ///
    /// Taking [`AND_YOU_MAY_LOOK_FIRST`] out of [`SEVERAL_STEPS`] gives
    /// [`ONE_EXAMPLE_PER_DOOR`] back. That is what makes a measured difference
    /// between the two **attributable**: without it, a third set written out in
    /// full could gain a reworded sentence anywhere and a better grade would be
    /// a mystery with two explanations.
    #[test]
    fn the_third_set_is_the_second_plus_one_paragraph() {
        assert!(
            SEVERAL_STEPS.contains(AND_YOU_MAY_LOOK_FIRST),
            "the paragraph the third set is supposed to add is not in it"
        );
        assert_eq!(
            SEVERAL_STEPS.replace(AND_YOU_MAY_LOOK_FIRST, ""),
            ONE_EXAMPLE_PER_DOOR,
            "the third set differs from the second by more than that paragraph"
        );
    }

    /// **The fourth set is the second plus one paragraph, and nothing else**,
    /// for the third set's reason above.
    #[test]
    fn the_fourth_set_is_the_second_plus_one_paragraph() {
        assert!(
            ANSWERS_IN_WORDS.contains(AND_ANSWER_WHEN_YOU_KNOW),
            "the paragraph the fourth set is supposed to add is not in it"
        );
        assert_eq!(
            ANSWERS_IN_WORDS.replace(AND_ANSWER_WHEN_YOU_KNOW, ""),
            ONE_EXAMPLE_PER_DOOR,
            "the fourth set differs from the second by more than that paragraph"
        );
    }

    /// **And it is not what a turn shows, until it has been measured.**
    ///
    /// `SHOWN_TO_A_TURN` is a measured choice (ADR 0037, decision 2). Naming an
    /// unmeasured set there would make every grade a statement about text
    /// nobody graded under, which is the fault this whole enum exists to
    /// prevent. When `several-steps` has been put to the fixed ten, this test
    /// is the one to come back to.
    #[test]
    fn what_a_turn_shows_is_a_set_that_has_been_measured() {
        assert_eq!(
            Instructions::SHOWN_TO_A_TURN,
            Instructions::OneExamplePerDoor
        );
        assert_ne!(
            Instructions::SHOWN_TO_A_TURN,
            Instructions::SeveralSteps,
            "several-steps became what a turn shows; it needs a grade first"
        );
    }

    /// Every set has its own text, its own name and its own digest.
    ///
    /// Three sets make this worth asserting where two did not: two constants
    /// that drifted into being equal would make `of_digest` answer with
    /// whichever came first in `ALL`, and a grade would name a set at random.
    #[test]
    fn no_two_sets_are_the_same_text_name_or_digest() {
        for (nth, one) in Instructions::ALL.into_iter().enumerate() {
            for other in Instructions::ALL.into_iter().skip(nth + 1) {
                assert_ne!(one.text(), other.text(), "{one:?} and {other:?}");
                assert_ne!(one.named(), other.named(), "{one:?} and {other:?}");
                assert_ne!(one.digest(), other.digest(), "{one:?} and {other:?}");
            }
            assert_eq!(Instructions::of_digest(&one.digest()), Some(one));
        }
    }

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

    /// Every set has a digest, and each digest finds its own set.
    ///
    /// Four now. The shape of this test was `let [first, second] = …`, which
    /// is the honest kind of break: adding a set failed to compile here rather
    /// than quietly leaving the new one unchecked.
    #[test]
    fn a_digest_names_one_set_of_instructions() {
        let [first, second, third, fourth] = Instructions::ALL.map(Instructions::digest);
        assert_ne!(first, second);
        assert_ne!(second, third);
        assert_ne!(first, third);
        assert_ne!(fourth, first);
        assert_ne!(fourth, second);
        assert_ne!(fourth, third);
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
        assert_eq!(
            Instructions::of_digest(&third),
            Some(Instructions::SeveralSteps)
        );
        assert_eq!(
            Instructions::of_digest(&fourth),
            Some(Instructions::AnswersInWords)
        );
        assert_eq!(Instructions::of_digest(&"0".repeat(64)), None);
        assert!(
            Instructions::OneExamplePerDoor
                .text()
                .ends_with("These are the only verbs there are:")
        );
    }
}
