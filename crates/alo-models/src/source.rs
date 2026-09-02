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

use serde::{Deserialize, Serialize};

/// Where a provider runs, as **the provider states it** — never inferred.
///
/// Deliberately not an enum of places. alo OS is built in Europe and is not
/// only for Europe: a hospital in Ohio and a bank in Singapore have the same
/// problem and the same right to name the region they must stay inside. A type
/// that knew only about the EU would make everybody else a special case.
///
/// Guessing from a domain name is how a customer ends up in breach while
/// looking at a reassuring label, so a provider that has not said is
/// [`Region::Unknown`], and unknown never satisfies a policy that names a
/// region.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Region {
    /// Declared by the provider, in the provider's own words — "the EU",
    /// "Switzerland", "the United States".
    Declared(String),
    /// The provider has not said. Not a synonym for "probably fine".
    Unknown,
}

impl Region {
    /// Whether this is the named region. Compared case-insensitively on the
    /// stated name, because "the EU" and "the eu" are the same promise.
    #[must_use]
    pub fn is(&self, region: &str) -> bool {
        match self {
            Self::Declared(said) => said.eq_ignore_ascii_case(region.trim()),
            Self::Unknown => false,
        }
    }
}

/// One of the three places a model may answer from (ADR 0008).
///
/// It is written down and read back, because "where did that answer come from"
/// has to be answerable afterwards and not only at the moment the answer
/// appeared — that is what `alo-record` keeps. Reading one back grants nothing
/// and reaches nothing: it names a place, and naming a place is all it does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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

    /// Whether this satisfies a policy naming a region.
    ///
    /// Local and paired machines qualify wherever the customer is: their
    /// machines are in their region by definition. A hosted service qualifies
    /// only when the provider has **declared** that region;
    /// [`Region::Unknown`] never does, on purpose.
    #[must_use]
    pub fn is_in(&self, region: &str) -> bool {
        match self {
            Self::ThisMachine | Self::PairedMachine { .. } => true,
            Self::Hosted {
                region: declared, ..
            } => declared.is(region),
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
                Region::Declared(where_) => format!("by {provider}, in {where_}"),
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
///
/// **The default permits everything, and that is deliberate.** Which provider
/// somebody uses is their decision — Mistral, alo, their own endpoint, or
/// nothing at all. This type exists so an organisation that *has* a rule can
/// state it and have it enforced, not so that alo OS can have one of its own.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SourcePolicy {
    /// Anything the person chooses. The default, everywhere, unless an
    /// organisation has said otherwise on a machine it manages.
    #[default]
    Anywhere,
    /// Local and paired machines only — nothing leaves the building.
    InTheBuilding,
    /// Anything declared to run in the named region, hosted included. The
    /// region is the organisation's to name: "the EU", "Switzerland",
    /// "the United States".
    InRegion(String),
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
            Self::InRegion(region) => source.is_in(region),
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
            Self::InRegion(region) => format!(
                "this machine is set to use inference in {region} only, and {} does not meet that",
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

    fn eu() -> Region {
        Region::Declared("the EU".to_owned())
    }

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
    fn a_provider_that_has_not_said_where_it_runs_satisfies_no_region() {
        assert!(!hosted("someone", Region::Unknown).is_in("the EU"));
        assert!(hosted("alo", eu()).is_in("the EU"));
        assert!(!hosted("someone", Region::Declared("the US".to_owned())).is_in("the EU"));
    }

    /// The region is whatever an organisation names, not a list we shipped.
    /// alo OS is built in Europe and is not only for Europe.
    #[test]
    fn any_region_can_be_required_not_only_the_eu() {
        let swiss = hosted(
            "a swiss provider",
            Region::Declared("Switzerland".to_owned()),
        );
        assert!(SourcePolicy::InRegion("Switzerland".to_owned()).permits(&swiss));
        assert!(!SourcePolicy::InRegion("the EU".to_owned()).permits(&swiss));

        let us = hosted(
            "a us provider",
            Region::Declared("the United States".to_owned()),
        );
        assert!(SourcePolicy::InRegion("the United States".to_owned()).permits(&us));
    }

    /// A person's own machines are in their region wherever that is, so a
    /// regional policy never locks somebody out of their own hardware.
    #[test]
    fn your_own_machines_satisfy_any_region() {
        let policy = SourcePolicy::InRegion("Singapore".to_owned());
        assert!(policy.permits(&InferenceSource::ThisMachine));
        assert!(policy.permits(&InferenceSource::PairedMachine {
            machine: "the office box".to_owned()
        }));
    }

    /// The description is what a person reads before pasting a contract into a
    /// question, so it must say the uncomfortable thing plainly.
    #[test]
    fn an_undeclared_provider_says_so_rather_than_sounding_safe() {
        let said = hosted("someone", Region::Unknown).describe();
        assert!(said.contains("has not said where it runs"), "{said}");

        let said = hosted("alo", eu()).describe();
        assert!(said.contains("in the EU"), "{said}");

        assert_eq!(InferenceSource::ThisMachine.describe(), "on this machine");
    }

    #[test]
    fn a_policy_of_in_the_building_permits_a_paired_machine_and_no_provider() {
        let policy = SourcePolicy::InTheBuilding;
        assert!(policy.permits(&InferenceSource::ThisMachine));
        assert!(policy.permits(&InferenceSource::PairedMachine {
            machine: "box".to_owned()
        }));
        assert!(!policy.permits(&hosted("alo", eu())));
    }

    #[test]
    fn a_named_region_permits_a_declared_provider_but_not_an_undeclared_one() {
        let policy = SourcePolicy::InRegion("the EU".to_owned());
        assert!(policy.permits(&hosted("alo", eu())));
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
        assert!(!policy.permits(&hosted("alo", eu())));
    }

    /// A refusal explains itself. A policy nobody can understand is a policy
    /// people work around.
    #[test]
    fn a_refusal_says_what_the_policy_is_and_what_was_asked_for() {
        let refusal = SourcePolicy::InRegion("the EU".to_owned())
            .refusal(&hosted("someone", Region::Unknown))
            .unwrap_or_default();
        assert!(refusal.contains("inference in the EU only"), "{refusal}");
        assert!(refusal.contains("someone"), "{refusal}");
        assert!(
            SourcePolicy::Anywhere
                .refusal(&hosted("someone", Region::Unknown))
                .is_none()
        );
    }

    /// Where an answer came from outlives the answer, so a source has to
    /// survive being written down and read back — still saying the same thing
    /// about egress and about the region it satisfies.
    #[test]
    fn a_source_survives_being_written_down_and_read_back() {
        for source in [
            InferenceSource::ThisMachine,
            InferenceSource::PairedMachine {
                machine: "the studio workstation".to_owned(),
            },
            hosted("alo", eu()),
            hosted("someone", Region::Unknown),
        ] {
            let written = serde_json::to_string(&source).unwrap_or_default();
            let read = serde_json::from_str::<InferenceSource>(&written).ok();
            assert_eq!(read.as_ref(), Some(&source), "{written}");
            assert_eq!(
                read.map(|read| read.causes_egress()),
                Some(source.causes_egress()),
                "{written}"
            );
        }
    }

    /// The default permits everything, and that is the decision rather than an
    /// oversight: which provider somebody uses is theirs to choose. This type
    /// exists so an organisation that has a rule can state it, not so alo OS
    /// can have one.
    #[test]
    fn the_default_policy_permits_whatever_the_person_chooses() {
        assert_eq!(SourcePolicy::default(), SourcePolicy::Anywhere);
        let policy = SourcePolicy::default();
        for source in [
            InferenceSource::ThisMachine,
            hosted("mistral", Region::Declared("France".to_owned())),
            hosted("somebody", Region::Unknown),
        ] {
            assert!(policy.permits(&source), "{source}");
            assert!(policy.refusal(&source).is_none(), "{source}");
        }
    }
}
