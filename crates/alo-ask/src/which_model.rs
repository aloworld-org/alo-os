//! Which model on this machine answers, asked of the two places that can say.
//!
//! A question needs a model named before it is a question at all —
//! `alo_asking::Question::asked` refuses one without — and on a machine with
//! weights on the disk there are exactly two honest answers to *which*: the one
//! the person chose in their own settings, and, when they have chosen nothing,
//! the only one there is.
//!
//! # There is deliberately no third rule
//!
//! Nothing here ranks. A machine with three sets of weights and no setting is a
//! machine whose person has not said which model answers them, and picking one
//! for them would be this command inventing a preference — the mistake
//! [ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md) records
//! having made once already, where *"default" was the wrong word: it implies a
//! ranking*. So the refusal is `alo_asking::NotAQuestion::NoModel`, whose
//! sentence is **choose a model for this question to be answered by**, and the
//! person chooses.
//!
//! `alo_models::Catalogue::agent_for_cpu` is the method that *does* rank, and it
//! is not used here on purpose. It answers a different question — which model
//! gets the agent, measured on driving the verbs — over the whole catalogue
//! rather than over what has been downloaded, so its answer is regularly a model
//! this machine has never fetched. A person who typed a question wants the model
//! that is here.
//!
//! # And nothing here checks that the chosen model is installed
//!
//! [`the_persons_model`] hands back what the settings say and does not look for
//! it in [`Installed`]. The runtime is the authority on what it holds, a check
//! here would be a second statement of the same fact, and the gap between the
//! check and the question is exactly where the two could disagree. Asking for
//! weights that are not there answers `alo_models::RuntimeError::NotInstalled`,
//! whose sentence names the model — which is a better thing for a person to read
//! than a refusal this file composed.

use alo_choosing::Picked;
use alo_models::Installed;

/// The model the person chose, **when what they chose is a model on this
/// machine**.
///
/// `None` for the other two things a person can choose, and that is not a
/// refusal: a provider's model and a paired machine's model are not models here,
/// so they are no answer to the question this asks, and the next rule gets to
/// answer it instead. Whether a question *should* be put to this machine when
/// the person chose elsewhere is not this file's business either — this command
/// is named for the place it asks, and `alo_asking::Asking::to_this_machine` is
/// the only door it has.
#[must_use]
pub fn the_persons_model(chosen: Option<&Picked>) -> Option<&str> {
    match chosen {
        Some(Picked::OnThisMachine(theirs)) => Some(theirs.model()),
        Some(Picked::FromAProvider { .. } | Picked::FromAPairedMachine(_)) | None => None,
    }
}

/// The installed model, when there is exactly one of it.
///
/// `None` for none and `None` for several, and the same answer for both is the
/// point: in each case this machine cannot say which model answers, and the one
/// sentence that follows tells the person to choose. They are not the same
/// situation to be in — one needs weights brought and the other needs a setting
/// made — and neither is a situation this file may resolve by picking.
#[must_use]
pub fn the_only_one(installed: &[Installed]) -> Option<&str> {
    match installed {
        [only] => Some(only.id.as_str()),
        [] | [_, _, ..] => None,
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on a value that could not be built is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_choosing::{Chosen, Which};

    /// An installed model, as a runtime reports one.
    fn installed(id: &str) -> Installed {
        Installed {
            id: id.to_owned(),
            bytes_on_disk: 4_000_000_000,
            quantisation: None,
        }
    }

    #[test]
    fn the_model_a_person_chose_on_this_machine_is_the_one() {
        let chosen = Picked::OnThisMachine(
            Chosen::of(Which::Catalogue, "a-model").expect("a model they picked"),
        );
        assert_eq!(the_persons_model(Some(&chosen)), Some("a-model"));
    }

    /// **A provider's model is not a model here.** It is the name of something
    /// somebody else's machine runs, and handing it to the runtime on this one
    /// would ask for weights nobody downloaded — so this answers nothing and
    /// lets what is installed answer instead.
    #[test]
    fn a_model_chosen_from_a_provider_is_not_a_model_on_this_machine() {
        let chosen = Picked::from_a_provider("Mistral", "mistral-small-latest")
            .expect("a provider they use");
        assert_eq!(the_persons_model(Some(&chosen)), None);
    }

    #[test]
    fn a_person_who_has_chosen_nothing_has_chosen_no_model() {
        assert_eq!(the_persons_model(None), None);
    }

    #[test]
    fn the_only_installed_model_is_the_one() {
        assert_eq!(the_only_one(&[installed("a-model")]), Some("a-model"));
    }

    /// **Nothing installed is not a model**, and the refusal that follows is the
    /// one that tells the person to choose. It is the case this file would most
    /// easily have got wrong by answering the first element of an empty list.
    #[test]
    fn a_machine_with_no_weights_names_no_model() {
        assert_eq!(the_only_one(&[]), None);
    }

    /// **Several is not a model either.** The test that matters: a list in some
    /// order is not a preference, and reading the first of it would have made
    /// one out of whatever order the runtime happened to answer in.
    #[test]
    fn several_installed_models_name_none_rather_than_the_first() {
        let three = [installed("a-model"), installed("b-model"), installed("c")];
        assert_eq!(the_only_one(&three), None);
    }
}
