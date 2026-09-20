//! The strings this crate's own tests are written against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//! The vocabulary is this crate's, the capability model's, which declares a
//! verb, and `alo-proxy`'s — a person refused their own proxy on a machine an
//! organisation manages reads that crate's sentence rather than a second one
//! here (ADR 0060 §5), and a machine assembles every crate's words into one
//! vocabulary anyway (`alo-saying`).

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
    alo_proxy::declare_into(&mut vocabulary).unwrap();
    declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}
