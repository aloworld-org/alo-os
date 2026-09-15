//! The typed consent: the name of the disk alo OS replaces, and nothing less.
//!
//! ADR 0023 §1 asks for *a typed consent — not a checkbox — for anything
//! destructive*, and the installer plan's task 3 for one *naming the disk*. So
//! the person agrees by typing the name of one of the disks they were shown,
//! and the name they type is also which disk they chose: there is no second
//! question whose answer could disagree with the first.
//!
//! # Read forgivingly in form, never in substance
//!
//! Upper or lower case, and extra spaces, do not matter; a person copying a
//! maker's name by hand gets those wrong and means the same disk. Anything
//! else is not that disk: a prefix, a disk that was not offered, the disk
//! Windows is on. Nothing typed is a person who stopped.

use crate::deciding::ForAloOs;
use crate::ended::Refusal;

/// The disk the person typed the name of.
///
/// # Errors
/// [`Refusal::NotAgreed`] for nothing typed, and [`Refusal::NotADisksName`] for
/// anything that is not exactly one offered disk's name.
pub fn chosen<'offered>(
    typed: &str,
    offered: &'offered [ForAloOs],
) -> Result<&'offered ForAloOs, Refusal> {
    let typed = normalised(typed);
    if typed.is_empty() {
        return Err(Refusal::NotAgreed);
    }
    let mut matching = offered
        .iter()
        .filter(|disk| normalised(&disk.shown) == typed);
    match (matching.next(), matching.next()) {
        (Some(disk), None) => Ok(disk),
        _ => Err(Refusal::NotADisksName),
    }
}

/// A name, with its case and its white space made not to matter.
fn normalised(written: &str) -> String {
    written
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_installing::DiskName;

    use super::*;
    use crate::identities::DiskNumber;

    /// Two disks offered.
    fn offered() -> Vec<ForAloOs> {
        vec![
            ForAloOs {
                number: DiskNumber(1),
                shown: "Msft Virtual Disk 1".to_owned(),
                after_the_restart: DiskName::named("wwn-0x6002248011").unwrap(),
            },
            ForAloOs {
                number: DiskNumber(2),
                shown: "Samsung SSD 870 EVO".to_owned(),
                after_the_restart: DiskName::named("ata-Samsung_SSD_870_EVO_S5Y1").unwrap(),
            },
        ]
    }

    /// **The name typed is the disk chosen**, whatever its case and spacing.
    #[test]
    fn the_name_typed_is_the_disk_chosen() {
        let offered = offered();
        assert_eq!(
            chosen("samsung  ssd 870 evo\r\n", &offered).unwrap().number,
            DiskNumber(2)
        );
        assert_eq!(
            chosen("Msft Virtual Disk 1", &offered).unwrap().number,
            DiskNumber(1)
        );
    }

    /// **Nothing typed is a person who stopped.**
    #[test]
    fn nothing_typed_is_a_person_who_stopped() {
        assert_eq!(chosen("", &offered()), Err(Refusal::NotAgreed));
        assert_eq!(chosen("   \r\n", &offered()), Err(Refusal::NotAgreed));
    }

    /// **Anything but exactly an offered disk's name is refused** — yes, a
    /// prefix, the disk Windows is on, or a disk that was not offered.
    #[test]
    fn anything_but_an_offered_disks_name_is_refused() {
        for typed in [
            "yes",
            "y",
            "I agree",
            "Msft Virtual Disk",
            "Msft Virtual Disk 0",
            "Samsung SSD 870",
            "Samsung SSD 870 EVO Plus",
            "1",
            "wwn-0x6002248011",
        ] {
            assert_eq!(
                chosen(typed, &offered()),
                Err(Refusal::NotADisksName),
                "{typed}"
            );
        }
        assert_eq!(chosen("anything", &[]), Err(Refusal::NotADisksName));
    }
}
