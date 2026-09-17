//! **Which device a person meant**, given what is here and what they pinned.
//!
//! This is the whole of what this crate decides. The server knows which devices
//! exist and which are in use; it has no view about what somebody wanted, and a
//! machine that cannot tell the difference between *the policy chose the laptop
//! speakers* and *this person always uses the desk speakers* will be wrong on
//! exactly the day it matters.

use crate::device::{Kind, OneDevice};
use crate::heard::Heard;
use crate::keeping::Kept;
use crate::words::{self, Word};

/// **Why this device**, which is what a person is owed when it is not the one
/// they pinned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    /// A person pinned it and it is here.
    Pinned,
    /// A person pinned something else and it is not here, so the machine chose.
    ThePinnedOneIsAway,
    /// Nothing was pinned, so the machine chose, which is the ordinary day.
    TheMachineChose,
}

impl Why {
    /// **What a person is told**, or [`None`] where there is nothing to say.
    ///
    /// Only one of the three is worth a sentence. *The machine chose and you
    /// pinned nothing* is not news, and *the machine chose what you pinned* is
    /// the thing working.
    #[must_use]
    pub const fn word(self) -> Option<Word> {
        match self {
            Self::ThePinnedOneIsAway => Some(words::PINNED_ONE_IS_AWAY),
            Self::Pinned | Self::TheMachineChose => None,
        }
    }
}

/// **The device a person meant for one kind**, and why it is that one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatWasMeant {
    /// The device, or [`None`] on a machine with none of that kind.
    device: Option<OneDevice>,
    /// Why it is that one.
    why: Why,
}

impl WhatWasMeant {
    /// The device, or [`None`] where this machine has none of that kind — which
    /// is a real machine and not an error.
    #[must_use]
    pub const fn device(&self) -> Option<&OneDevice> {
        self.device.as_ref()
    }

    /// Why it is that one.
    #[must_use]
    pub const fn why(&self) -> Why {
        self.why
    }

    /// **Whether the machine has to be told anything**, which it does only when
    /// what a person meant is not what is in use.
    #[must_use]
    pub fn differs_from_what_is_in_use(&self, heard: &Heard, kind: Kind) -> bool {
        match self.device.as_ref() {
            Some(device) => heard.chosen(kind) != Some(device.identity()),
            None => false,
        }
    }
}

/// **Which device a person meant for this kind, now.**
///
/// The pinned one where it is here. Where it is not, whatever the machine chose
/// — with [`Why::ThePinnedOneIsAway`], so that a person who wonders why their
/// call is coming out of the laptop can be told rather than left to guess.
#[must_use]
pub fn what_was_meant(heard: &Heard, kept: &Kept, kind: Kind) -> WhatWasMeant {
    if let Some(pinned) = kept.pinned.chosen_from(heard.devices(), kind) {
        return WhatWasMeant {
            device: Some(pinned.clone()),
            why: Why::Pinned,
        };
    }
    let in_use = heard.chosen(kind).and_then(|chosen| {
        heard
            .devices()
            .of_kind(kind)
            .find(|device| device.identity() == chosen)
    });
    WhatWasMeant {
        device: in_use.cloned(),
        why: if kept.pinned.for_kind(kind).is_some() {
            Why::ThePinnedOneIsAway
        } else {
            Why::TheMachineChose
        },
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::device::Identity;
    use crate::testing::a_machine_with;

    /// **A pinned device that is here is what a person meant**, whatever the
    /// machine chose.
    #[test]
    fn the_pinned_device_is_what_was_meant_even_when_the_machine_chose_another() {
        let heard = a_machine_with(
            &["laptop-speakers", "desk-speakers"],
            Some("laptop-speakers"),
        );
        let desk = heard
            .devices()
            .all()
            .iter()
            .find(|device| device.identity().as_str() == "desk-speakers")
            .expect("the desk speakers are here")
            .clone();
        let kept = Kept {
            pinned: crate::Pinned::nothing().and(&desk),
            ..Kept::default()
        };

        let meant = what_was_meant(&heard, &kept, Kind::Output);
        assert_eq!(
            meant.device().map(OneDevice::identity),
            Some(desk.identity())
        );
        assert_eq!(meant.why(), Why::Pinned);
        assert!(
            meant.differs_from_what_is_in_use(&heard, Kind::Output),
            "the machine is using something else and nothing would be said to it"
        );
        assert!(meant.why().word().is_none());
    }

    /// **A pin for a device that is away is not silence**, and a person is told
    /// once why the sound is coming out of something else.
    #[test]
    fn a_pin_for_a_device_that_is_away_leaves_the_machines_choice_and_says_so() {
        let heard = a_machine_with(&["laptop-speakers"], Some("laptop-speakers"));
        let away = OneDevice::reported(
            Identity::reported("desk-speakers").expect("named"),
            "Desk speakers",
            Kind::Output,
        );
        let kept = Kept {
            pinned: crate::Pinned::nothing().and(&away),
            ..Kept::default()
        };

        let meant = what_was_meant(&heard, &kept, Kind::Output);
        assert_eq!(
            meant.device().map(|device| device.identity().as_str()),
            Some("laptop-speakers")
        );
        assert_eq!(meant.why(), Why::ThePinnedOneIsAway);
        assert!(!meant.differs_from_what_is_in_use(&heard, Kind::Output));
        assert!(
            meant.why().word().is_some(),
            "nobody is told why their pinned device is not the one playing"
        );
    }

    /// **Nothing pinned is the ordinary day**, and it says nothing.
    #[test]
    fn with_nothing_pinned_the_machines_choice_stands_and_nothing_is_said() {
        let heard = a_machine_with(&["laptop-speakers"], Some("laptop-speakers"));
        let meant = what_was_meant(&heard, &Kept::default(), Kind::Output);
        assert_eq!(meant.why(), Why::TheMachineChose);
        assert!(meant.why().word().is_none());
    }

    /// **A machine with no devices of that kind is a machine, not a failure.**
    #[test]
    fn a_machine_with_no_devices_of_that_kind_is_not_an_error() {
        let heard = a_machine_with(&[], None);
        let meant = what_was_meant(&heard, &Kept::default(), Kind::Output);
        assert!(meant.device().is_none());
        assert!(!meant.differs_from_what_is_in_use(&heard, Kind::Output));
    }
}
