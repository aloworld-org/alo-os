//! One place that could be asked instead, and the sentence a person approves
//! to allow it.
//!
//! # This is `alo-capability`'s `Proposal`, and it deliberately is not one
//!
//! ADR 0001 §5 says a change comes back as a proposal carrying a sentence
//! describing exactly what it will do, that what a person approves is that
//! sentence, and that **an approval is never a session**. An offer is that
//! shape, for a question rather than for a change to the machine.
//!
//! It is not an `alo_capability::Proposal` because there is no verb here, no
//! grant and no path. Building one would mean inventing a verb whose argument
//! is a person's own question, which puts the thing ADR 0001 §4 keeps *out* of
//! the capability model — what somebody typed — inside it. So the shape is
//! borrowed and the machinery is not, and this file says so rather than leaving
//! somebody to wonder where the proposal went.
//!
//! # An offer cannot be made from outside
//!
//! `Offer::of` is `pub(crate)` and [`crate::Elsewhere`] is the only caller,
//! which is what makes an offer *proof that the policy permitted this place*.
//! A caller that could build one would be a caller that could offer somebody a
//! provider their organisation forbade, and the person approving it would have
//! no way to tell the difference — it is the same reasoning that keeps
//! `alo_egress::Departing` constructible only by the indicator.
//!
//! # Three sentences, because leaving is the thing being approved
//!
//! Where the question would go is not a detail beside the offer; it is what the
//! offer is about. So *nothing leaves*, *it leaves this machine and stays on
//! your network* and *it leaves the building* are three whole sentences rather
//! than one with a clause glued on, which is `alo-egress`' item 9h decision met
//! from a fourth side.

use alo_models::InferenceSource;
use alo_strings::{Filling, Said, Strings};

use crate::words;

/// One place this machine could put the question instead, awaiting one
/// approval.
///
/// Carries nothing but the place. The question is not here — this crate never
/// holds one — so an offer is a decision about *where*, made by somebody who
/// already knows *what* they asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// Where the question would go.
    source: InferenceSource,
}

impl Offer {
    /// This place, which the policy has already permitted.
    ///
    /// `pub(crate)`: see this module's documentation. The only caller is
    /// [`crate::Elsewhere`], which asks the policy first.
    pub(crate) fn of(source: InferenceSource) -> Self {
        Self { source }
    }

    /// Where the question would go.
    #[must_use]
    pub fn source(&self) -> &InferenceSource {
        &self.source
    }

    /// Whether approving this would send anything off this machine.
    ///
    /// True for a paired machine as well as a provider: *it only went down the
    /// corridor* is the exception that would erode law 1, and it is the
    /// difference between two of the three sentences below rather than a reason
    /// to stay quiet.
    #[must_use]
    pub fn causes_egress(&self) -> bool {
        self.source.causes_egress()
    }

    /// The string this crate declares for this offer.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self.source {
            InferenceSource::ThisMachine => words::ASK_HERE_INSTEAD,
            InferenceSource::PairedMachine { .. } => words::ASK_IN_THE_BUILDING_INSTEAD,
            InferenceSource::Hosted { .. } => words::ASK_OUTSIDE_INSTEAD,
        }
    }

    /// The sentence a person approves, in the language they read.
    ///
    /// The place goes in through
    /// [`and_said`](alo_strings::Filling::and_said), because it is a clause
    /// `alo-models` words rather than a piece of data: a sentence somebody
    /// translated with an English place still inside it is not a translated
    /// sentence, and this is the one sentence in the crate where that matters
    /// most.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::answering_words`] answers with the key, marked.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &self.word().key(),
            &Filling::nothing().and_said("source", &self.source.said(strings)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{here, hosted, in_english, paired, translated};

    /// **The sentence says where the question goes and that it is once.** Both
    /// halves are what is being approved: a person who reads only *ask
    /// somewhere else* has approved a place they were not told.
    #[test]
    fn the_sentence_names_the_place_and_says_it_is_worth_one_question() {
        let strings = in_english();
        assert_eq!(
            Offer::of(hosted()).said(&strings).text(),
            "have this question answered by alo, in the EU instead, just this once — the question \
             would leave this machine and the building"
        );
        assert_eq!(
            Offer::of(paired()).said(&strings).text(),
            "have this question answered on the studio workstation, on your network instead, just \
             this once — the question would leave this machine and stay on your network"
        );
        assert_eq!(
            Offer::of(here()).said(&strings).text(),
            "have this question answered on this machine instead, just this once — it would not \
             leave this machine"
        );
    }

    /// **The three places are three sentences, and each one says what leaving
    /// means there.** A machine in the next room is egress and says so; this
    /// machine is not and says that.
    #[test]
    fn what_leaves_is_different_in_each_of_the_three_and_each_one_says_so() {
        assert!(!Offer::of(here()).causes_egress());
        assert!(Offer::of(paired()).causes_egress());
        assert!(Offer::of(hosted()).causes_egress());

        let strings = in_english();
        assert!(
            Offer::of(paired())
                .said(&strings)
                .text()
                .contains("stay on your network")
        );
        assert!(
            Offer::of(hosted())
                .said(&strings)
                .text()
                .contains("and the building")
        );
    }

    /// **A sentence is only as translated as the place inside it.** An offer
    /// somebody translated, naming a provider in English, is a line half its
    /// reader cannot read — and an offer is the one sentence in this crate a
    /// person acts on.
    #[test]
    fn an_offer_naming_an_untranslated_place_does_not_claim_to_be_translated() {
        let half = translated(&[(
            words::ASK_OUTSIDE_INSTEAD,
            "diese Frage stattdessen {source} beantworten lassen, nur dieses eine Mal — sie würde \
             diesen Rechner und das Haus verlassen",
        )]);
        let said = Offer::of(hosted()).said(&half);
        assert!(!said.is_translated(), "{said}");
        assert!(said.text().starts_with("diese Frage"), "{said}");

        // With the place translated too, the whole line is.
        let whole = translated(&[
            (
                words::ASK_OUTSIDE_INSTEAD,
                "diese Frage stattdessen {source} beantworten lassen, nur dieses eine Mal — sie \
                 würde diesen Rechner und das Haus verlassen",
            ),
            (
                alo_models::words::BY_A_PROVIDER,
                "von {provider}, in {region}",
            ),
        ]);
        let said = Offer::of(hosted()).said(&whole);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().contains("von alo"), "{said}");
        // The provider's name and the region it stated are theirs, not the
        // language's.
        assert!(said.text().contains("the EU"), "{said}");
    }

    /// **The offer does not go blank because nobody declared the words.**
    /// Whoever has to fix it finds out from the sentence rather than from an
    /// empty dialogue with a button on it.
    #[test]
    fn an_offer_without_the_words_still_names_the_key() {
        let nothing = Strings::of(alo_strings::Vocabulary::empty());
        let said = Offer::of(hosted()).said(&nothing);
        assert!(said.is_a_bug());
        assert!(said.text().contains("answering.offer.outside"), "{said}");
    }
}
