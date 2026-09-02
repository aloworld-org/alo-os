//! The sentence a person approves, and where its words come from.
//!
//! ADR 0001 names the residual risk of the whole capability model: if the
//! sentence describing a change is vague, the approval is uninformed, and the
//! model is the last thing that should be choosing those words. So it does not.
//! A [`Sentence`] is written once, by whoever declares the verb, with gaps in
//! it where the arguments go; the gaps are filled from [`crate::Value`]s that
//! have already been validated, and nothing else is ever inserted.
//!
//! ```text
//! move {file} into {into}
//! ```
//!
//! What a person approves is the filled-in result. Two rules follow, and both
//! are enforced in [`crate::verb`] rather than here, because they are facts
//! about a verb rather than about a template: the sentence names **every**
//! argument the verb declares, and it names nothing else. An argument missing
//! from the sentence is an argument the person did not agree to.
//!
//! # One string, not two
//!
//! A sentence is a [`alo_strings::Word`] — a key, the English beside it, and
//! the note a translator needs — and there is no other way to make one. That is
//! the whole of item 9g. Before it, a verb was declared with a template and a
//! *translation* of that template was declared separately, so what a shell
//! showed and what the approval and the record kept were two renderings that a
//! test had to hope were equal. Now the declaration and the translator's row
//! are the same `Word`, [`Sentence::key`] is what a [`crate::Call`] carries, and
//! the words are asked for wherever somebody reads them.
//!
//! **There is one parser, and it is `alo-strings`'.** This file used to have its
//! own, which was the same bug one level down: `Verb::checked` would check a
//! reading of the template that `alo_strings::Vocabulary` was not obliged to
//! agree with — the two disagreed about `{{`, so a sentence with a literal brace
//! in it was a verb this crate refused and a phrase that crate accepted. A
//! [`alo_strings::Template`] is now what both of them read.

use alo_strings::{Filling, Key, Template, TemplateError, Word};

/// How a verb's approval sentence is written, and what names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    /// What names this sentence in the vocabulary — what a [`crate::Call`]
    /// carries, and what a translator's file is sorted by.
    key: Key,
    /// The sentence in the language the code is written in, which is the one
    /// [`crate::verb::Verb::checked`] holds to the rules above and the one a
    /// translator is handed.
    source: Template,
}

/// Why a sentence could not be written.
///
/// **This one keeps its English and its `Display`.** It is not read by whoever
/// is using the machine: every variant is a refusal of a *declaration*, so the
/// reader is whoever wrote the verb, at the moment their declaration fails its
/// own tests. It is `alo-shortcuts`' `DefaultsError` in another crate — a
/// sentence in whichever language happened to be loaded is not what that person
/// needs.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum SentenceError {
    /// A template that is only its arguments.
    #[error("say what happens in words — a sentence that is only its arguments describes nothing")]
    NoWords,
    /// A sentence `alo-strings` will not hold: an unclosed gap, a gap with no
    /// name, nothing at all.
    #[error(transparent)]
    Unreadable(#[from] TemplateError),
}

impl Sentence {
    /// Read a verb's sentence out of the word that declares it.
    ///
    /// The `Word` is the one thing a translator is given, so a verb declared
    /// from one cannot be checked against a string somebody else will translate.
    ///
    /// # Errors
    /// [`SentenceError`], saying what to fix in the declaration.
    pub fn of(word: Word) -> Result<Self, SentenceError> {
        let source = Template::written(word.says())?;
        if !has_words(&source) {
            return Err(SentenceError::NoWords);
        }
        Ok(Self {
            key: word.key(),
            source,
        })
    }

    /// What names this sentence in the vocabulary.
    #[must_use]
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// The sentence in the language the code is written in, gaps and all.
    ///
    /// This is the source, in the sense `alo-strings` means it: the sentence
    /// somebody translates rather than the sentence everybody is shown. What a
    /// person reads is [`crate::Call::sentence`].
    #[must_use]
    pub fn as_written(&self) -> &str {
        self.source.as_written()
    }

    /// The arguments this sentence names, in the order it names them.
    pub fn mentions(&self) -> impl Iterator<Item = &str> {
        self.source.gaps().iter().map(String::as_str)
    }
}

/// Whether a template says anything at all once its gaps are taken out.
///
/// A template of `"{file} {into}"` is two arguments and a space, and a space is
/// not a description of what is about to happen.
fn has_words(source: &Template) -> bool {
    let mut empty = Filling::nothing();
    for gap in source.gaps() {
        empty = empty.and(gap.clone(), "");
    }
    source
        .fill(&empty)
        .text()
        .chars()
        .any(|character| !character.is_whitespace())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    const MOVING: Word = Word::saying("testing.sentence.move", "move {file} into {into}");

    /// A sentence knows what names it and what it names, and both come out of
    /// the one word a translator will be handed.
    #[test]
    fn a_sentence_is_the_word_that_declares_it() {
        let sentence = Sentence::of(MOVING).unwrap();
        assert_eq!(
            sentence.key(),
            &Key::named("testing.sentence.move").unwrap()
        );
        assert_eq!(sentence.as_written(), "move {file} into {into}");
        assert_eq!(
            sentence.mentions().collect::<Vec<_>>(),
            vec!["file", "into"]
        );
    }

    /// A template that would leave a brace in the sentence a person reads is
    /// refused where it is written, not where it is shown — and it is refused
    /// by the same reader that will refuse a translation of it.
    #[test]
    fn a_template_that_cannot_be_read_is_refused() {
        for written in ["move {file into the archive", "move {} there", ""] {
            assert!(
                matches!(
                    Sentence::of(Word::saying("testing.sentence.bad", written)),
                    Err(SentenceError::Unreadable(_))
                ),
                "{written:?}"
            );
        }
    }

    /// A sentence that is only its arguments tells a person two paths and not
    /// what is about to happen to them.
    #[test]
    fn a_sentence_has_to_say_what_happens() {
        for written in ["{file} {into}", "{file}", "   {file}  "] {
            assert_eq!(
                Sentence::of(Word::saying("testing.sentence.bare", written)),
                Err(SentenceError::NoWords),
                "{written:?}"
            );
        }
        assert!(Sentence::of(Word::saying("testing.sentence.ok", "archive {file}")).is_ok());
    }

    /// **The two readings of one sentence agree, because there is only one.**
    /// A literal brace is what the old parser in this file refused and
    /// `alo-strings` accepts, and a verb declared with one used to be a
    /// declaration this crate would not take and a phrase a translator would
    /// have been handed anyway.
    #[test]
    fn the_sentence_this_crate_checks_is_the_one_a_translator_is_handed() {
        let braced = Word::saying(
            "testing.sentence.braced",
            "rename {file} to {{draft}} {name}",
        );
        let sentence = Sentence::of(braced).unwrap();
        assert_eq!(
            sentence.mentions().collect::<Vec<_>>(),
            vec!["file", "name"]
        );
        assert_eq!(
            braced.phrase().unwrap().source().gaps(),
            sentence.mentions().collect::<Vec<_>>()
        );
    }
}
