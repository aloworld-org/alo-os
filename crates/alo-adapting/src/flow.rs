//! **The road a person walks to teach their model something: five decisions.**
//!
//! `docs/features.md`: *LoRA/QLoRA over a granted folder or a tenant's records,
//! **as a flow rather than a toolchain***. Task 4 of
//! `docs/autonomy/v0-5-models-a-person-adapts-and-subscribes-to-plan.md`.
//!
//! A toolchain asks somebody to hold a rank, a learning rate and an epoch count
//! in their head before they may begin. A flow asks them five things they
//! already know the answers to:
//!
//! 1. **which folder** — one they have already granted, picked in the picker;
//! 2. **what they want it to be better at**, in their own words;
//! 3. **what it will learn from**, shown before anything starts, with what it
//!    will skip and why;
//! 4. **start** — one approval, because this is a change (ADR 0001);
//! 5. **what it cost, and keep it or not.**
//!
//! # Nothing here names anything we rented
//!
//! No step says LoRA, a rank, a learning rate, an epoch, a checkpoint or the
//! name of the trainer — *a person never learns the name of anything we rented*
//! — and `tests/the_flow_names_nothing_rented.rs` reads this file and the
//! vocabulary to keep it so. Whoever wants those values has
//! `docs/contracts/fine-tuning-values.md` and a file; they are not a step.
//!
//! # An agent may propose this and may never start it
//!
//! Training on a person's documents is the most personal act this machine
//! performs: it makes something that carries their writing. So an agent's verb
//! **proposes**, and a person approves the sentence — [`Flow::proposed_by`] is
//! the only road from an agent, and it begins at the same step a person's own
//! road begins at, with nothing pre-approved.

use std::path::Path;

use alo_picking::Picked;

use crate::dataset::Dataset;
use crate::words::{self, Word};

/// **Where a person is on the road.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Which folder. One they granted, and never a path typed.
    WhichFolder,
    /// What they want the model to be better at, in their words.
    WhatItShouldLearn,
    /// What it will learn from, shown before anything runs.
    WhatItWillLearnFrom,
    /// One approval, because this changes the machine.
    Start,
    /// What it cost, and whether they keep it.
    KeepItOrNot,
}

impl Step {
    /// The five, in the order a person meets them.
    pub const ALL: [Self; 5] = [
        Self::WhichFolder,
        Self::WhatItShouldLearn,
        Self::WhatItWillLearnFrom,
        Self::Start,
        Self::KeepItOrNot,
    ];

    /// The sentence a person reads at this step.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::WhichFolder => words::WHICH_FOLDER,
            Self::WhatItShouldLearn => words::WHAT_IT_SHOULD_LEARN,
            Self::WhatItWillLearnFrom => words::WHAT_IT_WILL_LEARN_FROM,
            Self::Start => words::START_TEACHING,
            Self::KeepItOrNot => words::KEEP_IT_OR_NOT,
        }
    }
}

/// **One person's road through a fine-tune.**
///
/// A type rather than a screen: the shell draws it, and what may happen next is
/// decided here so that two surfaces cannot disagree about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flow {
    /// Where they are.
    step: Step,
    /// The folder they picked, once they have.
    folder: Option<Picked>,
    /// What they said they want, in their own words.
    what_they_want: Option<String>,
    /// What it will learn from, once the machine has looked.
    dataset: Option<Dataset>,
    /// Whether an agent proposed this rather than a person beginning it.
    proposed_by_an_agent: Option<String>,
    /// Whether the person approved it.
    approved: bool,
}

impl Flow {
    /// A person begins.
    #[must_use]
    pub fn begun() -> Self {
        Self {
            step: Step::WhichFolder,
            folder: None,
            what_they_want: None,
            dataset: None,
            proposed_by_an_agent: None,
            approved: false,
        }
    }

    /// **An agent proposes one**, naming itself. It begins where a person's
    /// own begins, and nothing is approved by having been proposed.
    #[must_use]
    pub fn proposed_by(agent: &str) -> Self {
        Self {
            proposed_by_an_agent: Some(agent.to_owned()),
            ..Self::begun()
        }
    }

    /// Where the person is.
    #[must_use]
    pub fn step(&self) -> Step {
        self.step
    }

    /// Which agent proposed this, where one did.
    #[must_use]
    pub fn proposed_by_an_agent(&self) -> Option<&str> {
        self.proposed_by_an_agent.as_deref()
    }

    /// Whether a person has approved it.
    #[must_use]
    pub fn approved(&self) -> bool {
        self.approved
    }

    /// The folder, once picked.
    #[must_use]
    pub fn folder(&self) -> Option<&Path> {
        self.folder.as_ref().map(Picked::folder)
    }

    /// What they said they want the model to be better at.
    #[must_use]
    pub fn what_they_want(&self) -> Option<&str> {
        self.what_they_want.as_deref()
    }

    /// What it will learn from, once the machine has looked.
    #[must_use]
    pub fn dataset(&self) -> Option<&Dataset> {
        self.dataset.as_ref()
    }

    /// **They picked a folder** — one they granted.
    #[must_use]
    pub fn with_the_folder(mut self, folder: Picked) -> Self {
        self.folder = Some(folder);
        self.step = Step::WhatItShouldLearn;
        self
    }

    /// **They said what they want it to be better at**, in their own words.
    ///
    /// Empty is not an answer, and the step does not move: a fine-tune nobody
    /// can say the purpose of is one nobody can judge afterwards.
    #[must_use]
    pub fn wanting(mut self, in_their_words: &str) -> Self {
        if in_their_words.trim().is_empty() {
            return self;
        }
        self.what_they_want = Some(in_their_words.trim().to_owned());
        self.step = Step::WhatItWillLearnFrom;
        self
    }

    /// **The machine looked, and this is what it will learn from.**
    #[must_use]
    pub fn having_looked(mut self, dataset: Dataset) -> Self {
        self.dataset = Some(dataset);
        self.step = Step::Start;
        self
    }

    /// **One approval, from the person.**
    ///
    /// The only way past [`Step::Start`], whoever proposed the fine-tune. An
    /// agent that proposed it cannot approve it; nothing here takes an agent's
    /// word for this.
    #[must_use]
    pub fn a_person_approved(mut self) -> Self {
        if self.step == Step::Start {
            self.approved = true;
            self.step = Step::KeepItOrNot;
        }
        self
    }

    /// **Whether training may begin.**
    ///
    /// Every step answered and a person's approval given. This is the one
    /// question the thing that runs a fine-tune asks, so that *nothing trains
    /// without an approval* is a property of the type.
    #[must_use]
    pub fn may_begin(&self) -> bool {
        self.approved
            && self.folder.is_some()
            && self.what_they_want.is_some()
            && self.dataset.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The five steps are five sentences**, each its own.
    #[test]
    fn every_step_is_a_sentence_of_its_own() {
        let mut keys: Vec<String> = Step::ALL
            .into_iter()
            .map(|step| step.word().key().to_string())
            .collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), Step::ALL.len(), "two steps share a sentence");
    }

    /// **Nothing trains until a person has answered every step and approved.**
    #[test]
    fn nothing_may_begin_until_a_person_has_answered_and_approved() {
        let begun = Flow::begun();
        assert_eq!(begun.step(), Step::WhichFolder);
        assert!(!begun.may_begin());

        // Approval cannot be reached by asking for it early.
        let early = Flow::begun().a_person_approved();
        assert!(
            !early.approved(),
            "an approval was taken before the question"
        );
        assert!(!early.may_begin());
    }

    /// **An answer that says nothing does not move the road on.**
    #[test]
    fn a_purpose_nobody_stated_is_not_an_answer() {
        let picked = crate::dataset::tests::granted("Invoices");
        let flow = Flow::begun().with_the_folder(picked.0).wanting("   ");
        assert_eq!(flow.step(), Step::WhatItShouldLearn);
        assert_eq!(flow.what_they_want(), None);
    }

    /// **An agent proposes, and a person approves** — never the other way
    /// about, whoever asked for it.
    #[test]
    fn an_agent_may_propose_and_may_not_approve() {
        let (picked, _inside, _home) = crate::dataset::tests::granted("Invoices");
        let proposed = Flow::proposed_by("@the-agent");
        assert_eq!(proposed.proposed_by_an_agent(), Some("@the-agent"));
        assert_eq!(
            proposed.step(),
            Step::WhichFolder,
            "an agent's proposal starts where a person's does"
        );
        assert!(!proposed.approved());

        let dataset = crate::dataset::tests_support::one_file_dataset().0;
        let ready = proposed
            .with_the_folder(picked)
            .wanting("know my invoices")
            .having_looked(dataset);
        assert_eq!(ready.step(), Step::Start);
        assert!(
            !ready.may_begin(),
            "a proposal became a run without a person"
        );

        let approved = ready.a_person_approved();
        assert!(approved.may_begin());
        assert_eq!(approved.step(), Step::KeepItOrNot);
        assert_eq!(
            approved.proposed_by_an_agent(),
            Some("@the-agent"),
            "who proposed it is not forgotten once it is approved"
        );
    }
}
