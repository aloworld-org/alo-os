//! The invocation, the moment and the strings this crate's tests are written
//! against.
//!
//! Every file here has the same things to build before it can say anything — a
//! moment, something a person had on their screen, and a vocabulary — and
//! building them from one fixture is what stops five files inventing five
//! desktops that resemble each other. The vocabulary is
//! [`crate::context_words`] beside `alo-capability`'s, which is the arrangement
//! a shell has: one vocabulary, one area per crate.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;
use std::time::{Duration, SystemTime};

use alo_strings::{Form, Language, Strings, Translation, Vocabulary, Word};

use crate::context::Context;
use crate::document::Document;
use crate::focused::Focused;
use crate::selection::Selection;
use crate::words::{SELECTION_SHORTENED, context_words};

/// A fixed moment, so that a turn ending is arithmetic rather than a wait.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the turns and the grants in these tests last.
pub(crate) fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The document most of these tests are about.
pub(crate) fn march() -> &'static Path {
    Path::new("/home/anna/Invoices/march.pdf")
}

/// The window most of these tests have in front of the person.
pub(crate) fn blender() -> Focused {
    Focused::titled("org.blender.Blender", "untitled.blend").unwrap()
}

/// An invocation with all three parts of a context in it, which is the case
/// most worth being careful about: everything offered, one thing granted.
pub(crate) fn everything_offered() -> Context {
    Context::at_invocation(noon())
        .and_document(Document::open(march()).unwrap())
        .and_selection(Selection::of("the invoice from March").unwrap())
        .and_window(blender())
}

/// This crate's words beside the capability model's, with nothing translated:
/// what a machine that has no translations shows.
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

/// The countable string translated into Polish, one form at a time.
///
/// Polish rather than German because it has three forms where English has two,
/// so a test written against it fails if anything ever decides the reader's
/// language counts the way the source does.
pub(crate) fn translated_counting(forms: &[(Form, &str)]) -> Strings {
    let vocabulary = everything();
    let mut into_polish = Translation::into_language(polish());
    for (form, says) in forms {
        into_polish = into_polish.says(SELECTION_SHORTENED.key().for_form(*form), *says);
    }
    let speaking = vocabulary.check(into_polish).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[polish()]);
    strings
}

/// German, as `alo-strings` names a language.
fn german() -> Language {
    Language::written("de").unwrap()
}

/// Polish, which counts in three forms.
fn polish() -> Language {
    Language::written("pl").unwrap()
}

/// This crate's vocabulary and the capability model's, in one.
///
/// Both, because a turn that cannot be begun is refused in the grants' own
/// words rather than in this crate's — item 9f's rule, and the reason
/// `GrantError` travels out of [`crate::Turn::beginning`] whole.
fn everything() -> Vocabulary {
    let mut vocabulary = context_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    vocabulary
}
