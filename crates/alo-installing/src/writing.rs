//! Writing alo OS onto the chosen disk.
//!
//! `bootc install to-disk`, the tool inside the base the image is built on, and
//! the one `docs/booting.md` already gives — with two differences that are both
//! about where this runs.
//!
//! **`--source-imgref registry:…`.** The documented invocation runs *inside* the
//! image it installs, which needs the whole image in a container store first.
//! This environment lives in memory, and a 6.6 GB image unpacked into memory is
//! a machine that needs more of it than a certified laptop may have. Given a
//! source, the tool pulls the release straight into the new disk's own store
//! instead — measured in a virtual machine with 3 GB of memory on 2026-09-15.
//!
//! **The chosen disk by its own name**, `/dev/disk/by-id/…`, never the kernel's
//! `/dev/sdX` for it, so the disk written is the one the person named even if
//! the machine enumerated its disks in a different order on this start.
//!
//! The source is the pinned release **by digest**, the same reference
//! `crate::verifying` checked. Nothing else is passed about what the installed
//! machine follows afterwards: the tool records the reference it installed, and
//! what the machine updates to next is the updates plan's decision to make by
//! digest, not this environment's.
//!
//! **`--filesystem btrfs`**, which is the one argument here that a person can
//! never take back.
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) rewinds
//! *undo what the agent did* from the base's own read-only snapshot of a
//! person's home, and a snapshot needs a filesystem that has them. **A
//! filesystem is chosen at install and cannot be converted afterwards**, so a
//! machine installed on `ext4` — which has no subvolume and no snapshot — could
//! never undo anything without being reinstalled. The value is
//! [`alo_image::THE_ONLY_FILESYSTEM`] rather than a second spelling here,
//! because it is also written in `docs/booting.md`, and two spellings of a
//! decision that cannot be undone is one too many. What the base makes of it is
//! measured in `docs/quirks.md` rather than assumed.

use alo_image::{THE_ONLY_FILESYSTEM, ThePin};

use crate::disk::DiskName;

/// The write, with everything it is given already checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Writing {
    /// The release, by digest.
    reference: String,
    /// The disk, by its own name.
    disk: DiskName,
}

impl Writing {
    /// The pinned release onto this disk.
    #[must_use]
    pub fn of(pin: &ThePin, disk: &DiskName) -> Self {
        Self {
            reference: pin.reference(),
            disk: disk.clone(),
        }
    }

    /// The disk it writes.
    #[must_use]
    pub fn disk(&self) -> &DiskName {
        &self.disk
    }

    /// The writer's arguments.
    ///
    /// `--wipe` because the person agreed to replace this disk, and a disk with
    /// a table on it is otherwise refused by the tool — which is the tool asking
    /// the question the person already answered.
    #[must_use]
    pub fn arguments(&self) -> Vec<String> {
        vec![
            "install".to_owned(),
            "to-disk".to_owned(),
            "--source-imgref".to_owned(),
            format!("registry:{}", self.reference),
            "--wipe".to_owned(),
            "--filesystem".to_owned(),
            THE_ONLY_FILESYSTEM.to_owned(),
            self.disk.path().display().to_string(),
        ]
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::path::Path;

    use super::*;

    /// The write pulls the pinned digest onto the disk by its own name, and
    /// passes nothing a shell would read.
    #[test]
    fn the_write_pulls_the_pinned_digest_onto_the_named_disk() {
        let pin = ThePin::read(
            &std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join(alo_image::THE_PIN))
                .unwrap(),
        )
        .unwrap();
        let disk = DiskName::named("virtio-alo-target").unwrap();
        let arguments = Writing::of(&pin, &disk).arguments();
        assert_eq!(
            arguments,
            [
                "install",
                "to-disk",
                "--source-imgref",
                &format!("registry:ghcr.io/aloworld-org/alo-os@{}", pin.digest()),
                "--wipe",
                "--filesystem",
                "btrfs",
                "/dev/disk/by-id/virtio-alo-target",
            ]
        );
        assert!(
            !arguments.iter().any(|argument| argument.contains(':')
                && argument.contains(pin.version())
                && !argument.contains('@')),
            "the release is never named by its tag"
        );
    }

    /// **The disk is written with the one filesystem an undo can be taken on**,
    /// named once, and taken from the crate that holds every writer to it.
    ///
    /// A person cannot convert this afterwards, so an installer that named
    /// `ext4` here would be a machine that can never undo what an agent did
    /// without being reinstalled (ADR 0045). A second `--filesystem` in the
    /// arguments would be the same defect arriving quietly, which is why this
    /// counts them rather than looking one up.
    #[test]
    fn the_disk_is_written_with_the_one_filesystem_an_undo_can_be_taken_on() {
        let pin = ThePin::read(
            &std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join(alo_image::THE_PIN))
                .unwrap(),
        )
        .unwrap();
        let disk = DiskName::named("virtio-alo-target").unwrap();
        let arguments = Writing::of(&pin, &disk).arguments();

        let named: Vec<&String> = arguments
            .iter()
            .zip(arguments.iter().skip(1))
            .filter(|(argument, _)| argument.as_str() == "--filesystem")
            .map(|(_, value)| value)
            .collect();
        assert_eq!(
            named,
            [THE_ONLY_FILESYSTEM],
            "the installer names {} filesystem(s): {named:?}",
            named.len()
        );
    }
}
