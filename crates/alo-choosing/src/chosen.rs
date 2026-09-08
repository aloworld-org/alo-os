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

/// Which of this machine's two lists of models a choice names.
///
/// A closed list of the lists that exist. A provider somebody added and a
/// machine somebody paired with are two more places a question could be
/// answered (ADR 0008), and neither is here: this machine keeps no list of
/// either, so a choice naming one could not be resolved into anything. What
/// that means for a settings file is in `docs/contracts/person-settings.md`:
/// such a file fails to read as [`crate::NotSet::NotUnderstood`], naming the
/// two lists there are, rather than reading as a setting that quietly does
/// nothing.
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
        }
    }

    /// The local choice, where that is what this is.
    #[must_use]
    pub const fn on_this_machine(&self) -> Option<&Chosen> {
        match self {
            Self::OnThisMachine(chosen) => Some(chosen),
            Self::FromAProvider { .. } => None,
        }
    }

    /// The provider's name, where that is what this is.
    #[must_use]
    pub fn provider(&self) -> Option<&str> {
        match self {
            Self::OnThisMachine(_) => None,
            Self::FromAProvider { provider, .. } => Some(provider),
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
