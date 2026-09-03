//! The places, the failures and the vocabularies this crate's tests are
//! written against.
//!
//! Every file here has the same three things to build before it can say
//! anything — somewhere a question was put, something that went wrong there,
//! and a vocabulary — and building them from one fixture is what stops five
//! files inventing five machines that resemble each other.
//!
//! The vocabulary is [`crate::answering_words`] **beside `alo-models`'**,
//! which is the arrangement a shell has: one vocabulary, one area per crate.
//! Both, because every sentence this crate says has a clause `alo-models` words
//! inside it, and a fixture holding only half of that would make every test
//! here pass against a machine no shell ever builds.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_models::{InferenceSource, Region, SourcePolicy};
use alo_strings::{Language, Strings, Translation, Vocabulary, Word};

use crate::answering::Answering;
use crate::failed::Failed;
use crate::offer::Offer;
use crate::words::answering_words;
use crate::wrong::WentWrong;

/// The machine the person is sitting at.
pub(crate) fn here() -> InferenceSource {
    InferenceSource::ThisMachine
}

/// The machine down the corridor, paired deliberately (ADR 0003).
pub(crate) fn paired() -> InferenceSource {
    InferenceSource::PairedMachine {
        machine: "the studio workstation".to_owned(),
    }
}

/// A provider that has said where it runs.
pub(crate) fn hosted() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "alo".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// A provider that has not — the one ADR 0008 says must never be made to look
/// like one that has.
pub(crate) fn somewhere() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "someone".to_owned(),
        region: Region::Unknown,
    }
}

/// A question put here, which went wrong this way, on a machine that also has
/// these places and forbids none of them.
pub(crate) fn failing(
    source: InferenceSource,
    why: WentWrong,
    others: &[InferenceSource],
) -> Failed {
    Answering::chosen(source, &SourcePolicy::Anywhere)
        .unwrap()
        .did_not_answer(why, others, &SourcePolicy::Anywhere)
        .unwrap()
}

/// An offer made by a different failure entirely, over a place no other fixture
/// here offers.
///
/// What a person answering a dialogue that has been open too long hands back,
/// and the only thing [`crate::Failed::take`] refuses.
pub(crate) fn an_offer_from_another_failure() -> Offer {
    failing(here(), WentWrong::NothingAnswered, &[somewhere()])
        .elsewhere()
        .offers()
        .first()
        .cloned()
        .unwrap()
}

/// This crate's words beside `alo-models`', with nothing translated: what a
/// machine that has no translations shows.
pub(crate) fn in_english() -> Strings {
    Strings::of(everything())
}

/// The same, with these words translated into German and German preferred.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = everything();
    let mut into_german = Translation::into_language(german());
    for (word, says) in words {
        into_german = into_german.says(word.key(), *says);
    }
    let speaking = vocabulary.check(into_german).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german()]);
    strings
}

/// German, as `alo-strings` names a language.
fn german() -> Language {
    Language::written("de").unwrap()
}

/// This crate's vocabulary and `alo-models`', in one.
fn everything() -> Vocabulary {
    let mut vocabulary = answering_words().unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    vocabulary
}
