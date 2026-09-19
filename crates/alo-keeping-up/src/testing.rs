//! The strings and builds this crate's unit tests are written against.
//!
//! Built from [`crate::keeping_up_words`], the real list, so no test here
//! invents a vocabulary that merely resembles it. Nothing here is compiled
//! into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a test fixture, a panic on a None or an Err is the failure being reported"
)]

use alo_strings::Strings;

use crate::digest::Digest;
use crate::standing::Ready;
use crate::vouching::Vouching;
use crate::words::keeping_up_words;

/// This crate's own words, with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(keeping_up_words().unwrap())
}

/// A whole digest, from one repeated pair.
pub(crate) fn whole(pair: &str) -> Digest {
    Digest::read(&format!("sha256:{}", pair.repeat(32))).unwrap()
}

/// An update ready from one build to another.
///
/// The crate makes a [`Ready`] only from an offer heard during a check on the
/// indicator, and an indicator needs a moment this crate's source never names;
/// that road is walked in `tests/what_an_update_may_never_do.rs`.
pub(crate) fn ready_between(running: &str, offered: &str) -> Ready {
    Ready::for_a_test(
        whole(running),
        whole(offered),
        Vouching::ThePlaceVouchesForIt,
    )
}

/// The same, for a build nobody has vouched for.
pub(crate) fn ready_between_unvouched(running: &str, offered: &str) -> Ready {
    Ready::for_a_test(
        whole(running),
        whole(offered),
        Vouching::NobodyHasVouchedForIt,
    )
}
