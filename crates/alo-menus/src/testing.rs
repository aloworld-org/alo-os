//! The strings this crate's own tests are written against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::Strings;

use crate::words::menu_words;

/// This crate's own words, with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(menu_words().unwrap())
}
