//! How much longer a grant on the list has: a stretch of time, or nothing to
//! wait for.
//!
//! A row used to carry a `Duration` and nothing else, which was true while
//! every grant ended. ADR 0001 was amended on 2026-10-25 — a person may give
//! a folder until they take it back — and a duration has no honest value for
//! that grant. `Duration::MAX` is a number somebody will one day compare
//! against, and zero is the opposite fact.
//!
//! It is this crate's own type rather than `alo_capability::Until`, which
//! carries the **moment** a grant stops. A row deliberately does not: see
//! [`crate::Seen::lasts`] for why a list holds what is left rather than when
//! it ends.
//!
//! A file of its own because it is the one thing in this crate a surface must
//! branch on, and `seen.rs` already answers *what is one row*.

use std::time::Duration;

/// How much longer a grant has, at the moment the list was derived.
///
/// ```
/// use alo_granted::Lasts;
/// use std::time::Duration;
///
/// let hour = Lasts::For(Duration::from_secs(60 * 60));
/// assert!(!hour.is_until_removed());
/// assert_eq!(hour.left(), Some(Duration::from_secs(3_600)));
///
/// assert!(Lasts::UntilRemoved.is_until_removed());
/// assert_eq!(Lasts::UntilRemoved.left(), None);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lasts {
    /// This much longer, and then it stops. Never zero: a grant that has run
    /// out never becomes a row at all.
    For(Duration),
    /// Until the person takes it away, and not before.
    ///
    /// **This is the row that has to be found.** Every other length protects
    /// a person by running out; this one does not, so the whole of its
    /// protection is that it is on the list they revoke from.
    UntilRemoved,
}

impl Lasts {
    /// How much is left, or [`None`] for a grant with no end.
    ///
    /// For a surface that shows a countdown. **It is not how to tell the two
    /// apart** — [`Self::is_until_removed`] is, and a surface that reads this
    /// `None` as *expired* has the fact backwards.
    #[must_use]
    pub const fn left(self) -> Option<Duration> {
        match self {
            Self::For(left) => Some(left),
            Self::UntilRemoved => None,
        }
    }

    /// Whether this grant runs until the person takes it away.
    #[must_use]
    pub const fn is_until_removed(self) -> bool {
        matches!(self, Self::UntilRemoved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two are opposite facts and neither answers for the other.
    #[test]
    fn a_stretch_of_time_and_no_end_are_told_apart() {
        let hour = Lasts::For(Duration::from_secs(60 * 60));
        assert_ne!(hour, Lasts::UntilRemoved);
        assert!(!hour.is_until_removed());
        assert!(Lasts::UntilRemoved.is_until_removed());
    }

    /// Nothing here can be read as a length of time, which is the mistake the
    /// type exists to prevent.
    #[test]
    fn no_end_has_no_duration_to_compare_against() {
        assert_eq!(Lasts::UntilRemoved.left(), None);
        assert!(Lasts::For(Duration::from_secs(1)).left().is_some());
    }
}
