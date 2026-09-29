//! The stretch of time a statement is about, with both ends said out loud.
//!
//! **Half-open: `from` is included and `until` is not.** That is the one choice
//! here and it is made for a specific reason rather than taste.
//!
//! An attestation is checkable only if two people rendering *the same period*
//! from the same record produce the same bytes. *The same day* is not a period
//! until somebody says whether midnight belongs to the day before or the day
//! after, and two auditors who answer differently get two digests over one set of
//! facts, with nothing to say which of them is wrong. Half-open also makes
//! consecutive periods meet exactly once: `[Mon, Tue)` and `[Tue, Wed)` cover
//! Monday and Tuesday with no entry counted twice and none missed, which a
//! closed-both-ends period cannot do at all.
//!
//! The ends are rendered into the statement rather than assumed, so a reader
//! never has to know this convention to check the arithmetic — see
//! [`crate::rendering`].

use std::time::SystemTime;

/// A stretch of time a statement is about: `from` included, `until` not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    /// The first moment inside it.
    from: SystemTime,

    /// The first moment after it.
    until: SystemTime,
}

/// What is wrong with a period somebody asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAPeriod {
    /// It ends before it starts, or at the same moment.
    ///
    /// An empty period is refused rather than allowed to render an empty
    /// statement. *Nothing left this machine* is a strong claim, and one made by
    /// a period that could not have contained anything is the worst kind of
    /// evidence — true, checkable, and about nothing.
    ItEndsBeforeItStarts,
}

impl Period {
    /// A period from `from` up to but not including `until`.
    ///
    /// # Errors
    ///
    /// [`NotAPeriod::ItEndsBeforeItStarts`] if `until` is not after `from`.
    pub fn of(from: SystemTime, until: SystemTime) -> Result<Self, NotAPeriod> {
        if until <= from {
            return Err(NotAPeriod::ItEndsBeforeItStarts);
        }
        Ok(Self { from, until })
    }

    /// The first moment inside it.
    #[must_use]
    pub fn from(&self) -> SystemTime {
        self.from
    }

    /// The first moment after it.
    #[must_use]
    pub fn until(&self) -> SystemTime {
        self.until
    }

    /// Whether a moment falls inside it.
    #[must_use]
    pub fn holds(&self, at: SystemTime) -> bool {
        at >= self.from && at < self.until
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use super::*;

    /// A moment, for readability in the tests below.
    fn at(seconds: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
    }

    /// **The ends behave the way the documentation says**, which is the whole
    /// reason to have named them.
    #[test]
    fn the_start_is_inside_and_the_end_is_not() {
        let period = Period::of(at(100), at(200)).expect("a period a hundred seconds long");
        assert!(period.holds(at(100)), "the first moment is inside it");
        assert!(period.holds(at(199)));
        assert!(!period.holds(at(200)), "the last moment is not");
        assert!(!period.holds(at(99)));
    }

    /// **Consecutive periods meet exactly once**, which is what half-open buys and
    /// what a closed-both-ends period cannot do.
    #[test]
    fn two_periods_that_meet_count_the_boundary_once() {
        let monday = Period::of(at(0), at(100)).expect("a period");
        let tuesday = Period::of(at(100), at(200)).expect("a period");
        let boundary = at(100);
        assert!(!monday.holds(boundary));
        assert!(tuesday.holds(boundary));
        assert_eq!(
            usize::from(monday.holds(boundary)) + usize::from(tuesday.holds(boundary)),
            1,
            "the moment where two periods meet belongs to exactly one of them"
        );
    }

    /// A period that could hold nothing is refused rather than allowed to state
    /// that nothing left.
    #[test]
    fn a_period_that_ends_before_it_starts_is_refused() {
        assert_eq!(
            Period::of(at(200), at(100)),
            Err(NotAPeriod::ItEndsBeforeItStarts)
        );
        assert_eq!(
            Period::of(at(100), at(100)),
            Err(NotAPeriod::ItEndsBeforeItStarts),
            "an instant is not a period"
        );
    }
}
