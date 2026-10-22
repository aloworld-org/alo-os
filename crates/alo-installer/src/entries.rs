//! Which of the firmware's listed systems is which — asked twice: whether one
//! is alo OS, and which one starts a removable device.
//!
//! `bcdedit /enum firmware` prints its headings in the language Windows is set
//! to, and its element names — `identifier`, `description` — in English on
//! every Windows. What is read is only whether any element's value is exactly
//! [`THE_ENTRYS_NAME`], so neither the headings nor the element names matter.

use crate::identities::Entry;
use crate::program::THE_ENTRYS_NAME;

/// What firmware puts in the description of an entry for a removable device.
///
/// Firmware writes these itself and in its own words — *UEFI: SanDisk Ultra,
/// Partition 1* on one machine, *UEFI OS* on another — so this is a convention
/// rather than an interface, and it is used only to narrow the search before
/// the device's own name decides it.
const WHAT_FIRMWARE_CALLS_A_DEVICE: &str = "UEFI";

/// Whether an entry named alo OS is among what was printed.
#[must_use]
pub fn one_is_named_alo_os(printed: &str) -> bool {
    printed.lines().any(is_the_name)
}

/// The identifier of the one entry named alo OS, when there is exactly one.
///
/// Read from the block it is in: `bcdedit` prints an entry as a heading, then
/// its elements one to a line, and the identifier belongs to the entry whose
/// description is read under it. **Two entries of that name is not a choice**
/// — a computer with two is one nothing here can reason about — so it answers
/// with nothing, and the caller says so.
#[must_use]
pub fn the_one_named_alo_os(printed: &str) -> Option<Entry> {
    let mut identifier: Option<&str> = None;
    let mut found: Vec<&str> = Vec::new();
    for line in printed.lines() {
        let line = line.trim();
        if line.is_empty() {
            identifier = None;
            continue;
        }
        if let Some(value) = value_of("identifier", line) {
            identifier = Some(value);
        }
        if is_the_name(line)
            && let Some(its) = identifier
        {
            found.push(its);
        }
    }
    match found.as_slice() {
        [one] => Entry::of(one),
        _ => None,
    }
}

/// Whether this line's value is exactly the entry's name.
fn is_the_name(line: &str) -> bool {
    line.trim()
        .split_once(char::is_whitespace)
        .is_some_and(|(_, value)| value.trim() == THE_ENTRYS_NAME)
}

/// The entry that starts this removable device, when exactly one does.
///
/// **`docs/features.md`: *and Windows starts the stick, so nobody hunts for a
/// boot key.*** Restarting straight into it means pointing the firmware's
/// one-shot boot sequence at the entry the firmware keeps for that device, and
/// `Program::StartingTheEntryNext` takes the identifier this returns.
///
/// # Matched on the device's own name, and refused when the firmware gives none
///
/// There is no interface that says *this entry is that disk*. What there is:
/// the disk's friendly name as `Get-Disk` reports it — *SanDisk Ultra* — and a
/// description the firmware usually builds from the same string. So the search
/// is narrowed to entries whose description looks like a device's, then decided
/// by whether the disk's name is in it.
///
/// **[`None`] is a real and ordinary answer.** A firmware that lists the stick
/// as *UEFI OS*, or does not list it until somebody picks it once by hand,
/// cannot be matched — and the honest thing is to say so and let a person pick
/// it from their boot menu, rather than point the next start at whichever
/// removable entry happened to come first. **Two matches is [`None`] too**, for
/// the reason [`the_one_named_alo_os`] gives: a computer with two is not
/// something this can reason about.
#[must_use]
pub fn the_one_that_starts(printed: &str, named: &str) -> Option<Entry> {
    let named = named.trim();
    if named.is_empty() {
        return None;
    }
    let wanted = named.to_lowercase();
    let mut identifier: Option<&str> = None;
    let mut found: Vec<&str> = Vec::new();
    for line in printed.lines() {
        let line = line.trim();
        if line.is_empty() {
            identifier = None;
            continue;
        }
        if let Some(value) = value_of("identifier", line) {
            identifier = Some(value);
        }
        if let Some(description) = value_of("description", line)
            && description.contains(WHAT_FIRMWARE_CALLS_A_DEVICE)
            && description.to_lowercase().contains(&wanted)
            && let Some(its) = identifier
        {
            found.push(its);
        }
    }
    match found.as_slice() {
        [one] => Entry::of(one),
        _ => None,
    }
}

/// The value of this element on this line, when the line is that element.
fn value_of<'line>(element: &str, line: &'line str) -> Option<&'line str> {
    let (name, value) = line.split_once(char::is_whitespace)?;
    (name == element).then(|| value.trim())
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "in a test, a panic naming what is wrong is the failure being reported"
    )]

    use super::*;

    /// The firmware's list on a machine with Windows alone, in German.
    const WINDOWS_ALONE: &str = "Firmware-Start-Manager\n\
         ---------------------\n\
         identifier              {fwbootmgr}\n\
         displayorder            {bootmgr}\n\
         timeout                 2\n\
         \n\
         Windows-Start-Manager\n\
         ---------------------\n\
         identifier              {bootmgr}\n\
         device                  partition=\\Device\\HarddiskVolume1\n\
         path                    \\EFI\\Microsoft\\Boot\\bootmgfw.efi\n\
         description             Windows Boot Manager\n";

    /// **An entry an earlier start left is found**, and Windows alone is not
    /// mistaken for one.
    #[test]
    fn an_entry_an_earlier_start_left_is_found() {
        assert!(!one_is_named_alo_os(WINDOWS_ALONE));
        let left = format!(
            "{WINDOWS_ALONE}\nFirmware-Anwendung (101fffff)\n\
             identifier              {{6b1d5c2a-0f3e-11ef-9a1b-00155d012345}}\n\
             device                  partition=E:\n\
             description             alo OS\n"
        );
        assert!(one_is_named_alo_os(&left));
    }

    /// **The identifier read is the one in the entry's own block**, and two
    /// entries of that name are not one to choose between.
    #[test]
    fn the_entry_named_alo_os_is_found_by_its_own_identifier() {
        let one = format!(
            "{WINDOWS_ALONE}\nFirmware Application (101fffff)\n\
             identifier              {{6b1d5c2a-0f3e-11ef-9a1b-00155d012345}}\n\
             device                  partition=E:\n\
             description             alo OS\n"
        );
        assert_eq!(
            the_one_named_alo_os(&one).map(|entry| entry.as_str().to_owned()),
            Some("{6b1d5c2a-0f3e-11ef-9a1b-00155d012345}".to_owned())
        );
        assert_eq!(the_one_named_alo_os(WINDOWS_ALONE), None);
        let two = format!(
            "{one}\nFirmware Application (101fffff)\n\
             identifier              {{7c2e6d3b-1a4f-22ef-8b2c-00155d0abcde}}\n\
             description             alo OS\n"
        );
        assert_eq!(the_one_named_alo_os(&two), None);
    }

    /// A name that only contains alo OS is not the name.
    #[test]
    fn a_name_that_only_contains_it_is_not_it() {
        assert!(!one_is_named_alo_os(
            "description             alo OS Beta\n"
        ));
        assert!(!one_is_named_alo_os("alo OS\n"));
    }

    /// The firmware's list on a machine with a stick plugged in, in German —
    /// the same locale the fixture above uses, for the same reason: the
    /// headings are translated and the element names are not.
    ///
    /// **Shaped from `bcdedit`'s documented output and the fixture above, not
    /// captured from a machine.** This build machine has no Windows. What a
    /// real firmware writes in a removable entry's description is the part
    /// least certain here, which is why the lookup refuses rather than guesses
    /// when it does not match.
    const A_STICK_IS_IN: &str = "Firmware-Start-Manager\n\
         -----------------------\n\
         identifier              {fwbootmgr}\n\
         \n\
         Windows-Start-Manager\n\
         ---------------------\n\
         identifier              {bootmgr}\n\
         description             Windows Boot Manager\n\
         \n\
         Firmware Application (101fffff)\n\
         -------------------------------\n\
         identifier              {a1b2c3d4-0001-11ef-9a1b-00155d012345}\n\
         description             UEFI: SanDisk Ultra, Partition 1\n";

    /// **The entry that starts the stick is found by the disk's own name.**
    #[test]
    fn the_entry_that_starts_the_stick_is_found_by_the_disks_name() {
        let found = the_one_that_starts(A_STICK_IS_IN, "SanDisk Ultra")
            .expect("the firmware names this device in an entry of its own");
        assert_eq!(found.as_str(), "{a1b2c3d4-0001-11ef-9a1b-00155d012345}");
    }

    /// **However it is capitalised**, because the firmware's capitals are the
    /// firmware's and `Get-Disk`'s are Windows'.
    #[test]
    fn the_disks_name_is_matched_however_it_is_capitalised() {
        assert!(the_one_that_starts(A_STICK_IS_IN, "sandisk ULTRA").is_some());
    }

    /// **A firmware that does not name the device is refused, not guessed at.**
    ///
    /// This is the case the whole function exists to get right: an entry that
    /// is plainly a removable device but says only *UEFI OS*. Pointing the next
    /// start at it because it is the only one would restart a person into
    /// whatever that is.
    #[test]
    fn a_device_the_firmware_does_not_name_is_refused() {
        let nameless = A_STICK_IS_IN.replace("UEFI: SanDisk Ultra, Partition 1", "UEFI OS");
        assert_eq!(the_one_that_starts(&nameless, "SanDisk Ultra"), None);
        // The control: the same list with the name in it does answer, so the
        // refusal is about the description and not about the fixture.
        assert!(the_one_that_starts(A_STICK_IS_IN, "SanDisk Ultra").is_some());
    }

    /// **Two of the same device is not a choice.**
    ///
    /// The same rule the entry above it keeps: a computer with two is one
    /// nothing here can reason about, so it answers with nothing.
    #[test]
    fn two_entries_for_one_device_are_not_a_choice() {
        let twice = format!(
            "{A_STICK_IS_IN}\nFirmware Application (101fffff)\n\
             -------------------------------\n\
             identifier              {{b2c3d4e5-0002-11ef-9a1b-00155d012345}}\n\
             description             UEFI: SanDisk Ultra, Partition 2\n"
        );
        assert_eq!(the_one_that_starts(&twice, "SanDisk Ultra"), None);
    }

    /// **Windows' own entry is never taken for a stick**, even when the name
    /// asked for appears in the list for another reason.
    #[test]
    fn windows_own_entry_is_never_taken_for_a_stick() {
        // A description with the name but without what firmware calls a
        // device: the narrowing is what keeps this out.
        let confusable = A_STICK_IS_IN.replace(
            "UEFI: SanDisk Ultra, Partition 1",
            "Windows on SanDisk Ultra",
        );
        assert_eq!(the_one_that_starts(&confusable, "SanDisk Ultra"), None);
    }

    /// **No name is no answer**, rather than the first removable entry.
    #[test]
    fn no_name_is_no_answer() {
        for nothing in ["", "   ", "\n"] {
            assert_eq!(
                the_one_that_starts(A_STICK_IS_IN, nothing),
                None,
                "{nothing:?}"
            );
        }
    }

    /// **The identifier is the one in the entry's own block.**
    ///
    /// *Added because a mutation found its absence*: deleting the reset on a
    /// blank line changed nothing, because no fixture had an entry whose
    /// description matched while its identifier belonged to the block above.
    /// Without the reset this answers with the **previous** entry's identifier,
    /// and a person who asked for the stick is restarted into whatever that
    /// was — the same fault [`the_one_named_alo_os`] already had a test for,
    /// which is what made the gap worth looking for.
    ///
    /// **And the first version of this test was vacuous**, which is worth more
    /// than the test. Its preceding block carried `{bootmgr}`, and
    /// `Entry::of` takes **only a braced GUID** — so without the reset the
    /// function still answered `None`, because the identifier it wrongly
    /// carried was one `Entry::of` rejects. The assertion passed for a reason
    /// that had nothing to do with what it claimed. The block below carries a
    /// GUID, so the wrong answer is now a reachable one.
    #[test]
    fn the_identifier_is_the_one_in_the_entrys_own_block() {
        let no_identifier = "Some other system\n\
             -----------------\n\
             identifier              {c3d4e5f6-0003-11ef-9a1b-00155d012345}\n\
             description             Some other system\n\
             \n\
             Firmware Application (101fffff)\n\
             -------------------------------\n\
             description             UEFI: SanDisk Ultra, Partition 1\n";
        // The control: that identifier is one `Entry::of` accepts, so the
        // wrong answer this guards against is an answer it could give.
        assert!(
            Entry::of("{c3d4e5f6-0003-11ef-9a1b-00155d012345}").is_some(),
            "the control: without this, None could mean the identifier was merely rejected"
        );
        assert_eq!(the_one_that_starts(no_identifier, "SanDisk Ultra"), None);
    }

    /// **An empty list answers nothing** and does not panic.
    #[test]
    fn an_empty_list_answers_nothing() {
        assert_eq!(the_one_that_starts("", "SanDisk Ultra"), None);
    }
}
