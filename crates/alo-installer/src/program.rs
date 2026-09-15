//! Every program this installer runs on Windows, and nothing else.
//!
//! Law 2 binds an agent and there is no agent here. The shape is kept anyway,
//! because it is the shape that makes the most dangerous program alo OS ships
//! reviewable: every program is a variant, its arguments are typed values that
//! were already checked (`crate::identities`, and byte counts this installer
//! computed), and there is no variant that takes a command or a line of text.
//! A reader who wants to know everything this installer can do to a computer
//! reads this file.
//!
//! **Disk and start-up changes go through Windows' own tools** — the storage
//! cmdlets of Windows PowerShell and `bcdedit` — and never through an IOCTL
//! this crate would have to write (the installer plan's task 3; `unsafe_code`
//! is forbidden). Every script is a constant with numbers put into it, handed
//! over by `crate::encoded` so no character of it is re-read on the way.
//!
//! | Reads, and change nothing | |
//! |---|---|
//! | [`Program::AskingWhetherThisIsAnAdministrator`] | the process's groups |
//! | [`Program::ReadingHowItStarts`] | UEFI or BIOS, and whether Secure Boot is on |
//! | [`Program::ReadingTheTpm`] | whether there is a TPM, and whether it is ready |
//! | [`Program::ReadingBitLocker`] | the Windows volume's encryption state |
//! | [`Program::ReadingTheMemory`] | installed memory |
//! | [`Program::ReadingTheWindowsVolume`] | where Windows is, its size, how far it can shrink |
//! | [`Program::ListingTheDisks`] | every disk and its partitions |
//! | [`Program::ListingTheStartEntries`] | the systems the firmware can start |
//!
//! | Changes, in the order they are made | Put back by |
//! |---|---|
//! | [`Program::Shrinking`] | [`Program::GrowingWindowsBack`] |
//! | [`Program::MakingTheArea`] | [`Program::RemovingTheArea`] |
//! | [`Program::PreparingTheArea`] | removing the area |
//! | [`Program::AddingTheEntry`] | [`Program::RemovingTheEntry`] |
//! | [`Program::PointingTheEntryAtTheArea`], [`Program::PointingTheEntryAtTheLoader`], [`Program::ListingTheEntry`] | removing the entry |
//! | [`Program::TakingAwayTheLetter`] | nothing to put back |
//! | [`Program::StartingTheEntryNext`] | [`Program::ForgettingTheNextStart`] |
//! | [`Program::Restarting`] | — the point after which this program is gone |

use crate::encoded;
use crate::identities::{DiskNumber, Entry, Letter, PartitionNumber};

/// The type Windows gives a basic data partition, which is what the installer's
/// FAT area is.
pub const BASIC_DATA: &str = "{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}";

/// What the entry is called among the systems a computer can start.
pub const THE_ENTRYS_NAME: &str = "alo OS";

/// Where, on the area, the firmware starts the environment from.
pub const THE_LOADER: &str = "\\EFI\\BOOT\\BOOTX64.EFI";

/// What every script starts with: stop at the first error rather than carry
/// on past it, draw no progress bars into the output, and print UTF-8 so a
/// disk's maker's name arrives as it was written.
const EVERY_SCRIPT_BEGINS: &str = "$ErrorActionPreference = 'Stop'; \
     $ProgressPreference = 'SilentlyContinue'; \
     [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; ";

/// One program this installer runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Program {
    /// Whether this process holds an administrator's rights.
    AskingWhetherThisIsAnAdministrator,
    /// UEFI or BIOS, and Secure Boot.
    ReadingHowItStarts,
    /// The TPM.
    ReadingTheTpm,
    /// BitLocker on the Windows volume.
    ReadingBitLocker,
    /// Installed memory.
    ReadingTheMemory,
    /// The Windows volume: its disk, partition, place, size, and how far it shrinks.
    ReadingTheWindowsVolume,
    /// Every disk and its partitions.
    ListingTheDisks,
    /// The firmware's list of systems.
    ListingTheStartEntries,

    /// Shrink the Windows partition to this many bytes.
    Shrinking {
        /// Its disk.
        disk: DiskNumber,
        /// The Windows partition.
        partition: PartitionNumber,
        /// Its new size.
        to: u64,
    },
    /// Make the area, at exactly this place and size, and print its number.
    MakingTheArea {
        /// The disk Windows is on.
        disk: DiskNumber,
        /// Where the space freed by shrinking begins.
        offset: u64,
        /// How big.
        size: u64,
    },
    /// Format the area as FAT with the installer's label, give it a letter,
    /// and print the letter.
    PreparingTheArea {
        /// Its disk.
        disk: DiskNumber,
        /// Its number.
        partition: PartitionNumber,
        /// Where it must begin — checked before it is formatted, so no other
        /// partition can be.
        offset: u64,
    },
    /// Make a start-up entry named [`THE_ENTRYS_NAME`], and print its identifier.
    AddingTheEntry,
    /// Point the entry at the area.
    PointingTheEntryAtTheArea {
        /// The entry.
        entry: Entry,
        /// The area's letter.
        letter: Letter,
    },
    /// Point the entry at the loader on the area.
    PointingTheEntryAtTheLoader {
        /// The entry.
        entry: Entry,
    },
    /// Put the entry last among the systems the firmware lists.
    ListingTheEntry {
        /// The entry.
        entry: Entry,
    },
    /// Take the area's letter away, and stop Windows giving it one again.
    TakingAwayTheLetter {
        /// Its disk.
        disk: DiskNumber,
        /// Its number.
        partition: PartitionNumber,
        /// The letter it has.
        letter: Letter,
    },
    /// Start the entry on the next start, once.
    StartingTheEntryNext {
        /// The entry.
        entry: Entry,
    },
    /// Restart the computer.
    Restarting,

    /// Forget the one-time next start.
    ForgettingTheNextStart,
    /// Remove the entry.
    RemovingTheEntry {
        /// The entry.
        entry: Entry,
    },
    /// Remove the area — only if it is still where it was made.
    RemovingTheArea {
        /// Its disk.
        disk: DiskNumber,
        /// Its number.
        partition: PartitionNumber,
        /// Where it was made.
        offset: u64,
    },
    /// Grow the Windows partition back to the size it had.
    GrowingWindowsBack {
        /// Its disk.
        disk: DiskNumber,
        /// The Windows partition.
        partition: PartitionNumber,
        /// The size it had.
        to: u64,
    },
}

/// Which of Windows' own programs runs, beneath the system directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    /// Windows PowerShell 5.1, which every Windows 10 and 11 has.
    PowerShell,
    /// The start-up configuration editor.
    Bcdedit,
    /// The tool that lists a process's groups.
    Whoami,
    /// The tool that restarts the computer.
    Shutdown,
}

impl Tool {
    /// Where it is, beneath `%SystemRoot%\System32`.
    ///
    /// Whole paths from the system directory, so nothing on a search path — a
    /// file beside the download, say — decides which program runs.
    #[must_use]
    pub fn beneath_the_system_directory(self) -> &'static str {
        match self {
            Self::PowerShell => "WindowsPowerShell\\v1.0\\powershell.exe",
            Self::Bcdedit => "bcdedit.exe",
            Self::Whoami => "whoami.exe",
            Self::Shutdown => "shutdown.exe",
        }
    }
}

impl Program {
    /// Whether it changes the computer.
    ///
    /// The one question a test of a refusal asks of every program that ran.
    #[must_use]
    pub fn changes(&self) -> bool {
        !matches!(
            self,
            Self::AskingWhetherThisIsAnAdministrator
                | Self::ReadingHowItStarts
                | Self::ReadingTheTpm
                | Self::ReadingBitLocker
                | Self::ReadingTheMemory
                | Self::ReadingTheWindowsVolume
                | Self::ListingTheDisks
                | Self::ListingTheStartEntries
        )
    }

    /// Which tool runs it.
    #[must_use]
    pub fn tool(&self) -> Tool {
        match self {
            Self::AskingWhetherThisIsAnAdministrator => Tool::Whoami,
            Self::Restarting => Tool::Shutdown,
            Self::ListingTheStartEntries
            | Self::AddingTheEntry
            | Self::PointingTheEntryAtTheArea { .. }
            | Self::PointingTheEntryAtTheLoader { .. }
            | Self::ListingTheEntry { .. }
            | Self::StartingTheEntryNext { .. }
            | Self::ForgettingTheNextStart
            | Self::RemovingTheEntry { .. } => Tool::Bcdedit,
            _ => Tool::PowerShell,
        }
    }

    /// The script, for a program Windows PowerShell runs.
    #[must_use]
    pub fn script(&self) -> Option<String> {
        let body = match self {
            Self::ReadingHowItStarts => "$secure = $null; \
                 try { $secure = [bool](Confirm-SecureBootUEFI) } catch { $secure = $null }; \
                 ConvertTo-Json -Compress -InputObject ([ordered]@{ \
                   FirmwareType = [string]$env:firmware_type; SecureBoot = $secure })"
                .to_owned(),
            // Unelevated, `Get-Tpm` does not fail: it returns the sentence
            // *Administrator privilege is required* as a string, whose
            // `TpmPresent` is nothing and would read as *no TPM*. Measured on
            // this repository's development machine on 2026-09-15; an answer
            // without the property is thrown, and so is not known.
            Self::ReadingTheTpm => "$tpm = Get-Tpm; \
                 if ($null -eq $tpm.PSObject.Properties['TpmPresent']) { throw 'the TPM did not answer' }; \
                 ConvertTo-Json -Compress -InputObject ([ordered]@{ \
                   TpmPresent = [bool]$tpm.TpmPresent; TpmReady = [bool]$tpm.TpmReady })"
                .to_owned(),
            Self::ReadingBitLocker => "$volume = Get-BitLockerVolume -MountPoint $env:SystemDrive; \
                 ConvertTo-Json -Compress -InputObject ([ordered]@{ \
                   VolumeStatus = [string]$volume.VolumeStatus; \
                   ProtectionStatus = [string]$volume.ProtectionStatus })"
                .to_owned(),
            Self::ReadingTheMemory => "$system = Get-CimInstance -ClassName Win32_ComputerSystem; \
                 ConvertTo-Json -Compress -InputObject ([ordered]@{ \
                   TotalPhysicalMemory = [uint64]$system.TotalPhysicalMemory })"
                .to_owned(),
            Self::ReadingTheWindowsVolume => "$letter = $env:SystemDrive.Substring(0, 1); \
                 $partition = Get-Partition -DriveLetter $letter; \
                 $supported = Get-PartitionSupportedSize -DiskNumber $partition.DiskNumber \
                   -PartitionNumber $partition.PartitionNumber; \
                 $volume = Get-Volume -DriveLetter $letter; \
                 ConvertTo-Json -Compress -InputObject ([ordered]@{ \
                   DriveLetter = [string]$letter; \
                   DiskNumber = [uint32]$partition.DiskNumber; \
                   PartitionNumber = [uint32]$partition.PartitionNumber; \
                   Offset = [uint64]$partition.Offset; \
                   Size = [uint64]$partition.Size; \
                   SizeMin = [uint64]$supported.SizeMin; \
                   SizeRemaining = [uint64]$volume.SizeRemaining })"
                .to_owned(),
            Self::ListingTheDisks => "$disks = @(Get-Disk | Sort-Object Number | ForEach-Object { \
                   $disk = $_; \
                   $partitions = @(Get-Partition -DiskNumber $disk.Number -ErrorAction SilentlyContinue | \
                     ForEach-Object { \
                       $label = ''; \
                       try { $label = [string]($_ | Get-Volume).FileSystemLabel } catch { $label = '' }; \
                       [ordered]@{ PartitionNumber = [uint32]$_.PartitionNumber; \
                         GptType = [string]$_.GptType; Label = $label } }); \
                   [ordered]@{ Number = [uint32]$disk.Number; \
                     FriendlyName = [string]$disk.FriendlyName; \
                     SerialNumber = [string]$disk.SerialNumber; \
                     BusType = [string]$disk.BusType; \
                     UniqueId = [string]$disk.UniqueId; \
                     Size = [uint64]$disk.Size; \
                     PartitionStyle = [string]$disk.PartitionStyle; \
                     IsReadOnly = [bool]$disk.IsReadOnly; \
                     Partitions = $partitions } }); \
                 ConvertTo-Json -Compress -Depth 4 -InputObject $disks"
                .to_owned(),
            Self::Shrinking {
                disk,
                partition,
                to,
            }
            | Self::GrowingWindowsBack {
                disk,
                partition,
                to,
            } => format!(
                "Resize-Partition -DiskNumber {disk} -PartitionNumber {partition} -Size {to}"
            ),
            Self::MakingTheArea { disk, offset, size } => format!(
                "$made = New-Partition -DiskNumber {disk} -Offset {offset} -Size {size} \
                   -GptType '{BASIC_DATA}'; \
                 ConvertTo-Json -Compress -InputObject ([ordered]@{{ \
                   PartitionNumber = [uint32]$made.PartitionNumber; Offset = [uint64]$made.Offset }})"
            ),
            Self::PreparingTheArea {
                disk,
                partition,
                offset,
            } => format!(
                "{}\
                 $null = Format-Volume -Partition $area -FileSystem FAT32 \
                   -NewFileSystemLabel '{}' -Confirm:$false -Force; \
                 Add-PartitionAccessPath -DiskNumber {disk} -PartitionNumber {partition} \
                   -AssignDriveLetter; \
                 $area = Get-Partition -DiskNumber {disk} -PartitionNumber {partition}; \
                 ConvertTo-Json -Compress -InputObject ([ordered]@{{ \
                   DriveLetter = [string]$area.DriveLetter }})",
                still_the_area(*disk, *partition, *offset),
                alo_installing::THIS_INSTALLER
            ),
            Self::TakingAwayTheLetter {
                disk,
                partition,
                letter,
            } => format!(
                "Set-Partition -DiskNumber {disk} -PartitionNumber {partition} \
                   -NoDefaultDriveLetter $true; \
                 Remove-PartitionAccessPath -DiskNumber {disk} -PartitionNumber {partition} \
                   -AccessPath '{}'",
                letter.root()
            ),
            Self::RemovingTheArea {
                disk,
                partition,
                offset,
            } => format!(
                "{}Remove-Partition -DiskNumber {disk} -PartitionNumber {partition} -Confirm:$false",
                still_the_area(*disk, *partition, *offset)
            ),
            Self::AskingWhetherThisIsAnAdministrator
            | Self::ListingTheStartEntries
            | Self::AddingTheEntry
            | Self::PointingTheEntryAtTheArea { .. }
            | Self::PointingTheEntryAtTheLoader { .. }
            | Self::ListingTheEntry { .. }
            | Self::StartingTheEntryNext { .. }
            | Self::Restarting
            | Self::ForgettingTheNextStart
            | Self::RemovingTheEntry { .. } => return None,
        };
        Some(format!("{EVERY_SCRIPT_BEGINS}{body}"))
    }

    /// Its arguments, each one a whole argument.
    #[must_use]
    pub fn arguments(&self) -> Vec<String> {
        if let Some(script) = self.script() {
            return [
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-EncodedCommand",
            ]
            .map(str::to_owned)
            .into_iter()
            .chain([encoded::for_powershell(&script)])
            .collect();
        }
        let words: Vec<String> = match self {
            Self::AskingWhetherThisIsAnAdministrator => {
                ["/groups", "/fo", "csv", "/nh"].map(str::to_owned).to_vec()
            }
            Self::ListingTheStartEntries => ["/enum", "firmware"].map(str::to_owned).to_vec(),
            Self::AddingTheEntry => ["/copy", "{bootmgr}", "/d", THE_ENTRYS_NAME]
                .map(str::to_owned)
                .to_vec(),
            Self::PointingTheEntryAtTheArea { entry, letter } => vec![
                "/set".to_owned(),
                entry.as_str().to_owned(),
                "device".to_owned(),
                format!("partition={}", letter.drive()),
            ],
            Self::PointingTheEntryAtTheLoader { entry } => vec![
                "/set".to_owned(),
                entry.as_str().to_owned(),
                "path".to_owned(),
                THE_LOADER.to_owned(),
            ],
            Self::ListingTheEntry { entry } => vec![
                "/set".to_owned(),
                "{fwbootmgr}".to_owned(),
                "displayorder".to_owned(),
                entry.as_str().to_owned(),
                "/addlast".to_owned(),
            ],
            Self::StartingTheEntryNext { entry } => vec![
                "/set".to_owned(),
                "{fwbootmgr}".to_owned(),
                "bootsequence".to_owned(),
                entry.as_str().to_owned(),
            ],
            Self::ForgettingTheNextStart => ["/deletevalue", "{fwbootmgr}", "bootsequence"]
                .map(str::to_owned)
                .to_vec(),
            Self::RemovingTheEntry { entry } => {
                vec!["/delete".to_owned(), entry.as_str().to_owned()]
            }
            Self::Restarting => ["/r", "/t", "0"].map(str::to_owned).to_vec(),
            _ => Vec::new(),
        };
        words
    }
}

/// The lines that stop a script unless the partition it names is still the
/// one this installer made, where it made it.
///
/// Partition numbers on a disk are Windows' to reassign, and formatting or
/// removing by number alone would be one renumbering away from destroying
/// somebody's partition. The place a partition begins does not move.
fn still_the_area(disk: DiskNumber, partition: PartitionNumber, offset: u64) -> String {
    format!(
        "$area = Get-Partition -DiskNumber {disk} -PartitionNumber {partition}; \
         if ([uint64]$area.Offset -ne {offset}) {{ throw 'not the installer area' }}; "
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// One of every change, with made-up values.
    fn every_change() -> Vec<Program> {
        let entry = Entry::of("{6b1d5c2a-0f3e-11ef-9a1b-00155d012345}").unwrap();
        let letter = Letter::of("E").unwrap();
        let (disk, partition) = (DiskNumber(0), PartitionNumber(3));
        vec![
            Program::Shrinking {
                disk,
                partition,
                to: 1,
            },
            Program::MakingTheArea {
                disk,
                offset: 2,
                size: 3,
            },
            Program::PreparingTheArea {
                disk,
                partition,
                offset: 2,
            },
            Program::AddingTheEntry,
            Program::PointingTheEntryAtTheArea {
                entry: entry.clone(),
                letter,
            },
            Program::PointingTheEntryAtTheLoader {
                entry: entry.clone(),
            },
            Program::ListingTheEntry {
                entry: entry.clone(),
            },
            Program::TakingAwayTheLetter {
                disk,
                partition,
                letter,
            },
            Program::StartingTheEntryNext {
                entry: entry.clone(),
            },
            Program::Restarting,
            Program::ForgettingTheNextStart,
            Program::RemovingTheEntry { entry },
            Program::RemovingTheArea {
                disk,
                partition,
                offset: 2,
            },
            Program::GrowingWindowsBack {
                disk,
                partition,
                to: 1,
            },
        ]
    }

    /// Every read, which is every program that does not change anything.
    const EVERY_READ: [Program; 8] = [
        Program::AskingWhetherThisIsAnAdministrator,
        Program::ReadingHowItStarts,
        Program::ReadingTheTpm,
        Program::ReadingBitLocker,
        Program::ReadingTheMemory,
        Program::ReadingTheWindowsVolume,
        Program::ListingTheDisks,
        Program::ListingTheStartEntries,
    ];

    /// **A read changes nothing, and a change is never mistaken for a read.**
    #[test]
    fn reads_and_changes_are_told_apart() {
        for read in EVERY_READ {
            assert!(!read.changes(), "{read:?}");
            if let Some(script) = read.script() {
                for verb in [
                    "Resize-",
                    "New-",
                    "Remove-",
                    "Format-",
                    "Set-",
                    "Add-",
                    "Clear-",
                    "Initialize-",
                ] {
                    assert!(!script.contains(verb), "{read:?} contains {verb}");
                }
            }
        }
        for change in every_change() {
            assert!(change.changes(), "{change:?}");
        }
    }

    /// **Every program has arguments, and a PowerShell one is only ever an
    /// encoded script** — never a command line PowerShell re-reads.
    #[test]
    fn every_program_is_whole_arguments() {
        for program in EVERY_READ.into_iter().chain(every_change()) {
            let arguments = program.arguments();
            assert!(!arguments.is_empty(), "{program:?}");
            match program.tool() {
                Tool::PowerShell => {
                    assert!(program.script().is_some(), "{program:?}");
                    assert!(arguments.contains(&"-EncodedCommand".to_owned()));
                    assert!(!arguments.contains(&"-Command".to_owned()));
                }
                Tool::Bcdedit | Tool::Whoami | Tool::Shutdown => {
                    assert!(program.script().is_none(), "{program:?}");
                }
            }
        }
    }

    /// **Formatting and removing the area are each stopped by a partition that
    /// is not where the area was made.**
    #[test]
    fn formatting_or_removing_checks_the_area_is_still_where_it_was_made() {
        let (disk, partition) = (DiskNumber(1), PartitionNumber(5));
        for program in [
            Program::PreparingTheArea {
                disk,
                partition,
                offset: 4096,
            },
            Program::RemovingTheArea {
                disk,
                partition,
                offset: 4096,
            },
        ] {
            let script = program.script().unwrap();
            let check = script.find("-ne 4096").unwrap();
            let acting = script
                .find("Format-Volume")
                .or_else(|| script.find("Remove-Partition"))
                .unwrap();
            assert!(check < acting, "{program:?}");
        }
    }

    /// The tools are Windows' own, at whole paths beneath the system directory.
    #[test]
    fn the_tools_are_windows_own_at_whole_paths() {
        for tool in [
            Tool::PowerShell,
            Tool::Bcdedit,
            Tool::Whoami,
            Tool::Shutdown,
        ] {
            assert!(
                std::path::Path::new(tool.beneath_the_system_directory())
                    .extension()
                    .is_some_and(|extension| extension == "exe")
            );
        }
        assert_eq!(
            Program::StartingTheEntryNext {
                entry: Entry::of("{6b1d5c2a-0f3e-11ef-9a1b-00155d012345}").unwrap()
            }
            .arguments(),
            [
                "/set",
                "{fwbootmgr}",
                "bootsequence",
                "{6b1d5c2a-0f3e-11ef-9a1b-00155d012345}"
            ]
        );
    }
}
