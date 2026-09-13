//! Two totals and an interval, made into a rate.
//!
//! The kernel keeps totals since a process began. A person asking *why is it
//! slow?* wants what it is doing now, which is the difference between two
//! totals over the time between them. This file is that arithmetic and
//! nothing else: it does not know what a process is, and it does not consult
//! a clock.
//!
//! # A rate is only a number when both totals were
//!
//! A total the kernel withheld at either reading gives a rate the kernel
//! withheld. The alternative — a rate over the one total that was there —
//! would be a number derived from a guess, and every [`Number`] that is not
//! [`Number::Known`] exists so that a window never has to show one.

use std::time::Duration;

use crate::source::{Number, Source};

/// The difference between two totals, or the reason there is none.
///
/// The later reading's state wins when it is not a number, because it is the
/// more recent fact about the file; then the earlier's. A difference that
/// would go backwards is nought — the kernel's counters do not run backwards
/// for one process, and a pid reused by another is told apart before this is
/// reached.
fn difference(later: &Number, earlier: &Number) -> Result<(u64, Source), Number> {
    match (later, earlier) {
        (Number::Known(now), Number::Known(then)) => {
            Ok((now.value.saturating_sub(then.value), now.from.clone()))
        }
        (Number::Known(_), missing) | (missing, _) => Err(missing.clone()),
    }
}

/// Bytes, or anything counted, per second over the interval.
///
/// The interval is the caller's and is never nought here: [`crate::Reading::since`]
/// refuses one before any rate is made.
#[must_use]
pub(crate) fn per_second(later: &Number, earlier: &Number, interval: Duration) -> Number {
    match difference(later, earlier) {
        Ok((delta, from)) => {
            let millis = interval.as_millis().max(1);
            let per_second = u128::from(delta).saturating_mul(1000) / millis;
            Number::known(u64::try_from(per_second).unwrap_or(u64::MAX), from)
        }
        Err(missing) => missing,
    }
}

/// A process's share of the whole machine's processor, in thousandths.
///
/// The process's ticks over the interval against every processor's ticks in
/// every state over the same interval, which is what `/proc/stat`'s `cpu`
/// line sums. A single thread busy on one of eight processors is `125`. The
/// machine's ticks are never nought here: [`crate::Reading::since`] refuses
/// two readings between which they did not move.
#[must_use]
pub(crate) fn share(later: &Number, earlier: &Number, machine_ticks: u64) -> Number {
    match difference(later, earlier) {
        Ok((delta, from)) => {
            let thousandths =
                u128::from(delta).saturating_mul(1000) / u128::from(machine_ticks.max(1));
            Number::known(u64::try_from(thousandths).unwrap_or(u64::MAX), from)
        }
        Err(missing) => missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A source for these tests.
    fn from() -> Source {
        Source::of("/proc/42/io", "wchar")
    }

    /// The difference over the interval, per second, named for the later
    /// reading's file.
    #[test]
    fn a_rate_is_the_difference_over_the_interval_passed_in() {
        let earlier = Number::known(1000, from());
        let later = Number::known(5000, from());
        assert_eq!(
            per_second(&later, &earlier, Duration::from_secs(2)),
            Number::known(2000, from())
        );
        assert_eq!(
            per_second(&later, &earlier, Duration::from_millis(500)),
            Number::known(8000, from())
        );
        // The interval is what was passed in, and nothing here knows how long
        // it really was.
        assert_eq!(
            per_second(&later, &earlier, Duration::from_secs(4000)),
            Number::known(1, from())
        );
    }

    /// A share is thousandths of every processor's time.
    #[test]
    fn a_share_is_thousandths_of_the_whole_machine() {
        let earlier = Number::known(100, from());
        let later = Number::known(200, from());
        assert_eq!(share(&later, &earlier, 800), Number::known(125, from()));
        assert_eq!(share(&later, &earlier, 100), Number::known(1000, from()));
        assert_eq!(share(&later, &later, 100), Number::known(0, from()));
    }

    /// **A total that was not a number at either reading is not a rate**, and
    /// the later reading's state is the one reported.
    #[test]
    fn a_rate_over_a_number_the_kernel_did_not_give_is_not_a_number() {
        let known = Number::known(5000, from());
        let withheld = Number::Withheld {
            from: from(),
            why: "permission denied".to_owned(),
        };
        let not_said = Number::NotSaid { from: from() };
        assert_eq!(
            per_second(&withheld, &known, Duration::from_secs(1)),
            withheld
        );
        assert_eq!(
            per_second(&known, &withheld, Duration::from_secs(1)),
            withheld
        );
        assert_eq!(
            per_second(&not_said, &withheld, Duration::from_secs(1)),
            not_said
        );
        assert_eq!(share(&known, &not_said, 100), not_said);
    }

    /// A counter cannot go backwards for one process, so a later total
    /// below an earlier one is nought and not an enormous number.
    #[test]
    fn a_counter_that_appears_to_go_backwards_is_nought() {
        let earlier = Number::known(5000, from());
        let later = Number::known(10, from());
        assert_eq!(
            per_second(&later, &earlier, Duration::from_secs(1)),
            Number::known(0, from())
        );
    }
}
