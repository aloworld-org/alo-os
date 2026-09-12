//! What makes one warning about size the same as another, and what makes it
//! different.
//!
//! This is the whole of what [`crate::Warning`] remembers. It is not a
//! sentence and it is not something a person can be shown: it is the
//! **identity** of a warning, so that the second time the same weights are
//! costed on the same machine, the machine can tell that it is the same thing.
//!
//! # Two halves: which weights, and what they cost here
//!
//! `docs/features.md`: *a model too large for the memory in this laptop is
//! said so plainly, **once** — and then run anyway.* Once per what? Per the
//! weights, on this machine: the same file costed again at the next question
//! is the same warning, and a second set of weights is a second warning. The
//! memory figure is part of it, because a person who docks their laptop into
//! a machine with more memory has changed the thing the sentence was about —
//! and a warning that ignored it would suppress a sentence that is now
//! different, or repeat one that is now the same.
//!
//! # Only the case worth saying has an identity
//!
//! [`TooLarge::of`] answers [`None`] for weights that fit. There is nothing to
//! warn about, so there is nothing to remember having warned about, and a
//! memory holding *these fit* would be a memory of something nobody was told.
//!
//! # Nothing here can be turned into a sentence
//!
//! There is no `said` on this type, for `unavailable.rs`'s reason: what the
//! memory holds is a memory of *having said something*, and a memory that
//! could be rendered would be a machine able to put the warning back on a
//! screen at some later moment — which is the nag, with a delay on it.

use alo_models::{Cost, Weights};

/// One warning about weights larger than this machine's memory, as a thing to
/// compare.
///
/// Made from [`alo_models::Weights`] and a memory figure, and from nothing
/// else. There is no public field and no constructor from an id and a number,
/// so a caller cannot assemble a warning that was never costed and then
/// suppress the real one it resembles.
///
/// `PartialEq` and not `Eq`, because a memory figure is an `f32` and this crate
/// does not pretend two floats can be compared more strictly than they can.
///
/// ```
/// use alo_models::{Weights, costing::GIGABYTE};
/// use alo_telling::TooLarge;
///
/// let theirs = Weights::checked("their-own-70b", 40 * GIGABYTE).expect("a name");
///
/// // Forty gigabytes on a sixteen gigabyte machine is a warning; on a
/// // sixty-four gigabyte one it is nothing at all.
/// assert!(TooLarge::of(&theirs, 16.0).is_some());
/// assert!(TooLarge::of(&theirs, 64.0).is_none());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TooLarge {
    /// Which weights: the id the runtime answers to.
    id: String,
    /// What they cost on this machine, both numbers.
    cost: Cost,
}

impl TooLarge {
    /// The identity of one warning: these weights, on a machine with this much
    /// memory — or [`None`] when they fit, because then there is nothing to
    /// warn about and nothing to remember.
    #[must_use]
    pub fn of(weights: &Weights, machine_gb: f32) -> Option<Self> {
        let cost = weights.costs_on(machine_gb);
        cost.larger_than_memory().then(|| Self {
            id: weights.id.clone(),
            cost,
        })
    }

    /// Which weights this is about.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// What they cost on the machine this was about.
    #[must_use]
    pub fn cost(&self) -> Cost {
        self.cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::weights_of;
    use alo_models::costing::GIGABYTE;

    /// The identity is the two halves the sentence is about, read off the
    /// weights and the machine rather than handed in beside them.
    #[test]
    fn a_warning_is_which_weights_and_what_they_cost_here() {
        let warning = TooLarge::of(&weights_of("theirs", 40 * GIGABYTE), 16.0);
        let warning = warning.as_ref();
        assert_eq!(warning.map(TooLarge::id), Some("theirs"));
        assert!(warning.is_some_and(|w| w.cost().larger_than_memory()));
        assert_eq!(warning.map(|w| w.cost().machine_gb()), Some(16.0));
    }

    /// **Weights that fit have no identity**, because nothing is said about
    /// them and there is nothing to remember.
    #[test]
    fn weights_that_fit_are_not_a_warning() {
        assert_eq!(
            TooLarge::of(&weights_of("theirs", 4 * GIGABYTE), 16.0),
            None
        );
        assert_eq!(
            TooLarge::of(&weights_of("theirs", 16 * GIGABYTE), 16.0),
            None
        );
    }

    /// **The same weights on the same machine, twice, is one warning** —
    /// which is what makes saying it once possible at all.
    #[test]
    fn the_same_weights_on_the_same_machine_have_one_identity() {
        assert_eq!(
            TooLarge::of(&weights_of("theirs", 40 * GIGABYTE), 16.0),
            TooLarge::of(&weights_of("theirs", 40 * GIGABYTE), 16.0)
        );
    }

    /// **Different weights are a different warning**, and so are the same
    /// weights on a machine with a different amount of memory: the sentence
    /// was about both.
    #[test]
    fn other_weights_or_another_machine_are_a_different_identity() {
        let theirs = TooLarge::of(&weights_of("theirs", 40 * GIGABYTE), 16.0);
        assert_ne!(
            theirs,
            TooLarge::of(&weights_of("others", 40 * GIGABYTE), 16.0)
        );
        assert_ne!(
            theirs,
            TooLarge::of(&weights_of("theirs", 40 * GIGABYTE), 32.0)
        );
        // And a size that changed — the file was replaced — is a different
        // warning too, because the number beside the sentence changed.
        assert_ne!(
            theirs,
            TooLarge::of(&weights_of("theirs", 80 * GIGABYTE), 16.0)
        );
    }
}
