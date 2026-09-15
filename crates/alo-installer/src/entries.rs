//! Whether the firmware's list of systems already has an entry named alo OS.
//!
//! `bcdedit /enum firmware` prints its headings in the language Windows is set
//! to, and its element names — `identifier`, `description` — in English on
//! every Windows. What is read is only whether any element's value is exactly
//! [`THE_ENTRYS_NAME`], so neither the headings nor the element names matter.

use crate::program::THE_ENTRYS_NAME;

/// Whether an entry named alo OS is among what was printed.
#[must_use]
pub fn one_is_named_alo_os(printed: &str) -> bool {
    printed.lines().any(|line| {
        line.trim()
            .split_once(char::is_whitespace)
            .is_some_and(|(_, value)| value.trim() == THE_ENTRYS_NAME)
    })
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

    /// A name that only contains alo OS is not the name.
    #[test]
    fn a_name_that_only_contains_it_is_not_it() {
        assert!(!one_is_named_alo_os(
            "description             alo OS Beta\n"
        ));
        assert!(!one_is_named_alo_os("alo OS\n"));
    }
}
