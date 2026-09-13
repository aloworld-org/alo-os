//! Why a catalogue entry has no grade.
//!
//! [`crate::Driving::NotMeasured`] used to be the whole of what an entry said
//! when nobody had run `alo-driving` against it — and it read, to anybody
//! choosing a model, as *probably fine, nobody checked*. Three different facts
//! wore that one grade: the machine measuring the catalogue could not hold the
//! model, the runtime could not answer with the file, or there were no weights
//! to measure. They send a person to different places, so an entry now says
//! which, in a sentence a person reads in their own language.
//!
//! # Every reason is about the machine that measured, never the reader's
//!
//! A catalogue is measured on one machine and shipped to thousands. *Too large
//! for this machine* beside an entry on a workstation that could run it
//! comfortably would be a claim about the wrong machine, so the sentence says
//! *the machine that measured this catalogue*, and the block names that machine,
//! its memory, the day and the runtime — the same three things
//! [`crate::MeasuredOn`] asks of a grade, because a reason is also the result of
//! somebody trying.

use alo_strings::{Filling, Said, Strings};
use serde::Deserialize;

use crate::measured_on::MeasuredOn;
use crate::words::{self, Word};

/// The three reasons an entry can carry instead of a grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WhyUnmeasured {
    /// The machine that measures the catalogue does not have the memory to
    /// run the model inside the time `alo-models` waits for an answer.
    TooLargeForTheMeasuringMachine,
    /// The pinned runtime loaded the file and could not answer with it.
    TheRuntimeRefusedTheFile,
    /// There are no published weights the entry can name.
    WeightsNotPublished,
}

impl WhyUnmeasured {
    /// The string this crate declares for this reason.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::TooLargeForTheMeasuringMachine => words::UNMEASURED_TOO_LARGE,
            Self::TheRuntimeRefusedTheFile => words::UNMEASURED_RUNTIME_REFUSED,
            Self::WeightsNotPublished => words::UNMEASURED_NOT_PUBLISHED,
        }
    }
}

/// **Why an entry has no grade, and where that was found.**
///
/// Present only beside `drives_verbs = "not-measured"`, and
/// [`crate::Catalogue::parse`] refuses it beside a grade. Every field is
/// required: a reason with no machine is the same claim a grade with no machine
/// is.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unmeasured {
    /// Which of the three.
    pub because: WhyUnmeasured,

    /// The machine the attempt was made on, with its memory.
    pub machine: String,

    /// The day it was made, as `YYYY-MM-DD`.
    pub date: String,

    /// The runtime it was made under, with its version.
    pub runtime: String,
}

impl Unmeasured {
    /// What a person reads beside the entry.
    ///
    /// Never fails, for [`crate::NoAgentHere::lines`]'s reason: a `Strings`
    /// that was never given [`crate::model_words`] answers with the key,
    /// marked.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.because.word().key(), &Filling::nothing())
    }

    /// The machine, day and runtime, as the statement a grade would carry.
    #[must_use]
    pub fn tried_on(&self) -> MeasuredOn {
        MeasuredOn {
            machine: self.machine.clone(),
            date: self.date.clone(),
            runtime: self.runtime.clone(),
        }
    }

    /// What is wrong with this statement, or [`None`] when it holds together.
    ///
    /// The same three checks a grade's machine is held to.
    pub(crate) fn what_is_wrong_with_it(&self) -> Option<&'static str> {
        self.tried_on().what_is_wrong_with_it()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    fn sound(because: WhyUnmeasured) -> Unmeasured {
        Unmeasured {
            because,
            machine: "Apple M3, 8 GB unified memory".to_owned(),
            date: "2026-09-13".to_owned(),
            runtime: "Ollama 0.34.0".to_owned(),
        }
    }

    const EVERY_REASON: [WhyUnmeasured; 3] = [
        WhyUnmeasured::TooLargeForTheMeasuringMachine,
        WhyUnmeasured::TheRuntimeRefusedTheFile,
        WhyUnmeasured::WeightsNotPublished,
    ];

    /// **Each reason is its own sentence, in the vocabulary, and says it is
    /// not a grade.**
    #[test]
    fn every_reason_is_a_sentence_of_its_own() {
        let strings = in_english();
        let said: Vec<String> = EVERY_REASON
            .iter()
            .map(|why| sound(*why).said(&strings).text().to_owned())
            .collect();
        for (i, one) in said.iter().enumerate() {
            assert!(one.starts_with("not measured yet"), "{one}");
            for other in said.iter().skip(i + 1) {
                assert_ne!(one, other);
            }
        }
        for why in EVERY_REASON {
            assert!(words::EVERY_WORD.contains(&why.word()), "{why:?}");
        }
    }

    /// **No reason is said about the reader's machine.** The catalogue is
    /// measured on one machine and read on others.
    #[test]
    fn no_reason_claims_something_about_the_readers_machine() {
        let strings = in_english();
        for why in EVERY_REASON {
            let text = sound(why).said(&strings).text().to_owned();
            assert!(!text.contains("this machine"), "{why:?}: {text}");
            assert!(!text.contains("your"), "{why:?}: {text}");
        }
    }

    /// A reason is refused on the same three things a grade's machine is.
    #[test]
    fn a_reason_with_no_machine_is_refused() {
        assert_eq!(
            sound(WhyUnmeasured::WeightsNotPublished).what_is_wrong_with_it(),
            None
        );
        let blank = Unmeasured {
            machine: " ".to_owned(),
            ..sound(WhyUnmeasured::TheRuntimeRefusedTheFile)
        };
        assert!(
            blank
                .what_is_wrong_with_it()
                .is_some_and(|why| why.contains("no machine in particular"))
        );
        let no_day = Unmeasured {
            date: "yesterday".to_owned(),
            ..sound(WhyUnmeasured::TooLargeForTheMeasuringMachine)
        };
        assert!(
            no_day
                .what_is_wrong_with_it()
                .is_some_and(|why| why.contains("no day"))
        );
    }

    /// **The reason is one of three, spelled as the catalogue spells it**, and
    /// a reason nobody defined fails to load rather than reading as one.
    #[test]
    fn a_reason_nobody_defined_is_not_a_reason() {
        let written = |because: &str| {
            format!(
                "because = \"{because}\"\nmachine = \"a machine, 8 GB\"\n\
                 date = \"2026-09-13\"\nruntime = \"Ollama 0.34.0\"\n"
            )
        };
        for (spelled, why) in [
            (
                "too-large-for-the-measuring-machine",
                WhyUnmeasured::TooLargeForTheMeasuringMachine,
            ),
            (
                "the-runtime-refused-the-file",
                WhyUnmeasured::TheRuntimeRefusedTheFile,
            ),
            ("weights-not-published", WhyUnmeasured::WeightsNotPublished),
        ] {
            let read: Unmeasured = toml::from_str(&written(spelled)).unwrap();
            assert_eq!(read.because, why);
        }
        assert!(toml::from_str::<Unmeasured>(&written("it-felt-slow")).is_err());
    }
}
