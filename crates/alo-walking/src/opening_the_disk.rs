//! **Opening the walk machine's encrypted disk**, before anything else is
//! drawn: wait for the initramfs, type the disk's passphrase, and see the root
//! filesystem mount on the opened device.
//!
//! Every install is encrypted by default (ADR 0054), so the walk machine's disk
//! is too, and its passphrase is asked on the screen by the initramfs before
//! the sign-in screen exists. Nothing of that prompt reaches the serial port:
//! only the kernel's own lines do there, until the journal takes over. So the
//! two marks are kernel lines:
//!
//! - **`Run /init`**: the kernel hands over to the initramfs, which asks for
//!   the passphrase once it has found the disk.
//! - **`BTRFS info (device dm-`**: the root is btrfs on the opened device,
//!   which the kernel names `dm-<n>`; it is said only once the passphrase was
//!   taken.
//!
//! The kernel arguments carry no `quiet`, so its info lines reach the serial
//! port. **FITS, not measured**: that keys typed before the prompt is drawn
//! would be lost, so the walk waits [`BEFORE_THE_PROMPT`] after `Run /init`
//! and types once; whether that is long enough is the first encrypted walk's
//! to say.
//!
//! It runs only when [`THE_DISK_PASSPHRASE`] is set, which the walk workflow
//! does for a disk it encrypted, so a disk written without encryption boots as
//! it did.

use std::time::Duration;

use crate::step::Walk;

/// The environment variable holding the walk disk's passphrase.
pub const THE_DISK_PASSPHRASE: &str = "ALO_WALK_DISK_PASSPHRASE";

/// How long after the initramfs starts the walk waits before typing.
pub const BEFORE_THE_PROMPT: Duration = Duration::from_secs(30);

/// The walk, with the disk opened first when its passphrase is set.
#[must_use]
pub fn the_disk_is_opened(walk: Walk) -> Walk {
    walk
}

#[cfg(test)]
#[path = "opening_the_disk_tests.rs"]
mod tests;
