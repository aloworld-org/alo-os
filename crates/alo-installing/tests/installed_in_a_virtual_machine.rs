//! The boot environment, built from its recipe and booted in a virtual machine.
//!
//! The installer plan's task 2, measured rather than believed: a UEFI machine
//! with **Secure Boot on** under the Microsoft certificates a laptop ships with,
//! a first disk laid out the way Windows lays one out with the environment
//! staged beside it, and an empty second disk. The environment is started the
//! way a firmware starts it — the signed shim, the signed loader, the signed
//! kernel — and then:
//!
//! - [`the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`]:
//!   it says every step, pulls the pinned release from the registry, checks the
//!   owner's signature, writes the second disk and restarts; the first disk
//!   hashes the same before and after; and the second disk, started on its own,
//!   comes up with `alo-agentd` running;
//! - [`the_environment_installs_beside_windows_on_its_own_disk_and_every_file_stays`]:
//!   the road for a computer with one disk. The same environment, staged with
//!   the choice the installer on Windows writes, installs into the space that
//!   installer made beside an NTFS Windows volume; Windows' partitions hash the
//!   same, the person's files and Windows' loader read back byte for byte, and
//!   the computer started again comes up in alo OS with `alo-agentd` running;
//! - [`a_release_signed_by_another_key_writes_nothing_and_says_so`]: the same
//!   environment holding a key that is not the owner's refuses, says so on the
//!   console in the vocabulary's words, and neither disk has a byte changed;
//! - [`a_loader_the_firmware_does_not_trust_is_refused`]: the machine the first
//!   two are measured in really enforces Secure Boot — a staged loader with one
//!   byte changed never starts.
//!
//! # What it needs, and what it costs
//!
//! Linux with `qemu-system-x86_64`, the firmware without Secure Boot
//! (`/usr/share/OVMF`, for the refusal, which starts a kernel directly),
//! `podman` (root) and `sfdisk`, and the network: the recipe is built, the
//! Secure Boot firmware is taken out of the pinned base, and the release is
//! pulled from `ghcr.io`. The install measured about twelve minutes
//! on 2026-09-15 with hardware virtualisation; the whole test, with a cached
//! build, about twenty. The road beside Windows also installs `ntfsprogs` into
//! a container of the base, to make Windows' volume and read its files back.
//!
//! **Where it runs:** `.github/workflows/install-check.yml`, started by hand,
//! on a hosted runner with KVM. Nothing runs it on a pull request or a merge.
//!
//! **Hardware virtualisation is used where it works, and asked rather than
//! assumed** ([`Processor`]): a host can show `/dev/kvm` with nothing behind it
//! (`docs/quirks.md`, *WSL on a VMware guest shows `/dev/kvm` and has no KVM
//! behind it*). Without it the machines run emulated, every deadline here is
//! [`EMULATION_IS_SLOWER`] times as long, and nothing measured is a timing. It is
//! `#[ignore]`d so the workspace suite never starts a virtual machine, and is
//! run by name. A machine missing any of the above **fails** with the list of
//! what it is missing — a test that passed because it could not run would be
//! the worst kind of evidence.
//!
//! # Where the machine is not a laptop
//!
//! The plan names a Hyper-V generation-2 machine. This test uses QEMU's q35 with
//! OVMF, which is the same shape — UEFI, GPT, Secure Boot with Microsoft's
//! certificates — driven from the Linux the gates run in; the report for this
//! task says why, and `docs/booting.md` gives the Hyper-V steps for a person.
//!
//! **The sign-in is stood in for.** The image ships no accounts (ADR 0024), and
//! `alo-agentd.service` runs inside the person's session: it is wanted by
//! `user@1000.service`, which a sign-in starts. The second boot hands systemd two
//! credentials through the firmware tables — a unit that prints what is running
//! to the serial line and powers off, and a drop-in that starts
//! `user@1000.service` the way a sign-in would. Nothing on the installed disk is
//! changed to do it.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs::{File, OpenOptions};
use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use alo_installing::EVERY_WORD;

/// One virtual machine at a time: the two tests share a build and a directory.
static ONE_MACHINE: Mutex<()> = Mutex::new(());

/// The disk the person chose, as the machine names it.
const THE_CHOSEN_DISK: &str = "virtio-alo-target";

/// The firmware with Secure Boot enforced, as the base names the file.
///
/// **Taken out of the same pinned base the environment is built from**, rather
/// than from whatever the host happens to package. The base's own firmware is
/// the one built for the base's own boot chain: Ubuntu's `ovmf`
/// 2025.11-3ubuntu7 dies with a page fault starting the base's signed loader,
/// and Fedora's `edk2-ovmf` starts it (`docs/quirks.md`, *EDK II's strict image
/// protection page-faults the base's signed loader*). Secure Boot is enforced in
/// both, and is never switched off (ADR 0033 §4) — which
/// [`a_loader_the_firmware_does_not_trust_is_refused`] measures rather than
/// assumes.
const SECURE_FIRMWARE: &str = "OVMF_CODE.secboot.fd";

/// Its variables, with Microsoft's certificates enrolled — the ones a laptop
/// ships with.
const MICROSOFT_VARIABLES: &str = "OVMF_VARS.secboot.fd";

/// The firmware without Secure Boot, for starting a kernel directly.
const PLAIN_FIRMWARE: &str = "/usr/share/OVMF/OVMF_CODE_4M.fd";

/// Its variables.
const PLAIN_VARIABLES: &str = "/usr/share/OVMF/OVMF_VARS_4M.fd";

/// A firmware, together with the only machine it is started in.
///
/// **One value, so the two cannot be mismatched.** The firmware without Secure
/// Boot is not built for System Management Mode; started on a machine whose
/// variable flash only SMM code may write, it cannot keep its variables there and
/// saves them instead as a file, `NvVars`, on the first FAT it finds — which here
/// is the staged installer's partition. That write is what made the refusal
/// change the first disk on 2026-09-15 (`docs/quirks.md`, *OVMF without SMM
/// saves its variables onto a FAT disk*), and it is the machine's, never alo OS's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Firmware {
    /// Secure Boot enforced, under Microsoft's certificates, built for SMM.
    SecureBoot,
    /// Without Secure Boot, and without SMM, for starting a kernel directly.
    Plain,
}

impl Firmware {
    /// The firmware's code.
    fn code(self) -> PathBuf {
        match self {
            Self::SecureBoot => the_firmware_directory().join(SECURE_FIRMWARE),
            Self::Plain => PathBuf::from(PLAIN_FIRMWARE),
        }
    }

    /// The variables it starts from.
    fn variables(self) -> PathBuf {
        match self {
            Self::SecureBoot => the_firmware_directory().join(MICROSOFT_VARIABLES),
            Self::Plain => PathBuf::from(PLAIN_VARIABLES),
        }
    }

    /// The machine it is started in: SMM, with flash only SMM may write, for
    /// the firmware built for it, and an ordinary q35 for the one that is not.
    fn machine(self) -> &'static [&'static str] {
        match self {
            Self::SecureBoot => &[
                "-machine",
                "q35,smm=on",
                "-global",
                "driver=cfi.pflash01,property=secure,value=on",
            ],
            Self::Plain => &["-machine", "q35"],
        }
    }
}

/// How large the second disk is. The release is about ten gigabytes installed.
const THE_SECOND_DISK: u64 = 24 * 1024 * 1024 * 1024;

/// One mebibyte.
const MIB: u64 = 1024 * 1024;

/// The first disk, as Windows lays one out, with the environment staged last.
///
/// Sizes in mebibytes, GPT type, and name.
const WINDOWS_LAYOUT: [(u64, &str, &str); 5] = [
    (
        100,
        "C12A7328-F81F-11D2-BA4B-00A0C93EC93B",
        "EFI system partition",
    ),
    (
        16,
        "E3C9E316-0B5C-4DB8-817D-F92DF00215AE",
        "Microsoft reserved partition",
    ),
    (
        1024,
        "EBD0A0A2-B9E5-4433-87C0-68B6B72699C7",
        "Basic data partition",
    ),
    (
        512,
        "DE94BBA4-06D1-4D40-A16A-BFD50179D6AC",
        "Windows recovery",
    ),
    (
        600,
        "C12A7328-F81F-11D2-BA4B-00A0C93EC93B",
        "alo OS installer",
    ),
];

/// This repository.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// Where this test keeps its disks and its build.
fn work() -> PathBuf {
    // Cargo's own directory for integration tests, inside the build directory
    // and so on a disk: `/tmp` may be a small tmpfs in memory, and in WSL it
    // is — 3.9 GB, shared, filled to 100% by these disks on 2026-09-15.
    let at = Path::new(env!("CARGO_TARGET_TMPDIR")).join("alo-installing-vm");
    std::fs::create_dir_all(&at).expect("the work directory can be made");
    at
}

/// **What a test made, removed when the test ends — pass or fail.**
///
/// The installer plan's rule since 2026-09-15, when these disks filled the
/// development PC's drive: a virtual-machine test removes its disks when it
/// finishes, keeping only the serial logs its report names. A guard rather
/// than a last line, because a test that fails ends at its `panic!`, and the
/// unwinding that follows still drops what the test holds.
struct Leftovers {
    /// The directory the names are in.
    at: PathBuf,
    /// Files and directories, removed whichever each turns out to be.
    names: Vec<&'static str>,
}

impl Leftovers {
    /// These names, in the work directory.
    fn in_the_work_directory(names: &[&'static str]) -> Self {
        Self::in_directory(work(), names)
    }

    /// These names, in `at`.
    fn in_directory(at: PathBuf, names: &[&'static str]) -> Self {
        Self {
            at,
            names: names.to_vec(),
        }
    }
}

impl Drop for Leftovers {
    fn drop(&mut self) {
        for name in &self.names {
            let path = self.at.join(name);
            // Either may not exist: the test may have ended before making it.
            drop(std::fs::remove_dir_all(&path));
            drop(std::fs::remove_file(&path));
        }
    }
}

/// The English of a word by its key, with the chosen disk in its gap.
fn english(named: &str) -> String {
    EVERY_WORD
        .iter()
        .find(|word| word.named() == named)
        .unwrap_or_else(|| panic!("{named} is declared"))
        .says()
        .replace("{disk}", THE_CHOSEN_DISK)
}

/// Run a program to its end, and fail the test with what it said if it failed.
fn ran(program: &str, arguments: &[&str]) -> String {
    let output = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|why| panic!("{program} could not be started: {why}"));
    assert!(
        output.status.success(),
        "{program} {arguments:?} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Everything this test needs that the machine does not have.
fn what_is_missing() -> Vec<String> {
    let mut missing = Vec::new();
    for program in ["podman", "qemu-system-x86_64", "sfdisk"] {
        let found = Command::new("which")
            .arg(program)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        if !found {
            missing.push(program.to_owned());
        }
    }
    // The Secure Boot firmware is not among these: it is taken out of the
    // pinned base by `the_firmware_is_taken_from_the_base`, not packaged by the
    // host.
    // Not `/dev/kvm`: it can be there with nothing behind it, and a machine
    // without it runs emulated (`Processor`).
    for file in [PLAIN_FIRMWARE, PLAIN_VARIABLES] {
        if !Path::new(file).exists() {
            missing.push(file.to_owned());
        }
    }
    missing
}

/// Fail, naming everything missing, rather than pass without running.
fn the_machine_can_run_this() {
    let missing = what_is_missing();
    assert!(
        missing.is_empty(),
        "this test boots a virtual machine and cannot on this host; missing: {}",
        missing.join(", ")
    );
}

/// The environment, built from its recipe, as the directory of files the recipe
/// produces.
fn the_environment_built() -> PathBuf {
    let built = work().join("environment");
    drop(std::fs::remove_dir_all(&built));
    let repository = the_repository();
    let output = format!("type=local,dest={}", built.display());
    ran(
        "podman",
        &[
            "build",
            "--file",
            &repository
                .join("image/installing/Containerfile")
                .display()
                .to_string(),
            "--output",
            &output,
            &repository.display().to_string(),
        ],
    );
    for file in [
        "EFI/BOOT/BOOTX64.EFI",
        "EFI/BOOT/grubx64.efi",
        "EFI/BOOT/grub.cfg",
        "EFI/alo-installing/vmlinuz",
        "EFI/alo-installing/initramfs.img",
    ] {
        assert!(
            built.join(file).is_file(),
            "the build did not produce {file}"
        );
    }
    built
}

/// The base the recipe builds on, as it names it.
fn the_base() -> String {
    let recipe = std::fs::read_to_string(the_repository().join("image/installing/Containerfile"))
        .expect("the recipe is there");
    recipe
        .lines()
        .find_map(|line| line.trim().strip_prefix("ARG THE_BASE="))
        .expect("the recipe names its base")
        .trim()
        .to_owned()
}

/// Where the firmware taken out of the base is kept.
fn the_firmware_directory() -> PathBuf {
    work().join("firmware")
}

/// **The Secure Boot firmware, taken out of the base the environment is built
/// from**, so that the machine these tests use enforces Secure Boot with the
/// firmware built for the boot chain the base ships.
///
/// Done once: the files are kept in the work directory between runs.
fn the_firmware_is_taken_from_the_base() {
    let at = the_firmware_directory();
    if at.join(SECURE_FIRMWARE).is_file() && at.join(MICROSOFT_VARIABLES).is_file() {
        return;
    }
    std::fs::create_dir_all(&at).expect("the firmware directory can be made");
    let carrier = "alo-installing-vm-firmware";
    drop(
        Command::new("podman")
            .args(["rm", "--force", carrier])
            .output(),
    );
    ran(
        "podman",
        &[
            "run",
            "--detach",
            "--name",
            carrier,
            "--volume",
            &format!("{}:/firmware", at.display()),
            &the_base(),
            "sleep",
            "infinity",
        ],
    );
    ran(
        "podman",
        &[
            "exec",
            carrier,
            "dnf",
            "install",
            "--assumeyes",
            "edk2-ovmf",
        ],
    );
    for file in [SECURE_FIRMWARE, MICROSOFT_VARIABLES] {
        ran(
            "podman",
            &[
                "exec",
                carrier,
                "cp",
                "--dereference",
                &format!("/usr/share/edk2/ovmf/{file}"),
                &format!("/firmware/{file}"),
            ],
        );
    }
    ran("podman", &["rm", "--force", carrier]);
    for file in [SECURE_FIRMWARE, MICROSOFT_VARIABLES] {
        assert!(at.join(file).is_file(), "the base did not hand over {file}");
    }
}

/// A FAT partition holding the environment and the person's choice, as the
/// program that stages it would write it.
fn staged(environment: &Path) -> PathBuf {
    staged_choosing(
        environment,
        &format!("set alo_installing_to={THE_CHOSEN_DISK}\n"),
    )
}

/// The same partition, holding this choice: the file the installer on Windows
/// writes beside the loader's configuration, word for word.
fn staged_choosing(environment: &Path, choice: &str) -> PathBuf {
    std::fs::write(environment.join("EFI/BOOT/chosen.cfg"), choice)
        .expect("the choice can be written");

    let work = work();
    let copied = inside_the_stager(&work, environment);
    let partition = work.join("staged.img");
    drop(std::fs::remove_file(&partition));
    let volume = format!("{}:/work", work.display());
    let stager = "alo-installing-vm-stager";
    drop(
        Command::new("podman")
            .args(["rm", "--force", stager])
            .output(),
    );
    ran(
        "podman",
        &[
            "run",
            "--detach",
            "--privileged",
            "--name",
            stager,
            "--volume",
            &volume,
            &the_base(),
            "sleep",
            "infinity",
        ],
    );
    let kib = ((WINDOWS_LAYOUT[4].0 - 1) * 1024).to_string();
    for step in [
        vec![
            "mkfs.fat",
            "-C",
            "-F",
            "32",
            "-n",
            alo_installing::THIS_INSTALLER,
            "/work/staged.img",
            &kib,
        ],
        // Under `/tmp`, not `/mnt`: in the bootc base `/mnt` is a link to
        // `var/mnt`, which a container of it does not have.
        vec!["mkdir", "-p", "/tmp/staged"],
        vec!["mount", "-o", "loop", "/work/staged.img", "/tmp/staged"],
        vec!["cp", "-r", &copied, "/tmp/staged/"],
        vec!["umount", "/tmp/staged"],
    ] {
        let mut arguments = vec!["exec", stager];
        arguments.extend(step);
        ran("podman", &arguments);
    }
    ran("podman", &["rm", "--force", stager]);
    partition
}

/// **The environment the stager copies is the one it was handed**, as the
/// stager's container sees it: the work directory is mounted at `/work`.
///
/// Until 2026-09-16 the copy named `/work/environment` whatever it was handed,
/// so the loader with one byte changed was never staged — the genuine loader
/// was, the firmware rightly started it, and the test of Secure Boot's refusal
/// could never have passed. A directory outside the work directory is not in
/// the container at all, and is refused rather than silently swapped for
/// another.
fn inside_the_stager(work: &Path, environment: &Path) -> String {
    let within = environment.strip_prefix(work).unwrap_or_else(|_| {
        panic!(
            "{} is not in the work directory {}, which is all the stager can see",
            environment.display(),
            work.display()
        )
    });
    assert!(
        within
            .components()
            .all(|part| matches!(part, std::path::Component::Normal(_))),
        "{} is not a plain directory inside the work directory",
        environment.display()
    );
    assert!(
        within.components().next().is_some(),
        "the work directory itself is not an environment"
    );
    format!("/work/{}/.", within.display())
}

/// A byte generator nobody can mistake for a file system, the same every run.
struct Pattern(u64);

impl Pattern {
    /// Fill a buffer.
    fn fill(&mut self, buffer: &mut [u8]) {
        for chunk in buffer.chunks_mut(8) {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            let bytes = self.0.to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
    }
}

/// The first disk: Windows' partitions filled with known bytes, and the staged
/// environment in the last one.
fn the_windows_disk(staged: &Path) -> PathBuf {
    let disk = work().join("windows.raw");
    drop(std::fs::remove_file(&disk));
    let total: u64 = WINDOWS_LAYOUT.iter().map(|(size, _, _)| size).sum::<u64>() + 4;
    File::create(&disk)
        .and_then(|file| file.set_len(total * MIB))
        .expect("the first disk can be made");

    let mut layout = String::from("label: gpt\nfirst-lba: 2048\n");
    for (size, kind, name) in WINDOWS_LAYOUT {
        layout.push_str(&format!("size={size}MiB, type={kind}, name=\"{name}\"\n"));
    }
    let mut sfdisk = Command::new("sfdisk")
        .arg(&disk)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .expect("sfdisk starts");
    sfdisk
        .stdin
        .take()
        .expect("sfdisk reads its layout")
        .write_all(layout.as_bytes())
        .expect("the layout is written");
    assert!(
        sfdisk.wait().expect("sfdisk ends").success(),
        "sfdisk refused the layout"
    );

    let table: serde_json::Value =
        serde_json::from_str(&ran("sfdisk", &["--json", &disk.display().to_string()]))
            .expect("sfdisk prints its table");
    let starts: Vec<u64> = table["partitiontable"]["partitions"]
        .as_array()
        .expect("the table has partitions")
        .iter()
        .map(|partition| partition["start"].as_u64().expect("a start") * 512)
        .collect();
    assert_eq!(starts.len(), WINDOWS_LAYOUT.len());

    let mut file = OpenOptions::new()
        .write(true)
        .open(&disk)
        .expect("the disk opens");
    let mut pattern = Pattern(0x0a10_05a1_0ae5_0001);
    let mut buffer = vec![0_u8; MIB as usize];
    for (index, (size, _, _)) in WINDOWS_LAYOUT.iter().enumerate().take(4) {
        file.seek(SeekFrom::Start(starts[index])).expect("a seek");
        for _ in 0..*size {
            pattern.fill(&mut buffer);
            file.write_all(&buffer).expect("a write");
        }
    }
    file.seek(SeekFrom::Start(starts[4])).expect("a seek");
    let mut from = File::open(staged).expect("the staged partition opens");
    std::io::copy(&mut from, &mut file).expect("the staged partition is written");
    file.sync_all().expect("the disk is synced");
    disk
}

/// An empty second disk.
fn an_empty_second_disk() -> PathBuf {
    let disk = work().join("target.raw");
    drop(std::fs::remove_file(&disk));
    File::create(&disk)
        .and_then(|file| file.set_len(THE_SECOND_DISK))
        .expect("the second disk can be made");
    disk
}

/// The SHA-256 of a whole file.
fn hashed(file: &Path) -> Vec<u8> {
    let mut context = ring::digest::Context::new(&ring::digest::SHA256);
    let mut reading = File::open(file).expect("the file opens");
    let mut buffer = vec![0_u8; 4 * MIB as usize];
    loop {
        let read = reading.read(&mut buffer).expect("a read");
        if read == 0 {
            break;
        }
        context.update(&buffer[..read]);
    }
    context.finish().as_ref().to_vec()
}

/// The first disk as it was: its hash, and a copy to say where it changed.
struct AsItWas {
    /// The SHA-256 of the whole disk.
    hash: Vec<u8>,
    /// A byte-for-byte copy.
    copy: PathBuf,
}

/// Hash the first disk and keep a copy of it.
fn as_it_was(disk: &Path) -> AsItWas {
    let copy = work().join("windows.before");
    std::fs::copy(disk, &copy).expect("the first disk can be copied");
    AsItWas {
        hash: hashed(disk),
        copy,
    }
}

/// **The first disk hashes the same as it did** — and when it does not, the
/// failure names every mebibyte that changed and the partition it is in, so a
/// write to a disk nobody chose is a place to look rather than two hashes.
fn the_first_disk_is_unchanged(disk: &Path, was: &AsItWas, when: &str) {
    if hashed(disk) == was.hash {
        return;
    }
    let starts: Vec<(u64, &str)> = {
        let mut at = 1;
        WINDOWS_LAYOUT
            .iter()
            .map(|(size, _, name)| {
                let start = at;
                at += size;
                (start, *name)
            })
            .collect()
    };
    let mut now = File::open(disk).expect("the first disk opens");
    let mut then = File::open(&was.copy).expect("its copy opens");
    let (mut a, mut b) = (vec![0_u8; MIB as usize], vec![0_u8; MIB as usize]);
    let mut changed = Vec::new();
    for mebibyte in 0.. {
        let read = now.read(&mut a).expect("a read");
        let kept = then.read(&mut b).expect("a read");
        if read == 0 && kept == 0 {
            break;
        }
        if a[..read] != b[..kept] {
            let within = starts
                .iter()
                .rev()
                .find(|(start, _)| mebibyte >= *start)
                .map_or("the partition table", |(_, name)| *name);
            changed.push(format!("MiB {mebibyte} ({within})"));
        }
    }
    panic!(
        "the first disk changed {when}, at {} mebibyte(s): {}",
        changed.len(),
        changed
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );
}

/// Firmware variables of this test's own, copied from the firmware's.
fn variables(firmware: Firmware, called: &str) -> PathBuf {
    let at = work().join(called);
    std::fs::copy(firmware.variables(), &at).expect("the firmware variables can be copied");
    at
}

/// How much longer everything takes when the processor is emulated.
///
/// Measured on 2026-09-15 by the update tests' lane: a Fedora machine that
/// starts in seconds with hardware virtualisation took 190 seconds to its login
/// prompt emulated. Eight times is that ratio, with room for a busy host.
const EMULATION_IS_SLOWER: u32 = 8;

/// How the machines' processor is provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Processor {
    /// Hardware virtualisation, which the host has been seen to start.
    Hardware,
    /// Emulated in software, because the host could not start the other.
    Emulated,
}

impl Processor {
    /// This host's, found by **starting** a machine with hardware
    /// virtualisation and seeing whether it stays up — never by looking for
    /// `/dev/kvm`, which can be there with nothing behind it.
    fn of_this_host() -> Self {
        static FOUND: std::sync::OnceLock<Processor> = std::sync::OnceLock::new();
        *FOUND.get_or_init(|| {
            let Ok(mut probe) = Command::new("qemu-system-x86_64")
                .args([
                    "-accel",
                    "kvm",
                    "-machine",
                    "none",
                    "-nodefaults",
                    "-display",
                    "none",
                    "-monitor",
                    "none",
                    "-S",
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            else {
                return Self::Emulated;
            };
            // A host without it refuses at once; one with it sits paused, as
            // `-S` asks, until it is stopped.
            let began = Instant::now();
            while began.elapsed() < Duration::from_secs(5) {
                if probe.try_wait().ok().flatten().is_some() {
                    return Self::Emulated;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            drop(probe.kill());
            drop(probe.wait());
            Self::Hardware
        })
    }

    /// The arguments that say so.
    fn arguments(self) -> [&'static str; 4] {
        match self {
            Self::Hardware => ["-accel", "kvm", "-cpu", "host"],
            Self::Emulated => ["-accel", "tcg", "-cpu", "max"],
        }
    }

    /// A deadline measured with hardware virtualisation, as long as it is on
    /// this processor.
    fn allowing(self, measured: Duration) -> Duration {
        match self {
            Self::Hardware => measured,
            Self::Emulated => measured * EMULATION_IS_SLOWER,
        }
    }
}

/// The arguments every machine here starts with.
fn a_machine(
    processor: Processor,
    firmware: Firmware,
    variables: &Path,
    serial: &Path,
) -> Vec<String> {
    processor
        .arguments()
        .iter()
        .chain(&["-smp", "4", "-m", "3072"])
        .chain(firmware.machine())
        .chain(&[
            "-nic",
            "user,model=virtio-net-pci",
            "-display",
            "none",
            "-no-reboot",
        ])
        .map(|argument| (*argument).to_owned())
        .chain([
            "-drive".to_owned(),
            format!(
                "if=pflash,format=raw,unit=0,readonly=on,file={}",
                firmware.code().display()
            ),
            "-drive".to_owned(),
            format!("if=pflash,format=raw,unit=1,file={}", variables.display()),
            "-serial".to_owned(),
            format!("file:{}", serial.display()),
        ])
        .collect()
}

/// A disk attached with a serial number, so the machine names it by its own
/// identity, and an optional place in the firmware's order.
fn a_disk(file: &Path, serial: &str, boot_order: Option<u32>) -> Vec<String> {
    let order = boot_order
        .map(|at| format!(",bootindex={at}"))
        .unwrap_or_default();
    vec![
        "-drive".to_owned(),
        format!("if=none,id={serial},format=raw,file={}", file.display()),
        "-device".to_owned(),
        format!("virtio-blk-pci,drive={serial},serial={serial}{order}"),
    ]
}

/// Start a machine.
fn started(arguments: &[String]) -> Child {
    Command::new("qemu-system-x86_64")
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the virtual machine starts")
}

/// What the serial line has said so far.
fn said(serial: &Path) -> String {
    std::fs::read(serial)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

/// Whether what the serial line has said so far is a machine that will wait for
/// a person rather than power itself off — `waiting`, said on a line of its own.
///
/// An environment that could not install says its refusal, then `waiting`, and
/// stays on until somebody turns it off. On 2026-09-16 the install test did not
/// know that and waited out its whole deadline — 71 minutes after the failure
/// was already on the serial line, and it would have been hours more under
/// emulation. A sentence quoted inside another line is not the environment
/// saying it.
fn waits_for_a_person(told: &str, waiting: &str) -> bool {
    told.lines().any(|line| line.trim() == waiting)
}

/// What the installed machine said about why its services are as they are:
/// their status and their journal, between the markers the watching unit prints.
///
/// On 2026-09-16 the first installed disk to boot under Secure Boot reported
/// `alo-agentd.service` as `failed` and nothing else, because the machine's own
/// console is not the serial line; the failure says why from now on.
fn why_it_said(told: &str) -> String {
    let Some((_, after)) = told.split_once("ALO-WHY-BEGIN") else {
        return "nothing: the watching unit never reached its account of why".to_owned();
    };
    after
        .split_once("ALO-WHY-END")
        .map_or(after, |(why, _)| why)
        .replace('\r', "")
        .trim()
        .to_owned()
}

/// Wait for a machine to power itself off, or kill it and fail — at once, when
/// it has said `waiting` and so never will ([`waits_for_a_person`]).
fn powered_off(mut machine: Child, within: Duration, serial: &Path, waiting: Option<&str>) {
    let began = Instant::now();
    loop {
        if let Some(status) = machine.try_wait().expect("the machine can be asked") {
            assert!(status.success(), "the machine ended with {status}");
            return;
        }
        if let Some(waiting) = waiting {
            let told = said(serial);
            if waits_for_a_person(&told, waiting) {
                drop(machine.kill());
                drop(machine.wait());
                panic!(
                    "the machine said it could not finish and is waiting for a person, after \
                     {:?}; its serial line ends:\n{}",
                    began.elapsed(),
                    &told[told.len().saturating_sub(4000)..]
                );
            }
        }
        if began.elapsed() > within {
            drop(machine.kill());
            drop(machine.wait());
            let told = said(serial);
            panic!(
                "the machine did not finish within {within:?}; its serial line ends:\n{}",
                &told[told.len().saturating_sub(4000)..]
            );
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

/// Every sentence, in order, somewhere in what the serial line said.
fn said_in_order(serial: &str, sentences: &[String]) {
    let mut from = 0;
    for sentence in sentences {
        let Some(at) = serial[from..].find(sentence.as_str()) else {
            panic!(
                "the console never said, after what came before it: {sentence}\n\nit said:\n{}",
                &serial[serial.len().saturating_sub(6000)..]
            );
        };
        from += at + sentence.len();
    }
}

/// Standard base64, for the credentials and the key.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for (i, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[((n >> shift) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// A systemd credential handed over in the firmware's tables.
fn a_credential(named: &str, content: &str) -> Vec<String> {
    vec![
        "-smbios".to_owned(),
        format!(
            "type=11,value=io.systemd.credential.binary:{named}={}",
            base64(content.as_bytes())
        ),
    ]
}

/// **The environment, started by a firmware with Secure Boot on, installs the
/// pinned release onto the second disk and says every step; the first disk
/// hashes the same afterwards; and the second disk boots to `alo-agentd`
/// running.**
#[test]
#[ignore = "boots a virtual machine and pulls the release from the registry; run by name"]
fn the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service() {
    let _one = ONE_MACHINE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _made = Leftovers::in_the_work_directory(&[
        "environment",
        "staged.img",
        "windows.raw",
        "windows.before",
        "target.raw",
        "installing-vars.fd",
        "installed-vars.fd",
    ]);
    the_machine_can_run_this();
    let processor = Processor::of_this_host();
    the_firmware_is_taken_from_the_base();

    let environment = the_environment_built();
    let windows = the_windows_disk(&staged(&environment));
    let target = an_empty_second_disk();
    let before = as_it_was(&windows);

    // The install, started the way a firmware starts it.
    let serial = work().join("installing.log");
    drop(std::fs::remove_file(&serial));
    let vars = variables(Firmware::SecureBoot, "installing-vars.fd");
    let mut arguments = a_machine(processor, Firmware::SecureBoot, &vars, &serial);
    arguments.extend(a_disk(&windows, "alo-windows", Some(1)));
    arguments.extend(a_disk(&target, "alo-target", None));
    powered_off(
        started(&arguments),
        processor.allowing(Duration::from_secs(60 * 60)),
        &serial,
        Some(&english("installing.restart-when-ready")),
    );

    the_install_said_every_step(&said(&serial));
    the_first_disk_is_unchanged(&windows, &before, "during the install");
    assert!(
        std::fs::metadata(&target)
            .expect("the second disk")
            .blocks()
            > 0,
        "nothing was written to the second disk"
    );

    // The installed disk, started on its own, with the sign-in stood in for.
    let serial = work().join("installed.log");
    drop(std::fs::remove_file(&serial));
    let vars = variables(Firmware::SecureBoot, "installed-vars.fd");
    let mut arguments = a_machine(processor, Firmware::SecureBoot, &vars, &serial);
    arguments.extend(a_disk(&target, "alo-target", Some(1)));
    arguments.extend(a_disk(&windows, "alo-windows", None));
    arguments.extend(the_sign_in_stood_in_for());
    powered_off(
        started(&arguments),
        processor.allowing(Duration::from_secs(20 * 60)),
        &serial,
        None,
    );

    the_services_are_running(&said(&serial));
    the_first_disk_is_unchanged(&windows, &before, "when alo OS booted");
}

/// **The environment said every step of an install that finished, in order,
/// with Secure Boot on, and no refusal.**
fn the_install_said_every_step(told: &str) {
    assert!(
        told.contains("Secure boot enabled"),
        "the environment was not started with Secure Boot on"
    );
    said_in_order(
        told,
        &[
            english("installing.starting"),
            english("installing.reading-the-choice"),
            english("installing.looking-for-the-disk"),
            english("installing.checking-the-disk"),
            english("installing.connecting"),
            english("installing.checking-it-is-genuine"),
            english("installing.genuine"),
            english("installing.installing"),
            english("installing.installed"),
        ],
    );
    for refusal in [
        "installing.not-genuine",
        "installing.not-reachable",
        "installing.not-installed",
        "installing.restart-when-ready",
    ] {
        assert!(
            !told.contains(&english(refusal)),
            "the console said {refusal}"
        );
    }
}

/// The two credentials that stand in for a sign-in on an installed machine: a
/// unit that says on the serial line what is running and why, then powers the
/// machine off, and a drop-in that starts the person's session as a sign-in
/// would. Nothing on the installed disk is changed to do it.
fn the_sign_in_stood_in_for() -> Vec<String> {
    let mut arguments = a_credential(
        "systemd.extra-unit.alo-vm-watching.service",
        "[Unit]\n\
         Description=What is running, said on the serial line for the installer's test\n\
         After=multi-user.target user@1000.service alo-agentd.service\n\
         [Service]\n\
         Type=oneshot\n\
         ExecStartPre=/usr/bin/sleep 30\n\
         ExecStart=/usr/bin/systemctl show --property=Id,ActiveState,SubState alo-boundaryd.service alo-agentd.service\n\
         ExecStart=-/usr/bin/echo\n\
         ExecStart=-/usr/bin/echo ALO-WHY-BEGIN\n\
         ExecStart=-/usr/bin/systemctl status --no-pager --full --lines=0 alo-boundaryd.service alo-agentd.service user@1000.service\n\
         ExecStart=-/usr/bin/journalctl --boot --no-pager --output=short-monotonic --lines=80 --unit=alo-boundaryd.service --unit=alo-agentd.service --unit=user@1000.service\n\
         ExecStart=-/usr/bin/echo ALO-WHY-END\n\
         ExecStartPost=/usr/bin/systemctl --no-block poweroff\n\
         StandardOutput=file:/dev/ttyS0\n\
         StandardError=file:/dev/ttyS0\n",
    );
    arguments.extend(a_credential(
        "systemd.unit-dropin.multi-user.target~alo-vm-watching",
        "[Unit]\nWants=user@1000.service alo-vm-watching.service\n",
    ));
    arguments
}

/// **The installed machine, started on its own, came up with `alo-boundaryd`
/// active and `alo-agentd` running**, as its watching unit said on the serial
/// line — and when not, the failure carries the machine's own account of why.
fn the_services_are_running(told: &str) {
    for unit in ["alo-boundaryd.service", "alo-agentd.service"] {
        let block = told
            .split("\n\n")
            .map(|block| block.replace('\r', ""))
            .find(|block| block.lines().any(|line| line == format!("Id={unit}")))
            .unwrap_or_else(|| {
                panic!(
                    "the installed machine never said how {unit} is; its serial line ends:\n{}",
                    &told[told.len().saturating_sub(4000)..]
                )
            });
        assert!(
            block.lines().any(|line| line == "ActiveState=active")
                && (unit != "alo-agentd.service"
                    || block.lines().any(|line| line == "SubState=running")),
            "{unit} is not running on the installed machine:\n{block}\n\nwhy, as the machine \
             said it:\n{}",
            why_it_said(told)
        );
    }
}

/// The disk alo OS is installed beside Windows on, as the machine names it.
const BESIDE_DISK: &str = "virtio-alo-windows";

/// The GPT type of a start-up area, the EFI system partition.
const START_UP_AREA_TYPE: &str = "C12A7328-F81F-11D2-BA4B-00A0C93EC93B";

/// The GPT type of a basic data partition: Windows' own volume, and both of
/// the partitions the installer on Windows makes, which `New-Partition` gives
/// this type.
const BASIC_DATA_TYPE: &str = "EBD0A0A2-B9E5-4433-87C0-68B6B72699C7";

/// **The one disk of a computer that keeps Windows**, as the installer on
/// Windows leaves it before the restart: Windows' four partitions, and the
/// installer's two in the space a shrink of Windows freed.
///
/// Where each starts and how big it is, in mebibytes, its GPT type, and its
/// name, **in the order the partition table numbers them**. The installer's two
/// are numbered last, as `New-Partition` numbers them, but sit where the shrink
/// freed space: straight after Windows' own volume and before its recovery
/// partition, which is where they are on a real disk. So a write that strayed
/// past the space alo OS was given lands in Windows' recovery partition, and is
/// seen there.
const BESIDE_WINDOWS: [(u64, u64, &str, &str); 6] = [
    (1, 100, START_UP_AREA_TYPE, "EFI system partition"),
    (
        101,
        16,
        "E3C9E316-0B5C-4DB8-817D-F92DF00215AE",
        "Microsoft reserved partition",
    ),
    (117, 1024, BASIC_DATA_TYPE, "Basic data partition"),
    (
        26_317,
        512,
        "DE94BBA4-06D1-4D40-A16A-BFD50179D6AC",
        "Windows recovery",
    ),
    (1_141, 600, BASIC_DATA_TYPE, "Basic data partition"),
    (1_741, 24_576, BASIC_DATA_TYPE, "Basic data partition"),
];

/// Where in [`BESIDE_WINDOWS`] the start-up area is: partition 1.
const THE_START_UP_AREA_AT: usize = 0;

/// Where Windows' own volume is: partition 3, NTFS, holding the person's files.
const THE_WINDOWS_VOLUME_AT: usize = 2;

/// Where the installer's staging area is: partition 5.
const THE_AREA_AT: usize = 4;

/// Where the space the installer made for alo OS is: partition 6.
const THE_SPACE_AT: usize = 5;

/// **Windows' own partitions, which nothing on this road may write**: its
/// reserved partition, its volume and its recovery partition. The start-up area
/// is not among them, because alo OS's loader is put into it beside Windows'
/// own; what is kept there is Windows' files ([`START_UP_FILES`]).
const WINDOWS_OWN: [usize; 3] = [1, 2, 3];

/// The person's files on Windows' volume, at its root: each name, its size in
/// bytes, and the seed of its bytes. A short one the volume keeps inside its own
/// file table, and larger ones it keeps in runs of clusters across the volume.
const WINDOWS_FILES: [(&str, u64, u64); 4] = [
    ("A letter to the bank.txt", 612, 1),
    ("Holiday photograph.jpg", 3 * MIB, 2),
    ("Thesis.docx", 40 * MIB, 3),
    ("pagefile.sys", 64 * MIB, 4),
];

/// Windows' own loader and its boot configuration, on the start-up area alo
/// OS's loader goes beside: each path, its size in bytes, and the seed of its
/// bytes.
const START_UP_FILES: [(&str, u64, u64); 2] = [
    ("EFI/Microsoft/Boot/bootmgfw.efi", MIB, 11),
    ("EFI/Microsoft/Boot/BCD", 32 * 1024, 12),
];

/// The bytes of a file this test puts on Windows' disk, the same every run.
fn known_bytes(size: u64, seed: u64) -> Vec<u8> {
    let mut bytes =
        vec![0_u8; usize::try_from(size).expect("a file this test makes fits in memory")];
    Pattern(0x0a10_05a1_0be5_1de0 ^ seed.wrapping_mul(0x9e37_79b9_7f4a_7c15)).fill(&mut bytes);
    bytes
}

/// **A container of the pinned base, with the work directory at `/work`**,
/// removed when this is dropped — pass or fail, so a step that failed does not
/// leave it running.
struct InTheBase(&'static str);

impl InTheBase {
    /// Start one, by this name.
    fn started(named: &'static str) -> Self {
        drop(
            Command::new("podman")
                .args(["rm", "--force", named])
                .output(),
        );
        ran(
            "podman",
            &[
                "run",
                "--detach",
                "--privileged",
                "--name",
                named,
                "--volume",
                &format!("{}:/work", work().display()),
                &the_base(),
                "sleep",
                "infinity",
            ],
        );
        Self(named)
    }

    /// Run one program in it, to its end, failing the test if it failed.
    fn run(&self, step: &[&str]) -> String {
        let mut arguments = vec!["exec", self.0];
        arguments.extend_from_slice(step);
        ran("podman", &arguments)
    }
}

impl Drop for InTheBase {
    fn drop(&mut self) {
        drop(
            Command::new("podman")
                .args(["rm", "--force", self.0])
                .output(),
        );
    }
}

/// The files [`WINDOWS_FILES`] and [`START_UP_FILES`] name, written into the
/// work directory for the base to copy onto the disk.
fn the_files_to_keep() {
    let files = work().join("beside-files");
    drop(std::fs::remove_dir_all(&files));
    let every = WINDOWS_FILES
        .iter()
        .map(|(name, size, seed)| (files.join("windows").join(name), *size, *seed))
        .chain(
            START_UP_FILES
                .iter()
                .map(|(path, size, seed)| (files.join("start-up").join(path), *size, *seed)),
        );
    for (at, size, seed) in every {
        std::fs::create_dir_all(at.parent().expect("a file is in a directory"))
            .expect("a directory can be made");
        std::fs::write(&at, known_bytes(size, seed)).expect("a file can be written");
    }
}

/// A file's bytes, written into the disk from this mebibyte on — leaving every
/// mebibyte that is all zeros as the hole a new disk already is, so a file
/// system of 24 gigabytes that is mostly empty costs what it holds.
fn written_into(disk: &mut File, from: &Path, at_mebibyte: u64) {
    let mut reading = File::open(from).expect("the partition's file opens");
    let mut buffer = vec![0_u8; MIB as usize];
    let mut offset = at_mebibyte * MIB;
    loop {
        let read = reading.read(&mut buffer).expect("a read");
        if read == 0 {
            break;
        }
        if buffer[..read].iter().any(|byte| *byte != 0) {
            disk.seek(SeekFrom::Start(offset)).expect("a seek");
            disk.write_all(&buffer[..read]).expect("a write");
        }
        offset += u64::try_from(read).expect("a read fits");
    }
}

/// **The disk beside Windows**, laid out as [`BESIDE_WINDOWS`] says and filled
/// as a computer keeping Windows has it: a FAT start-up area holding Windows'
/// loader, a reserved partition and a recovery partition of known bytes, an NTFS
/// volume holding the person's files, the staged environment, and the space for
/// alo OS — formatted FAT32 and labelled as the installer labels it.
///
/// The file systems are made by the base's own tools (`mkfs.fat`, and
/// `ntfsprogs`' `mkntfs` and `ntfscp`), in files of their own, and then written
/// into the disk at their partitions' places.
fn the_disk_beside_windows(staged: &Path) -> PathBuf {
    the_files_to_keep();
    let work = work();
    for name in [
        "beside-start-up.img",
        "beside-windows.img",
        "beside-space.img",
    ] {
        drop(std::fs::remove_file(work.join(name)));
    }
    File::create(work.join("beside-windows.img"))
        .and_then(|file| file.set_len(BESIDE_WINDOWS[THE_WINDOWS_VOLUME_AT].1 * MIB))
        .expect("Windows' volume can be made");

    let kib = |index: usize| (BESIDE_WINDOWS[index].1 * 1024).to_string();
    let base = InTheBase::started("alo-installing-vm-beside");
    base.run(&["dnf", "install", "--assumeyes", "ntfsprogs"]);
    base.run(&[
        "mkfs.fat",
        "-C",
        "-F",
        "32",
        "-n",
        "SYSTEM",
        "/work/beside-start-up.img",
        &kib(THE_START_UP_AREA_AT),
    ]);
    base.run(&["mkdir", "-p", "/tmp/start-up"]);
    base.run(&[
        "mount",
        "-o",
        "loop",
        "/work/beside-start-up.img",
        "/tmp/start-up",
    ]);
    base.run(&[
        "cp",
        "-r",
        "/work/beside-files/start-up/.",
        "/tmp/start-up/",
    ]);
    base.run(&["umount", "/tmp/start-up"]);
    base.run(&[
        "mkntfs",
        "--force",
        "--quick",
        "--label",
        "Windows",
        "--partition-start",
        &(BESIDE_WINDOWS[THE_WINDOWS_VOLUME_AT].0 * MIB / 512).to_string(),
        "--sectors-per-track",
        "63",
        "--heads",
        "255",
        "/work/beside-windows.img",
    ]);
    for (name, _, _) in WINDOWS_FILES {
        base.run(&[
            "ntfscp",
            "--force",
            "/work/beside-windows.img",
            &format!("/work/beside-files/windows/{name}"),
            &format!("/{name}"),
        ]);
    }
    base.run(&[
        "mkfs.fat",
        "-C",
        "-F",
        "32",
        "-n",
        alo_installing::THE_SPACE_THE_INSTALLER_MADE,
        "/work/beside-space.img",
        &kib(THE_SPACE_AT),
    ]);
    drop(base);

    let disk = work.join("beside.raw");
    drop(std::fs::remove_file(&disk));
    let end = BESIDE_WINDOWS
        .iter()
        .map(|(start, size, _, _)| start + size)
        .max()
        .expect("the layout has partitions");
    File::create(&disk)
        .and_then(|file| file.set_len((end + 1) * MIB))
        .expect("the disk can be made");
    let mut layout = String::from("label: gpt\nfirst-lba: 2048\n");
    for (start, size, kind, name) in BESIDE_WINDOWS {
        layout.push_str(&format!(
            "start={}, size={size}MiB, type={kind}, name=\"{name}\"\n",
            start * MIB / 512
        ));
    }
    let mut sfdisk = Command::new("sfdisk")
        .arg(&disk)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .expect("sfdisk starts");
    sfdisk
        .stdin
        .take()
        .expect("sfdisk reads its layout")
        .write_all(layout.as_bytes())
        .expect("the layout is written");
    assert!(
        sfdisk.wait().expect("sfdisk ends").success(),
        "sfdisk refused the layout:\n{layout}"
    );

    let mut file = OpenOptions::new()
        .write(true)
        .open(&disk)
        .expect("the disk opens");
    let start = |index: usize| BESIDE_WINDOWS[index].0;
    written_into(
        &mut file,
        &work.join("beside-start-up.img"),
        start(THE_START_UP_AREA_AT),
    );
    written_into(
        &mut file,
        &work.join("beside-windows.img"),
        start(THE_WINDOWS_VOLUME_AT),
    );
    written_into(&mut file, staged, start(THE_AREA_AT));
    written_into(
        &mut file,
        &work.join("beside-space.img"),
        start(THE_SPACE_AT),
    );
    let mut pattern = Pattern(0x0a10_05a1_0be5_0002);
    let mut buffer = vec![0_u8; MIB as usize];
    for index in [1, 3] {
        let (at, size, _, _) = BESIDE_WINDOWS[index];
        file.seek(SeekFrom::Start(at * MIB)).expect("a seek");
        for _ in 0..size {
            pattern.fill(&mut buffer);
            file.write_all(&buffer).expect("a write");
        }
    }
    file.sync_all().expect("the disk is synced");
    disk
}

/// The SHA-256 of one partition of the disk beside Windows.
fn hashed_partition(disk: &Path, index: usize) -> Vec<u8> {
    let (at, size, _, _) = BESIDE_WINDOWS[index];
    let mut reading = File::open(disk).expect("the disk opens");
    reading.seek(SeekFrom::Start(at * MIB)).expect("a seek");
    let mut context = ring::digest::Context::new(&ring::digest::SHA256);
    let mut buffer = vec![0_u8; MIB as usize];
    for _ in 0..size {
        reading.read_exact(&mut buffer).expect("a read");
        context.update(&buffer);
    }
    context.finish().as_ref().to_vec()
}

/// One partition of the disk, copied into a file of its own, so that reading
/// its files never mounts the disk itself.
fn copied_out(disk: &Path, index: usize, into: &Path) {
    let (at, size, _, _) = BESIDE_WINDOWS[index];
    let mut reading = File::open(disk).expect("the disk opens");
    reading.seek(SeekFrom::Start(at * MIB)).expect("a seek");
    let mut writing = File::create(into).expect("the copy can be made");
    std::io::copy(&mut reading.take(size * MIB), &mut writing).expect("the partition is copied");
}

/// What Windows has on the disk: the hash of each of its own partitions, and
/// how the partition table describes each of its four.
struct WindowsAsItWas {
    /// The SHA-256 of each partition [`WINDOWS_OWN`] names, in that order.
    hashes: Vec<Vec<u8>>,
    /// The table's entries for Windows' four partitions: start, size, type,
    /// identity and name.
    entries: Vec<serde_json::Value>,
}

/// What Windows has on the disk now.
fn windows_as_it_is(disk: &Path) -> WindowsAsItWas {
    let table: serde_json::Value =
        serde_json::from_str(&ran("sfdisk", &["--json", &disk.display().to_string()]))
            .expect("sfdisk prints its table");
    let windows_starts: Vec<u64> = BESIDE_WINDOWS[..4]
        .iter()
        .map(|(start, _, _, _)| start * MIB / 512)
        .collect();
    let entries: Vec<serde_json::Value> = table["partitiontable"]["partitions"]
        .as_array()
        .expect("the table has partitions")
        .iter()
        .filter(|entry| {
            entry["start"]
                .as_u64()
                .is_some_and(|start| windows_starts.contains(&start))
        })
        .map(|entry| {
            let mut entry = entry.clone();
            // The file's own path, which the table prints as each partition's
            // node, is not a fact about the partition.
            if let Some(fields) = entry.as_object_mut() {
                fields.remove("node");
            }
            entry
        })
        .collect();
    WindowsAsItWas {
        hashes: WINDOWS_OWN
            .iter()
            .map(|index| hashed_partition(disk, *index))
            .collect(),
        entries,
    }
}

/// **Windows' own partitions are byte for byte what they were, and the table
/// describes its four exactly as it did** — and when not, the failure names the
/// partition.
fn windows_is_unchanged(disk: &Path, was: &WindowsAsItWas, when: &str) {
    let now = windows_as_it_is(disk);
    for ((index, then), now) in WINDOWS_OWN.iter().zip(&was.hashes).zip(&now.hashes) {
        assert!(
            then == now,
            "Windows' {} (partition {}) changed {when}",
            BESIDE_WINDOWS[*index].3,
            index + 1
        );
    }
    assert_eq!(
        was.entries.len(),
        4,
        "the table did not list Windows' four partitions to begin with: {:?}",
        was.entries
    );
    assert_eq!(
        was.entries, now.entries,
        "the partition table describes Windows' partitions differently {when}"
    );
}

/// **Every file the person had is there, byte for byte** — on Windows' volume,
/// read with `ntfsprogs`' own reader, and Windows' loader on the start-up area —
/// each read from a copy of its partition, never from the disk itself.
fn every_file_is_there(disk: &Path, when: &str) {
    let work = work();
    let read = work.join("beside-read");
    drop(std::fs::remove_dir_all(&read));
    std::fs::create_dir_all(read.join("windows")).expect("a directory can be made");
    copied_out(
        disk,
        THE_START_UP_AREA_AT,
        &work.join("beside-start-up-read.img"),
    );
    copied_out(
        disk,
        THE_WINDOWS_VOLUME_AT,
        &work.join("beside-windows-read.img"),
    );

    let base = InTheBase::started("alo-installing-vm-reading");
    base.run(&["dnf", "install", "--assumeyes", "ntfsprogs"]);
    base.run(&["mkdir", "-p", "/tmp/start-up"]);
    base.run(&[
        "mount",
        "-o",
        "loop,ro",
        "/work/beside-start-up-read.img",
        "/tmp/start-up",
    ]);
    for (path, _, _) in START_UP_FILES {
        base.run(&[
            "install",
            "-D",
            &format!("/tmp/start-up/{path}"),
            &format!("/work/beside-read/start-up/{path}"),
        ]);
    }
    base.run(&["umount", "/tmp/start-up"]);
    for (name, _, _) in WINDOWS_FILES {
        base.run(&[
            "sh",
            "-c",
            &format!(
                "ntfscat /work/beside-windows-read.img '/{name}' > '/work/beside-read/windows/{name}'"
            ),
        ]);
    }
    drop(base);

    let every = WINDOWS_FILES
        .iter()
        .map(|(name, size, seed)| (read.join("windows").join(name), *size, *seed))
        .chain(
            START_UP_FILES
                .iter()
                .map(|(path, size, seed)| (read.join("start-up").join(path), *size, *seed)),
        );
    for (at, size, seed) in every {
        let found = std::fs::read(&at)
            .unwrap_or_else(|why| panic!("{} could not be read {when}: {why}", at.display()));
        assert!(
            found == known_bytes(size, seed),
            "{} is not byte for byte what was put there, {when}: {} bytes read, {size} put there",
            at.display(),
            found.len()
        );
    }
}

/// **alo OS installed beside Windows on Windows' own disk, the road a computer
/// with one disk takes, and nothing of Windows' lost**: the environment, started
/// with Secure Boot on, says every step and installs into the space the
/// installer made; Windows' reserved, volume and recovery partitions are byte for
/// byte what they were, and so is the table's account of them; the person's
/// files on Windows' NTFS volume and Windows' own loader read back byte for
/// byte; and the computer, started again with the firmware's variables the
/// install left, comes up in alo OS with `alo-agentd` running.
///
/// **The choice is staged by the installer's own writer**
/// (`alo_installer::the_choice_beside_what_is_there`), so what the loader reads
/// is what Windows writes rather than a line typed into this test, and what
/// reaches the environment is whatever the loader really hands its kernel.
#[test]
#[ignore = "boots a virtual machine and pulls the release from the registry; run by name"]
fn the_environment_installs_beside_windows_on_its_own_disk_and_every_file_stays() {
    let _one = ONE_MACHINE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _made = Leftovers::in_the_work_directory(&[
        "environment",
        "staged.img",
        "beside.raw",
        "beside-files",
        "beside-start-up.img",
        "beside-windows.img",
        "beside-space.img",
        "beside-read",
        "beside-start-up-read.img",
        "beside-windows-read.img",
        "beside-vars.fd",
    ]);
    the_machine_can_run_this();
    let processor = Processor::of_this_host();
    the_firmware_is_taken_from_the_base();

    let environment = the_environment_built();
    let partition = |index: usize| {
        alo_installing::PartitionName::named(&format!("{BESIDE_DISK}-part{}", index + 1))
            .expect("a partition's name")
    };
    let choice = alo_installer::the_choice_beside_what_is_there(
        &alo_installing::DiskName::named(BESIDE_DISK).expect("a disk's name"),
        &partition(THE_SPACE_AT),
        &partition(THE_START_UP_AREA_AT),
    );
    let disk = the_disk_beside_windows(&staged_choosing(&environment, &choice));
    let before = windows_as_it_is(&disk);
    every_file_is_there(&disk, "before the install");

    // The install, started the way a firmware starts it. One set of firmware
    // variables for both starts, as one computer has: the entry the install
    // makes for alo OS is the one the second start finds.
    let serial = work().join("beside-installing.log");
    drop(std::fs::remove_file(&serial));
    let vars = variables(Firmware::SecureBoot, "beside-vars.fd");
    let restart_when_ready = english("installing.restart-when-ready");
    let mut arguments = a_machine(processor, Firmware::SecureBoot, &vars, &serial);
    arguments.extend(a_disk(&disk, "alo-windows", Some(1)));
    powered_off(
        started(&arguments),
        processor.allowing(Duration::from_secs(60 * 60)),
        &serial,
        Some(&restart_when_ready),
    );

    the_install_said_every_step(&said(&serial));
    windows_is_unchanged(&disk, &before, "during the install");
    every_file_is_there(&disk, "after the install");

    // The same computer started again, with the sign-in stood in for and no
    // place in the order given: the firmware starts what the install left.
    let serial = work().join("beside-installed.log");
    drop(std::fs::remove_file(&serial));
    let mut arguments = a_machine(processor, Firmware::SecureBoot, &vars, &serial);
    arguments.extend(a_disk(&disk, "alo-windows", None));
    arguments.extend(the_sign_in_stood_in_for());
    powered_off(
        started(&arguments),
        processor.allowing(Duration::from_secs(20 * 60)),
        &serial,
        Some(&restart_when_ready),
    );

    let told = said(&serial);
    assert!(
        !told.contains(&english("installing.starting")),
        "the computer started the installer again rather than alo OS; its serial line ends:\n{}",
        &told[told.len().saturating_sub(4000)..]
    );
    the_services_are_running(&told);
    windows_is_unchanged(&disk, &before, "when alo OS booted");
    every_file_is_there(&disk, "after alo OS booted");
}

/// **The machine the install is measured in really enforces Secure Boot**: a
/// staged loader with one byte changed is refused by the firmware, nothing of
/// ours runs, no kernel starts, and the first disk is unchanged.
///
/// Without this, the test above passing would be evidence of a firmware that
/// starts anything. Secure Boot is never switched off to make a test pass
/// (ADR 0033 §4), and this is how that is known rather than asserted.
#[test]
#[ignore = "boots a virtual machine; run by name"]
fn a_loader_the_firmware_does_not_trust_is_refused() {
    let _one = ONE_MACHINE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _made = Leftovers::in_the_work_directory(&[
        "environment",
        "environment-tampered",
        "staged.img",
        "windows.raw",
        "windows.before",
        "refused-loader-vars.fd",
    ]);
    the_machine_can_run_this();
    let processor = Processor::of_this_host();
    the_firmware_is_taken_from_the_base();

    let environment = the_environment_built();
    let tampered = work().join("environment-tampered");
    drop(std::fs::remove_dir_all(&tampered));
    ran(
        "cp",
        &[
            "--recursive",
            &environment.display().to_string(),
            &tampered.display().to_string(),
        ],
    );
    let loader = tampered.join("EFI/BOOT/BOOTX64.EFI");
    let mut bytes = std::fs::read(&loader).expect("the staged loader reads");
    let at = bytes.len() / 2;
    bytes[at] ^= 0xff;
    std::fs::write(&loader, &bytes).expect("the changed loader is written");

    let windows = the_windows_disk(&staged(&tampered));
    let before = as_it_was(&windows);

    let serial = work().join("refused-loader.log");
    drop(std::fs::remove_file(&serial));
    let vars = variables(Firmware::SecureBoot, "refused-loader-vars.fd");
    let mut arguments = a_machine(processor, Firmware::SecureBoot, &vars, &serial);
    arguments.extend(a_disk(&windows, "alo-windows", Some(1)));
    let mut machine = started(&arguments);

    // A refused loader leaves the firmware in its own boot menu, which nothing
    // here answers: the test waits for the refusal and then turns the machine
    // off.
    let began = Instant::now();
    while !said(&serial).contains("Access Denied") {
        if began.elapsed() > processor.allowing(Duration::from_secs(5 * 60))
            || machine
                .try_wait()
                .expect("the machine can be asked")
                .is_some()
        {
            drop(machine.kill());
            drop(machine.wait());
            let told = said(&serial);
            panic!(
                "the firmware never refused the changed loader; its serial line ends:\n{}",
                &told[told.len().saturating_sub(4000)..]
            );
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    drop(machine.kill());
    drop(machine.wait());

    let told = said(&serial);
    assert!(
        told.contains("rejected probably by Secure Boot"),
        "the firmware refused the loader for some other reason:\n{told}"
    );
    assert!(
        !told.contains("Linux version"),
        "a kernel started from a loader the firmware does not trust:\n{told}"
    );
    for never in [
        "installing.starting",
        "installing.genuine",
        "installing.installing",
    ] {
        assert!(
            !told.contains(&english(never)),
            "the console said {never}, so the environment ran"
        );
    }
    the_first_disk_is_unchanged(&windows, &before, "while the firmware refused the loader");
}

/// A P-256 public key that is not the owner's, as a PEM file.
fn somebody_elses_key() -> String {
    let random = ring::rand::SystemRandom::new();
    let pkcs8 = ring::signature::EcdsaKeyPair::generate_pkcs8(
        &ring::signature::ECDSA_P256_SHA256_ASN1_SIGNING,
        &random,
    )
    .expect("a key pair");
    let pair = ring::signature::EcdsaKeyPair::from_pkcs8(
        &ring::signature::ECDSA_P256_SHA256_ASN1_SIGNING,
        pkcs8.as_ref(),
        &random,
    )
    .expect("the key pair reads");
    use ring::signature::KeyPair as _;
    // SubjectPublicKeyInfo for an uncompressed P-256 point: the algorithm and
    // curve identifiers, then the point.
    let mut spki = vec![
        0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x08,
        0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00,
    ];
    spki.extend_from_slice(pair.public_key().as_ref());
    let encoded = base64(&spki);
    let lines: Vec<&str> = encoded
        .as_bytes()
        .chunks(64)
        .map(|line| std::str::from_utf8(line).expect("base64 is ASCII"))
        .collect();
    format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
        lines.join("\n")
    )
}

/// One file in a `newc` archive, which the kernel lays over the initramfs.
fn newc(archive: &mut Vec<u8>, name: &str, mode: u32, content: &[u8]) {
    let fields = [
        0,
        mode,
        0,
        0,
        1,
        0,
        u32::try_from(content.len()).expect("small"),
        0,
        0,
        0,
        0,
        u32::try_from(name.len() + 1).expect("small"),
        0,
    ];
    archive.extend_from_slice(b"070701");
    for field in fields {
        archive.extend_from_slice(format!("{field:08x}").as_bytes());
    }
    archive.extend_from_slice(name.as_bytes());
    archive.push(0);
    while !archive.len().is_multiple_of(4) {
        archive.push(0);
    }
    archive.extend_from_slice(content);
    while !archive.len().is_multiple_of(4) {
        archive.push(0);
    }
}

/// **The same environment, holding a key that is not the owner's, refuses the
/// release: it says so on the console in the vocabulary's words, it never
/// begins to write, and neither disk has a byte changed.**
#[test]
#[ignore = "boots a virtual machine and reaches the registry; run by name"]
fn a_release_signed_by_another_key_writes_nothing_and_says_so() {
    let _one = ONE_MACHINE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _made = Leftovers::in_the_work_directory(&[
        "environment",
        "staged.img",
        "windows.raw",
        "windows.before",
        "target.raw",
        "initramfs-with-another-key.img",
        "refusing-vars.fd",
    ]);
    the_machine_can_run_this();
    let processor = Processor::of_this_host();

    let environment = the_environment_built();
    let windows = the_windows_disk(&staged(&environment));
    let target = an_empty_second_disk();
    let before = as_it_was(&windows);

    // The environment's own initramfs, with somebody else's key laid over the
    // owner's — the one thing that differs from what the recipe built.
    let mut initramfs = std::fs::read(environment.join("EFI/alo-installing/initramfs.img"))
        .expect("the initramfs reads");
    while !initramfs.len().is_multiple_of(4) {
        initramfs.push(0);
    }
    let mut archive = Vec::new();
    for directory in [
        "usr",
        "usr/lib",
        "usr/lib/alo",
        "usr/lib/alo/installing",
        "usr/lib/alo/installing/signing",
    ] {
        newc(&mut archive, directory, 0o040_755, &[]);
    }
    newc(
        &mut archive,
        "usr/lib/alo/installing/signing/alo-os.pub",
        0o100_644,
        somebody_elses_key().as_bytes(),
    );
    newc(&mut archive, "TRAILER!!!", 0, &[]);
    initramfs.extend_from_slice(&archive);
    let changed = work().join("initramfs-with-another-key.img");
    std::fs::write(&changed, &initramfs).expect("the changed initramfs is written");

    // The kernel line the loader would give, with the person's choice in it.
    let entry =
        std::fs::read_to_string(environment.join("EFI/BOOT/grub.cfg")).expect("the entry reads");
    let line = entry
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("linux "))
        .expect("the entry starts a kernel")
        .split_whitespace()
        .skip(1)
        .collect::<Vec<_>>()
        .join(" ")
        .replace("${alo_installing_to}", THE_CHOSEN_DISK);

    let serial = work().join("refusing.log");
    drop(std::fs::remove_file(&serial));
    let vars = variables(Firmware::Plain, "refusing-vars.fd");
    let mut arguments = a_machine(processor, Firmware::Plain, &vars, &serial);
    arguments.extend([
        "-kernel".to_owned(),
        environment
            .join("EFI/alo-installing/vmlinuz")
            .display()
            .to_string(),
        "-initrd".to_owned(),
        changed.display().to_string(),
        "-append".to_owned(),
        line,
    ]);
    arguments.extend(a_disk(&windows, "alo-windows", None));
    arguments.extend(a_disk(&target, "alo-target", None));
    let mut machine = started(&arguments);

    // A refusal does not restart the machine, so the test waits for the last
    // line and then turns it off.
    let last = english("installing.restart-when-ready");
    let began = Instant::now();
    while !said(&serial).contains(&last) {
        if began.elapsed() > processor.allowing(Duration::from_secs(15 * 60))
            || machine
                .try_wait()
                .expect("the machine can be asked")
                .is_some()
        {
            drop(machine.kill());
            drop(machine.wait());
            let told = said(&serial);
            panic!(
                "the environment never finished refusing; its serial line ends:\n{}",
                &told[told.len().saturating_sub(4000)..]
            );
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    drop(machine.kill());
    drop(machine.wait());

    let told = said(&serial);
    said_in_order(
        &told,
        &[
            english("installing.starting"),
            english("installing.checking-it-is-genuine"),
            english("installing.not-genuine"),
            last,
        ],
    );
    assert!(
        told.contains("This download is not a genuine alo OS, so nothing was changed"),
        "the refusal is not said in the plan's words"
    );
    // Why the check refused is on the serial line in the checker's own words,
    // named as the checker's — kept where a technician reads it, not lost.
    assert!(
        told.lines()
            .any(|line| line.trim().starts_with("/usr/bin/cosign: ")),
        "the serial line does not say why the check refused:\n{}",
        &told[told.len().saturating_sub(4000)..]
    );
    for never in [
        "installing.genuine",
        "installing.installing",
        "installing.installed",
    ] {
        assert!(
            !told.contains(&format!("\n{}", english(never))),
            "the console said {never}"
        );
    }
    assert_eq!(
        std::fs::metadata(&target)
            .expect("the second disk")
            .blocks(),
        0,
        "the second disk was written"
    );
    // The firmware kept what it keeps in its own flash. Were it unable to, it
    // would save it onto the first FAT it found, and the check below would be
    // measuring the machine rather than alo OS.
    assert_ne!(
        hashed(&vars),
        hashed(&Firmware::Plain.variables()),
        "the firmware wrote nothing to its own variable flash, so it had nowhere to keep \
         its variables but a disk"
    );
    the_first_disk_is_unchanged(&windows, &before, "while the environment refused");
}

/// **Each firmware starts only in the machine it was built for**: SMM and flash
/// only SMM may write for the Secure Boot build, and neither for the build
/// without it — which, given that flash, saves its variables onto a disk.
#[test]
fn a_firmware_is_given_only_flash_it_can_write() {
    let secure = Firmware::SecureBoot.machine().join(" ");
    assert!(secure.contains("smm=on") && secure.contains("property=secure,value=on"));
    let plain = Firmware::Plain.machine().join(" ");
    assert!(!plain.contains("smm=on"), "{plain}");
    assert!(!plain.contains("secure"), "{plain}");
    for firmware in [Firmware::SecureBoot, Firmware::Plain] {
        let started = a_machine(
            Processor::Emulated,
            firmware,
            Path::new("vars.fd"),
            Path::new("serial.log"),
        );
        assert_eq!(
            started
                .iter()
                .filter(|argument| *argument == "-machine")
                .count(),
            1,
            "{started:?}"
        );
        assert!(
            started
                .iter()
                .any(|argument| argument.ends_with(&format!("file={}", firmware.code().display()))),
            "{started:?}"
        );
    }
}

/// **An emulated processor is said as one, and is given the time it needs**:
/// never hardware virtualisation's arguments, and every deadline longer.
#[test]
fn an_emulated_processor_is_never_asked_for_hardware_and_waits_longer() {
    let emulated = Processor::Emulated.arguments().join(" ");
    assert_eq!(emulated, "-accel tcg -cpu max");
    assert_eq!(
        Processor::Hardware.arguments().join(" "),
        "-accel kvm -cpu host"
    );
    let an_hour = Duration::from_secs(60 * 60);
    assert_eq!(Processor::Hardware.allowing(an_hour), an_hour);
    assert!(Processor::Emulated.allowing(an_hour) > an_hour);
    let started = a_machine(
        Processor::Emulated,
        Firmware::SecureBoot,
        Path::new("vars.fd"),
        Path::new("serial.log"),
    );
    assert!(
        !started.iter().any(|argument| argument == "kvm"),
        "{started:?}"
    );
}

/// **What a test made is gone when it ends, and gone when it fails**: files and
/// directories alike, a name never made is no error, and what the guard was not
/// given — a serial log a report names — is left where it is.
#[test]
fn what_a_test_made_is_removed_pass_or_fail() {
    let at = Path::new(env!("CARGO_TARGET_TMPDIR")).join("alo-installing-leftovers");
    drop(std::fs::remove_dir_all(&at));
    std::fs::create_dir_all(at.join("environment/EFI")).expect("a directory can be made");
    for file in ["windows.raw", "environment/EFI/grub.cfg", "installing.log"] {
        std::fs::write(at.join(file), b"made").expect("a file can be made");
    }

    drop(Leftovers::in_directory(
        at.clone(),
        &["windows.raw", "environment", "never-made.raw"],
    ));
    assert!(!at.join("windows.raw").exists());
    assert!(!at.join("environment").exists());
    assert!(at.join("installing.log").is_file(), "a log was removed");

    std::fs::write(at.join("target.raw"), b"made").expect("a file can be made");
    let failed = std::panic::catch_unwind(|| {
        let _made = Leftovers::in_directory(at.clone(), &["target.raw"]);
        panic!("the test failed");
    });
    assert!(failed.is_err());
    assert!(
        !at.join("target.raw").exists(),
        "a failed test left its disk behind"
    );
    drop(std::fs::remove_dir_all(&at));
}

/// **The stager copies the environment it was handed, and nothing else**: a
/// changed copy beside the built one is the changed copy inside the container,
/// and a directory the container cannot see is refused rather than swapped for
/// the one it can.
#[test]
fn the_stager_copies_the_environment_it_was_handed() {
    let work = Path::new("/build/tmp/alo-installing-vm");
    assert_eq!(
        inside_the_stager(work, &work.join("environment")),
        "/work/environment/."
    );
    assert_eq!(
        inside_the_stager(work, &work.join("environment-tampered")),
        "/work/environment-tampered/."
    );
    for outside in [
        PathBuf::from("/elsewhere/environment-tampered"),
        work.join("../environment"),
        work.to_path_buf(),
    ] {
        assert!(
            std::panic::catch_unwind(|| inside_the_stager(work, &outside)).is_err(),
            "{} was staged as if the stager could see it",
            outside.display()
        );
    }
}

/// **An install that could not finish ends the wait when it says so**: the
/// serial line of 2026-09-16's run, which ended *could not be installed* and
/// then waited for a person, is a machine waiting; a run still installing, one
/// that finished, and one that only quotes the sentence inside another line are
/// not.
#[test]
fn an_install_that_could_not_finish_ends_the_wait_when_it_says_so() {
    let waiting = english("installing.restart-when-ready");
    let failed = format!(
        "\r\n{}\r\n\r\n/usr/bin/bootc: Deploying container image...done (3 minutes)\r\n\
         \r\n/usr/bin/bootc: error: Installing to disk: No such file or directory (os error 2)\r\n\
         \r\n{}\r\n\r\n{waiting}\r\n[ 2894.875975] EXT4-fs (vdb3): unmounting filesystem.\r\n",
        english("installing.installing"),
        english("installing.not-installed"),
    );
    assert!(waits_for_a_person(&failed, &waiting));

    let still = format!(
        "\r\n{}\r\n\r\n{}\r\n",
        english("installing.installing"),
        english("installing.still-installing")
    );
    assert!(!waits_for_a_person(&still, &waiting));
    let finished = format!("{still}\r\n{}\r\n", english("installing.installed"));
    assert!(!waits_for_a_person(&finished, &waiting));
    let quoted = format!("{still}/usr/bin/bootc: the console said \"{waiting}\"\r\n");
    assert!(!waits_for_a_person(&quoted, &waiting));
}

/// **A service that is not running is reported with the machine's own account of
/// why**: what lies between the watching unit's markers, without the serial
/// line's carriage returns — the rest of what it said is not the reason — and
/// saying plainly when the unit never got that far.
#[test]
fn a_service_that_is_not_running_is_reported_with_why() {
    let told = "Id=alo-agentd.service\r\nActiveState=failed\r\nSubState=failed\r\n\r\n\
                ALO-WHY-BEGIN\r\n\u{d7} alo-agentd.service - alo OS agent service\r\n\
                [   41.0] alo-agentd[812]: the record's folder is not there\r\n\
                ALO-WHY-END\r\n[   45.1] reboot: Power down\r\n";
    let why = why_it_said(told);
    assert!(why.starts_with('\u{d7}'), "{why}");
    assert!(why.ends_with("the record's folder is not there"), "{why}");
    assert!(!why.contains('\r') && !why.contains("ActiveState") && !why.contains("Power down"));

    let cut_short = "Id=alo-agentd.service\nALO-WHY-BEGIN\nthe journal began";
    assert_eq!(why_it_said(cut_short), "the journal began");
    assert!(why_it_said("Id=alo-agentd.service\nActiveState=failed\n").starts_with("nothing"));
}
