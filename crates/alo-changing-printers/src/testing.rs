//! The strings this crate's own tests are written against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//! The vocabulary is this crate's and the two it puts words beside — the
//! capability model's, which declares a verb, and the printing crate's, whose
//! sentence says a printer was set up.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{Strings, Vocabulary};

use crate::words::declare_into;

/// Nothing translated: what a machine with no translations shows.
pub(crate) fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_printing::declare_into(&mut vocabulary).unwrap();
    declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}
