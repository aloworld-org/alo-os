//! What a call was, as the record keeps it.
//!
//! ADR 0001 §7 asks the record for four answers, and this file is the first of
//! them: **what ran.** A verb name on its own does not answer it — `move_file`
//! is not evidence of anything — so an entry keeps the arguments the call was
//! made with and the sentence that was generated from them.
//!
//! The sentence is the important one. It is the words a person read and
//! approved, filled from the validated arguments and from nothing the model
//! wrote, so a record of it is a record of what somebody agreed to rather than
//! a description composed afterwards. For a read, which nobody is asked about,
//! it is still the sentence that *would* have been shown, which is the same
//! thing said in the same words.
//!
//! **The strings are passed in rather than the words** (item 9g), as they have
//! been for a refusal since 9e and for a held-back egress since 9h. A [`Call`]
//! carries what names its sentence and the values that fill it; the record
//! renders it here, with the vocabulary the person in front of the machine
//! reads, so what is written down is what they were shown. A record that kept
//! the source language would be a second account of the same moment, and a
//! security review reading it would be reading a sentence nobody saw.
//!
//! Built from a [`Call`] and never turned back into one. There is no path from
//! this type to anything that runs.

use std::collections::BTreeMap;

use alo_capability::{Call, Effect};
use alo_strings::Strings;
use serde::{Deserialize, Serialize};

use crate::line::Line;
use crate::written::Written;

/// What a call was: the verb, what it does, its arguments, and its sentence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct What {
    /// The verb, by name.
    verb: Line,
    /// Whether it answered a question or changed something.
    effect: Effect,
    /// The sentence generated from the arguments below.
    sentence: Line,
    /// Every argument, by name, sorted — so one call reads the same way
    /// however its arguments arrived.
    arguments: BTreeMap<String, Written>,
}

impl What {
    /// Write down a call, in the words the person was shown.
    #[must_use]
    pub fn of(call: &Call, strings: &Strings) -> Self {
        Self {
            verb: Line::of(call.verb()),
            effect: call.effect(),
            sentence: Line::of(call.sentence(strings).text()),
            arguments: call
                .values()
                .iter()
                .map(|(name, value)| (name.clone(), Written::from(value)))
                .collect(),
        }
    }

    /// The verb, by name.
    #[must_use]
    pub fn verb(&self) -> &Line {
        &self.verb
    }

    /// Whether it answered a question or changed something.
    #[must_use]
    pub fn effect(&self) -> Effect {
        self.effect
    }

    /// The sentence a person read — or would have read, for a read nobody was
    /// asked about.
    #[must_use]
    pub fn sentence(&self) -> &Line {
        &self.sentence
    }

    /// Every argument it was called with, by name.
    #[must_use]
    pub fn arguments(&self) -> &BTreeMap<String, Written> {
        &self.arguments
    }

    /// One argument, by name.
    #[must_use]
    pub fn argument(&self, name: &str) -> Option<&Written> {
        self.arguments.get(name)
    }

    /// Whether this call touched that path.
    ///
    /// The question a security review asks about one file, answered from the
    /// arguments that were recorded rather than by reading the sentence.
    #[must_use]
    pub fn touched(&self, path: &str) -> bool {
        self.arguments
            .values()
            .filter_map(Written::as_path)
            .any(|touched| touched.as_os_str() == path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_calls::{MOVING_SENTENCE, archiving_march, listing_invoices};
    use crate::testing::{in_english, translated};

    /// What ran is the sentence somebody approved, plus the arguments it was
    /// generated from — so the record says what happened in the words the
    /// person read rather than in a description written afterwards.
    #[test]
    fn what_ran_is_kept_in_the_words_a_person_read() {
        let what = What::of(&archiving_march(), &in_english());
        assert!(what.verb().is("move_file"));
        assert_eq!(what.effect(), Effect::Change);
        assert!(
            what.sentence()
                .is("move /home/anna/Invoices/march.pdf into /home/anna/Archive")
        );
        assert_eq!(what.arguments().len(), 2);
        assert_eq!(
            what.argument("file").map(Written::describe),
            Some(Line::of("/home/anna/Invoices/march.pdf"))
        );
        assert!(what.argument("overwrite").is_none());
    }

    /// **What is written down is the sentence the person read** — item 9g, from
    /// the record's side.
    ///
    /// A [`Call`] carries what names its sentence and the values that fill it,
    /// and the words are asked for here, with the vocabulary in front of the
    /// machine. So somebody who approved a German sentence has a German
    /// sentence in their record, rather than a second rendering in whichever
    /// language the verb happened to be declared in.
    #[test]
    fn what_is_written_down_is_the_sentence_the_person_read() {
        let strings = translated(&[(MOVING_SENTENCE, "{file} nach {into} verschieben")]);
        let what = What::of(&archiving_march(), &strings);
        assert!(
            what.sentence()
                .is("/home/anna/Invoices/march.pdf nach /home/anna/Archive verschieben"),
            "{}",
            what.sentence()
        );
    }

    /// A read is recorded with the sentence it would have been described by,
    /// because "the agent read this folder" is as much a thing that happened as
    /// a change is.
    #[test]
    fn a_read_is_recorded_like_anything_else() {
        let what = What::of(&listing_invoices(), &in_english());
        assert_eq!(what.effect(), Effect::Read);
        assert!(what.sentence().is("list what is in /home/anna/Invoices"));
    }

    /// Which files were touched is answered from the arguments, not by reading
    /// the sentence back — a sentence is prose, and prose is not a query.
    #[test]
    fn what_a_call_touched_is_a_question_the_record_answers() {
        let what = What::of(&archiving_march(), &in_english());
        assert!(what.touched("/home/anna/Invoices/march.pdf"));
        assert!(what.touched("/home/anna/Archive"));
        assert!(!what.touched("/home/anna/Invoices/april.pdf"));
        assert!(!what.touched("/home/anna"));
    }

    /// The record outlives the session that wrote it, so what ran has to
    /// survive being written down and read back — arguments and all.
    #[test]
    fn what_ran_survives_being_written_down_and_read_back() {
        let what = What::of(&archiving_march(), &in_english());
        let written = serde_json::to_string(&what).unwrap_or_default();
        assert!(written.contains("move_file"), "{written}");
        assert_eq!(serde_json::from_str::<What>(&written).ok(), Some(what));
    }
}
