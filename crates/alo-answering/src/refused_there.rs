//! What a paired machine says when it will not answer a question, as a word
//! this machine renders.
//!
//! A machine down the corridor that is asked a question and will not answer it
//! says why in a word of its own on the wire (`docs/contracts/local-network-wire.md`,
//! *the question path*): its person's pairing does not let this machine ask its
//! models, its person chose a provider and it answers for nobody, or nothing
//! there has been chosen to answer at all. Those are three different things
//! for the person here to do — ask the other person to widen the pairing, ask
//! them to pick a model, or choose something else to answer their own
//! questions — so they are three reasons rather than one *it did not answer*.
//!
//! # The word is theirs, the sentence is ours
//!
//! What arrives is an identifier the other machine chose from a closed list,
//! never a sentence it composed: `crate::wrong`'s rule that nothing here holds
//! text anybody else wrote is kept, because the other machine's person reads
//! another language and this machine's person reads this one. The sentence is
//! declared in `crate::words` and rendered in the language the person **here**
//! reads.
//!
//! # And only a paired machine can say one
//!
//! A provider has no pairing to refuse under, and this machine does not ask
//! itself across a wire. So each of these is refused where it is reported about
//! any other place, as `crate::NotWhatFailed::NoMachineThere` — the opposite
//! shape from a refused key, which is refused *at* a paired machine.

use crate::words;

/// Why a paired machine would not answer, in its own word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusedThere {
    /// The pairing, as that machine keeps it, does not permit asking its
    /// models — `not-permitted` on the wire.
    NotPermitted,
    /// Its person chose a provider, and a question from another machine is
    /// never forwarded to one — `answers-elsewhere` on the wire.
    AnswersElsewhere,
    /// Nothing there is chosen or running to answer, or its settings do not
    /// hold — `not-answered-here` on the wire.
    NothingChosenThere,
}

impl RefusedThere {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> words::Word {
        match self {
            Self::NotPermitted => words::NOT_PERMITTED_THERE,
            Self::AnswersElsewhere => words::ANSWERS_ELSEWHERE_THERE,
            Self::NothingChosenThere => words::NOTHING_CHOSEN_THERE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RefusedThere;

    /// **Three refusals, three sentences**: each sends the person here to a
    /// different thing to do.
    #[test]
    fn each_refusal_from_a_paired_machine_is_its_own_sentence() {
        let mut keys: Vec<String> = [
            RefusedThere::NotPermitted,
            RefusedThere::AnswersElsewhere,
            RefusedThere::NothingChosenThere,
        ]
        .iter()
        .map(|refused| refused.word().named().to_owned())
        .collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), 3);
    }
}
