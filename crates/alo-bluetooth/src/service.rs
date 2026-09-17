//! **Asking the rented Bluetooth service what there is, and telling it the four
//! things a person can do about it.**
//!
//! A trait rather than the client itself, so that every decision this crate
//! makes is tested against every shape of answer on any machine, and so that the
//! machine with a radio in it is the only place that needs one. `crate::bluez`
//! is the client a machine runs.
//!
//! The refusals carry English for a service log, never for a person: what a
//! person reads is in `crate::words`.

use crate::choosing::Chosen;
use crate::radio::Radio;
use crate::reported::{Address, TheDevices};

/// The Bluetooth service could not be asked.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAnswering {
    /// Nothing on this machine answers for Bluetooth.
    #[error("nothing on this machine answers for Bluetooth: {0}")]
    NothingAnswers(String),
    /// It is there and it would not answer.
    #[error("this machine's Bluetooth service would not answer: {0}")]
    NoAnswer(String),
    /// It answered something this crate cannot read.
    #[error("this machine's Bluetooth service answered something this crate cannot read: {0}")]
    NotUnderstood(String),
}

/// The Bluetooth service did not do it.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotDone {
    /// Nobody said yes, so nothing was attempted.
    #[error(
        "nobody confirmed the pairing, and a device pairs with this machine only when a \
             person says yes to what they were shown"
    )]
    NobodySaidYes,
    /// The service could not be asked first.
    #[error(transparent)]
    NotAnswering(#[from] NotAnswering),
    /// Nothing was paired with, because nothing was chosen.
    #[error(transparent)]
    NotChosen(#[from] crate::choosing::NotChosen),
    /// It was told and it refused.
    #[error("this machine's Bluetooth service refused to {doing}: {said}")]
    Refused {
        /// What it was asked to do.
        doing: String,
        /// What it said.
        said: String,
    },
}

/// **What this machine's Bluetooth is doing**, asked.
pub trait Devices {
    /// Everything the service reports at this moment, including whether this
    /// machine has a radio at all.
    ///
    /// # Errors
    /// [`NotAnswering`]. Never an empty list standing in for a question that
    /// could not be asked.
    fn now(&self) -> Result<TheDevices, NotAnswering>;
}

/// **And the four things a person can do about it.**
pub trait BluetoothService: Devices {
    /// **Pair with a device a person chose.**
    ///
    /// What the device asks for on the way — six digits to compare, six to type
    /// on it, a code printed on its underside — is asked of the person by
    /// `crate::pairing_agent` while this is running, and nothing is answered on
    /// their behalf. A pairing a person declined comes back as
    /// [`NotDone::NobodySaidYes`].
    ///
    /// # Errors
    /// [`NotDone`].
    fn pair(&self, chosen: &Chosen) -> Result<(), NotDone>;

    /// **Forget a device**, which removes its keys from this machine.
    ///
    /// One act: not *disconnect*, not *disconnect and then forget when you find
    /// the other switch*. After it, this machine holds nothing about the device
    /// and the device holds nothing this machine will answer.
    ///
    /// # Errors
    /// [`NotDone`].
    fn forget(&self, address: &Address) -> Result<(), NotDone>;

    /// **Turn the radio on, or off** — and off means off for every device.
    ///
    /// # Errors
    /// [`NotDone`].
    fn switch_radio(&self, to: Radio) -> Result<(), NotDone>;

    /// **Look for devices**, or stop looking.
    ///
    /// Looking is deliberate and it ends: a machine that is permanently
    /// discoverable and permanently looking is a machine announcing itself to a
    /// room it cannot see.
    ///
    /// # Errors
    /// [`NotDone`].
    fn look(&self, looking: bool) -> Result<(), NotDone>;
}

/// **Look for devices**, which takes a radio that is on.
///
/// The check is here rather than only in the client because *off means off* is
/// a promise rather than a detail of one machine's service: a radio that is off
/// is not asked to look, and a machine that asked would be announcing itself to
/// a room with the switch in the off position.
///
/// # Errors
/// [`NotDone::Refused`] where the radio is off, or the service would not.
pub fn look_for_devices(
    service: &impl BluetoothService,
    devices: &TheDevices,
    looking: bool,
) -> Result<(), NotDone> {
    if looking && !devices.radio().anything_may_connect() {
        return Err(NotDone::Refused {
            doing: "look for devices".to_owned(),
            said: "the radio is off, and off means off".to_owned(),
        });
    }
    service.look(looking)
}

/// **Pair with a device, from the list a person was looking at.**
///
/// The road this crate exists to make the only one: what the service found, the
/// device a person picked out of it, and then the service — which asks the
/// person whatever the device requires, through the agent, and pairs only if
/// they say yes.
///
/// # Errors
/// [`NotDone`] where the radio is off, nobody said yes, or the service refused.
pub fn pair_with(
    service: &impl BluetoothService,
    devices: &TheDevices,
    address: &Address,
) -> Result<Chosen, NotDone> {
    let chosen = Chosen::from(devices, address)?;
    service.pair(&chosen)?;
    Ok(chosen)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::reported::Kind;
    use crate::testing::{AService, Told, a_device};

    /// **A radio that is off is not asked to look**, and stopping is always
    /// allowed: a person turning the radio off while this machine is looking
    /// must not be refused the *stop*.
    #[test]
    fn nothing_looks_for_devices_while_the_radio_is_off() {
        let service = AService::with(vec![a_device(
            "AA:BB:CC:DD:EE:01",
            "A headset",
            Kind::Audio,
            false,
        )])
        .with_the_radio_off();
        let devices = service.now().expect("the service answered");

        assert!(matches!(
            look_for_devices(&service, &devices, true),
            Err(NotDone::Refused { .. })
        ));
        assert!(service.told().is_empty());

        look_for_devices(&service, &devices, false).expect("stopping is always allowed");
        assert_eq!(service.told(), [Told::Look(false)]);
    }

    /// **And with it on, looking is one act and it ends.**
    #[test]
    fn looking_is_started_and_stopped_and_nothing_else_happens() {
        let service = AService::with(Vec::new());
        let devices = service.now().expect("the service answered");
        look_for_devices(&service, &devices, true).expect("the radio is on");
        look_for_devices(&service, &devices, false).expect("and it stops");
        assert_eq!(service.told(), [Told::Look(true), Told::Look(false)]);
    }
}
