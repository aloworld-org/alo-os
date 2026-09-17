//! What the kernel says about the battery at one moment.

use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// **How full the battery is**, in hundredths of itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Charge(u8);

impl Charge {
    /// Full.
    pub const FULL: u8 = 100;

    /// **Where a person is told once that it is getting low.**
    ///
    /// Twenty rather than ten: the number is not about the battery, it is about
    /// how long somebody has to find a cable, and ten per cent of a worn-out
    /// battery is a few minutes.
    pub const LOW: u8 = 20;

    /// **Where a person is told once that it is nearly gone.**
    pub const CRITICAL: u8 = 5;

    /// A charge the kernel reported.
    ///
    /// # Errors
    /// [`NotAReading::PastFull`] above a hundred: a battery reporting 127 per
    /// cent is a battery whose reading means nothing, and showing it would mean
    /// standing behind it.
    pub const fn reported(hundredths: u8) -> Result<Self, NotAReading> {
        if hundredths > Self::FULL {
            return Err(NotAReading::PastFull {
                said: hundredths as u16,
            });
        }
        Ok(Self(hundredths))
    }

    /// How full it is.
    #[must_use]
    pub const fn hundredths(self) -> u8 {
        self.0
    }

    /// Whether this is at or below where a person is told it is low.
    #[must_use]
    pub const fn is_low(self) -> bool {
        self.0 <= Self::LOW
    }

    /// Whether this is at or below where a person is told it is nearly gone.
    #[must_use]
    pub const fn is_critical(self) -> bool {
        self.0 <= Self::CRITICAL
    }
}

/// **What the battery is doing.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Charging {
    /// It is filling.
    Charging,
    /// It is emptying: the machine is running on it.
    Discharging,
    /// It is full and the machine is on the mains.
    Full,
    /// It is neither, which some machines say while they decide.
    NotCharging,
    /// The kernel would not say.
    Unknown,
}

impl Charging {
    /// What the kernel wrote, read.
    #[must_use]
    pub fn reported(said: &str) -> Self {
        match said.trim() {
            "Charging" => Self::Charging,
            "Discharging" => Self::Discharging,
            "Full" => Self::Full,
            "Not charging" => Self::NotCharging,
            _ => Self::Unknown,
        }
    }

    /// Whether the machine is running off the battery now.
    #[must_use]
    pub const fn is_on_the_battery(self) -> bool {
        matches!(self, Self::Discharging)
    }
}

/// **One reading of the battery.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading {
    /// How full.
    charge: Charge,
    /// What it is doing.
    charging: Charging,
    /// How much is going through it now, in microamperes, where the kernel
    /// said. Negative is out of the battery on some machines and positive on
    /// others, so only its size is used.
    through_it: Option<i64>,
    /// When it was taken.
    taken_at: SystemTime,
}

impl Reading {
    /// A reading taken now.
    #[must_use]
    pub const fn taken(
        charge: Charge,
        charging: Charging,
        through_it: Option<i64>,
        taken_at: SystemTime,
    ) -> Self {
        Self {
            charge,
            charging,
            through_it,
            taken_at,
        }
    }

    /// How full it is.
    #[must_use]
    pub const fn charge(&self) -> Charge {
        self.charge
    }

    /// What it is doing.
    #[must_use]
    pub const fn charging(&self) -> Charging {
        self.charging
    }

    /// How much is going through it, whichever way round this machine writes
    /// it down.
    #[must_use]
    pub const fn through_it(&self) -> Option<u64> {
        match self.through_it {
            Some(through_it) => Some(through_it.unsigned_abs()),
            None => None,
        }
    }

    /// When it was taken.
    #[must_use]
    pub const fn taken_at(&self) -> SystemTime {
        self.taken_at
    }

    /// How long ago, from a later reading.
    #[must_use]
    pub fn since(&self, earlier: &Self) -> Duration {
        self.taken_at
            .duration_since(earlier.taken_at)
            .unwrap_or_default()
    }
}

/// Why something is not a reading this crate will hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotAReading {
    /// A battery past full.
    #[error(
        "this machine says its battery is {said} per cent full, which is not a reading anybody \
         should be shown"
    )]
    PastFull {
        /// What it said.
        said: u16,
    },
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A reading past full is refused**, rather than shown.
    #[test]
    fn a_battery_past_full_is_not_a_reading() {
        assert!(matches!(
            Charge::reported(127),
            Err(NotAReading::PastFull { said: 127 })
        ));
        assert_eq!(Charge::reported(100).expect("full").hundredths(), 100);
    }

    /// **Low and nearly gone are where the file says**, and both are inclusive:
    /// a person at exactly twenty is at twenty.
    #[test]
    fn low_and_critical_are_where_this_file_says_they_are() {
        let at = |per_cent| Charge::reported(per_cent).expect("a charge");
        assert!(at(20).is_low() && !at(21).is_low());
        assert!(at(5).is_critical() && !at(6).is_critical());
        assert!(
            at(5).is_low(),
            "nearly gone is also low, so anything reading the two does not have to know the order"
        );
    }

    /// **However a machine writes the current down, only its size is used.**
    #[test]
    fn the_current_is_read_without_caring_which_way_round_it_is_written() {
        let now = SystemTime::UNIX_EPOCH;
        let one = Reading::taken(
            Charge::reported(50).expect("a charge"),
            Charging::Discharging,
            Some(-1_600),
            now,
        );
        let another = Reading::taken(
            Charge::reported(50).expect("a charge"),
            Charging::Discharging,
            Some(1_600),
            now,
        );
        assert_eq!(one.through_it(), another.through_it());
        assert_eq!(one.through_it(), Some(1_600));
        assert!(one.charging().is_on_the_battery());
    }

    /// **What the kernel wrote is read, and anything else is unknown** rather
    /// than guessed at.
    #[test]
    fn what_the_kernel_wrote_is_read_and_nothing_is_guessed() {
        assert_eq!(Charging::reported("Charging"), Charging::Charging);
        assert_eq!(Charging::reported(" Discharging\n"), Charging::Discharging);
        assert_eq!(Charging::reported("Full"), Charging::Full);
        assert_eq!(Charging::reported("Not charging"), Charging::NotCharging);
        assert_eq!(Charging::reported("something else"), Charging::Unknown);
        assert!(!Charging::Unknown.is_on_the_battery());
    }
}
