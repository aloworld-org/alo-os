//! The sentence a person approves, and where its words come from.
//!
//! ADR 0001 names the residual risk of the whole capability model: if the
//! sentence describing a change is vague, the approval is uninformed, and the
//! model is the last thing that should be choosing those words. So it does not.
//! A [`Sentence`] is written once, by whoever declares the verb, with holes in
//! it where the arguments go; the holes are filled from [`Value`]s that have
//! already been validated, and nothing else is ever inserted.
//!
//! ```text
//! move {file} into {folder}
//! ```
//!
//! What a person approves is the rendered result. Two rules follow, and both
//! are enforced in [`crate::verb`] rather than here, because they are facts
//! about a verb rather than about a template: the sentence names **every**
//! argument the verb declares, and it names nothing else. An argument missing
//! from the sentence is an argument the person did not agree to.
//!
//! The words are the source, in the sense `alo-strings` means it: the sentence
//! somebody translates rather than the sentence everybody is shown. A sentence
//! is a list of parts rather than a format string precisely so that translating
//! it moves [`Part::Words`] and nothing else.
//!
//! **A rendered sentence is still the source language, and that is the one
//! thing item 9e did not move.** [`crate::Call`] renders its sentence when the
//! call is made and keeps the result, so a shell shows a translated sentence
//! (`alo-files` looks the key up) while the approval and the record keep the
//! rendering made here. Making those one thing means a `Call` carrying a key
//! and a filling rather than a string, which changes what a record *is* — it is
//! item 9f in `docs/autonomy/QUEUE.md`, and it is not a rename.

use std::collections::BTreeMap;

use crate::arg::Value;

/// One piece of a sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    /// Words that are always the same, whatever the arguments are.
    Words(String),
    /// The place an argument's value goes, by the argument's name.
    Argument(String),
}

/// How a verb's approval sentence is generated from its validated arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    /// The parts, in the order they are read.
    parts: Vec<Part>,
}

/// Why a sentence could not be written, or could not be filled in.
///
/// **This one keeps its English and its `Display`.** It is not read by whoever
/// is using the machine: every variant is a refusal of a *template*, so the
/// reader is whoever wrote the verb, at the moment their declaration fails its
/// own tests. It is `alo-shortcuts`' `DefaultsError` in another crate — a
/// sentence in whichever language happened to be loaded is not what that person
/// needs. [`crate::CallError::Unsayable`] is where a person hears about it
/// instead, in words they can act on.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum SentenceError {
    /// No template at all.
    #[error("write the sentence the person will approve — it is the thing they agree to")]
    Empty,
    /// A template that is only its arguments.
    #[error("say what happens in words — a sentence that is only its arguments describes nothing")]
    NoWords,
    /// A `{` with no `}` after it.
    #[error("close every {{argument}} with a brace, or it will appear in the sentence as written")]
    Unclosed,
    /// Braces with nothing between them.
    #[error("name the argument inside the braces")]
    Unnamed,
    /// A name in the sentence that nothing gave a value for.
    #[error("the sentence names {argument}, and nothing gave it a value")]
    NoValueFor {
        /// The argument the sentence wanted.
        argument: String,
    },
}

impl Sentence {
    /// Read a template: words, with `{argument}` where a value goes.
    ///
    /// # Errors
    /// [`SentenceError`], saying what to fix in the template.
    pub fn parse(template: &str) -> Result<Self, SentenceError> {
        let mut parts = Vec::new();
        let mut words = String::new();
        let mut name = String::new();
        let mut inside = false;
        for character in template.chars() {
            match (inside, character) {
                (false, '{') => {
                    if !words.is_empty() {
                        parts.push(Part::Words(std::mem::take(&mut words)));
                    }
                    inside = true;
                }
                (false, other) => words.push(other),
                (true, '{') => return Err(SentenceError::Unclosed),
                (true, '}') => {
                    let named = name.trim().to_owned();
                    if named.is_empty() {
                        return Err(SentenceError::Unnamed);
                    }
                    parts.push(Part::Argument(named));
                    name.clear();
                    inside = false;
                }
                (true, other) => name.push(other),
            }
        }
        if inside {
            return Err(SentenceError::Unclosed);
        }
        if !words.is_empty() {
            parts.push(Part::Words(words));
        }
        if parts.is_empty() {
            return Err(SentenceError::Empty);
        }
        if !parts.iter().any(is_readable_words) {
            return Err(SentenceError::NoWords);
        }
        Ok(Self { parts })
    }

    /// The arguments this sentence names, in the order it names them.
    pub fn mentions(&self) -> impl Iterator<Item = &str> {
        self.parts.iter().filter_map(|part| match part {
            Part::Argument(name) => Some(name.as_str()),
            Part::Words(_) => None,
        })
    }

    /// The parts, for anything that has to render this itself — a shell that
    /// wants the arguments emphasised, or a translation that has to move them.
    #[must_use]
    pub fn parts(&self) -> &[Part] {
        &self.parts
    }

    /// Fill it in, from arguments that have already been validated.
    ///
    /// # Errors
    /// [`SentenceError::NoValueFor`] when the sentence names an argument that
    /// is not among the values. A verb that passed [`crate::verb::Verb::checked`]
    /// and a call that filled every argument cannot reach it — but this returns
    /// a result rather than assuming that, because the assumption would be the
    /// one place in the crate where a sentence could be built from a hole.
    pub fn render(&self, values: &BTreeMap<String, Value>) -> Result<String, SentenceError> {
        let mut sentence = String::new();
        for part in &self.parts {
            match part {
                Part::Words(words) => sentence.push_str(words),
                Part::Argument(name) => {
                    let value = values.get(name).ok_or_else(|| SentenceError::NoValueFor {
                        argument: name.clone(),
                    })?;
                    sentence.push_str(&value.describe());
                }
            }
        }
        Ok(sentence)
    }
}

/// Whether a part is words with something readable in them.
///
/// A template of `"{file} {folder}"` parses into two arguments and a space, and
/// a space is not a description of what is about to happen.
fn is_readable_words(part: &Part) -> bool {
    match part {
        Part::Words(words) => words.chars().any(|c| !c.is_whitespace()),
        Part::Argument(_) => false,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn values() -> BTreeMap<String, Value> {
        let mut values = BTreeMap::new();
        values.insert(
            "file".to_owned(),
            Value::Path(PathBuf::from("/home/anna/Invoices/march.pdf")),
        );
        values.insert(
            "folder".to_owned(),
            Value::Path(PathBuf::from("/home/anna/Archive")),
        );
        values
    }

    /// The whole point of the file: the words are the verb's, the values are
    /// the call's, and nothing the model wrote appears in either.
    #[test]
    fn a_sentence_is_filled_in_from_validated_values() {
        let sentence = Sentence::parse("move {file} into {folder}").unwrap();
        assert_eq!(
            sentence.render(&values()).unwrap(),
            "move /home/anna/Invoices/march.pdf into /home/anna/Archive"
        );
        assert_eq!(
            sentence.mentions().collect::<Vec<_>>(),
            vec!["file", "folder"]
        );
    }

    /// A template that would leave a brace in the sentence a person reads is
    /// refused where it is written, not where it is shown.
    #[test]
    fn a_template_that_cannot_be_read_is_refused() {
        assert_eq!(
            Sentence::parse("move {file into the archive").unwrap_err(),
            SentenceError::Unclosed
        );
        assert_eq!(
            Sentence::parse("move {file{folder}}").unwrap_err(),
            SentenceError::Unclosed
        );
        assert_eq!(
            Sentence::parse("move {} there").unwrap_err(),
            SentenceError::Unnamed
        );
        assert_eq!(Sentence::parse("").unwrap_err(), SentenceError::Empty);
    }

    /// A sentence that is only its arguments tells a person two paths and not
    /// what is about to happen to them.
    #[test]
    fn a_sentence_has_to_say_what_happens() {
        assert_eq!(
            Sentence::parse("{file} {folder}").unwrap_err(),
            SentenceError::NoWords
        );
        assert_eq!(
            Sentence::parse("{file}").unwrap_err(),
            SentenceError::NoWords
        );
        assert!(Sentence::parse("archive {file}").is_ok());
    }

    /// A hole with nothing to put in it does not quietly become an empty
    /// space in a sentence somebody is about to approve.
    #[test]
    fn a_sentence_is_never_rendered_with_a_hole_left_in_it() {
        let sentence = Sentence::parse("move {file} into {folder}").unwrap();
        let mut short = values();
        short.remove("folder");
        assert_eq!(
            sentence.render(&short).unwrap_err(),
            SentenceError::NoValueFor {
                argument: "folder".to_owned()
            }
        );
    }

    /// The parts are kept apart so that a shell can emphasise the arguments,
    /// and so that translating the words later moves nothing else.
    #[test]
    fn the_words_and_the_arguments_stay_apart() {
        let sentence = Sentence::parse("rename {file} to {name}").unwrap();
        assert_eq!(
            sentence.parts(),
            &[
                Part::Words("rename ".to_owned()),
                Part::Argument("file".to_owned()),
                Part::Words(" to ".to_owned()),
                Part::Argument("name".to_owned()),
            ]
        );
    }
}
