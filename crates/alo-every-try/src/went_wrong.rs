//! **Where a try left the road**, in words a summary can show as they are.

/// How a try left the road it was expected to take. Steps are counted from 1.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WentWrong {
    /// It asked for another verb.
    ///
    /// **What it gave that verb is said beside it** (2026-11-17), so a summary
    /// shows the evidence with the verdict: asked to scroll to a heading, a
    /// model that chose `accessible.type_text` six times in ten left no trace
    /// of what it meant to type.
    #[error("at step {step} it asked for {asked} ({given}) where {expected} was expected")]
    AnotherVerb {
        /// Which step.
        step: usize,
        /// The verb the road expected.
        expected: String,
        /// What it asked for instead.
        asked: String,
        /// The arguments it gave, as `name = value`, cut short past
        /// [`THE_MOST_SAID`] characters.
        given: String,
    },
    /// It sent the verb through the other door: a change as a read, or a read
    /// as a change.
    #[error("at step {step} it sent {verb} as a {door}, which that verb is not")]
    TheWrongDoor {
        /// Which step.
        step: usize,
        /// The verb.
        verb: String,
        /// The door it used: `read` or `change`.
        door: &'static str,
    },
    /// The verb was right and the machine would refuse its arguments: one it
    /// does not take, one left out, one given twice, or a value of the wrong
    /// kind.
    #[error("at step {step} it gave {verb} arguments the machine refuses: {why}")]
    ArgumentsRefused {
        /// Which step.
        step: usize,
        /// The verb.
        verb: String,
        /// The machine's own sentence about it.
        why: String,
    },
    /// The arguments were accepted and one of them was not the value the road
    /// decides.
    #[error("at step {step} it gave {argument} as `{given}` where `{expected}` was expected")]
    AnotherValue {
        /// Which step.
        step: usize,
        /// The argument.
        argument: String,
        /// The value the road expected.
        expected: String,
        /// What it gave.
        given: String,
    },
    /// It ran a command where a verb was expected.
    #[error("at step {step} it ran the command `{command}` where {expected} was expected")]
    RanACommand {
        /// Which step.
        step: usize,
        /// The verb the road expected.
        expected: String,
        /// The command.
        command: String,
    },
    /// It answered the person before the road was finished.
    #[error("it answered in words after {done} of {of} steps: {said}")]
    StoppedShort {
        /// How many steps it had taken.
        done: usize,
        /// How many the road has.
        of: usize,
        /// The start of what it said.
        said: String,
    },
    /// It took the whole road and then asked for more instead of answering.
    #[error("after the whole road it asked for {asked} instead of answering")]
    WentOn {
        /// What it asked for.
        asked: String,
    },
    /// It used every round the turn allows.
    #[error("it used every round a turn allows, after {done} of {of} steps")]
    RanOutOfRounds {
        /// How many steps it had taken.
        done: usize,
        /// How many the road has.
        of: usize,
    },
    /// The turn ended some other way: refused, stopped, or waiting for the
    /// person, none of which a road at full access expects.
    #[error("the turn ended after {done} of {of} steps: {why}")]
    EndedOtherwise {
        /// How many steps it had taken.
        done: usize,
        /// How many the road has.
        of: usize,
        /// How it ended.
        why: String,
    },
}

/// The most characters of a model's arguments [`WentWrong::AnotherVerb`]
/// says, so a summary's line stays one line.
pub const THE_MOST_SAID: usize = 160;

/// A model's arguments as a summary says them: `name = value`, by commas, cut
/// short with `…` past [`THE_MOST_SAID`] characters.
#[must_use]
pub fn as_said(given: &[(&str, alo_capability::Given)]) -> String {
    let said = given
        .iter()
        .map(|(name, value)| format!("{name} = {value:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    if said.chars().count() > THE_MOST_SAID {
        let cut: String = said.chars().take(THE_MOST_SAID).collect();
        format!("{cut}…")
    } else {
        said
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_are_said_by_name_and_cut_short_past_the_most() {
        let short = as_said(&[("name", alo_capability::Given::text("Downloads"))]);
        assert!(
            short.starts_with("name = ") && short.contains("Downloads"),
            "{short}"
        );
        let long = "x".repeat(THE_MOST_SAID * 2);
        let said = as_said(&[("text", alo_capability::Given::text(&long))]);
        assert_eq!(said.chars().count(), THE_MOST_SAID + 1, "{said}");
        assert!(said.ends_with('…'));
        let around = as_said(&[("t", alo_capability::Given::text(""))])
            .chars()
            .count();
        let exactly = as_said(&[(
            "t",
            alo_capability::Given::text("y".repeat(THE_MOST_SAID - around)),
        )]);
        assert_eq!(exactly.chars().count(), THE_MOST_SAID, "{exactly}");
        assert!(!exactly.ends_with('…'), "{exactly}");
    }
}
