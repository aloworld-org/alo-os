//! Which model a machine gives the agent, and what it says when none of them
//! can have it.
//!
//! This is [`crate::Catalogue`]'s recommendation, and it is the one
//! [ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md) names as
//! having been wrong: *a model that runs beautifully on a laptop and cannot
//! emit a valid verb call is useless as an agent, and the catalogue would
//! currently recommend it*. So the recommendation asks three questions rather
//! than two — does it run here, may it be used, and has it been measured
//! driving the verbs — and answers with a refusal rather than with nothing when
//! the answer is no.
//!
//! # The method is not called `default_for_cpu` any more
//!
//! ADR 0007's own correction is that **"default" was the wrong word**: it
//! implies a ranking, and the ranking invited the reading that the CPU path is
//! the real one and everything else is a bonus. What the machine is actually
//! answering is *which model gets the agent here*, so that is what the method is
//! called. Listing what a machine can **run** is still
//! [`Catalogue::runnable_on_cpu`](crate::Catalogue::runnable_on_cpu), unchanged
//! and still a plain answer: running a model and giving it the agent are two
//! questions, and conflating them is the mistake this file exists to undo.
//!
//! # A refusal that names the alternatives, and picks neither
//!
//! ADR 0008 permits three places and forbids moving between them silently. A
//! machine with no model that clears the bar therefore says so and names the
//! other two — and [`NoAgentHere::lines`] is the only road to the sentence, so
//! there is no way to show somebody the refusal without the alternatives under
//! it. Nothing here returns an [`crate::InferenceSource`]: this crate cannot
//! substitute a place, because it has no method that answers with one.

use alo_strings::{Filling, Said, Strings};

use crate::catalogue::{Catalogue, Model, OnCpu};
use crate::words;

/// Why this machine has no model to give the agent.
///
/// Three reasons, and telling them apart is the whole value of the type: *there
/// was nothing to choose from*, *nobody has measured what there was*, and *what
/// was measured is not good enough* send a person to three different places,
/// and a machine with one sentence for all three would be claiming a
/// measurement in the case where none was run.
///
/// The counts are accessors rather than gaps in the sentence, which is this
/// crate's rule from item 10: a line saying *2 models* would be English's two
/// plural shapes standing in for Polish's three. Whoever draws the panel counts
/// them in the reader's own language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoAgentHere {
    /// Nothing in the catalogue both runs on this machine and may be used
    /// without reading a licence first, so there was nothing to measure
    /// against.
    NothingToChooseFrom,
    /// There were models to choose from and none of them has been measured.
    /// Not a verdict on those models — see [`crate::Driving::NotMeasured`].
    NoneMeasured {
        /// How many models run here and may be used.
        to_choose_from: usize,
    },
    /// There were models to choose from, some were measured, and none of the
    /// measured ones drives the verbs dependably enough.
    NoneClearsTheBar {
        /// How many models run here and may be used.
        to_choose_from: usize,
        /// How many of those anybody has measured. The rest are unmeasured, and
        /// the difference is why this carries two numbers rather than one.
        measured: usize,
    },
}

impl NoAgentHere {
    /// How many models this machine could have chosen between.
    #[must_use]
    pub fn to_choose_from(self) -> usize {
        match self {
            Self::NothingToChooseFrom => 0,
            Self::NoneMeasured { to_choose_from }
            | Self::NoneClearsTheBar { to_choose_from, .. } => to_choose_from,
        }
    }

    /// How many of those anybody has measured.
    #[must_use]
    pub fn measured(self) -> usize {
        match self {
            Self::NothingToChooseFrom | Self::NoneMeasured { .. } => 0,
            Self::NoneClearsTheBar { measured, .. } => measured,
        }
    }

    /// The string this crate declares for this reason.
    #[must_use]
    pub fn word(self) -> words::Word {
        match self {
            Self::NothingToChooseFrom => words::NOTHING_TO_CHOOSE_FROM,
            Self::NoneMeasured { .. } => words::NONE_MEASURED,
            Self::NoneClearsTheBar { .. } => words::NONE_CLEARS_THE_BAR,
        }
    }

    /// What a person is shown: why, and what is still open to them.
    ///
    /// **Two lines, and there is no method that gives you only the first.**
    /// That is the shape of *never substituted silently*: a screen cannot
    /// accidentally show somebody that their machine has no agent without also
    /// showing them the two places that would still answer, because the type
    /// does not offer that.
    ///
    /// Two strings rather than one long sentence, which is this crate's rule
    /// from [`crate::Tried`]: the separator between two lines is not
    /// punctuation a program can pick, so the panel draws them as lines and a
    /// translator writes each one whole.
    ///
    /// Never fails and never panics, for the reason
    /// [`crate::NotAllowed::said`] does not: a `Strings` that was never given
    /// [`crate::model_words`] answers with the key, marked, and the machine
    /// still has no model for the agent either way.
    #[must_use]
    pub fn lines(self, strings: &Strings) -> [Said; 2] {
        [
            strings.say(&self.word().key(), &Filling::nothing()),
            strings.say(&words::THE_OTHER_PLACES.key(), &Filling::nothing()),
        ]
    }
}

impl Catalogue {
    /// **Which model this machine gives the agent, with no graphics card.**
    ///
    /// Three questions, in this order, and the order is what makes the refusal
    /// say something true. It must **run** here — inside the memory the machine
    /// has, and not so slowly that a turn's several calls become a wait
    /// (ADR 0007). It must be **usable** without somebody reading a licence
    /// first, which is [`Model::safe_default_for_business`] and is conservative
    /// on purpose. And it must have been **measured driving the verbs**, which
    /// is [`crate::Driving::clears_the_bar`].
    ///
    /// Among those, comfortable before workable and then larger before smaller,
    /// which is the ordering this method had before the bar existed: a model
    /// that answers slowly is not a better agent for being cleverer, because a
    /// turn makes several calls and the waiting multiplies.
    ///
    /// # Errors
    /// [`NoAgentHere`], saying which of the three questions emptied the list —
    /// and naming the two places ADR 0008 leaves open, rather than quietly
    /// becoming one of them.
    pub fn agent_for_cpu(&self, ram_gb: f32) -> Result<&Model, NoAgentHere> {
        let choices = self.to_choose_from_on_cpu(ram_gb);
        if choices.is_empty() {
            return Err(NoAgentHere::NothingToChooseFrom);
        }
        let to_choose_from = choices.len();
        let measured = choices
            .iter()
            .filter(|model| model.drives_verbs.has_been_measured())
            .count();
        choices
            .into_iter()
            .filter(|model| model.can_be_the_agent())
            .max_by(|a, b| {
                let rank = |m: &Model| u8::from(m.on_cpu == OnCpu::Comfortable);
                rank(a)
                    .cmp(&rank(b))
                    .then(a.parameters_b.total_cmp(&b.parameters_b))
            })
            .ok_or(if measured == 0 {
                NoAgentHere::NoneMeasured { to_choose_from }
            } else {
                NoAgentHere::NoneClearsTheBar {
                    to_choose_from,
                    measured,
                }
            })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// A catalogue written for one question, so what each entry answers is
    /// visible in the test that reads it.
    fn catalogue(entries: &[(&str, f32, &str, &str, &str)]) -> Catalogue {
        let mut text = String::new();
        for (id, parameters_b, on_cpu, commercial, driving) in entries {
            text.push_str(&format!(
                "[[model]]\nid = \"{id}\"\nname = \"{id}\"\npublisher = \"p\"\n\
                 parameters_b = {parameters_b}\nquantisation = \"Q4_K_M\"\n\
                 download_bytes = 1\nmin_vram_gb = 2.0\nmin_ram_gb = 4.0\n\
                 on_cpu = \"{on_cpu}\"\ndrives_verbs = \"{driving}\"\n\
                 upstream = \"https://example.test/{id}\"\n\
                 licence = {{ name = \"L\", commercial_use = \"{commercial}\", \
                 note = \"conditions, stated\" }}\n\n"
            ));
        }
        Catalogue::parse(&text).unwrap()
    }

    /// **A model below the bar is not offered for agent work**, however well it
    /// runs. This is the item's own sentence, and the catalogue it is asked of
    /// is one where the unmeasured model is the biggest and fastest — so a
    /// method that had kept its old ordering and forgotten the bar would return
    /// it.
    #[test]
    fn a_model_that_has_not_been_measured_is_not_given_the_agent() {
        let c = catalogue(&[
            ("big", 9.0, "comfortable", "permitted", "not-measured"),
            ("small", 2.0, "comfortable", "permitted", "reliably"),
        ]);
        assert_eq!(c.agent_for_cpu(16.0).unwrap().id, "small");

        let c = catalogue(&[("big", 9.0, "comfortable", "permitted", "not-measured")]);
        assert_eq!(
            c.agent_for_cpu(16.0).unwrap_err(),
            NoAgentHere::NoneMeasured { to_choose_from: 1 }
        );
    }

    /// A model that was measured and did not clear the bar is refused with the
    /// reason that actually happened, which is not the same reason as never
    /// having been measured.
    #[test]
    fn a_measurement_that_failed_is_not_reported_as_no_measurement() {
        let c = catalogue(&[
            ("tried", 3.0, "comfortable", "permitted", "sometimes"),
            ("worse", 2.0, "comfortable", "permitted", "rarely"),
            ("unknown", 4.0, "comfortable", "permitted", "not-measured"),
        ]);
        assert_eq!(
            c.agent_for_cpu(16.0).unwrap_err(),
            NoAgentHere::NoneClearsTheBar {
                to_choose_from: 3,
                measured: 2,
            }
        );
        assert_eq!(c.agent_for_cpu(16.0).unwrap_err().to_choose_from(), 3);
        assert_eq!(c.agent_for_cpu(16.0).unwrap_err().measured(), 2);
    }

    /// The licence gate survived the bar being added: a machine is not quietly
    /// handed the one model an organisation may not use, however well it drives
    /// the verbs.
    #[test]
    fn the_choice_is_still_licence_gated() {
        let c = catalogue(&[(
            "conditional",
            3.0,
            "comfortable",
            "with-conditions",
            "reliably",
        )]);
        assert_eq!(
            c.agent_for_cpu(16.0).unwrap_err(),
            NoAgentHere::NothingToChooseFrom
        );
        assert_eq!(c.agent_for_cpu(16.0).unwrap_err().to_choose_from(), 0);
        // It still runs here. Running a model and giving it the agent are two
        // questions, and this method answers only the second.
        assert_eq!(c.runnable_on_cpu(16.0).len(), 1);
    }

    /// Comfortable before workable, then larger before smaller — the ordering
    /// this method had before the bar existed, kept.
    #[test]
    fn a_comfortable_model_beats_a_cleverer_one_that_makes_a_person_wait() {
        let c = catalogue(&[
            ("clever", 9.0, "workable", "permitted", "reliably"),
            ("quick", 3.0, "comfortable", "permitted", "reliably"),
            ("quicker", 2.0, "comfortable", "permitted", "reliably"),
        ]);
        assert_eq!(c.agent_for_cpu(16.0).unwrap().id, "quick");
    }

    /// A machine too small for anything gets the answer that is true of it:
    /// there was nothing to choose between, so nothing was measured and nothing
    /// failed a measurement.
    #[test]
    fn a_machine_with_nothing_that_runs_on_it_says_so() {
        let c = catalogue(&[("big", 9.0, "comfortable", "permitted", "reliably")]);
        assert_eq!(
            c.agent_for_cpu(1.0).unwrap_err(),
            NoAgentHere::NothingToChooseFrom
        );
        assert!(c.runnable_on_cpu(1.0).is_empty());
    }

    /// **The refusal names the alternatives, and there is no way to show it
    /// without them.** ADR 0008 permits three places and forbids moving between
    /// them silently; this is the sentence a person reads instead of a machine
    /// choosing for them.
    #[test]
    fn every_refusal_carries_the_other_places_and_picks_neither() {
        let strings = in_english();
        for refusal in [
            NoAgentHere::NothingToChooseFrom,
            NoAgentHere::NoneMeasured { to_choose_from: 3 },
            NoAgentHere::NoneClearsTheBar {
                to_choose_from: 3,
                measured: 3,
            },
        ] {
            let [why, elsewhere] = refusal.lines(&strings);
            assert!(!why.text().is_empty(), "{refusal:?}");
            assert!(elsewhere.text().contains("paired with"), "{elsewhere}");
            assert!(elsewhere.text().contains("provider"), "{elsewhere}");
            assert!(
                elsewhere.text().contains("will not choose for you"),
                "{elsewhere}"
            );
        }
    }

    /// The three reasons are three sentences. A machine that said the same
    /// thing about a measurement it ran and one it did not would be making the
    /// claim this whole item exists to stop.
    #[test]
    fn the_three_reasons_do_not_share_a_sentence() {
        let strings = in_english();
        let said = |refusal: NoAgentHere| refusal.lines(&strings)[0].text().to_owned();
        let nothing = said(NoAgentHere::NothingToChooseFrom);
        let unmeasured = said(NoAgentHere::NoneMeasured { to_choose_from: 2 });
        let failed = said(NoAgentHere::NoneClearsTheBar {
            to_choose_from: 2,
            measured: 2,
        });
        assert_ne!(nothing, unmeasured);
        assert_ne!(unmeasured, failed);
        assert_ne!(nothing, failed);
        assert!(unmeasured.contains("has been measured"), "{unmeasured}");
    }

    /// Both lines are read in the reader's own language, and a machine that had
    /// translated the refusal but not the alternatives says so rather than
    /// looking finished.
    #[test]
    fn the_refusal_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::NONE_MEASURED,
            "kein Modell auf diesem Rechner wurde gemessen",
        )]);
        let [why, elsewhere] = NoAgentHere::NoneMeasured { to_choose_from: 2 }.lines(&strings);
        assert!(why.is_translated());
        assert!(why.text().contains("gemessen"), "{why}");
        assert!(!elsewhere.is_translated());
    }

    /// **Nothing here answers with a place to ask instead.** The guarantee is
    /// the absence of a method rather than a rule somebody remembers: this
    /// crate can name the alternatives and cannot return one, so there is no
    /// path from *no agent here* to a question leaving the machine.
    #[test]
    fn a_refusal_carries_no_source_to_fall_back_to() {
        // Every variant, every field, named. A variant that gained an
        // `InferenceSource` would stop this compiling, so whoever adds one is
        // asked the question rather than told the answer.
        let all_it_holds = |refusal: NoAgentHere| match refusal {
            NoAgentHere::NothingToChooseFrom => 0,
            NoAgentHere::NoneMeasured { to_choose_from } => to_choose_from,
            NoAgentHere::NoneClearsTheBar {
                to_choose_from,
                measured,
            } => to_choose_from + measured,
        };
        assert_eq!(all_it_holds(NoAgentHere::NothingToChooseFrom), 0);
        assert_eq!(
            all_it_holds(NoAgentHere::NoneMeasured { to_choose_from: 1 }),
            1
        );
        assert_eq!(
            all_it_holds(NoAgentHere::NoneClearsTheBar {
                to_choose_from: 3,
                measured: 2,
            }),
            5
        );
    }
}
