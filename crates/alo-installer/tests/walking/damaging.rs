//! Taking alo OS's loader away from a disk, with the machine off.
//!
//! The menu's first term (ADR 0062) is that a computer which cannot start alo
//! OS starts Windows, with nobody at the keyboard. The only way to measure
//! that is to make alo OS unstartable on a machine that has just installed it,
//! and the only honest place to do it is from outside the guest, with nothing
//! running: the loader is renamed on the disk the firmware would start it
//! from, and then the machine is started again and left alone.
//!
//! **This is the one place in the walk that writes to a guest's disk.**
//! [`super::reading`] attaches every disk read-only and says so; this module
//! attaches one writable, changes exactly one name, and gives back what it
//! did, so that a test can say in its own words what was broken and check that
//! it was broken. Nothing else here touches a disk.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Where the loader the base installs sits on the partition the firmware
/// starts it from. The firmware's entry names this same file
/// (`alo_installing::THE_LOADER_THE_BASE_INSTALLS`), written here as a path
/// under a mount point.
pub const THE_LOADER: &str = "EFI/fedora/shimx64.efi";

/// What a taken-away loader is called instead: a rename, never a delete, so
/// that the disk still holds the file and the change is one name.
pub const TAKEN_AWAY: &str = "EFI/fedora/shimx64.efi.taken-away";

/// What was done to a disk, for the test to print and to hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TakenAway {
    /// The partition the loader was found on, as in `/dev/nbd2p2`.
    pub partition: String,
    /// The loader's size in bytes before it was renamed.
    pub was_bytes: u64,
    /// Whether the loader's own name is gone from the partition afterwards.
    pub gone: bool,
}

/// Take alo OS's loader away from a disk no machine is running from, and say
/// what was done.
///
/// The partition is not named by number: the disk is searched for the one that
/// holds [`THE_LOADER`], because which partition that is belongs to the image
/// being installed and not to this test.
///
/// # Panics
/// When a machine is running, when the disk cannot be attached, or when no
/// partition of it holds the loader.
#[must_use]
pub fn take_the_loader_away(disk: &Path, yard: &Path) -> TakenAway {
    assert!(
        !super::machine::is_running(),
        "a machine is running, and its disk would be written while it reads it"
    );
    let attached = Writable::new(disk, yard);
    let at = attached.at.display().to_string();
    for partition in 1..=6 {
        let device = format!("{}p{partition}", attached.device.display());
        if !Path::new(&device).exists() {
            continue;
        }
        let mounted = Command::new("mount")
            .args(["-t", "vfat", &device, &at])
            .status()
            .is_ok_and(|status| status.success());
        if !mounted {
            continue;
        }
        let loader = attached.at.join(THE_LOADER);
        if let Ok(about) = std::fs::metadata(&loader) {
            let taken = attached.at.join(TAKEN_AWAY);
            std::fs::rename(&loader, &taken).expect("the loader could not be renamed");
            let gone = !loader.exists() && taken.exists();
            let _ = Command::new("sync").status();
            return TakenAway {
                partition: device,
                was_bytes: about.len(),
                gone,
            };
        }
        let _ = Command::new("umount").arg(&at).status();
    }
    panic!(
        "no partition of {} holds {THE_LOADER}, so alo OS's loader is not where its \
         firmware entry says it is",
        disk.display()
    );
}

/// **The file taken away is the file the firmware entry names.** If the
/// installed loader ever moves, this test fails rather than the walk quietly
/// renaming a file nothing starts and calling the fall-through proven.
#[test]
fn the_loader_taken_away_is_the_one_the_entry_starts() {
    let named = alo_installing::THE_LOADER_THE_BASE_INSTALLS
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_owned();
    assert_eq!(named, THE_LOADER);
    assert!(TAKEN_AWAY.starts_with(THE_LOADER));
    assert_ne!(TAKEN_AWAY, THE_LOADER);
}

/// A disk attached so it can be written, with a mount point of its own —
/// unmounted and detached however the writing ends.
struct Writable {
    /// The device the disk is attached as.
    device: PathBuf,
    /// Where a partition of it is mounted.
    at: PathBuf,
}

impl Writable {
    /// Attach the disk to a device nothing is using and make the mount point.
    fn new(disk: &Path, yard: &Path) -> Self {
        let _ = Command::new("modprobe")
            .args(["nbd", "max_part=8"])
            .output();
        let device = (0..3)
            .find_map(|_| attach(disk))
            .expect("the disk could not be attached to any device");
        let at = yard.join(format!("damaging-{}", std::process::id()));
        let _ = Command::new("umount").arg(&at).output();
        std::fs::create_dir_all(&at).expect("a mount point");
        Self { device, at }
    }
}

impl Drop for Writable {
    fn drop(&mut self) {
        let _ = Command::new("umount").arg(&self.at).output();
        let _ = std::fs::remove_dir(&self.at);
        let _ = Command::new("qemu-nbd")
            .arg("--disconnect")
            .arg(&self.device)
            .output();
        std::thread::sleep(Duration::from_secs(2));
    }
}

/// Attach a disk writable to a device nothing is using, and give it back once
/// it answers — or nothing, having let it go again. A device that was just
/// disconnected answers the next connection with an I/O error, which is why
/// the device chosen is one whose size is zero and why this is tried again.
fn attach(disk: &Path) -> Option<PathBuf> {
    let free = (2..8).find(|n| {
        std::fs::read_to_string(format!("/sys/block/nbd{n}/size"))
            .is_ok_and(|size| size.trim() == "0")
    })?;
    let device = PathBuf::from(format!("/dev/nbd{free}"));
    let connected = Command::new("qemu-nbd")
        .arg(format!("--connect={}", device.display()))
        .arg(disk)
        .status()
        .is_ok_and(|status| status.success());
    if connected {
        for _ in 0..20 {
            let sized = std::fs::read_to_string(format!("/sys/block/nbd{free}/size"))
                .is_ok_and(|size| size.trim() != "0");
            if sized && PathBuf::from(format!("/dev/nbd{free}p1")).exists() {
                return Some(device);
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    let _ = Command::new("qemu-nbd")
        .arg("--disconnect")
        .arg(&device)
        .output();
    std::thread::sleep(Duration::from_secs(5));
    None
}
