//! The strings this crate's unit tests are written against.
//!
//! Built from [`crate::keeping_up_words`], the real list, so no test here
//! invents a vocabulary that merely resembles it. Nothing here is compiled
//! into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a test fixture, a panic on a None or an Err is the failure being reported"
)]

use alo_strings::Strings;

use crate::words::keeping_up_words;

/// This crate's own words, with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(keeping_up_words().unwrap())
}
