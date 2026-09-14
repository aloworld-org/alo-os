//! **Which of an entry's grades says whether it may be given the agent.**
//!
//! Task 15 of `docs/autonomy/v0-5-the-models-measured-plan.md`, and the second
//! half of [ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md)'s
//! decision 5. A grade is a measurement of a way of asking, and the way that
//! decides is the way an agent turn asks. Since `e99be94` an agent's next
//! request is asked of the pinned runtime in the envelope, so an entry measured
//! that way is judged by that grade, and the free grade is what a person is
//! shown as history. An entry nobody measured in the envelope is judged by the
//! free grade, because it is the only measurement there is — never by a grade
//! assumed for it.
//!
//! Neither grade is rewritten, and both stay in the catalogue side by side.
//!
//! **What this does not read**, and on purpose: a grade under other
//! instructions ([`crate::AlsoUnder`]) or at another quantisation
//! ([`crate::AlsoAt`]). An agent turn is shown neither of the two sets of
//! instructions a grade names, and it runs the file the entry names — so a
//! grade earned under instructions a turn is not shown, or on a file the entry
//! does not name, is not a grade for the turn
//! ([ADR 0034](../../../docs/decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md),
//! decision 4).
//!
//! **What a turn is shown is now the product's to say**, and this still does not
//! read it. `alo-instructing` holds the words a model is shown and names the set
//! a turn shows them in, and
//! [ADR 0037](../../../docs/decisions/0037-the-words-a-turn-shows-a-model-are-the-products-own.md)
//! decision 3 is why nothing moves here yet: until `alo-turn` composes what it
//! shows a model from that crate, a turn is shown whatever its caller wrote, and
//! a grade read for it would be a grade about somebody else's words. When the
//! wiring lands, decision 4 of that ADR is what this file becomes — the grade
//! earned under the turn's own instructions, and no grade at all for an entry
//! never measured under them.

use alo_strings::{Filling, Said, Strings};

use crate::catalogue::Model;
use crate::driving::Driving;
use crate::words;

/// **How the grade that decides was earned.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskedTheWay {
    /// Held to the protocol's envelope — the way an agent turn asks.
    InTheEnvelope,
    /// Asked with nothing holding the answer's shape, because no measurement
    /// was made the way a turn asks.
    Freely,
}

impl AskedTheWay {
    /// The sentence a person is shown beside the grade.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let word = match self {
            Self::InTheEnvelope => words::GRADED_IN_THE_ENVELOPE,
            Self::Freely => words::GRADED_FREELY,
        };
        strings.say(&word.key(), &Filling::nothing())
    }
}

impl Model {
    /// **The grade that says whether this entry may be given the agent, and how
    /// it was earned**: the enveloped grade where one was measured, and the free
    /// grade only where none was.
    #[must_use]
    pub fn grade_for_the_turn(&self) -> (Driving, AskedTheWay) {
        match self.drives_verbs_in_the_envelope {
            Some(grade) => (grade, AskedTheWay::InTheEnvelope),
            None => (self.drives_verbs, AskedTheWay::Freely),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::Catalogue;
    use crate::testing::in_english;

    /// **Every entry the catalogue ships is judged by the grade for the way turns
    /// ask**: its enveloped grade where it has one, its free grade otherwise.
    #[test]
    fn every_shipped_entry_is_judged_by_its_enveloped_grade_or_else_its_free_one() {
        for model in Catalogue::built_in().unwrap().models {
            let (grade, asked) = model.grade_for_the_turn();
            match model.drives_verbs_in_the_envelope {
                Some(enveloped) => {
                    assert_eq!((grade, asked), (enveloped, AskedTheWay::InTheEnvelope));
                }
                None => assert_eq!((grade, asked), (model.drives_verbs, AskedTheWay::Freely)),
            }
            assert_eq!(
                model.can_be_the_agent(),
                grade.clears_the_bar(),
                "{}",
                model.id
            );
        }
    }

    /// The two ways are two sentences, and neither is the other.
    #[test]
    fn the_two_ways_are_said_apart() {
        let strings = in_english();
        let enveloped = AskedTheWay::InTheEnvelope.said(&strings).text().to_owned();
        let freely = AskedTheWay::Freely.said(&strings).text().to_owned();
        assert_ne!(enveloped, freely);
        assert!(enveloped.contains("agent turn"), "{enveloped}");
        assert!(
            freely.contains("not the way an agent turn asks"),
            "{freely}"
        );
    }
}
