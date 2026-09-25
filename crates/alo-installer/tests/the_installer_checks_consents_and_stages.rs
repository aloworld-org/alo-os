//! The installer, walked from the first line to the restart against a scripted
//! Windows — every refusal, the consent, the staging, and putting it back.
//!
//! The machine here is a list of answers: what each of Windows' programs
//! prints, what the person types, and a directory of files. Every program that
//! runs is recorded, and **the question every refusal is asked is whether any
//! program that changes the computer ran** — which is the claim *so nothing was
//! changed* makes, held to what actually happened.
//!
//! What this cannot show is Windows itself doing any of it; the installer
//! plan's virtual-machine task does that, and `tests/reading_this_windows.rs`
//! reads a real Windows with the same checks.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_installer::{
    EVERY_FILE_IT_NEEDS, Ended, Program, Ran, Refusal, Released, Remains, THE_CHOICE,
    THE_DIRECTORY, THE_LIST, TheMachine, check, install, installer_words, sha256_hex,
};
use alo_strings::{Said, Strings};

/// Where the scripted download is.
const DOWNLOADED: &str = "/Downloads";

/// The identifier the scripted start-up editor gives the entry it makes.
const THE_ENTRY: &str = "{6b1d5c2a-0f3e-11ef-9a1b-00155d012345}";

/// A Windows the installer can install on: UEFI with Secure Boot off, a TPM,
/// BitLocker on, 32 GB of memory, Windows on disk 0 with room, an empty 32 GB
/// disk 1, and nothing left by an earlier start.
struct Scripted {
    /// What each program prints, by the program's kind; a missing kind fails.
    answers: BTreeMap<&'static str, Ran>,
    /// Programs that fail, by kind.
    failing: Vec<&'static str>,
    /// What the person types for the first question, which is the disk's name.
    typed: String,
    /// What the person types for the questions after it, in order.
    then_typed: Vec<String>,
    /// The files on the machine.
    files: BTreeMap<PathBuf, Vec<u8>>,
    /// Files whose writes read back as something else.
    corrupting: Vec<PathBuf>,
    /// Every program that ran, in order.
    ran: Vec<Program>,
    /// Every sentence said, in order.
    said: Vec<String>,
    /// Every question asked.
    asked: Vec<String>,
    /// What the Windows volume reads as once a shrink has been attempted.
    volume_once_shrinking_ran: Option<String>,
}

/// A program's kind, for scripting answers without its arguments.
fn kind(program: &Program) -> &'static str {
    match program {
        Program::AskingWhetherThisIsAnAdministrator => "administrator",
        Program::ReadingHowItStarts => "starting",
        Program::ReadingTheTpm => "tpm",
        Program::ReadingBitLocker => "bitlocker",
        Program::ReadingTheMemory => "memory",
        Program::ReadingTheWindowsVolume => "volume",
        Program::ListingTheDisks => "disks",
        Program::ListingTheStartEntries => "entries",
        Program::ReadingFastStartup => "fast-startup",
        Program::GivingTheStartPartitionALetter { .. } => "give-esp-letter",
        Program::TakingTheStartPartitionsLetterAway { .. } => "take-esp-letter",
        Program::MakingTheShortcut => "shortcut",
        Program::RemovingWhatWasLeft => "remove-what-was-left",
        Program::TurningFastStartupOff => "fast-startup-off",
        Program::TurningFastStartupBackOn { .. } => "fast-startup-back-on",
        Program::Shrinking { .. } => "shrink",
        Program::MakingTheArea { .. } => "make-area",
        Program::PreparingTheArea { .. } => "prepare-area",
        Program::WritingTheEntry { .. } => "write-entry",
        Program::ListingTheEntry { .. } => "list-entry",
        Program::TakingAwayTheLetter { .. } => "take-letter",
        Program::StartingTheEntryNext { .. } => "next",
        Program::Restarting => "restart",
        Program::ForgettingTheNextStart => "forget-next",
        Program::RemovingTheEntry { .. } => "remove-entry",
        Program::RemovingTheArea { .. } => "remove-area",
        Program::GrowingWindowsBack { .. } => "grow-back",
    }
}

/// A program that succeeded, printing this.
fn printed(text: &str) -> Ran {
    Ran {
        succeeded: true,
        printed: text.to_owned(),
    }
}

/// The disk layout of a machine alo OS can be installed beside.
const DISKS: &str = r#"[
  {"Number":0,"FriendlyName":"Samsung SSD 980 1TB","SerialNumber":"S64ANS0T123456A","BusType":"NVMe","UniqueId":"","Size":1000204886016,"PartitionStyle":"GPT","IsReadOnly":false,
   "Partitions":[{"PartitionNumber":1,"GptType":"{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}","Label":""},
                 {"PartitionNumber":2,"GptType":"{e3c9e316-0b5c-4db8-817d-f92df00215ae}","Label":""},
                 {"PartitionNumber":3,"GptType":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","Label":"Windows"},
                 {"PartitionNumber":4,"GptType":"{de94bba4-06d1-4d40-a16a-bfd50179d6ac}","Label":"Recovery"}]},
  {"Number":1,"FriendlyName":"Msft Virtual Disk","SerialNumber":"","BusType":"SAS","UniqueId":"60022480AAAABBBBCCCCDDDDEEEEFFFF","Size":34359738368,"PartitionStyle":"RAW","IsReadOnly":false,"Partitions":[]}
]"#;

/// The Windows volume on disk 0.
const VOLUME: &str = r#"{"DriveLetter":"C","DiskNumber":0,"PartitionNumber":3,"Offset":290455552,"Size":998000000000,"SizeMin":200000000000,"SizeRemaining":600000000000}"#;

/// Where the area begins: after Windows shrunk by a gibibyte, aligned.
fn area_begins() -> u64 {
    let size: u64 = 998_000_000_000;
    let to = (size - alo_installer::GIB) - (size - alo_installer::GIB) % alo_installer::MIB;
    290_455_552 + to
}

impl Scripted {
    /// The machine above, with a genuine download beside the installer.
    fn installable() -> (Self, Released) {
        let mut files = BTreeMap::new();
        let directory = Path::new(DOWNLOADED).join(THE_DIRECTORY);
        let mut list = String::new();
        for inside in EVERY_FILE_IT_NEEDS {
            let bytes = format!("the bytes of {inside}").into_bytes();
            list.push_str(&format!("{}  {inside}\n", sha256_hex(&bytes)));
            files.insert(beneath(&directory, inside), bytes);
        }
        files.insert(directory.join(THE_LIST), list.clone().into_bytes());
        // The program itself, which staging copies to where a person finds it.
        files.insert(
            Path::new(DOWNLOADED).join("alo-installer.exe"),
            b"the bytes of the installer".to_vec(),
        );
        let released = Released::of(Some(&sha256_hex(list.as_bytes())));

        let answers = BTreeMap::from([
            (
                "administrator",
                printed(
                    "\"Mandatory Label\\High Mandatory Level\",\"Label\",\"S-1-16-12288\",\"\"\n",
                ),
            ),
            (
                "starting",
                printed(r#"{"FirmwareType":"UEFI","SecureBoot":false}"#),
            ),
            ("tpm", printed(r#"{"TpmPresent":true,"TpmReady":true}"#)),
            (
                "bitlocker",
                printed(r#"{"VolumeStatus":"FullyEncrypted","ProtectionStatus":"On"}"#),
            ),
            ("memory", printed(r#"{"TotalPhysicalMemory":34190917632}"#)),
            ("volume", printed(VOLUME)),
            ("disks", printed(DISKS)),
            (
                "entries",
                printed("identifier {bootmgr}\ndescription Windows Boot Manager\n"),
            ),
            // Off on this scripted computer, so the question is not asked and
            // no test here answers it by accident; the tests that are about
            // the question turn it on (`fast_startup_is_asked_about`).
            (
                "fast-startup",
                printed(r#"{"HiberbootEnabled":0,"HibernateEnabled":1}"#),
            ),
            (
                "shortcut",
                printed(r#"{"Shortcut":"C:\\ProgramData\\Restart into alo OS.lnk"}"#),
            ),
            ("remove-what-was-left", printed("")),
            ("fast-startup-off", printed(r#"{"HiberbootEnabled":0}"#)),
            ("fast-startup-back-on", printed(r#"{"HiberbootEnabled":1}"#)),
            ("shrink", printed("")),
            (
                "make-area",
                printed(&format!(
                    r#"{{"PartitionNumber":5,"Offset":{}}}"#,
                    area_begins()
                )),
            ),
            ("prepare-area", printed(r#"{"DriveLetter":"E"}"#)),
            (
                "write-entry",
                printed(&format!(
                    r#"{{"Identifier":"{THE_ENTRY}","Option":"Boot0005","Slot":4}}"#
                )),
            ),
            ("list-entry", printed("")),
            ("take-letter", printed("")),
            ("next", printed("")),
            ("restart", printed("")),
            ("forget-next", printed("")),
            ("remove-entry", printed("")),
            ("remove-area", printed("")),
            ("grow-back", printed("")),
        ]);
        (
            Self {
                answers,
                failing: Vec::new(),
                typed: "Msft Virtual Disk\n".to_owned(),
                then_typed: Vec::new(),
                files,
                corrupting: Vec::new(),
                ran: Vec::new(),
                said: Vec::new(),
                asked: Vec::new(),
                volume_once_shrinking_ran: None,
            },
            released,
        )
    }

    /// Answer this program with this instead.
    fn answering(mut self, kind: &'static str, text: &str) -> Self {
        self.answers.insert(kind, printed(text));
        self
    }

    /// Make this program fail.
    fn failing(mut self, kind: &'static str) -> Self {
        self.failing.push(kind);
        self
    }

    /// Whether any program that changes the computer ran.
    fn changed_anything(&self) -> bool {
        self.ran.iter().any(Program::changes)
    }

    /// The kinds of every program that ran, in order.
    fn kinds(&self) -> Vec<&'static str> {
        self.ran.iter().map(kind).collect()
    }
}

/// A path inside the area, beneath a root.
fn beneath(root: &Path, inside: &str) -> PathBuf {
    inside
        .split('/')
        .fold(root.to_path_buf(), |path, part| path.join(part))
}

impl TheMachine for Scripted {
    fn say(&mut self, said: &Said) {
        assert!(said.unfilled().is_empty(), "unfilled: {}", said.text());
        assert!(!said.is_a_bug(), "a key with no sentence: {}", said.text());
        self.said.push(said.text().to_owned());
    }

    fn ask(&mut self, said: &Said) -> String {
        self.asked.push(said.text().to_owned());
        // The first question is the disk's name; anything asked after it is
        // answered from `then_typed`, in order, and an empty list means the
        // person typed the same thing again — which no second question here
        // accepts.
        if self.asked.len() > 1 && !self.then_typed.is_empty() {
            return self.then_typed.remove(0);
        }
        self.typed.clone()
    }

    fn run(&mut self, program: &Program) -> std::io::Result<Ran> {
        self.ran.push(program.clone());
        let kind = kind(program);
        let shrinking_ran = self
            .ran
            .iter()
            .any(|ran| matches!(ran, Program::Shrinking { .. }));
        if let (Program::ReadingTheWindowsVolume, true, Some(volume)) =
            (program, shrinking_ran, &self.volume_once_shrinking_ran)
        {
            return Ok(printed(volume));
        }
        if self.failing.contains(&kind) {
            return Ok(Ran::default());
        }
        Ok(self
            .answers
            .get(kind)
            .cloned()
            .unwrap_or_else(|| panic!("nothing scripted for {kind}")))
    }

    fn downloaded_into(&mut self) -> std::io::Result<PathBuf> {
        Ok(PathBuf::from(DOWNLOADED))
    }

    fn this_program(&mut self) -> std::io::Result<PathBuf> {
        Ok(PathBuf::from(DOWNLOADED).join("alo-installer.exe"))
    }

    fn read(&mut self, file: &Path) -> std::io::Result<Vec<u8>> {
        let bytes = self
            .files
            .get(file)
            .cloned()
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
        if self.corrupting.iter().any(|corrupt| corrupt == file) {
            return Ok(b"something else".to_vec());
        }
        Ok(bytes)
    }

    fn write(&mut self, file: &Path, bytes: &[u8]) -> std::io::Result<()> {
        self.files.insert(file.to_path_buf(), bytes.to_vec());
        Ok(())
    }

    fn pause(&mut self, _for_as_long_as: Duration) {}
}

/// Everything the installer can say, in English.
fn strings() -> Strings {
    Strings::of(installer_words().unwrap())
}

/// Run the installer on a machine, and hand both back.
fn run(mut machine: Scripted, released: Released) -> (Ended, Scripted) {
    let ended = install(&mut machine, &strings(), released);
    (ended, machine)
}

/// The English of a refusal's sentence, as the machine would have been told it.
fn sentence_of(refusal: &Refusal) -> String {
    let (word, filling) = refusal.said_as();
    strings().say(&word.key(), &filling).text().to_owned()
}

/// Assert a run was refused for this reason, said it last, and changed nothing.
fn refused_without_change(ended: &Ended, machine: &Scripted, refusal: &Refusal) {
    assert_eq!(
        ended,
        &Ended::Refused(refusal.clone()),
        "{:?}",
        machine.said
    );
    assert!(
        !machine.changed_anything(),
        "{refusal:?} changed the computer: {:?}",
        machine.kinds()
    );
    assert_eq!(machine.said.last(), Some(&sentence_of(refusal)));
    assert!(
        machine
            .said
            .last()
            .unwrap()
            .contains("so nothing was changed")
    );
}

// ---------------------------------------------------------------------------
// The legitimate road.
// ---------------------------------------------------------------------------

/// **The whole road: checked, said, agreed by name, staged in order, and
/// restarted** — with the environment's exact bytes and the chosen disk's
/// after-restart name on the area.
#[test]
fn a_computer_that_can_take_alo_os_is_checked_told_asked_staged_and_restarted() {
    let (machine, released) = Scripted::installable();
    let (ended, machine) = run(machine, released);
    assert_eq!(
        ended,
        Ended::Staged { restarted: true },
        "{:#?}",
        machine.said
    );

    assert_eq!(
        machine.kinds(),
        [
            "administrator",
            "starting",
            "tpm",
            "bitlocker",
            "memory",
            "volume",
            "disks",
            "entries",
            "fast-startup",
            "shrink",
            "make-area",
            "prepare-area",
            "shortcut",
            "write-entry",
            "list-entry",
            "take-letter",
            "next",
            "restart",
        ]
    );

    // Shrunk by the area, the area exactly where that freed.
    let shrink = machine.ran.iter().find_map(|program| match program {
        Program::Shrinking { to, .. } => Some(*to),
        _ => None,
    });
    assert_eq!(shrink, Some(area_begins() - 290_455_552));
    assert!(machine.ran.contains(&Program::MakingTheArea {
        disk: alo_installer::DiskNumber(0),
        offset: area_begins(),
        size: alo_installer::THE_AREA,
    }));

    // The files, and the choice, on the area.
    let area = PathBuf::from("E:\\");
    for inside in EVERY_FILE_IT_NEEDS {
        assert_eq!(
            machine.files.get(&beneath(&area, inside)),
            Some(&format!("the bytes of {inside}").into_bytes()),
            "{inside}"
        );
    }
    assert_eq!(
        machine.files.get(&beneath(&area, THE_CHOICE)),
        Some(&b"set alo_installing_to=wwn-0x60022480aaaabbbbccccddddeeeeffff\n".to_vec())
    );

    // What was found was said, and what would happen was said, before the
    // question — and nothing changed before it was answered.
    let asked_at = machine
        .said
        .iter()
        .position(|said| said.starts_with("This computer restarts once into the installer"))
        .unwrap();
    for expected in [
        "This download is a genuine alo OS",
        "This computer starts with UEFI, which alo OS needs",
        "Secure Boot is off",
        "This computer has a security chip (TPM), and it is ready",
        "Windows on C: is encrypted with BitLocker. It stays encrypted, and the installer does not read what is on it",
        "Windows on C: has 558 GB free of 929 GB",
        "This computer has 32 GB of memory",
        "Windows is on the disk Samsung SSD 980 1TB (931 GB)",
        "The disk Msft Virtual Disk (32 GB) is empty, and alo OS can be installed onto it",
        "Nothing on this computer has been changed yet. If you agree, this is what happens, in this order:",
        "Windows on C: is made 1 GB smaller. Your files and applications stay where they are",
        "A 1 GB area for the installer is made in that space, on the disk Samsung SSD 980 1TB",
        "An entry named alo OS is added to the systems this computer can start. Windows stays the one it starts normally",
    ] {
        let at = machine
            .said
            .iter()
            .position(|said| said == expected)
            .unwrap_or_else(|| panic!("never said: {expected}\n{:#?}", machine.said));
        assert!(at <= asked_at, "{expected} was said after the question");
    }
    assert_eq!(machine.asked.len(), 1);
    assert!(machine.asked[0].starts_with("To agree, type the name of the disk"));
    assert_eq!(
        machine.said.last().map(String::as_str),
        Some(
            "Everything is ready. This computer restarts in a few seconds, and installing continues after the restart"
        )
    );
}

/// **The next start is set last, after every other change** — so a run killed
/// before it leaves a Windows that starts as it always did.
#[test]
fn the_next_start_is_the_last_change_before_the_restart() {
    let (machine, released) = Scripted::installable();
    let (_, machine) = run(machine, released);
    let changes: Vec<&str> = machine
        .ran
        .iter()
        .filter(|program| program.changes())
        .map(kind)
        .collect();
    assert_eq!(
        changes.iter().rev().take(2).collect::<Vec<_>>(),
        [&"restart", &"next"]
    );
}

/// A restart Windows did not carry out is said as that.
#[test]
fn a_restart_that_did_not_happen_is_said() {
    let (machine, released) = Scripted::installable();
    let (ended, machine) = run(machine.failing("restart"), released);
    assert_eq!(ended, Ended::Staged { restarted: false });
    assert!(
        machine
            .said
            .last()
            .unwrap()
            .contains("could not restart by itself")
    );
}

// ---------------------------------------------------------------------------
// Every refusal before anything changes.
// ---------------------------------------------------------------------------

/// **Not an administrator is refused before anything else is even read.**
#[test]
fn not_an_administrator_is_refused_before_anything_is_read() {
    let (machine, released) = Scripted::installable();
    let machine = machine.answering(
        "administrator",
        "\"Mandatory Label\\Medium Mandatory Level\",\"Label\",\"S-1-16-8192\",\"\"\n",
    );
    let (ended, machine) = run(machine, released);
    refused_without_change(&ended, &machine, &Refusal::NotAnAdministrator);
    assert_eq!(machine.kinds(), ["administrator"]);

    let (machine, released) = Scripted::installable();
    let (ended, machine) = run(machine.failing("administrator"), released);
    refused_without_change(&ended, &machine, &Refusal::NotAnAdministrator);
}

/// **A download that is not genuine is refused in the plan's words**: a build
/// with no released list, a list that is not the released one, and a file that
/// is not what the list says. No check of the computer is even run.
#[test]
fn a_download_that_is_not_genuine_is_refused_in_the_plans_words() {
    let (machine, _) = Scripted::installable();
    let (ended, machine) = run(machine, Released::of(None));
    refused_without_change(&ended, &machine, &Refusal::NotGenuine);
    assert_eq!(
        machine.said.last().unwrap(),
        "This download is not a genuine alo OS, so nothing was changed"
    );
    assert_eq!(machine.kinds(), ["administrator"]);

    let (machine, _) = Scripted::installable();
    let (ended, machine) = run(machine, Released::of(Some(&sha256_hex(b"another list"))));
    refused_without_change(&ended, &machine, &Refusal::NotGenuine);

    let (mut machine, released) = Scripted::installable();
    let initramfs = beneath(
        &Path::new(DOWNLOADED).join(THE_DIRECTORY),
        "EFI/alo-installing/initramfs.img",
    );
    machine
        .files
        .insert(initramfs, b"an initramfs somebody changed".to_vec());
    let (ended, machine) = run(machine, released);
    refused_without_change(&ended, &machine, &Refusal::NotGenuine);

    // No sentence at any point named a key, a signature or a checksum.
    for said in &machine.said {
        let lower = said.to_lowercase();
        for forbidden in ["key", "signature", "sha", "checksum", "digest"] {
            assert!(!lower.contains(forbidden), "{said}");
        }
    }
}

/// **A download with a file missing is incomplete**, which is a different
/// sentence from not genuine.
#[test]
fn a_download_with_a_file_missing_is_incomplete() {
    let (mut machine, released) = Scripted::installable();
    machine.files.remove(&beneath(
        &Path::new(DOWNLOADED).join(THE_DIRECTORY),
        "EFI/BOOT/grubx64.efi",
    ));
    let (ended, machine) = run(machine, released);
    refused_without_change(&ended, &machine, &Refusal::Incomplete);

    let (mut machine, released) = Scripted::installable();
    machine
        .files
        .remove(&Path::new(DOWNLOADED).join(THE_DIRECTORY).join(THE_LIST));
    let (ended, machine) = run(machine, released);
    refused_without_change(&ended, &machine, &Refusal::Incomplete);
}

/// **With Secure Boot on the installer refuses and says why** — after saying
/// everything it found, and with nothing in any sentence telling the person to
/// change the setting (ADR 0033 §4).
#[test]
fn with_secure_boot_on_it_refuses_says_why_and_suggests_nothing() {
    let (machine, released) = Scripted::installable();
    let machine = machine.answering("starting", r#"{"FirmwareType":"UEFI","SecureBoot":true}"#);
    let (ended, machine) = run(machine, released);
    refused_without_change(&ended, &machine, &Refusal::SecureBootOn);
    assert_eq!(
        machine.said.last().unwrap(),
        "Secure Boot is on, and the part of alo OS that starts a computer is not yet approved to start with it, so nothing was changed"
    );
    assert!(machine.said.iter().any(|said| said == "Secure Boot is on"));
    assert!(
        machine.asked.is_empty(),
        "a refused computer was asked to agree"
    );
    for said in &machine.said {
        let lower = said.to_lowercase();
        for suggestion in [
            "turn off",
            "switch off",
            "disable",
            "change the setting",
            "firmware",
        ] {
            assert!(!lower.contains(suggestion), "{said}");
        }
    }
}

/// **Secure Boot that could not be read is refused too** — never taken as off.
#[test]
fn secure_boot_that_could_not_be_read_is_refused() {
    let (machine, released) = Scripted::installable();
    let machine = machine.answering("starting", r#"{"FirmwareType":"UEFI","SecureBoot":null}"#);
    let (ended, machine) = run(machine, released);
    refused_without_change(&ended, &machine, &Refusal::SecureBootNotRead);
}

/// **A BIOS computer, or one whose way of starting Windows would not say, is
/// refused.**
#[test]
fn a_bios_computer_or_an_unread_one_is_refused() {
    let (machine, released) = Scripted::installable();
    let machine = machine.answering("starting", r#"{"FirmwareType":"Legacy","SecureBoot":null}"#);
    let (ended, machine) = run(machine, released);
    refused_without_change(&ended, &machine, &Refusal::NotUefi);

    let (machine, released) = Scripted::installable();
    let (ended, machine) = run(machine.failing("starting"), released);
    refused_without_change(&ended, &machine, &Refusal::StartingNotRead);
}

/// **Disks, or the Windows volume, that could not be read are refused**, and so
/// is a firmware list of systems that could not be read.
#[test]
fn what_could_not_be_read_is_refused() {
    for failing in ["disks", "volume"] {
        let (machine, released) = Scripted::installable();
        let (ended, machine) = run(machine.failing(failing), released);
        refused_without_change(&ended, &machine, &Refusal::DisksNotRead);
    }
    let (machine, released) = Scripted::installable();
    let (ended, machine) = run(machine.failing("entries"), released);
    refused_without_change(&ended, &machine, &Refusal::EntriesNotRead);
}

/// **A Windows disk laid out the older way is refused.**
#[test]
fn a_windows_disk_that_is_not_gpt_is_refused() {
    let (machine, released) = Scripted::installable();
    let machine = machine.answering(
        "disks",
        &DISKS.replacen(r#""PartitionStyle":"GPT""#, r#""PartitionStyle":"MBR""#, 1),
    );
    let (ended, machine) = run(machine, released);
    refused_without_change(
        &ended,
        &machine,
        &Refusal::WindowsDiskNotSupported("Samsung SSD 980 1TB".to_owned()),
    );
}

/// **What an earlier start left — its entry, or its area — is refused**, and
/// never removed by a run that did not make it.
#[test]
fn what_an_earlier_start_left_is_refused_and_not_removed() {
    let (machine, released) = Scripted::installable();
    let machine = machine.answering(
        "entries",
        &format!("identifier {THE_ENTRY}\ndescription alo OS\n"),
    );
    let (ended, machine) = run(machine, released);
    refused_without_change(&ended, &machine, &Refusal::AlreadyStarted);

    let (machine, released) = Scripted::installable();
    let machine = machine.answering(
        "disks",
        &DISKS.replace(r#""Label":"Recovery""#, r#""Label":"ALO-INSTALL""#),
    );
    let (ended, machine) = run(machine, released);
    refused_without_change(&ended, &machine, &Refusal::AlreadyStarted);
}

/// **BitLocker part way through encrypting is refused.**
#[test]
fn bitlocker_part_way_through_is_refused() {
    let (machine, released) = Scripted::installable();
    let machine = machine.answering(
        "bitlocker",
        r#"{"VolumeStatus":"EncryptionInProgress","ProtectionStatus":"Off"}"#,
    );
    let (ended, machine) = run(machine, released);
    refused_without_change(
        &ended,
        &machine,
        &Refusal::BitLockerChanging(alo_installer::Letter::of("C").unwrap()),
    );
}

/// **Too little free space is refused with the numbers**, and a BitLocker
/// state Windows would not report is said and decides nothing.
#[test]
fn too_little_space_is_refused_with_the_numbers() {
    let (machine, released) = Scripted::installable();
    let machine = machine
        .answering("volume", &VOLUME.replace("600000000000", "10000000000"))
        .failing("bitlocker");
    let (ended, machine) = run(machine, released);
    refused_without_change(
        &ended,
        &machine,
        &Refusal::NotEnoughSpace {
            volume: alo_installer::Letter::of("C").unwrap(),
            needed: alo_installer::THE_AREA + alo_installer::WINDOWS_KEEPS_FREE,
            free: 10_000_000_000,
        },
    );
    assert_eq!(
        machine.said.last().unwrap(),
        "Windows on C: needs 17 GB free for this and has 9 GB, so nothing was changed"
    );
    assert!(
        machine.said.iter().any(|said| said
            == "Whether Windows on C: is encrypted with BitLocker could not be found out")
    );
}

/// **A computer with no empty second disk is refused**: installing onto the
/// disk Windows is on is not what this environment does.
#[test]
fn a_computer_with_no_empty_second_disk_is_refused() {
    let (machine, released) = Scripted::installable();
    let machine = machine.answering(
        "disks",
        &DISKS.replace(r#""Partitions":[]"#, r#""Partitions":[{"PartitionNumber":1,"GptType":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","Label":"Games"}]"#),
    );
    let (ended, machine) = run(machine, released);
    refused_without_change(&ended, &machine, &Refusal::NoDiskForAloOs);
    assert!(machine.said.iter().any(|said| said
        == "The disk Msft Virtual Disk (32 GB) already holds files or a system, so alo OS does not use it"));
}

/// **Nothing typed stops, and anything but an offered disk's name is refused**
/// — the disk Windows is on included.
#[test]
fn the_consent_is_the_disks_name_and_nothing_else() {
    for (typed, refusal) in [
        ("\r\n", Refusal::NotAgreed),
        ("yes\n", Refusal::NotADisksName),
        ("Samsung SSD 980 1TB\n", Refusal::NotADisksName),
        ("Msft Virtual\n", Refusal::NotADisksName),
    ] {
        let (mut machine, released) = Scripted::installable();
        machine.typed = typed.to_owned();
        let (ended, machine) = run(machine, released);
        refused_without_change(&ended, &machine, &refusal);
        assert_eq!(machine.asked.len(), 1, "{typed:?}");
    }
}

// ---------------------------------------------------------------------------
// Failing part way, and putting it back.
// ---------------------------------------------------------------------------

/// **A step that fails puts back every change before it, newest first**, at
/// every step, and says nothing was changed only once that is true.
#[test]
fn a_failure_at_each_step_puts_back_everything_before_it() {
    let every_step = [
        ("shrink", vec![]),
        ("make-area", vec!["grow-back"]),
        ("prepare-area", vec!["remove-area", "grow-back"]),
        ("shortcut", vec!["remove-area", "grow-back"]),
        (
            "write-entry",
            vec!["remove-what-was-left", "remove-area", "grow-back"],
        ),
        (
            "list-entry",
            vec![
                "remove-entry",
                "remove-what-was-left",
                "remove-area",
                "grow-back",
            ],
        ),
        (
            "take-letter",
            vec![
                "remove-entry",
                "remove-what-was-left",
                "remove-area",
                "grow-back",
            ],
        ),
        (
            "next",
            vec![
                "forget-next",
                "remove-entry",
                "remove-what-was-left",
                "remove-area",
                "grow-back",
            ],
        ),
    ];
    for (failing, put_back) in every_step {
        let (machine, released) = Scripted::installable();
        let (ended, machine) = run(machine.failing(failing), released);
        assert_eq!(ended, Ended::Refused(Refusal::PutBack), "{failing}");
        let kinds = machine.kinds();
        let failed_at = kinds.iter().position(|kind| *kind == failing).unwrap();
        let after: Vec<&str> = kinds[failed_at + 1..]
            .iter()
            .copied()
            .filter(|kind| *kind != "volume")
            .collect();
        assert_eq!(after, put_back, "{failing}");
        assert!(!kinds.contains(&"restart"), "{failing} restarted");
        assert_eq!(
            machine.said.last().unwrap(),
            "Preparing this computer did not finish. Everything that was changed has been put back, so nothing was changed"
        );
    }
}

/// **A shrink that reported failure and did shrink is grown back**, and one
/// that did not is left alone.
#[test]
fn a_shrink_that_reported_failure_is_asked_about_not_believed() {
    let (machine, released) = Scripted::installable();
    let (_, machine) = run(machine.failing("shrink"), released);
    assert!(!machine.kinds().contains(&"grow-back"));

    let (mut machine, released) = Scripted::installable();
    machine.volume_once_shrinking_ran =
        Some(VOLUME.replace("\"Size\":998000000000", "\"Size\":996000000000"));
    let (ended, machine) = run(machine.failing("shrink"), released);
    assert_eq!(
        machine
            .kinds()
            .iter()
            .filter(|kind| **kind == "volume")
            .count(),
        2
    );
    assert_eq!(ended, Ended::Refused(Refusal::PutBack));
    assert_eq!(machine.kinds().last(), Some(&"grow-back"));
}

/// **A copy that does not read back as written is a failure**, and is put back.
#[test]
fn a_copy_that_does_not_read_back_is_put_back() {
    let (mut machine, released) = Scripted::installable();
    machine.corrupting.push(beneath(
        &PathBuf::from("E:\\"),
        "EFI/alo-installing/vmlinuz",
    ));
    let (ended, machine) = run(machine, released);
    assert_eq!(ended, Ended::Refused(Refusal::PutBack));
    assert!(!machine.kinds().contains(&"write-entry"));
    assert!(machine.kinds().ends_with(&["remove-area", "grow-back"]));
}

/// **An area made anywhere but where the shrink freed is put back at once**,
/// before it is formatted.
#[test]
fn an_area_made_in_the_wrong_place_is_never_formatted() {
    let (machine, released) = Scripted::installable();
    let machine = machine.answering("make-area", r#"{"PartitionNumber":5,"Offset":4096}"#);
    let (ended, machine) = run(machine, released);
    assert_eq!(ended, Ended::Refused(Refusal::PutBack));
    assert!(!machine.kinds().contains(&"prepare-area"));
    assert!(machine.ran.contains(&Program::RemovingTheArea {
        disk: alo_installer::DiskNumber(0),
        partition: alo_installer::PartitionNumber(5),
        offset: 4096,
    }));
}

/// **When putting back fails too, the installer says exactly what remains** and
/// never that nothing was changed.
#[test]
fn what_could_not_be_put_back_is_said_exactly() {
    let (machine, released) = Scripted::installable();
    let machine = machine
        .failing("next")
        .failing("forget-next")
        .failing("remove-area");
    let (ended, machine) = run(machine, released);
    let c = alo_installer::Letter::of("C").unwrap();
    assert_eq!(
        ended,
        Ended::NotPutBack(vec![
            Remains::TheNextStart,
            Remains::TheArea("Samsung SSD 980 1TB".to_owned()),
            Remains::Smaller(c),
        ])
    );
    // With the area still there, Windows is not asked to grow into it.
    assert!(!machine.kinds().contains(&"grow-back"));
    let tail: Vec<&str> = machine
        .said
        .iter()
        .rev()
        .take(4)
        .rev()
        .map(String::as_str)
        .collect();
    assert_eq!(
        tail,
        [
            "Preparing this computer did not finish, and not everything could be put back. Windows still starts. This remains:",
            "The next restart starts the alo OS installer, which installs onto the disk you named",
            "An area labelled ALO-INSTALL remains on the disk Samsung SSD 980 1TB",
            "Windows on C: is 1 GB smaller than it was",
        ]
    );
    assert!(
        machine
            .said
            .iter()
            .all(|said| !said.contains("nothing was changed"))
    );
}

// ---------------------------------------------------------------------------
// The checks themselves.
// ---------------------------------------------------------------------------

/// **Checking the computer runs reads and nothing else.**
#[test]
fn checking_the_computer_changes_nothing() {
    let (mut machine, _) = Scripted::installable();
    let found = check(&mut machine);
    assert!(!machine.changed_anything());
    assert_eq!(found.starting.secure_boot, Some(false));
    assert_eq!(found.memory, Some(34_190_917_632));
}

// ---------------------------------------------------------------------------
// Which system this computer starts when nobody chooses.
// ---------------------------------------------------------------------------

/// A machine whose start partition holds the loader's environment block, with
/// this saved in it, and whose person types this.
fn asked_which_system_starts(saved: &str, typed: &str) -> Scripted {
    let (mut machine, _) = Scripted::installable();
    let mut block = alo_starting::EnvironmentBlock::empty();
    block.keep(alo_starting::SAVED_ENTRY, saved).unwrap();
    let letter = alo_installer::Letter::of(alo_installer::THE_START_PARTITIONS_LETTER).unwrap();
    machine
        .files
        .insert(alo_installer::the_block(letter), block.written().unwrap());
    machine.answers.insert("give-esp-letter", printed(""));
    machine.answers.insert("take-esp-letter", printed(""));
    machine.typed = format!("{typed}\n");
    machine
}

/// The one file, as the loader's own side reads it afterwards.
fn what_the_loaders_side_reads(machine: &Scripted) -> alo_starting::System {
    let letter = alo_installer::Letter::of(alo_installer::THE_START_PARTITIONS_LETTER).unwrap();
    let bytes = machine
        .files
        .get(&alo_installer::the_block(letter))
        .unwrap();
    let block = alo_starting::EnvironmentBlock::read(bytes).unwrap();
    alo_starting::TheStartingChoice::read(&block)
}

/// **Both sides read the same answer, because there is one answer.** What the
/// Windows program writes, the crate that owns the loader's side reads back as
/// the system the person chose — and the file keeps the length it had.
#[test]
fn the_default_windows_writes_is_the_default_alo_os_reads() {
    let machine = asked_which_system_starts("", alo_installer::DEFAULT_CHANGE_IT.says());
    let letter = alo_installer::Letter::of(alo_installer::THE_START_PARTITIONS_LETTER).unwrap();
    let was = machine
        .files
        .get(&alo_installer::the_block(letter))
        .unwrap()
        .len();
    let mut machine = machine;
    let ended = alo_installer::which_system_starts(&mut machine, &strings());

    assert_eq!(
        ended,
        alo_installer::TheDefault::Changed(alo_starting::System::Windows),
        "{:#?}",
        machine.said
    );
    assert_eq!(
        what_the_loaders_side_reads(&machine),
        alo_starting::System::Windows
    );
    assert_eq!(
        machine
            .files
            .get(&alo_installer::the_block(letter))
            .unwrap()
            .len(),
        was,
        "the block was written at a different length than it was read at"
    );
    // The partition is reached and let go again, and nothing else is run.
    assert_eq!(machine.kinds(), ["give-esp-letter", "take-esp-letter"]);
}

/// **And the other way round**: a computer that starts Windows is changed to
/// start alo OS, which the loader's side reads as alo OS.
#[test]
fn the_same_holds_the_other_way_round() {
    let mut machine = asked_which_system_starts(
        alo_starting::THE_WINDOWS_ENTRY,
        alo_installer::DEFAULT_CHANGE_IT.says(),
    );
    let ended = alo_installer::which_system_starts(&mut machine, &strings());
    assert_eq!(
        ended,
        alo_installer::TheDefault::Changed(alo_starting::System::AloOs)
    );
    assert_eq!(
        what_the_loaders_side_reads(&machine),
        alo_starting::System::AloOs
    );
}

/// **A person who types nothing changes nothing**, and the answer they were
/// shown is the one the file holds.
#[test]
fn the_default_is_shown_and_left_alone_unless_the_word_is_typed() {
    for typed in ["", "   ", "yes", "windows", "alo OS"] {
        let mut machine = asked_which_system_starts(alo_starting::THE_WINDOWS_ENTRY, typed);
        let ended = alo_installer::which_system_starts(&mut machine, &strings());
        assert_eq!(
            ended,
            alo_installer::TheDefault::Kept(alo_starting::System::Windows),
            "{typed:?}"
        );
        assert_eq!(
            what_the_loaders_side_reads(&machine),
            alo_starting::System::Windows,
            "{typed:?}"
        );
        assert!(
            machine
                .said
                .iter()
                .any(|said| said.contains("Windows") && said.contains("nobody chooses")),
            "{typed:?}: {:#?}",
            machine.said
        );
    }
}

/// **The file is on the partition both systems share, at the path the loader's
/// own side names**, and the Windows side spells it with Windows' separators.
#[test]
fn the_file_is_the_one_the_loader_reads() {
    let letter = alo_installer::Letter::of(alo_installer::THE_START_PARTITIONS_LETTER).unwrap();
    let file = alo_installer::the_block(letter);
    let named = file.to_string_lossy().replace('\\', "/");
    assert!(
        named.ends_with(alo_starting::THE_BLOCK_ON_THE_ESP),
        "{named} is not {}",
        alo_starting::THE_BLOCK_ON_THE_ESP
    );
    assert!(named.starts_with("S:/"), "{named}");
}

/// **Nothing to read is said rather than guessed at**, and a file that is not
/// an environment block is never written over.
#[test]
fn a_start_partition_without_the_block_is_said() {
    let mut machine = Scripted::installable().0;
    machine.answers.insert("give-esp-letter", printed(""));
    machine.answers.insert("take-esp-letter", printed(""));
    let ended = alo_installer::which_system_starts(&mut machine, &strings());
    assert_eq!(ended, alo_installer::TheDefault::NotThere);
    assert!(
        machine
            .said
            .contains(&sentence(alo_installer::DEFAULT_NOT_THERE))
    );

    let mut machine = asked_which_system_starts("", alo_installer::DEFAULT_CHANGE_IT.says());
    let letter = alo_installer::Letter::of(alo_installer::THE_START_PARTITIONS_LETTER).unwrap();
    let file = alo_installer::the_block(letter);
    machine
        .files
        .insert(file.clone(), b"something else".to_vec());
    let ended = alo_installer::which_system_starts(&mut machine, &strings());
    assert_eq!(ended, alo_installer::TheDefault::NotRead);
    assert_eq!(
        machine.files.get(&file).map(Vec::as_slice),
        Some(b"something else".as_slice()),
        "a file that is not an environment block was written over"
    );
}

/// **A start partition Windows will not reach is said, and nothing is
/// written.**
#[test]
fn a_start_partition_that_cannot_be_reached_is_said() {
    let mut machine = asked_which_system_starts("", alo_installer::DEFAULT_CHANGE_IT.says())
        .failing("give-esp-letter");
    let ended = alo_installer::which_system_starts(&mut machine, &strings());
    assert_eq!(ended, alo_installer::TheDefault::NotReached);
    assert_eq!(
        what_the_loaders_side_reads(&machine),
        alo_starting::System::AloOs
    );
    assert!(
        machine
            .said
            .contains(&sentence(alo_installer::DEFAULT_NOT_REACHED))
    );
}

// ---------------------------------------------------------------------------
// The way back into alo OS, from inside Windows.
// ---------------------------------------------------------------------------

/// The firmware's list on a computer alo OS was installed onto.
const ENTRIES_WITH_ALO_OS: &str = "identifier              {bootmgr}\n\
     description             Windows Boot Manager\n\
     \n\
     identifier              {6b1d5c2a-0f3e-11ef-9a1b-00155d012345}\n\
     description             alo OS\n";

/// A machine whose firmware lists alo OS, and whose person types this.
fn asked_to_switch(typed: &str) -> (Scripted, Released) {
    let (mut machine, released) = Scripted::installable();
    machine = machine.answering("entries", ENTRIES_WITH_ALO_OS);
    machine.typed = format!("{typed}\n");
    (machine, released)
}

/// **The switch sets the next start and nothing else**, once the person has
/// typed the word — never the order, never the default, no disk.
#[test]
fn the_way_back_sets_the_next_start_and_nothing_else() {
    let (machine, _) = asked_to_switch(alo_installer::SWITCH_AGREED.says());
    let mut machine = machine;
    let switched = alo_installer::restart_into_alo_os(&mut machine, &strings());
    assert_eq!(
        switched,
        alo_installer::Switched::Restarting { restarted: true },
        "{:#?}",
        machine.said
    );
    assert_eq!(machine.kinds(), ["entries", "next", "restart"]);
    assert!(
        machine
            .said
            .contains(&sentence(alo_installer::SWITCH_WILL_RESTART))
    );
    // The question names the word to type, and says it rather than leaving the
    // gap: measured on 2026-09-23, it asked for `{word}` on a real Windows.
    let asked = machine.asked.last().expect("it asked");
    assert!(!asked.contains('{'), "{asked}");
    assert!(
        asked.contains(alo_installer::SWITCH_AGREED.says()),
        "{asked}"
    );
}

/// **A person who types nothing changes nothing.**
#[test]
fn the_way_back_asks_first_and_takes_no_for_an_answer() {
    for typed in ["", "   ", "yes", "alo OS", "restart now"] {
        let (machine, _) = asked_to_switch(typed);
        let mut machine = machine;
        let switched = alo_installer::restart_into_alo_os(&mut machine, &strings());
        assert_eq!(switched, alo_installer::Switched::NotAgreed, "{typed:?}");
        assert_eq!(machine.kinds(), ["entries"], "{typed:?}");
        assert!(!machine.changed_anything(), "{typed:?}");
        assert!(
            machine
                .said
                .contains(&sentence(alo_installer::SWITCH_NOT_AGREED)),
            "{typed:?}"
        );
    }
}

/// **With no entry for alo OS it says so and changes nothing**, and the same
/// when the firmware's list could not be read at all.
#[test]
fn the_way_back_says_when_there_is_nothing_to_start() {
    let (machine, _) = asked_to_switch(alo_installer::SWITCH_AGREED.says());
    let mut machine = machine.answering(
        "entries",
        "identifier              {bootmgr}\ndescription             Windows Boot Manager\n",
    );
    let switched = alo_installer::restart_into_alo_os(&mut machine, &strings());
    assert_eq!(switched, alo_installer::Switched::NotThere);
    assert!(!machine.changed_anything());
    assert!(
        machine
            .said
            .contains(&sentence(alo_installer::SWITCH_NOT_THERE))
    );

    let (machine, _) = asked_to_switch(alo_installer::SWITCH_AGREED.says());
    let mut machine = machine.failing("entries");
    let switched = alo_installer::restart_into_alo_os(&mut machine, &strings());
    assert_eq!(switched, alo_installer::Switched::NotRead);
    assert!(!machine.changed_anything());
    assert!(
        machine
            .said
            .contains(&sentence(alo_installer::SWITCH_NOT_READ))
    );
}

/// **A next start that could not be set is forgotten rather than left half
/// set**, and the computer is not restarted.
#[test]
fn a_next_start_that_failed_is_forgotten_and_said() {
    let (machine, _) = asked_to_switch(alo_installer::SWITCH_AGREED.says());
    let mut machine = machine.failing("next");
    let switched = alo_installer::restart_into_alo_os(&mut machine, &strings());
    assert_eq!(switched, alo_installer::Switched::NotSet);
    assert_eq!(machine.kinds(), ["entries", "next", "forget-next"]);
    assert!(!machine.kinds().contains(&"restart"));
    assert!(
        machine
            .said
            .contains(&sentence(alo_installer::SWITCH_NOT_SET))
    );
}

/// **The installer leaves the way back where a person finds it**: its own
/// bytes under Windows' place for programs, and a shortcut that starts them
/// with the switch's word.
#[test]
fn the_installer_leaves_a_copy_of_itself_and_a_shortcut() {
    let (machine, released) = Scripted::installable();
    let (ended, machine) = run(machine, released);
    assert_eq!(ended, Ended::Staged { restarted: true });
    let left = Path::new(alo_installer::THE_PROGRAMS_HOME).join(alo_installer::THE_PROGRAMS_NAME);
    assert_eq!(
        machine.files.get(&left).map(Vec::as_slice),
        Some(b"the bytes of the installer".as_slice())
    );
    assert!(machine.kinds().contains(&"shortcut"));
    let script = Program::MakingTheShortcut.script().unwrap();
    assert!(script.contains(alo_installer::THE_SWITCHS_WORD), "{script}");
    assert!(
        script.contains(alo_installer::THE_PROGRAMS_NAME),
        "{script}"
    );
}

// ---------------------------------------------------------------------------
// Fast Startup: the one question with two answers.
// ---------------------------------------------------------------------------

/// A computer whose Fast Startup is on, whose person types these answers after
/// the disk's name.
fn with_fast_startup_on(answers: &[&str]) -> (Scripted, Released) {
    let (mut machine, released) = Scripted::installable();
    machine = machine.answering(
        "fast-startup",
        r#"{"HiberbootEnabled":1,"HibernateEnabled":1}"#,
    );
    machine.then_typed = answers.iter().map(|typed| format!("{typed}\n")).collect();
    (machine, released)
}

/// The English of one of the installer's sentences.
fn sentence(word: alo_strings::Word) -> String {
    strings()
        .say(&word.key(), &alo_strings::Filling::nothing())
        .text()
        .to_owned()
}

/// **With Fast Startup on, the person is asked in the owner's words, and
/// *turn off* sets Windows' own value** — and never removes hibernation
/// (ADR 0064 term 9).
#[test]
fn fast_startup_on_is_asked_about_and_turned_off_when_the_person_says_so() {
    let (machine, released) = with_fast_startup_on(&["turn off"]);
    let (ended, machine) = run(machine, released);
    assert_eq!(ended, Ended::Staged { restarted: true });
    assert!(
        machine
            .said
            .contains(&sentence(alo_installer::ASK_FAST_STARTUP)),
        "{:#?}",
        machine.said
    );
    assert!(machine.kinds().contains(&"fast-startup-off"));
    // Turned off before any disk was touched, and once.
    let kinds = machine.kinds();
    let off = kinds.iter().position(|kind| *kind == "fast-startup-off");
    let shrink = kinds.iter().position(|kind| *kind == "shrink");
    assert!(off < shrink && off.is_some());
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| **kind == "fast-startup-off")
            .count(),
        1
    );
}

/// **Leaving it on changes nothing about it**, and the install goes on.
#[test]
fn fast_startup_left_on_changes_nothing_about_it() {
    let (machine, released) = with_fast_startup_on(&["leave on"]);
    let (ended, machine) = run(machine, released);
    assert_eq!(ended, Ended::Staged { restarted: true });
    assert!(!machine.kinds().contains(&"fast-startup-off"));
    assert!(
        machine
            .said
            .contains(&sentence(alo_installer::FAST_STARTUP_LEFT_ON))
    );
}

/// **An answer that is neither is asked again, and after that it is left on.**
#[test]
fn an_answer_that_is_neither_is_asked_again_and_then_left_on() {
    let (machine, released) = with_fast_startup_on(&["", "maybe", "leave on"]);
    let (ended, machine) = run(machine, released);
    assert_eq!(ended, Ended::Staged { restarted: true });
    assert!(!machine.kinds().contains(&"fast-startup-off"));
    let asked = machine
        .asked
        .iter()
        .filter(|said| said.contains("turn off"))
        .count();
    assert_eq!(asked, 3, "{:#?}", machine.asked);

    let (machine, released) = with_fast_startup_on(&["", "", ""]);
    let (ended, machine) = run(machine, released);
    assert_eq!(ended, Ended::Staged { restarted: true });
    assert!(!machine.kinds().contains(&"fast-startup-off"));
    assert!(
        machine
            .said
            .contains(&sentence(alo_installer::FAST_STARTUP_LEFT_ON))
    );
}

/// **With Fast Startup off, or unreadable, nothing is asked and nothing is
/// changed about it.**
#[test]
fn fast_startup_that_is_off_or_unread_is_not_asked_about() {
    for read in [
        r#"{"HiberbootEnabled":0,"HibernateEnabled":1}"#,
        // `powercfg /h off` leaves the first value at 1 (`docs/quirks.md`).
        r#"{"HiberbootEnabled":1,"HibernateEnabled":0}"#,
        "not an answer",
    ] {
        let (machine, released) = Scripted::installable();
        let (ended, machine) = run(machine.answering("fast-startup", read), released);
        assert_eq!(ended, Ended::Staged { restarted: true });
        assert!(!machine.kinds().contains(&"fast-startup-off"), "{read}");
        assert!(
            !machine
                .said
                .contains(&sentence(alo_installer::ASK_FAST_STARTUP)),
            "{read}"
        );
        assert_eq!(machine.asked.len(), 1, "{read}");
    }
}

/// **A Fast Startup that was turned off is put back when staging fails**, to
/// the value Windows held, and what could not be put back is said.
#[test]
fn fast_startup_is_put_back_when_a_later_step_fails() {
    let (machine, released) = with_fast_startup_on(&["turn off"]);
    let (ended, machine) = run(machine.failing("shrink"), released);
    assert_eq!(
        ended,
        Ended::Refused(Refusal::PutBack),
        "{:#?}",
        machine.said
    );
    assert_eq!(
        machine.kinds().last(),
        Some(&"fast-startup-back-on"),
        "{:?}",
        machine.kinds()
    );

    let (machine, released) = with_fast_startup_on(&["turn off"]);
    let (ended, machine) = run(
        machine.failing("shrink").failing("fast-startup-back-on"),
        released,
    );
    assert_eq!(
        ended,
        Ended::NotPutBack(vec![Remains::FastStartupOff]),
        "{:#?}",
        machine.said
    );
    assert!(
        machine
            .said
            .contains(&sentence(alo_installer::REMAINS_FAST_STARTUP_OFF))
    );
}

/// **The choice the installer writes is the line the environment's loader
/// reads**, beside the loader, in the file the loader sources.
#[test]
fn the_choice_is_what_the_environments_loader_reads() {
    let grub = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../image/installing/grub.cfg"),
    )
    .expect("the environment's loader configuration is in the repository");
    let file = THE_CHOICE.rsplit('/').next().unwrap();
    // `${config_directory}`, the directory the loader read its own configuration
    // from, and never `${cmdpath}`, which the base's signed loader leaves empty
    // (`docs/quirks.md`, *Fedora's signed loader leaves `cmdpath` empty*).
    assert!(
        grub.contains(&format!("${{config_directory}}/{file}")),
        "{grub}"
    );
    let entry: String = grub
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect();
    assert!(!entry.contains("${cmdpath}"), "{grub}");
    let variable = alo_installer::THE_CHOICE_BEGINS
        .strip_prefix("set ")
        .unwrap()
        .trim_end_matches('=');
    assert!(grub.contains(&format!("${{{variable}}}")), "{grub}");
    assert!(THE_CHOICE.starts_with("EFI/BOOT/"));

    let recipe = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../image/installing/Containerfile"),
    )
    .expect("the environment's recipe is in the repository");
    for needed in EVERY_FILE_IT_NEEDS {
        assert!(
            recipe.contains(needed),
            "the recipe does not produce {needed}"
        );
    }
    assert!(
        recipe.contains(
            alo_installer::THE_LOADER
                .trim_start_matches('\\')
                .replace('\\', "/")
                .as_str()
        )
    );
}
