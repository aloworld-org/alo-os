//! The two systems a machine can start, and the one file Windows is started by.
//!
//! There are exactly two, because this is what a machine installed *alongside*
//! Windows has on it
//! ([ADR 0023](../../../docs/decisions/0023-installed-from-the-machine-it-replaces.md)
//! §4). A machine alo OS replaced has one, and nothing here is written to it.

/// The file Windows's own start-up program is, as a loader reads a path.
///
/// Forward slashes: this is how the configuration [`crate::Menu`] writes spells
/// it.
pub const THE_WINDOWS_LOADER: &str = "/EFI/Microsoft/Boot/bootmgfw.efi";

/// The same file, spelt the way a firmware's own start-up entry carries it and
/// the way Windows writes a path.
///
/// [`crate::Entry::starts_windows`] looks for this in what the firmware
/// reported, and `docs/booting.md` writes it this way for the same reason: it
/// is what a person has seen on their own machine.
pub const THE_WINDOWS_LOADER_IN_A_PATH: &str = "\\EFI\\Microsoft\\Boot\\bootmgfw.efi";

/// The name the generated menu gives its Windows entry, which is also what is
/// written into the environment block when Windows is chosen.
///
/// It is **ours**, not the base's: an identifier we generate and can therefore
/// compare, where alo OS's own entries are the base's and carry the kernel they
/// start in their names. [`crate::TheStartingChoice`] says why that asymmetry
/// is the honest shape rather than an oversight.
pub const THE_WINDOWS_ENTRY: &str = "alo-windows";

/// One of the two systems a machine with Windows still on it can start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum System {
    /// alo OS.
    AloOs,
    /// The Windows that was on the machine before alo OS.
    Windows,
}

impl System {
    /// Both of them, alo OS first, which is the order the menu offers them in.
    pub const BOTH: [Self; 2] = [Self::AloOs, Self::Windows];

    /// The other one.
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::AloOs => Self::Windows,
            Self::Windows => Self::AloOs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Each is the other's other, and there are two.**
    #[test]
    fn each_is_the_others_other() {
        assert_eq!(System::BOTH.len(), 2);
        for system in System::BOTH {
            assert_ne!(system.other(), system);
            assert_eq!(system.other().other(), system);
        }
    }

    /// **The loader is named once, as a loader reads a path**, and it is
    /// Windows's own rather than anything this repository ships.
    #[test]
    fn the_loader_is_windowss_own_and_is_named_once() {
        assert!(THE_WINDOWS_LOADER.starts_with('/'));
        assert!(!THE_WINDOWS_LOADER.contains('\\'));
        assert!(THE_WINDOWS_LOADER.ends_with("/bootmgfw.efi"));
        assert!(!THE_WINDOWS_ENTRY.contains(char::is_whitespace));
    }

    /// **The two spellings are one path.** A loader reads it with forward
    /// slashes and a firmware's entry carries it with backslashes, and a
    /// machine where the two had drifted would offer a Windows it could not
    /// recognise afterwards.
    #[test]
    fn the_two_spellings_of_the_loader_are_one_path() {
        assert_eq!(
            THE_WINDOWS_LOADER_IN_A_PATH.replace('\\', "/"),
            THE_WINDOWS_LOADER
        );
    }
}
