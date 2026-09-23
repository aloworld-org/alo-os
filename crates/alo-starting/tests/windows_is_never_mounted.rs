//! **alo OS never mounts the Windows partition read-write.**
//!
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
//! left Fast Startup open — a Windows that was shut down rather than
//! restarted leaves its volume hibernated on the disk — and said that this
//! sentence *holds either way*. It was decided on 2026-09-22, after this test
//! was written: ADR 0064 term 9, **the installer asks**, with `HiberbootEnabled`
//! set to `0` when the person says *Turn off*. This sentence holds whichever
//! they answer, and the asking itself is the installer plan's task 4.
//!
//! # What this proves, exactly
//!
//! Three things, each read off the disk rather than asserted:
//!
//! 1. **The way across does not mount anything.** The generated menu reaches
//!    Windows by handing Windows's own start-up program to the firmware, and
//!    nothing in it mounts a filesystem.
//! 2. **This crate could not mount anything.** It depends on no disk service
//!    and names no filesystem, read off its own manifest and its own source.
//! 3. **The image asks for no such mount.** Nothing `image/` ships names a
//!    Windows filesystem, an automatic mount of one, or a tool that would make
//!    one.
//!
//! # What it does not prove, and says so
//!
//! It is not a measurement on a machine with a Windows on it. That belongs to
//! the walk in the virtual machine on the development PC, which
//! `docs/booting.md` names; this is the half a test can hold, which is that
//! nothing alo OS ships ever asks for the mount.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what could not be read is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_starting::{Menu, THE_COUNTDOWN, THE_WINDOWS_LOADER};

/// What a Windows volume is mounted with, or named as, wherever it is written.
const A_MOUNT_OF_WINDOWS: [&str; 3] = ["ntfs", "fuseblk", "exfat"];

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// Every file under `directory`, as a repository-relative path and its text.
fn everything_under(directory: &Path, below: &str, into: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = format!("{below}/{name}");
        if entry.path().is_dir() {
            if name != "target" {
                everything_under(&entry.path(), &path, into);
            }
        } else if let Ok(text) = fs::read_to_string(entry.path()) {
            into.push((path, text));
        }
    }
}

/// **The way across to Windows hands a program over; it mounts nothing.**
#[test]
fn the_way_across_to_windows_mounts_nothing() {
    let written = Menu::offering("Windows", THE_COUNTDOWN)
        .expect("a menu is generated")
        .written();
    assert!(
        written.contains(&format!("chainloader {THE_WINDOWS_LOADER}")),
        "{written}"
    );
    let said = written.to_lowercase();
    for never in A_MOUNT_OF_WINDOWS {
        assert!(!said.contains(never), "the menu says {never:?}: {written}");
    }
    for never in ["mount", " rw", "rw,", "loopback", "cryptomount"] {
        assert!(!said.contains(never), "the menu says {never:?}: {written}");
    }
}

/// **This crate could not mount anything.** It depends on no disk service, and
/// names no filesystem anywhere in its source — the stricter reading, and the
/// right one: *alo OS never mounts the Windows partition* is a promise about
/// files somebody audits, not about a link graph they would have to compute.
#[test]
fn this_crate_could_not_mount_anything() {
    let mut written = Vec::new();
    everything_under(
        &the_repository().join("crates").join("alo-starting"),
        "crates/alo-starting",
        &mut written,
    );
    assert!(written.len() >= 10, "{} file(s) were read", written.len());

    let manifest = written
        .iter()
        .find(|(path, _)| path.ends_with("/Cargo.toml"))
        .map(|(_, text)| text.clone())
        .expect("this crate has a manifest");
    for a_disk_service in ["alo-drives", "alo-brokerd", "udisks", "rustix", "libc"] {
        assert!(
            !manifest.contains(&format!("{a_disk_service} =")),
            "alo-starting depends on {a_disk_service}"
        );
    }

    for (path, text) in &written {
        if path.contains("/tests/") {
            continue;
        }
        // A file's own tests are where the words this refuses are written down
        // to be refused. What is read is the source above them.
        let said = text
            .split("#[cfg(test)]")
            .next()
            .unwrap_or_default()
            .to_lowercase();
        for never in A_MOUNT_OF_WINDOWS {
            assert!(!said.contains(never), "{path} names {never:?}");
        }
    }
}

/// **The image asks for no mount of a Windows volume.** Nothing it ships names
/// one, mounts one automatically, or carries the tool that would.
#[test]
fn nothing_the_image_ships_mounts_a_windows_volume() {
    let mut written = Vec::new();
    everything_under(&the_repository().join("image"), "image", &mut written);
    assert!(
        written.len() >= 5,
        "{} file(s) of the image were read",
        written.len()
    );
    for (path, text) in &written {
        let said = text.to_lowercase();
        for never in A_MOUNT_OF_WINDOWS {
            assert!(!said.contains(never), "{path} names {never:?}");
        }
        assert!(!said.contains("x-systemd.automount"), "{path} automounts");
    }
}
