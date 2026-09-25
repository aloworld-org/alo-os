//! The systems the firmware can start, as the firmware's own tool lists them.
//!
//! Read for three things this environment has to know once alo OS is on the
//! disk: which entry starts what was just installed, what it is called, and
//! which entry starts the Windows beside it. Everything else on the list is
//! kept as it is and put back in the order it was in.
//!
//! # The shape, and why only this much of it is read
//!
//! `efibootmgr` prints one line for each entry — `Boot0004* Windows Boot
//! Manager` and then the device path — and one line for the order. What is
//! read here is the number, the name, and the file the entry starts, because
//! those are the three the tidying acts on. The rest of a device path is the
//! firmware's own business and is never rewritten by this environment
//! (ADR 0011).

/// The file the base's own loader is, on the disk alo OS was installed onto.
///
/// Measured on the installer's own road, 2026-09-22 and again 2026-09-23: what
/// `bootc install` leaves behind is an entry named *Fedora* that starts
/// `\EFI\fedora\shimx64.efi` on the installed disk's EFI system partition
/// (`docs/quirks.md`). The name is the base's; the file is what the firmware
/// has to be pointed at whatever the entry is called.
pub const THE_LOADER_THE_BASE_INSTALLS: &str = "\\EFI\\fedora\\shimx64.efi";

/// What the entry for alo OS is called once this environment has tidied up.
///
/// The installer's staging entry carries the same name
/// (`alo_installer::THE_ENTRYS_NAME`), and by the time this runs that entry is
/// gone: a person looking at their firmware's own menu sees one alo OS, which
/// is the installed one.
pub const THE_ENTRYS_NAME: &str = "alo OS";

/// One entry the firmware can start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Its number, as in `0004`.
    pub number: String,
    /// What it is called.
    pub named: String,
    /// The file it starts, when its device path names one.
    pub file: Option<String>,
    /// The partition that file is on, when its device path names one.
    pub partition: Option<u32>,
}

/// Every entry the firmware lists, and the order it starts them in.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Entries {
    /// Every entry, in the order they were printed.
    pub every: Vec<Entry>,
    /// The numbers in `BootOrder`, in order.
    pub order: Vec<String>,
}

impl Entries {
    /// What the firmware's own tool printed.
    #[must_use]
    pub fn read(printed: &str) -> Self {
        let mut every = Vec::new();
        let mut order = Vec::new();
        for line in printed.lines() {
            let line = line.trim_end();
            if let Some(rest) = line.strip_prefix("BootOrder:") {
                order = rest
                    .split(',')
                    .map(|number| number.trim().to_owned())
                    .filter(|number| !number.is_empty())
                    .collect();
                continue;
            }
            let Some(entry) = Entry::read(line) else {
                continue;
            };
            every.push(entry);
        }
        Self { every, order }
    }

    /// The entry that starts what `bootc install` left on the disk.
    #[must_use]
    pub fn the_installed_system(&self) -> Option<&Entry> {
        let mut starting = self
            .every
            .iter()
            .filter(|entry| entry.starts(THE_LOADER_THE_BASE_INSTALLS));
        match (starting.next(), starting.next()) {
            (Some(one), None) => Some(one),
            // Two entries starting the same loader is a machine nothing here
            // can reason about — on a computer alo OS was installed onto twice,
            // say — and it is left exactly as it is.
            _ => None,
        }
    }

    /// The entry that starts Windows, by the file it starts.
    #[must_use]
    pub fn windows(&self) -> Option<&Entry> {
        self.every
            .iter()
            .find(|entry| entry.starts(alo_starting::THE_WINDOWS_LOADER_IN_A_PATH))
    }

    /// The order alo OS first, Windows directly behind it, and everything else
    /// after them in the order the firmware already had
    /// ([ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
    /// term 1).
    ///
    /// Nothing is dropped: an entry the firmware lists and this does not
    /// recognise keeps its place behind the two.
    #[must_use]
    pub fn with_alo_os_first(&self, alo_os: &str, windows: Option<&str>) -> Vec<String> {
        let mut order: Vec<String> = vec![alo_os.to_owned()];
        if let Some(windows) = windows {
            order.push(windows.to_owned());
        }
        for number in self
            .order
            .iter()
            .chain(self.every.iter().map(|entry| &entry.number))
        {
            if !order.iter().any(|already| already == number) {
                order.push(number.clone());
            }
        }
        order
    }
}

impl Entry {
    /// One line of the firmware's list, when it is an entry.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        let rest = line.strip_prefix("Boot")?;
        let (number, rest) = rest.split_at_checked(4)?;
        if !number.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        // `*` marks an entry the firmware will start; both are entries.
        let rest = rest.strip_prefix('*').unwrap_or(rest);
        let rest = rest.strip_prefix(|c: char| c.is_whitespace())?;
        let (named, path) = rest.split_once('\t').unwrap_or((rest, ""));
        Some(Self {
            number: number.to_owned(),
            named: named.trim().to_owned(),
            file: file_in(path),
            partition: partition_in(path),
        })
    }

    /// Whether this entry starts that file, whichever way the path spells it.
    #[must_use]
    pub fn starts(&self, file: &str) -> bool {
        let wanted = file.replace('/', "\\").to_lowercase();
        self.file
            .as_ref()
            .is_some_and(|mine| mine.replace('/', "\\").to_lowercase() == wanted)
    }
}

/// The file a device path names, as in `File(\EFI\fedora\shimx64.efi)`.
fn file_in(path: &str) -> Option<String> {
    let (_, rest) = path.split_once("File(")?;
    let (file, _) = rest.split_once(')')?;
    Some(file.to_owned())
}

/// The partition a device path names, as in `HD(2,GPT,…)`.
fn partition_in(path: &str) -> Option<u32> {
    let (_, rest) = path.split_once("HD(")?;
    let (number, _) = rest.split_once(',')?;
    number.trim().parse().ok()
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;

    /// What the firmware's own tool printed on the walk's machine after an
    /// install, 2026-09-23 — the entry `bootc` left, Windows, and the
    /// firmware's own two.
    const AFTER_AN_INSTALL: &str = "BootCurrent: 000B\n\
         Timeout: 0 seconds\n\
         BootOrder: 000B,0004,0003,0000,0001\n\
         Boot0000* BootManagerMenuApp\tFvVol(5c60f367-a505-419a-859e-2a4ff6ca6fe5)\n\
         Boot0001* EFI Firmware Setup\tFvVol(5c60f367-a505-419a-859e-2a4ff6ca6fe5)\n\
         Boot0003* UEFI QEMU HARDDISK ALOWINDOWS1 \tPciRoot(0x0)/Pci(0x1f,0x2)\n\
         Boot0004* Windows Boot Manager\tHD(1,GPT,0506f28d-c7cb-426e-ad7b-b9ec92014753,0x800,0x96000)/File(\\EFI\\Microsoft\\Boot\\bootmgfw.efi)\n\
         Boot000B* Fedora\tHD(2,GPT,1505d88b-67da-4187-b683-f36a1169e81e,0x1000,0x100000)/File(\\EFI\\fedora\\shimx64.efi)\n";

    /// **The three things this environment reads are read**, and nothing else
    /// is invented.
    #[test]
    fn the_entries_and_the_order_are_read() {
        let entries = Entries::read(AFTER_AN_INSTALL);
        assert_eq!(entries.order, ["000B", "0004", "0003", "0000", "0001"]);
        assert_eq!(entries.every.len(), 5);

        let installed = entries.the_installed_system().expect("the installed entry");
        assert_eq!(installed.number, "000B");
        assert_eq!(installed.named, "Fedora");
        assert_eq!(installed.partition, Some(2));

        let windows = entries.windows().expect("Windows' own entry");
        assert_eq!(windows.number, "0004");
        assert_eq!(windows.partition, Some(1));

        // An entry with no file — the firmware's own menu — is an entry, and
        // starts nothing this looks for.
        let menu = &entries.every[0];
        assert_eq!(menu.named, "BootManagerMenuApp");
        assert_eq!(menu.file, None);
        assert!(!menu.starts(THE_LOADER_THE_BASE_INSTALLS));
    }

    /// **alo OS first, Windows directly behind it, and everything else after
    /// them in the order it was in** — and nothing is dropped.
    #[test]
    fn the_order_puts_windows_directly_behind_alo_os() {
        let entries = Entries::read(AFTER_AN_INSTALL);
        let order = entries.with_alo_os_first("000C", Some("0004"));
        assert_eq!(order, ["000C", "0004", "000B", "0003", "0000", "0001"]);
        assert_eq!(
            order.len(),
            entries.every.len() + 1,
            "an entry was dropped or said twice: {order:?}"
        );
    }

    /// **A computer with no Windows is ordered just as carefully.**
    #[test]
    fn a_machine_without_windows_still_has_an_order() {
        let entries = Entries::read(AFTER_AN_INSTALL);
        assert_eq!(
            entries.with_alo_os_first("000B", None),
            ["000B", "0004", "0003", "0000", "0001"]
        );
    }

    /// **Two entries starting the same loader is not a choice**, and nothing
    /// is renamed on such a machine.
    #[test]
    fn two_entries_for_one_loader_are_left_alone() {
        let twice = format!(
            "{AFTER_AN_INSTALL}Boot000C* Fedora\tHD(2,GPT,1505d88b-67da-4187-b683-f36a1169e81e,0x1000,0x100000)/File(\\EFI\\fedora\\shimx64.efi)\n"
        );
        assert_eq!(Entries::read(&twice).the_installed_system(), None);
    }

    /// **Nothing but an entry line is read as an entry.**
    #[test]
    fn nothing_else_is_an_entry() {
        for line in [
            "BootCurrent: 000B",
            "Timeout: 0 seconds",
            "BootOrder: 000B,0004",
            "Booting is hard",
            "",
            "BootZZZZ* Not a number\tFvVol(…)",
        ] {
            assert_eq!(Entry::read(line), None, "{line:?}");
        }
    }
}
