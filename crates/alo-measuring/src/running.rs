//! What is running, and what it is using: the answer, as rates.
//!
//! A [`Running`] is what [`crate::Reading::since`] makes of two readings.
//! Every process still running has a row, every number in it is a rate or a
//! present size, and every process that ended between the readings is in a
//! second list rather than in the first with zeros.
//!
//! These are records of facts with nothing to protect, so the row types have
//! public fields.

use std::time::Duration;

use alo_strings::{Counting, Filling, Said, Strings};

use crate::source::Number;
use crate::words;

/// One running process, and what it is using.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Process {
    /// The process id.
    pub pid: u32,

    /// What the process calls itself.
    pub name: String,

    /// Resident memory now, in bytes: `VmRSS` in `/proc/<pid>/status`.
    pub memory: Number,

    /// Its share of the whole machine's processor over the interval, in
    /// thousandths: its `utime` and `stime` from `/proc/<pid>/stat` against
    /// the `cpu` line of `/proc/stat`. One thread busy on one of eight
    /// processors is `125`.
    pub processor: Number,

    /// Bytes read through its files per second: `rchar` in `/proc/<pid>/io`.
    pub read: Number,

    /// Bytes written through its files per second: `wchar` in
    /// `/proc/<pid>/io`.
    pub written: Number,

    /// Bytes received per second on its network, loopback left out:
    /// `/proc/<pid>/net/dev`. Whose network that is, [`Self::network`] says.
    pub received: Number,

    /// Bytes sent per second on its network, loopback left out:
    /// `/proc/<pid>/net/dev`.
    pub sent: Number,

    /// Whose traffic the two counts above are.
    pub network: Network,
}

/// Whose network traffic a process's counts are.
///
/// The kernel counts per network namespace, not per process. A process in a
/// namespace of its own — a sandboxed application — has counts that are its
/// alone; one in the default namespace shares them with most of the machine,
/// and a window that showed the number without saying so would be attributing
/// the whole machine's traffic to whichever row it was on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Network {
    /// The namespace, as `/proc/<pid>/ns/net` numbers it, or [`None`] where
    /// the kernel would not say.
    pub namespace: Option<u64>,

    /// How many **other** running processes in this list share the namespace,
    /// and so the counts.
    pub shared_with: usize,
}

impl Network {
    /// What a window says beside the counts, in the reader's language.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        if self.shared_with == 0 {
            strings.say(
                &words::NETWORK_THIS_PROCESS_ALONE.key(),
                &Filling::nothing(),
            )
        } else {
            let others = u64::try_from(self.shared_with).unwrap_or(u64::MAX);
            strings.count(
                &words::NETWORK_SHARED.key(),
                &Counting::of(others),
                &Filling::of("others", others.to_string()),
            )
        }
    }
}

/// A process that was running at the earlier reading and was not at the
/// later one.
///
/// Reported by name and pid rather than as a row of zeros: a process using
/// nothing and a process that is not there are different facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gone {
    /// The pid it had.
    pub pid: u32,
    /// What it called itself.
    pub name: String,
}

impl Gone {
    /// What a window says on its row, in the reader's language.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&words::GONE.key(), &Filling::nothing())
    }
}

/// What is running, and what it is using, over an interval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Running {
    /// The interval the rates are over, as the caller gave it.
    interval: Duration,
    /// Every process running at the later reading, in pid order.
    processes: Vec<Process>,
    /// Every process running at the earlier reading and not at the later.
    gone: Vec<Gone>,
    /// Memory in the machine, in bytes: `MemTotal` in `/proc/meminfo`.
    memory_total: Number,
    /// Memory that could be given to a program without swapping, in bytes:
    /// `MemAvailable` in `/proc/meminfo`.
    memory_available: Number,
}

impl Running {
    /// Made by [`crate::Reading::since`], and by nothing else.
    pub(crate) fn of(
        interval: Duration,
        processes: Vec<Process>,
        gone: Vec<Gone>,
        memory_total: Number,
        memory_available: Number,
    ) -> Self {
        Self {
            interval,
            processes,
            gone,
            memory_total,
            memory_available,
        }
    }

    /// The interval the rates are over.
    #[must_use]
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Every process running at the later reading, in pid order.
    #[must_use]
    pub fn processes(&self) -> &[Process] {
        &self.processes
    }

    /// One process, by pid.
    #[must_use]
    pub fn process(&self, pid: u32) -> Option<&Process> {
        self.processes.iter().find(|process| process.pid == pid)
    }

    /// Every process that ended between the readings.
    #[must_use]
    pub fn gone(&self) -> &[Gone] {
        &self.gone
    }

    /// Memory in the machine, in bytes.
    #[must_use]
    pub fn memory_total(&self) -> &Number {
        &self.memory_total
    }

    /// Memory that could be given to a program without swapping, in bytes.
    #[must_use]
    pub fn memory_available(&self) -> &Number {
        &self.memory_available
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// English, with nothing translated.
    fn in_english() -> Strings {
        Strings::of(crate::measuring_words().unwrap())
    }

    /// A count shared with nobody says so; a count shared with one, or with
    /// many, says how many others.
    #[test]
    fn whose_network_a_count_is_is_said_beside_it() {
        let strings = in_english();
        let alone = Network {
            namespace: Some(1),
            shared_with: 0,
        };
        assert_eq!(
            alone.said(&strings).text(),
            "Network traffic counted for this process alone."
        );
        let one = Network {
            namespace: Some(1),
            shared_with: 1,
        };
        assert_eq!(
            one.said(&strings).text(),
            "Network traffic counted together with one other process."
        );
        let many = Network {
            namespace: Some(1),
            shared_with: 211,
        };
        let said = many.said(&strings);
        assert_eq!(
            said.text(),
            "Network traffic counted together with 211 other processes."
        );
        assert!(!said.is_a_bug(), "{said}");
    }

    /// A process that ended says so on its row.
    #[test]
    fn a_process_that_ended_says_so() {
        let gone = Gone {
            pid: 42,
            name: "cat".to_owned(),
        };
        let said = gone.said(&in_english());
        assert_eq!(said.text(), "Ended since the last reading.");
        assert!(!said.is_a_bug(), "{said}");
    }
}
