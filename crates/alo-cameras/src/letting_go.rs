//! **Off, held below the door: the machine lets go of the device.**
//!
//! A switch that is only checked where grants are checked is a switch that
//! covers the programs that ask politely. This is the other half, and it is the
//! half that makes *off* a word worth using: where the machine can, it tells the
//! kernel to let go of the camera, and then there is **nothing to open** — for an
//! application with a grant, for one without, and for anything that never asked
//! anybody.
//!
//! # The device number is a road, never an identity
//!
//! `/dev/video0` is what the kernel called this camera today, and
//! [`crate::CameraId`] is deliberately not it (ADR 0040). But a road to the
//! hardware is exactly what a device file is, and this is the one place that
//! uses it as one: the media server says which device file this camera is behind,
//! the machine's own `/sys` says which driver holds it, and the driver is told to
//! let it go. Nothing is remembered under a number; the number is read, used, and
//! dropped.
//!
//! # Where the machine cannot, it says so
//!
//! A camera on a driver that holds several devices at once cannot be let go of
//! by itself, and a microphone usually sits on a card that also plays sound —
//! letting go of it would take a person's speakers with it. [`NotLetGo`] says
//! which, and the switch stays a door rather than becoming a lie.

use std::path::{Path, PathBuf};

/// Where the kernel lists the machine's video devices.
pub const WHERE_VIDEO_DEVICES_ARE: &str = "/sys/class/video4linux";

/// **What happened when the machine was told to let go.**
///
/// It carries where the driver is as well as what the device is called, because
/// the road that found them — the device file — is gone by the time this exists,
/// and a machine that cannot give a camera back has not turned it off, it has
/// broken it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetGo {
    /// Where the driver that was holding it is.
    under: PathBuf,
    /// What the driver calls the device it let go of.
    device: String,
}

impl LetGo {
    /// The driver that was holding it.
    #[must_use]
    pub fn driver(&self) -> &str {
        self.under
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    }

    /// Where that driver is.
    #[must_use]
    pub fn under(&self) -> &Path {
        &self.under
    }

    /// What the driver calls it.
    #[must_use]
    pub fn device(&self) -> &str {
        &self.device
    }
}

/// Why the machine could not let go of a device.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotLetGo {
    /// There is no such device file on this machine.
    #[error("{device_file} is not a device on this machine")]
    NoSuchDevice {
        /// What was looked for.
        device_file: String,
    },
    /// The kernel does not say which driver holds it.
    #[error(
        "this machine does not say which driver holds {device_file}, so it cannot be told to let \
         go of it"
    )]
    NoDriverHoldsIt {
        /// What was looked for.
        device_file: String,
    },
    /// The machine would not do it.
    #[error("this machine would not let go of {device_file}: {said}")]
    Refused {
        /// What was asked for.
        device_file: String,
        /// What it said.
        said: String,
    },
}

/// Where a device file's driver is, and what that driver calls the device.
///
/// # Errors
/// [`NotLetGo`] where the device is not there or nothing says who holds it.
pub fn who_holds(device_file: &str) -> Result<(PathBuf, String), NotLetGo> {
    let named = Path::new(device_file)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| NotLetGo::NoSuchDevice {
            device_file: device_file.to_owned(),
        })?;
    let under = Path::new(WHERE_VIDEO_DEVICES_ARE)
        .join(named)
        .join("device");
    let device = std::fs::canonicalize(&under).map_err(|_| NotLetGo::NoSuchDevice {
        device_file: device_file.to_owned(),
    })?;
    let driver =
        std::fs::canonicalize(under.join("driver")).map_err(|_| NotLetGo::NoDriverHoldsIt {
            device_file: device_file.to_owned(),
        })?;
    let called = device
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| NotLetGo::NoDriverHoldsIt {
            device_file: device_file.to_owned(),
        })?
        .to_owned();
    Ok((driver, called))
}

/// **Tell the kernel to let go of this camera.**
///
/// After it, the device file is gone and nothing on this machine can open it —
/// which is what a person means by *off*.
///
/// # Errors
/// [`NotLetGo`], including on a machine where this is not something a program
/// may do.
pub fn let_go_of(device_file: &str) -> Result<LetGo, NotLetGo> {
    // Who holds it is read **before** it is let go of, and not after: the
    // moment the driver lets go, the device file is gone and so is every road
    // through it. The first version of this asked afterwards, found nothing,
    // and left a machine with no camera and no way back to one.
    let (under, device) = who_holds(device_file)?;
    tell(&under, "unbind", &device, device_file)?;
    Ok(LetGo { under, device })
}

/// **Give it back**, when a person turns the switch on again.
///
/// # Errors
/// [`NotLetGo`].
pub fn take_back(let_go: &LetGo) -> Result<(), NotLetGo> {
    tell(&let_go.under, "bind", &let_go.device, &let_go.device)
}

/// Write a device's name into one of a driver's own doors.
fn tell(driver: &Path, door: &str, device: &str, about: &str) -> Result<(), NotLetGo> {
    std::fs::write(driver.join(door), device).map_err(|why| NotLetGo::Refused {
        device_file: about.to_owned(),
        said: why.to_string(),
    })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A device that is not there is said to be not there**, rather than
    /// reported as let go of.
    #[test]
    fn a_device_that_is_not_here_cannot_be_let_go_of() {
        let why = let_go_of("/dev/video-that-is-not-here").expect_err("there is no such device");
        assert!(matches!(why, NotLetGo::NoSuchDevice { .. }));
        assert!(!why.to_string().is_empty());
    }

    /// **A path that is not a device file at all is refused the same way.**
    #[test]
    fn something_that_is_not_a_device_file_is_refused() {
        assert!(matches!(who_holds("/"), Err(NotLetGo::NoSuchDevice { .. })));
    }
}
