//! The strings the record's own tests are written against.
//!
//! A refusal reaches the record as the value it was, and [`crate::Entry::refused`]
//! asks it for words. What it asks with, here, is the vocabulary the refusal
//! came from — `alo-capability`'s own list — because that is the arrangement on
//! a real machine: one vocabulary, every crate's strings in it, one rendering
//! shown to the person and written down.
//!
//! A file of its own rather than more of [`crate::test_calls`], which is the
//! afternoon those tests are about: calls, grants and departures. What the
//! machine can say changes for a different reason.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::Strings;

/// Every string the capability model can say, with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(alo_capability::capability_words().unwrap())
}
