//! **A mute is the source stopped, not the volume taken to zero.**
//!
//! A microphone at gain zero is a microphone that is still listening. The
//! server still has the stream, the indicator still says the microphone is in
//! use — correctly — and whether anything can be heard depends on a number that
//! anything able to set it can set back.
//!
//! That difference matters exactly when somebody is relying on it: in a meeting,
//! before saying something they would not say to everyone. So a mute here is the
//! source stopped, and `tests/a_mute_is_silence_not_a_low_volume.rs` reads what
//! the stream carries rather than what the volume says.

use serde::{Deserialize, Serialize};

/// **Whether a device is muted, and what that means.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mute {
    /// The source runs and carries what it hears.
    #[default]
    Off,
    /// **The source is stopped.** Not attenuated, not faded: stopped, so that
    /// nothing downstream carries anything to be recovered.
    On,
}

impl Mute {
    /// Whether anything reaches anywhere from this device.
    #[must_use]
    pub const fn anything_is_heard(self) -> bool {
        matches!(self, Self::Off)
    }

    /// The other one.
    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::Off => Self::On,
            Self::On => Self::Off,
        }
    }

    /// **What a person reads.**
    #[must_use]
    pub const fn word(self) -> crate::words::Word {
        match self {
            Self::Off => crate::words::NOT_MUTED,
            Self::On => crate::words::MUTED,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A machine nobody has touched is not muted**, and says so rather than
    /// leaving a person to guess from a silent room.
    #[test]
    fn a_device_is_not_muted_until_somebody_mutes_it() {
        assert_eq!(Mute::default(), Mute::Off);
        assert!(Mute::default().anything_is_heard());
    }

    /// **Muted means nothing is heard**, and it is one act to undo.
    #[test]
    fn muted_means_nothing_is_heard() {
        assert!(!Mute::On.anything_is_heard());
        assert_eq!(Mute::On.toggled(), Mute::Off);
        assert_eq!(Mute::Off.toggled(), Mute::On);
        assert_ne!(Mute::On.word().key(), Mute::Off.word().key());
    }
}
