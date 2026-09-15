//! The environment this installer stages, and whether this download's copy of it
//! is genuine.
//!
//! What a person downloads is this program and, beside it, the directory
//! `image/installing/Containerfile` builds ([`THE_DIRECTORY`]): the base's
//! signed loaders, a kernel, and the initramfs that installs alo OS. In that
//! directory is one more file, [`THE_LIST`] — the SHA-256 of every file the
//! environment is, one per line, as `sha256sum` writes them.
//!
//! # Genuine means: the list is the one this release was built with
//!
//! The SHA-256 of the list itself is compiled into the program when a release
//! is built ([`THE_RELEASED_LIST`], set by the release's workflow), and the
//! executable is what the release signs. So the chain is: the signed program
//! names the list, the list names every file, and each file is read once,
//! compared, and the **same bytes** are what get written — nothing is read
//! twice, so nothing can change between the check and the copy.
//!
//! A program built without a list — any build that is not a release — has
//! nothing to hold a download to, and says *this download is not a genuine alo
//! OS, so nothing was changed*. That is true of it.
//!
//! **A person never sees any of this** (ADR 0036, as the owner accepted it): the
//! check is not shown, offered, or skippable; a mismatch is
//! [`crate::words::NOT_GENUINE`], and there is no way past it.
//!
//! # What the environment itself checks
//!
//! That the release of alo OS it pulls is signed by the owner's key, with the
//! pin and the public half built into it (`alo_installing::Environment`). This
//! file is the step before: that the environment is the one this release
//! carries, so the key it checks with is the one this repository committed.

use std::path::{Path, PathBuf};

use alo_installing::DiskName;
use ring::digest::{SHA256, digest};

use crate::machine::TheMachine;
use crate::sizes::{MIB, THE_AREA};

/// The directory beside this program holding the environment.
pub const THE_DIRECTORY: &str = "alo-installing";

/// The list of every file the environment is, inside [`THE_DIRECTORY`].
pub const THE_LIST: &str = "alo-installing.sha256";

/// The files an environment cannot start without, as `docs/booting.md` lists
/// what the recipe produces.
pub const EVERY_FILE_IT_NEEDS: [&str; 6] = [
    "EFI/BOOT/BOOTX64.EFI",
    "EFI/BOOT/grubx64.efi",
    "EFI/BOOT/mmx64.efi",
    "EFI/BOOT/grub.cfg",
    "EFI/alo-installing/vmlinuz",
    "EFI/alo-installing/initramfs.img",
];

/// Where the person's choice is written, beside the loader's configuration.
pub const THE_CHOICE: &str = "EFI/BOOT/chosen.cfg";

/// What the choice's one line begins with; the disk's name follows it.
pub const THE_CHOICE_BEGINS: &str = "set alo_installing_to=";

/// The SHA-256 of [`THE_LIST`], in hexadecimal, as the release that built this
/// program set it — or nothing, for a program that is not a release.
pub const THE_RELEASED_LIST: Option<&str> = option_env!("ALO_INSTALLER_ENVIRONMENT_SHA256");

/// Room the FAT file system takes for itself in the area.
const THE_FILE_SYSTEMS_OWN: u64 = 16 * MIB;

/// The list a download is held to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Released(Option<[u8; 32]>);

/// The environment, read, compared, and ready to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheEnvironment {
    /// Each file, by its path inside the area, and its bytes.
    files: Vec<(String, Vec<u8>)>,
}

/// Why a download's environment is not staged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotStaged {
    /// Something that came with the installer is missing, unreadable, or not
    /// what a release ships.
    Incomplete,
    /// It is not what this release was built with.
    NotGenuine,
}

impl Released {
    /// The list this program was built with.
    #[must_use]
    pub fn this_build() -> Self {
        Self::of(THE_RELEASED_LIST)
    }

    /// A list, from its hexadecimal SHA-256 — nothing, for anything that is not one.
    #[must_use]
    pub fn of(hex: Option<&str>) -> Self {
        Self(hex.and_then(sha256_from_hex))
    }
}

impl TheEnvironment {
    /// The environment beside this program, held to the list it was released with.
    ///
    /// # Errors
    /// [`NotStaged::NotGenuine`] for a program with no released list, a list
    /// that is not that one, or a file that is not what the list says;
    /// [`NotStaged::Incomplete`] for anything missing or unreadable, a list that
    /// does not name every file an environment needs, or files that would not
    /// fit in the area.
    pub fn read(machine: &mut impl TheMachine, released: Released) -> Result<Self, NotStaged> {
        let directory = machine
            .downloaded_into()
            .map_err(|_| NotStaged::Incomplete)?
            .join(THE_DIRECTORY);
        let list = machine
            .read(&directory.join(THE_LIST))
            .map_err(|_| NotStaged::Incomplete)?;
        let expected = released.0.ok_or(NotStaged::NotGenuine)?;
        if digest(&SHA256, &list).as_ref() != expected {
            return Err(NotStaged::NotGenuine);
        }
        let listed = listed(&list).ok_or(NotStaged::Incomplete)?;

        let mut files = Vec::with_capacity(listed.len());
        let mut total: u64 = 0;
        for (sha256, path) in listed {
            let bytes = machine
                .read(&beneath(&directory, &path))
                .map_err(|_| NotStaged::Incomplete)?;
            if digest(&SHA256, &bytes).as_ref() != sha256 {
                return Err(NotStaged::NotGenuine);
            }
            total = total.saturating_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX));
            files.push((path, bytes));
        }
        if total > THE_AREA - THE_FILE_SYSTEMS_OWN {
            return Err(NotStaged::Incomplete);
        }
        Ok(Self { files })
    }

    /// Each file, by its path inside the area, and its bytes.
    #[must_use]
    pub fn files(&self) -> &[(String, Vec<u8>)] {
        &self.files
    }
}

/// The one line that tells the environment which disk to install onto.
#[must_use]
pub fn the_choice(disk: &DiskName) -> String {
    format!("{THE_CHOICE_BEGINS}{}\n", disk.as_str())
}

/// A path inside the area, beneath a root on this machine.
#[must_use]
pub fn beneath(root: &Path, inside: &str) -> PathBuf {
    inside
        .split('/')
        .fold(root.to_path_buf(), |path, part| path.join(part))
}

/// Whether bytes read back are the bytes written.
#[must_use]
pub fn is_the_same(written: &[u8], read_back: &[u8]) -> bool {
    digest(&SHA256, written).as_ref() == digest(&SHA256, read_back).as_ref()
}

/// The list's lines, each a SHA-256 and a path — or [`None`] for a list that is
/// not one a release writes.
fn listed(list: &[u8]) -> Option<Vec<([u8; 32], String)>> {
    let text = std::str::from_utf8(list).ok()?;
    let mut listed: Vec<([u8; 32], String)> = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let (hex, path) = line.split_once("  ")?;
        let sha256 = sha256_from_hex(hex)?;
        // `sha256sum` marks a file read in binary mode with a leading `*`.
        let path = path.strip_prefix('*').unwrap_or(path);
        if !is_a_path_inside_the_area(path) || listed.iter().any(|(_, seen)| seen == path) {
            return None;
        }
        listed.push((sha256, path.to_owned()));
    }
    let complete = EVERY_FILE_IT_NEEDS
        .iter()
        .all(|needed| listed.iter().any(|(_, path)| path == needed));
    let chooses_for_the_person = listed
        .iter()
        .any(|(_, path)| path.eq_ignore_ascii_case(THE_CHOICE));
    (complete && !chooses_for_the_person).then_some(listed)
}

/// A path beneath `EFI/` made only of plain names: no `..`, no drive, no
/// backslash, nothing that could land a file anywhere but inside the area.
fn is_a_path_inside_the_area(path: &str) -> bool {
    path.starts_with("EFI/")
        && path.split('/').all(|part| {
            !part.is_empty()
                && !part.starts_with('.')
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        })
}

/// Thirty-two bytes, from sixty-four hexadecimal digits.
fn sha256_from_hex(hex: &str) -> Option<[u8; 32]> {
    let hex = hex.trim().as_bytes();
    if hex.len() != 64 {
        return None;
    }
    let mut bytes = [0_u8; 32];
    for (byte, pair) in bytes.iter_mut().zip(hex.chunks(2)) {
        let text = std::str::from_utf8(pair).ok()?;
        *byte = u8::from_str_radix(text, 16).ok()?;
    }
    Some(bytes)
}

/// Sixty-four hexadecimal digits, from bytes — for tests and the release's
/// workflow alike.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    digest(&SHA256, bytes)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A list naming every needed file.
    fn a_list() -> String {
        EVERY_FILE_IT_NEEDS
            .iter()
            .map(|path| format!("{}  {path}\n", sha256_hex(path.as_bytes())))
            .collect()
    }

    /// A complete list is read, `sha256sum`'s binary marker and all.
    #[test]
    fn a_complete_list_is_read() {
        assert_eq!(listed(a_list().as_bytes()).unwrap().len(), 6);
        let binary = a_list().replace("  EFI", "  *EFI");
        assert_eq!(listed(binary.as_bytes()).unwrap().len(), 6);
    }

    /// **A list that is incomplete, repeats itself, writes the person's choice,
    /// or names a path outside the area is not a list.**
    #[test]
    fn a_list_that_could_misplace_a_file_is_not_a_list() {
        let first_line_gone: String = a_list().lines().skip(1).map(|l| format!("{l}\n")).collect();
        assert_eq!(listed(first_line_gone.as_bytes()), None);

        let repeated = format!("{}{}", a_list(), a_list().lines().next().unwrap());
        assert_eq!(listed(repeated.as_bytes()), None);

        let choosing = format!("{}{}  {THE_CHOICE}\n", a_list(), sha256_hex(b"x"));
        assert_eq!(listed(choosing.as_bytes()), None);

        for outside in [
            "EFI/../Windows/System32/x.dll",
            "Windows/x",
            "EFI\\BOOT\\x",
            "C:/EFI/x",
            "EFI//x",
            "EFI/.hidden",
            "/EFI/x",
        ] {
            let list = format!("{}{}  {outside}\n", a_list(), sha256_hex(b"x"));
            assert_eq!(listed(list.as_bytes()), None, "{outside}");
        }
    }

    /// **A program built without a released list has nothing genuine.**
    #[test]
    fn no_released_list_is_nothing_genuine() {
        assert_eq!(Released::of(None), Released(None));
        assert_eq!(Released::of(Some("not hex")), Released(None));
        assert_eq!(Released::of(Some(&"ab".repeat(31))), Released(None));
        assert!(Released::of(Some(&sha256_hex(b"list"))).0.is_some());
    }

    /// The choice is one line naming the disk, which `image/installing/grub.cfg`
    /// sources.
    #[test]
    fn the_choice_is_one_line_naming_the_disk() {
        let disk = DiskName::named("wwn-0x60022480aaaabbbbccccddddeeeeffff").unwrap();
        assert_eq!(
            the_choice(&disk),
            "set alo_installing_to=wwn-0x60022480aaaabbbbccccddddeeeeffff\n"
        );
        assert_eq!(
            beneath(Path::new("E:"), "EFI/BOOT/chosen.cfg"),
            Path::new("E:").join("EFI").join("BOOT").join("chosen.cfg")
        );
    }
}
