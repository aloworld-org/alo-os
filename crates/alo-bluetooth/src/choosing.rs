//! **A device is paired with because a person chose it from what was found.**
//!
//! [`Chosen`] cannot be made out of an address somebody typed, out of a name, or
//! out of a device that asked to be paired. It is made from a [`Found`] taken
//! out of [`TheDevices`] the service reported at that moment — so the only road
//! to a pairing starts at a list a person was looking at.
//!
//! That is the whole of this file, and it is a type rather than a rule written
//! in a comment because *no automatic pairing of anything, including devices
//! that ask to be paired* is the plan's constraint, and a constraint that lives
//! in a comment is one the next change forgets.

use crate::reported::{Address, Found, TheDevices};

/// **A device a person picked out of what this machine found.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chosen {
    /// The device, as the service reported it when they picked it.
    device: Found,
}

impl Chosen {
    /// **The device a person picked**, from the list they were looking at.
    ///
    /// # Errors
    /// [`NotChosen`] where that address is not in the list — a device that has
    /// gone away since, or an address nothing found. Neither is paired with.
    pub fn from(devices: &TheDevices, address: &Address) -> Result<Self, NotChosen> {
        if !devices.radio().anything_may_connect() {
            return Err(NotChosen::TheRadioIsOff);
        }
        let device = devices
            .at(address)
            .ok_or_else(|| NotChosen::NotInWhatWasFound {
                address: address.as_str().to_owned(),
            })?;
        Ok(Self {
            device: device.clone(),
        })
    }

    /// The device.
    #[must_use]
    pub const fn device(&self) -> &Found {
        &self.device
    }

    /// Its address.
    #[must_use]
    pub const fn address(&self) -> &Address {
        self.device.address()
    }
}

/// Why nothing was paired with.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotChosen {
    /// That address is not among what was found.
    #[error(
        "{address} is not among the devices this machine found, and a device nobody was looking \
         at is not a device anybody chose"
    )]
    NotInWhatWasFound {
        /// The address that was asked for.
        address: String,
    },
    /// The radio is off.
    #[error("this machine's Bluetooth radio is off, and off means off")]
    TheRadioIsOff,
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::radio::Radio;
    use crate::reported::{DeviceName, Kind};

    fn a_headset() -> Found {
        let address = Address::reported("AA:BB:CC:DD:EE:FF").expect("an address");
        let called = DeviceName::reported("A headset", &address);
        Found::reported(address, called, Kind::Audio, false)
    }

    /// **Only what was found can be chosen.**
    #[test]
    fn an_address_nobody_was_shown_is_not_a_device_anybody_chose() {
        let found = TheDevices::reported(vec![a_headset()], Radio::On, true);
        let elsewhere = Address::reported("11:22:33:44:55:66").expect("an address");
        assert!(matches!(
            Chosen::from(&found, &elsewhere),
            Err(NotChosen::NotInWhatWasFound { .. })
        ));
        let chosen = Chosen::from(&found, a_headset().address()).expect("it was in the list");
        assert_eq!(chosen.device(), &a_headset());
    }

    /// **With the radio off, nothing is chosen at all** — including a device
    /// that was in a list a moment ago.
    #[test]
    fn with_the_radio_off_nothing_is_paired_with() {
        let found = TheDevices::reported(vec![a_headset()], Radio::Off, true);
        assert_eq!(
            Chosen::from(&found, a_headset().address()),
            Err(NotChosen::TheRadioIsOff)
        );
    }
}
