//! Who caused the turn that failed — the one thing that makes a machine saying
//! something again not a nag.
//!
//! ADR 0009: *a machine that cannot reach a model says so once, where it
//! happened, and continues. It does not follow somebody around asking them to
//! buy credit.* The word doing the work in that sentence is **follow**. A
//! machine that repeats itself unbidden is following somebody around; a machine
//! that answers a person who has just asked again is answering a question.
//!
//! So this is not a flag about importance and it is not a severity. It is the
//! answer to one question — *did a person just do something, or did the machine
//! do this by itself?* — and it is the caller's to answer honestly, because the
//! caller is the only place that knows.
//!
//! # Where the line falls, so a shell does not have to guess
//!
//! [`WhoAsked::ThePerson`] is a turn that exists because somebody acted at this
//! moment: they typed a question, they pressed the key, they answered an offer
//! to ask somewhere else, they chose *try again*. The failure that follows is
//! the answer to what they did, and withholding it would be the key that
//! silently does nothing — the worst outcome the agent overlay's seam already
//! ruled out.
//!
//! [`WhoAsked::TheMachine`] is everything else: a retry nobody asked for, a
//! surface refreshing itself, a second component asking the same question in
//! the same moment, work that resumed when the network came back. None of those
//! is a person waiting for an answer, so a sentence produced by one of them is
//! a sentence that arrives unasked — which is the nag, whatever it says.
//!
//! # Why the default is the strict one
//!
//! There is no `Default` here on purpose. A caller that did not think about
//! this question would inherit whichever answer somebody wrote first, and both
//! answers are wrong to inherit: `ThePerson` turns every background retry into
//! a telling, and `TheMachine` silences somebody who genuinely asked again.
//! Making it an argument with no default costs one word at each call site and
//! buys a decision that was actually made.

/// Who caused the turn that failed.
///
/// Two, and a third belongs here only if it is a different thing to *do* about
/// a repeated failure — not a different thing to have caused one. *A scheduled
/// task* and *a retry* are one answer to the question this type asks.
///
/// **There is no answer a caller can fall into**, and this is what says so:
///
/// ```compile_fail
/// let _: alo_telling::WhoAsked = Default::default();
/// ```
///
/// It fails with **E0277, the trait bound `WhoAsked: Default` is not
/// satisfied**. A default would be a decision nobody made, and both possible
/// defaults are wrong in the direction that matters: one turns every background
/// retry into a telling, the other silences somebody who asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoAsked {
    /// A person did something at this moment and is waiting for the result.
    ///
    /// They are always told, including when they have been told the same thing
    /// before: they asked, and an answer to a question is not a reminder.
    ThePerson,
    /// The machine did: a retry, a refresh, a resumed piece of work, a second
    /// surface asking the same question.
    ///
    /// Told only when it is something the person has not been told, because a
    /// sentence nobody asked for is the thing ADR 0009 calls following somebody
    /// around.
    TheMachine,
}

impl WhoAsked {
    /// Whether this is somebody waiting for an answer right now.
    ///
    /// Written as a question rather than matched at the one call site that
    /// needs it, because the sentence `is_a_person_waiting` is the whole of the
    /// rule and the `match` was one negation away from meaning the opposite.
    #[must_use]
    pub fn is_a_person_waiting(self) -> bool {
        matches!(self, Self::ThePerson)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One of the two is a person waiting, and it is the one whose name says
    /// so.
    #[test]
    fn one_of_the_two_is_somebody_waiting_for_an_answer() {
        assert!(WhoAsked::ThePerson.is_a_person_waiting());
        assert!(!WhoAsked::TheMachine.is_a_person_waiting());
    }

    /// The two are told apart by value, so neither can be reached by mistaking
    /// one name for the other.
    #[test]
    fn the_two_answers_are_told_apart_by_value_rather_than_by_name() {
        assert_ne!(WhoAsked::ThePerson, WhoAsked::TheMachine);
    }
}
