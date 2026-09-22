//! What the host must have before a walk is worth starting, asked rather than
//! assumed.
//!
//! A test that passed because it could not run is the worst kind of evidence,
//! so a machine missing any of this **fails** with the list of what it is
//! missing rather than skipping.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Where the walk keeps its machines, its media and its logs.
///
/// Never inside the checkout: it holds tens of gigabytes, including somebody
/// else's Windows.
pub const THE_YARD: &str = "/root/t10";

/// The Windows the walk installs, as it arrives from Microsoft.
///
/// An evaluation edition is the only Windows a test may install without a
/// licence: no account, no key and no form. It expires, so it is something the
/// walk fetches and throws away, never something this repository keeps.
pub const THE_WINDOWS: &str = "https://go.microsoft.com/fwlink/?linkid=2334167";

/// How big that download is, so a half-fetched one is not booted.
pub const THE_WINDOWS_IS: u64 = 7_092_807_680;

/// The firmware: Secure Boot capable, with nothing enrolled.
///
/// The build **without** Secure Boot cannot answer `Confirm-SecureBootUEFI` at
/// all, and `deciding.rs` refuses *could not be found out* as firmly as it
/// refuses *on* — so the walk would measure a refusal instead of the
/// installer. This build with the unenrolled variables answers *off*, which is
/// the only state ADR 0033 §4 lets the shipped installer run in.
pub const THE_FIRMWARE: &str = "/usr/share/OVMF/OVMF_CODE_4M.secboot.fd";

/// The firmware's variables, before anything has been enrolled in them.
pub const THE_BLANK_VARIABLES: &str = "/usr/share/OVMF/OVMF_VARS_4M.fd";

/// Every program the walk runs on the host.
///
/// `qemu-nbd`, `ntfs-3g` and `sfdisk` read a guest's disk with the machine off;
/// `podman` builds the environment from its recipe; and MinGW's linker builds
/// the installer for Windows from Linux (`download.rs`).
const EVERY_PROGRAM: [&str; 13] = [
    "rpm2cpio",
    "cpio",
    "qemu-system-x86_64",
    "qemu-img",
    "qemu-nbd",
    "swtpm",
    "swtpm_setup",
    "genisoimage",
    "socat",
    "ntfs-3g",
    "sfdisk",
    "podman",
    "x86_64-w64-mingw32-gcc",
];

/// What the host is missing, if anything.
#[must_use]
pub fn what_is_missing() -> Vec<String> {
    let mut missing = Vec::new();
    for program in EVERY_PROGRAM {
        if !on_the_path(program) {
            missing.push(format!("the program {program}"));
        }
    }
    for file in [THE_FIRMWARE, THE_BLANK_VARIABLES] {
        if !Path::new(file).is_file() {
            missing.push(format!("the firmware file {file}"));
        }
    }
    if !Path::new("/dev/kvm").exists() {
        missing.push("hardware virtualisation (/dev/kvm)".to_owned());
    }
    if !accelerates() {
        missing.push("a /dev/kvm with KVM behind it".to_owned());
    }
    missing
}

/// Whether a program can be run.
fn on_the_path(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .is_ok_and(|ran| ran.status.success())
        || Command::new("sh")
            .args(["-c", &format!("command -v {program}")])
            .output()
            .is_ok_and(|ran| ran.status.success())
}

/// **Whether `/dev/kvm` has KVM behind it**, which a host can show without
/// (`docs/quirks.md`, *WSL on a VMware guest shows `/dev/kvm` and has no KVM
/// behind it*). QEMU is asked to start with it and stop again.
fn accelerates() -> bool {
    Command::new("qemu-system-x86_64")
        .args([
            "-accel", "kvm", "-machine", "q35", "-display", "none", "-version",
        ])
        .output()
        .is_ok_and(|ran| ran.status.success())
}

/// The directory the walk works in, made if it is not there.
///
/// # Panics
/// When it cannot be made.
#[must_use]
pub fn the_yard() -> PathBuf {
    let yard = PathBuf::from(THE_YARD);
    std::fs::create_dir_all(&yard).expect("the walk's own directory");
    yard
}

/// How much room the host's own disk has, in gibibytes.
///
/// **Read of the host's disk and not of the Linux one**: under WSL the Linux
/// file system reports the virtual disk's size, which is not a measure of
/// anything (`docs/quirks.md`).
#[must_use]
pub fn room_in_gib() -> Option<u64> {
    let where_the_host_is = if Path::new("/mnt/c").is_dir() {
        "/mnt/c"
    } else {
        "/"
    };
    let printed = Command::new("df")
        .args(["--output=avail", "-BG", where_the_host_is])
        .output()
        .ok()?;
    String::from_utf8_lossy(&printed.stdout)
        .lines()
        .nth(1)?
        .trim()
        .trim_end_matches('G')
        .parse()
        .ok()
}

/// The least room a walk may start with.
///
/// Under this, the host's own disk fills, and on WSL that stops the whole
/// distribution and every other lane on the machine with it.
pub const ROOM_A_WALK_NEEDS: u64 = 25;
