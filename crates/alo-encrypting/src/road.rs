//! The order enrolment happens in, which is what makes the recovery key
//! unskippable.
//!
//! Six steps, once, at install. This crate does not take them — it names them,
//! in the one order
//! [ADR 0054](../../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md)
//! decided, so that task 6 turns a decided road into a tested command sequence
//! against a virtual disk rather than into a sequence somebody invented on the
//! day.
//!
//! # The one thing about the order that is a decision and not a detail
//!
//! **The recovery key is made, shown and typed back before the way the person
//! unlocks is enrolled.** Until they have confirmed it, the only thing that
//! opens the volume is the installer's own first key and the recovery key
//! itself — so an install abandoned at that moment is abandoned cleanly, and a
//! machine that reaches its first start is a machine whose recovery key is on
//! somebody's paper. The other order — enrol the PIN first, ask about the
//! recovery key after — makes *no recovery key* a thing that can happen to a
//! working machine, and it is the order every product that has lost somebody's
//! data uses.
//!
//! # What is not here
//!
//! No command, no argument, no program name, no device. The rented tools are
//! `cryptsetup` and `systemd-cryptenroll` and their invocations are task 6's;
//! naming them here would put an argument list in a crate whose whole claim is
//! that it cannot run anything.

/// One step of enrolling encryption on this machine's disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// The disk is made into an encrypted volume, with a first key that only
    /// the installer holds and that nobody is ever shown.
    TheDiskIsMadeIntoAVolume,
    /// The recovery key is made and enrolled — by the rented tool, which
    /// generates it; nothing of ours chooses a key.
    TheRecoveryKeyIsMade,
    /// It is put on the screen, once.
    TheRecoveryKeyIsShownOnce,
    /// The person types it back, which is the only confirmation there is.
    ThePersonTypesItBack,
    /// The way this person will open the machine — sealed to the chip with a
    /// PIN, or a passphrase — is enrolled.
    TheWayThePersonUnlocksIsEnrolled,
    /// The installer's first key is wiped out of the volume, so that what
    /// opens the disk afterwards is what the person has and nothing else.
    TheInstallersFirstKeyIsWipedAway,
}

/// The road, in the order it is taken, and there is no other.
pub const THE_ROAD: [Step; 6] = [
    Step::TheDiskIsMadeIntoAVolume,
    Step::TheRecoveryKeyIsMade,
    Step::TheRecoveryKeyIsShownOnce,
    Step::ThePersonTypesItBack,
    Step::TheWayThePersonUnlocksIsEnrolled,
    Step::TheInstallersFirstKeyIsWipedAway,
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Where a step comes on the road, or nowhere.
    fn where_it_comes(step: Step) -> Option<usize> {
        THE_ROAD.iter().position(|it| *it == step)
    }

    /// Every step is on the road, exactly once.
    #[test]
    fn every_step_is_on_the_road_once() {
        for step in [
            Step::TheDiskIsMadeIntoAVolume,
            Step::TheRecoveryKeyIsMade,
            Step::TheRecoveryKeyIsShownOnce,
            Step::ThePersonTypesItBack,
            Step::TheWayThePersonUnlocksIsEnrolled,
            Step::TheInstallersFirstKeyIsWipedAway,
        ] {
            assert_eq!(
                THE_ROAD.iter().filter(|it| **it == step).count(),
                1,
                "{step:?}"
            );
        }
    }

    /// **The recovery key is made, shown and typed back before anything the
    /// person will unlock with exists** — and the installer's own key is gone
    /// by the end.
    #[test]
    fn the_recovery_key_is_kept_before_the_way_it_unlocks_is_enrolled() {
        let made = where_it_comes(Step::TheRecoveryKeyIsMade);
        let shown = where_it_comes(Step::TheRecoveryKeyIsShownOnce);
        let typed = where_it_comes(Step::ThePersonTypesItBack);
        let enrolled = where_it_comes(Step::TheWayThePersonUnlocksIsEnrolled);
        assert!(made < shown, "{made:?} {shown:?}");
        assert!(shown < typed, "{shown:?} {typed:?}");
        assert!(typed < enrolled, "{typed:?} {enrolled:?}");
        assert_eq!(
            THE_ROAD.last(),
            Some(&Step::TheInstallersFirstKeyIsWipedAway)
        );
        assert_eq!(
            THE_ROAD.first(),
            Some(&Step::TheDiskIsMadeIntoAVolume),
            "there is nothing to enrol a key into before there is a volume"
        );
    }
}
