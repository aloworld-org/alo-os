//! One reading: every process under `/proc`, and the machine, at one moment.
//!
//! The order of reads inside one process is the order of the files, and what
//! each failure means is decided here, once:
//!
//! - **The process's `stat` cannot be read** — the process is not in the
//!   list. If it is gone, it is gone; if the kernel withheld the one file
//!   that says when it started, it cannot be told from a successor that
//!   reuses its pid and cannot be listed honestly. On Linux `stat` is readable
//!   by everyone, so the second does not happen on a working machine.
//! - **A later file of the process is gone** — the process ended between
//!   two of its own reads, and it is not in the list. It was not running at
//!   the moment the reading is of.
//! - **A later file is withheld** — the process is in the list and that
//!   number is [`Number::Withheld`], naming the file.
//! - **A file is there without the line** — [`Number::NotSaid`], naming the
//!   file.
//!
//! The machine's own two files are different: without `/proc/stat` there is
//! no processor for anything to be a share of, so it is the one file whose
//! failure refuses the whole reading.

use std::path::Path;

use crate::kernel::{Kernel, is_gone};
use crate::reading::Reading;
use crate::refusing::NotMeasured;
use crate::sampled::{Machine, Sampled};
use crate::source::{Known, Number, Source};
use crate::{io, kilobytes, listing, netdev, stat};

/// Why a file of a process could not be read.
enum Unread {
    /// The process ended.
    Gone,
    /// The kernel would not show the file, and what it said.
    Withheld(String),
}

/// A file of a process, read.
fn read(kernel: &dyn Kernel, at: &Path) -> Result<String, Unread> {
    kernel.read(at).map_err(|why| {
        if is_gone(&why) {
            Unread::Gone
        } else {
            Unread::Withheld(why.to_string())
        }
    })
}

/// Everything under `proc`, as one reading.
///
/// # Errors
///
/// [`NotMeasured::Unreadable`] when `proc` cannot be listed or its `stat`
/// cannot be read. A process that cannot be read is left out, not an error.
pub(crate) fn everything(kernel: &dyn Kernel, proc: &Path) -> Result<Reading, NotMeasured> {
    let machine = machine(kernel, proc)?;
    let mut processes = Vec::new();
    for pid in listing::pids(kernel, proc)? {
        if let Some(sampled) = one(kernel, proc, pid) {
            processes.push(sampled);
        }
    }
    Ok(Reading::of(machine, processes))
}

/// The machine's two files.
fn machine(kernel: &dyn Kernel, proc: &Path) -> Result<Machine, NotMeasured> {
    let stat_at = proc.join("stat");
    let text = kernel
        .read(&stat_at)
        .map_err(|why| NotMeasured::Unreadable {
            at: stat_at.clone(),
            why: why.to_string(),
        })?;
    let ticks = stat::machine(&text).ok_or_else(|| NotMeasured::Unreadable {
        at: stat_at.clone(),
        why: "it has no cpu line".to_owned(),
    })?;
    let ticks = Known {
        value: ticks,
        from: Source::of(stat_at, "cpu"),
    };

    let meminfo_at = proc.join("meminfo");
    let (memory_total, memory_available) = match kernel.read(&meminfo_at) {
        Ok(text) => (
            in_kilobytes(&text, &meminfo_at, "MemTotal"),
            in_kilobytes(&text, &meminfo_at, "MemAvailable"),
        ),
        Err(why) => (
            withheld(&meminfo_at, "MemTotal", &why.to_string()),
            withheld(&meminfo_at, "MemAvailable", &why.to_string()),
        ),
    };
    Ok(Machine {
        ticks,
        memory_total,
        memory_available,
    })
}

/// One process, or [`None`] if it is not running at this moment.
fn one(kernel: &dyn Kernel, proc: &Path, pid: u32) -> Option<Sampled> {
    let directory = proc.join(pid.to_string());

    let stat_at = directory.join("stat");
    let parsed = stat::process(&read(kernel, &stat_at).ok()?)?;
    let ticks = Number::known(parsed.ticks(), Source::of(&stat_at, "utime+stime"));

    let status_at = directory.join("status");
    let memory = match read(kernel, &status_at) {
        Ok(text) => in_kilobytes(&text, &status_at, "VmRSS"),
        Err(Unread::Gone) => return None,
        Err(Unread::Withheld(why)) => withheld(&status_at, "VmRSS", &why),
    };

    let io_at = directory.join("io");
    let (read_bytes, written) = match read(kernel, &io_at) {
        Ok(text) => match io::counted(&text) {
            Some(counted) => (
                Number::known(counted.read, Source::of(&io_at, "rchar")),
                Number::known(counted.written, Source::of(&io_at, "wchar")),
            ),
            None => (not_said(&io_at, "rchar"), not_said(&io_at, "wchar")),
        },
        Err(Unread::Gone) => return None,
        Err(Unread::Withheld(why)) => (
            withheld(&io_at, "rchar", &why),
            withheld(&io_at, "wchar", &why),
        ),
    };

    let dev_at = directory.join("net").join("dev");
    let (received, sent) = match read(kernel, &dev_at) {
        Ok(text) => match netdev::crossed(&text) {
            Some(crossed) => (
                Number::known(crossed.received, Source::of(&dev_at, "bytes received")),
                Number::known(crossed.sent, Source::of(&dev_at, "bytes sent")),
            ),
            None => (
                not_said(&dev_at, "bytes received"),
                not_said(&dev_at, "bytes sent"),
            ),
        },
        Err(Unread::Gone) => return None,
        Err(Unread::Withheld(why)) => (
            withheld(&dev_at, "bytes received", &why),
            withheld(&dev_at, "bytes sent", &why),
        ),
    };

    let namespace = match kernel.link(&directory.join("ns").join("net")) {
        Ok(link) => netdev::namespace(&link.to_string_lossy()),
        Err(why) if is_gone(&why) => return None,
        Err(_) => None,
    };

    Some(Sampled {
        pid,
        name: parsed.name,
        started: parsed.starttime,
        memory,
        ticks,
        read: read_bytes,
        written,
        received,
        sent,
        namespace,
    })
}

/// A number the kernel writes in kilobytes, or the line's absence.
fn in_kilobytes(text: &str, at: &Path, field: &'static str) -> Number {
    kilobytes::bytes_on_line(text, field).map_or_else(
        || not_said(at, field),
        |bytes| Number::known(bytes, Source::of(at, field)),
    )
}

/// A number the kernel would not show.
fn withheld(at: &Path, field: &'static str, why: &str) -> Number {
    Number::Withheld {
        from: Source::of(at, field),
        why: why.to_owned(),
    }
}

/// A number the file has no line for.
fn not_said(at: &Path, field: &'static str) -> Number {
    Number::NotSaid {
        from: Source::of(at, field),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
pub(crate) mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    /// A kernel a test writes: files by path, and what happens when each is
    /// read.
    #[derive(Default)]
    pub(crate) struct Written {
        /// Every file, by path, with its text or the kind of error reading it
        /// gives.
        pub files: BTreeMap<PathBuf, Result<String, std::io::ErrorKind>>,
        /// Every link, by path.
        pub links: BTreeMap<PathBuf, String>,
        /// Every path read, in order, so a test can say what was opened.
        pub opened: RefCell<Vec<PathBuf>>,
    }

    impl Written {
        /// A machine with a `stat` and a `meminfo`, and no processes yet.
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

        /// A file, with this text.
        pub(crate) fn file(&mut self, at: &str, text: &str) -> &mut Self {
            self.files.insert(PathBuf::from(at), Ok(text.to_owned()));
            self
        }

        /// A file that gives this error when read.
        pub(crate) fn failing(&mut self, at: &str, kind: std::io::ErrorKind) -> &mut Self {
            self.files.insert(PathBuf::from(at), Err(kind));
            self
        }

        /// An ordinary process with every file, using these totals.
        pub(crate) fn process(&mut self, pid: u32, name: &str, started: u64, totals: Totals) {
            let dir = format!("/proc/{pid}");
            self.file(
                &format!("{dir}/stat"),
                &format!(
                    "{pid} ({name}) S 1 1 1 0 -1 4194304 0 0 0 0 {} {} 0 0 20 0 1 0 {started} 0 0 0",
                    totals.ticks, 0
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
                format!("net:[{}]", totals.namespace),
            );
        }
    }

    /// The totals a written process has.
    #[derive(Debug, Clone, Copy)]
    pub(crate) struct Totals {
        /// Its ticks.
        pub ticks: u64,
        /// Its resident memory, in kilobytes as the kernel writes it.
        pub memory_kb: u64,
        /// Its `rchar`.
        pub read: u64,
        /// Its `wchar`.
        pub written: u64,
        /// Its namespace's bytes received.
        pub received: u64,
        /// Its namespace's bytes sent.
        pub sent: u64,
        /// Its namespace.
        pub namespace: u64,
    }

    impl Totals {
        /// Round numbers, easy to check by eye.
        pub(crate) const fn round() -> Self {
            Self {
                ticks: 100,
                memory_kb: 2000,
                read: 3000,
                written: 4000,
                received: 5000,
                sent: 6000,
                namespace: 4_026_531_840,
            }
        }
    }

    impl Kernel for Written {
        fn list(&self, directory: &Path) -> std::io::Result<Vec<String>> {
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
                return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
            }
            Ok(names)
        }

        fn read(&self, file: &Path) -> std::io::Result<String> {
            self.opened.borrow_mut().push(file.to_path_buf());
            match self.files.get(file) {
                Some(Ok(text)) => Ok(text.clone()),
                Some(Err(kind)) => Err(std::io::Error::from(*kind)),
                None => Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
            }
        }

        fn link(&self, link: &Path) -> std::io::Result<PathBuf> {
            self.links
                .get(link)
                .map(PathBuf::from)
                .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
        }
    }

    /// Where a written kernel's `/proc` is.
    pub(crate) fn proc() -> &'static Path {
        Path::new("/proc")
    }

    /// **Every number is read from the file it is named for**, and the file
    /// is the one under the process's own directory.
    #[test]
    fn every_number_names_the_file_and_line_it_was_read_from() {
        let mut kernel = Written::a_machine(1000);
        kernel.process(42, "cat", 500, Totals::round());
        let reading = everything(&kernel, proc()).unwrap();
        assert_eq!(reading.processes().len(), 1, "one process was written");
        let process = reading.process(42).unwrap();
        assert_eq!(process.pid, 42);
        assert_eq!(process.name, "cat");
        assert_eq!(process.started, 500);
        let expected = [
            (&process.memory, 2000 * 1024, "/proc/42/status", "VmRSS"),
            (&process.ticks, 100, "/proc/42/stat", "utime+stime"),
            (&process.read, 3000, "/proc/42/io", "rchar"),
            (&process.written, 4000, "/proc/42/io", "wchar"),
            (
                &process.received,
                5000,
                "/proc/42/net/dev",
                "bytes received",
            ),
            (&process.sent, 6000, "/proc/42/net/dev", "bytes sent"),
        ];
        for (number, value, file, field) in expected {
            assert_eq!(number.value(), Some(value), "{field}");
            assert_eq!(number.from(), &Source::of(file, field));
        }
        assert_eq!(process.namespace, Some(4_026_531_840));
        assert_eq!(reading.machine().ticks.value, 1000);
        assert_eq!(
            reading.machine().ticks.from,
            Source::of("/proc/stat", "cpu")
        );
        assert_eq!(
            reading.machine().memory_total,
            Number::known(16_000_000 * 1024, Source::of("/proc/meminfo", "MemTotal"))
        );
        assert_eq!(
            reading.machine().memory_available.value(),
            Some(12_000_000 * 1024)
        );
    }

    /// **A file the kernel would not show is withheld, not zero**, and it
    /// names the file. The other numbers of the same process are still read.
    #[test]
    fn a_file_the_kernel_withholds_is_withheld_and_not_a_zero() {
        let mut kernel = Written::a_machine(1000);
        kernel.process(42, "theirs", 500, Totals::round());
        kernel.failing("/proc/42/io", std::io::ErrorKind::PermissionDenied);
        let reading = everything(&kernel, proc()).unwrap();
        let process = reading.process(42).unwrap();
        assert!(
            matches!(&process.read, Number::Withheld { from, .. } if from == &Source::of("/proc/42/io", "rchar")),
            "{:?}",
            process.read
        );
        assert!(matches!(&process.written, Number::Withheld { .. }));
        assert_eq!(process.read.value(), None);
        assert_eq!(process.memory.value(), Some(2000 * 1024));
        assert_eq!(process.sent.value(), Some(6000));
    }

    /// **A process that ended between the listing and its own reads is not in
    /// the list**, because it was not running at the moment the reading is
    /// of — and a process whose `stat` is not there is likewise not there.
    #[test]
    fn a_process_that_ends_while_being_read_is_not_listed() {
        let mut kernel = Written::a_machine(1000);
        kernel.process(42, "stays", 500, Totals::round());
        kernel.process(43, "goes", 501, Totals::round());
        kernel.failing("/proc/43/io", std::io::ErrorKind::NotFound);
        kernel.process(44, "went", 502, Totals::round());
        kernel.files.remove(Path::new("/proc/44/stat"));
        let reading = everything(&kernel, proc()).unwrap();
        let pids: Vec<u32> = reading.processes().iter().map(|p| p.pid).collect();
        assert_eq!(pids, [42]);
    }

    /// **A line the file does not have is *not said*, not zero.** A kernel
    /// thread has no `VmRSS`.
    #[test]
    fn a_line_the_file_does_not_have_is_not_said_and_not_a_zero() {
        let mut kernel = Written::a_machine(1000);
        kernel.process(2, "kthreadd", 1, Totals::round());
        kernel.file("/proc/2/status", "Name:\tkthreadd\nThreads:\t1\n");
        let reading = everything(&kernel, proc()).unwrap();
        let process = reading.process(2).unwrap();
        assert_eq!(
            process.memory,
            Number::NotSaid {
                from: Source::of("/proc/2/status", "VmRSS")
            }
        );
    }

    /// **Without `/proc/stat` there is no reading**, and the refusal names
    /// the file; without `/proc` at all, the same.
    #[test]
    fn a_machine_whose_stat_cannot_be_read_gives_no_reading_and_names_the_file() {
        let mut kernel = Written::a_machine(1000);
        kernel.failing("/proc/stat", std::io::ErrorKind::PermissionDenied);
        let refused = everything(&kernel, proc()).unwrap_err();
        assert_eq!(
            refused,
            NotMeasured::Unreadable {
                at: PathBuf::from("/proc/stat"),
                why: std::io::Error::from(std::io::ErrorKind::PermissionDenied).to_string(),
            }
        );

        let nothing = Written::default();
        let refused = everything(&nothing, proc()).unwrap_err();
        assert!(
            matches!(&refused, NotMeasured::Unreadable { at, .. } if at == Path::new("/proc/stat")),
            "{refused}"
        );
    }

    /// The machine's memory being unreadable is a gap in the reading, not a
    /// refusal of it: there is still a list.
    #[test]
    fn a_machine_whose_meminfo_cannot_be_read_still_has_a_list() {
        let mut kernel = Written::a_machine(1000);
        kernel.process(42, "cat", 500, Totals::round());
        kernel.failing("/proc/meminfo", std::io::ErrorKind::PermissionDenied);
        let reading = everything(&kernel, proc()).unwrap();
        assert_eq!(reading.processes().len(), 1);
        assert!(matches!(
            reading.machine().memory_total,
            Number::Withheld { .. }
        ));
    }

    /// **Nothing is read outside `/proc`.** Every path the kernel was asked
    /// for is under it: what this crate reads is the kernel's files and
    /// nothing else on the disk.
    #[test]
    fn every_file_opened_is_under_proc() {
        let mut kernel = Written::a_machine(1000);
        kernel.process(42, "cat", 500, Totals::round());
        drop(everything(&kernel, proc()).unwrap());
        let opened = kernel.opened.borrow();
        assert!(!opened.is_empty());
        for path in opened.iter() {
            assert!(path.starts_with("/proc"), "{} was opened", path.display());
        }
    }
}
