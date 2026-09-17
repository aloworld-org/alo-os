//! A camera, as the machine reports it and as a person recognises it.

use serde::{Deserialize, Serialize};

/// **What a camera is, across a reboot and a replug.**
///
/// Never `/dev/video0`.
///
/// The number the kernel hands a camera is the order the machine happened to
/// find it in: plug in a second camera, or boot on a colder morning, and
/// yesterday's `video0` is today's `video2`. A grant made against a number is a
/// grant that quietly moves to another device, which is the exact failure
/// [ADR 0040](../../../docs/decisions/0040-what-an-applications-grant-is-over.md)
/// is about — a person granting *the camera* and meaning the one above their
/// screen.
///
/// So the identity is what the device says about itself, by way of the media
/// server: which port it is plugged into and what it calls itself. It is the
/// same shape `alo-sound` keeps its devices under, and for the same reason.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CameraId(String);

impl CameraId {
    /// The identity the media server reports for this camera.
    ///
    /// # Errors
    /// [`NotACamera::Unnamed`] where the server said nothing, and
    /// [`NotACamera::ANumber`] where what it said is a device file: a camera
    /// held under a number cannot be granted, remembered, or told from the next
    /// one plugged in.
    pub fn reported(named: &str) -> Result<Self, NotACamera> {
        let named = named.trim();
        if named.is_empty() {
            return Err(NotACamera::Unnamed);
        }
        if named.starts_with("/dev/") {
            return Err(NotACamera::ANumber {
                written: named.to_owned(),
            });
        }
        Ok(Self(named.to_owned()))
    }

    /// What it is.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// **Whether a camera has a shutter or a light of its own.**
///
/// Reported honestly, which mostly means saying that this machine cannot tell.
/// A camera's light is wired to its firmware; a machine can neither turn it on
/// nor read it, and a product that claimed otherwise would be claiming the one
/// thing a person is entitled to check for themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OfItsOwn {
    /// The device says it has one.
    ItHasOne,
    /// The device says it has none.
    ItHasNone,
    /// The device says nothing about it, and neither does this machine.
    ThisMachineCannotTell,
}

/// **One camera this machine has.**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OneCamera {
    /// What it is.
    identity: CameraId,
    /// What a person sees.
    called: String,
    /// Whether it has a shutter a person can close by hand.
    shutter: OfItsOwn,
    /// Whether it has a light that comes on when it is in use.
    light: OfItsOwn,
}

impl OneCamera {
    /// A camera the machine reported.
    #[must_use]
    pub fn reported(identity: CameraId, called: &str, shutter: OfItsOwn, light: OfItsOwn) -> Self {
        Self {
            identity,
            called: called.trim().to_owned(),
            shutter,
            light,
        }
    }

    /// What it is.
    #[must_use]
    pub const fn identity(&self) -> &CameraId {
        &self.identity
    }

    /// What a person sees. Empty where the machine had no friendly name, in
    /// which case a surface shows the identity rather than inventing one.
    #[must_use]
    pub fn called(&self) -> &str {
        &self.called
    }

    /// Whether it has a shutter of its own.
    #[must_use]
    pub const fn shutter(&self) -> OfItsOwn {
        self.shutter
    }

    /// Whether it has a light of its own.
    #[must_use]
    pub const fn light(&self) -> OfItsOwn {
        self.light
    }
}

/// **Every camera this machine has now.**
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TheCameras {
    /// In the order the machine listed them.
    all: Vec<OneCamera>,
}

impl TheCameras {
    /// The cameras the machine reported.
    #[must_use]
    pub const fn reported(all: Vec<OneCamera>) -> Self {
        Self { all }
    }

    /// Everything, in the machine's own order.
    #[must_use]
    pub fn every(&self) -> &[OneCamera] {
        &self.all
    }

    /// Whether this camera is here now.
    #[must_use]
    pub fn holds(&self, identity: &CameraId) -> bool {
        self.all.iter().any(|one| one.identity() == identity)
    }
}

/// Why something is not a camera this crate will hold.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotACamera {
    /// The machine named nothing.
    #[error(
        "a camera the machine did not name: without an identity it cannot be granted or told \
             from the next one plugged in"
    )]
    Unnamed,
    /// A device file rather than an identity.
    #[error(
        "{written} is the number this machine happened to give a camera today, not what the \
         camera is: a grant made against it would move to whatever is there tomorrow"
    )]
    ANumber {
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

    /// **A camera is never held under a device number.**
    #[test]
    fn a_device_number_is_refused_as_an_identity() {
        assert!(matches!(
            CameraId::reported("/dev/video0"),
            Err(NotACamera::ANumber { .. })
        ));
        assert_eq!(CameraId::reported("  "), Err(NotACamera::Unnamed));
        assert_eq!(
            CameraId::reported("v4l2_input.pci-0000_00_14.0-usb-0_1_1.0")
                .expect("an identity")
                .as_str(),
            "v4l2_input.pci-0000_00_14.0-usb-0_1_1.0"
        );
    }

    /// **What this machine cannot tell, it says it cannot tell.**
    #[test]
    fn a_camera_says_what_it_has_and_this_machine_does_not_guess() {
        let camera = OneCamera::reported(
            CameraId::reported("v4l2_input.one").expect("an identity"),
            " The camera above the screen ",
            OfItsOwn::ItHasNone,
            OfItsOwn::ThisMachineCannotTell,
        );
        assert_eq!(camera.called(), "The camera above the screen");
        assert_eq!(camera.shutter(), OfItsOwn::ItHasNone);
        assert_eq!(
            camera.light(),
            OfItsOwn::ThisMachineCannotTell,
            "a machine claiming to know about a camera's light is claiming the one thing a person \
             can check for themselves"
        );

        let cameras = TheCameras::reported(vec![camera.clone()]);
        assert!(cameras.holds(camera.identity()));
        assert_eq!(cameras.every().len(), 1);
    }
}
