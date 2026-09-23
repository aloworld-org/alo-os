//! Whether the firmware's list of systems already has an entry named alo OS.
//!
//! `bcdedit /enum firmware` prints its headings in the language Windows is set
//! to, and its element names — `identifier`, `description` — in English on
//! every Windows. What is read is only whether any element's value is exactly
//! [`THE_ENTRYS_NAME`], so neither the headings nor the element names matter.

use crate::identities::Entry;
use crate::program::THE_ENTRYS_NAME;

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

/// The value of this element on this line, when the line is that element.
fn value_of<'line>(element: &str, line: &'line str) -> Option<&'line str> {
    let (name, value) = line.split_once(char::is_whitespace)?;
    (name == element).then(|| value.trim())
}

#[cfg(test)]
mod tests {
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
}
