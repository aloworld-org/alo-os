//! What testing a provider found, and why a provider was not tested at all.
//!
//! Both shapes of one answer, in one file, because they are read by one person
//! in one dialogue — the settings panel they have just typed a key into.
//! [`Found`] is what the one request came back with; [`NotVetted`] is why no
//! request was made.
//!
//! # Three values, because the person has three things they can do
//!
//! `docs/features.md`, v0.5: *test a provider before saving it, so a mistyped
//! key is found now rather than in the middle of a question.* The outcome of
//! that test is **one of exactly three values**, and the three are the three
//! different things somebody does next:
//!
//! | | What happened | What the person does |
//! |---|---|---|
//! | [`Found::Answered`] | The provider answered, and the key was accepted | Saves it |
//! | [`Found::RefusedTheKey`] | The provider answered, and would not take the key | Fixes the key — or saves it anyway, if they say so |
//! | [`Found::CouldNotBeReached`] | No working provider answered at that address | Fixes the address, or waits — or saves it anyway, if they say so |
//!
//! **The reason travels inside the value**, as `alo_models::NotTried`, rather
//! than being flattened into three sentences of this crate's own. *Nothing
//! answered at that address — check the address* and *the provider answered
//! 503, which is a problem at their end* are both a provider that could not be
//! reached, and they send a person to two different places; a single sentence
//! for the bucket would send one of them to the wrong one. So the three values
//! are what a caller branches on, and the sentence is `alo-models`', which is
//! the crate that read the wire and already says each of those things in words
//! `alo-saying` collects.
//!
//! # And four reasons nothing was sent
//!
//! [`NotVetted`] is not a fourth outcome of the test. It is the test not
//! happening, and each of its reasons says so in the words of whoever decided:
//!
//! | | Whose words |
//! |---|---|
//! | [`NotVetted::CannotBeTestedFromHere`] | This crate's — the one sentence it adds |
//! | [`NotVetted::CannotBeShown`] | `alo-egress`': the provider's name could not go on the indicator |
//! | [`NotVetted::HeldBack`] | `alo-egress`': the rule this machine is under refused the egress |
//! | [`NotVetted::Forbidden`] | `alo-models`': the same rule, asked by the wire itself |
//!
//! The last two are one rule heard twice, and the second hearing is the one a
//! reporter does not get to assume away: `alo_egress::EgressPolicy` is made
//! from `alo_models::SourcePolicy` and the two agree about every source there
//! is (a test in `alo-egress` says so), so a request the indicator permitted is
//! one the wire's own check permits too. If it ever were not, the line is taken
//! off the indicator — nothing left — and the wire's refusal is carried here
//! whole rather than matched with a wildcard and lost.
//!
//! # No `Display` on anything here
//!
//! Every one of these is read by the person whose key it is, so the only road
//! to words is `said(&Strings)`, which is this workspace's rule since item 9f.
//! And no sentence reachable from this file has the key in it, because no type
//! here holds one: what was tried is `alo_models::Tried` and `alo_models::NotTried`,
//! and neither can be made from a key.

use alo_egress::{DestinationError, NotPermitted};
use alo_models::{NotAllowed, NotTried, Tried};
use alo_strings::{Filling, Said, Strings};

use crate::words;

/// What the one request to a provider came back with.
///
/// Exactly three, and each is a different thing for the person to do next —
/// this file's header has the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found {
    /// It answered, and the key was accepted. Save it.
    ///
    /// Carries what the provider said it offers, as `alo_models::Tried` holds
    /// it: the names checked before any of them can reach a screen, and the
    /// two caveats a panel draws beneath the answer.
    Answered(Tried),
    /// It answered, and refused the key it was given — or was given none and
    /// would not answer without one.
    ///
    /// **The whole reason this feature exists**: found while somebody is still
    /// looking at the field they typed the key into. The reason inside says
    /// which of the two it was, because telling somebody their key was
    /// rejected when they never typed one sends them to check a key that does
    /// not exist.
    RefusedTheKey(NotTried),
    /// No working provider answered at that address.
    ///
    /// Nothing listening, a redirect somewhere nobody agreed to, something
    /// that answered but not like a provider, or a provider saying it is having
    /// trouble. The reason inside says which, because they send a person to
    /// different places — and a provider that is down today is not a wrong
    /// provider, so the person may still save it if they say so.
    CouldNotBeReached(NotTried),
}

impl Found {
    /// What the wire came back with, sorted into the three things a person can
    /// do about it.
    ///
    /// `pub(crate)`: a [`Found`] is made by a provider actually answering, in
    /// [`crate::Vetting`], and by nothing else.
    ///
    /// # Errors
    /// The policy's own refusal, when the wire's check refused the request
    /// after the indicator had permitted it. Not an outcome of the test — the
    /// test did not happen — so it is handed back for [`crate::Vetting`] to
    /// take the line off the indicator and report as [`NotVetted::Forbidden`].
    pub(crate) fn of(tried: Result<Tried, NotTried>) -> Result<Self, NotAllowed> {
        match tried {
            Ok(tried) => Ok(Self::Answered(tried)),
            Err(why @ (NotTried::KeyNotAccepted | NotTried::NeedsAKey)) => {
                Ok(Self::RefusedTheKey(why))
            }
            Err(
                why @ (NotTried::Unreachable
                | NotTried::Redirected
                | NotTried::NotUnderstood
                | NotTried::NotWell(_)),
            ) => Ok(Self::CouldNotBeReached(why)),
            Err(NotTried::Forbidden(refusal)) => Err(refusal),
        }
    }

    /// Whether this is a provider that can be saved without anybody being
    /// asked: it answered, and the key was accepted.
    #[must_use]
    pub fn is_a_working_provider(&self) -> bool {
        matches!(self, Self::Answered(_))
    }

    /// The string said for this outcome.
    ///
    /// `alo-models`' in every case, because that crate read the wire and
    /// already says what it found.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::Answered(tried) => tried.word(),
            Self::RefusedTheKey(why) | Self::CouldNotBeReached(why) => why.word(),
        }
    }

    /// The line a person reads when the test comes back, in their own language.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// `alo_models::model_words` answers with the key, marked. **The key the
    /// person typed is not in it and cannot be**: nothing this is made from
    /// holds one.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::Answered(tried) => tried.said(strings),
            Self::RefusedTheKey(why) | Self::CouldNotBeReached(why) => why.said(strings),
        }
    }

    /// What else the person needs to know beneath the line, one line each.
    ///
    /// `alo_models::Tried::caveats` for a provider that answered — a list that
    /// was cut, names that could not be shown — and nothing for the other two,
    /// which have said everything there is to say.
    #[must_use]
    pub fn caveats(&self, strings: &Strings) -> Vec<Said> {
        match self {
            Self::Answered(tried) => tried.caveats(strings),
            Self::RefusedTheKey(_) | Self::CouldNotBeReached(_) => Vec::new(),
        }
    }
}

/// Why a provider was not tested: no request was made.
///
/// Not a fourth outcome — this file's header has the argument — and every one
/// of them is a reason the person may still save the provider as they do
/// today, because a test that did not happen has found nothing wrong with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotVetted {
    /// This crate does not know how to reach the provider, so it does not
    /// guess.
    ///
    /// What this crate knows is one convention and an address it can turn into
    /// somewhere to connect. A provider with neither — an address with no host
    /// in it, or a scheme alo OS does not open — is one nothing here will
    /// invent an endpoint for: the honest answer is that it cannot be tested
    /// from here, and the save proceeds as it does today.
    CannotBeTestedFromHere,
    /// The provider's name could not be put on the indicator, and law 1 does
    /// not permit an egress nobody can be shown.
    ///
    /// A provider is named by whoever added it, and `alo_models::Provider`
    /// asks only that the name is not empty — so a name carrying a line break
    /// reaches here, where it is refused rather than drawn onto the one surface
    /// a person is expected to trust.
    CannotBeShown(DestinationError),
    /// The rule this machine is under refused the egress, in the rule's own
    /// words, before anything was sent.
    HeldBack(NotPermitted),
    /// The same rule, heard by the wire's own check after the indicator had
    /// permitted the request. Nothing was sent and the line is off the
    /// indicator.
    ///
    /// The two rules agree about every source there is, so this is the arm a
    /// reporter is not allowed to assume away rather than one anybody expects
    /// to see.
    Forbidden(NotAllowed),
}

impl NotVetted {
    /// The string said for this reason.
    ///
    /// This crate's own for the first, and the deciding crate's for the other
    /// three — a rule's refusal is worded by whoever made it.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::CannotBeTestedFromHere => words::CANNOT_BE_TESTED_FROM_HERE,
            Self::CannotBeShown(why) => why.word(),
            Self::HeldBack(refused) => refused.why().word(),
            Self::Forbidden(refusal) => refusal.word(),
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` that was never given the
    /// deciding crate's words answers with the key, marked. **What was refused
    /// never depends on the string table** — the test had already not happened
    /// before this was called.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::CannotBeTestedFromHere => strings.say(&self.word().key(), &Filling::nothing()),
            Self::CannotBeShown(why) => why.said(strings),
            Self::HeldBack(refused) => refused.said(strings),
            Self::Forbidden(refusal) => refusal.said(strings),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use alo_models::{InferenceSource, Region};

    /// **Every way the wire can come back lands in one of the three values**,
    /// and the list is walked so a reason added to `alo_models::NotTried` later
    /// is a compiler error here rather than whatever the last arm said.
    #[test]
    fn every_answer_from_the_wire_is_one_of_three_things_to_do() {
        for (tried, expected) in [
            (
                Err(NotTried::KeyNotAccepted),
                Found::RefusedTheKey(NotTried::KeyNotAccepted),
            ),
            (
                Err(NotTried::NeedsAKey),
                Found::RefusedTheKey(NotTried::NeedsAKey),
            ),
            (
                Err(NotTried::Unreachable),
                Found::CouldNotBeReached(NotTried::Unreachable),
            ),
            (
                Err(NotTried::Redirected),
                Found::CouldNotBeReached(NotTried::Redirected),
            ),
            (
                Err(NotTried::NotUnderstood),
                Found::CouldNotBeReached(NotTried::NotUnderstood),
            ),
            (
                Err(NotTried::NotWell(503)),
                Found::CouldNotBeReached(NotTried::NotWell(503)),
            ),
        ] {
            assert_eq!(Found::of(tried), Ok(expected));
        }
        // Only the first is a provider to save without asking anybody.
        assert!(!Found::RefusedTheKey(NotTried::KeyNotAccepted).is_a_working_provider());
        assert!(!Found::CouldNotBeReached(NotTried::Unreachable).is_a_working_provider());
    }

    /// **The rule's refusal is not an outcome.** It is handed back for the
    /// door to take the line off the indicator, carrying the rule's own words.
    #[test]
    fn a_refusal_by_the_rule_is_not_one_of_the_three() {
        let refusal = NotAllowed::NotThisMachine {
            source: InferenceSource::Hosted {
                provider: "Mistral".to_owned(),
                region: Region::Unknown,
            },
        };
        assert_eq!(
            Found::of(Err(NotTried::Forbidden(refusal.clone()))),
            Err(refusal)
        );
    }

    /// **Each value says the precise thing, not the bucket.** A provider
    /// answering 503 and an address nothing listens on are both
    /// [`Found::CouldNotBeReached`], and a person reading them is sent to two
    /// different places.
    #[test]
    fn the_sentence_is_the_precise_reason_rather_than_the_bucket() {
        let strings = in_english();
        let down = Found::CouldNotBeReached(NotTried::NotWell(503)).said(&strings);
        let nothing = Found::CouldNotBeReached(NotTried::Unreachable).said(&strings);
        assert!(down.text().contains("503"), "{down}");
        assert!(down.text().contains("their end"), "{down}");
        assert!(nothing.text().contains("check the address"), "{nothing}");
        assert_ne!(down.text(), nothing.text());

        let refused = Found::RefusedTheKey(NotTried::KeyNotAccepted).said(&strings);
        let none_given = Found::RefusedTheKey(NotTried::NeedsAKey).said(&strings);
        assert!(refused.text().contains("the whole key"), "{refused}");
        assert!(none_given.text().contains("add the one it"), "{none_given}");
        for said in [&down, &nothing, &refused, &none_given] {
            assert!(!said.is_a_bug(), "{said}");
        }
    }

    /// The two outcomes that are not a working provider have no caveats to
    /// draw: their sentence has said everything there is to say. A provider
    /// that answered carries the list's own, which `vetting.rs` asserts against
    /// a server, because a `Tried` is made by a provider answering and by
    /// nothing else.
    #[test]
    fn a_provider_that_did_not_work_has_no_caveats_beneath_its_sentence() {
        let strings = in_english();
        assert!(
            Found::RefusedTheKey(NotTried::KeyNotAccepted)
                .caveats(&strings)
                .is_empty()
        );
        assert!(
            Found::CouldNotBeReached(NotTried::NotWell(503))
                .caveats(&strings)
                .is_empty()
        );
    }

    /// **A test that never happened says why in the words of whoever
    /// decided**, and the one sentence this crate adds says what to do.
    #[test]
    fn a_provider_that_was_not_tested_says_whose_decision_that_was() {
        let strings = in_english();
        let here = NotVetted::CannotBeTestedFromHere.said(&strings);
        assert!(!here.is_a_bug(), "{here}");
        assert!(here.text().contains("cannot be tested from here"), "{here}");
        assert!(here.text().contains("nothing was sent"), "{here}");
        assert!(here.text().contains("save it"), "{here}");

        let unshowable = NotVetted::CannotBeShown(DestinationError::NotPrintable).said(&strings);
        assert!(
            unshowable.text().contains("cannot be shown"),
            "{unshowable}"
        );

        let forbidden = NotVetted::Forbidden(NotAllowed::OutsideTheBuilding {
            source: InferenceSource::Hosted {
                provider: "Mistral".to_owned(),
                region: Region::Declared("the EU".to_owned()),
            },
        })
        .said(&strings);
        assert!(forbidden.text().contains("in the building"), "{forbidden}");
        assert!(forbidden.text().contains("Mistral"), "{forbidden}");
    }
}
