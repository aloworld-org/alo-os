//! Where the organisation's rule and the person's choice meet.
//!
//! One method, and it is
//! [ADR 0016](../../../docs/decisions/0016-the-organisation-bounds-and-the-person-chooses.md)'s
//! *when they disagree*: the bound wins, and what a person chose is either
//! permitted at the place they chose or refused there. There is no method here
//! that answers with a place they did not pick.
//!
//! # Silent substitution is what this shape exists to prevent
//!
//! It is the comfortable thing to build: a person picks somewhere, the rule
//! forbids it, and the machine answers anyway from a place that is permitted.
//! Nothing looks broken, and the person believes they know where their question
//! went. So [`Chosen::asking`] hands back the permission for **the place that
//! was chosen** or the rule's own refusal, and a caller that wanted to fall
//! back would have to find another choice to make — which this crate cannot
//! make, because [`crate`] says why it deliberately cannot recommend one.
//!
//! # The refusal is `alo-models`' and is not reworded here
//!
//! `alo_models::NotAllowed` names the rule and the place it refused, and
//! `NotAllowed::said` renders it. This crate adds nothing to it, and the
//! addition ADR 0016 asks for — *and an administrator set that rule* — is
//! deliberately not written yet: **no rule an organisation can set refuses
//! anything a person can currently choose**, because both lists a choice can
//! name are this machine and no policy forbids a machine answering on itself.
//! There is a test below that says so. A sentence for a refusal that cannot
//! happen would be a string a translator was handed for nothing, and the item
//! that makes it reachable is the item that adds a place a question can leave
//! for.
//!
//! # Absent is not the same as permissive, except in what it permits
//!
//! `bound` is an [`Option`], because ADR 0016 says a personal machine has no
//! policy at all — *not empty, not permissive by default, absent*. What is done
//! with the absence is `alo_models::SourcePolicy::Anywhere`, which permits
//! everything and refuses nothing, so the two coincide in every answer they
//! give. Where they differ is what can be said afterwards: a machine with no
//! organisation has nobody to name in a refusal, and it has none to make.

use alo_answering::Answering;
use alo_models::{NotAllowed, SourcePolicy};

use crate::chosen::Chosen;

/// What a personal machine's bound is, where there is none.
///
/// A rule that permits everything and refuses nothing, so nothing about a
/// machine with no organisation depends on a file existing.
const UNBOUNDED: SourcePolicy = SourcePolicy::Anywhere;

impl Chosen {
    /// The permission to put a question where this person chose.
    ///
    /// `bound` is the rule an organisation set on this machine, and [`None`] is
    /// a machine no organisation manages.
    ///
    /// **Spent by being used.** `alo_answering::Answering` is not `Clone` and
    /// one of them means one attempt, so this is called once per question
    /// rather than once per machine — which is also what makes a rule tightened
    /// this morning the rule in force this afternoon.
    ///
    /// # Errors
    /// `alo_models::NotAllowed`, naming the rule that refused and the place it
    /// refused, in whichever language the person turns out to read. Nothing is
    /// attempted, nothing is sent, and no other place is offered in its stead.
    pub fn asking(&self, bound: Option<&SourcePolicy>) -> Result<Answering, NotAllowed> {
        Answering::chosen(self.source(), bound.unwrap_or(&UNBOUNDED))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::chosen::Which;
    use alo_models::InferenceSource;

    /// The ordinary choice these tests are about.
    fn a_model() -> Chosen {
        Chosen::of(Which::Catalogue, "a-model").unwrap()
    }

    /// **A machine no organisation manages permits what the person chose**, and
    /// it does so without a policy file existing anywhere.
    #[test]
    fn a_machine_with_no_organisation_permits_the_choice_the_person_made() {
        let asking = a_model().asking(None).unwrap();
        assert_eq!(asking.source(), &InferenceSource::ThisMachine);
        assert!(!asking.causes_egress());
    }

    /// **And so does every rule an organisation can set.** This is the case
    /// worth asserting rather than the refusal: a person who picked a model on
    /// their own machine is not locked out of their own hardware by the
    /// strictest policy ADR 0004 permits, and both lists a choice can name are
    /// their own machine.
    ///
    /// It is also why there is no refusal to test here yet. When a person can
    /// choose a provider or a machine in the next room, this test stops being
    /// the whole story and the sentence naming who set the rule is written
    /// then.
    #[test]
    fn no_rule_an_organisation_can_set_forbids_this_machine_answering_on_itself() {
        for bound in [
            SourcePolicy::Anywhere,
            SourcePolicy::InTheBuilding,
            SourcePolicy::InRegion("Singapore".to_owned()),
            SourcePolicy::ThisMachineOnly,
        ] {
            for which in [Which::Catalogue, Which::Brought] {
                let chosen = Chosen::of(which, "a-model").unwrap();
                assert!(chosen.asking(Some(&bound)).is_ok(), "{bound:?} {which:?}");
            }
        }
    }

    /// **A rule that refuses is carried whole rather than reworded**, which is
    /// item 9e's decision met here: the words are `alo-models`', so the screen
    /// and the record cannot be two accounts of one moment. Asked of the rule
    /// directly, because nothing this crate can express reaches it.
    #[test]
    fn a_rule_that_refuses_names_itself_and_the_place_it_refused() {
        let somewhere_else = InferenceSource::Hosted {
            provider: "someone".to_owned(),
            region: alo_models::Region::Unknown,
        };
        assert_eq!(
            Answering::chosen(somewhere_else.clone(), &SourcePolicy::ThisMachineOnly).unwrap_err(),
            NotAllowed::NotThisMachine {
                source: somewhere_else
            }
        );
    }
}
