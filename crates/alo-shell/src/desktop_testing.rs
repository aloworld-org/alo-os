//! What the ordinary desktop's tests are written against: a person's
//! appearance, a kernel whose every file a test wrote, and folders on a real
//! disk.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_appearance::{Appearance, TimeOfDay};
use alo_measuring::{Kernel, Reading};
use alo_strings::Direction;

use crate::DesktopLook;
pub(crate) use crate::approval_testing::words;

/// A person's appearance as a machine ships it.
pub(crate) fn an_appearance() -> Appearance {
    Appearance::shipped()
}

/// Midday.
pub(crate) fn noon() -> TimeOfDay {
    TimeOfDay::checked(12, 0).unwrap()
}

/// `appearance` at midday, read `reading`.
pub(crate) fn noon_look(appearance: &Appearance, reading: Direction) -> DesktopLook {
    DesktopLook::of(appearance, noon(), reading)
}

/// One second, which is the interval every reading here is taken over.
pub(crate) fn a_second() -> Duration {
    Duration::from_secs(1)
}

/// What one written process is using, as totals.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Totals {
    /// Processor time, in ticks.
    pub(crate) ticks: u64,
    /// Resident memory, in kilobytes as the kernel writes it.
    pub(crate) memory_kb: u64,
    /// Bytes read.
    pub(crate) read: u64,
    /// Bytes written.
    pub(crate) written: u64,
    /// Bytes received on its network.
    pub(crate) received: u64,
    /// Bytes sent on its network.
    pub(crate) sent: u64,
}

/// A kernel a test writes: files by path, and what reading each does.
#[derive(Debug, Default, Clone)]
pub(crate) struct Written {
    /// Every file, with its text or the error reading it gives.
    files: BTreeMap<PathBuf, Result<String, io::ErrorKind>>,
    /// Every link.
    links: BTreeMap<PathBuf, String>,
}

impl Written {
    /// A machine whose processors have counted `ticks`, and no processes yet.
    pub(crate) fn a_machine(ticks: u64) -> Self {
        let mut written = Self::default();
        written.file(
            "/proc/stat",
            &format!("cpu  {ticks} 0 0 0 0 0 0 0 0 0\ncpu0 1 0 0 0 0 0 0 0 0 0\n"),
        );
        written.file(
            "/proc/meminfo",
            "MemTotal:       16000000 kB\nMemAvailable:   12000000 kB\n",
        );
        written
    }

    /// A file with this text.
    fn file(&mut self, at: &str, text: &str) {
        self.files.insert(PathBuf::from(at), Ok(text.to_owned()));
    }

    /// A file the kernel will not show.
    pub(crate) fn withholding(&mut self, at: &str) {
        self.files
            .insert(PathBuf::from(at), Err(io::ErrorKind::PermissionDenied));
    }

    /// A process that is gone.
    pub(crate) fn ending(&mut self, pid: u32) {
        let prefix = PathBuf::from(format!("/proc/{pid}"));
        self.files.retain(|at, _| !at.starts_with(&prefix));
        self.links.retain(|at, _| !at.starts_with(&prefix));
    }

    /// A process with every file, started at `started`, using `totals`, on
    /// network namespace `namespace`.
    pub(crate) fn process(
        &mut self,
        pid: u32,
        name: &str,
        started: u64,
        totals: Totals,
        namespace: u64,
    ) {
        let dir = format!("/proc/{pid}");
        self.file(
            &format!("{dir}/stat"),
            &format!(
                "{pid} ({name}) S 1 1 1 0 -1 4194304 0 0 0 0 {} 0 0 0 20 0 1 0 {started} 0 0 0",
                totals.ticks
            ),
        );
        self.file(
            &format!("{dir}/status"),
            &format!("Name:\t{name}\nVmRSS:\t{} kB\n", totals.memory_kb),
        );
        self.file(
            &format!("{dir}/io"),
            &format!("rchar: {}\nwchar: {}\n", totals.read, totals.written),
        );
        self.file(
            &format!("{dir}/net/dev"),
            &format!(
                "Inter-| Receive | Transmit\n face |bytes packets errs drop fifo frame compressed multicast|bytes packets errs drop fifo colls carrier compressed\n    lo: 5 0 0 0 0 0 0 0 5 0 0 0 0 0 0 0\n  eth0: {} 0 0 0 0 0 0 0 {} 0 0 0 0 0 0 0\n",
                totals.received, totals.sent
            ),
        );
        self.links.insert(
            PathBuf::from(format!("{dir}/ns/net")),
            format!("net:[{namespace}]"),
        );
    }

    /// What this kernel says now.
    pub(crate) fn reading(&self) -> Reading {
        Reading::of_kernel(self, Path::new("/proc")).unwrap()
    }
}

impl Kernel for Written {
    fn list(&self, directory: &Path) -> io::Result<Vec<String>> {
        let mut names: Vec<String> = self
            .files
            .keys()
            .chain(self.links.keys())
            .filter_map(|path| path.strip_prefix(directory).ok())
            .filter_map(|rest| rest.components().next())
            .map(|first| first.as_os_str().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names.dedup();
        if names.is_empty() {
            return Err(io::Error::from(io::ErrorKind::NotFound));
        }
        Ok(names)
    }

    fn read(&self, file: &Path) -> io::Result<String> {
        match self.files.get(file) {
            Some(Ok(text)) => Ok(text.clone()),
            Some(Err(kind)) => Err(io::Error::from(*kind)),
            None => Err(io::Error::from(io::ErrorKind::NotFound)),
        }
    }

    fn link(&self, link: &Path) -> io::Result<PathBuf> {
        self.links
            .get(link)
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }
}

/// An afternoon on a written machine, as two readings a second apart: an
/// editor busy on the processor and the disk, a sandboxed application alone on
/// its network, another person's process whose files the kernel would not show,
/// a process that ended between the readings, and one that began between them.
pub(crate) fn an_afternoon() -> (Reading, Reading) {
    let quiet = Totals {
        ticks: 10,
        memory_kb: 1000,
        read: 100,
        written: 200,
        received: 300,
        sent: 400,
    };
    let mut kernel = Written::a_machine(1000);
    kernel.process(101, "editor", 50, quiet, 4_026_531_840);
    kernel.process(202, "sandboxed", 60, quiet, 4_026_532_000);
    kernel.process(303, "theirs", 70, quiet, 4_026_531_840);
    kernel.withholding("/proc/303/io");
    kernel.process(404, "finished", 80, quiet, 4_026_531_840);
    let earlier = kernel.reading();

    let mut later = Written::a_machine(2000);
    later.process(
        101,
        "editor",
        50,
        Totals {
            ticks: 260,
            memory_kb: 52_000,
            read: 900_100,
            written: 70_200,
            ..quiet
        },
        4_026_531_840,
    );
    later.process(
        202,
        "sandboxed",
        60,
        Totals {
            received: 12_300,
            sent: 4_400,
            ..quiet
        },
        4_026_532_000,
    );
    later.process(303, "theirs", 70, quiet, 4_026_531_840);
    later.withholding("/proc/303/io");
    later.process(505, "newcomer", 900, quiet, 4_026_531_840);
    later.ending(404);
    (earlier, later.reading())
}

/// A folder on a real disk, with a tree of files a test can count by hand.
pub(crate) struct Folder {
    /// Held so it outlives the test.
    _held: tempfile::TempDir,
    /// Where it is.
    pub(crate) at: PathBuf,
}

/// `Documents`, holding `letters` (two files of 1000 and 2000 bytes, and a
/// folder `old` with one of 500), `photo.jpg` of 40 000 bytes, and `notes.txt`
/// of 12 bytes.
pub(crate) fn documents() -> Folder {
    let held = tempfile::tempdir().unwrap();
    let at = held.path().join("Documents");
    let letters = at.join("letters");
    let old = letters.join("old");
    std::fs::create_dir_all(&old).unwrap();
    std::fs::write(letters.join("to-ada.txt"), vec![b'a'; 1000]).unwrap();
    std::fs::write(letters.join("to-grace.txt"), vec![b'g'; 2000]).unwrap();
    std::fs::write(old.join("draft.txt"), vec![b'd'; 500]).unwrap();
    std::fs::write(at.join("photo.jpg"), vec![0; 40_000]).unwrap();
    std::fs::write(at.join("notes.txt"), b"hello, world").unwrap();
    Folder { _held: held, at }
}
