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
//!
//! **Neither of the two things here that a person reads has a `Display`**
//! (item 9f). A `Display` is one `to_string()` away from a screen whose author
//! had no reason to think about language, so the only road to words is
//! [`InferenceSource::shown`] and [`SourcePolicy::refusal`], both of which need
//! the strings the reader in front of the machine actually reads.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::refusing::NotAllowed;
use crate::words;

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

    /// The string this crate declares for describing this place.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::ThisMachine => words::ON_THIS_MACHINE,
            Self::PairedMachine { .. } => words::ON_A_PAIRED_MACHINE,
            Self::Hosted {
                region: Region::Declared(_),
                ..
            } => words::BY_A_PROVIDER,
            Self::Hosted {
                region: Region::Unknown,
                ..
            } => words::BY_A_PROVIDER_SOMEWHERE,
        }
    }

    /// What to show a person **where the answer appears** — not in a settings
    /// page they would have to go looking for.
    ///
    /// Somebody about to paste a contract into a question is entitled to know
    /// where it is going before they paste it, and they are entitled to read it
    /// in their own language.
    ///
    /// A `String` rather than a [`Said`], because this is a clause: it is shown
    /// on its own beside an answer *and* it goes inside every refusal
    /// [`SourcePolicy`] makes, so a refusal and the place named in it are one
    /// language. That is `alo-capability`'s `Reach::shown` in this crate.
    ///
    /// **A sentence that puts this clause inside itself wants [`said`](Self::said)
    /// instead**, so that whether *it* was translated is part of whether the
    /// whole line was. A `String` cannot carry that, and a German refusal with
    /// an English clause in the middle of it would answer
    /// `Said::is_translated` with `true`.
    #[must_use]
    pub fn shown(&self, strings: &Strings) -> String {
        self.said(strings).into_text()
    }

    /// The same clause, carrying where its words came from.
    ///
    /// One rendering, not two: [`shown`](Self::shown) is this with the
    /// provenance dropped, for a caller putting the clause on a line of its
    /// own. Anything filling a gap in another sentence uses this one and
    /// `alo_strings::Filling::and_said`, because a sentence is only as
    /// translated as its least translated piece (item 11a).
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::ThisMachine => Filling::nothing(),
            Self::PairedMachine { machine } => Filling::of("machine", machine.clone()),
            Self::Hosted { provider, region } => {
                let named = Filling::of("provider", provider.clone());
                match region {
                    Region::Declared(where_) => named.and("region", where_.clone()),
                    Region::Unknown => named,
                }
            }
        };
        strings.say(&self.word().key(), &filling)
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

    /// Why a source is not permitted — the rule that refused it and the place
    /// it refused, as a value.
    ///
    /// **Not a sentence** (item 9f). Wording it here would mean handing this
    /// type a `Strings`, and then whether a question may be asked somewhere
    /// would depend on somebody having loaded a vocabulary. [`NotAllowed::said`]
    /// renders it where it is read, so the screen and the record cannot be two
    /// accounts of one moment.
    ///
    /// [`Anywhere`](Self::Anywhere) refuses nothing and so answers [`None`]
    /// without a branch of its own: there is no variant of [`NotAllowed`]
    /// standing in for a rule that never refuses.
    #[must_use]
    pub fn refusal(&self, source: &InferenceSource) -> Option<NotAllowed> {
        if self.permits(source) {
            return None;
        }
        match self {
            Self::Anywhere => None,
            Self::InTheBuilding => Some(NotAllowed::OutsideTheBuilding {
                source: source.clone(),
            }),
            Self::InRegion(region) => Some(NotAllowed::OutsideTheRegion {
                region: region.clone(),
                source: source.clone(),
            }),
            Self::ThisMachineOnly => Some(NotAllowed::NotThisMachine {
                source: source.clone(),
            }),
        }
    }
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
        let strings = crate::testing::in_english();
        let said = hosted("someone", Region::Unknown).shown(&strings);
        assert!(said.contains("has not said where it runs"), "{said}");

        let said = hosted("alo", eu()).shown(&strings);
        assert!(said.contains("in the EU"), "{said}");

        assert_eq!(
            InferenceSource::ThisMachine.shown(&strings),
            "on this machine"
        );
    }

    /// **Where an answer came from is read in the reader's own language**, and
    /// the parts of it that are not language — a provider's name, the region it
    /// stated — come through as they were written.
    #[test]
    fn where_an_answer_came_from_is_said_in_the_language_the_person_reads() {
        let strings = crate::testing::translated(&[
            (crate::words::BY_A_PROVIDER, "von {provider}, in {region}"),
            (
                crate::words::ON_A_PAIRED_MACHINE,
                "auf {machine}, in Ihrem Netzwerk",
            ),
        ]);
        let said = hosted("alo", eu()).shown(&strings);
        assert!(said.starts_with("von alo"), "{said}");
        assert!(said.contains("the EU"), "{said}");

        let said = InferenceSource::PairedMachine {
            machine: "the studio workstation".to_owned(),
        }
        .shown(&strings);
        assert!(said.contains("Ihrem Netzwerk"), "{said}");
        assert!(said.contains("the studio workstation"), "{said}");
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
    /// people work around — and the words are `refusing.rs`'s, so what is
    /// checked here is that the right rule refused and that it carries what it
    /// refused.
    #[test]
    fn a_refusal_names_the_rule_and_carries_what_was_asked_for() {
        let somewhere = hosted("someone", Region::Unknown);
        let refusal = SourcePolicy::InRegion("the EU".to_owned()).refusal(&somewhere);
        assert_eq!(
            refusal,
            Some(NotAllowed::OutsideTheRegion {
                region: "the EU".to_owned(),
                source: somewhere.clone(),
            })
        );
        assert_eq!(SourcePolicy::Anywhere.refusal(&somewhere), None);
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
            assert!(policy.permits(&source), "{source:?}");
            assert!(policy.refusal(&source).is_none(), "{source:?}");
        }
    }
}
