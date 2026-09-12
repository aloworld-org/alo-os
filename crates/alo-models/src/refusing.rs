//! Why this machine will not have a question answered where it was asked to be.
//!
//! [`SourcePolicy::permits`](crate::SourcePolicy::permits) answers yes or no.
//! This is the other half: when the answer is no, *which* rule said so and what
//! was asked for, carried as a value and worded when somebody reads it.
//!
//! # A refusal is decided without words and worded afterwards
//!
//! This is item 9e's decision reaching the second crate that decides something.
//! [`SourcePolicy`](crate::SourcePolicy) is asked before a socket is opened —
//! `Trying::under` asks it first and sends nothing at all if it refuses — and a
//! decision that had to be handed a `Strings` would be a decision that depends
//! on somebody having loaded a vocabulary. So the refusal carries the rule and
//! the place, and [`NotAllowed::said`] renders it where it is read.
//!
//! What that buys is the same thing it bought in `alo-capability`: the screen a
//! person is looking at and the record `alo-record` keeps render **the same
//! value**, so one of them cannot be English while the other is Latvian.
//!
//! # Three rules, four sentences, and no fifth
//!
//! [`SourcePolicy::Anywhere`](crate::SourcePolicy::Anywhere) permits
//! everything, so it has no variant here. That is worth saying because the
//! previous shape of this — a `String` with a branch that could not happen —
//! needed a sentence for a case that never arrives, and a repository that
//! forbids `unreachable!()` had written *"no policy forbids this"* to fill the
//! hole. There is now no hole: a policy that refuses nothing produces no
//! refusal, and the type says so.
//!
//! The rule naming a region has **two** sentences, because it refuses two
//! different things. A provider that said where it runs, and it is somewhere
//! else, is [`NotAllowed::OutsideTheRegion`]. A provider that has not said is
//! [`NotAllowed::RegionUnstated`], and its sentence says exactly that — never
//! that the provider runs elsewhere, which is the one thing nobody knows.
//! `docs/features.md` promises *reported as unknown, never assumed to be
//! nearby*, and a refusal that folded the two together would have the record
//! claiming a provider was outside a region when all that was true is that it
//! never said.

use alo_strings::{Filling, Said, Strings};

use crate::source::InferenceSource;
use crate::words;

/// Why a source is not permitted on this machine.
///
/// Carries the place that was asked about, so the sentence can name it rather
/// than telling somebody that something, somewhere, was not allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAllowed {
    /// The machine keeps questions in the building, and this would leave it.
    OutsideTheBuilding {
        /// Where the answer would have come from.
        source: InferenceSource,
    },
    /// The machine requires a named region, and this provider **said** it
    /// runs somewhere else.
    OutsideTheRegion {
        /// The region the organisation named, in their own words.
        region: String,
        /// Where the answer would have come from.
        source: InferenceSource,
    },
    /// The machine requires a named region, and this provider has **not said**
    /// where it runs.
    ///
    /// A value of its own rather than [`Self::OutsideTheRegion`] with a note,
    /// because the two are different facts and a record keeps whichever it is
    /// handed: *outside the region* is a claim about where the provider runs,
    /// and for this provider no such claim can be made. Unknown never
    /// satisfies a policy naming a region, and it is never reported as
    /// anything but unknown.
    RegionUnstated {
        /// The region the organisation named, in their own words.
        region: String,
        /// The provider, as the person named it — what the sentence names,
        /// since there is no place to name.
        provider: String,
        /// Where the answer would have come from: a hosted source whose region
        /// is [`crate::Region::Unknown`].
        source: InferenceSource,
    },
    /// The machine answers only on itself.
    NotThisMachine {
        /// Where the answer would have come from.
        source: InferenceSource,
    },
}

impl NotAllowed {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::OutsideTheBuilding { .. } => words::OUTSIDE_THE_BUILDING,
            Self::OutsideTheRegion { .. } => words::OUTSIDE_THE_REGION,
            Self::RegionUnstated { .. } => words::REGION_UNSTATED,
            Self::NotThisMachine { .. } => words::NOT_ON_THIS_MACHINE,
        }
    }

    /// Where the answer would have come from, whichever rule refused it.
    #[must_use]
    pub fn source(&self) -> &InferenceSource {
        match self {
            Self::OutsideTheBuilding { source, .. }
            | Self::OutsideTheRegion { source, .. }
            | Self::RegionUnstated { source, .. }
            | Self::NotThisMachine { source } => source,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::model_words`] answers with the
    /// key, marked, and `Said::is_a_bug`. **What is refused never depends on
    /// the string table** — the refusal was decided before this was called, and
    /// calling it cannot change the answer.
    ///
    /// The place goes in through
    /// [`and_said`](alo_strings::Filling::and_said) rather than as text, so a
    /// refusal somebody translated with an English clause still inside it is
    /// not reported as translated. That is item 11a's rule, and this sentence
    /// was written before there was a door for it.
    ///
    /// [`Self::RegionUnstated`] names the provider rather than the place,
    /// because there is no place: its clause would only say again that the
    /// provider has not said where it runs, and the sentence already does.
    /// A provider's name is data, never translated, so that sentence is as
    /// translated as its own words are.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::RegionUnstated {
                region, provider, ..
            } => Filling::of("region", region.clone()).and("provider", provider.clone()),
            Self::OutsideTheRegion { region, .. } => Filling::nothing()
                .and_said("source", &self.source().said(strings))
                .and("region", region.clone()),
            Self::OutsideTheBuilding { .. } | Self::NotThisMachine { .. } => {
                Filling::nothing().and_said("source", &self.source().said(strings))
            }
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::source::{Region, SourcePolicy};
    use crate::testing::{in_english, translated};

    fn somewhere() -> InferenceSource {
        InferenceSource::Hosted {
            provider: "someone".to_owned(),
            region: Region::Unknown,
        }
    }

    /// A refusal says what the machine is set to *and* what was asked for. A
    /// policy nobody can understand is a policy people work around.
    #[test]
    fn a_refusal_says_what_the_rule_is_and_what_was_asked_for() {
        let strings = in_english();
        let said = SourcePolicy::InRegion("the EU".to_owned())
            .refusal(&somewhere())
            .unwrap()
            .said(&strings);
        assert!(said.text().contains("inference in the EU only"), "{said}");
        assert!(said.text().contains("someone"), "{said}");
    }

    /// A provider that said it runs somewhere else.
    fn elsewhere() -> InferenceSource {
        InferenceSource::Hosted {
            provider: "someone".to_owned(),
            region: Region::Declared("the United States".to_owned()),
        }
    }

    /// **A provider that has not said where it runs is refused as unknown,
    /// and the sentence says so — not that it runs elsewhere.**
    ///
    /// The refusal path and the acceptance side by side: the same rule, one
    /// provider that said *the United States* and one that said nothing, and
    /// two different values with two different sentences. The one for the
    /// provider that said nothing names the provider and the region and says
    /// *has not said*; nothing in it claims to know where the provider is.
    #[test]
    fn a_provider_that_has_not_said_where_it_runs_is_refused_as_unknown_not_as_elsewhere() {
        let strings = in_english();
        let rule = SourcePolicy::InRegion("the EU".to_owned());

        let unstated = rule.refusal(&somewhere()).unwrap();
        assert_eq!(
            unstated,
            NotAllowed::RegionUnstated {
                region: "the EU".to_owned(),
                provider: "someone".to_owned(),
                source: somewhere(),
            }
        );
        let said = unstated.said(&strings);
        assert!(said.text().contains("inference in the EU only"), "{said}");
        assert!(
            said.text().contains("someone has not said where it runs"),
            "{said}"
        );
        assert!(said.text().contains("unknown"), "{said}");
        assert!(!said.text().contains("does not meet"), "{said}");
        assert!(!said.text().contains("outside"), "{said}");
        assert!(!said.text().contains("United States"), "{said}");

        // And the provider that did say is refused as what it said.
        let stated = rule.refusal(&elsewhere()).unwrap();
        assert!(
            matches!(stated, NotAllowed::OutsideTheRegion { .. }),
            "{stated:?}"
        );
        let said = stated.said(&strings);
        assert!(said.text().contains("in the United States"), "{said}");
        assert!(!said.text().contains("has not said"), "{said}");
    }

    /// **Unknown is a value of its own and never a region.** A rule naming a
    /// region literally called "unknown" — or nothing, or spaces — still refuses
    /// a provider that has not said, because [`Region::Unknown`] is not a
    /// region that happens to be spelled that way.
    #[test]
    fn unknown_is_not_a_region_that_happens_to_be_called_unknown() {
        for named in ["unknown", "Unknown", "", "  "] {
            let refusal = SourcePolicy::InRegion(named.to_owned()).refusal(&somewhere());
            assert!(
                matches!(refusal, Some(NotAllowed::RegionUnstated { .. })),
                "{named:?}: {refusal:?}"
            );
        }
    }

    /// **The sentence for a provider that has not said is read whole in the
    /// reader's language**, with the provider's name and the organisation's
    /// region carried as they were written.
    #[test]
    fn the_unstated_refusal_is_translated_whole_and_carries_the_names() {
        let strings = translated(&[(
            words::REGION_UNSTATED,
            "dieser Rechner ist auf Inferenz nur in {region} eingestellt, und {provider} hat \
             nicht gesagt, wo er läuft — unbekannt zählt nicht als dort",
        )]);
        let said = SourcePolicy::InRegion("der EU".to_owned())
            .refusal(&somewhere())
            .unwrap()
            .said(&strings);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().contains("nur in der EU"), "{said}");
        assert!(said.text().contains("someone hat nicht gesagt"), "{said}");
        assert!(!said.text().contains("has not said"), "{said}");
    }

    /// **A refusal and the place named inside it are in one language.** The
    /// source is described by this crate too, so a German machine does not read
    /// a German sentence with an English clause in the middle of it.
    #[test]
    fn a_refusal_and_the_place_it_names_are_in_one_language() {
        let strings = translated(&[
            (
                words::NOT_ON_THIS_MACHINE,
                "dieser Rechner ist so eingestellt, dass er nur selbst antwortet, und {source} ist \
                 anderswo",
            ),
            (
                words::BY_A_PROVIDER_SOMEWHERE,
                "von {provider}, der nicht gesagt hat, wo er läuft",
            ),
        ]);
        let said = SourcePolicy::ThisMachineOnly
            .refusal(&somewhere())
            .unwrap()
            .said(&strings);
        assert!(said.is_translated());
        assert!(said.text().contains("nicht gesagt hat"), "{said}");
        assert!(!said.text().contains("has not said"), "{said}");
        // The provider's name is the person's, not the language's.
        assert!(said.text().contains("someone"), "{said}");
    }

    /// **A sentence is only as translated as the clause inside it.** A refusal
    /// somebody translated, naming a place nobody has, is a German line with an
    /// English middle — and reporting it as translated would hide the one thing
    /// `alo-strings` exists to make visible.
    #[test]
    fn a_refusal_with_an_untranslated_place_in_it_does_not_claim_to_be_translated() {
        let strings = translated(&[(
            words::NOT_ON_THIS_MACHINE,
            "dieser Rechner ist so eingestellt, dass er nur selbst antwortet, und {source} ist \
             anderswo",
        )]);
        let said = SourcePolicy::ThisMachineOnly
            .refusal(&somewhere())
            .unwrap()
            .said(&strings);
        // The sentence itself is German, and the clause naming the provider is
        // not — so the line as a whole is not translated.
        assert!(
            matches!(said.came_from(), alo_strings::CameFrom::Translation(_)),
            "{said}"
        );
        assert!(!said.is_translated(), "{said}");
        assert!(said.text().contains("has not said where it runs"), "{said}");
    }

    /// **A refusal never depends on a string table.** With no words at all the
    /// policy refuses exactly what it refused before, and the answer names the
    /// rule by its key so whoever forgot to declare this crate's words finds
    /// out from the sentence rather than from a blank line.
    #[test]
    fn a_refusal_without_the_words_still_names_the_rule() {
        let nothing = Strings::of(alo_strings::Vocabulary::empty());
        let said = SourcePolicy::InTheBuilding
            .refusal(&somewhere())
            .unwrap()
            .said(&nothing);
        assert!(said.is_a_bug());
        assert!(
            said.text().contains("models.policy.outside-the-building"),
            "{said}"
        );
    }

    /// Every refusal carries the place, so nothing downstream has to be handed
    /// the source a second time to say what happened.
    #[test]
    fn every_refusal_carries_what_was_asked_about() {
        for policy in [
            SourcePolicy::InTheBuilding,
            SourcePolicy::ThisMachineOnly,
            SourcePolicy::InRegion("Switzerland".to_owned()),
        ] {
            let refusal = policy.refusal(&somewhere()).unwrap();
            assert_eq!(refusal.source(), &somewhere(), "{policy:?}");
        }
    }

    /// A policy that refuses nothing produces no refusal, and there is no
    /// variant standing in for one — which is the branch this file's header is
    /// about.
    #[test]
    fn a_policy_that_permits_everything_produces_no_refusal_at_all() {
        assert_eq!(SourcePolicy::Anywhere.refusal(&somewhere()), None);
        assert_eq!(
            SourcePolicy::InTheBuilding.refusal(&InferenceSource::ThisMachine),
            None
        );
    }
}
