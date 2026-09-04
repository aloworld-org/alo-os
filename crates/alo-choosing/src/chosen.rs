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

use alo_models::InferenceSource;

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
