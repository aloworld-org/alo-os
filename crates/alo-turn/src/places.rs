//! Where a question may go on this machine, at the moment it is asked.
//!
//! Two things, and they are together because a turn needs both at once and
//! neither is the turn's own: the rule this machine is under, and every other
//! place the person has set up.
//!
//! # The rule is passed in, and it is the one in force now
//!
//! `alo_models::SourcePolicy` is an organisation's (ADR 0004), and it can be
//! tightened while a turn is open. So it is not held by [`crate::Machine`]
//! beside the indicator and the record: it arrives at the door, at the moment a
//! question would be put somewhere, which is item 3's rule about the grants
//! arriving at egress. A machine that borrowed the rule at the start of a turn
//! would be a machine nobody could tighten a rule on during one.
//!
//! # The others are for the offer, and for nothing else
//!
//! **Nothing in this crate reads them.** They are handed to `alo-answering`,
//! which makes offers out of them that only a person can take — the list is
//! what a person would be shown if the place they chose does not answer, not a
//! list of places to try. ADR 0008 is why that distinction is carried by a
//! field name and by a type rather than by a comment: a machine that fell back
//! would need no new type, only a second call in the same function.
//!
//! The order is the person's, as they set them up, because it is the order they
//! will be read in.

use alo_models::{InferenceSource, SourcePolicy};

/// The rule in force now, and everywhere else this machine could ask.
///
/// Borrowed for the length of one question. Nothing is copied out of it and
/// nothing is remembered from it: the next question asks again.
#[derive(Debug, Clone, Copy)]
pub struct Places<'a> {
    /// What this machine is set to permit.
    policy: &'a SourcePolicy,
    /// Every other place the person has set up, in their order.
    others: &'a [InferenceSource],
}

impl<'a> Places<'a> {
    /// A machine under this rule, with nowhere else set up.
    ///
    /// The ordinary shape of a machine somebody has just bought: one place a
    /// question goes, and a failure there is a failure with no offer beside it.
    #[must_use]
    pub fn under(policy: &'a SourcePolicy) -> Self {
        Self {
            policy,
            others: &[],
        }
    }

    /// The same, with every other place the person has set up.
    ///
    /// The one that just failed does not have to be left out — `alo-answering`
    /// removes it, and removes anything the rule forbids, so a caller handing
    /// over its whole list cannot accidentally offer somebody the place that
    /// has already not answered.
    #[must_use]
    pub fn and_everywhere_else(self, others: &'a [InferenceSource]) -> Self {
        Self { others, ..self }
    }

    /// What this machine is set to permit.
    #[must_use]
    pub fn policy(&self) -> &'a SourcePolicy {
        self.policy
    }

    /// Every other place the person has set up.
    #[must_use]
    pub fn everywhere_else(&self) -> &'a [InferenceSource] {
        self.others
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alo_models::Region;

    fn alo() -> InferenceSource {
        InferenceSource::Hosted {
            provider: "alo".to_owned(),
            region: Region::Declared("the EU".to_owned()),
        }
    }

    /// A machine with one place set up offers nothing else, and says so as an
    /// empty list rather than as an absence a caller has to interpret.
    #[test]
    fn a_machine_with_nowhere_else_set_up_has_nowhere_else() {
        let places = Places::under(&SourcePolicy::Anywhere);
        assert!(places.everywhere_else().is_empty());
        assert_eq!(places.policy(), &SourcePolicy::Anywhere);
    }

    /// The order is the person's, because it is the order they will read the
    /// offers in.
    #[test]
    fn everywhere_else_is_kept_in_the_order_the_person_set_them_up() {
        let others = [InferenceSource::ThisMachine, alo()];
        let places = Places::under(&SourcePolicy::InTheBuilding).and_everywhere_else(&others);
        assert_eq!(places.everywhere_else(), &others);
        assert_eq!(places.policy(), &SourcePolicy::InTheBuilding);
    }
}
