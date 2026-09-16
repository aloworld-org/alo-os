//! Sticky, slow and bounce keys, as the values a keyboard reads.
//!
//! One filter rather than three settings read separately, because a keyboard
//! applies one: a person who needs slow keys and bounce keys has one keyboard,
//! and two crates deciding separately what it does is how the second one gets
//! forgotten.
//!
//! The keyboard crate the desktop plan is building reads this. Until it exists
//! the values are still decided here, because *what a setting changes* is this
//! crate's question and *how a key is read* is that one's — and a value nobody
//! reads yet is better than a setting that means nothing until somebody writes
//! the other half.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// The longest a key may have to be held, or a repeat ignored, before the
/// filter is doing something a person did not ask for.
const LONGEST: Duration = Duration::from_millis(2_000);

/// **What a keyboard does with a key press when these settings are on.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct KeyFilter {
    /// A modifier stays held after it is let go, until the next key.
    pub sticky: bool,
    /// How long a key must be held before it counts, or [`None`].
    pub held_before_it_counts: Option<Duration>,
    /// How long after a key before the same key counts again, or [`None`].
    pub ignored_after_the_same_key: Option<Duration>,
}

impl KeyFilter {
    /// Nothing filtered: what a keyboard does for somebody who asked for none
    /// of this.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            sticky: false,
            held_before_it_counts: None,
            ignored_after_the_same_key: None,
        }
    }

    /// Whether this filter changes anything at all.
    #[must_use]
    pub const fn changes_anything(&self) -> bool {
        self.sticky
            || self.held_before_it_counts.is_some()
            || self.ignored_after_the_same_key.is_some()
    }

    /// A delay a person asked for, checked.
    ///
    /// # Errors
    /// [`NotADelay`] for nothing at all, or for longer than two seconds — a
    /// keyboard that ignores a key for longer than that is a keyboard somebody
    /// will think is broken, and they will not know which setting did it.
    pub fn a_delay_of(how_long: Duration) -> Result<Duration, NotADelay> {
        if how_long.is_zero() {
            return Err(NotADelay::NoTimeAtAll);
        }
        if how_long > LONGEST {
            return Err(NotADelay::LongerThanAnybodyMeans { longest: LONGEST });
        }
        Ok(how_long)
    }
}

/// Why a delay was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotADelay {
    /// Zero, which is the setting being off rather than a delay.
    #[error("a delay of no time at all is the setting turned off")]
    NoTimeAtAll,
    /// Longer than anybody means by *slow*.
    #[error("longer than {longest:?}, which a person would read as a broken keyboard")]
    LongerThanAnybodyMeans {
        /// The longest a delay may be.
        longest: Duration,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_filter_that_changes_nothing_says_so() {
        assert!(!KeyFilter::none().changes_anything());
        assert!(
            KeyFilter {
                sticky: true,
                ..KeyFilter::none()
            }
            .changes_anything()
        );
        assert!(
            KeyFilter {
                held_before_it_counts: Some(Duration::from_millis(300)),
                ..KeyFilter::none()
            }
            .changes_anything()
        );
    }

    /// **A delay is a length of time somebody meant.**
    #[test]
    fn a_delay_of_nothing_or_of_forever_is_refused() {
        assert_eq!(
            KeyFilter::a_delay_of(Duration::ZERO),
            Err(NotADelay::NoTimeAtAll)
        );
        assert!(matches!(
            KeyFilter::a_delay_of(Duration::from_secs(3)),
            Err(NotADelay::LongerThanAnybodyMeans { .. })
        ));
        assert_eq!(
            KeyFilter::a_delay_of(Duration::from_millis(300)),
            Ok(Duration::from_millis(300))
        );
        assert_eq!(KeyFilter::a_delay_of(LONGEST), Ok(LONGEST));
    }
}
