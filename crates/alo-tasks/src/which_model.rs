//! Which model answers, and when a change to it takes effect.
//!
//! The owner's words, 2026-10-12: *model changes should apply to the next
//! request or a deliberate restart, never silently midway through an action.
//! Switching to an external provider must make any transfer of conversation or
//! files explicit.*
//!
//! # Why a change waits rather than applies
//!
//! A task is a conversation and an action in progress. A model swapped under a
//! running action gives a person an answer whose first half came from one place
//! and whose second half came from another — and if the second place is a
//! provider, **half their conversation has left the machine because they
//! changed a setting.** That is a silent fallback by another route, and
//! `docs/features.md` forbids those by name.
//!
//! So a change is **waiting** until the next request. The current action
//! finishes with the model it started with, and a person can see both: what is
//! answering now, and what will answer next.
//!
//! # The provider case is not a special case of the same rule
//!
//! Waiting is not enough on its own. Changing to a provider means the next
//! request, its conversation and whatever files it carries leave the machine —
//! so the change is **offered rather than applied**, and a person says yes to
//! the leaving rather than to the model.

use alo_models::InferenceSource;

/// Which model is answering, and which a person has asked for next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhichModel {
    /// What is answering now. Every request in flight uses this.
    answering: InferenceSource,
    /// What a person chose for next time, if they chose something else.
    ///
    /// **A second field rather than a replacement**, which is the whole of the
    /// rule: with one field there is no moment at which a change can be made to
    /// wait, and an action in flight reads whatever is there.
    next: Option<InferenceSource>,
}

/// What asking for a different model requires before it can even wait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeforeItChanges {
    /// Nothing. The new place is on this machine, or is a service at this
    /// machine's own address, so asking it sends nothing anywhere.
    NothingLeaves,
    /// The question, the conversation and any files would leave this machine.
    ///
    /// **Carried as its own answer rather than a flag**, so a caller cannot
    /// apply the change without having been handed the sentence that says what
    /// it costs. `alo_models::InferenceSource::causes_egress` is the
    /// measurement; this is the consequence.
    ItWouldLeaveThisMachine {
        /// Where it would go, for the sentence a person reads.
        to: InferenceSource,
    },
}

impl WhichModel {
    /// A task answering from here, with nothing chosen for next time.
    #[must_use]
    pub const fn answering_from(answering: InferenceSource) -> Self {
        Self {
            answering,
            next: None,
        }
    }

    /// What is answering now.
    #[must_use]
    pub const fn answering(&self) -> &InferenceSource {
        &self.answering
    }

    /// Whether a provider answers now — a hosted service, whatever its region —
    /// which is what the task window draws a cloud beside (2026-10-31).
    #[must_use]
    pub const fn a_provider_answers(&self) -> bool {
        matches!(self.answering, InferenceSource::Hosted { .. })
    }

    /// What a person asked to use next, if they asked for anything.
    #[must_use]
    pub const fn next(&self) -> Option<&InferenceSource> {
        self.next.as_ref()
    }

    /// Whether a change is waiting for the next request.
    #[must_use]
    pub const fn a_change_is_waiting(&self) -> bool {
        self.next.is_some()
    }

    /// What this change would cost, asked **before** it is made.
    ///
    /// A caller asks this, shows what it says, and only then calls
    /// [`Self::will_answer_next`]. Two steps rather than one, because a single
    /// call that both decided and applied would be a change a person could make
    /// without being told what it does.
    #[must_use]
    pub fn before_changing_to(to: &InferenceSource) -> BeforeItChanges {
        if to.causes_egress() {
            BeforeItChanges::ItWouldLeaveThisMachine { to: to.clone() }
        } else {
            BeforeItChanges::NothingLeaves
        }
    }

    /// Use this one for the next request.
    ///
    /// **It does not touch what is answering now**, which is the rule. Choosing
    /// the model that is already answering clears the waiting change rather
    /// than storing one, so a person who changes their mind back is not left
    /// with a change pending to where they already are.
    pub fn will_answer_next(&mut self, next: InferenceSource) {
        self.next = (next != self.answering).then_some(next);
    }

    /// Begin the next request, taking any waiting change with it.
    ///
    /// **The one place the answering model moves**, which is what makes
    /// *never midway* checkable: every other method leaves it alone, and this
    /// one is called between requests by construction.
    pub fn the_next_request_begins(&mut self) {
        if let Some(next) = self.next.take() {
            self.answering = next;
        }
    }
}

#[cfg(test)]
#[path = "which_model_tests.rs"]
mod tests;
