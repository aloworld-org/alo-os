//! What an organisation permits to leave.
//!
//! The rule and nothing else: which egress is permitted, and — when one is not
//! — which rule refused it, as a value. What that refusal *says* is
//! [`crate::refusing`]'s, because a policy asked before a socket opens must not
//! depend on a vocabulary having been loaded.
//!
//! [ADR 0001](../../../docs/decisions/0001-the-capability-model.md) §8: *egress
//! policy lives in a Rust service, not a settings checkbox.* This file is that
//! service's decision, separated from the machinery that enforces it so it can
//! be read and argued with on its own.
//!
//! **The default permits everything, and that is the decision rather than an
//! oversight**, exactly as it is for [`SourcePolicy`]. Where a person's machine
//! reaches is their business; alo OS ships the mechanism for an organisation
//! that *has* a rule to state it, and never a default that decides for them.
//! "Built in Europe, not only for Europe" is a sentence in `CLAUDE.md` and this
//! enum is where it either is or is not true: the region is a name the customer
//! gives, not a list we shipped.
//!
//! **It is one rule, stated once.** `alo-models` already decides where a
//! *question* may be answered, and a machine whose two egress rules could
//! disagree would be a machine nobody could describe. So an [`EgressPolicy`] is
//! made from a [`SourcePolicy`] rather than written down a second time, and
//! that the two agree about every source there is, is a test rather than an
//! intention.

use alo_models::SourcePolicy;

use crate::leaving::Leaving;
use crate::refusing::{NotPermitted, Refusal};

/// What may leave this machine
/// ([ADR 0004](../../../docs/decisions/0004-the-organisations-machine.md)
/// §policy).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EgressPolicy {
    /// Wherever the person's machine needs to reach. The default, everywhere,
    /// unless an organisation has said otherwise on a machine it manages.
    #[default]
    Anywhere,
    /// Paired machines only — nothing leaves the building.
    InTheBuilding,
    /// Anywhere declared to be in the named region. The region is the
    /// organisation's to name: "the EU", "Switzerland", "the United States".
    InRegion(String),
    /// Nothing leaves at all.
    NothingLeaves,
}

impl EgressPolicy {
    /// Whether this egress is permitted.
    #[must_use]
    pub fn permits(&self, leaving: &Leaving) -> bool {
        match self {
            Self::Anywhere => true,
            Self::InTheBuilding => leaving.destination().stays_in_the_building(),
            Self::InRegion(region) => leaving.destination().is_in(region),
            Self::NothingLeaves => false,
        }
    }

    /// Why this egress is not permitted — the rule that refused it and the
    /// egress it refused, as a value.
    ///
    /// `None` when it is permitted, so this is the refusal and the question in
    /// one, and there is no second decision that could disagree with the first.
    ///
    /// **Not a sentence** (item 9h, following `alo-models` in 9f). Wording it
    /// here would mean handing this type a `Strings`, and then whether a
    /// connection may open would depend on somebody having loaded a vocabulary.
    /// [`NotPermitted::said`] renders it where it is read, so the screen and
    /// the record cannot be two accounts of one moment — a policy nobody can
    /// understand is still a policy people work around, and this is where that
    /// is answered rather than where it is worded.
    ///
    /// [`Anywhere`](Self::Anywhere) refuses nothing and so answers [`None`]
    /// without a branch of its own: there is no variant of [`Refusal`] standing
    /// in for a rule that never refuses.
    #[must_use]
    pub fn refusal(&self, leaving: &Leaving) -> Option<NotPermitted> {
        if self.permits(leaving) {
            return None;
        }
        let why = match self {
            Self::Anywhere => return None,
            Self::InTheBuilding => Refusal::OutsideTheBuilding,
            Self::InRegion(region) => Refusal::OutsideTheRegion {
                region: region.clone(),
            },
            Self::NothingLeaves => Refusal::NothingMayLeave,
        };
        Some(NotPermitted::new(leaving.clone(), why))
    }
}

impl From<&SourcePolicy> for EgressPolicy {
    /// The one rule an organisation stated, applied to everything an agent can
    /// cause rather than only to where a question is answered.
    fn from(policy: &SourcePolicy) -> Self {
        match policy {
            SourcePolicy::Anywhere => Self::Anywhere,
            SourcePolicy::InTheBuilding => Self::InTheBuilding,
            SourcePolicy::InRegion(region) => Self::InRegion(region.clone()),
            SourcePolicy::ThisMachineOnly => Self::NothingLeaves,
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
    use crate::destination::Destination;
    use crate::leaving::Why;
    use alo_capability::Grantee;
    use alo_models::{InferenceSource, Region};

    fn mail() -> Grantee {
        Grantee::named("@mail")
    }

    fn eu() -> Region {
        Region::Declared("the EU".to_owned())
    }

    fn to(destination: Destination) -> Leaving {
        Leaving::because(&mail(), Why::Sending, destination)
    }

    fn everywhere() -> [Leaving; 4] {
        [
            to(Destination::paired("the studio workstation").unwrap()),
            to(Destination::provider("alo", eu()).unwrap()),
            to(Destination::provider("someone", Region::Unknown).unwrap()),
            to(Destination::at("alo.example").unwrap()),
        ]
    }

    /// The default permits whatever the person's machine needs to reach. This
    /// type exists so an organisation that has a rule can state it, not so alo
    /// OS can have one.
    #[test]
    fn the_default_permits_whatever_the_person_needs() {
        assert_eq!(EgressPolicy::default(), EgressPolicy::Anywhere);
        for leaving in everywhere() {
            assert!(EgressPolicy::default().permits(&leaving), "{leaving:?}");
            assert!(EgressPolicy::default().refusal(&leaving).is_none());
        }
    }

    /// In the building means paired machines and nothing else — not a provider,
    /// and not a host that happens to answer on the same wire.
    #[test]
    fn in_the_building_permits_a_paired_machine_and_nothing_else() {
        let policy = EgressPolicy::InTheBuilding;
        assert!(policy.permits(&to(Destination::paired("the box").unwrap())));
        assert!(!policy.permits(&to(Destination::provider("alo", eu()).unwrap())));
        assert!(!policy.permits(&to(Destination::at("files.local").unwrap())));
    }

    /// The region is whatever an organisation names, and it is satisfied by a
    /// declaration rather than by a plausible address.
    #[test]
    fn any_region_can_be_required_and_only_a_declaration_satisfies_it() {
        let swiss = to(Destination::provider(
            "a swiss provider",
            Region::Declared("Switzerland".to_owned()),
        )
        .unwrap());
        assert!(EgressPolicy::InRegion("Switzerland".to_owned()).permits(&swiss));
        assert!(!EgressPolicy::InRegion("the EU".to_owned()).permits(&swiss));

        let policy = EgressPolicy::InRegion("the EU".to_owned());
        assert!(policy.permits(&to(Destination::provider("alo", eu()).unwrap())));
        assert!(!policy.permits(&to(Destination::at("mail.eu").unwrap())));
        assert!(
            policy.permits(&to(Destination::paired("the box").unwrap())),
            "a person's own machines are in their region wherever that is"
        );
    }

    /// **Nothing leaves means nothing leaves**, not even the next room. This is
    /// the policy a machine with a local model can run all day under, and the
    /// one law 1's zero-egress measurement is taken beneath.
    #[test]
    fn nothing_leaves_permits_nothing_at_all() {
        let policy = EgressPolicy::NothingLeaves;
        for leaving in everywhere() {
            assert!(!policy.permits(&leaving), "{leaving:?}");
            assert!(policy.refusal(&leaving).is_some(), "{leaving:?}");
        }
    }

    /// **One rule, stated once.** The wider boundary and the inference boundary
    /// agree about every source there is — a machine whose two egress rules
    /// could disagree would be a machine nobody could describe.
    #[test]
    fn the_wider_boundary_agrees_with_the_inference_one_about_every_source() {
        let sources = [
            InferenceSource::ThisMachine,
            InferenceSource::PairedMachine {
                machine: "the studio workstation".to_owned(),
            },
            InferenceSource::Hosted {
                provider: "alo".to_owned(),
                region: eu(),
            },
            InferenceSource::Hosted {
                provider: "someone".to_owned(),
                region: Region::Unknown,
            },
        ];
        for source in &sources {
            for sources_policy in [
                SourcePolicy::Anywhere,
                SourcePolicy::InTheBuilding,
                SourcePolicy::InRegion("the EU".to_owned()),
                SourcePolicy::ThisMachineOnly,
            ] {
                let egress = EgressPolicy::from(&sources_policy);
                match Leaving::asking(&mail(), source) {
                    Ok(leaving) => assert_eq!(
                        egress.permits(&leaving),
                        sources_policy.permits(source),
                        "{sources_policy:?} disagreed about {source:?}"
                    ),
                    // A question answered here never departs, so there is
                    // nothing for the wider boundary to decide about — and the
                    // narrower one permits it under every policy there is.
                    Err(_) => assert!(sources_policy.permits(source), "{sources_policy:?}"),
                }
            }
        }
    }

    /// A refusal says which rule refused and carries what it refused — the
    /// words are `refusing.rs`'s, and are asked for where somebody reads them.
    #[test]
    fn a_refusal_names_the_rule_and_carries_what_it_refused() {
        let leaving = to(Destination::provider("someone", Region::Unknown).unwrap());
        let refused = EgressPolicy::InRegion("the EU".to_owned())
            .refusal(&leaving)
            .unwrap();
        assert_eq!(
            refused.why(),
            &Refusal::OutsideTheRegion {
                region: "the EU".to_owned()
            }
        );
        assert_eq!(refused.leaving(), &leaving);
    }

    /// **A policy that refuses nothing produces no refusal**, and there is no
    /// variant standing in for one. The previous shape of this needed a
    /// sentence for a state nobody could reach, and wrote *"no policy forbids
    /// this"* to fill the hole.
    #[test]
    fn a_policy_that_permits_everything_produces_no_refusal_at_all() {
        for leaving in everywhere() {
            assert_eq!(EgressPolicy::Anywhere.refusal(&leaving), None);
        }
        assert_eq!(
            EgressPolicy::InTheBuilding.refusal(&to(Destination::paired("the box").unwrap())),
            None
        );
    }
}
