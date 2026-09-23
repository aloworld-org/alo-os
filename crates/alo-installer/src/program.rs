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
//! cmdlets of Windows PowerShell, `bcdedit`, and, for the one thing `bcdedit`
//! cannot do (an entry without Windows' optional data, see
//! [`Program::WritingTheEntry`]), Windows' documented firmware-variable call
//! from a script — and never through an IOCTL this crate would have to write
//! (the installer plan's task 3; `unsafe_code` is forbidden). Every script is
//! a constant with numbers put into it, handed over by `crate::encoded` so no
//! character of it is re-read on the way.
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
//! | [`Program::ReadingFastStartup`] | whether Windows' Fast Startup is on |
//!
//! | Changes, in the order they are made | Put back by |
//! |---|---|
//! | [`Program::TurningFastStartupOff`] | [`Program::TurningFastStartupBackOn`] |
//! | [`Program::Shrinking`] | [`Program::GrowingWindowsBack`] |
//! | [`Program::MakingTheArea`] | [`Program::RemovingTheArea`] |
//! | [`Program::PreparingTheArea`] | removing the area |
//! | [`Program::WritingTheEntry`] | [`Program::RemovingTheEntry`] |
//! | [`Program::ListingTheEntry`] | removing the entry |
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

/// Where the copy of this program that offers *Restart into alo OS* is left.
///
/// Under Windows' own place for installed programs, in a directory of its own,
/// so a person looking for it finds it where they look for everything else and
/// *remove alo OS* has one directory to take away.
pub const THE_PROGRAMS_HOME: &str = "C:\\Program Files\\alo OS";

/// What the copy is called there.
pub const THE_PROGRAMS_NAME: &str = "alo-restart-into-alo-os.exe";

/// The shortcut a person starts it from, in the Start menu of every account on
/// the computer.
pub const THE_SHORTCUT: &str =
    "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\Restart into alo OS.lnk";

/// Where Windows keeps the setting behind Fast Startup.
const THE_POWER_KEY: &str = "HKLM:\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Power";

/// The value under it that Fast Startup is.
const FAST_STARTUP: &str = "HiberbootEnabled";

/// The value under it that says whether this computer hibernates at all.
///
/// Fast Startup is hibernation of the kernel's own session, so a computer
/// whose hibernation is off does not do it whatever the value above says —
/// and `powercfg /h off` leaves that value at 1 (`docs/quirks.md`).
const HIBERNATION: &str = "HibernateEnabled";

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
    /// Whether Windows' Fast Startup is on, from Windows' own value.
    ReadingFastStartup,

    /// Make the shortcut that starts the copy left in place, and read it back.
    ///
    /// Its target and its argument are [`THE_PROGRAMS_HOME`],
    /// [`THE_PROGRAMS_NAME`] and the switch's own word, which this crate holds:
    /// there is no path and no line of text a caller could put into it.
    MakingTheShortcut,
    /// Take away the copy left in place, and its shortcut.
    RemovingWhatWasLeft,

    /// Turn Windows' Fast Startup off, and read the value back.
    ///
    /// **`HiberbootEnabled`, and never `powercfg /h off`** (ADR 0064 term 9):
    /// the person agreed to turn Fast Startup off, and `powercfg /h off`
    /// removes hibernation from the computer altogether — a different act,
    /// which also throws away whatever is hibernated at the time.
    TurningFastStartupOff,
    /// Put Windows' Fast Startup back to the value it had, and read it back.
    TurningFastStartupBackOn {
        /// The value Windows held before the installer changed it.
        was: u32,
    },

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
    /// Write the start-up entry named [`THE_ENTRYS_NAME`] into the firmware
    /// itself — the area's partition and [`THE_LOADER`], and **no optional
    /// data** — read it back, and print the identifier Windows lists it under.
    ///
    /// **Why not `bcdedit`**, measured on a running Windows (the installer
    /// plan's task 10, 2026-09-21), reading each `Boot####` variable back:
    /// every entry `bcdedit /copy {bootmgr}` makes carries the 136 bytes of
    /// optional data Windows' own boot manager entry carries
    /// (`WINDOWS\0…BCDOBJECT={…}`), and deleting every boot-manager value from
    /// the copy leaves them; shim takes them as the name of what to start, fails
    /// to open it, and falls back — and on Ubuntu's OVMF that fallback
    /// page-faults. `bcdedit /create … /application firmware` is refused (*the
    /// application type switch specified is not valid*), and a copy of an
    /// existing firmware application keeps the original's device path whatever
    /// `device` and `path` are set to. So the load option is written as the UEFI
    /// specification lays it out, through `SetFirmwareEnvironmentVariableEx` —
    /// Windows' documented call for exactly this — from a script like every
    /// other program here, with no line of `unsafe` in this crate. Windows then
    /// lists it as a *Firmware Application* and keeps it without optional data;
    /// `bcdedit` lists it, sets it as the next start and removes it by the
    /// identifier printed.
    ///
    /// **Measured with this program on 2026-09-22**, the variable read from the
    /// host after the machine stopped: the area's slot and `\EFI\BOOT\BOOTX64.EFI`
    /// with 0 bytes of optional data; on the installer's own restart shim
    /// started its second stage with no fallback on Fedora's `edk2-ovmf`, and
    /// Linux and the environment followed (`docs/quirks.md`).
    ///
    /// The partition in the device path is the GPT entry's own slot, read from
    /// the disk's partition table, and never Windows' partition number: the
    /// two differ (measured: the area was Windows' partition 5 in GPT slot 4),
    /// and a firmware matches a hard-drive node by slot and signature.
    WritingTheEntry {
        /// The disk the area is on.
        disk: DiskNumber,
        /// The area's number, as Windows gives it.
        partition: PartitionNumber,
        /// Where the area must begin — checked before anything is written.
        offset: u64,
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
                | Self::ReadingFastStartup
        )
    }

    /// Setting Fast Startup's own value to exactly this, and reading it back.
    fn setting_fast_startup(to: u32) -> String {
        format!(
            "Set-ItemProperty -Path '{THE_POWER_KEY}' -Name '{FAST_STARTUP}' \
               -Value {to} -Type DWord; \
             $value = (Get-ItemProperty -Path '{THE_POWER_KEY}' -Name '{FAST_STARTUP}').'{FAST_STARTUP}'; \
             if ([uint32]$value -ne {to}) {{ throw 'the value did not take' }}; \
             ConvertTo-Json -Compress -InputObject ([ordered]@{{ HiberbootEnabled = [uint32]$value }})"
        )
    }

    /// Which tool runs it.
    #[must_use]
    pub fn tool(&self) -> Tool {
        match self {
            Self::AskingWhetherThisIsAnAdministrator => Tool::Whoami,
            Self::Restarting => Tool::Shutdown,
            Self::ListingTheStartEntries
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
            // The one value, read and written where Windows keeps it. Reading
            // it says what is there and changes nothing; writing it sets
            // exactly that value and reads it back, so a write that did not
            // take is a step that failed rather than one believed.
            Self::ReadingFastStartup => format!(
                "$power = Get-ItemProperty -Path '{THE_POWER_KEY}' -ErrorAction SilentlyContinue; \
                 $value = $power.'{FAST_STARTUP}'; \
                 $hibernation = $power.'{HIBERNATION}'; \
                 ConvertTo-Json -Compress -InputObject ([ordered]@{{ \
                   HiberbootEnabled = $(if ($null -eq $value) {{ $null }} else {{ [uint32]$value }}); \
                   HibernateEnabled = $(if ($null -eq $hibernation) {{ $null }} else {{ [uint32]$hibernation }}) }})"
            ),
            // The shortcut is made through Windows' own shell object, which is
            // how a shortcut is made on Windows, and read back: a shortcut that
            // was not written is a step that failed.
            Self::MakingTheShortcut => format!(
                "New-Item -ItemType Directory -Path '{THE_PROGRAMS_HOME}' -Force | Out-Null; \
                 $shell = New-Object -ComObject WScript.Shell; \
                 $shortcut = $shell.CreateShortcut('{THE_SHORTCUT}'); \
                 $shortcut.TargetPath = '{THE_PROGRAMS_HOME}\\{THE_PROGRAMS_NAME}'; \
                 $shortcut.Arguments = '{}'; \
                 $shortcut.WorkingDirectory = '{THE_PROGRAMS_HOME}'; \
                 $shortcut.Description = 'Restart this computer into alo OS'; \
                 $shortcut.Save(); \
                 if (-not (Test-Path -LiteralPath '{THE_SHORTCUT}')) {{ throw 'no shortcut' }}; \
                 ConvertTo-Json -Compress -InputObject ([ordered]@{{ Shortcut = '{THE_SHORTCUT}' }})",
                crate::switching::THE_SWITCHS_WORD
            ),
            Self::RemovingWhatWasLeft => format!(
                "Remove-Item -LiteralPath '{THE_SHORTCUT}' -Force -ErrorAction SilentlyContinue; \
                 Remove-Item -LiteralPath '{THE_PROGRAMS_HOME}' -Recurse -Force \
                   -ErrorAction SilentlyContinue; \
                 if (Test-Path -LiteralPath '{THE_SHORTCUT}') {{ throw 'the shortcut is still there' }}"
            ),
            Self::TurningFastStartupOff => Self::setting_fast_startup(0),
            Self::TurningFastStartupBackOn { was } => Self::setting_fast_startup(*was),
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
            Self::WritingTheEntry {
                disk,
                partition,
                offset,
            } => format!(
                "{}{}",
                still_the_area(*disk, *partition, *offset),
                WRITING_THE_ENTRY
                    .replace("@DISK@", &disk.to_string())
                    .replace("@NAME@", THE_ENTRYS_NAME)
                    .replace("@LOADER@", THE_LOADER)
            ),
            Self::AskingWhetherThisIsAnAdministrator
            | Self::ListingTheStartEntries
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

/// What [`Program::WritingTheEntry`] runs after the check that the area is
/// still where it was made (which leaves the area in `$area`).
///
/// In order: the firmware-variable privilege, which an administrator holds but
/// must switch on; the area's GPT slot, read from the disk's own partition
/// table and matched by the area's unique GUID *and* its first sector; a load
/// option of exactly attributes, length, description and one device path —
/// the area's hard-drive node and [`THE_LOADER`] — and nothing after it; a
/// `Boot####` number no variable and no `BootOrder` entry uses; the write; the
/// variable read back and compared byte for byte; and the identifier Windows
/// lists it under, found by its description and its type (a firmware
/// application, `0x101fffff`) in the BCD store rather than in `bcdedit`'s
/// printed words, which are in Windows' own language.
///
/// **Nothing it wrote is left behind when it fails after writing**: the
/// variable is removed before the failure is thrown, so a failed write is a
/// step that changed nothing, as the staging journal assumes.
const WRITING_THE_ENTRY: &str = r#"$sector = [uint32](Get-Disk -Number @DISK@).LogicalSectorSize;
Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
public static class AloFirmwareEntry {
  const string Global = "{8BE4DF61-93CA-11D2-AA0D-00E098032B8C}";
  [StructLayout(LayoutKind.Sequential)] struct Luid { public uint Low; public int High; }
  [StructLayout(LayoutKind.Sequential)] struct Privilege { public uint Count; public Luid Id; public uint Attributes; }
  [DllImport("advapi32.dll", SetLastError = true)] static extern bool OpenProcessToken(IntPtr process, uint access, out IntPtr token);
  [DllImport("advapi32.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern bool LookupPrivilegeValue(string system, string name, out Luid id);
  [DllImport("advapi32.dll", SetLastError = true)] static extern bool AdjustTokenPrivileges(IntPtr token, bool disableAll, ref Privilege state, uint length, IntPtr previous, IntPtr returned);
  [DllImport("kernel32.dll")] static extern IntPtr GetCurrentProcess();
  [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern uint GetFirmwareEnvironmentVariableEx(string name, string guid, byte[] buffer, uint size, ref uint attributes);
  [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern bool SetFirmwareEnvironmentVariableEx(string name, string guid, byte[] value, uint size, uint attributes);
  [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern SafeFileHandle CreateFile(string name, uint access, uint share, IntPtr security, uint disposition, uint flags, IntPtr template);

  public static void SwitchOnThePrivilege() {
    IntPtr token;
    if (!OpenProcessToken(GetCurrentProcess(), 0x28, out token)) throw new InvalidOperationException("OpenProcessToken " + Marshal.GetLastWin32Error());
    Luid id;
    if (!LookupPrivilegeValue(null, "SeSystemEnvironmentPrivilege", out id)) throw new InvalidOperationException("LookupPrivilegeValue " + Marshal.GetLastWin32Error());
    Privilege state = new Privilege { Count = 1, Id = id, Attributes = 2 };
    if (!AdjustTokenPrivileges(token, false, ref state, 0, IntPtr.Zero, IntPtr.Zero) || Marshal.GetLastWin32Error() != 0) throw new InvalidOperationException("the firmware privilege is not held");
  }

  static byte[] ReadDisk(uint disk, ulong at, int count) {
    using (SafeFileHandle handle = CreateFile(@"\\.\PhysicalDrive" + disk, 0x80000000, 3, IntPtr.Zero, 3, 0, IntPtr.Zero)) {
      if (handle.IsInvalid) throw new IOException("the disk could not be opened: " + Marshal.GetLastWin32Error());
      using (FileStream stream = new FileStream(handle, FileAccess.Read, 1)) {
        stream.Seek((long)at, SeekOrigin.Begin);
        byte[] bytes = new byte[count];
        int got = 0;
        while (got < count) { int n = stream.Read(bytes, got, count - got); if (n <= 0) throw new IOException("the disk ended early"); got += n; }
        return bytes;
      }
    }
  }

  public static uint GptSlot(uint disk, Guid partition, ulong firstSector, uint sector) {
    byte[] header = ReadDisk(disk, sector, (int)sector);
    if (System.Text.Encoding.ASCII.GetString(header, 0, 8) != "EFI PART") throw new InvalidDataException("the disk has no GPT header");
    ulong entries = BitConverter.ToUInt64(header, 72);
    uint count = BitConverter.ToUInt32(header, 80);
    uint size = BitConverter.ToUInt32(header, 84);
    if (size < 128 || count == 0 || count > 1024) throw new InvalidDataException("the GPT header is not one this installer reads");
    ulong length = (ulong)count * size;
    ulong rounded = (length + sector - 1) / sector * sector;
    byte[] table = ReadDisk(disk, entries * sector, (int)rounded);
    uint found = 0;
    for (uint i = 0; i < count; i++) {
      int at = (int)(i * size);
      byte[] unique = new byte[16];
      Array.Copy(table, at + 16, unique, 0, 16);
      if (new Guid(unique) == partition && BitConverter.ToUInt64(table, at + 32) == firstSector) {
        if (found != 0) throw new InvalidDataException("the area is in the partition table twice");
        found = i + 1;
      }
    }
    if (found == 0) throw new InvalidDataException("the area is not in the partition table");
    return found;
  }

  public static byte[] LoadOption(string description, uint slot, ulong firstSector, ulong sectors, Guid signature, string file) {
    MemoryStream path = new MemoryStream();
    BinaryWriter w = new BinaryWriter(path);
    w.Write((byte)4); w.Write((byte)1); w.Write((ushort)42);
    w.Write(slot); w.Write(firstSector); w.Write(sectors); w.Write(signature.ToByteArray());
    w.Write((byte)2); w.Write((byte)2);
    byte[] name = System.Text.Encoding.Unicode.GetBytes(file + "\0");
    w.Write((byte)4); w.Write((byte)4); w.Write((ushort)(4 + name.Length)); w.Write(name);
    w.Write((byte)0x7f); w.Write((byte)0xff); w.Write((ushort)4);
    w.Flush();
    byte[] list = path.ToArray();
    MemoryStream option = new MemoryStream();
    BinaryWriter o = new BinaryWriter(option);
    o.Write((uint)1);
    o.Write((ushort)list.Length);
    o.Write(System.Text.Encoding.Unicode.GetBytes(description + "\0"));
    o.Write(list);
    o.Flush();
    return option.ToArray();
  }

  public static byte[] Read(string name) {
    byte[] buffer = new byte[65536];
    uint attributes = 0;
    uint size = GetFirmwareEnvironmentVariableEx(name, Global, buffer, (uint)buffer.Length, ref attributes);
    if (size == 0) return null;
    byte[] value = new byte[size];
    Array.Copy(buffer, value, size);
    return value;
  }

  public static string FreeBootName() {
    byte[] order = Read("BootOrder") ?? new byte[0];
    for (int n = 0; n < 0x1000; n++) {
      bool listed = false;
      for (int i = 0; i + 1 < order.Length; i += 2) if (BitConverter.ToUInt16(order, i) == n) listed = true;
      string name = "Boot" + n.ToString("X4");
      if (!listed && Read(name) == null) return name;
    }
    throw new InvalidOperationException("every Boot#### number is in use");
  }

  public static void Write(string name, byte[] value) {
    if (!SetFirmwareEnvironmentVariableEx(name, Global, value, (uint)value.Length, 7)) throw new IOException("the firmware refused " + name + ": " + Marshal.GetLastWin32Error());
  }

  public static void Remove(string name) {
    SetFirmwareEnvironmentVariableEx(name, Global, null, 0, 7);
  }

  public static bool Same(byte[] one, byte[] other) {
    if (one == null || other == null || one.Length != other.Length) return false;
    for (int i = 0; i < one.Length; i++) if (one[i] != other[i]) return false;
    return true;
  }
}
'@;
[AloFirmwareEntry]::SwitchOnThePrivilege();
$firstSector = [uint64]([uint64]$area.Offset / $sector);
$sectors = [uint64]([uint64]$area.Size / $sector);
$slot = [AloFirmwareEntry]::GptSlot(@DISK@, [Guid]$area.Guid, $firstSector, $sector);
$option = [AloFirmwareEntry]::LoadOption('@NAME@', $slot, $firstSector, $sectors, [Guid]$area.Guid, '@LOADER@');
$name = [AloFirmwareEntry]::FreeBootName();
[AloFirmwareEntry]::Write($name, $option);
try {
  if (-not [AloFirmwareEntry]::Same([AloFirmwareEntry]::Read($name), $option)) { throw 'the entry read back is not the entry written' };
  $null = & "$env:SystemRoot\System32\bcdedit.exe" /enum firmware;
  $listed = @(Get-ChildItem -LiteralPath 'HKLM:\BCD00000000\Objects' | Where-Object {
    $described = Get-ItemProperty -LiteralPath (Join-Path $_.PSPath 'Elements\12000004') -Name Element -ErrorAction SilentlyContinue;
    $typed = Get-ItemProperty -LiteralPath (Join-Path $_.PSPath 'Description') -Name Type -ErrorAction SilentlyContinue;
    $described -and $typed -and $described.Element -ceq '@NAME@' -and $typed.Type -eq 0x101fffff });
  if ($listed.Count -ne 1) { throw ('Windows lists the entry ' + $listed.Count + ' times') };
  ConvertTo-Json -Compress -InputObject ([ordered]@{ Identifier = [string]$listed[0].PSChildName; Option = $name; Slot = $slot })
} catch {
  [AloFirmwareEntry]::Remove($name);
  throw
}"#;

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
            Program::MakingTheShortcut,
            Program::RemovingWhatWasLeft,
            Program::TurningFastStartupOff,
            Program::TurningFastStartupBackOn { was: 1 },
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
            Program::WritingTheEntry {
                disk,
                partition,
                offset: 2,
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
    const EVERY_READ: [Program; 9] = [
        Program::ReadingFastStartup,
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

    /// **Fast Startup is switched off by its own value, and hibernation is
    /// never removed.**
    ///
    /// ADR 0064 term 9: the person agreed to turn Fast Startup off.
    /// `powercfg /h off` removes hibernation from the computer altogether and
    /// throws away anything hibernated at the time, which is not what they
    /// agreed to — so no program of this installer may name it.
    #[test]
    fn fast_startup_is_the_value_and_never_powercfg() {
        for program in EVERY_READ.into_iter().chain(every_change()) {
            let script = program.script().unwrap_or_default();
            let said = format!("{script} {}", program.arguments().join(" ")).to_lowercase();
            // The value that says whether the computer hibernates is read by
            // its own name (`HibernateEnabled`), which is why the name alone
            // is not on this list — what is forbidden is the tool that turns
            // hibernation off.
            for never in ["powercfg", "/h off"] {
                assert!(!said.contains(never), "{program:?} names {never}");
            }
        }
        let off = Program::TurningFastStartupOff.script().unwrap();
        assert!(off.contains("HiberbootEnabled") && off.contains("-Value 0"));
        assert!(
            Program::TurningFastStartupBackOn { was: 1 }
                .script()
                .unwrap()
                .contains("-Value 1")
        );
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

    /// **The entry is written, never copied from Windows' boot manager.**
    ///
    /// A copy of `{bootmgr}` carries 136 bytes of Windows' optional data that
    /// shim misreads (measured, `docs/quirks.md`), and `bcdedit` has no way to
    /// make an entry without it; so no program copies `{bootmgr}`, and the one
    /// that writes the entry checks the area before it writes, writes a load
    /// option that ends at its device path, reads it back, and removes what it
    /// wrote when anything after the write fails.
    #[test]
    fn the_entry_is_written_without_windows_optional_data() {
        for program in EVERY_READ.into_iter().chain(every_change()) {
            let everything = format!(
                "{:?} {}",
                program.arguments(),
                program.script().unwrap_or_default()
            );
            assert!(!everything.contains("/copy"), "{program:?}");
        }
        let script = Program::WritingTheEntry {
            disk: DiskNumber(0),
            partition: PartitionNumber(5),
            offset: 66_872_934_400,
        }
        .script()
        .unwrap();
        let checked = script.find("-ne 66872934400").unwrap();
        let written = script
            .find("[AloFirmwareEntry]::Write($name, $option)")
            .unwrap();
        assert!(checked < written);
        // The load option is attributes, length, description and the path
        // list, and nothing after it: no optional data.
        let option = script.find("public static byte[] LoadOption").unwrap();
        let ends = script[option..].find("return option.ToArray();").unwrap();
        let body = &script[option..option + ends];
        assert_eq!(body.matches("o.Write(").count(), 4, "{body}");
        assert!(body.contains("o.Write(list);\n    o.Flush();"), "{body}");
        // The slot comes from the partition table, matched by GUID and place.
        assert!(script.contains("GptSlot(0, [Guid]$area.Guid, $firstSector, $sector)"));
        assert!(script.contains(&format!("'{THE_LOADER}'")));
        // What it wrote is read back, and removed when anything after fails.
        assert!(script.contains("::Same([AloFirmwareEntry]::Read($name), $option)"));
        let caught = script.rfind("} catch {").unwrap();
        assert!(script[caught..].contains("[AloFirmwareEntry]::Remove($name)"));
        // Handed over whole: an encoded command has to fit a command line.
        let arguments = Program::WritingTheEntry {
            disk: DiskNumber(0),
            partition: PartitionNumber(5),
            offset: 66_872_934_400,
        }
        .arguments();
        let line: usize = arguments.iter().map(|word| word.len() + 1).sum();
        assert!(line < 32_000, "{line}");
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
