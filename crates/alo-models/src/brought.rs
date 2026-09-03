//! Everything somebody brought to this machine, beside the catalogue rather
//! than inside it.
//!
//! Two lists, and the separation is the point. [`crate::Catalogue`] is what alo
//! OS offers — parsed out of a file that shipped with the release, with a
//! licence gate on it because offering something is what makes a licence ours
//! to state. [`Brought`] is what the person put there, and it is the list that
//! makes *the catalogue recommends; it does not gate* true rather than a
//! sentence in `docs/features.md`.
//!
//! # Ids are matched exactly
//!
//! `Providers::get` matches a name however it is capitalised, because a
//! provider's name is a word a person typed and typed again. An id here is what
//! the model runtime answers to, which is the case item 1 settled for every
//! other identity in alo OS: **matched exactly, because matching loosely
//! matches more than the person picked.** Two ids differing in case are two
//! models to the runtime, and a list that merged them would ask a question of
//! weights nobody chose.
//!
//! # What this list deliberately does not know
//!
//! **Whether an id is also in the catalogue.** It could be asked — the
//! catalogue is one call away — and the answer would put an opinion about what
//! alo OS offers inside the list of what somebody else brought, which is the
//! coupling this file exists without. Whoever holds both lists at once is the
//! machine deciding which model answers, and that is not decided anywhere yet.
//!
//! **How much memory the machine has.** [`Brought::for_the_agent`] filters on
//! the measurement and on nothing else. A list method that dropped weights too
//! large for this machine would be the refusal [`crate::Cost`] refuses to be,
//! arriving where nobody would think to look for it.

use serde::{Deserialize, Serialize};

use crate::weights::{Weights, WeightsError};

/// The weights somebody brought to this machine.
///
/// `Serialize` for the reason [`crate::Providers`] is: this is what a settings
/// file holds. Where it is written and when is the daemon's, and does not exist
/// yet.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Brought {
    /// In the order they were brought.
    #[serde(default)]
    pub weights: Vec<Weights>,
}

impl Brought {
    /// Take one more set of weights.
    ///
    /// # Errors
    /// [`WeightsError::AlreadyBrought`] when the id is on the list already —
    /// two entries answering to one name would make *this model answered it*
    /// ambiguous, which is `Providers::add`'s reasoning one list over.
    pub fn add(&mut self, weights: Weights) -> Result<(), WeightsError> {
        if self.weights.iter().any(|w| w.id == weights.id) {
            return Err(WeightsError::AlreadyBrought(weights.id));
        }
        self.weights.push(weights);
        Ok(())
    }

    /// Take one off the list, saying whether it was there.
    ///
    /// The weights themselves are untouched: this is a person saying alo OS
    /// need not hold an opinion about a model any more, not a person deleting
    /// gigabytes. Removing weights from the disk is
    /// [`crate::ModelRuntime::remove`], which is a different act with a
    /// different cost.
    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.weights.len();
        self.weights.retain(|w| w.id != id.trim());
        self.weights.len() != before
    }

    /// One by the id the runtime answers to.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Weights> {
        self.weights.iter().find(|w| w.id == id.trim())
    }

    /// The ones alo OS would give an agent turn.
    ///
    /// Filtered on the measurement alone — see this file's header for what is
    /// deliberately not asked here.
    #[must_use]
    pub fn for_the_agent(&self) -> Vec<&Weights> {
        self.weights
            .iter()
            .filter(|w| w.can_be_the_agent())
            .collect()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::costing::GIGABYTE;
    use crate::driving::Driving;
    use crate::testing::in_english;

    fn theirs(id: &str) -> Weights {
        Weights::checked(id, 4 * GIGABYTE).unwrap()
    }

    /// Two entries answering to one name would make *this model answered it*
    /// ambiguous, and naming what answered is the whole point.
    #[test]
    fn the_same_weights_cannot_be_brought_twice() {
        let mut brought = Brought::default();
        assert!(brought.add(theirs("their-own")).is_ok());
        let again = brought.add(theirs("their-own")).unwrap_err();
        assert_eq!(again, WeightsError::AlreadyBrought("their-own".to_owned()));
        assert!(
            again.said(&in_english()).text().contains("their-own"),
            "{again:?}"
        );
        assert_eq!(brought.weights.len(), 1);
    }

    /// **Two ids differing in case are two models**, because an id is what a
    /// runtime answers to rather than a word a person chose — item 1's rule,
    /// and the opposite answer from `Providers`, which holds names people type.
    #[test]
    fn an_id_is_matched_exactly_because_a_runtime_matches_it_exactly() {
        let mut brought = Brought::default();
        brought.add(theirs("Their-Own")).unwrap();
        assert!(brought.add(theirs("their-own")).is_ok());
        assert_eq!(brought.weights.len(), 2);

        assert!(brought.get("Their-Own").is_some());
        assert!(brought.get("THEIR-OWN").is_none());
        assert!(brought.remove(" their-own "));
        assert!(brought.get("their-own").is_none());
        assert!(brought.get("Their-Own").is_some());
        assert!(!brought.remove("their-own"));
    }

    /// **What can be the agent is not filtered by memory.** A model larger than
    /// any machine here stays on the list a turn is chosen from, because the
    /// refusal this product does not make must not reappear in a list method
    /// nobody would think to look in.
    #[test]
    fn what_can_be_the_agent_is_decided_by_the_measurement_and_nothing_else() {
        let mut brought = Brought::default();
        brought
            .add(
                Weights::checked("enormous", 400 * GIGABYTE)
                    .unwrap()
                    .measured(Driving::Reliably),
            )
            .unwrap();
        brought.add(theirs("small-and-unmeasured")).unwrap();
        brought
            .add(theirs("small-and-poor").measured(Driving::Rarely))
            .unwrap();

        let chosen = brought.for_the_agent();
        assert_eq!(chosen.len(), 1);
        assert_eq!(chosen.first().unwrap().id, "enormous");
        assert!(chosen.first().unwrap().costs_on(16.0).larger_than_memory());
    }

    /// A machine nobody brought anything to has an empty list rather than a
    /// missing one, and nothing on it for the agent.
    #[test]
    fn a_machine_nobody_brought_anything_to_offers_nothing_and_refuses_nothing() {
        let brought = Brought::default();
        assert!(brought.weights.is_empty());
        assert!(brought.for_the_agent().is_empty());
        assert!(brought.get("anything").is_none());
    }

    /// The list is what a settings file holds, so it survives being written and
    /// read back — including a machine that has brought nothing.
    #[test]
    fn the_list_is_written_and_read_back_as_it_was() {
        let mut brought = Brought::default();
        brought
            .add(theirs("their-own").measured(Driving::Sometimes))
            .unwrap();
        let written = serde_json::to_string(&brought).unwrap();
        let read: Brought = serde_json::from_str(&written).unwrap();
        assert_eq!(read.weights, brought.weights);

        let empty: Brought = serde_json::from_str("{}").unwrap();
        assert!(empty.weights.is_empty());
    }

    /// Taking weights off this list is not taking them off the disk, and the
    /// two are different acts with different costs.
    #[test]
    fn removing_from_the_list_leaves_the_weights_where_they_are() {
        let mut brought = Brought::default();
        let weights = theirs("their-own");
        brought.add(weights.clone()).unwrap();
        assert!(brought.remove("their-own"));
        // The value is untouched and can be brought again without asking a
        // runtime for anything.
        assert!(brought.add(weights).is_ok());
    }
}
