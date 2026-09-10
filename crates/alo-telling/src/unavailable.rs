//! What makes one unavailability the same as another, and what makes it
//! different.
//!
//! This is the whole of what [`crate::Telling`] remembers. It is not a
//! sentence, it is not a failure and it is not something a person can be shown:
//! it is the **identity** of a telling, so that the second time the same thing
//! happens the machine can tell that it is the same thing.
//!
//! # Two halves, because those are the two the plan names
//!
//! *Telling it again takes something changing — the source, the reason, or the
//! person asking again themselves.* The third of those is not about the
//! unavailability at all ([`crate::WhoAsked`] is), so what is here is the first
//! two: **where the question was put**, and **what went wrong there**.
//!
//! # The status is part of the reason, deliberately
//!
//! `alo_answering::WentWrong::HavingTrouble` carries the number a service
//! answered with, and that number is **inside the sentence a person reads**. So
//! `503` and `500` are two different sentences, and suppressing the second
//! would show somebody a line about a failure that is not the one that
//! happened. That is the acceptance's *a different reason is a different
//! telling* read strictly rather than conveniently: a machine that swallowed
//! the second failure would hide the one that mattered, and a machine that
//! swallowed it because two numbers were near each other would be doing the
//! same thing with a rule nobody wrote down.
//!
//! # Nothing here can be turned into a sentence
//!
//! There is no `said` on this type and there will not be one. What
//! [`crate::Telling`] holds is a memory of *having said something*, and a
//! memory that could be rendered would be a machine able to put a telling back
//! on a screen at some later moment — which is exactly the *where it happened*
//! half of ADR 0009 failing. A telling exists where the turn failed, and
//! nowhere else, because there is nowhere else it could be built from.

use alo_answering::{Failed, WentWrong};
use alo_models::InferenceSource;

/// One way this machine could not reach a model, as a thing to compare.
///
/// Made from an `alo_answering::Failed` and from nothing else. There is no
/// public field, no `From`, no deserialiser and no constructor that takes a
/// source and a reason separately — which matters more here than it looks,
/// because `alo_answering::WentWrong::can_happen` refuses some pairings
/// outright, and a door taking the two apart would let a caller assemble an
/// unavailability that cannot happen and then suppress the real one it
/// resembles.
///
/// ```
/// use alo_answering::{Answering, WentWrong};
/// use alo_models::{InferenceSource, SourcePolicy};
/// use alo_telling::Unavailable;
///
/// let failed = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere)
///     .expect("nothing forbids answering here")
///     .did_not_answer(WentWrong::NothingAnswered, &[], &SourcePolicy::Anywhere)
///     .expect("a runtime that is not running can happen here");
///
/// let unavailable = Unavailable::of(&failed);
/// assert_eq!(unavailable.source(), &InferenceSource::ThisMachine);
/// assert_eq!(unavailable.why(), WentWrong::NothingAnswered);
/// ```
///
/// An unavailability cannot be assembled out of a place and a reason, and this
/// is what says so — there is no such constructor to call:
///
/// ```compile_fail
/// use alo_answering::WentWrong;
/// use alo_models::InferenceSource;
///
/// let _ = alo_telling::Unavailable::because(InferenceSource::ThisMachine, WentWrong::RanOut);
/// ```
///
/// nor can one be reached past, because neither half is a field anybody
/// outside this crate can touch:
///
/// ```compile_fail
/// fn reworded(unavailable: alo_telling::Unavailable) -> alo_answering::WentWrong {
///     unavailable.why
/// }
/// ```
///
/// Both are checked by unmarking them: the first fails with **E0599, no
/// function or associated item named `because`**, and the second with **E0616,
/// field `why` of struct `Unavailable` is private** — neither on a typo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unavailable {
    /// Where the question was put.
    source: InferenceSource,
    /// What went wrong there.
    why: WentWrong,
}

impl Unavailable {
    /// The identity of one failure: where it was put, and what went wrong.
    ///
    /// The only way to an [`Unavailable`]. Takes the failure by reference,
    /// because reading its identity is not spending it — the door onwards is
    /// `alo_answering::Failed::take` and it stays open.
    #[must_use]
    pub fn of(failed: &Failed) -> Self {
        Self {
            source: failed.source().clone(),
            why: failed.why(),
        }
    }

    /// Where the question was put.
    #[must_use]
    pub fn source(&self) -> &InferenceSource {
        &self.source
    }

    /// What went wrong there.
    #[must_use]
    pub fn why(&self) -> WentWrong {
        self.why
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{failing, here, hosted, paired};

    /// The identity is exactly the two halves the plan names, and it is read
    /// off the failure rather than handed in beside it.
    #[test]
    fn an_unavailability_is_where_it_happened_and_what_went_wrong() {
        let unavailable = Unavailable::of(&failing(hosted(), WentWrong::RanOut, &[]));
        assert_eq!(unavailable.source(), &hosted());
        assert_eq!(unavailable.why(), WentWrong::RanOut);
    }

    /// **The same thing twice is the same thing**, which is what makes
    /// suppression possible at all. The two failures here are separate values
    /// built from separate attempts, as two turns really would be.
    #[test]
    fn the_same_failure_happening_twice_has_one_identity() {
        assert_eq!(
            Unavailable::of(&failing(here(), WentWrong::NothingAnswered, &[])),
            Unavailable::of(&failing(here(), WentWrong::NothingAnswered, &[]))
        );
    }

    /// **A different place is a different thing**, even when the reason is
    /// identical — *nothing answered on this machine* and *nothing was answered
    /// by alo, in the EU* are two sentences about two halves of somebody's
    /// arrangements.
    #[test]
    fn the_same_reason_somewhere_else_is_a_different_identity() {
        assert_ne!(
            Unavailable::of(&failing(here(), WentWrong::NothingAnswered, &[])),
            Unavailable::of(&failing(hosted(), WentWrong::NothingAnswered, &[]))
        );
        assert_ne!(
            Unavailable::of(&failing(paired(), WentWrong::NothingAnswered, &[])),
            Unavailable::of(&failing(here(), WentWrong::NothingAnswered, &[]))
        );
    }

    /// **A different reason in the same place is a different thing.** This is
    /// the direction the acceptance is strictest about: a machine that
    /// swallowed the second failure would hide the one that mattered.
    #[test]
    fn a_different_reason_in_the_same_place_is_a_different_identity() {
        assert_ne!(
            Unavailable::of(&failing(hosted(), WentWrong::RanOut, &[])),
            Unavailable::of(&failing(hosted(), WentWrong::KeyNotAccepted, &[]))
        );
    }

    /// **And two statuses are two reasons**, because the number is inside the
    /// sentence: *it answered 503* suppressing *it answered 500* would show
    /// somebody a line about a failure that did not happen.
    #[test]
    fn two_statuses_from_one_service_are_two_identities() {
        assert_ne!(
            Unavailable::of(&failing(hosted(), WentWrong::HavingTrouble(503), &[])),
            Unavailable::of(&failing(hosted(), WentWrong::HavingTrouble(500), &[]))
        );
        assert_eq!(
            Unavailable::of(&failing(hosted(), WentWrong::HavingTrouble(503), &[])),
            Unavailable::of(&failing(hosted(), WentWrong::HavingTrouble(503), &[]))
        );
    }

    /// **What else the machine could have been asked instead is not part of
    /// it.** Two failures at one place for one reason are the same telling
    /// whether or not a provider was configured in between, because what a
    /// person is told is the same in both — and an identity that moved with the
    /// settings would tell somebody the same thing again for adding a provider
    /// they never used.
    #[test]
    fn where_else_this_machine_could_ask_is_not_part_of_the_identity() {
        assert_eq!(
            Unavailable::of(&failing(here(), WentWrong::NothingAnswered, &[])),
            Unavailable::of(&failing(
                here(),
                WentWrong::NothingAnswered,
                &[hosted(), paired()]
            ))
        );
    }
}
