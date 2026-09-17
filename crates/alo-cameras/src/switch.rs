//! **The switch that turns the camera or the microphone off for everyone.**
//!
//! Not a grant, and not a permission: a grant is about one application, and this
//! is about the machine. A person who turns the camera off has turned it off for
//! the application they granted it to last week, for the one they are about to
//! install, and for the agent — and **the switch wins**. There is no application
//! important enough to be an exception, because an exception is what a person
//! would have to know about to be able to trust the switch at all.
//!
//! # Off is held below the door, not at it
//!
//! A switch checked where a grant is checked is a switch that only covers the
//! programs that ask politely. [`crate::letting_go`] is the other half: where the
//! machine can, it **takes the device away from the kernel**, so a program that
//! never asked anybody finds nothing there either. Where it cannot, this crate
//! says so rather than implying a promise it is not keeping.

use serde::{Deserialize, Serialize};

use crate::words::{self, Word};

/// **Which of the two.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Which {
    /// Everything that sees.
    Camera,
    /// Everything that hears.
    Microphone,
}

impl Which {
    /// Both of them, in the order a person meets them.
    pub const BOTH: [Self; 2] = [Self::Camera, Self::Microphone];
}

/// **On, or off for everyone.**
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Switch {
    /// Anything a person has granted it to may use it.
    #[default]
    On,
    /// Nothing may, whatever it was granted.
    Off,
}

impl Switch {
    /// Whether anything at all may open it.
    #[must_use]
    pub const fn anything_may_open_it(self) -> bool {
        matches!(self, Self::On)
    }

    /// The other one.
    #[must_use]
    pub const fn switched(self) -> Self {
        match self {
            Self::On => Self::Off,
            Self::Off => Self::On,
        }
    }
}

/// **What a person has switched off on this machine.**
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TheSwitches {
    /// The camera.
    #[serde(default)]
    pub camera: Switch,
    /// The microphone.
    #[serde(default)]
    pub microphone: Switch,
}

impl TheSwitches {
    /// Both on, which is where a machine starts.
    #[must_use]
    pub fn both_on() -> Self {
        Self::default()
    }

    /// How this one is set.
    #[must_use]
    pub const fn of(&self, which: Which) -> Switch {
        match which {
            Which::Camera => self.camera,
            Which::Microphone => self.microphone,
        }
    }

    /// The same, with this one switched.
    #[must_use]
    pub const fn switching(mut self, which: Which) -> Self {
        match which {
            Which::Camera => self.camera = self.camera.switched(),
            Which::Microphone => self.microphone = self.microphone.switched(),
        }
        self
    }

    /// **Whether this may be opened at all**, which is asked before a grant is
    /// looked at rather than after.
    ///
    /// # Errors
    /// [`TurnedOff`] where a person has turned it off, whatever was granted.
    pub const fn may_open(&self, which: Which) -> Result<(), TurnedOff> {
        if self.of(which).anything_may_open_it() {
            return Ok(());
        }
        Err(TurnedOff { which })
    }
}

/// **A person turned it off.**
///
/// Carries which one, because *the camera is off* and *the microphone is off*
/// are two different things to be told and a person fixes them with two
/// different switches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("a person has turned this machine's {which:?} off for everything")]
pub struct TurnedOff {
    /// Which one.
    pub which: Which,
}

impl TurnedOff {
    /// What the person is shown.
    #[must_use]
    pub const fn word(self) -> Word {
        match self.which {
            Which::Camera => words::THE_CAMERA_IS_OFF,
            Which::Microphone => words::THE_MICROPHONE_IS_OFF,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A machine starts with both on**, and turning one off leaves the other
    /// alone: a person who covers their camera has not muted their calls.
    #[test]
    fn each_switch_is_its_own() {
        let both = TheSwitches::both_on();
        assert!(both.may_open(Which::Camera).is_ok());
        assert!(both.may_open(Which::Microphone).is_ok());

        let camera_off = both.switching(Which::Camera);
        assert_eq!(
            camera_off.may_open(Which::Camera),
            Err(TurnedOff {
                which: Which::Camera
            })
        );
        assert!(camera_off.may_open(Which::Microphone).is_ok());
        assert_eq!(camera_off.switching(Which::Camera), both);
    }

    /// **Each refusal says which one**, because two switches need two sentences.
    #[test]
    fn a_refusal_names_the_switch_that_would_undo_it() {
        for which in Which::BOTH {
            let off = TheSwitches::both_on().switching(which);
            let why = off.may_open(which).expect_err("it is off");
            assert_eq!(why.which, which);
            assert!(!why.word().says().is_empty());
        }
        assert_ne!(
            TurnedOff {
                which: Which::Camera
            }
            .word()
            .says(),
            TurnedOff {
                which: Which::Microphone
            }
            .word()
            .says()
        );
    }
}
