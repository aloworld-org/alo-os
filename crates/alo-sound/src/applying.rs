//! **What a person set, put back on the machine.**
//!
//! A device that is plugged in again comes back as the server's defaults: loud,
//! not muted, and chosen or not by the server's own policy. What a person set is
//! in this crate's file (ADR 0038), and this is the one place that takes the
//! difference between the two and tells the machine about it.
//!
//! It is written as *bring into line* rather than *apply on plug-in* on purpose.
//! There is no event to be missed: whenever anything asks, the answer is the
//! same — here is what the machine is doing, here is what the person set, and
//! here is the difference. A missed plug-in event is then a device that is a
//! moment late, never a mute that quietly did not happen.

use crate::asking::Sound;
use crate::device::{Identity, Kind};
use crate::keeping::Kept;
use crate::meant::{WhatWasMeant, what_was_meant};
use crate::refusing::NotDone;
use crate::words::Word;

/// **What was done to the machine, and what a person is told.**
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Brought {
    /// The devices now in use, one per kind this machine has.
    in_use: Vec<(Kind, Identity)>,
    /// The devices whose volume or mute was put back.
    put_back: Vec<Identity>,
    /// What a person is told, which is usually nothing.
    say: Vec<Word>,
}

impl Brought {
    /// The device in use for each kind.
    #[must_use]
    pub fn in_use(&self) -> &[(Kind, Identity)] {
        &self.in_use
    }

    /// The devices whose volume or mute this crate had to put back.
    #[must_use]
    pub fn put_back(&self) -> &[Identity] {
        &self.put_back
    }

    /// What a person is told. Empty on an ordinary day.
    #[must_use]
    pub fn say(&self) -> &[Word] {
        &self.say
    }
}

/// **Make the machine do what a person set.**
///
/// Ask it what it is doing, work out what was meant, and tell it the
/// difference: which device to use for each kind, and the volume and mute of
/// every device that is here.
///
/// # Errors
/// [`NotDone`] where the machine could not be read, or refused something it was
/// told. A refusal stops the rest: a machine that has just refused to mute a
/// microphone is not a machine to carry on issuing instructions to as though it
/// had not.
pub fn bring_into_line(sound: &mut impl Sound, kept: &Kept) -> Result<Brought, NotDone> {
    let heard = sound.heard_now()?;
    let mut brought = Brought::default();

    for kind in [Kind::Output, Kind::Input] {
        let meant: WhatWasMeant = what_was_meant(&heard, kept, kind);
        let Some(device) = meant.device() else {
            continue;
        };
        if meant.differs_from_what_is_in_use(&heard, kind) {
            sound.choose(device.identity(), kind)?;
        }
        brought.in_use.push((kind, device.identity().clone()));
        brought.say.extend(meant.why().word());
    }

    for device in heard.devices().all() {
        let identity = device.identity();
        let Some(doing) = heard.doing(identity) else {
            continue;
        };
        let mut put_back = false;
        if let Some(volume) = kept.volume.get(identity).copied()
            && volume != doing.volume()
        {
            sound.set_volume(identity, volume)?;
            put_back = true;
        }
        if let Some(mute) = kept.muted.get(identity).copied()
            && mute != doing.mute()
        {
            sound.set_mute(identity, mute)?;
            put_back = true;
        }
        if put_back {
            brought.put_back.push(identity.clone());
        }
    }

    Ok(brought)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::device::{OneDevice, Volume};
    use crate::mute::Mute;
    use crate::testing::{AMachine, Told, a_machine, a_machine_with};

    fn identity(named: &str) -> Identity {
        Identity::reported(named).expect("named")
    }

    /// **A headset plugged in again is the device it was**, muted if that is how
    /// it was left and at the volume it was left at — and the machine is told
    /// nothing about the devices that were already right.
    #[test]
    fn a_device_that_came_back_is_put_where_a_person_left_it() {
        let heard = a_machine(
            &[("laptop-speakers", Kind::Output), ("headset", Kind::Output)],
            Some("laptop-speakers"),
            None,
        );
        let headset = OneDevice::reported(identity("headset"), "A headset", Kind::Output);
        let mut kept = Kept {
            pinned: crate::Pinned::nothing().and(&headset),
            ..Kept::default()
        };
        kept.volume
            .insert(identity("headset"), Volume::of(40).expect("a volume"));
        kept.muted.insert(identity("headset"), Mute::On);

        let mut machine = AMachine::answering(heard);
        let brought = bring_into_line(&mut machine, &kept).expect("a machine that answers");

        assert_eq!(
            machine.told(),
            [
                Told::Use("headset".to_owned(), Kind::Output),
                Told::Volume("headset".to_owned(), Volume::of(40).expect("a volume")),
                Told::Muted("headset".to_owned(), Mute::On),
            ],
            "a device that came back did not get what a person had set on it"
        );
        assert_eq!(brought.put_back(), [identity("headset")]);
        assert_eq!(brought.in_use(), [(Kind::Output, identity("headset"))]);
        assert!(brought.say().is_empty());
    }

    /// **A machine that is already doing what a person set is left alone.**
    /// Telling a server to set what it has already set is how a volume slider
    /// flickers and how a mute toggles under somebody's hand.
    #[test]
    fn a_machine_already_doing_it_is_told_nothing() {
        let heard = a_machine_with(&["laptop-speakers"], Some("laptop-speakers"));
        let mut machine = AMachine::answering(heard);
        let brought = bring_into_line(&mut machine, &Kept::default()).expect("answers");
        assert!(machine.told().is_empty());
        assert!(brought.put_back().is_empty());
        assert_eq!(
            brought.in_use(),
            [(Kind::Output, identity("laptop-speakers"))]
        );
    }

    /// **A pinned device that is away leaves the machine's choice alone**, and
    /// the one sentence about it is carried back rather than left for a person
    /// to work out.
    #[test]
    fn a_pinned_device_that_is_away_changes_nothing_and_is_said_once() {
        let heard = a_machine_with(&["laptop-speakers"], Some("laptop-speakers"));
        let away = OneDevice::reported(identity("desk-speakers"), "Desk speakers", Kind::Output);
        let kept = Kept {
            pinned: crate::Pinned::nothing().and(&away),
            ..Kept::default()
        };

        let mut machine = AMachine::answering(heard);
        let brought = bring_into_line(&mut machine, &kept).expect("answers");
        assert!(
            machine.told().is_empty(),
            "sound was moved to a device that is not here"
        );
        assert_eq!(brought.say().len(), 1);
    }

    /// **What was set for a device that is not here is kept and not sent.** The
    /// file outlives the hardware, which is the point of keeping it by identity
    /// — and a machine told to mute something it does not have would refuse,
    /// which would stop everything after it.
    #[test]
    fn a_setting_for_a_device_that_is_not_here_is_kept_and_not_sent() {
        let heard = a_machine_with(&["laptop-speakers"], Some("laptop-speakers"));
        let mut kept = Kept::default();
        kept.muted
            .insert(identity("a-headset-sold-last-year"), Mute::On);
        kept.volume.insert(
            identity("laptop-speakers"),
            Volume::of(30).expect("a volume"),
        );

        let mut machine = AMachine::answering(heard);
        let brought = bring_into_line(&mut machine, &kept).expect("answers");
        assert_eq!(
            machine.told(),
            [Told::Volume(
                "laptop-speakers".to_owned(),
                Volume::of(30).expect("a volume")
            )],
            "a setting for a device that is not here should be kept and not sent"
        );
        assert_eq!(brought.put_back(), [identity("laptop-speakers")]);
    }
}
