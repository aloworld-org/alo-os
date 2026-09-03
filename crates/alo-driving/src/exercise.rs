//! One thing a model is asked to do, and the prompt that asks it.
//!
//! An exercise is a request in words and the verb a correct answer calls. It is
//! deliberately not a whole expected call: what the bar measures is whether a
//! model reaches the right verb **with arguments this machine would act on**,
//! and scoring the argument *values* would be measuring how closely it copied a
//! sentence rather than whether it can produce a call at all.
//!
//! That is weaker than it sounds, because [`alo_capability::Takes`] is not
//! weak. A path must be a full path with no `..` and no control characters in
//! it, a name is one name and never a journey, a count is inside the range the
//! verb declared, and a choice is one of the options the verb wrote down. A
//! model that answers `folder: "the invoices folder"` fails, because that is
//! not a path — so the structural gate carries real weight without anybody
//! having to write down a right answer.
//!
//! # The prompt is built from the verbs' own words
//!
//! What a model is told each verb does is
//! [`alo_capability::Verb::purpose_as_written`] and
//! [`alo_capability::Arg::purpose_as_written`] — the sentence a translator is
//! handed, not a second description written for a test. Item 9b's rule, one
//! crate on: two descriptions of one verb are two things that can disagree, and
//! the one the measurement used would be the one nobody maintains.
//!
//! # It is in English, and that is a limit rather than a decision
//!
//! The prompt is not a string a person reads, so it is not an
//! `alo_strings::Word` and this crate declares no vocabulary. What follows is
//! that the grade says how a model drives the verbs **when it is asked in
//! English**, and a model asked in Latvian may do worse. `docs/quirks.md`
//! records it. Measuring in twenty-four languages is a real question and it is
//! not this one.

use alo_capability::{Effect, Takes, Verb, Verbs};

/// One request put to a model, and the verb a correct answer calls.
///
/// Both halves are `&'static str`, so the set is written in the source and
/// cannot arrive from anywhere: an exercise read from a file would be a
/// measurement whose questions somebody could choose after seeing the answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exercise {
    /// A short stable name, so a report can say which one failed.
    named: &'static str,
    /// What the model is asked to do, in words.
    asked: &'static str,
    /// The verb a correct answer calls.
    verb: &'static str,
}

impl Exercise {
    /// Declare one.
    #[must_use]
    pub const fn asking(named: &'static str, asked: &'static str, verb: &'static str) -> Self {
        Self { named, asked, verb }
    }

    /// The short stable name of this exercise.
    #[must_use]
    pub fn named(&self) -> &'static str {
        self.named
    }

    /// What the model is asked to do.
    #[must_use]
    pub fn asked(&self) -> &'static str {
        self.asked
    }

    /// The verb a correct answer calls.
    #[must_use]
    pub fn verb(&self) -> &'static str {
        self.verb
    }
}

/// What every model is told before it is asked anything.
///
/// It describes the message `alo_protocol::FromAnAgent` reads and nothing else.
/// The two keys and the two doors are the whole of the envelope; a model that
/// cannot reproduce those cannot reproduce a verb call either.
pub const HOW_TO_ANSWER: &str = "\
You are talking to a computer, not to a person. Answer with one line of JSON and
nothing else: no explanation, no code fence, no second line.

The line has this shape:

{\"format\":1,\"asks\":{\"read\":{\"verb\":\"NAME\",\"given\":[{\"named\":\"ARGUMENT\",\"is\":VALUE}]}}}

Use \"read\" for a verb that only answers a question, and \"propose\" for a verb
that changes something; each verb below says which it is. VALUE is text in
quotes, or a whole number with no quotes. Give every argument the verb takes and
no others. A path is always a full path.

These are the only verbs there are:";

/// The whole prompt for one exercise: how to answer, the verbs, and the
/// request.
///
/// The verbs come from the registry the measurement is being run against, so a
/// model is asked about the verbs the machine really has. Every model faces the
/// same text for the same registry, which is what makes two grades comparable
/// at all.
#[must_use]
pub fn prompt(exercise: &Exercise, verbs: &Verbs) -> String {
    let mut text = String::from(HOW_TO_ANSWER);
    for verb in verbs.all() {
        text.push_str(&describe(verb));
    }
    text.push_str("\n\nThe request: ");
    text.push_str(exercise.asked());
    text
}

/// One verb, as a model is told about it.
fn describe(verb: &Verb) -> String {
    let door = match verb.effect() {
        Effect::Read => "read",
        Effect::Change => "propose",
    };
    let mut text = format!(
        "\n\n- {} ({door}) — {}",
        verb.name(),
        verb.purpose_as_written()
    );
    for arg in verb.args() {
        text.push_str(&format!(
            "\n  - {} ({}) — {}",
            arg.name(),
            takes(arg.takes()),
            arg.purpose_as_written()
        ));
    }
    text
}

/// What one argument accepts, in a clause.
fn takes(takes: &Takes) -> String {
    match takes {
        Takes::Path => "a full path".to_owned(),
        Takes::Application => "an installed application's identifier".to_owned(),
        Takes::Name { longest } => format!("one name, at most {longest} characters"),
        Takes::Count { least, most } => format!("a whole number from {least} to {most}"),
        Takes::Choice(options) => {
            let names: Vec<&str> = options.iter().map(alo_capability::Offered::name).collect();
            format!("one of: {}", names.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::the_verbs;

    /// **A model is told what a verb is for in the verb's own words.** Two
    /// descriptions of one verb are two things that can disagree, and the one
    /// the measurement used would be the one nobody maintains.
    #[test]
    fn the_prompt_describes_each_verb_in_the_words_the_verb_declared() {
        let verbs = the_verbs();
        let text = prompt(
            &Exercise::asking("a", "do something", "list_folder"),
            &verbs,
        );
        for verb in verbs.all() {
            assert!(text.contains(verb.name()), "{}", verb.name());
            assert!(text.contains(verb.purpose_as_written()), "{}", verb.name());
            for arg in verb.args() {
                assert!(text.contains(arg.purpose_as_written()), "{}", arg.name());
            }
        }
    }

    /// Every kind of argument is described, including the two that carry a
    /// bound: a model told "a whole number" and refused for sending 5000 has
    /// been measured on something nobody asked it.
    #[test]
    fn every_bound_a_verb_declared_is_in_the_prompt() {
        let text = prompt(
            &Exercise::asking("a", "do something", "list_folder"),
            &the_verbs(),
        );
        assert!(text.contains("a full path"), "{text}");
        assert!(text.contains("at most 255 characters"), "{text}");
        assert!(text.contains("a whole number from 1 to 1000"), "{text}");
        assert!(
            text.contains("one of: left_half, right_half, whole_screen"),
            "{text}"
        );
        assert!(
            text.contains("an installed application's identifier"),
            "{text}"
        );
    }

    /// The two doors are named, because sending a change through the read door
    /// is one of the ways a model fails and it must not fail for want of being
    /// told which is which.
    #[test]
    fn the_prompt_says_which_door_each_verb_takes() {
        let text = prompt(
            &Exercise::asking("a", "do something", "list_folder"),
            &the_verbs(),
        );
        assert!(text.contains("list_folder (read)"), "{text}");
        assert!(text.contains("move_file (propose)"), "{text}");
    }

    /// The request is the last thing in the prompt, so nothing after it can be
    /// mistaken for part of it.
    #[test]
    fn the_request_comes_last() {
        let text = prompt(
            &Exercise::asking("a", "list what is in /home/anna", "list_folder"),
            &the_verbs(),
        );
        assert!(
            text.ends_with("The request: list what is in /home/anna"),
            "{text}"
        );
    }
}
