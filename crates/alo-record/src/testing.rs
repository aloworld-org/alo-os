//! The strings the record's own tests are written against.
//!
//! A refusal reaches the record as the value it was, and
//! [`crate::Entry::refused`] asks it for words. What it asks with, here, is the
//! vocabulary the refusals came from — `alo-capability`'s list and
//! `alo-egress`' — because that is the arrangement on a real machine: one
//! vocabulary, every crate's strings in it, one rendering shown to the person
//! and written down.
//!
//! Two crates rather than one since item 9h, and the second is the reason this
//! fixture is worth having: [`crate::Entry::held_back`] renders a refusal the
//! egress policy made, and a record that rendered it against a vocabulary
//! missing those words would keep a key where the person read a sentence.
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

/// Every string the two crates a record renders from can say, with nothing
/// translated.
pub(crate) fn in_english() -> Strings {
    let mut vocabulary = alo_capability::capability_words().unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}
