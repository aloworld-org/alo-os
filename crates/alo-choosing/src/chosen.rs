//! What the person picked to answer their questions, and which list they picked
//! it from.
//!
//! # A choice names its list, and that is not tidiness
//!
//! There are two lists of models on a machine — the catalogue alo OS ships and
//! `alo_models::Brought`, the weights somebody put there themselves — and
//! neither knows about the other on purpose.
//! [ADR 0016](../../../docs/decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
//! settles what that costs a settings file: a model called `mistral-small` in
//! the catalogue and a file somebody brought under the same name are two
//! different answers to *what runs my turn*, and a setting that could not tell
//! them apart would pick one by accident. So [`Which`] travels with the name,
//! and the ambiguity is resolved where it was created rather than by making the
//! two lists know about each other.
//!
//! # Both of them are this machine, and that is the whole of what is decided
//!
//! [`Chosen::source`] answers `alo_models::InferenceSource::ThisMachine` for
//! both, because that is what both are: weights on this disk, read by a runtime
//! on this machine. Everything downstream follows from it — nothing on the
//! indicator, nothing in the record that left, and a working day that produces
//! zero inference egress, which is law 1's measured claim rather than its
//! promise.
//!
//! The list is not thrown away for it. Which list a model came from is what
//! decides whose terms it is under and what it cost the disk, and both of those
//! are read by a panel that has this value in front of it.

use alo_models::{InferenceSource, Providers};
use alo_strings::{Filling, Said, Strings};

use crate::settings::MayUseAnother;

use crate::words;

use crate::paired::{AMachine, WHAT_THAT_MACHINE_CHOSE};

/// Which of this machine's two lists of models a choice names.
///
/// A closed list of the lists of models on this machine. A provider somebody
/// added and a machine somebody paired with are two more places a question
/// could be answered (ADR 0008), and neither is a list of models here: they
/// are [`Picked::FromAProvider`] and [`Picked::FromAPairedMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    /// The catalogue alo OS ships, where every model states its licence.
    Catalogue,
    /// Weights somebody brought themselves, which alo OS never offered and
    /// which are theirs.
    Brought,
}

/// A model named nothing.
///
/// Not a refusal with words of its own, for item 9b's reason: whoever needs a
/// sentence about it is reading a file, and [`crate::NotSet::Nameless`] is the
/// one that can name the file it is in. This says only that a name of nothing
/// is not a choice — a settings panel that wrote an empty string wrote no
/// choice at all, and a machine that took it would ask a runtime for a model
/// called nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoModel;

/// What this person chose to answer their questions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chosen {
    /// Which list it came from.
    which: Which,
    /// What that list calls it, exactly as the list does.
    model: String,
}

impl Chosen {
    /// This model, from this list.
    ///
    /// The name is kept exactly as it was written, which is item 1's rule about
    /// identities: a runtime matches the name it was given, and trimming or
    /// lower-casing here would be this crate quietly asking for a different
    /// model than the one somebody picked.
    ///
    /// # Errors
    /// [`NoModel`] when the name is empty or nothing but spaces — the shape the
    /// mistake really arrives in, which is a value cleared rather than a line
    /// removed.
    pub fn of(which: Which, model: &str) -> Result<Self, NoModel> {
        if model.trim().is_empty() {
            return Err(NoModel);
        }
        Ok(Self {
            which,
            model: model.to_owned(),
        })
    }

    /// Which of the two lists this came from.
    #[must_use]
    pub const fn which(&self) -> Which {
        self.which
    }

    /// What the list calls it, which is what a runtime is asked for.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Where a question put to this choice is answered.
    ///
    /// `ThisMachine` for both lists, and there is no other answer this type can
    /// give: whichever list the weights are on, they are on this disk.
    #[must_use]
    pub const fn source(&self) -> InferenceSource {
        InferenceSource::ThisMachine
    }
}

/// A provider named nothing.
///
/// The same shape of mistake as [`NoModel`] and a different word for it,
/// because the two are fixed in different places: a nameless model is a model
/// picker that wrote an empty string, and a nameless provider is a provider
/// list that did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoProvider;

/// **What this person chose to answer their questions**, which is one of the
/// three the owner named on 2026-09-08.
///
/// `docs/features.md`: *local models*, *your own API provider*, and *alo*. This
/// type is the first two, and the third is the second — [ADR 0014](../../../docs/decisions/0014-alos-own-model-is-a-provider-like-any-other.md)
/// makes alo's own service **exactly one more provider**, with no default, no
/// pre-selection and no special case anywhere in the code. So there is no
/// variant for it here, and its absence is the decision being kept rather than
/// an omission.
///
/// **These are model-source choices and not privacy levels.** Where a question
/// is answered follows from the choice; it is not the choice. What a person is
/// promised about confinement is a separate matter and
/// [ADR 0021](../../../docs/decisions/0021-what-a-service-on-this-machine-vouches-for.md)
/// is proposed and unaccepted.
///
/// # Why a provider is a name and a model, and a local choice is a list and a
/// model
///
/// Both name **two** things, and they are two different pairs. On this machine
/// the pair is *which list* and *which entry*, because a machine has two lists
/// and a name can be on both. For a provider the pair is *which provider* and
/// *which model*, because one provider offers many and the person picked one of
/// them. A shape that carried one name for both would have to guess which
/// question it was answering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Picked {
    /// A model on this machine, from one of its two lists.
    OnThisMachine(Chosen),

    /// A provider the person added, and the model they picked from it.
    ///
    /// The provider is named, not described: what it is and where it runs live
    /// in `alo_models::Providers`, which is the person's own list, so that a
    /// question about a provider is answered by the list rather than by
    /// whatever a settings file happened to repeat.
    FromAProvider {
        /// What the person called it in their own list.
        provider: String,

        /// What they asked that provider for, exactly as they wrote it.
        model: String,
    },

    /// A machine this person is paired with, whose own model answers.
    ///
    /// Carries no model, because which model answers there is that machine's
    /// person's setting (ADR 0008) — `crate::paired` has the argument, and
    /// [`crate::AMachine::permitted`] is the only door a surface has to one.
    FromAPairedMachine(AMachine),
}

impl Picked {
    /// A provider by name, and the model to ask it for.
    ///
    /// # Errors
    /// [`NoProvider`] when either is empty or nothing but spaces. A provider
    /// with no name cannot be looked up, and a provider asked for a model
    /// called nothing is a question nobody can answer.
    pub fn from_a_provider(provider: &str, model: &str) -> Result<Self, NoProvider> {
        let (provider, model) = (provider.trim(), model.trim());
        if provider.is_empty() || model.is_empty() {
            return Err(NoProvider);
        }
        Ok(Self::FromAProvider {
            provider: provider.to_owned(),
            model: model.to_owned(),
        })
    }

    /// What the thing answering is asked for, whichever it is.
    #[must_use]
    pub fn model(&self) -> &str {
        match self {
            Self::OnThisMachine(chosen) => chosen.model(),
            Self::FromAProvider { model, .. } => model,
            Self::FromAPairedMachine(_) => WHAT_THAT_MACHINE_CHOSE,
        }
    }

    /// The local choice, where that is what this is.
    #[must_use]
    pub const fn on_this_machine(&self) -> Option<&Chosen> {
        match self {
            Self::OnThisMachine(chosen) => Some(chosen),
            Self::FromAProvider { .. } | Self::FromAPairedMachine(_) => None,
        }
    }

    /// The provider's name, where that is what this is.
    #[must_use]
    pub fn provider(&self) -> Option<&str> {
        match self {
            Self::OnThisMachine(_) | Self::FromAPairedMachine(_) => None,
            Self::FromAProvider { provider, .. } => Some(provider),
        }
    }

    /// The paired machine, where that is what this is.
    #[must_use]
    pub const fn paired_machine(&self) -> Option<&AMachine> {
        match self {
            Self::FromAPairedMachine(machine) => Some(machine),
            Self::OnThisMachine(_) | Self::FromAProvider { .. } => None,
        }
    }

    /// **Where a question put to this choice is answered** — asked of the
    /// person's own list, never of the choice alone.
    ///
    /// This is the one method in this crate that must not be convenient.
    /// [`Chosen::source`] can be a `const fn` answering `ThisMachine` because
    /// for a model on one of this machine's lists there is no other answer. A
    /// provider's answer is a fact about the provider — its region, whether it
    /// is this machine at all — and it lives in [`Providers`]. A version of
    /// this that guessed would be a local choice and a remote one becoming the
    /// same value, which is precisely the silent switch between local and
    /// remote processing that nothing here may do.
    ///
    /// [`None`] when the choice names a provider that is not in the list. That
    /// cannot happen inside [`crate::Settings`], which refuses such a file, and
    /// it is not made impossible by the type because a caller may hold a choice
    /// and a list that were never checked against each other.
    #[must_use]
    pub fn source(&self, providers: &Providers) -> Option<InferenceSource> {
        match self {
            Self::OnThisMachine(chosen) => Some(chosen.source()),
            Self::FromAProvider { provider, .. } => {
                providers.get(provider).map(alo_models::Provider::source)
            }
            Self::FromAPairedMachine(machine) => Some(machine.source()),
        }
    }

    /// That this answer did not come from where the person chose, when it did
    /// not.
    ///
    /// [`None`] when they chose a model on this machine, because then nothing
    /// was substituted and a caveat would be a caveat on everything.
    ///
    /// # Why this lives here rather than at the caller
    ///
    /// [ADR 0008](../../../docs/decisions/0008-where-inference-happens.md) is
    /// explicit that a hosted API is *never a silent fallback*, and on
    /// 2026-10-18 `alo-ask` was measured doing exactly that: a settings file
    /// naming `mistral-small-latest` from Mistral, a 1.7B local model
    /// answering, and *on this machine* as the only hint — which reads as
    /// confirmation rather than as a substitution.
    ///
    /// **The sentence is here because the fact is here.** A caller that
    /// matched on the variants itself would be a second opinion about what a
    /// person chose, and the next caller would have to get the same match
    /// right again. This is one method, and a caller either shows what it
    /// hands back or does not.
    ///
    /// # What it does not do
    ///
    /// It does not say **why** the chosen place was not used, and it does not
    /// refuse anything. Why is a property of the machine at that moment rather
    /// than of anything the person did, and refusing would stop a person
    /// getting answers they get today — which law 5 forbids. The owner decided
    /// on 2026-10-18 that whether alo may use another model or API when the
    /// chosen one is unavailable is **a setting the person sets**; that setting
    /// does not exist yet, and this sentence is what is true either way.
    ///
    /// That nothing else may answer, when the person said so.
    ///
    /// `docs/features.md`, starred: *Unless the person says otherwise, in
    /// Settings* — the owner's instruction of 2026-10-18. This is the *off*
    /// half of it, and [`None`] is every other case:
    ///
    /// | they chose | they said | this answers |
    /// |---|---|---|
    /// | a model here | anything | [`None`] — there is nothing to fall back from |
    /// | elsewhere | nothing | [`None`] — **they have not refused**, so they are answered where alo can and told their choice was not used |
    /// | elsewhere | [`MayUseAnother::Yes`] | [`None`] — they allowed it |
    /// | elsewhere | [`MayUseAnother::No`] | the sentence, and the caller answers nothing |
    ///
    /// **The second row is the one that matters.** Reading *nobody has
    /// decided* as *they refused* would stop a person getting answers they get
    /// today, which law 5 calls taking a choice away — and
    /// [`crate::Settings::may_use_another`] keeps the two apart for exactly
    /// this call.
    ///
    /// Never fails and never panics: a `Strings` that was never given this
    /// crate's words through [`crate::declare_into`] answers with the key,
    /// marked.
    #[must_use]
    pub fn nothing_else_may_answer(
        &self,
        said: Option<MayUseAnother>,
        strings: &Strings,
    ) -> Option<Said> {
        if said != Some(MayUseAnother::No) {
            return None;
        }
        match self {
            Self::OnThisMachine(_) => None,
            Self::FromAProvider { provider, model } => Some(strings.say(
                &words::NOTHING_ELSE_MAY_ANSWER.key(),
                &Filling::of("provider", provider.clone()).and("model", model.clone()),
            )),
            Self::FromAPairedMachine(machine) => Some(strings.say(
                &words::NOTHING_ELSE_MAY_ANSWER_MACHINE.key(),
                &Filling::of("machine", machine.machine().to_owned()),
            )),
        }
    }

    /// # It compares, rather than assuming a substitution happened
    ///
    /// `came_from` is where the answer really came from, and this answers
    /// [`None`] when that is the place the person chose. **The first version
    /// of this method did not take it**, and would have told a person whose
    /// provider answered them perfectly well that their choice was not used —
    /// which is the same class of false statement as the silence it was
    /// written to fix, pointing the other way. It was caught by writing the
    /// test for the pairing that should not happen.
    ///
    /// Today `alo-ask` only ever builds [`InferenceSource::ThisMachine`], so
    /// the comparison changes nothing yet. That is exactly why it is here now:
    /// the method is correct for the day the provider road exists instead of
    /// becoming wrong on it.
    ///
    /// Never fails and never panics: a `Strings` that was never given this
    /// crate's words through [`crate::declare_into`] answers with the key,
    /// marked.
    #[must_use]
    pub fn not_the_place_you_chose(
        &self,
        came_from: &InferenceSource,
        strings: &Strings,
    ) -> Option<Said> {
        match self {
            // Nothing was substituted, and a caveat on every answer is a
            // caveat on none of them.
            Self::OnThisMachine(_) => None,
            Self::FromAProvider { provider, model } => {
                // **Matched on the provider's name**, which is what a person
                // put in both places: their choice names it and the source an
                // answer carries names it. A `Hosted` answer from some other
                // provider is still not what they chose.
                let answered_by_it = matches!(
                    came_from,
                    InferenceSource::Hosted { provider: from, .. } if from == provider
                );
                (!answered_by_it).then(|| {
                    strings.say(
                        &words::NOT_THE_PROVIDER_YOU_CHOSE.key(),
                        &Filling::of("provider", provider.clone()).and("model", model.clone()),
                    )
                })
            }
            Self::FromAPairedMachine(machine) => {
                let answered_by_it = matches!(
                    came_from,
                    InferenceSource::PairedMachine { machine: from } if from == machine.machine()
                );
                (!answered_by_it).then(|| {
                    strings.say(
                        &words::NOT_THE_MACHINE_YOU_CHOSE.key(),
                        &Filling::of("machine", machine.machine().to_owned()),
                    )
                })
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **The sentence a person gets when a provider they chose was not used.**
    ///
    /// This is the test for the fault measured on 2026-10-18: a settings file
    /// naming `mistral-small-latest` from Mistral, a 1.7B local model
    /// answering, and nothing saying so. ADR 0008: *never a silent fallback.*
    #[test]
    fn a_provider_that_was_not_used_is_said_so_and_names_what_was_chosen() {
        let chosen = Picked::FromAProvider {
            provider: "Mistral".to_owned(),
            model: "mistral-small-latest".to_owned(),
        };

        let said = chosen
            .not_the_place_you_chose(&InferenceSource::ThisMachine, &crate::testing::in_english())
            .expect("a provider was chosen and this answer did not come from it");

        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert_eq!(
            said.text(),
            "you chose mistral-small-latest from Mistral, and this answer did not come from there"
        );
    }

    /// **A paired machine that was not used is said so, and names no model.**
    ///
    /// Which model answers on another machine is that machine's person's
    /// setting (ADR 0008), so this sentence cannot name one.
    #[test]
    fn a_paired_machine_that_was_not_used_names_the_machine_and_no_model() {
        let machine = AMachine::permitted(
            "the studio machine",
            &crate::testing::PairedFor::a_day("the studio machine"),
            crate::testing::a_moment(),
        )
        .expect("a pairing that permits asking its models");
        let chosen = Picked::FromAPairedMachine(machine);

        let said = chosen
            .not_the_place_you_chose(&InferenceSource::ThisMachine, &crate::testing::in_english())
            .expect("a machine was chosen and this answer did not come from it");

        assert!(!said.is_a_bug(), "{said}");
        assert_eq!(
            said.text(),
            "you chose the model on the studio machine, and this answer did not come from there"
        );
    }

    /// **A model on this machine gets no caveat at all.**
    ///
    /// Nothing was substituted, and a caveat on every answer is a caveat on
    /// none of them.
    #[test]
    fn a_model_on_this_machine_is_not_a_substitution_and_says_nothing() {
        let chosen =
            Picked::OnThisMachine(Chosen::of(Which::Catalogue, "smollm2-1.7b-instruct").unwrap());
        assert!(
            chosen
                .not_the_place_you_chose(
                    &InferenceSource::ThisMachine,
                    &crate::testing::in_english()
                )
                .is_none()
        );
    }

    /// A region, for the hosted sources these tests build.
    ///
    /// Which region it is does not matter to any of them: what is compared is
    /// the provider's name, and the region is here because the variant has it.
    fn a_region() -> alo_models::Region {
        alo_models::Region::Unknown
    }

    /// **A person who said no is not answered, and is told which setting did
    /// it.**
    #[test]
    fn a_person_who_said_no_is_told_nothing_else_may_answer() {
        let chosen = Picked::FromAProvider {
            provider: "Mistral".to_owned(),
            model: "mistral-small-latest".to_owned(),
        };
        let said = chosen
            .nothing_else_may_answer(Some(MayUseAnother::No), &crate::testing::in_english())
            .expect("they turned it off, so nothing else may answer");

        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert_eq!(
            said.text(),
            "you chose mistral-small-latest from Mistral, and you have told alo not to use \
             anything else — so this was not answered"
        );
    }

    /// **A person who has not been asked is not treated as having refused.**
    ///
    /// The one that matters. Reading *nobody has decided* as *they refused*
    /// would stop somebody getting answers they get today, which law 5 calls
    /// taking a choice away.
    #[test]
    fn a_person_who_has_not_been_asked_has_not_refused() {
        let chosen = Picked::FromAProvider {
            provider: "Mistral".to_owned(),
            model: "mistral-small-latest".to_owned(),
        };
        assert!(
            chosen
                .nothing_else_may_answer(None, &crate::testing::in_english())
                .is_none(),
            "a person who has never seen the setting was refused an answer"
        );
    }

    /// **A person who allowed it is not refused either.**
    #[test]
    fn a_person_who_allowed_it_is_not_refused() {
        let chosen = Picked::FromAProvider {
            provider: "Mistral".to_owned(),
            model: "mistral-small-latest".to_owned(),
        };
        assert!(
            chosen
                .nothing_else_may_answer(Some(MayUseAnother::Yes), &crate::testing::in_english())
                .is_none()
        );
    }

    /// **And somebody who chose a model here has nothing to fall back from.**
    #[test]
    fn a_model_on_this_machine_never_refuses_for_this_reason() {
        let chosen =
            Picked::OnThisMachine(Chosen::of(Which::Catalogue, "smollm2-1.7b-instruct").unwrap());
        for said in [None, Some(MayUseAnother::Yes), Some(MayUseAnother::No)] {
            assert!(
                chosen
                    .nothing_else_may_answer(said, &crate::testing::in_english())
                    .is_none(),
                "{said:?}"
            );
        }
    }

    /// **A paired machine gets its own sentence, naming no model.**
    #[test]
    fn a_paired_machine_that_may_not_be_replaced_names_the_machine() {
        let machine = AMachine::permitted(
            "the studio machine",
            &crate::testing::PairedFor::a_day("the studio machine"),
            crate::testing::a_moment(),
        )
        .expect("a pairing that permits asking its models");
        let said = Picked::FromAPairedMachine(machine)
            .nothing_else_may_answer(Some(MayUseAnother::No), &crate::testing::in_english())
            .expect("they turned it off");

        assert!(!said.is_a_bug(), "{said}");
        assert!(
            said.text().contains("the studio machine"),
            "{}",
            said.text()
        );
    }

    /// **A provider that really answered is not reported as unused.**
    ///
    /// The pairing that would be wrong the other way, and the test that caught
    /// the first version of [`Picked::not_the_place_you_chose`] not taking a
    /// source at all. It is unreachable from `alo-ask` today, which only ever
    /// builds `ThisMachine` — and that is why it is asserted here rather than
    /// left for the day the provider road exists.
    #[test]
    fn a_provider_that_really_answered_is_not_reported_as_a_substitution() {
        let chosen = Picked::FromAProvider {
            provider: "Mistral".to_owned(),
            model: "mistral-small-latest".to_owned(),
        };
        let from_mistral = InferenceSource::Hosted {
            provider: "Mistral".to_owned(),
            region: a_region(),
        };
        assert!(
            chosen
                .not_the_place_you_chose(&from_mistral, &crate::testing::in_english())
                .is_none(),
            "a person whose chosen provider answered them was told it did not"
        );
    }

    /// **Another provider answering is still not what they chose.**
    ///
    /// Matched on the provider's name rather than on the variant, so a hosted
    /// answer from somewhere else does not satisfy a choice of Mistral.
    #[test]
    fn a_different_provider_answering_is_still_a_substitution() {
        let chosen = Picked::FromAProvider {
            provider: "Mistral".to_owned(),
            model: "mistral-small-latest".to_owned(),
        };
        let from_somebody_else = InferenceSource::Hosted {
            provider: "a different provider".to_owned(),
            region: a_region(),
        };
        assert!(
            chosen
                .not_the_place_you_chose(&from_somebody_else, &crate::testing::in_english())
                .is_some()
        );
    }

    /// **And the paired machine's comparison is by name too.**
    #[test]
    fn the_paired_machine_that_really_answered_is_not_reported_as_a_substitution() {
        let machine = AMachine::permitted(
            "the studio machine",
            &crate::testing::PairedFor::a_day("the studio machine"),
            crate::testing::a_moment(),
        )
        .expect("a pairing that permits asking its models");
        let chosen = Picked::FromAPairedMachine(machine);

        let from_the_studio = InferenceSource::PairedMachine {
            machine: "the studio machine".to_owned(),
        };
        assert!(
            chosen
                .not_the_place_you_chose(&from_the_studio, &crate::testing::in_english())
                .is_none()
        );

        let from_another = InferenceSource::PairedMachine {
            machine: "somebody else's machine".to_owned(),
        };
        assert!(
            chosen
                .not_the_place_you_chose(&from_another, &crate::testing::in_english())
                .is_some()
        );
    }

    /// The name is what a runtime will be asked for, so it comes back exactly
    /// as it went in.
    #[test]
    fn the_model_is_kept_exactly_as_it_was_written() {
        let chosen = Chosen::of(Which::Catalogue, "Mistral-Small:7b").unwrap();
        assert_eq!(chosen.model(), "Mistral-Small:7b");
        assert_eq!(chosen.which(), Which::Catalogue);
    }

    /// **A model named nothing is not a choice**, and neither is one named with
    /// spaces — which is what a settings panel writes when somebody clears a
    /// field rather than removing the setting.
    #[test]
    fn a_model_named_nothing_is_refused() {
        assert_eq!(Chosen::of(Which::Catalogue, ""), Err(NoModel));
        assert_eq!(Chosen::of(Which::Brought, "   "), Err(NoModel));
    }

    /// **The same name on the two lists is two choices**, which is the whole
    /// reason the list travels with the name.
    #[test]
    fn the_same_name_from_two_lists_is_not_the_same_choice() {
        let catalogued = Chosen::of(Which::Catalogue, "mistral-small").unwrap();
        let brought = Chosen::of(Which::Brought, "mistral-small").unwrap();
        assert_ne!(catalogued, brought);
        assert_eq!(catalogued.model(), brought.model());
    }

    /// **Both lists are this machine.** Law 1's zero-egress claim rests on it,
    /// so it is a test rather than a sentence: a choice from either list causes
    /// nothing to leave.
    #[test]
    fn a_model_from_either_list_is_answered_on_this_machine_and_leaves_nothing() {
        for which in [Which::Catalogue, Which::Brought] {
            let chosen = Chosen::of(which, "a-model").unwrap();
            assert_eq!(chosen.source(), InferenceSource::ThisMachine);
            assert!(!chosen.source().causes_egress(), "{which:?}");
        }
    }
}
