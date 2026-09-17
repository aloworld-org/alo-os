//! The radio, and what off means.

use serde::{Deserialize, Serialize};

use crate::words::{self, Word};

/// **Whether this machine's Bluetooth radio is on.**
///
/// # Off means off for every device
///
/// Not *disconnected from the ones you are using now*, not *off until something
/// asks*: a device that was connected a moment ago is not connected, and a
/// device that asks to connect is not answered. A radio switch that left one
/// paired thing talking would be a switch a person could not trust, and the
/// point of the switch is being able to trust it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Radio {
    /// Nothing is connected and nothing may connect.
    #[default]
    Off,
    /// It is on.
    On,
}

impl Radio {
    /// Whether anything at all may be talking to this machine over Bluetooth.
    #[must_use]
    pub const fn anything_may_connect(self) -> bool {
        matches!(self, Self::On)
    }

    /// What a person is shown beside the switch.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::On => words::RADIO_ON,
            Self::Off => words::RADIO_OFF,
        }
    }

    /// The other one, for a switch.
    #[must_use]
    pub const fn switched(self) -> Self {
        match self {
            Self::On => Self::Off,
            Self::Off => Self::On,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A machine starts with the radio off**, which is the only default that
    /// is not a decision made on somebody's behalf.
    #[test]
    fn nothing_may_connect_to_a_machine_nobody_has_switched_on() {
        assert_eq!(Radio::default(), Radio::Off);
        assert!(!Radio::default().anything_may_connect());
        assert!(Radio::On.anything_may_connect());
        assert_eq!(Radio::On.switched(), Radio::Off);
        assert_eq!(Radio::Off.switched(), Radio::On);
    }
}
