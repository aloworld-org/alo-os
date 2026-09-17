//! A device, as the server reports it and as a person recognises it.

use serde::{Deserialize, Serialize};

/// **What a device is for.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// Sound comes out of it.
    Output,
    /// Sound goes into it.
    Input,
}

/// **What a device is, across being unplugged and plugged in again.**
///
/// The server's own numbers change when a cable moves. What does not is what
/// the device says about itself — the name the server derives from the hardware
/// — so that is what a person's volume, mute and pin are kept under.
///
/// A device that came back as something new would drop all three every time
/// somebody moved a cable, and **a pin that forgets is worse than no pin**: a
/// person stops checking, and then one day the call goes to the laptop
/// speakers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Identity(String);

impl Identity {
    /// The identity the server reports for this device.
    ///
    /// # Errors
    /// [`NotADevice::Unnamed`] where the server said nothing: a device with no
    /// identity cannot be pinned, remembered or told apart from the next one to
    /// arrive, and pretending otherwise would attach somebody's settings to
    /// whatever happened to be plugged in.
    pub fn reported(named: &str) -> Result<Self, NotADevice> {
        let named = named.trim();
        if named.is_empty() {
            return Err(NotADevice::Unnamed);
        }
        Ok(Self(named.to_owned()))
    }

    /// What it is called.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// **How loud a device is**, in hundredths, so a file holds a whole number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Volume(u16);

impl Volume {
    /// The loudest this crate will set.
    pub const LOUDEST: u16 = 100;

    /// A volume a person chose.
    ///
    /// # Errors
    /// [`NotADevice::LouderThanLoud`] above a hundred. Some servers amplify
    /// past it; doing that on somebody's behalf is how a machine damages a pair
    /// of headphones and a pair of ears.
    pub const fn of(hundredths: u16) -> Result<Self, NotADevice> {
        if hundredths > Self::LOUDEST {
            return Err(NotADevice::LouderThanLoud {
                asked_for: hundredths,
            });
        }
        Ok(Self(hundredths))
    }

    /// As loud as this crate will set, which is where a device with no volume
    /// control of its own sits.
    #[must_use]
    pub const fn loudest() -> Self {
        Self(Self::LOUDEST)
    }

    /// How loud it is.
    #[must_use]
    pub const fn hundredths(self) -> u16 {
        self.0
    }
}

/// **One device this machine has.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneDevice {
    /// What it is, across a replug.
    identity: Identity,
    /// What a person sees.
    called: String,
    /// What it is for.
    kind: Kind,
}

impl OneDevice {
    /// A device the server reported.
    #[must_use]
    pub fn reported(identity: Identity, called: &str, kind: Kind) -> Self {
        Self {
            identity,
            called: called.trim().to_owned(),
            kind,
        }
    }

    /// What it is, across a replug.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// What a person sees. Empty where the server gave no friendly name, in
    /// which case a surface shows the identity rather than inventing one.
    #[must_use]
    pub fn called(&self) -> &str {
        &self.called
    }

    /// What it is for.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }
}

/// **Everything the server reports right now.**
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Devices {
    /// The devices, in the order the server listed them.
    all: Vec<OneDevice>,
}

impl Devices {
    /// The devices the server reported.
    #[must_use]
    pub fn reported(all: Vec<OneDevice>) -> Self {
        Self { all }
    }

    /// Everything, in the server's own order.
    #[must_use]
    pub fn all(&self) -> &[OneDevice] {
        &self.all
    }

    /// Only the outputs, or only the inputs.
    pub fn of_kind(&self, kind: Kind) -> impl Iterator<Item = &OneDevice> {
        self.all.iter().filter(move |device| device.kind() == kind)
    }

    /// Whether this device is here now.
    #[must_use]
    pub fn holds(&self, identity: &Identity) -> bool {
        self.all.iter().any(|device| device.identity() == identity)
    }
}

/// Why something is not a device this crate will hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotADevice {
    /// The server named nothing.
    #[error(
        "a device the server did not name: without an identity it cannot be pinned, remembered \
         or told from the next one plugged in"
    )]
    Unnamed,
    /// Louder than this crate will set.
    #[error(
        "{asked_for} hundredths is louder than loud, and amplifying past it is how a machine \
             damages headphones and ears"
    )]
    LouderThanLoud {
        /// What was asked for.
        asked_for: u16,
    },
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    #[test]
    fn a_device_the_server_did_not_name_is_refused() {
        assert_eq!(Identity::reported("  "), Err(NotADevice::Unnamed));
        assert_eq!(
            Identity::reported("alsa_output.usb-Headset")
                .expect("a named device")
                .as_str(),
            "alsa_output.usb-Headset"
        );
    }

    #[test]
    fn a_volume_louder_than_loud_is_refused() {
        assert!(matches!(
            Volume::of(150),
            Err(NotADevice::LouderThanLoud { asked_for: 150 })
        ));
        assert_eq!(Volume::of(Volume::LOUDEST).expect("loud").hundredths(), 100);
    }

    #[test]
    fn devices_are_told_apart_by_what_they_are_for() {
        let devices = Devices::reported(vec![
            OneDevice::reported(
                Identity::reported("out-1").expect("named"),
                "Desk speakers",
                Kind::Output,
            ),
            OneDevice::reported(
                Identity::reported("in-1").expect("named"),
                "Headset microphone",
                Kind::Input,
            ),
        ]);
        assert_eq!(devices.of_kind(Kind::Output).count(), 1);
        assert_eq!(devices.of_kind(Kind::Input).count(), 1);
        assert!(devices.holds(&Identity::reported("out-1").expect("named")));
        assert!(!devices.holds(&Identity::reported("out-2").expect("named")));
    }
}
