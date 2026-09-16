//! The strings and the stand-ins this crate's own tests are written against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! [`NeverAsked`] is the evaluator a test hands over when the answer must not
//! depend on one — it panics if anything asks it, which is how *the script was
//! not consulted* is a fact rather than an assumption. [`Answering`] is the one
//! that answers, and writes down what it was asked, so a test can say what did
//! and did not reach a network's own script.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;

use alo_strings::{Language, Strings, Translation, Vocabulary};

use crate::automatic::{ConfigurationAddress, NotEvaluated, TheEvaluator};
use crate::words::{Word, declare_into};

/// Everything this crate says.
fn its_own() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// Nothing translated: what a machine with no translations shows.
pub(crate) fn in_english() -> Strings {
    Strings::of(its_own())
}

/// Some of these words translated into German, and German preferred.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = its_own();
    let german = Language::written("de").unwrap();
    let mut translation = Translation::into_language(german.clone());
    for (word, says) in words {
        translation = translation.says(word.key(), *says);
    }
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german]);
    strings
}

/// An evaluator nothing may ask.
///
/// Handed over wherever a test says *this answer did not come from a script*.
/// A decision that quietly consulted one would pass against a permissive
/// stand-in and fail nowhere.
pub(crate) struct NeverAsked;

impl TheEvaluator for NeverAsked {
    fn asked(
        &self,
        _: &ConfigurationAddress,
        address: &str,
        _: &str,
    ) -> Result<String, NotEvaluated> {
        panic!("nothing should have been asked about {address}")
    }
}

/// An evaluator that answers, and remembers what it was asked.
pub(crate) struct Answering {
    /// What it answers, or why it will not.
    answer: Result<String, NotEvaluated>,
    /// Every question it was asked: where the configuration is, the address,
    /// and the host.
    asked: RefCell<Vec<(String, String, String)>>,
}

impl Answering {
    /// An evaluator that answers this.
    pub(crate) fn saying(answer: &str) -> Self {
        Self {
            answer: Ok(answer.to_owned()),
            asked: RefCell::new(Vec::new()),
        }
    }

    /// An evaluator that will not answer, for this reason.
    pub(crate) fn refusing(why: NotEvaluated) -> Self {
        Self {
            answer: Err(why),
            asked: RefCell::new(Vec::new()),
        }
    }

    /// Everything it was asked about.
    pub(crate) fn asked_about(&self) -> Vec<(String, String, String)> {
        self.asked.borrow().clone()
    }
}

impl TheEvaluator for Answering {
    fn asked(
        &self,
        at: &ConfigurationAddress,
        address: &str,
        host: &str,
    ) -> Result<String, NotEvaluated> {
        self.asked
            .borrow_mut()
            .push((at.as_str().to_owned(), address.to_owned(), host.to_owned()));
        self.answer.clone()
    }
}
