//! The battery, as the lock screen may show it: how full, and whether it is
//! charging.
//!
//! Read by whoever reads the machine's power supply and handed in at the moment
//! the lock screen is made; this crate reads no hardware. It is on the lock
//! screen because somebody who walks up to a locked laptop has a fair question
//! — *will this last the meeting* — whose answer tells a stranger nothing about
//! the person whose machine it is.

/// A reading of the battery.
///
/// A machine with no battery has no reading, which is `None` where one is
/// taken rather than a battery at zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Battery {
    /// How full, from nought to a hundred.
    percent: u8,
    /// Whether power is going in.
    charging: bool,
}

/// A reading that is not one.
///
/// A reader that produced it is alo OS's own bug, read by whoever is fixing
/// that reader, so it keeps its English.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("a battery cannot be {0} percent full")]
pub struct NotABattery(pub u8);

impl Battery {
    /// This much charge, charging or not.
    ///
    /// # Errors
    /// [`NotABattery`] above a hundred percent.
    pub const fn reading(percent: u8, charging: bool) -> Result<Self, NotABattery> {
        if percent > 100 {
            return Err(NotABattery(percent));
        }
        Ok(Self { percent, charging })
    }

    /// How full, from nought to a hundred.
    #[must_use]
    pub const fn percent(self) -> u8 {
        self.percent
    }

    /// Whether power is going in.
    #[must_use]
    pub const fn is_charging(self) -> bool {
        self.charging
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nought to a hundred is a reading; more than that is a reader's bug.
    #[test]
    fn a_battery_is_between_empty_and_full() {
        assert_eq!(Battery::reading(0, false).map(Battery::percent), Ok(0));
        assert_eq!(
            Battery::reading(100, true).map(Battery::is_charging),
            Ok(true)
        );
        assert_eq!(Battery::reading(101, false), Err(NotABattery(101)));
    }
}
