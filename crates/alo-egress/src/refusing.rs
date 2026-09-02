//! Why an egress was not permitted, and what a person is told about it.
//!
//! [`EgressPolicy::permits`](crate::EgressPolicy) answers yes or no.
//! [`EgressPolicy::refusal`](crate::EgressPolicy::refusal) answers the other
//! half: which rule said so and what it stopped, carried as a value and worded
//! when somebody reads it.
//!
//! # A refusal is decided without words and worded afterwards
//!
//! This is item 9e's decision reaching the last crate that decides something,
//! and it matters more here than anywhere it has been so far.
//! [`EgressPolicy`](crate::EgressPolicy) is asked **before a socket is opened** — [`Indicator::beginning`] asks it and
//! hands back a [`Departing`](crate::Departing) or nothing at all — so a policy
//! that had to be handed a `Strings` would be a policy that depends on somebody
//! having loaded a vocabulary. A machine whose egress rules stop working
//! because a translation failed to load is not a machine sold on sovereignty.
//!
//! So the refusal carries the rule and the egress, and [`NotPermitted::said`]
//! renders it where it is read. What that buys is what it bought in
//! `alo-capability` and `alo-models`: the screen a person is looking at and the
//! record `alo-record` keeps render **the same value**, so one of them cannot be
//! English while the other is Latvian.
//!
//! # Three rules, three sentences, and no fourth
//!
//! [`EgressPolicy::Anywhere`](crate::EgressPolicy::Anywhere) permits
//! everything, so it has no variant here. That is worth saying because the
//! previous shape of this — a `String` with a branch that could not happen —
//! needed a sentence for a case that never arrives, and a repository that
//! forbids `unreachable!()` had written *"no policy forbids this"* to fill the
//! hole. There is now no hole: a policy that refuses nothing produces no
//! refusal, and the type says so.
//!
//! # Why the rule and the refusal are two types
//!
//! [`Refusal`] is which rule refused. [`NotPermitted`] is that rule together
//! with the egress it stopped, and it is the only one of the two that can be
//! worded, because every one of the three sentences names where the thing was
//! going and that place lives in the [`Leaving`].
//!
//! They are two rather than one for a reason that outlives the sentence.
//! `alo-record` writes a held-back entry from a [`NotPermitted`] and from
//! nothing else (item 5a), so a refusal the policy never made must not be
//! constructible — which is why this type has fields nobody outside the crate
//! can set and a `pub(crate)` constructor the policy alone calls. An enum
//! carrying the egress in each variant would have been one type, and every one
//! of its variants would have been a way to write down a refusal that never
//! happened.
//!
//! [`Indicator::beginning`]: crate::Indicator::beginning

use alo_strings::{Filling, Said, Strings};

use crate::leaving::Leaving;
use crate::words;

/// Which rule refused an egress.
///
/// Carries only what the rule itself said, because the place it refused is in
/// the [`Leaving`] beside it and one moment described twice is two things that
/// can disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The machine keeps everything in the building, and this would leave it.
    OutsideTheBuilding,
    /// The machine reaches a named region only, and this does not meet it.
    OutsideTheRegion {
        /// The region the organisation named, in their own words.
        region: String,
    },
    /// The machine lets nothing leave at all.
    NothingMayLeave,
}

impl Refusal {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::OutsideTheBuilding => words::OUTSIDE_THE_BUILDING,
            Self::OutsideTheRegion { .. } => words::OUTSIDE_THE_REGION,
            Self::NothingMayLeave => words::NOTHING_MAY_LEAVE,
        }
    }
}

/// An egress that was refused, and what it was a refusal of.
///
/// The egress comes back for the same reason [`alo_capability::Refused`]
/// carries its call: a refusal is recorded, and one that threw away what it
/// refused could only say that something was stopped.
///
/// Deliberately **not** a `std::error::Error`, which is item 9b's rule reaching
/// the last crate that held English: an `Error` is one `to_string()` from a
/// screen whose author had no reason to think about language. The only road to
/// words is [`NotPermitted::said`], and every answer it gives says whether
/// anybody translated it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotPermitted {
    /// What was refused.
    leaving: Leaving,
    /// Which rule refused it.
    why: Refusal,
}

impl NotPermitted {
    /// A refusal of this egress, by this rule.
    ///
    /// The policy's alone. See this file's header: a refusal that could be
    /// written by anybody would be a held-back entry in the record that nothing
    /// actually stopped.
    pub(crate) fn new(leaving: Leaving, why: Refusal) -> Self {
        Self { leaving, why }
    }

    /// What was refused.
    #[must_use]
    pub fn leaving(&self) -> &Leaving {
        &self.leaving
    }

    /// Which rule refused it.
    #[must_use]
    pub fn why(&self) -> &Refusal {
        &self.why
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::egress_words`] answers with the
    /// key, marked, and `Said::is_a_bug`. **What is refused never depends on
    /// the string table** — the refusal was decided before this was called, and
    /// calling it cannot change the answer.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = Filling::of("destination", self.leaving.destination().shown(strings));
        let filling = match &self.why {
            Refusal::OutsideTheRegion { region } => filling.and("region", region.clone()),
            Refusal::OutsideTheBuilding | Refusal::NothingMayLeave => filling,
        };
        strings.say(&self.why.word().key(), &filling)
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
    use crate::policy::EgressPolicy;
    use crate::testing::{in_english, translated};
    use alo_capability::Grantee;
    use alo_models::Region;

    fn to_someone() -> Leaving {
        Leaving::because(
            &Grantee::named("@mail"),
            Why::Asking,
            Destination::provider("someone", Region::Unknown).unwrap(),
        )
    }

    /// A refusal says what the machine is set to *and* where the thing was
    /// going. A policy nobody can understand is a policy people work around.
    #[test]
    fn a_refusal_says_what_the_rule_is_and_where_it_was_going() {
        let strings = in_english();
        let said = EgressPolicy::InRegion("the EU".to_owned())
            .refusal(&to_someone())
            .unwrap()
            .said(&strings);
        assert!(said.text().contains("reach the EU only"), "{said}");
        assert!(said.text().contains("someone"), "{said}");
        assert!(said.text().contains("has not said where it runs"), "{said}");
    }

    /// **A refusal and the place named inside it are in one language.** The
    /// destination is described by this crate too, so a German machine does not
    /// read a German sentence with an English clause in the middle of it.
    #[test]
    fn a_refusal_and_the_place_it_names_are_in_one_language() {
        let strings = translated(&[
            (
                words::NOTHING_MAY_LEAVE,
                "dieser Rechner ist so eingestellt, dass nichts ihn verlässt, und {destination} \
                 ist anderswo",
            ),
            (
                words::A_PROVIDER_SOMEWHERE,
                "{provider}, der nicht gesagt hat, wo er läuft",
            ),
        ]);
        let said = EgressPolicy::NothingLeaves
            .refusal(&to_someone())
            .unwrap()
            .said(&strings);
        assert!(said.is_translated());
        assert!(said.text().contains("nicht gesagt hat"), "{said}");
        assert!(!said.text().contains("has not said"), "{said}");
        // The provider's name is the person's, not the language's.
        assert!(said.text().contains("someone"), "{said}");
    }

    /// **A refusal never depends on a string table.** With no words at all the
    /// policy refuses exactly what it refused before, and the answer names the
    /// rule by its key so whoever forgot to declare this crate's words finds
    /// out from the sentence rather than from a blank line.
    #[test]
    fn a_refusal_without_the_words_still_names_the_rule() {
        let nothing = Strings::of(alo_strings::Vocabulary::empty());
        let said = EgressPolicy::InTheBuilding
            .refusal(&to_someone())
            .unwrap()
            .said(&nothing);
        assert!(said.is_a_bug());
        assert!(
            said.text().contains("egress.policy.outside-the-building"),
            "{said}"
        );
    }

    /// Every refusal carries what it refused, so whatever records it can say
    /// what the agent tried rather than only that something was stopped.
    #[test]
    fn a_refusal_carries_what_it_refused() {
        for policy in [
            EgressPolicy::InTheBuilding,
            EgressPolicy::NothingLeaves,
            EgressPolicy::InRegion("Switzerland".to_owned()),
        ] {
            let refused = policy.refusal(&to_someone()).unwrap();
            assert_eq!(refused.leaving(), &to_someone(), "{policy:?}");
            assert_eq!(refused.leaving().agent(), &Grantee::named("@mail"));
        }
    }

    /// Which rule refused is answerable without asking for words, because a
    /// settings panel showing *what your organisation's rule stopped* is a
    /// different question from *what do I tell this person*.
    #[test]
    fn which_rule_refused_is_answerable_without_a_vocabulary() {
        assert_eq!(
            EgressPolicy::InRegion("the EU".to_owned())
                .refusal(&to_someone())
                .map(|refused| refused.why().clone()),
            Some(Refusal::OutsideTheRegion {
                region: "the EU".to_owned()
            })
        );
        assert_eq!(
            EgressPolicy::InTheBuilding
                .refusal(&to_someone())
                .map(|refused| refused.why().clone()),
            Some(Refusal::OutsideTheBuilding)
        );
    }
}
