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
//! `NotAllowed::said` renders it. This crate adds nothing to it.
//!
//! **That paragraph used to end differently, and it had stopped being true.** It
//! said no rule an organisation can set refuses anything a person can currently
//! choose, because both lists a choice can name are this machine — so the
//! addition ADR 0016 asks for, *and an administrator set that rule*, would be a
//! string handed to a translator for a refusal that could not happen.
//!
//! A person can now choose a **provider**. [`crate::Picked`] resolves one
//! through their own list, `alo_models::Provider::source` answers
//! `InferenceSource::Hosted`, and `SourcePolicy::ThisMachineOnly` refuses
//! exactly that. `a_rule_can_now_refuse_a_place_a_person_can_choose` is that,
//! held up — and it is the condition the queue's item 21l named as the day this
//! becomes writable.
//!
//! The sentence itself is **still not written here**, and the reason has changed
//! from *unreachable* to *undecided*: what a person is told belongs beside
//! whoever says it, the daemon's provider path reaches `alo_answering` directly
//! rather than through this crate, and wording it in two places is how a screen
//! and a record become two accounts of one moment.
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
use alo_models::{NotAllowed, Providers, SourcePolicy};

use crate::chosen::{Chosen, Picked};

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

impl Picked {
    /// The permission to put a question where this person chose, whichever of
    /// the three that is.
    ///
    /// `providers` is the person's own list, and it is required rather than
    /// optional because that is the only place a provider's source can come
    /// from. [`Chosen::asking`] is the local half of this and can do without
    /// one; there is no version of this that can.
    ///
    /// # Errors
    /// `alo_models::NotAllowed` when the rule in force refuses the place they
    /// chose — naming the rule and the place, in the language they read.
    /// Nothing is attempted, nothing is sent, and **no other place is offered
    /// in its stead**: a question bound for a provider does not become a
    /// question for a model on this machine because the provider was refused.
    ///
    /// [`None`] — rather than a refusal — when the choice names a provider this
    /// list does not have. That is not a rule refusing a place; it is a
    /// question with no place in it, and dressing it as a policy refusal would
    /// tell somebody their organisation had stopped them when nothing had.
    /// [`crate::Settings`] refuses such a file, so it cannot arise from one.
    pub fn asking(
        &self,
        providers: &Providers,
        bound: Option<&SourcePolicy>,
    ) -> Option<Result<Answering, NotAllowed>> {
        let source = self.source(providers)?;
        Some(Answering::chosen(source, bound.unwrap_or(&UNBOUNDED)))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::chosen::{Picked, Which};
    use alo_models::{InferenceSource, Providers};

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
    /// It is **no longer the whole story**, and the test below is the rest of
    /// it: a person can choose a provider now, and a rule can refuse one.
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

    /// **A rule can now refuse a place a person can choose**, which it could
    /// not when this file was written.
    ///
    /// The refusal below is asked of the rule directly because nothing this
    /// crate could express reached it. This one goes through
    /// [`crate::Picked::asking`] — the person's own choice, resolved against
    /// their own list — so it is the crate's own API refusing, and that is what
    /// makes the sentence ADR 0016 asks for worth writing at all.
    ///
    /// **And the invariant that makes it sayable**: a refusal from here always
    /// means an organisation set a rule. `asking(None)` is `Anywhere`, which
    /// permits everything, so there is no road to a refusal on a machine with no
    /// policy — and therefore no refusal that would have to name an
    /// administrator who does not exist. Both halves are asserted.
    #[test]
    fn a_rule_can_now_refuse_a_place_a_person_can_choose() {
        let mut providers = Providers::default();
        providers
            .add(
                alo_models::Provider::checked(
                    "Mistral",
                    "https://api.mistral.ai",
                    alo_models::Region::Declared("the EU".to_owned()),
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        let picked = Picked::FromAProvider {
            provider: "Mistral".to_owned(),
            model: "a-model".to_owned(),
        };

        let asked = picked
            .asking(&providers, Some(&SourcePolicy::ThisMachineOnly))
            .unwrap();
        assert!(
            matches!(asked, Err(NotAllowed::NotThisMachine { .. })),
            "a rule permitting only this machine did not refuse a hosted provider: {asked:?}"
        );

        // And with no organisation, the same choice is permitted — so a refusal
        // from this crate is always one an administrator's rule caused.
        assert!(
            picked.asking(&providers, None).unwrap().is_ok(),
            "a machine with no policy refused a choice, so a refusal here would have no              administrator to name"
        );
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
