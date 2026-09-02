//! Where a question is answered, and what that costs in egress.
//!
//! [ADR 0008](../../../docs/decisions/0008-where-inference-happens.md) makes the
//! location of inference an explicit property rather than a configuration
//! detail, because law 1 says nothing leaves silently and sending a question to
//! a hosted API is the largest egress this product will ever cause.
//!
//! The type exists so that the question "where did that answer come from?"
//! cannot be answered vaguely anywhere in the system. A base URL cannot answer
//! it — `https://…` is equally an appliance in the next room and a provider on
//! another continent — so the kind of place is carried explicitly and travels
//! with every answer.

use std::fmt;

/// Where a provider runs, as **the provider states it** — never inferred.
///
/// Guessing from a domain name is how a customer ends up in breach while
/// looking at a reassuring label, so a provider that has not said is
/// [`Region::Unknown`], and unknown never satisfies a policy that requires the
/// EU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Region {
    /// Declared to run inside the EU or EEA.
    Eu,
    /// Declared to run somewhere else, named as the provider names it.
    Elsewhere(String),
    /// The provider has not said. Not a synonym for "probably fine".
    Unknown,
}

/// One of the three places a model may answer from (ADR 0008).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferenceSource {
    /// The weights are on this machine. Nothing leaves.
    ThisMachine,
    /// A machine on this network, paired deliberately (ADR 0003). The question
    /// leaves this machine and stays in the building.
    PairedMachine {
        /// The paired machine's name, as a person named it when pairing.
        machine: String,
    },
    /// A hosted service. The question leaves the building.
    Hosted {
        /// The provider, as a person would say it — "alo", not a hostname.
        provider: String,
        /// Where that provider says it runs.
        region: Region,
    },
}

impl InferenceSource {
    /// Whether asking this source causes anything to leave the machine.
    ///
    /// True for a paired machine as well as a hosted service. "It only went
    /// down the corridor" is exactly the exception that would erode law 1, so
    /// the indicator fires for both and the difference is said in words rather
    /// than by staying silent.
    #[must_use]
    pub fn causes_egress(&self) -> bool {
        !matches!(self, Self::ThisMachine)
    }

    /// Whether this source keeps the question inside the organisation's own
    /// building or network.
    #[must_use]
    pub fn stays_in_the_building(&self) -> bool {
        matches!(self, Self::ThisMachine | Self::PairedMachine { .. })
    }

    /// Whether this satisfies a policy of EU-only inference.
    ///
    /// Local and paired machines qualify by being where the customer is. A
    /// hosted service qualifies only when the provider has **declared** the EU:
    /// [`Region::Unknown`] does not, on purpose.
    #[must_use]
    pub fn is_eu_only(&self) -> bool {
        match self {
            Self::ThisMachine | Self::PairedMachine { .. } => true,
            Self::Hosted { region, .. } => *region == Region::Eu,
        }
    }

    /// What to show a person **where the answer appears** — not in a settings
    /// page they would have to go looking for.
    ///
    /// Somebody about to paste a contract into a question is entitled to know
    /// where it is going before they paste it.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::ThisMachine => "on this machine".to_owned(),
            Self::PairedMachine { machine } => format!("on {machine}, on your network"),
            Self::Hosted { provider, region } => match region {
                Region::Eu => format!("by {provider}, in the EU"),
                Region::Elsewhere(where_) => format!("by {provider}, in {where_}"),
                Region::Unknown => {
                    format!("by {provider}, which has not said where it runs")
                }
            },
        }
    }
}

impl fmt::Display for InferenceSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.describe())
    }
}

/// What an organisation permits on a machine (ADR 0004 §policy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourcePolicy {
    /// Anything the person chooses. The default on a personal machine.
    #[default]
    Anywhere,
    /// Local and paired machines only — nothing leaves the building.
    InTheBuilding,
    /// Anything that is declared to run in the EU, hosted included.
    EuOnly,
    /// This machine only. Nothing leaves at all.
    ThisMachineOnly,
}

impl SourcePolicy {
    /// Whether a source is permitted under this policy.
    #[must_use]
    pub fn permits(&self, source: &InferenceSource) -> bool {
        match self {
            Self::Anywhere => true,
            Self::InTheBuilding => source.stays_in_the_building(),
            Self::EuOnly => source.is_eu_only(),
            Self::ThisMachineOnly => matches!(source, InferenceSource::ThisMachine),
        }
    }

    /// Why a source is not permitted, in words a person on this machine can
    /// read — a policy nobody can understand is a policy people work around.
    #[must_use]
    pub fn refusal(&self, source: &InferenceSource) -> Option<String> {
        if self.permits(source) {
            return None;
        }
        Some(match self {
            Self::Anywhere => unreachable_policy(),
            Self::InTheBuilding => format!(
                "this machine is set to keep questions in the building, and {} would send this one outside it",
                source.describe()
            ),
            Self::EuOnly => format!(
                "this machine is set to use EU inference only, and {} does not meet that",
                source.describe()
            ),
            Self::ThisMachineOnly => format!(
                "this machine is set to answer only on itself, and {} is somewhere else",
                source.describe()
            ),
        })
    }
}

/// `Anywhere` permits everything, so this branch is unreachable — but law 2's
/// repository has no `unreachable!()` either, so it says something true instead.
fn unreachable_policy() -> String {
    "no policy forbids this".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hosted(provider: &str, region: Region) -> InferenceSource {
        InferenceSource::Hosted {
            provider: provider.to_owned(),
            region,
        }
    }

    /// Law 1: a paired machine is egress too. This is the exception somebody
    /// will one day argue for, so it is a test rather than a sentence.
    #[test]
    fn a_paired_machine_is_still_egress() {
        let paired = InferenceSource::PairedMachine {
            machine: "the studio workstation".to_owned(),
        };
        assert!(paired.causes_egress());
        assert!(paired.stays_in_the_building());
        assert!(!InferenceSource::ThisMachine.causes_egress());
    }

    /// A provider that has not said where it runs is not treated as if it had.
    #[test]
    fn a_provider_that_has_not_said_where_it_runs_does_not_satisfy_eu_only() {
        assert!(!hosted("someone", Region::Unknown).is_eu_only());
        assert!(hosted("alo", Region::Eu).is_eu_only());
        assert!(!hosted("someone", Region::Elsewhere("the US".to_owned())).is_eu_only());
    }

    /// The description is what a person reads before pasting a contract into a
    /// question, so it must say the uncomfortable thing plainly.
    #[test]
    fn an_undeclared_provider_says_so_rather_than_sounding_safe() {
        let said = hosted("someone", Region::Unknown).describe();
        assert!(said.contains("has not said where it runs"), "{said}");

        let eu = hosted("alo", Region::Eu).describe();
        assert!(eu.contains("in the EU"), "{eu}");

        assert_eq!(InferenceSource::ThisMachine.describe(), "on this machine");
    }

    #[test]
    fn a_policy_of_in_the_building_permits_a_paired_machine_and_no_provider() {
        let policy = SourcePolicy::InTheBuilding;
        assert!(policy.permits(&InferenceSource::ThisMachine));
        assert!(policy.permits(&InferenceSource::PairedMachine {
            machine: "box".to_owned()
        }));
        assert!(!policy.permits(&hosted("alo", Region::Eu)));
    }

    #[test]
    fn eu_only_permits_a_declared_eu_provider_but_not_an_undeclared_one() {
        let policy = SourcePolicy::EuOnly;
        assert!(policy.permits(&hosted("alo", Region::Eu)));
        assert!(!policy.permits(&hosted("someone", Region::Unknown)));
        assert!(policy.permits(&InferenceSource::ThisMachine));
    }

    #[test]
    fn this_machine_only_permits_nothing_else_not_even_the_next_room() {
        let policy = SourcePolicy::ThisMachineOnly;
        assert!(policy.permits(&InferenceSource::ThisMachine));
        assert!(!policy.permits(&InferenceSource::PairedMachine {
            machine: "box".to_owned()
        }));
        assert!(!policy.permits(&hosted("alo", Region::Eu)));
    }

    /// A refusal explains itself. A policy nobody can understand is a policy
    /// people work around.
    #[test]
    fn a_refusal_says_what_the_policy_is_and_what_was_asked_for() {
        let refusal = SourcePolicy::EuOnly
            .refusal(&hosted("someone", Region::Unknown))
            .unwrap_or_default();
        assert!(refusal.contains("EU inference only"), "{refusal}");
        assert!(refusal.contains("someone"), "{refusal}");
        assert!(
            SourcePolicy::Anywhere
                .refusal(&hosted("someone", Region::Unknown))
                .is_none()
        );
    }

    /// The default is a personal machine's default: nobody above the person.
    #[test]
    fn the_default_policy_permits_what_the_person_chooses() {
        assert_eq!(SourcePolicy::default(), SourcePolicy::Anywhere);
    }
}
