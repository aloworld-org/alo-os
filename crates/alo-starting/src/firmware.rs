//! What the firmware reports about the systems on this machine, and the one
//! thing alo OS ever asks it to change.
//!
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md),
//! *what stays as it was*: **Restart into Windows** is a one-start switch
//! through the firmware's next-start entry, and it leaves the menu's default
//! untouched. So there is exactly one change here — [`Firmware::start_next`] —
//! and everything else is a read.
//!
//! # An entry is compared, never interpreted
//!
//! This is `alo-drives`' rule about a drive, for a start-up entry. Whoever
//! proposes the change digests the bytes the firmware reported for an entry;
//! whoever carries it out asks the firmware for its entries **now**, digests
//! each of them the same way, and acts on the one that matches — or on nothing.
//! The broker is never told a loader, a path, a disk or a number.
//!
//! # And it is checked for being Windows before it is used
//!
//! A matching entry is not enough. [`Entry::starts_windows`] looks for
//! Windows's own start-up program in the path the firmware reported, so a verb
//! named *start Windows next* cannot set the machine to start something else
//! that happened to digest to what was approved. The name the firmware gives an
//! entry is never what is checked: that name is whatever was typed when the
//! entry was made, in whatever language, and on a machine that has been
//! reinstalled it is often wrong.

use crate::systems::THE_WINDOWS_LOADER_IN_A_PATH;

/// The bytes an entry's identity is made over, before the entry itself.
///
/// A version in the name, so that a change to what is digested is a change to
/// this word rather than a silent disagreement between the side that approves
/// and the side that acts.
pub const AS_REPORTED: &[u8] = b"alo-starting entry 1";

/// How many bytes of a start-up entry come before its description: the
/// attributes, and the length of the path that follows the description.
const BEFORE_THE_DESCRIPTION: usize = 6;

/// Why something the firmware reported is not a start-up entry.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAnEntry {
    /// Fewer bytes than the fixed part of an entry.
    #[error(
        "the firmware reported {how_many} bytes for start-up entry {number:04X}, and an entry is \
         at least {BEFORE_THE_DESCRIPTION}"
    )]
    TooShort {
        /// Which entry.
        number: u16,
        /// How many bytes it was.
        how_many: usize,
    },

    /// A description that never ends.
    #[error("the firmware's start-up entry {number:04X} has a name that does not end")]
    NotEnded {
        /// Which entry.
        number: u16,
    },

    /// A description that is not text.
    #[error("the firmware's start-up entry {number:04X} has a name that is not text")]
    NotText {
        /// Which entry.
        number: u16,
    },

    /// A path shorter than the entry says it is.
    #[error(
        "the firmware's start-up entry {number:04X} says its path is {said} bytes and reported \
         {how_many}"
    )]
    NotLongEnough {
        /// Which entry.
        number: u16,
        /// How long the entry said its path was.
        said: usize,
        /// How much was there.
        how_many: usize,
    },
}

/// One of the firmware's start-up entries, as the firmware reported it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The number the firmware keeps it under.
    number: u16,
    /// The name the firmware shows for it.
    described: String,
    /// Exactly the bytes the firmware reported, which the identity is made
    /// over.
    reported: Vec<u8>,
}

impl Entry {
    /// The entry the firmware reported under this number, as these bytes.
    ///
    /// # Errors
    /// [`NotAnEntry`], naming the entry and what was wrong with it. Nothing is
    /// guessed at and nothing is repaired: an entry this could not read is an
    /// entry alo OS does not act on.
    pub fn reported(number: u16, bytes: &[u8]) -> Result<Self, NotAnEntry> {
        let after_the_head = bytes
            .get(BEFORE_THE_DESCRIPTION..)
            .ok_or(NotAnEntry::TooShort {
                number,
                how_many: bytes.len(),
            })?;
        let said = bytes
            .get(4..BEFORE_THE_DESCRIPTION)
            .and_then(|pair| <[u8; 2]>::try_from(pair).ok())
            .map(|pair| usize::from(u16::from_le_bytes(pair)))
            .ok_or(NotAnEntry::TooShort {
                number,
                how_many: bytes.len(),
            })?;

        let mut name = Vec::new();
        let mut ended = false;
        let mut used = 0;
        for pair in after_the_head.as_chunks::<2>().0 {
            used += 2;
            let unit = u16::from_le_bytes(*pair);
            if unit == 0 {
                ended = true;
                break;
            }
            name.push(unit);
        }
        if !ended {
            return Err(NotAnEntry::NotEnded { number });
        }
        let described = String::from_utf16(&name).map_err(|_| NotAnEntry::NotText { number })?;

        let path = after_the_head
            .len()
            .checked_sub(used)
            .ok_or(NotAnEntry::NotEnded { number })?;
        if path < said {
            return Err(NotAnEntry::NotLongEnough {
                number,
                said,
                how_many: path,
            });
        }

        Ok(Self {
            number,
            described,
            reported: bytes.to_owned(),
        })
    }

    /// The number the firmware keeps this entry under.
    #[must_use]
    pub const fn number(&self) -> u16 {
        self.number
    }

    /// The name the firmware shows for it.
    ///
    /// For a person choosing among what is on their machine, and for nothing
    /// else: no decision anywhere reads it.
    #[must_use]
    pub fn described(&self) -> &str {
        &self.described
    }

    /// The bytes this entry's identity is made over.
    ///
    /// The word above, a zero byte, and exactly what the firmware reported —
    /// so an identity made by whoever proposes the change and one made by
    /// whoever carries it out are the same bytes or are different entries.
    ///
    /// **The number is deliberately not part of it.** An entry names what it
    /// starts; the slot a firmware happens to keep it in is where it is, and a
    /// firmware that renumbers its entries would otherwise turn a person's
    /// approval into a refusal. The cost is that the same entry written twice
    /// is one identity and two matches, which is exactly the case
    /// `alo_brokerd::NextStart` refuses rather than guesses at.
    #[must_use]
    pub fn as_reported(&self) -> Vec<u8> {
        let mut over = AS_REPORTED.to_vec();
        over.push(0);
        over.extend_from_slice(&self.reported);
        over
    }

    /// Whether this entry starts Windows.
    ///
    /// True when the bytes the firmware reported carry Windows's own start-up
    /// program in a path, whatever case it is written in — which is what a
    /// firmware's entry for Windows holds and what an entry for anything else
    /// does not.
    #[must_use]
    pub fn starts_windows(&self) -> bool {
        let looked_for: Vec<u16> = THE_WINDOWS_LOADER_IN_A_PATH
            .to_ascii_lowercase()
            .encode_utf16()
            .collect();
        let reported: Vec<u16> = self
            .reported
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                let unit = u16::from_le_bytes(*pair);
                match u8::try_from(unit) {
                    Ok(byte) => u16::from(byte.to_ascii_lowercase()),
                    Err(_) => unit,
                }
            })
            .collect();
        reported
            .windows(looked_for.len())
            .any(|each| each == looked_for)
    }
}

/// This machine could not be asked what it can start.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct NotAnswering(pub String);

/// This machine would not be told what to start next.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct NotDone(pub String);

/// The firmware of the machine alo OS is running on.
///
/// Two methods, and one of them changes anything: a machine's start-up order is
/// somebody's only way back into their own computer, so what can be written
/// through this is the **next start** and nothing else. There is deliberately
/// no way here to change the order, to add an entry or to remove one.
pub trait Firmware {
    /// Every start-up entry the firmware has, now.
    ///
    /// # Errors
    /// [`NotAnswering`] when the firmware could not be asked at all.
    fn entries(&self) -> Result<Vec<Entry>, NotAnswering>;

    /// Start the entry with this number next time, once.
    ///
    /// The machine's ordinary start-up order is left exactly as it was: the
    /// start after next is the one it would have been.
    ///
    /// # Errors
    /// [`NotDone`] when the firmware would not be told.
    fn start_next(&self, entry: u16) -> Result<(), NotDone>;
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::as_a_firmware_reports_it;

    /// **An entry reads back as its name and its own bytes.**
    #[test]
    fn an_entry_reads_back_as_its_name_and_its_own_bytes() {
        let bytes = as_a_firmware_reports_it("Windows Boot Manager", THE_WINDOWS_LOADER_IN_A_PATH);
        let entry = Entry::reported(1, &bytes).unwrap();
        assert_eq!(entry.number(), 1);
        assert_eq!(entry.described(), "Windows Boot Manager");
        assert!(entry.as_reported().ends_with(&bytes));
        assert!(entry.as_reported().starts_with(AS_REPORTED));
    }

    /// **Two entries with different bytes have different identities**, which is
    /// the whole of what a verb's argument is.
    #[test]
    fn two_entries_are_told_apart_by_what_was_reported() {
        let windows = Entry::reported(
            1,
            &as_a_firmware_reports_it("Windows Boot Manager", THE_WINDOWS_LOADER_IN_A_PATH),
        )
        .unwrap();
        let alo = Entry::reported(
            2,
            &as_a_firmware_reports_it("alo OS", "\\EFI\\fedora\\shimx64.efi"),
        )
        .unwrap();
        assert_ne!(windows.as_reported(), alo.as_reported());
    }

    /// **Windows is recognised by its own program and not by its name**, in
    /// whatever case a firmware wrote the path in.
    #[test]
    fn windows_is_recognised_by_its_program_in_any_case() {
        for path in [
            "\\EFI\\Microsoft\\Boot\\bootmgfw.efi",
            "\\efi\\microsoft\\boot\\bootmgfw.efi",
            "\\EFI\\MICROSOFT\\BOOT\\BOOTMGFW.EFI",
        ] {
            let entry = Entry::reported(1, &as_a_firmware_reports_it("anything at all", path));
            assert!(entry.unwrap().starts_windows(), "{path}");
        }
    }

    /// **An entry named after Windows that does not start it is not Windows.**
    /// The name is whatever somebody typed, and on a machine that has been
    /// reinstalled it is regularly wrong — so a verb named *start Windows next*
    /// would otherwise set the machine to start whatever this entry is.
    #[test]
    fn an_entry_called_windows_that_starts_something_else_is_not_windows() {
        for path in [
            "\\EFI\\Boot\\bootx64.efi",
            "\\EFI\\Microsoft\\Boot\\bootmgfw.txt",
            "\\EFI\\Microsoft\\Boot\\",
            "",
        ] {
            let entry = Entry::reported(1, &as_a_firmware_reports_it("Windows Boot Manager", path))
                .unwrap();
            assert!(!entry.starts_windows(), "{path}");
        }
    }

    /// **Anything that is not a start-up entry is refused rather than
    /// repaired** — each of the four ways the bytes can be wrong, named.
    #[test]
    fn what_is_not_an_entry_is_refused() {
        assert_eq!(
            Entry::reported(3, &[1, 2, 3]),
            Err(NotAnEntry::TooShort {
                number: 3,
                how_many: 3
            })
        );

        let mut never_ends = 1_u32.to_le_bytes().to_vec();
        never_ends.extend_from_slice(&0_u16.to_le_bytes());
        never_ends.extend_from_slice(&[b'a', 0, b'b', 0]);
        assert_eq!(
            Entry::reported(4, &never_ends),
            Err(NotAnEntry::NotEnded { number: 4 })
        );

        let mut half_a_character = 1_u32.to_le_bytes().to_vec();
        half_a_character.extend_from_slice(&0_u16.to_le_bytes());
        half_a_character.extend_from_slice(&[0xff, 0xd8, 0, 0]);
        assert_eq!(
            Entry::reported(5, &half_a_character),
            Err(NotAnEntry::NotText { number: 5 })
        );

        let mut short_path = 1_u32.to_le_bytes().to_vec();
        short_path.extend_from_slice(&64_u16.to_le_bytes());
        short_path.extend_from_slice(&[b'a', 0, 0, 0]);
        assert_eq!(
            Entry::reported(6, &short_path),
            Err(NotAnEntry::NotLongEnough {
                number: 6,
                said: 64,
                how_many: 0
            })
        );
    }

    /// **An entry whose name is empty is still an entry.** A firmware that
    /// reports one is unusual and not wrong, and refusing it would take a
    /// person's only Windows away over a blank label.
    #[test]
    fn an_entry_with_no_name_is_still_an_entry() {
        let entry = Entry::reported(
            7,
            &as_a_firmware_reports_it("", THE_WINDOWS_LOADER_IN_A_PATH),
        )
        .unwrap();
        assert_eq!(entry.described(), "");
        assert!(entry.starts_windows());
    }
}
