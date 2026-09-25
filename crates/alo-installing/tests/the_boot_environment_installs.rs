//! The installation, step by step, against a machine made of answers — the
//! road that installs, and every road that refuses.
//!
//! The virtual-machine test beside this file is the same sequence on a real
//! kernel with real disks and the real registry. This is where each refusal is
//! walked on its own, because a virtual machine that is booted once per refusal
//! is a quarter of an hour per sentence, and the question each refusal answers
//! — *was anything written, and was the person told why* — is a question about
//! the sequence, not about the machine.
//!
//! What every refusal here is held to, beyond its own sentence:
//!
//! - **the writer never ran**, so nothing was written to any disk;
//! - **no restart**, so the person can read why;
//! - the last line is *you can turn this computer off or restart it now*;
//! - every line on the screen came from the vocabulary `alo-saying` collects,
//!   and none of them is a missing key.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_installing::{
    Ended, Environment, Program, Ran, Refusal, STILL_EVERY, TheMachine, WHERE_IT_IS, install,
    installing_words,
};
use alo_strings::{Said, Strings};

/// The owner's release, as the checker printed its verification inside the
/// environment on 2026-09-15.
fn the_owners_verification(digest: &str) -> String {
    format!(
        r#"[{{"critical":{{"identity":{{"docker-reference":"ghcr.io/aloworld-org/alo-os@{digest}"}},"image":{{"docker-manifest-digest":"{digest}"}},"type":"https://sigstore.dev/cosign/sign/v1"}},"optional":{{}}}}]"#
    )
}

/// The machine the virtual-machine test builds, as the disk lister prints it:
/// Windows' partitions and the staged installer on the first disk, and an empty
/// second disk.
/// What the firmware's own tool prints after an install, on the walk's
/// machine: the entry `bootc` left named *Fedora*, Windows' own, and the
/// firmware's two.
const THE_START_ENTRIES: &str = "BootCurrent: 000B\n\
     Timeout: 0 seconds\n\
     BootOrder: 000B,0004,0000\n\
     Boot0000* BootManagerMenuApp\tFvVol(5c60f367-a505-419a-859e-2a4ff6ca6fe5)\n\
     Boot0004* Windows Boot Manager\tHD(1,GPT,0506f28d-c7cb-426e-ad7b-b9ec92014753,0x800,0x96000)/File(\\EFI\\Microsoft\\Boot\\bootmgfw.efi)\n\
     Boot000B* Fedora\tHD(2,GPT,1505d88b-67da-4187-b683-f36a1169e81e,0x1000,0x100000)/File(\\EFI\\fedora\\shimx64.efi)\n";

/// What the tool prints when it has made the entry this environment asks for.
const THE_ENTRY_IT_MADE: &str = "Boot000C* alo OS\tHD(2,GPT,1505d88b-67da-4187-b683-f36a1169e81e,0x1000,0x100000)/File(\\EFI\\fedora\\shimx64.efi)";

const THE_DISKS: &str = r#"{"blockdevices": [
   {"name": "/dev/vda", "type": "disk", "ro": false, "mountpoints": [null], "children": [
      {"name": "/dev/vda1", "type": "part", "ro": false, "mountpoints": [null], "parttype": "c12a7328-f81f-11d2-ba4b-00a0c93ec93b", "label": null},
      {"name": "/dev/vda2", "type": "part", "ro": false, "mountpoints": [null], "parttype": "e3c9e316-0b5c-4db8-817d-f92df00215ae", "label": null},
      {"name": "/dev/vda3", "type": "part", "ro": false, "mountpoints": [null], "parttype": "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7", "label": null},
      {"name": "/dev/vda4", "type": "part", "ro": false, "mountpoints": [null], "parttype": "de94bba4-06d1-4d40-a16a-bfd50179d6ac", "label": null},
      {"name": "/dev/vda5", "type": "part", "ro": false, "mountpoints": [null], "parttype": "c12a7328-f81f-11d2-ba4b-00a0c93ec93b", "label": "ALO-INSTALL"}
   ]},
   {"name": "/dev/vdb", "type": "disk", "ro": false, "mountpoints": [null]},
   {"name": "/dev/nvme0n1", "type": "disk", "ro": false, "mountpoints": [null], "children": [
      {"name": "/dev/nvme0n1p1", "type": "part", "ro": false, "mountpoints": [null], "parttype": "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7", "label": "DATA"}
   ]}
]}"#;

/// A machine that answers from a script, and remembers what it was asked.
struct Scripted {
    /// The kernel command line.
    command_line: std::io::Result<String>,
    /// Which disk names appear, and where the kernel put them.
    appears: Vec<(&'static str, &'static str)>,
    /// The answer each program gives, in the order programs are run.
    answers: VecDeque<std::io::Result<Ran>>,
    /// Every sentence said, in order.
    said: Vec<String>,
    /// Every line of the machinery's noted, in order.
    noted: Vec<String>,
    /// Every program run, in order.
    ran: Vec<Program>,
    /// How many times the long write was said to be still going.
    stills: usize,
    /// Whether a wired connection comes up.
    connects: bool,
}

impl Scripted {
    /// A machine with the test's disks, whose person chose `disk`.
    fn choosing(disk: &str) -> Self {
        Self {
            command_line: Ok(format!(
                "BOOT_IMAGE=/EFI/alo-installing/vmlinuz rd.systemd.unit=alo-installing.target \
                 rd.neednet=1 ip=dhcp console=ttyS0,115200 console=tty0 alo.installing.to={disk}"
            )),
            appears: vec![
                ("virtio-alo-target", "/dev/vdb"),
                ("virtio-alo-windows", "/dev/vda"),
                ("nvme-Data_Disk_123", "/dev/nvme0n1"),
            ],
            answers: VecDeque::new(),
            said: Vec::new(),
            noted: Vec::new(),
            ran: Vec::new(),
            stills: 0,
            connects: true,
        }
    }

    /// The next program answers this.
    fn answering(mut self, answer: std::io::Result<Ran>) -> Self {
        self.answers.push_back(answer);
        self
    }

    /// Whether anything was written.
    fn wrote(&self) -> bool {
        self.ran.iter().any(Program::writes)
    }

    /// Whether the machine was restarted.
    fn restarted(&self) -> bool {
        self.ran.contains(&Program::Restarting)
    }
}

impl TheMachine for Scripted {
    fn say(&mut self, said: &Said) {
        assert!(
            !said.is_a_bug(),
            "a missing or unfilled sentence: {}",
            said.text()
        );
        self.said.push(said.text().to_owned());
    }

    fn note(&mut self, line: &str) {
        self.noted.push(line.to_owned());
    }

    fn command_line(&mut self) -> std::io::Result<String> {
        match &self.command_line {
            Ok(line) => Ok(line.clone()),
            Err(why) => Err(std::io::Error::new(why.kind(), why.to_string())),
        }
    }

    fn wait_for(&mut self, disk: &alo_installing::DiskName, at_most: Duration) -> Option<PathBuf> {
        assert!(
            at_most >= Duration::from_secs(10),
            "a disk is given time to appear"
        );
        self.appears
            .iter()
            .find(|(name, _)| *name == disk.as_str())
            .map(|(_, device)| PathBuf::from(device))
    }

    fn run(&mut self, program: &Program, still: &Said, every: Duration) -> std::io::Result<Ran> {
        self.ran.push(program.clone());
        if program.writes() {
            // A write that takes three and a half intervals.
            assert_eq!(every, STILL_EVERY);
            for _ in 0..3 {
                self.say(still);
                self.stills += 1;
            }
        }
        if *program == Program::WaitingForTheNetwork {
            return Ok(Ran {
                succeeded: self.connects,
                ..Ran::default()
            });
        }
        if *program == Program::Restarting {
            return Ok(Ran {
                succeeded: true,
                ..Ran::default()
            });
        }
        self.answers
            .pop_front()
            .unwrap_or_else(|| panic!("{program:?} ran and the script has no answer for it"))
    }

    fn pause(&mut self, _for_as_long_as: Duration) {}
}

/// The words, in English.
fn strings() -> Strings {
    Strings::of(installing_words().expect("the words declare"))
}

/// The environment the recipe builds, from the pin this repository ships.
fn the_environment() -> Environment {
    let pinned = std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join(alo_image::THE_PIN));
    Environment::read(Path::new(WHERE_IT_IS), pinned, |_| true).expect("the shipped pin reads")
}

/// A program that succeeded and printed this.
fn succeeded(printed: &str) -> std::io::Result<Ran> {
    Ok(Ran {
        succeeded: true,
        printed: printed.to_owned(),
        complained: String::new(),
    })
}

/// A program that failed and complained this.
fn failed(complained: &str) -> std::io::Result<Ran> {
    Ok(Ran {
        succeeded: false,
        printed: String::new(),
        complained: complained.to_owned(),
    })
}

/// The English of a word, filled with the chosen disk where it has a gap.
fn english(word: alo_strings::Word, disk: &str) -> String {
    word.says().replace("{disk}", disk)
}

/// Everything held of a refusal: its sentence, nothing written, no restart, and
/// the last line.
fn refused_with(machine: &Scripted, ended: &Ended, refusal: &Refusal, sentence: &str) {
    assert_eq!(ended, &Ended::Refused(refusal.clone()));
    assert!(
        !machine.wrote(),
        "a refusal wrote to a disk: {:?}",
        machine.ran
    );
    assert!(!machine.restarted(), "a refusal restarted the machine");
    let last_two = &machine.said[machine.said.len() - 2..];
    assert_eq!(last_two[0], sentence);
    assert_eq!(
        last_two[1],
        alo_installing::EVERY_WORD
            .iter()
            .find(|word| word.named() == "installing.restart-when-ready")
            .expect("the last line is declared")
            .says()
    );
    assert!(sentence.contains("so nothing was changed"), "{sentence}");
}

/// A word by its key.
fn word(named: &str) -> alo_strings::Word {
    *alo_installing::EVERY_WORD
        .iter()
        .find(|word| word.named() == named)
        .unwrap_or_else(|| panic!("{named} is declared"))
}

/// **A genuine release onto an empty second disk is installed, every step is
/// said as it begins, the check comes before the write, the write is the chosen
/// disk by its own name, and the machine restarts.**
#[test]
fn a_genuine_release_onto_the_chosen_disk_is_installed_with_every_step_said() {
    let environment = the_environment();
    let mut machine = Scripted::choosing("virtio-alo-target")
        .answering(succeeded(THE_DISKS))
        .answering(succeeded(&the_owners_verification(
            environment.pin().digest(),
        )))
        .answering(succeeded(""))
        // What the tidying runs, in its own order: the firmware's list, the
        // entry made, the old one taken away, the order, the disks again, and
        // the area removed.
        .answering(succeeded(THE_START_ENTRIES))
        .answering(succeeded(THE_ENTRY_IT_MADE))
        .answering(succeeded(""))
        .answering(succeeded(""))
        .answering(succeeded(THE_DISKS))
        .answering(succeeded(""));

    let ended = install(&mut machine, &strings(), Ok(&environment));

    let disk = alo_installing::DiskName::named("virtio-alo-target").expect("a disk");
    assert_eq!(ended, Ended::Installed(disk.clone()));
    let still = english(word("installing.still-installing"), "");
    assert_eq!(
        machine.said,
        [
            english(word("installing.starting"), ""),
            english(word("installing.reading-the-choice"), ""),
            english(word("installing.looking-for-the-disk"), "virtio-alo-target"),
            english(word("installing.checking-the-disk"), "virtio-alo-target"),
            english(word("installing.connecting"), ""),
            english(word("installing.checking-it-is-genuine"), ""),
            english(word("installing.genuine"), ""),
            english(word("installing.installing"), "virtio-alo-target"),
            still.clone(),
            still.clone(),
            still,
            english(word("installing.tidying"), ""),
            // Taking the area away is a write, and a write says its own
            // sentence while it runs, as the install's does.
            english(word("installing.tidying"), ""),
            english(word("installing.tidying"), ""),
            english(word("installing.tidying"), ""),
            english(word("installing.tidied"), ""),
            english(word("installing.installed"), ""),
        ]
    );
    assert_eq!(
        machine.stills, 6,
        "the write's three, and the area removal's three"
    );

    assert_eq!(machine.ran.len(), 11);
    assert_eq!(machine.ran[0], Program::ListingTheDisks);
    assert_eq!(machine.ran[1], Program::WaitingForTheNetwork);
    assert!(matches!(machine.ran[2], Program::Verifying(_)));
    let Program::Writing(writing) = &machine.ran[3] else {
        panic!("the fourth program is the write: {:?}", machine.ran);
    };
    assert_eq!(writing.disk(), &disk);
    assert_eq!(
        writing.arguments().last().map(String::as_str),
        Some("/dev/disk/by-id/virtio-alo-target")
    );
    // Step 6, in order, and then the restart.
    assert_eq!(machine.ran[4], Program::ListingTheStartEntries);
    assert_eq!(
        machine.ran[5],
        Program::NamingTheEntry {
            disk: std::path::PathBuf::from("/dev/disk/by-id/virtio-alo-target"),
            partition: 2,
        }
    );
    assert_eq!(
        machine.ran[6],
        Program::RemovingTheEntry {
            number: "000B".to_owned(),
        }
    );
    assert_eq!(
        machine.ran[7],
        Program::OrderingTheEntries {
            order: ["000C", "0004", "000B", "0000"].map(str::to_owned).to_vec(),
        }
    );
    assert_eq!(machine.ran[8], Program::ListingTheDisks);
    assert_eq!(
        machine.ran[9],
        Program::RemovingTheArea {
            disk: "/dev/vda".to_owned(),
            partition: 5,
        }
    );
    assert_eq!(machine.ran[10], Program::Restarting);
    assert_eq!(
        machine.noted,
        Vec::<String>::new(),
        "an install that succeeded notes nothing"
    );
}

/// **A release whose signature does not verify writes nothing and says so** —
/// in the words the installer plan gives, and with no way past it.
#[test]
fn a_release_whose_signature_does_not_verify_writes_nothing_and_says_so() {
    let mut machine = Scripted::choosing("virtio-alo-target")
        .answering(succeeded(THE_DISKS))
        .answering(failed(
            "Error: no matching signatures: error verifying bundle: failed to verify signature",
        ));

    let ended = install(&mut machine, &strings(), Ok(&the_environment()));

    refused_with(
        &machine,
        &ended,
        &Refusal::NotGenuine,
        "This download is not a genuine alo OS, so nothing was changed",
    );
    assert!(
        !machine
            .said
            .iter()
            .any(|line| line == word("installing.genuine").says()),
        "a download that is not genuine was never called genuine"
    );
    assert_eq!(
        machine.noted,
        [
            "/usr/bin/cosign: Error: no matching signatures: error verifying bundle: failed to verify \
          signature"
        ],
        "why the check failed is kept where a technician reads it"
    );
    the_machinery_is_never_on_the_screen(&machine);
}

/// **No line a program complained of is ever a sentence on the screen.**
fn the_machinery_is_never_on_the_screen(machine: &Scripted) {
    for noted in &machine.noted {
        let (_, complaint) = noted
            .split_once(": ")
            .unwrap_or_else(|| panic!("a note names its program: {noted}"));
        assert!(
            !machine.said.iter().any(|said| said.contains(complaint)),
            "the screen said the machinery's words: {complaint}"
        );
    }
}

/// **A check that succeeds about some other release is not a pass**, and
/// writes nothing.
#[test]
fn a_verification_of_some_other_release_writes_nothing() {
    let other = format!("sha256:{}", "e".repeat(64));
    let mut machine = Scripted::choosing("virtio-alo-target")
        .answering(succeeded(THE_DISKS))
        .answering(succeeded(&the_owners_verification(&other)));

    let ended = install(&mut machine, &strings(), Ok(&the_environment()));

    refused_with(
        &machine,
        &ended,
        &Refusal::NotGenuine,
        word("installing.not-genuine").says(),
    );
}

/// **A release that cannot be reached writes nothing**, and is said as the
/// network rather than as a download that is not genuine.
#[test]
fn a_release_that_cannot_be_reached_writes_nothing_and_says_why() {
    let mut machine = Scripted::choosing("virtio-alo-target")
        .answering(succeeded(THE_DISKS))
        .answering(failed(
            "Error: Get \"https://ghcr.io/v2/\": dial tcp: lookup ghcr.io on [::1]:53: connection refused",
        ));

    let ended = install(&mut machine, &strings(), Ok(&the_environment()));

    refused_with(
        &machine,
        &ended,
        &Refusal::NotReachable,
        word("installing.not-reachable").says(),
    );
}

/// **No disk chosen: nothing is listed, checked or written.** The environment
/// never picks a disk by itself.
#[test]
fn with_no_disk_chosen_nothing_is_looked_at_or_written() {
    for line in [
        "BOOT_IMAGE=/EFI/alo-installing/vmlinuz rd.systemd.unit=alo-installing.target",
        "BOOT_IMAGE=/EFI/alo-installing/vmlinuz alo.installing.to=",
    ] {
        let mut machine = Scripted::choosing("virtio-alo-target");
        machine.command_line = Ok(line.to_owned());

        let ended = install(&mut machine, &strings(), Ok(&the_environment()));

        refused_with(
            &machine,
            &ended,
            &Refusal::NoDiskChosen,
            word("installing.no-disk-chosen").says(),
        );
        assert!(machine.ran.is_empty(), "{line}: {:?}", machine.ran);
    }
}

/// **A choice that is not one disk is not guessed at**: two disks, a path, or a
/// command line that could not be read.
#[test]
fn a_choice_that_is_not_one_disk_is_not_guessed_at() {
    for line in [
        Ok("alo.installing.to=virtio-alo-target alo.installing.to=nvme-Data_Disk_123".to_owned()),
        Ok("alo.installing.to=/dev/vdb".to_owned()),
        Ok("alo.installing.to=../../vdb".to_owned()),
        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
    ] {
        let mut machine = Scripted::choosing("virtio-alo-target");
        machine.command_line = line;

        let ended = install(&mut machine, &strings(), Ok(&the_environment()));

        refused_with(
            &machine,
            &ended,
            &Refusal::ChoiceNotUnderstood,
            word("installing.choice-not-understood").says(),
        );
        assert!(machine.ran.is_empty(), "{:?}", machine.ran);
    }
}

/// **Part of a disk is refused as part of a disk.**
#[test]
fn part_of_a_disk_is_refused() {
    let mut machine = Scripted::choosing("virtio-alo-target-part1");

    let ended = install(&mut machine, &strings(), Ok(&the_environment()));

    refused_with(
        &machine,
        &ended,
        &Refusal::NotAWholeDisk("virtio-alo-target-part1".to_owned()),
        &english(
            word("installing.not-a-whole-disk"),
            "virtio-alo-target-part1",
        ),
    );
    assert!(machine.ran.is_empty());
}

/// **A disk that never appears is refused**, and nothing else is looked at in
/// its place.
#[test]
fn a_disk_that_never_appears_is_refused() {
    let mut machine = Scripted::choosing("usb-Somebodys_Stick_0001");

    let ended = install(&mut machine, &strings(), Ok(&the_environment()));

    refused_with(
        &machine,
        &ended,
        &Refusal::DiskNotConnected(
            alo_installing::DiskName::named("usb-Somebodys_Stick_0001").expect("a disk"),
        ),
        &english(
            word("installing.disk-not-connected"),
            "usb-Somebodys_Stick_0001",
        ),
    );
    assert!(machine.ran.is_empty(), "{:?}", machine.ran);
}

/// **The disk this installer runs from is refused before anything is checked
/// over the network**, and so is a disk holding another operating system.
#[test]
fn the_installers_own_disk_and_another_systems_disk_are_refused_before_the_network() {
    for (chosen, refusal, sentence) in [
        (
            "virtio-alo-windows",
            Refusal::HoldsThisInstaller(
                alo_installing::DiskName::named("virtio-alo-windows").expect("a disk"),
            ),
            "installing.holds-this-installer",
        ),
        (
            "nvme-Data_Disk_123",
            Refusal::HoldsAnotherSystem(
                alo_installing::DiskName::named("nvme-Data_Disk_123").expect("a disk"),
            ),
            "installing.holds-another-system",
        ),
    ] {
        let mut machine = Scripted::choosing(chosen).answering(succeeded(THE_DISKS));

        let ended = install(&mut machine, &strings(), Ok(&the_environment()));

        refused_with(&machine, &ended, &refusal, &english(word(sentence), chosen));
        assert_eq!(machine.ran, [Program::ListingTheDisks], "{chosen}");
    }
}

/// **Disks that cannot be read are not assumed to be empty.**
#[test]
fn disks_that_cannot_be_read_are_not_assumed_empty() {
    for answer in [
        failed("lsblk: failed to access sysfs directory"),
        succeeded("not json"),
        Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
    ] {
        let mut machine = Scripted::choosing("virtio-alo-target").answering(answer);

        let ended = install(&mut machine, &strings(), Ok(&the_environment()));

        refused_with(
            &machine,
            &ended,
            &Refusal::DisksNotRead,
            word("installing.disks-not-read").says(),
        );
        assert_eq!(machine.ran, [Program::ListingTheDisks]);
    }
}

/// **A damaged environment refuses before it reads anything**, and a checker
/// that is not there is damage rather than a download that is not genuine.
#[test]
fn a_damaged_environment_refuses_before_reading_anything() {
    let mut machine = Scripted::choosing("virtio-alo-target");
    let ended = install(&mut machine, &strings(), Err(Refusal::Damaged));
    refused_with(
        &machine,
        &ended,
        &Refusal::Damaged,
        word("installing.damaged").says(),
    );
    assert!(machine.ran.is_empty());
    assert_eq!(
        machine.said.len(),
        3,
        "the first line, the refusal, and the last line"
    );

    let mut machine = Scripted::choosing("virtio-alo-target")
        .answering(succeeded(THE_DISKS))
        .answering(Err(std::io::Error::from(std::io::ErrorKind::NotFound)));
    let ended = install(&mut machine, &strings(), Ok(&the_environment()));
    refused_with(
        &machine,
        &ended,
        &Refusal::Damaged,
        word("installing.damaged").says(),
    );
}

/// **A write that does not finish is said as exactly what it is** — the chosen
/// disk may hold part of alo OS and nothing else changed — and the machine does
/// not restart into it.
#[test]
fn a_write_that_does_not_finish_is_said_plainly_and_never_restarts() {
    let environment = the_environment();
    for (answer, noted) in [
        (
            failed(
                "Copying blob sha256:0a1b\n\nerror: Installing to disk: Creating ostree deployment: \
                 No space left on device\n",
            ),
            vec![
                "/usr/bin/bootc: Copying blob sha256:0a1b",
                "/usr/bin/bootc: error: Installing to disk: Creating ostree deployment: No space \
                 left on device",
            ],
        ),
        (failed(""), vec!["/usr/bin/bootc: failed, and said nothing"]),
        (
            Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
            vec!["/usr/bin/bootc: entity not found"],
        ),
    ] {
        let mut machine = Scripted::choosing("virtio-alo-target")
            .answering(succeeded(THE_DISKS))
            .answering(succeeded(&the_owners_verification(
                environment.pin().digest(),
            )))
            .answering(answer);

        let ended = install(&mut machine, &strings(), Ok(&environment));

        let disk = alo_installing::DiskName::named("virtio-alo-target").expect("a disk");
        assert_eq!(ended, Ended::NotInstalled(disk));
        assert!(ended.wrote());
        assert!(!machine.restarted());
        let last_two = &machine.said[machine.said.len() - 2..];
        assert_eq!(
            last_two[0],
            english(word("installing.not-installed"), "virtio-alo-target")
        );
        assert!(!last_two[0].contains("nothing was changed"));
        assert_eq!(last_two[1], word("installing.restart-when-ready").says());
        assert_eq!(machine.noted, noted, "why the write stopped is kept");
        the_machinery_is_never_on_the_screen(&machine);
    }
}

/// **Every word this environment says is in the machine's one vocabulary**, so
/// a translation of any of them is checked and loaded like every other.
#[test]
fn every_word_is_in_the_vocabulary_alo_saying_collects() {
    let everything = alo_saying::everything_this_machine_can_say().expect("the vocabulary");
    let strings = Strings::of(everything);
    for word in alo_installing::EVERY_WORD {
        let said = strings.say(&word.key(), &alo_strings::Filling::of("disk", "virtio-a"));
        assert!(!said.is_a_bug(), "{} is not collected", word.named());
    }
}

/// **With no wired connection nothing is checked or written**, and the person
/// is told to look at the cable rather than that the download is not genuine.
#[test]
fn with_no_connection_nothing_is_checked_or_written() {
    let mut machine = Scripted::choosing("virtio-alo-target").answering(succeeded(THE_DISKS));
    machine.connects = false;

    let ended = install(&mut machine, &strings(), Ok(&the_environment()));

    refused_with(
        &machine,
        &ended,
        &Refusal::NotReachable,
        word("installing.not-reachable").says(),
    );
    assert_eq!(
        machine.ran,
        [Program::ListingTheDisks, Program::WaitingForTheNetwork]
    );
}

// ---------------------------------------------------------------------------
// What the install leaves behind.
// ---------------------------------------------------------------------------

/// A machine that has just installed, with these answers for the tidying.
fn tidying_with(answers: Vec<std::io::Result<alo_installing::Ran>>) -> Scripted {
    let mut machine = Scripted::choosing("virtio-alo-target");
    for answer in answers {
        machine = machine.answering(answer);
    }
    machine
}

/// The disk alo OS was installed onto, by its own name.
fn the_installed_disk() -> alo_installing::DiskName {
    alo_installing::DiskName::named("virtio-alo-target").expect("a disk")
}

/// **An entry already named alo OS is left exactly as it is**, and the order
/// and the area are still put right.
#[test]
fn an_entry_that_already_carries_the_name_is_not_made_again() {
    let already = THE_START_ENTRIES.replace("Fedora", "alo OS");
    let mut machine = tidying_with(vec![
        succeeded(&already),
        succeeded(""),
        succeeded(THE_DISKS),
        succeeded(""),
    ]);
    let tidied = alo_installing::tidy_up(&mut machine, &strings(), &the_installed_disk());

    assert!(tidied.whole(), "{tidied:?}");
    assert_eq!(machine.ran.len(), 4, "{:?}", machine.ran);
    assert!(
        !machine
            .ran
            .iter()
            .any(|program| matches!(program, Program::NamingTheEntry { .. })),
        "{:?}",
        machine.ran
    );
    assert_eq!(
        machine.ran[1],
        Program::OrderingTheEntries {
            order: ["000B", "0004", "0000"].map(str::to_owned).to_vec(),
        }
    );
}

/// **A machine with no staging area has nothing to remove**, and says the
/// tidying is whole.
#[test]
fn a_machine_without_an_area_is_tidied_whole() {
    let without = THE_DISKS.replace("\"ALO-INSTALL\"", "null");
    let mut machine = tidying_with(vec![
        succeeded(&THE_START_ENTRIES.replace("Fedora", "alo OS")),
        succeeded(""),
        succeeded(&without),
    ]);
    let tidied = alo_installing::tidy_up(&mut machine, &strings(), &the_installed_disk());

    assert!(tidied.whole(), "{tidied:?}");
    assert!(
        !machine
            .ran
            .iter()
            .any(|program| matches!(program, Program::RemovingTheArea { .. })),
        "a machine with no area had one removed: {:?}",
        machine.ran
    );
}

/// **A tidying that could not finish says alo OS is installed first**, notes
/// why where a technician reads it, and leaves the machine installed.
#[test]
fn what_could_not_be_tidied_is_said_without_taking_the_install_back() {
    let mut machine = tidying_with(vec![
        succeeded(&THE_START_ENTRIES.replace("Fedora", "alo OS")),
        failed("the firmware refused the order"),
        succeeded(THE_DISKS),
        failed("the partition would not go"),
    ]);
    let tidied = alo_installing::tidy_up(&mut machine, &strings(), &the_installed_disk());

    assert!(!tidied.whole());
    assert!(tidied.named);
    assert!(!tidied.ordered);
    assert!(!tidied.area_removed);
    assert_eq!(
        machine.said.last().map(String::as_str),
        Some(english(word("installing.tidy-not-whole"), "").as_str())
    );
    assert_eq!(machine.noted.len(), 2, "{:?}", machine.noted);
}

/// **A firmware that will not say what it can start is left alone.**
#[test]
fn a_firmware_that_says_nothing_is_not_guessed_at() {
    let mut machine = tidying_with(vec![failed("no efi variables")]);
    let tidied = alo_installing::tidy_up(&mut machine, &strings(), &the_installed_disk());

    assert!(!tidied.whole());
    assert_eq!(machine.ran.len(), 1);
    assert_eq!(
        machine.said.last().map(String::as_str),
        Some(english(word("installing.tidy-entries-not-read"), "").as_str())
    );
}
