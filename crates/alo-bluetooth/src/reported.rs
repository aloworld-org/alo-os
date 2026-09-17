//! What the rented Bluetooth service reported, and nothing it did not.
//!
//! Every value here is the service's own answer, held as a type rather than a
//! string somebody can typo. BlueZ is rented and unmodified (ADR 0011): this
//! crate reads what it says and decides what alo OS does about it.

use serde::{Deserialize, Serialize};

/// How long an address is when it is written out: `AA:BB:CC:DD:EE:FF`.
pub const AN_ADDRESS_IS: usize = 17;

/// **A device's address**, which is what it is across everything else changing.
///
/// A Bluetooth device's name is whatever its firmware says this week, and two
/// headsets in an office can say the same thing. The address is what a pairing
/// is kept under and what *forget this one* means.
///
/// **It is not an identity of a person.** A device that advertises a random
/// address advertises a random address, and this crate does not try to see
/// through that.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Address(String);

impl Address {
    /// The address as the service reported it.
    ///
    /// # Errors
    /// [`NotADevice::NotAnAddress`] for anything that is not six pairs of
    /// hexadecimal digits: an address this crate cannot read is one it cannot
    /// forget either, and a pairing that cannot be undone is not a pairing.
    pub fn reported(written: &str) -> Result<Self, NotADevice> {
        let written = written.trim().to_ascii_uppercase();
        let pairs: Vec<&str> = written.split(':').collect();
        let is_an_address = written.len() == AN_ADDRESS_IS
            && pairs.len() == 6
            && pairs
                .iter()
                .all(|pair| pair.len() == 2 && pair.bytes().all(|digit| digit.is_ascii_hexdigit()));
        if !is_an_address {
            return Err(NotADevice::NotAnAddress { written });
        }
        Ok(Self(written))
    }

    /// The address, written out.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// **What a device says it is called.**
///
/// Kept apart from [`Address`] because it is the device's claim rather than a
/// fact: it is what a person is shown, and it is never what anything is decided
/// by. A headset calling itself by another headset's name is still its own
/// address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeviceName(String);

impl DeviceName {
    /// The name the service reported, or the address where it reported none —
    /// because a row in a list of things to pair with cannot be blank.
    #[must_use]
    pub fn reported(said: &str, address: &Address) -> Self {
        let said = said.trim();
        if said.is_empty() {
            return Self(address.as_str().to_owned());
        }
        Self(said.to_owned())
    }

    /// What a person is shown.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// **What a paired device becomes on this machine.**
///
/// The kind is read from what the device says it is, and it decides what the
/// device is *for* — not what it is allowed to do, which is a different word
/// this crate is careful never to use (see [`crate::granting`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// Sound comes out of it, or goes into it: it joins `alo-sound`'s devices.
    Audio,
    /// It types.
    Keyboard,
    /// It moves a pointer.
    Mouse,
    /// Something else, which is most things and is not a failure.
    Other,
}

impl Kind {
    /// The major class in what Bluetooth calls a device's *class of device*.
    const MAJOR: u32 = 0x0000_1F00;
    /// The minor class, within the major.
    const MINOR: u32 = 0x0000_00FC;
    /// A major class of audio or video.
    const AUDIO: u32 = 0x0000_0400;
    /// A major class of peripheral — keyboards, mice, and the rest.
    const PERIPHERAL: u32 = 0x0000_0500;
    /// The peripheral minor bit that says *it types*.
    const TYPES: u32 = 0x0000_0040;
    /// The peripheral minor bit that says *it points*.
    const POINTS: u32 = 0x0000_0080;

    /// **What this device is**, from the class it reports.
    ///
    /// A device that reports nothing useful is [`Kind::Other`], which is an
    /// honest answer and the common one: it is a thing this machine has paired
    /// with and has no particular view about.
    #[must_use]
    pub const fn from_class(class: u32) -> Self {
        match class & Self::MAJOR {
            Self::AUDIO => Self::Audio,
            Self::PERIPHERAL => match class & Self::MINOR {
                minor if minor & Self::TYPES != 0 => Self::Keyboard,
                minor if minor & Self::POINTS != 0 => Self::Mouse,
                _ => Self::Other,
            },
            _ => Self::Other,
        }
    }

    /// Whether a device of this kind joins the machine's sound devices.
    #[must_use]
    pub const fn is_sound(self) -> bool {
        matches!(self, Self::Audio)
    }

    /// Whether a device of this kind can type or point, which is the pair a
    /// person should be told about twice: something that types is something
    /// that can type into anything.
    #[must_use]
    pub const fn is_hands(self) -> bool {
        matches!(self, Self::Keyboard | Self::Mouse)
    }
}

/// **One device the service has seen.**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Found {
    /// What it is.
    address: Address,
    /// What it says it is called.
    called: DeviceName,
    /// What it is for.
    kind: Kind,
    /// Whether this machine has already paired with it.
    paired: bool,
}

impl Found {
    /// A device as the service reported it.
    #[must_use]
    pub const fn reported(address: Address, called: DeviceName, kind: Kind, paired: bool) -> Self {
        Self {
            address,
            called,
            kind,
            paired,
        }
    }

    /// What it is.
    #[must_use]
    pub const fn address(&self) -> &Address {
        &self.address
    }

    /// What a person is shown.
    #[must_use]
    pub const fn called(&self) -> &DeviceName {
        &self.called
    }

    /// What it is for.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// Whether this machine has paired with it already.
    #[must_use]
    pub const fn is_paired(&self) -> bool {
        self.paired
    }
}

/// **Everything the service reports at one moment**, and whether this machine
/// has a radio at all.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TheDevices {
    /// What it found, in the order it listed them.
    found: Vec<Found>,
    /// Whether the radio is on.
    radio: crate::radio::Radio,
    /// Whether this machine has a Bluetooth radio at all.
    has_a_radio: bool,
}

impl TheDevices {
    /// What the service reported.
    #[must_use]
    pub const fn reported(
        found: Vec<Found>,
        radio: crate::radio::Radio,
        has_a_radio: bool,
    ) -> Self {
        Self {
            found,
            radio,
            has_a_radio,
        }
    }

    /// **A machine with no Bluetooth radio**, which is a real machine and says
    /// so rather than showing an empty list of things to pair with.
    #[must_use]
    pub fn without_a_radio() -> Self {
        Self {
            found: Vec::new(),
            radio: crate::radio::Radio::Off,
            has_a_radio: false,
        }
    }

    /// Everything found, in the service's own order.
    #[must_use]
    pub fn every(&self) -> &[Found] {
        &self.found
    }

    /// The ones this machine has paired with.
    pub fn paired(&self) -> impl Iterator<Item = &Found> {
        self.found.iter().filter(|device| device.is_paired())
    }

    /// The device at this address, if the service listed one.
    #[must_use]
    pub fn at(&self, address: &Address) -> Option<&Found> {
        self.found.iter().find(|device| device.address() == address)
    }

    /// Whether the radio is on.
    #[must_use]
    pub const fn radio(&self) -> crate::radio::Radio {
        self.radio
    }

    /// Whether this machine has a Bluetooth radio at all.
    #[must_use]
    pub const fn has_a_radio(&self) -> bool {
        self.has_a_radio
    }
}

/// Why something is not a device this crate will hold.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotADevice {
    /// Not six pairs of hexadecimal digits.
    #[error(
        "{written} is not a Bluetooth address, and a device this machine cannot address is one it \
         could not forget either"
    )]
    NotAnAddress {
        /// What was given.
        written: String,
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
    fn an_address_is_six_pairs_and_anything_else_is_refused() {
        assert_eq!(
            Address::reported("aa:bb:cc:dd:ee:ff")
                .expect("an address")
                .as_str(),
            "AA:BB:CC:DD:EE:FF",
            "an address is held in one case so that two spellings are one device"
        );
        for not in [
            "",
            "AA:BB:CC:DD:EE",
            "AA-BB-CC-DD-EE-FF",
            "ZZ:BB:CC:DD:EE:FF",
        ] {
            assert!(
                matches!(Address::reported(not), Err(NotADevice::NotAnAddress { .. })),
                "{not} was taken for an address"
            );
        }
    }

    /// **A device with no name is shown by its address**, never as a blank row
    /// in a list a person is about to choose from.
    #[test]
    fn a_device_that_said_no_name_is_shown_by_its_address() {
        let address = Address::reported("AA:BB:CC:DD:EE:FF").expect("an address");
        assert_eq!(
            DeviceName::reported("   ", &address).as_str(),
            "AA:BB:CC:DD:EE:FF"
        );
        assert_eq!(
            DeviceName::reported(" Headset ", &address).as_str(),
            "Headset"
        );
    }

    /// **What a device is, is read from what it reports.**
    #[test]
    fn what_a_device_is_comes_from_the_class_it_reports() {
        // A headset: audio/video major, headset minor.
        assert_eq!(Kind::from_class(0x0024_0404), Kind::Audio);
        // A keyboard, and a mouse, both peripherals.
        assert_eq!(Kind::from_class(0x0000_0540), Kind::Keyboard);
        assert_eq!(Kind::from_class(0x0000_0580), Kind::Mouse);
        // A keyboard-and-mouse combination types, which is the half that
        // matters to tell somebody about.
        assert_eq!(Kind::from_class(0x0000_05C0), Kind::Keyboard);
        // A phone, and a device that reported nothing.
        assert_eq!(Kind::from_class(0x0020_0204), Kind::Other);
        assert_eq!(Kind::from_class(0), Kind::Other);

        assert!(Kind::Audio.is_sound());
        assert!(!Kind::Keyboard.is_sound());
        assert!(Kind::Keyboard.is_hands() && Kind::Mouse.is_hands());
        assert!(!Kind::Other.is_hands());
    }

    /// **A machine with no radio says so**, rather than reading as a machine
    /// that found nothing.
    #[test]
    fn a_machine_with_no_radio_is_not_a_machine_that_found_nothing() {
        let none = TheDevices::without_a_radio();
        assert!(!none.has_a_radio());
        assert!(none.every().is_empty());

        let has_one = TheDevices::reported(Vec::new(), crate::radio::Radio::On, true);
        assert!(has_one.has_a_radio());
        assert!(has_one.every().is_empty());
        assert_ne!(
            none.has_a_radio(),
            has_one.has_a_radio(),
            "no radio and nothing found must never be the same answer"
        );
    }
}
