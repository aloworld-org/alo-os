//! **Turning the camera off, and what the machine does about it.**
//!
//! One act with two halves, and the second is the one that makes it worth
//! anything:
//!
//! 1. the switch is kept, so nothing that asks is given the camera
//!    ([`crate::switch`], [`crate::keeping`]);
//! 2. the machine **lets go of the hardware**, so there is nothing to ask for
//!    ([`crate::letting_go`]).
//!
//! Where the second cannot be done, this says which devices it could not do it
//! for, and the sentence a person is shown changes to match. A promise this
//! machine cannot keep is not made quietly.

use crate::letting_go::{self, LetGo, NotLetGo};
use crate::seen::Seen;
use crate::words::{self, Word};

/// **What turning the switch off actually did.**
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WhatWasDone {
    /// The devices the machine let go of, each knowing its way back.
    let_go: Vec<LetGo>,
    /// The devices it could not let go of, and why.
    still_there: Vec<NotLetGo>,
}

impl WhatWasDone {
    /// The devices the machine let go of.
    #[must_use]
    pub fn let_go(&self) -> &[LetGo] {
        &self.let_go
    }

    /// The ones it could not, with the reason each time.
    #[must_use]
    pub fn still_there(&self) -> &[NotLetGo] {
        &self.still_there
    }

    /// **Whether off is as strong as it sounds on this machine**: nothing left
    /// for anything to open.
    #[must_use]
    pub fn nothing_is_left_to_open(&self) -> bool {
        self.still_there.is_empty()
    }

    /// What a person is told, which is different on a machine that could only
    /// shut the door.
    #[must_use]
    pub fn word(&self) -> Word {
        if self.nothing_is_left_to_open() {
            return words::THE_DEVICE_WAS_LET_GO;
        }
        words::ONLY_THE_DOOR
    }
}

/// **Turn the cameras off for everyone**, as far as this machine allows.
///
/// Every camera the machine reported is let go of. Whatever could not be is in
/// [`WhatWasDone::still_there`] with the reason, and the switch is still off —
/// the door is shut either way, and this says how much more than a door it is.
#[must_use]
pub fn every_camera_let_go(seen: &Seen) -> WhatWasDone {
    let mut done = WhatWasDone::default();
    for device_file in seen.every_device_file() {
        match letting_go::let_go_of(device_file) {
            Ok(gone) => done.let_go.push(gone),
            Err(why) => done.still_there.push(why),
        }
    }
    done
}

/// **Give back every device this machine let go of**, when the switch goes on
/// again.
///
/// # Errors
/// The first refusal, which leaves the rest untouched: a machine that could not
/// give a camera back is one whose person has to be told, not one to carry on
/// past.
pub fn every_camera_given_back(done: &WhatWasDone) -> Result<(), NotLetGo> {
    for gone in &done.let_go {
        letting_go::take_back(gone)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A machine with no cameras has nothing to let go of**, and that is not
    /// the same as a machine that could not let go of anything: it says nothing
    /// is left to open, because nothing is.
    #[test]
    fn a_machine_with_no_cameras_has_nothing_left_to_open() {
        let done = every_camera_let_go(&Seen::default());
        assert!(done.let_go().is_empty());
        assert!(done.still_there().is_empty());
        assert!(done.nothing_is_left_to_open());
        assert_eq!(done.word().named(), words::THE_DEVICE_WAS_LET_GO.named());
        every_camera_given_back(&done).expect("there is nothing to give back");
    }
}
