//! **Why sending an adapted model is sending somebody's documents.**
//!
//! A fine-tune over a person's correspondence produces weights from which those
//! sentences can be drawn back out. A machine that called that *uploading a
//! model* would be describing the file and not the act: what leaves is the
//! contents of a folder somebody granted for training and for nothing else.
//!
//! So, in this repository:
//!
//! 1. **This crate cannot send anything.** It has no client, no socket and no
//!    door — `tests/nothing_here_can_send_anything.rs` reads its own shipped
//!    source and fails on any road to one.
//! 2. **If a later task sends one, it is an `alo_egress` departure** with the
//!    indicator lit and a record written, exactly as fetching a model is — and
//!    the sentence a person is shown says *your documents*, not *your model*.
//! 3. **alo's own service is not an exception.** ADR 0014 says our address is
//!    not privileged and a policy refusing hosted inference refuses ours. A
//!    subscription does not make somebody's correspondence less theirs.
//!
//! # The errand this would need does not exist
//!
//! `alo_egress::Errand` is a closed list of the reasons this machine may reach
//! the network: signing in, fetching a model, checking for an update,
//! installing an application and two more about applications. **None of them is
//! *sending what a model learned from this person's documents*,** and this plan
//! may not edit that crate.
//!
//! That is the right state for today — there is nothing here that could send one
//! — and it is a decision somebody must take before there is: the variant, its
//! sentence, and whether the indicator says something stronger than it says for
//! a model coming *down*. [`why_an_adapted_model_cannot_simply_be_sent`] is that
//! finding, written where the next person to try will find it.

/// **What sending an adapted model would actually be**, for whoever writes the
/// code that does it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhatItWouldBe {
    /// What is really leaving, in one line a person would recognise.
    pub what_leaves: &'static str,
    /// Why the obvious words for it are the wrong ones.
    pub why_the_usual_words_mislead: &'static str,
    /// What this machine would have to have first.
    pub what_it_needs_first: &'static str,
    /// Whether alo's own service changes the answer.
    pub our_own_service: &'static str,
}

/// The finding, as one statement.
#[must_use]
pub const fn why_an_adapted_model_cannot_simply_be_sent() -> WhatItWouldBe {
    WhatItWouldBe {
        what_leaves: "the contents of the folders a person granted for training: an adapted \
                      model carries the sentences it was trained on, and they can be drawn back \
                      out of it",
        why_the_usual_words_mislead: "\"uploading a model\" names the file and hides the act. A \
                                      person who would never send their correspondence to a \
                                      service may agree to send \"their model\" without knowing \
                                      those are the same bytes in a different shape",
        what_it_needs_first: "an `alo_egress::Errand` for it, which that closed list does not \
                              have, with a sentence that says whose documents are leaving — and \
                              an indicator that is at least as loud as the one for a model \
                              coming down, since this one carries the person's own writing",
        our_own_service: "no exception (ADR 0014): alo's address is not privileged in \
                          `alo-egress`, a policy refusing hosted inference refuses ours, and a \
                          subscription does not make somebody's correspondence less theirs",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The finding says what leaves, why the usual words are wrong, and what
    /// is missing** — or it is a note rather than a finding.
    #[test]
    fn the_finding_names_the_documents_the_words_and_the_missing_errand() {
        let finding = why_an_adapted_model_cannot_simply_be_sent();
        assert!(finding.what_leaves.contains("trained on"));
        assert!(
            finding
                .why_the_usual_words_mislead
                .contains("correspondence")
        );
        assert!(finding.what_it_needs_first.contains("Errand"));
        assert!(finding.our_own_service.contains("0014"));
    }
}
