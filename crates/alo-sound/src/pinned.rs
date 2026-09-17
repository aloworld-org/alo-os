//! **The device a person meant, whenever it is here.**
//!
//! Pinning is the one thing this crate holds that the server has no view about.
//! WirePlumber picks a device by its own policy — most recently plugged in,
//! most likely to be wanted — and it is usually right. A pin is a person saying
//! *no, this one*, and it is worth having precisely on the days the policy is
//! wrong.
//!
//! # A pin is not a default
//!
//! It applies **when the device is here**, and says nothing when it is not.
//! Somebody who pinned their desk speakers and then took the laptop to a café
//! does not want silence; they want whatever is there, which is what the server
//! would have chosen anyway.

use serde::{Deserialize, Serialize};

use crate::device::{Devices, Identity, Kind, OneDevice};

/// **What a person pinned**, per kind: one output, one input.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pinned {
    /// The output they pinned, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    output: Option<Identity>,
    /// The input they pinned, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    input: Option<Identity>,
}

impl Pinned {
    /// Nothing pinned, which is where a machine starts.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// Pin this device for its kind, replacing whatever was pinned for it.
    #[must_use]
    pub fn and(mut self, device: &OneDevice) -> Self {
        match device.kind() {
            Kind::Output => self.output = Some(device.identity().clone()),
            Kind::Input => self.input = Some(device.identity().clone()),
        }
        self
    }

    /// Unpin whatever was pinned for this kind.
    #[must_use]
    pub fn without(mut self, kind: Kind) -> Self {
        match kind {
            Kind::Output => self.output = None,
            Kind::Input => self.input = None,
        }
        self
    }

    /// What is pinned for this kind, whether or not it is here.
    #[must_use]
    pub const fn for_kind(&self, kind: Kind) -> Option<&Identity> {
        match kind {
            Kind::Output => self.output.as_ref(),
            Kind::Input => self.input.as_ref(),
        }
    }

    /// **Which device to use for this kind, given what is here now.**
    ///
    /// The pinned one where it is present. [`None`] where it is not — **not an
    /// error and not silence**: the server's own policy chooses, which is what
    /// a person at a café wants without having to remember they pinned
    /// something at home.
    #[must_use]
    pub fn chosen_from<'a>(&self, devices: &'a Devices, kind: Kind) -> Option<&'a OneDevice> {
        let wanted = self.for_kind(kind)?;
        devices
            .of_kind(kind)
            .find(|device| device.identity() == wanted)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn a_device(name: &str, kind: Kind) -> OneDevice {
        OneDevice::reported(Identity::reported(name).expect("named"), name, kind)
    }

    /// **A pinned device is chosen when it is here.**
    #[test]
    fn the_pinned_device_is_chosen_while_it_is_here() {
        let speakers = a_device("desk-speakers", Kind::Output);
        let laptop = a_device("laptop-speakers", Kind::Output);
        let pinned = Pinned::nothing().and(&speakers);

        let both = Devices::reported(vec![laptop.clone(), speakers.clone()]);
        assert_eq!(
            pinned.chosen_from(&both, Kind::Output),
            Some(&speakers),
            "the pinned device is here and something else was chosen"
        );
    }

    /// **And says nothing when it is not here**, so the server chooses.
    #[test]
    fn a_pin_says_nothing_when_the_device_is_elsewhere() {
        let speakers = a_device("desk-speakers", Kind::Output);
        let pinned = Pinned::nothing().and(&speakers);
        let at_a_cafe = Devices::reported(vec![a_device("laptop-speakers", Kind::Output)]);

        assert_eq!(
            pinned.chosen_from(&at_a_cafe, Kind::Output),
            None,
            "a pin for a device that is not here would be silence"
        );
        assert!(
            pinned.for_kind(Kind::Output).is_some(),
            "the pin was forgotten because the device was away"
        );
    }

    /// **One pin per kind**, and pinning an input does not move the output.
    #[test]
    fn pinning_an_input_leaves_the_output_where_it_was() {
        let speakers = a_device("desk-speakers", Kind::Output);
        let headset = a_device("headset-mic", Kind::Input);
        let pinned = Pinned::nothing().and(&speakers).and(&headset);
        assert_eq!(
            pinned.for_kind(Kind::Output).map(Identity::as_str),
            Some("desk-speakers")
        );
        assert_eq!(
            pinned.for_kind(Kind::Input).map(Identity::as_str),
            Some("headset-mic")
        );

        let unpinned = pinned.without(Kind::Input);
        assert!(unpinned.for_kind(Kind::Input).is_none());
        assert!(unpinned.for_kind(Kind::Output).is_some());
    }
}
