//! Every file of a guest's Windows, read from the host with the machine off.
//!
//! **Why here and not inside the guest.** Measured on 2026-09-21, hashing the
//! system directory's 623 programs inside the guest took 232 s, and its
//! libraries would have taken over twenty minutes a reading; the whole Windows
//! partition — about 250 000 files — read from the host took five to nine
//! minutes. And it is the stronger reading: nothing inside Windows is running
//! while its files are read, so nothing it holds open is skipped and nothing it
//! is writing is read half-written.
//!
//! **Nothing here can change the disk it reads.** The disk is attached with
//! `qemu-nbd --read-only`, and both partitions are mounted read-only on top of
//! that.
//!
//! **Nothing unread is dropped.** A file the reader cannot read stands in the
//! reading as [`UNREADABLE`] and a directory it cannot list as
//! [`UNLISTABLE`], so a thing that could not be examined is named rather than
//! missing, and one that becomes unreadable is a change.
//!
//! # What *byte-for-byte what they were* is held to
//!
//! A running Windows rewrites some of its own files on every start, so the
//! partition cannot be identical to the installed image after any start at
//! all. What it is held to is **the controls**, started the same number of
//! times: the same Windows with the installer never run, and with the
//! installer run the same way and refused at the consent. Nothing may differ
//! from the installed image that the refusal did not also change, outside the
//! directories in which the controls disagree with each other — sets that are
//! measured, never written down by hand, because a list of places to ignore is
//! a manifest narrowed until it cannot fail.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ring::digest::{Context, SHA256};

use super::machine::run;

/// Files left out of a reading, and only these: the page, swap and
/// hibernation files and System Volume Information, which Windows rewrites on
/// every start and which neither a person nor a start-up reads.
const LEFT_OUT: [&str; 4] = [
    "pagefile.sys",
    "swapfile.sys",
    "hiberfil.sys",
    "System Volume Information",
];

/// What stands where a file's digest would be, for a file the reader could not
/// read.
pub const UNREADABLE: &str = "UNREADABLE";

/// What stands for a directory the reader could not list.
pub const UNLISTABLE: &str = "UNLISTABLE";

/// One reading of a disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    /// Every file of the Windows partition, by path, and its digest — or
    /// [`UNREADABLE`], or [`UNLISTABLE`] for a directory.
    pub windows: BTreeMap<String, String>,
    /// Every file of the partition the firmware starts Windows from.
    pub start_partition: BTreeMap<String, String>,
    /// The partition table, as `sfdisk -d` prints it.
    pub table: String,
}

/// What differs between two readings, by path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Changed {
    /// Paths of the Windows partition.
    pub windows: BTreeSet<String>,
    /// Paths of the start partition.
    pub start_partition: BTreeSet<String>,
}

impl Reading {
    /// Read a disk that no machine is running from.
    ///
    /// # Panics
    /// When a machine is running, or the disk cannot be attached or mounted.
    #[must_use]
    pub fn of(disk: &Path, yard: &Path) -> Self {
        assert!(
            !super::machine::is_running(),
            "a machine is running, and its disk would be read while it changes"
        );
        let attached = Attached::new(disk, yard);
        let device = attached.device.display().to_string();
        let table = run("sfdisk", &["-d", &device]).replace(&device, "disk");
        run(
            "ntfs-3g",
            &[
                "-o",
                "ro,show_sys_files",
                &format!("{device}p3"),
                &attached.windows.display().to_string(),
            ],
        );
        run(
            "mount",
            &[
                "-o",
                "ro",
                "-t",
                "vfat",
                &format!("{device}p1"),
                &attached.start_partition.display().to_string(),
            ],
        );
        let windows = every_file(&attached.windows);
        assert!(
            windows.len() > 1000,
            "a reading of {} files is not a Windows",
            windows.len()
        );
        Self {
            windows,
            start_partition: every_file(&attached.start_partition),
            table,
        }
    }

    /// Everything the reader could not read or list, for the report.
    #[must_use]
    pub fn unread(&self) -> BTreeSet<String> {
        self.windows
            .iter()
            .chain(&self.start_partition)
            .filter(|(_, digest)| digest.as_str() == UNREADABLE || digest.as_str() == UNLISTABLE)
            .map(|(path, _)| path.clone())
            .collect()
    }

    /// What differs between this reading and another.
    #[must_use]
    pub fn changed_to(&self, other: &Self) -> Changed {
        Changed {
            windows: differing(&self.windows, &other.windows),
            start_partition: differing(&self.start_partition, &other.start_partition),
        }
    }
}

impl Changed {
    /// What changed here that did not change in the control.
    #[must_use]
    pub fn beyond(&self, control: &Self) -> Self {
        Self {
            windows: self.windows.difference(&control.windows).cloned().collect(),
            start_partition: self
                .start_partition
                .difference(&control.start_partition)
                .cloned()
                .collect(),
        }
    }

    /// What either this or another changed.
    #[must_use]
    pub fn and(mut self, other: &Self) -> Self {
        self.windows.extend(other.windows.iter().cloned());
        self.start_partition
            .extend(other.start_partition.iter().cloned());
        self
    }

    /// Whether nothing differs.
    #[must_use]
    pub fn is_nothing(&self) -> bool {
        self.windows.is_empty() && self.start_partition.is_empty()
    }

    /// What changed outside every directory in which the controls disagree
    /// with each other.
    ///
    /// Measured on 2026-09-21: two starts of the same settled Windows, the
    /// installer never run, disagreed on dozens of paths — logs named by the
    /// minute, OneDrive's own caches, the search index — and on the start
    /// partition's `BCD`, `BCD.LOG` and `BOOTSTAT.DAT`, which Windows rewrites
    /// at every start. A path in such a directory that changed beyond the
    /// control is run-to-run noise; a path anywhere else is not.
    #[must_use]
    pub fn outside(&self, noise: &Noise) -> Self {
        let keep = |paths: &BTreeSet<String>, directories: &BTreeSet<String>| {
            paths
                .iter()
                .filter(|path| !directories.contains(&directory_of(path)))
                .cloned()
                .collect()
        };
        Self {
            windows: keep(&self.windows, &noise.windows),
            start_partition: keep(&self.start_partition, &noise.start_partition),
        }
    }
}

/// The directories in which two controls of the same Windows disagree with
/// each other — measured from their readings, never listed by hand.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Noise {
    /// Directories of the Windows partition.
    pub windows: BTreeSet<String>,
    /// Directories of the start partition.
    pub start_partition: BTreeSet<String>,
}

impl Noise {
    /// The noise between two readings of controls.
    #[must_use]
    pub fn between(one: &Reading, other: &Reading) -> Self {
        let directories =
            |paths: BTreeSet<String>| paths.iter().map(|path| directory_of(path)).collect();
        let changed = one.changed_to(other);
        Self {
            windows: directories(changed.windows),
            start_partition: directories(changed.start_partition),
        }
    }

    /// This noise and another's, together.
    #[must_use]
    pub fn and(mut self, other: &Self) -> Self {
        self.windows.extend(other.windows.iter().cloned());
        self.start_partition
            .extend(other.start_partition.iter().cloned());
        self
    }
}

/// The directory a path is in, with every component that is an identifier —
/// a 40-digit key hash, a `{GUID}` — written as `<id>`.
///
/// Such a directory is named differently on every machine by construction:
/// each walk has a security chip of its own, so Windows' TPM key cache sits
/// under that chip's own hash, and diagnostic snapshots under fresh GUIDs.
/// Measured on 2026-09-21, those were the only paths the kills at steps 5, 6
/// and 7 changed outside the noise before identifiers were compared this way.
/// Nothing else in a path is rewritten.
fn directory_of(path: &str) -> String {
    let directory = path.rsplit_once('/').map_or("", |(directory, _)| directory);
    directory
        .split('/')
        .map(|part| {
            let hash = part.len() == 40 && part.bytes().all(|b| b.is_ascii_hexdigit());
            let guid = part.len() == 38
                && part.starts_with('{')
                && part.ends_with('}')
                && part
                    .bytes()
                    .skip(1)
                    .take(36)
                    .all(|b| b.is_ascii_hexdigit() || b == b'-');
            if hash || guid { "<id>" } else { part }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// The paths present in one and not the other, or with different digests.
fn differing(one: &BTreeMap<String, String>, other: &BTreeMap<String, String>) -> BTreeSet<String> {
    one.keys()
        .chain(other.keys())
        .filter(|path| one.get(*path) != other.get(*path))
        .cloned()
        .collect()
}

/// How many readings this test has taken, so each has mount points of its own.
static THE_READINGS: AtomicUsize = AtomicUsize::new(0);

/// A disk attached read-only, and two mount points of this reading's own —
/// unmounted and detached however the reading ends.
struct Attached {
    /// The device the disk is attached as.
    device: PathBuf,
    /// Where the Windows partition is mounted.
    windows: PathBuf,
    /// Where the start partition is mounted.
    start_partition: PathBuf,
}

impl Attached {
    /// Attach the disk to a device nothing is using, and make the mount points.
    ///
    /// **A device that was just disconnected answered the next connection with
    /// an I/O error**, twice on 2026-09-21, so the device is one whose size is
    /// zero, the attachment waits until it has a size and its partitions, and
    /// it is tried again on another device before the reading fails.
    ///
    /// **The mount points are never shared between readings**: on the same day
    /// a mount left behind by one reading sat under the next, which read a
    /// tree of 143 339 files where the disk held about 250 000.
    fn new(disk: &Path, yard: &Path) -> Self {
        let _ = Command::new("modprobe")
            .args(["nbd", "max_part=8"])
            .output();
        let device = (0..3)
            .find_map(|_| attach(disk))
            .expect("the disk could not be attached to any device");
        let own = format!(
            "{}-{}",
            std::process::id(),
            THE_READINGS.fetch_add(1, Ordering::Relaxed)
        );
        let windows = yard.join(format!("reading-windows-{own}"));
        let start_partition = yard.join(format!("reading-start-{own}"));
        let mounted = std::fs::read_to_string("/proc/mounts").unwrap_or_default();
        for at in [&windows, &start_partition] {
            assert!(
                !mounted.contains(&format!(" {} ", at.display())),
                "{} is already a mount point",
                at.display()
            );
            std::fs::create_dir_all(at).expect("a mount point");
        }
        Self {
            device,
            windows,
            start_partition,
        }
    }
}

impl Drop for Attached {
    fn drop(&mut self) {
        for at in [&self.windows, &self.start_partition] {
            let _ = Command::new("umount").arg(at).output();
            let _ = std::fs::remove_dir(at);
        }
        let _ = Command::new("qemu-nbd")
            .arg("--disconnect")
            .arg(&self.device)
            .output();
    }
}

/// Attach a disk read-only to a device nothing is using, and give it back once
/// it answers — or nothing, having let it go again.
fn attach(disk: &Path) -> Option<PathBuf> {
    let free = (2..8).find(|n| {
        std::fs::read_to_string(format!("/sys/block/nbd{n}/size"))
            .is_ok_and(|size| size.trim() == "0")
    })?;
    let device = PathBuf::from(format!("/dev/nbd{free}"));
    let connected = Command::new("qemu-nbd")
        .arg("--read-only")
        .arg(format!("--connect={}", device.display()))
        .arg(disk)
        .status()
        .is_ok_and(|status| status.success());
    if connected {
        for _ in 0..20 {
            let sized = std::fs::read_to_string(format!("/sys/block/nbd{free}/size"))
                .is_ok_and(|size| size.trim() != "0");
            if sized && PathBuf::from(format!("/dev/nbd{free}p3")).exists() {
                let answers = Command::new("sfdisk")
                    .arg("-d")
                    .arg(&device)
                    .output()
                    .is_ok_and(|ran| ran.status.success());
                if answers {
                    return Some(device);
                }
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    let _ = Command::new("qemu-nbd")
        .arg("--disconnect")
        .arg(&device)
        .output();
    std::thread::sleep(Duration::from_secs(5));
    None
}

/// Every file under a directory, by its path from there, and its SHA-256.
fn every_file(root: &Path) -> BTreeMap<String, String> {
    let mut files = Vec::new();
    let mut unlistable = Vec::new();
    gather(root, root, &mut files, &mut unlistable);
    let queue = Arc::new(Mutex::new(files));
    let read = Arc::new(Mutex::new(
        unlistable
            .into_iter()
            .map(|path| (path, UNLISTABLE.to_owned()))
            .collect::<BTreeMap<_, _>>(),
    ));
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let queue = Arc::clone(&queue);
            let read = Arc::clone(&read);
            std::thread::spawn(move || {
                loop {
                    let next = queue.lock().expect("the queue").pop();
                    let Some((path, full)) = next else { break };
                    let digest = digest_of(&full).unwrap_or_else(|| UNREADABLE.to_owned());
                    read.lock().expect("the reading").insert(path, digest);
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().expect("a reader");
    }
    Arc::try_unwrap(read)
        .expect("every reader finished")
        .into_inner()
        .expect("the reading")
}

/// Every file beneath a directory, not following links, leaving out
/// [`LEFT_OUT`] at the partition's root, and naming every directory that could
/// not be listed.
fn gather(
    root: &Path,
    directory: &Path,
    into: &mut Vec<(String, PathBuf)>,
    unlistable: &mut Vec<String>,
) {
    let inside = |full: &Path| {
        full.strip_prefix(root)
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    let Ok(entries) = std::fs::read_dir(directory) else {
        unlistable.push(inside(directory));
        return;
    };
    for entry in entries {
        let Ok(entry) = entry else {
            unlistable.push(inside(directory));
            continue;
        };
        let full = entry.path();
        let path = inside(&full);
        if directory == root && (LEFT_OUT.contains(&path.as_str()) || path.starts_with('$')) {
            continue;
        }
        let Ok(kind) = entry.file_type() else {
            into.push((path, full));
            continue;
        };
        if kind.is_dir() {
            gather(root, &full, into, unlistable);
        } else if kind.is_file() {
            into.push((path, full));
        }
    }
}

/// A file's SHA-256, or nothing when it cannot be read.
fn digest_of(file: &Path) -> Option<String> {
    use std::io::Read as _;
    let mut reading = std::fs::File::open(file).ok()?;
    let mut context = Context::new(&SHA256);
    let mut buffer = vec![0_u8; 1 << 20];
    loop {
        let got = reading.read(&mut buffer).ok()?;
        if got == 0 {
            break;
        }
        context.update(buffer.get(..got)?);
    }
    Some(
        context
            .finish()
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}
