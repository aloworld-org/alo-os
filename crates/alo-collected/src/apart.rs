//! How much of a reason a crate has to give for standing outside the one
//! vocabulary.
//!
//! One crate does today. `alo-agentd` is Linux, every module in it is compiled
//! out anywhere else, and a vocabulary assembled from it would hold three fewer
//! strings on a host with no daemon than on a machine with one — so one host
//! would refuse a translation file the other accepted. That is a real argument,
//! it is written down beside the list, and it is the only kind of exception this
//! check will take.
//!
//! What it will not take is a name on its own. An exception with no reason is
//! indistinguishable from the failure this whole crate exists to catch: a crate
//! that declares words, is collected nowhere, and says nothing to anybody in any
//! language. The difference between the two is entirely the sentence, so the
//! sentence is held to a floor — the same floor `alo-by-hand` and
//! `alo-reconciling` put under what a promise still owes, at the same height and
//! for the same reason.
//!
//! Nothing mechanical can judge whether a reason is *good*. The reader of the
//! list is the real check. This is what stops them reading a shrug.

/// How much of a reason a crate standing apart has to give, in characters.
///
/// `not yet` is seven and `it is Linux` is eleven, and neither tells the next
/// person why a translation would break if this were changed. Deliberately a
/// floor rather than a judge.
pub const A_REASON: usize = 40;

/// Whether a sentence is a reason rather than a shrug.
#[must_use]
pub fn is_a_reason(sentence: &str) -> bool {
    sentence.chars().count() >= A_REASON
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shrug is refused, and a reason of the shortest honest kind is not.
    #[test]
    fn a_shrug_is_refused_and_a_reason_is_not() {
        assert!(
            !is_a_reason("it is Linux"),
            "a fact about a crate was accepted as a reason to leave its words out \
             of the vocabulary a translation is checked against"
        );
        assert!(
            !is_a_reason("special"),
            "`special` was accepted as why a crate says nothing in anybody's \
             language"
        );
        assert!(
            is_a_reason(
                "it is Linux, so a vocabulary built here would be shorter on one \
                 host than another and a translation would be refused on one of them"
            ),
            "a sentence naming what would break was refused, so the floor is a wall"
        );
    }

    /// Counted in characters rather than bytes, so the floor does not depend on
    /// which language the argument happens to be written in.
    #[test]
    fn the_floor_is_characters_and_not_bytes() {
        let accented: String = "é".repeat(A_REASON - 1);
        assert!(
            !is_a_reason(&accented),
            "a sentence of {} characters cleared a floor of {A_REASON}, so the \
             floor is measured in bytes",
            A_REASON - 1
        );
        assert!(is_a_reason(&"é".repeat(A_REASON)));
    }
}
