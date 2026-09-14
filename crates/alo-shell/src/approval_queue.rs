//! The changes that arrived while another was in front of the person, in the
//! order they arrived.
//!
//! `alo_approving::Approving::ask` replaces whatever question was on the
//! screen, and says why: a person looking at two changes approves the second
//! without reading it. The other half of that rule is this list. A proposal
//! that arrives while a question is open is **kept here** rather than put up
//! over it, so nobody finds the sentence they were reading swapped for one
//! they were not — and nobody answers a question they did not read.
//!
//! The list holds numbers and never sentences: a question is worded at the
//! moment it is put in front of somebody, from a change that is still really
//! waiting, and not at the moment it arrived.

use std::collections::VecDeque;

use alo_capability::ProposalId;

/// The changes waiting their turn to be shown, first arrived first.
#[derive(Debug, Default)]
pub(crate) struct ApprovalQueue {
    /// Numbers of the turn's proposals, in the order they arrived.
    waiting: VecDeque<ProposalId>,
}

impl ApprovalQueue {
    /// Keep a change that arrived, behind every change that arrived before it.
    ///
    /// Answers `false`, and keeps nothing, for a number already kept: one
    /// proposal is one question, however many times it is announced.
    pub(crate) fn arrived(&mut self, id: ProposalId) -> bool {
        if self.waiting.contains(&id) {
            return false;
        }
        self.waiting.push_back(id);
        true
    }

    /// The change that arrived first, taken off the list.
    pub(crate) fn next(&mut self) -> Option<ProposalId> {
        self.waiting.pop_front()
    }

    /// Whether `id` is kept here.
    pub(crate) fn holds(&self, id: ProposalId) -> bool {
        self.waiting.contains(&id)
    }

    /// How many changes are waiting their turn.
    pub(crate) fn len(&self) -> usize {
        self.waiting.len()
    }

    /// Forget every change kept here, answering none of them.
    ///
    /// For a turn that has stopped keeping evidence: none of these could be
    /// carried out, and each is still the turn's to account for.
    pub(crate) fn forgotten(&mut self) {
        self.waiting.clear();
    }
}
