//! How much of an answer a sentence has to be.
//!
//! Nothing mechanical can judge whether *a person opens the folder in the file
//! manager* is honest. What can be held is a floor beneath a sentence, so that
//! `soon`, `todo` and `the agent does it` are refused before anybody has to read
//! them — and this is the same floor `alo-reconciling` puts under what a release
//! promise still owes, for the same reason and at the same height.
//!
//! The reader of `docs/by-hand.md` is the real check. This is what stops them
//! reading a shrug.

/// How much of an answer a sentence has to be, in characters.
///
/// `not yet` is seven and passes nothing. The shortest honest answer in this
/// repository's document names a surface and what a person does with it, which
/// does not fit in forty. Deliberately a floor rather than a judge.
pub const AN_ANSWER: usize = 40;

/// Whether a sentence is an answer rather than a shrug.
#[must_use]
pub fn is_an_answer(sentence: &str) -> bool {
    sentence.chars().count() >= AN_ANSWER
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shrug is refused, and an answer of the shortest honest kind is not.
    #[test]
    fn a_shrug_is_refused_and_an_answer_is_not() {
        assert!(
            !is_an_answer("not yet"),
            "`not yet` was accepted as how a person does something by hand"
        );
        assert!(
            !is_an_answer("the agent does it"),
            "the agent was accepted as the way to do something without the agent"
        );
        assert!(
            is_an_answer("a person renames the file in the file manager, in place"),
            "a sentence naming the surface and what a person does with it was \
             refused, so the floor is a wall"
        );
    }

    /// Counted in characters rather than bytes, because a sentence in Maltese or
    /// Greek would otherwise clear a floor an English one did not.
    #[test]
    fn the_floor_is_characters_and_not_bytes() {
        let accented: String = "é".repeat(AN_ANSWER - 1);
        assert!(
            !is_an_answer(&accented),
            "a sentence of {} characters cleared a floor of {AN_ANSWER}, so the \
             floor is measured in bytes",
            AN_ANSWER - 1
        );
        assert!(is_an_answer(&"é".repeat(AN_ANSWER)));
    }
}
