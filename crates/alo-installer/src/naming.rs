//! The name the environment will know a disk by, after the restart.
//!
//! The environment is told which disk to install onto as the disk's own name
//! under Linux's `/dev/disk/by-id/` (`docs/booting.md`), and this installer
//! runs on Windows, which has never heard of that directory. So the name is
//! made here from what Windows reports about the disk — its bus, its maker's
//! model name, its serial number and its unique identifier — the way udev's
//! own rules make it on the other side.
//!
//! # Only where the rule is known, and failing closed everywhere else
//!
//! | Bus, as Windows says | Name, as udev makes it |
//! |---|---|
//! | `NVMe`, an NGUID or EUI-64 in place of the serial | `nvme-eui.<identifier>` |
//! | `NVMe`, an `eui.` unique identifier | `nvme-eui.<identifier>` |
//! | `SATA` | `ata-<model>_<serial>` |
//! | `SAS`, `SCSI`, with an NAA identifier | `wwn-0x<identifier>` |
//!
//! **NVMe is named by its identifier, never by its model.** Measured on this
//! repository's development machine (a Dell with an SK hynix NVMe disk) on
//! 2026-09-15: Windows reports the model as `NVMe PVC10 SK hynix 512GB` — its own
//! bus prefixed to the name the disk gives Linux — and in place of the serial
//! number an identifier in groups of four ending in `.`
//! (`FD5B_42CE_BC8F_9D54_ACE4_2E00_5113_F94B.`), with the disk's EUI-64 as its
//! unique identifier. Linux's `wwid` for the same disk is the NGUID when there is
//! one and the EUI-64 otherwise, and udev names the disk `nvme-` and that.
//!
//! Spaces in a SATA model or serial become one `_`, as udev writes them. Any other
//! bus — USB above all, whose names depend on the bridge chip — has no name
//! here, and a disk with no name is not offered to install onto.
//!
//! **A wrong name cannot pick a wrong disk.** Every name above carries the
//! disk's serial number or unique identifier, so a name made wrongly names no
//! disk at all, and the environment then says *the disk you chose is not
//! connected, so nothing was changed*. None of these rules has yet been
//! checked against a real disk from both sides; `docs/booting.md` says so, and
//! the installer plan's virtual-machine test is where the first one is.

use alo_installing::DiskName;

/// The name the environment will look for, where Windows' report is enough to
/// make one.
#[must_use]
pub fn after_the_restart(
    bus: &str,
    model: &str,
    serial: &str,
    unique_id: &str,
) -> Option<DiskName> {
    let named = match bus.trim() {
        "NVMe" => format!(
            "nvme-eui.{}",
            identifier_in_groups(serial).or_else(|| eui_identifier(unique_id))?
        ),
        "SATA" => format!("ata-{}_{}", udev_words(model)?, udev_words(serial)?),
        "SAS" | "SCSI" => format!("wwn-0x{}", naa_in(unique_id)?),
        _ => return None,
    };
    DiskName::named(&named).ok()
}

/// A model or serial as udev writes it into a name: trimmed, every run of
/// white space one `_`, and nothing at all when there is nothing.
fn udev_words(written: &str) -> Option<String> {
    let words: Vec<&str> = written.split_whitespace().collect();
    (!words.is_empty()).then(|| words.join("_"))
}

/// The identifier Windows' NVMe driver reports in place of a serial number —
/// sixteen or thirty-two hexadecimal digits in groups of four joined by `_`,
/// ending in `.` — as Linux writes it: lower case, joined.
fn identifier_in_groups(serial: &str) -> Option<String> {
    let groups = serial.trim().strip_suffix('.')?;
    let parts: Vec<&str> = groups.split('_').collect();
    let shaped = matches!(parts.len(), 4 | 8)
        && parts
            .iter()
            .all(|part| part.len() == 4 && part.bytes().all(|b| b.is_ascii_hexdigit()));
    shaped.then(|| parts.concat().to_ascii_lowercase())
}

/// An `eui.` unique identifier of sixteen or thirty-two hexadecimal digits,
/// lower case.
fn eui_identifier(unique_id: &str) -> Option<String> {
    let hex = unique_id.trim().strip_prefix("eui.")?;
    let shaped = matches!(hex.len(), 16 | 32) && hex.bytes().all(|b| b.is_ascii_hexdigit());
    shaped.then(|| hex.to_ascii_lowercase())
}

/// An NAA identifier as Windows reports it — sixteen or thirty-two hexadecimal
/// digits whose first says NAA 5 or 6 — lower case.
fn naa_in(unique_id: &str) -> Option<String> {
    let id = unique_id.trim();
    let shaped = matches!(id.len(), 16 | 32)
        && id.bytes().all(|b| b.is_ascii_hexdigit())
        && matches!(id.as_bytes().first(), Some(b'5' | b'6'));
    shaped.then(|| id.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The name, as a string, for short assertions.
    fn named(bus: &str, model: &str, serial: &str, unique_id: &str) -> Option<String> {
        after_the_restart(bus, model, serial, unique_id).map(|name| name.as_str().to_owned())
    }

    /// Each bus with a known rule is named as udev names it.
    #[test]
    fn each_known_bus_is_named_as_udev_names_it() {
        // As this repository's development machine reports its own disk.
        assert_eq!(
            named(
                "NVMe",
                "NVMe PVC10 SK hynix 512GB",
                "FD5B_42CE_BC8F_9D54_ACE4_2E00_5113_F94B.",
                "eui.ACE42E005113F94B"
            ),
            Some("nvme-eui.fd5b42cebc8f9d54ace42e005113f94b".to_owned())
        );
        assert_eq!(
            named(
                "NVMe",
                "Samsung SSD 980 1TB",
                "S64ANS0T123456A",
                "eui.002538B411B2C3D4"
            ),
            Some("nvme-eui.002538b411b2c3d4".to_owned())
        );
        assert_eq!(
            named("NVMe", "WDC PC SN730", "E823_8FA6_BF53_0001.", ""),
            Some("nvme-eui.e8238fa6bf530001".to_owned())
        );
        assert_eq!(
            named(
                "SATA",
                "  Samsung SSD 860 EVO 500GB ",
                " S3Z1NB0K123456 ",
                ""
            ),
            Some("ata-Samsung_SSD_860_EVO_500GB_S3Z1NB0K123456".to_owned())
        );
        assert_eq!(
            named(
                "SAS",
                "Msft Virtual Disk",
                "",
                "60022480D52E6B1C3F7A9E2B4C8D1F00"
            ),
            Some("wwn-0x60022480d52e6b1c3f7a9e2b4c8d1f00".to_owned())
        );
    }

    /// **A disk whose name cannot be made has none** — never a guess.
    #[test]
    fn a_disk_whose_name_cannot_be_made_has_none() {
        assert_eq!(named("USB", "SanDisk Ultra", "4C530001", ""), None);
        assert_eq!(named("Virtual", "Msft Virtual Disk", "", ""), None);
        assert_eq!(named("NVMe", "Samsung SSD 980", "", ""), None);
        // An NVMe disk is never named by its model: Windows puts its own bus in
        // front of it.
        assert_eq!(
            named("NVMe", "Samsung SSD 980 1TB", "S64ANS0T123456A", ""),
            None
        );
        assert_eq!(named("NVMe", "Disk", "", "eui.12345"), None);
        assert_eq!(named("SATA", "", "S3Z1", ""), None);
        assert_eq!(named("SAS", "Msft Virtual Disk", "", "not-an-naa"), None);
        assert_eq!(named("SCSI", "Disk", "", "20022480D52E6B1C"), None);
        // A character udev would escape, and so a name the environment could
        // never be handed.
        assert_eq!(named("SATA", "Disk/Model", "123", ""), None);
    }
}
