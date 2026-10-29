//! **How each model does on a follow-up**, kept apart from every other grade.
//!
//! [ADR 0138](../../../docs/decisions/0138-a-follow-up-is-sent-with-the-conversation-so-far.md):
//! a reply typed in a task window reaches a model with the conversation so far,
//! and a grade earned on a first request says nothing about whether a model can
//! read one. The owner chose to measure follow-ups **separately for each model**
//! and to keep that result apart, so the grades in `data/catalogue.toml` — the
//! ones that decide whether a model may be given the agent — stay exactly what
//! they were and are read exactly as they were.
//!
//! # Its own file, and why
//!
//! `data/follow-ups.toml`, beside the catalogue rather than inside it, for two
//! reasons. A follow-up grade **decides nothing** — not
//! `Model::can_be_the_agent`, not a recommendation — and a field on the entry a
//! machine reads to decide would invite somebody to read it there. And a model
//! at a provider has no catalogue entry at all, yet it is the one model this
//! machine could measure a follow-up on first.
//!
//! # Per model and per digest, as a first-request grade is
//!
//! Each measurement names the model, where it was asked, the instructions
//! digest it was shown and the counts behind its grade, in a
//! [`MeasuredOn`] exactly like a catalogue grade's. A model with no measurement
//! under the words a turn shows today is **not yet measured**, and that is what
//! [`FollowUpsMeasured::for_the_turn`] says about it — never a grade borrowed
//! from a first request.

use serde::Deserialize;

use crate::driving::Driving;
use crate::graded_for_turns::THE_WORDS_A_TURN_SHOWS;
use crate::measured_on::MeasuredOn;

/// The file, as this build carries it.
const BUILT_IN: &str = include_str!("../data/follow-ups.toml");

/// One follow-up measurement.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AFollowUpGrade {
    /// The model, as whoever served it names it — a catalogue `id`, or a
    /// provider's own model name.
    pub model: String,
    /// Where it was asked: `this machine`, or the provider's name.
    pub at: String,
    /// The grade the counts earn.
    pub grade: Driving,
    /// Where, when and under which instructions it was earned, with the
    /// counts. The instructions are required here, because a follow-up grade
    /// with no digest says nothing about any turn.
    pub measured: MeasuredOn,
}

/// The file, read.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FollowUpsMeasured {
    /// Every follow-up measurement, in the order written.
    #[serde(default, rename = "follow_up")]
    pub each: Vec<AFollowUpGrade>,
}

/// Why the file could not be read.
///
/// English, and it keeps its `Display`, for `crate::CatalogueError`'s reason:
/// its reader is whoever edited the file.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FollowUpsError {
    /// Not the shape this reads.
    #[error("follow-ups.toml could not be read: {0}")]
    NotReadable(String),
    /// A measurement that names no instructions, or no counts.
    #[error(
        "the follow-up grade for {0} names no instructions digest or no counts, so it is about no \
         turn and cannot be re-derived"
    )]
    Unanchored(String),
    /// A grade written in a file of measurements that says it was not measured.
    #[error("the follow-up entry for {0} says not-measured; leave a model out instead")]
    NotAMeasurement(String),
}

impl FollowUpsMeasured {
    /// **The measurements this build carries.**
    ///
    /// # Errors
    /// [`FollowUpsError`], for a file this build should never have shipped.
    pub fn built_in() -> Result<Self, FollowUpsError> {
        Self::parse(BUILT_IN)
    }

    /// Read a file of follow-up measurements.
    ///
    /// # Errors
    /// [`FollowUpsError`]: the wrong shape, a measurement with no digest or
    /// no counts, or one that says it is not a measurement.
    pub fn parse(text: &str) -> Result<Self, FollowUpsError> {
        let read: Self =
            toml::from_str(text).map_err(|why| FollowUpsError::NotReadable(why.to_string()))?;
        for one in &read.each {
            let named = format!("{} at {}", one.model, one.at);
            if one.grade == Driving::NotMeasured {
                return Err(FollowUpsError::NotAMeasurement(named));
            }
            if one.measured.instructions.is_none()
                || one.measured.drove.is_none()
                || one.measured.of.is_none()
            {
                return Err(FollowUpsError::Unanchored(named));
            }
        }
        Ok(read)
    }

    /// **How this model does on a follow-up, under the words a turn shows
    /// today**, or [`Driving::NotMeasured`].
    ///
    /// A measurement under any other digest is history and is not read, for
    /// ADR 0133's reason: it is about words no turn shows now.
    #[must_use]
    pub fn for_the_turn(&self, model: &str, at: &str) -> Driving {
        self.each
            .iter()
            .find(|one| {
                one.model == model
                    && one.at == at
                    && one.measured.instructions.as_deref() == Some(THE_WORDS_A_TURN_SHOWS)
            })
            .map_or(Driving::NotMeasured, |one| one.grade)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "graded_on_follow_ups_tests.rs"]
mod tests;
