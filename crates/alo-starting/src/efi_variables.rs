//! The firmware of a real machine, read and told through the kernel's own
//! window onto it.
//!
//! A machine's start-up entries are firmware variables, and Linux presents them
//! as files: one per variable, each beginning with four bytes of the firmware's
//! own attributes and then the variable itself. This reads those files and
//! writes exactly one of them.
//!
//! # What it will not do
//!
//! **It writes `BootNext` and nothing else.** Not the order, not an entry, not
//! a deletion. A machine's start-up order is somebody's only way back into
//! their own computer, and a component that could rewrite it is a component
//! that can take a computer away from its owner — so there is no method here
//! that could, rather than a rule about not calling one
//! ([ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md),
//! *what stays as it was*).
//!
//! # An entry that cannot be read is left out, and that is safe
//!
//! A firmware regularly keeps entries nothing in this repository has seen. One
//! of them failing to parse must not take away a person's way into their own
//! Windows, and leaving it out cannot cause a wrong answer: an entry that is
//! not in the list can only fail to match what was approved, and a failure to
//! match is a refusal that changes nothing.
//!
//! # Where this has run
//!
//! Against a directory of files, in this crate's own tests. **Not on a machine
//! with firmware behind it**: `docs/booting.md` says what is owed, and the
//! kernel's rule about which variables may be written without being unlocked
//! first is one of the things that walk measures rather than this file
//! asserting it.

use std::fs;
use std::path::{Path, PathBuf};

use crate::firmware::{Entry, Firmware, NotAnswering, NotDone};

/// Where the kernel presents the firmware's variables.
pub const THE_VARIABLES: &str = "/sys/firmware/efi/efivars";

/// The group the firmware keeps its own start-up variables in.
///
/// Every variable this reads or writes is named `<name>-` and then this.
pub const THE_GLOBAL_GROUP: &str = "8be4df61-93ca-11d2-aa0d-00e098032b8c";

/// The variable naming what the machine starts next time, once.
pub const NEXT_START: &str = "BootNext";

/// What every start-up entry's variable is named before its number.
const AN_ENTRY: &str = "Boot";

/// How many hexadecimal digits a start-up entry's number is written in.
const DIGITS: usize = 4;

/// The four bytes of attributes every variable begins with: kept across a
/// restart, readable before the system starts, and readable after it.
///
/// Written back exactly as the firmware expects them; a variable written
/// without them is a variable the kernel refuses.
const KEPT_AND_READABLE: [u8; 4] = [7, 0, 0, 0];

/// The number a start-up entry's file name carries, if it is one.
///
/// `Boot0001-<the global group>` and nothing else: a name of another shape is
/// another firmware variable, and there are many.
#[must_use]
pub fn the_entry_named(name: &str) -> Option<u16> {
    let rest = name.strip_prefix(AN_ENTRY)?;
    let (number, group) = rest.split_once('-')?;
    if !group.eq_ignore_ascii_case(THE_GLOBAL_GROUP) || number.len() != DIGITS {
        return None;
    }
    u16::from_str_radix(number, 16).ok()
}

/// The bytes the next-start variable is written as, for this entry.
#[must_use]
pub fn the_next_start_written(entry: u16) -> Vec<u8> {
    let mut bytes = KEPT_AND_READABLE.to_vec();
    bytes.extend_from_slice(&entry.to_le_bytes());
    bytes
}

/// The firmware of the machine this is running on, read through a directory of
/// its variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheFirmware {
    /// The directory the variables are in.
    variables: PathBuf,
}

impl Default for TheFirmware {
    fn default() -> Self {
        Self::of_this_machine()
    }
}

impl TheFirmware {
    /// This machine's own firmware.
    #[must_use]
    pub fn of_this_machine() -> Self {
        Self::at(Path::new(THE_VARIABLES))
    }

    /// A firmware whose variables are in this directory, for a test.
    #[must_use]
    pub fn at(variables: &Path) -> Self {
        Self {
            variables: variables.to_owned(),
        }
    }

    /// Where the variables are.
    #[must_use]
    pub fn variables(&self) -> &Path {
        &self.variables
    }
}

impl Firmware for TheFirmware {
    fn entries(&self) -> Result<Vec<Entry>, NotAnswering> {
        let found = fs::read_dir(&self.variables).map_err(|why| {
            NotAnswering(format!(
                "{} could not be read, so this machine was not asked what it can start: {why}",
                self.variables.display()
            ))
        })?;
        let mut entries = Vec::new();
        for each in found.flatten() {
            let name = each.file_name().to_string_lossy().into_owned();
            let Some(number) = the_entry_named(&name) else {
                continue;
            };
            let Ok(bytes) = fs::read(each.path()) else {
                continue;
            };
            let Some(after_the_attributes) = bytes.get(KEPT_AND_READABLE.len()..) else {
                continue;
            };
            if let Ok(entry) = Entry::reported(number, after_the_attributes) {
                entries.push(entry);
            }
        }
        entries.sort_by_key(Entry::number);
        Ok(entries)
    }

    fn start_next(&self, entry: u16) -> Result<(), NotDone> {
        let at = self
            .variables
            .join(format!("{NEXT_START}-{THE_GLOBAL_GROUP}"));
        fs::write(&at, the_next_start_written(entry)).map_err(|why| {
            NotDone(format!(
                "{} could not be written, so this machine was not told what to start next: {why}",
                at.display()
            ))
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::systems::THE_WINDOWS_LOADER_IN_A_PATH;
    use crate::testing::as_a_firmware_reports_it;

    /// A directory of firmware variables, made for one test.
    fn a_firmware(named: &str) -> TheFirmware {
        let at = std::env::temp_dir().join(format!(
            "alo-starting-firmware-{}-{named}",
            std::process::id()
        ));
        drop(fs::remove_dir_all(&at));
        fs::create_dir_all(&at).unwrap();
        TheFirmware::at(&at)
    }

    /// Put a start-up entry in this firmware, as the kernel presents one.
    fn holding(firmware: &TheFirmware, number: u16, entry: &str, path: &str) {
        let mut bytes = KEPT_AND_READABLE.to_vec();
        bytes.extend_from_slice(&as_a_firmware_reports_it(entry, path));
        fs::write(
            firmware
                .variables()
                .join(format!("{AN_ENTRY}{number:04X}-{THE_GLOBAL_GROUP}")),
            bytes,
        )
        .unwrap();
    }

    /// **The entries a machine has are read, in the order of their numbers**,
    /// with the attributes taken off and the rest left exactly as reported.
    #[test]
    fn the_entries_a_machine_has_are_read_in_order() {
        let firmware = a_firmware("read");
        holding(&firmware, 3, "alo OS", "\\EFI\\fedora\\shimx64.efi");
        holding(
            &firmware,
            1,
            "Windows Boot Manager",
            THE_WINDOWS_LOADER_IN_A_PATH,
        );
        let entries = firmware.entries().unwrap();
        assert_eq!(entries.len(), 2);
        let numbers: Vec<u16> = entries.iter().map(Entry::number).collect();
        assert_eq!(numbers, [1, 3]);
        assert!(entries.first().unwrap().starts_windows());
        assert!(!entries.last().unwrap().starts_windows());
    }

    /// **Everything that is not a start-up entry is passed over.** A firmware
    /// keeps dozens of variables, and reading one of them as an entry is how a
    /// machine gets told to start something nobody has.
    #[test]
    fn nothing_that_is_not_a_start_up_entry_is_read_as_one() {
        let firmware = a_firmware("others");
        holding(
            &firmware,
            1,
            "Windows Boot Manager",
            THE_WINDOWS_LOADER_IN_A_PATH,
        );
        for other in [
            format!("BootOrder-{THE_GLOBAL_GROUP}"),
            format!("BootCurrent-{THE_GLOBAL_GROUP}"),
            format!("SecureBoot-{THE_GLOBAL_GROUP}"),
            format!("{AN_ENTRY}0001-11111111-2222-3333-4444-555555555555"),
            format!("{AN_ENTRY}1-{THE_GLOBAL_GROUP}"),
            format!("{AN_ENTRY}00001-{THE_GLOBAL_GROUP}"),
            "something else".to_owned(),
        ] {
            fs::write(firmware.variables().join(other), [7, 0, 0, 0, 1, 2]).unwrap();
        }
        let entries = firmware.entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries.first().unwrap().number(), 1);
    }

    /// **An entry that cannot be read is left out, and the rest are still
    /// read.** One unreadable entry must not take away a person's way into
    /// their own Windows, and leaving it out can only cause a refusal.
    #[test]
    fn an_entry_that_cannot_be_read_is_left_out_and_the_rest_are_read() {
        let firmware = a_firmware("unreadable");
        holding(
            &firmware,
            2,
            "Windows Boot Manager",
            THE_WINDOWS_LOADER_IN_A_PATH,
        );
        for (number, bytes) in [(4_u16, vec![7, 0, 0, 0, 1]), (5, Vec::new())] {
            fs::write(
                firmware
                    .variables()
                    .join(format!("{AN_ENTRY}{number:04X}-{THE_GLOBAL_GROUP}")),
                bytes,
            )
            .unwrap();
        }
        let entries = firmware.entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries.first().unwrap().number(), 2);
    }

    /// **A machine with no variables to read is said rather than read as
    /// having nothing to start** — which would be *there is no Windows here*
    /// on a machine that has one.
    #[test]
    fn a_machine_that_cannot_be_asked_says_so() {
        let firmware = TheFirmware::at(Path::new("/nonexistent-alo-starting/efivars"));
        let Err(NotAnswering(why)) = firmware.entries() else {
            panic!("a machine with no firmware to read answered");
        };
        assert!(why.contains("nonexistent-alo-starting"), "{why}");
    }

    /// **The next start is written as the firmware keeps it**: the attributes
    /// it expects, and the entry's number, and nothing else.
    #[test]
    fn the_next_start_is_written_as_the_firmware_keeps_it() {
        let firmware = a_firmware("next");
        firmware.start_next(0x1234).unwrap();
        let written = fs::read(
            firmware
                .variables()
                .join(format!("{NEXT_START}-{THE_GLOBAL_GROUP}")),
        )
        .unwrap();
        assert_eq!(written, vec![7, 0, 0, 0, 0x34, 0x12]);
        assert_eq!(written, the_next_start_written(0x1234));
    }

    /// **Writing the next start writes one file, and no other.** The order the
    /// machine ordinarily starts in is what a person's way back is made of, and
    /// this is the test that it was not touched.
    #[test]
    fn writing_the_next_start_leaves_every_other_variable_alone() {
        let firmware = a_firmware("only-next");
        holding(
            &firmware,
            1,
            "Windows Boot Manager",
            THE_WINDOWS_LOADER_IN_A_PATH,
        );
        let order = firmware
            .variables()
            .join(format!("BootOrder-{THE_GLOBAL_GROUP}"));
        fs::write(&order, [7, 0, 0, 0, 1, 0, 3, 0]).unwrap();
        let before: Vec<u8> = fs::read(&order).unwrap();

        firmware.start_next(1).unwrap();

        assert_eq!(fs::read(&order).unwrap(), before);
        let mut left: Vec<String> = fs::read_dir(firmware.variables())
            .unwrap()
            .flatten()
            .map(|each| each.file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(
            left,
            vec![
                format!("{AN_ENTRY}0001-{THE_GLOBAL_GROUP}"),
                format!("{NEXT_START}-{THE_GLOBAL_GROUP}"),
                format!("BootOrder-{THE_GLOBAL_GROUP}"),
            ]
        );
    }

    /// **A machine that will not be written to says so**, rather than
    /// answering as though a person's next start had been set.
    #[test]
    fn a_machine_that_will_not_be_written_to_says_so() {
        let firmware = TheFirmware::at(Path::new("/nonexistent-alo-starting/efivars"));
        let Err(NotDone(why)) = firmware.start_next(1) else {
            panic!("a machine with no firmware to write was told what to start");
        };
        assert!(why.contains(NEXT_START), "{why}");
    }

    /// **The name of a start-up entry is read strictly**, because everything
    /// else in that directory is another firmware variable.
    #[test]
    fn the_name_of_an_entry_is_read_strictly() {
        assert_eq!(
            the_entry_named(&format!("Boot0000-{THE_GLOBAL_GROUP}")),
            Some(0)
        );
        assert_eq!(
            the_entry_named(&format!("BootFFFF-{THE_GLOBAL_GROUP}")),
            Some(0xffff)
        );
        assert_eq!(
            the_entry_named(&format!("Boot000a-{}", THE_GLOBAL_GROUP.to_uppercase())),
            Some(10)
        );
        for name in [
            "Boot0001",
            "Boot-8be4df61-93ca-11d2-aa0d-00e098032b8c",
            "BootOrder-8be4df61-93ca-11d2-aa0d-00e098032b8c",
            "Bootzzzz-8be4df61-93ca-11d2-aa0d-00e098032b8c",
            "Boot0001-00000000-0000-0000-0000-000000000000",
            "",
        ] {
            assert_eq!(the_entry_named(name), None, "{name} was read as an entry");
        }
    }
}
