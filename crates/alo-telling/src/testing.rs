//! The places, the failures and the vocabulary this crate's tests are written
//! against.
//!
//! Every file here has the same three things to build before it can say
//! anything — somewhere a question was put, something that went wrong there,
//! and a vocabulary — and building them from one fixture is what stops four
//! files inventing four machines that resemble each other. The shape is
//! `alo-answering`'s own `testing.rs`, copied rather than re-decided, because
//! the fixtures a telling needs are the fixtures a failure needs.
//!
//! The vocabulary is [`crate::telling_words`] **beside `alo-answering`'s and
//! `alo-models`'**, which is the arrangement a shell has: one vocabulary, one
//! area per crate. All three, because a telling is four lines and only two of
//! them are this crate's — a fixture holding half of it would make every test
//! here pass against a machine no shell ever builds.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_answering::{Answering, Failed, WentWrong};
use alo_models::{InferenceSource, Region, SourcePolicy};
use alo_strings::{Strings, Vocabulary};

use alo_models::Weights;

use crate::telling::Telling;
use crate::told_once::ToldOnce;
use crate::warned_once::WarnedOnce;
use crate::warning::Warning;
use crate::who_asked::WhoAsked;
use crate::words::telling_words;

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

/// A provider that has not — a second place, for the tests that need one that
/// is nobody else's.
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

/// One telling, about a failure nobody has been told about before.
///
/// Goes through [`Telling::about`] rather than around it, because there is no
/// way around it — which is the point of the crate and is therefore also the
/// only honest way for its own fixtures to build one.
pub(crate) fn a_telling_about(
    source: InferenceSource,
    why: WentWrong,
    others: &[InferenceSource],
) -> ToldOnce {
    let mut telling = Telling::nothing_said_yet();
    match telling.about(failing(source, why, others), WhoAsked::ThePerson) {
        crate::telling::Tell::Say(told) => told,
        crate::telling::Tell::SaidAlready => {
            unreachable!("a fresh session has said nothing to anybody")
        }
    }
}

/// Weights somebody brought, of this size, measured by nobody.
pub(crate) fn weights_of(id: &str, bytes_on_disk: u64) -> Weights {
    Weights::checked(id, bytes_on_disk).unwrap()
}

/// One warning, about weights nobody has been warned about before, on a
/// machine with this much memory.
///
/// Goes through [`Warning::about`] rather than around it, because there is no
/// way around it.
pub(crate) fn a_warning_about(id: &str, bytes_on_disk: u64, machine_gb: f32) -> WarnedOnce {
    let mut warning = Warning::nothing_said_yet();
    match warning.about(
        &weights_of(id, bytes_on_disk),
        machine_gb,
        WhoAsked::ThePerson,
    ) {
        crate::warning::Warn::Say(warned) => warned,
        crate::warning::Warn::SaidAlready | crate::warning::Warn::Fits => {
            unreachable!("a fresh session has warned nobody, and these weights do not fit")
        }
    }
}

/// This crate's words beside `alo-answering`'s and `alo-models`', with nothing
/// translated: what a machine that has no translations shows.
pub(crate) fn in_english() -> Strings {
    Strings::of(everything())
}

/// The three vocabularies a telling is read out of, in one.
fn everything() -> Vocabulary {
    let mut vocabulary = telling_words().unwrap();
    alo_answering::declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    vocabulary
}
