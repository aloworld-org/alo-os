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
    /// What the person types.
    typed: String,
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
        Program::Shrinking { .. } => "shrink",
        Program::MakingTheArea { .. } => "make-area",
        Program::PreparingTheArea { .. } => "prepare-area",
        Program::AddingTheEntry => "add-entry",
        Program::PointingTheEntryAtTheArea { .. } => "point-area",
        Program::PointingTheEntryAtTheLoader { .. } => "point-loader",
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
                "add-entry",
                printed(&format!(
                    "The entry was successfully copied to {THE_ENTRY}."
                )),
            ),
            ("point-area", printed("")),
            ("point-loader", printed("")),
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
            "shrink",
            "make-area",
            "prepare-area",
            "add-entry",
            "point-area",
            "point-loader",
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
        ("add-entry", vec!["remove-area", "grow-back"]),
        (
            "point-area",
            vec!["remove-entry", "remove-area", "grow-back"],
        ),
        (
            "point-loader",
            vec!["remove-entry", "remove-area", "grow-back"],
        ),
        (
            "list-entry",
            vec!["remove-entry", "remove-area", "grow-back"],
        ),
        (
            "take-letter",
            vec!["remove-entry", "remove-area", "grow-back"],
        ),
        (
            "next",
            vec!["forget-next", "remove-entry", "remove-area", "grow-back"],
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
    assert!(!machine.kinds().contains(&"add-entry"));
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

/// **The choice the installer writes is the line the environment's loader
/// reads**, beside the loader, in the file the loader sources.
#[test]
fn the_choice_is_what_the_environments_loader_reads() {
    let grub = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../image/installing/grub.cfg"),
    )
    .expect("the environment's loader configuration is in the repository");
    let file = THE_CHOICE.rsplit('/').next().unwrap();
    assert!(grub.contains(&format!("${{cmdpath}}/{file}")), "{grub}");
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
