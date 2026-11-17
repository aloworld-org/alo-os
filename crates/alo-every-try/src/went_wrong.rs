//! **Where a try left the road**, in words a summary can show as they are.

/// How a try left the road it was expected to take. Steps are counted from 1.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WentWrong {
    /// It asked for another verb.
    #[error("at step {step} it asked for {asked} where {expected} was expected")]
    AnotherVerb {
        /// Which step.
        step: usize,
        /// The verb the road expected.
        expected: String,
        /// What it asked for instead.
        asked: String,
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
    /// The machine refused the step before asking anybody, as a booted machine
    /// would: an application to install it could not name as one the catalogue
    /// holds (`alo_turn::as_the_catalogue_names_it`). The model is shown the
    /// refusal and the try goes on from it, so this ends a try only where the
    /// model never corrects itself.
    #[error("at step {step} the machine refused {verb} before asking anybody: {}", .said.text())]
    RefusedBeforeAsking {
        /// Which step.
        step: usize,
        /// The verb.
        verb: String,
        /// The machine's refusal, as the model is shown it.
        said: alo_strings::Said,
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
