//! What the firmware says it started, in its own words.
//!
//! OVMF prints every start it makes on the serial line —
//! `BdsDxe: starting Boot0002 "alo OS" from HD(4,GPT,<guid>,0x7C8F800,0x200000)/\EFI\BOOT\BOOTX64.EFI`
//! — and that line is the only account of *which* entry started *from where*
//! that nothing else wrote: not `bcdedit`, which reports its own store, and not
//! the machine being up, which says nothing about which loader ran.
//!
//! # Two firmware builds, for two reasons
//!
//! Windows and the walk run on Ubuntu's OVMF 2025.11. The environment cannot:
//! that build page-faults starting the base's signed shim (`docs/quirks.md`,
//! *EDK II's strict image protection page-faults the base's signed loader*),
//! measured again on this walk's own road on 2026-09-21. So the road that has
//! to show the environment start runs on Fedora's `edk2-ovmf` 20250812-21, the
//! build task 9 measured starting it — the package fetched by its exact name
//! and digest, never whatever a mirror has newest.

use std::path::{Path, PathBuf};

use super::machine::run;

/// Fedora's firmware package, by its exact build.
pub const FEDORAS_FIRMWARE: &str = "https://kojipkgs.fedoraproject.org/packages/edk2/20250812/21.fc42/noarch/edk2-ovmf-20250812-21.fc42.noarch.rpm";

/// That package's SHA-256, measured when it was first fetched on 2026-09-21.
pub const FEDORAS_FIRMWARE_SHA256: &str =
    "1d5f11ffad27f9b5fa5792f16902968e0614104a9ae8e26e427573d62a34a418";

/// One start the firmware made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Started {
    /// The entry's number, as in `Boot0002`.
    pub entry: String,
    /// Its description, as in `alo OS`.
    pub description: String,
    /// The GPT partition number of its hard-drive node, when it has one.
    pub partition: Option<u32>,
    /// That partition's first sector.
    pub first_sector: Option<u64>,
    /// The file it started.
    pub file: String,
}

/// Every start the firmware printed on this serial line, in order.
#[must_use]
pub fn starts(said: &str) -> Vec<Started> {
    said.lines()
        .filter_map(|line| {
            let rest = line.split("BdsDxe: starting ").nth(1)?;
            let (entry, rest) = rest.split_once(' ')?;
            let rest = rest.strip_prefix('"')?;
            let (description, rest) = rest.split_once('"')?;
            let path = rest.trim().strip_prefix("from ")?;
            let (partition, first_sector) = path
                .split_once("HD(")
                .and_then(|(_, inside)| inside.split_once(')'))
                .map_or((None, None), |(fields, _)| {
                    let parts: Vec<&str> = fields.split(',').collect();
                    let number = parts.first().and_then(|n| n.parse().ok());
                    let first = parts
                        .get(3)
                        .and_then(|hex| u64::from_str_radix(hex.trim_start_matches("0x"), 16).ok());
                    (number, first)
                });
            let file = path
                .rsplit(')')
                .next()
                .unwrap_or_default()
                .trim_start_matches('/');
            Some(Started {
                entry: entry.to_owned(),
                description: description.to_owned(),
                partition,
                first_sector,
                file: file.to_owned(),
            })
        })
        .collect()
}

/// Fedora's firmware, fetched by its exact build, checked, and laid out as the
/// raw flash image QEMU wants.
///
/// # Panics
/// When it cannot be fetched, is not the package measured, or will not unpack.
#[must_use]
pub fn fedoras(yard: &Path) -> PathBuf {
    let at = yard.join("fedora-ovmf");
    let raw = at.join("OVMF_CODE_4M.secboot.fedora.fd");
    if raw.is_file() {
        return raw;
    }
    std::fs::create_dir_all(&at).expect("somewhere for the firmware");
    let package = at.join("edk2-ovmf.rpm");
    run(
        "curl",
        &[
            "-sfL",
            "-o",
            &package.display().to_string(),
            FEDORAS_FIRMWARE,
        ],
    );
    let digest = run("sha256sum", &[&package.display().to_string()]);
    assert!(
        digest.starts_with(FEDORAS_FIRMWARE_SHA256),
        "the firmware package is not the one measured: {digest}"
    );
    run(
        "sh",
        &[
            "-c",
            &format!(
                "cd {} && rpm2cpio edk2-ovmf.rpm | cpio -idm --quiet",
                at.display()
            ),
        ],
    );
    run(
        "qemu-img",
        &[
            "convert",
            "-O",
            "raw",
            &at.join("usr/share/edk2/ovmf/OVMF_CODE_4M.secboot.qcow2")
                .display()
                .to_string(),
            &raw.display().to_string(),
        ],
    );
    raw
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The firmware's line is read for its entry, its partition and its file.
    #[test]
    fn the_firmwares_line_is_read_for_what_it_started() {
        let said = "BdsDxe: loading Boot0002 \"alo OS\" from x\n\
                    BdsDxe: starting Boot0002 \"alo OS\" from HD(4,GPT,D4D5B530-7210-4A12-B7CA-FB973B0B0BA7,0x7C8F800,0x200000)/\\EFI\\BOOT\\BOOTX64.EFI\n\
                    BdsDxe: starting Boot0004 \"Windows Boot Manager\" from HD(1,GPT,DEBBD5DE-6252-4321-8EBE-13EB00431ED7,0x800,0x96000)/\\EFI\\Microsoft\\Boot\\bootmgfw.efi";
        let started = starts(said);
        assert_eq!(started.len(), 2);
        let alo = started.first().expect("the first start");
        assert_eq!(alo.entry, "Boot0002");
        assert_eq!(alo.description, "alo OS");
        assert_eq!(alo.partition, Some(4));
        assert_eq!(alo.first_sector, Some(0x7C8_F800));
        assert_eq!(alo.file, "\\EFI\\BOOT\\BOOTX64.EFI");
        assert_eq!(
            started.get(1).and_then(|windows| windows.partition),
            Some(1)
        );
    }
}
