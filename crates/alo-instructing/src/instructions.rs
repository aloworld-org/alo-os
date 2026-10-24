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

/// **Everything [`ONE_EXAMPLE_PER_DOOR`] says, and that a question may be
/// answered in words.**
///
/// Written out in full rather than composed, for [`SEVERAL_STEPS`]'s reason:
/// what a model is shown should be readable in one place — and held to
/// [`ONE_EXAMPLE_PER_DOOR`] plus [`ANSWERING_IN_WORDS_PARAGRAPH`] by a test,
/// so the two cannot drift and a difference in a grade is attributable.
///
/// # This is what a turn shows, and it is not yet graded
///
/// [`Instructions::SHOWN_TO_A_TURN`] names this set. **That is a change to
/// what ADR 0037 decision 2 settled**, and the reason is not taste: the set it
/// named is 80 of 80 at *driving the verbs*, and 0 of 1 at answering a
/// question, because under it answering a question cannot be expressed at all.
/// A measurement of one is not an answer about the other.
///
/// **A regrade is owed and is not done here.** It needs a model and the
/// harness in `alo-driving`, and this machine has neither — `ollama` reports
/// `models=0`. What is owed is the fixed ten under this set, on both: the
/// verb-driving exercises the 80 of 80 was measured on, and questions that
/// need no verb, which nothing has ever been measured on because nothing could
/// pass them.
pub const ANSWERING_IN_WORDS: &str = "\
You are talking to a computer, not to a person. Answer with one line of JSON and
nothing else: no explanation, no code fence, no second line.

For a verb that only answers a question, the line has this shape:

{\"format\":1,\"asks\":{\"read\":{\"verb\":\"NAME\",\"given\":[{\"named\":\"ARGUMENT\",\"is\":VALUE}]}}}

For a verb that changes something, the line has this shape:

{\"format\":1,\"asks\":{\"propose\":{\"verb\":\"NAME\",\"given\":[{\"named\":\"ARGUMENT\",\"is\":VALUE}]}}}

Each verb below says which it is: (read) or (propose). VALUE is text in quotes,
or a whole number with no quotes. Give every argument the verb takes and no
others. A path is always a full path.

If the request is a question you can answer from what you already know, and no
verb below would help, do not send JSON at all: write the answer in plain words
and nothing else. The lines above are how you ask this computer to do
something, and a question you can simply answer is not asking it to do
anything.

These are the only verbs there are:";

/// **The one paragraph [`ANSWERING_IN_WORDS`] adds to
/// [`ONE_EXAMPLE_PER_DOOR`].**
///
/// Held separately for [`AND_YOU_MAY_LOOK_FIRST`]'s reason: the difference
/// between two sets is a value rather than a diff somebody reads, so a
/// difference in a grade is attributable to these words and nothing else.
///
/// # What it fixes, measured on a booted machine
///
/// **A person could not ask alo a question.** On 2026-10-24 a real question —
/// *what is the capital of uganda* — reached a real provider through the Bar
/// and the turn ended `ItRanOutOfRounds` after eight steps.
///
/// The cause is a contradiction between two halves that were each right.
/// `alo_agent::driving` treats a reply that is **not** JSON as the answer in
/// words, which is how a turn ends in an answer at all. And
/// [`ONE_EXAMPLE_PER_DOOR`] opens *answer with one line of JSON and nothing
/// else*, then offers exactly two shapes, both of them verbs that act on this
/// machine. **There was no shape for an answer.** A model told that, asked a
/// question needing no verb, can only produce JSON that is not a request — and
/// the turn goes round until the rounds run out.
///
/// This paragraph says the exception out loud, because the sentence it has to
/// override is emphatic and a model reading both needs to be told which wins.
pub const ANSWERING_IN_WORDS_PARAGRAPH: &str = "\
If the request is a question you can answer from what you already know, and no
verb below would help, do not send JSON at all: write the answer in plain words
and nothing else. The lines above are how you ask this computer to do
something, and a question you can simply answer is not asking it to do
anything.

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
    /// [`ANSWERING_IN_WORDS`]: the second set, and that a question needing no
    /// verb is answered in plain words.
    ///
    /// **This is what a turn shows.** Under `OneExamplePerDoor` a question that
    /// needs no verb cannot be expressed at all, which a booted machine proved
    /// on 2026-10-24. See `ANSWERING_IN_WORDS` for what is owed.
    AnsweringInWords,
}

impl Instructions {
    /// Both, in the order they were written.
    pub const ALL: [Self; 4] = [
        Self::AsFirstWritten,
        Self::OneExamplePerDoor,
        Self::SeveralSteps,
        Self::AnsweringInWords,
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
    pub const SHOWN_TO_A_TURN: Self = Self::AnsweringInWords;

    /// The text a model is shown.
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            Self::AsFirstWritten => HOW_TO_ANSWER,
            Self::OneExamplePerDoor => ONE_EXAMPLE_PER_DOOR,
            Self::SeveralSteps => SEVERAL_STEPS,
            Self::AnsweringInWords => ANSWERING_IN_WORDS,
        }
    }

    /// How a harness and a report name them.
    #[must_use]
    pub fn named(self) -> &'static str {
        match self {
            Self::AsFirstWritten => "as-first-written",
            Self::OneExamplePerDoor => "one-example-per-door",
            Self::SeveralSteps => "several-steps",
            Self::AnsweringInWords => "answering-in-words",
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

    /// **And a set a turn shows carries what is owed about it.**
    ///
    /// This asserted `SHOWN_TO_A_TURN == OneExamplePerDoor` and said *naming an
    /// unmeasured set there would make every grade a statement about text
    /// nobody graded under.* That was right, and it is why the switch below is
    /// recorded rather than made quietly.
    ///
    /// **What changed is a fact the rule did not have.** Under
    /// `OneExamplePerDoor` a question needing no verb cannot be expressed at
    /// all: the text opens *answer with one line of JSON and nothing else* and
    /// offers two shapes, both verbs. A booted machine proved it on 2026-10-24
    /// — a real question through the Bar ended `ItRanOutOfRounds` after eight
    /// steps. So the 80-of-80 that named it is a measurement of **driving the
    /// verbs**, and says nothing about answering; on answering, the set it
    /// names scores nothing, inexpressibly.
    ///
    /// **A regrade is owed** and is named in ADR 0037's amendment: the fixed
    /// ten under `answering-in-words`, on `qwen2.5-7b-instruct`, which is the
    /// model the 80 of 80 was earned on. Any other model gives a number about
    /// a different thing.
    ///
    /// This test now holds the two things that are true: a turn shows a set
    /// this crate has, and the set it shows differs from the measured one by
    /// exactly one paragraph — so what a regrade would be measuring is that
    /// paragraph and nothing else.
    #[test]
    fn what_a_turn_shows_is_a_set_this_crate_has_and_differs_by_one_paragraph() {
        assert_eq!(
            Instructions::of_digest(&Instructions::SHOWN_TO_A_TURN.digest()),
            Some(Instructions::SHOWN_TO_A_TURN),
            "a turn is shown text with no digest in the catalogue"
        );
        assert_eq!(
            Instructions::SHOWN_TO_A_TURN,
            Instructions::AnsweringInWords
        );
        assert_eq!(
            ANSWERING_IN_WORDS.replace(ANSWERING_IN_WORDS_PARAGRAPH, ""),
            ONE_EXAMPLE_PER_DOOR,
            "what a turn shows differs from the measured set by more than the one paragraph, so \
             a regrade would not be measuring that paragraph"
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
        // **Named, which is what this test is for.** Which set it is, and
        // what is owed about it, is the test above.
        assert_eq!(
            Instructions::of_digest(&Instructions::SHOWN_TO_A_TURN.digest()),
            Some(Instructions::SHOWN_TO_A_TURN)
        );
        assert!(!Instructions::SHOWN_TO_A_TURN.named().is_empty());
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
    /// Four now rather than three, and this test has broken on purpose each
    /// time. The shape of it was `let [first, second] = …`, then three, and
    /// adding a set has failed to compile here every time rather than quietly
    /// leaving the new one unchecked.
    #[test]
    fn a_digest_names_one_set_of_instructions() {
        let [first, second, third, fourth] = Instructions::ALL.map(Instructions::digest);
        for (a, b) in [
            (&first, &second),
            (&second, &third),
            (&first, &third),
            (&fourth, &first),
            (&fourth, &second),
            (&fourth, &third),
        ] {
            assert_ne!(a, b, "two sets of instructions share a digest");
        }
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
        assert_eq!(Instructions::of_digest(&"0".repeat(64)), None);
        assert!(
            Instructions::OneExamplePerDoor
                .text()
                .ends_with("These are the only verbs there are:")
        );
    }
}
