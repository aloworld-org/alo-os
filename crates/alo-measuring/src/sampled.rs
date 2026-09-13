//! What the kernel says about one process, and about the machine, at one
//! moment — totals, before any arithmetic.
//!
//! These are records of facts with nothing to protect, so their fields are
//! public: a test compares them against the files they name, and a window may
//! read them as they are.

use crate::source::{Known, Number};

/// One process at one moment: every total the kernel keeps for it, each named
/// for the file it was read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sampled {
    /// The process id, which is the number of its directory under `/proc`.
    pub pid: u32,

    /// What the process calls itself: `/proc/<pid>/stat`, field 2, up to
    /// fifteen characters, as the kernel keeps it and without the parentheses
    /// it writes round it.
    pub name: String,

    /// When the process started, in ticks since the machine booted:
    /// `/proc/<pid>/stat`, field 22.
    ///
    /// **This is what tells one process from the next that reuses its pid.**
    /// A pid seen in two readings with two different starts is two processes,
    /// the first of them gone.
    pub started: u64,

    /// Resident memory in bytes: `VmRSS` in `/proc/<pid>/status`, which the
    /// kernel writes in kilobytes.
    pub memory: Number,

    /// Processor time in clock ticks, user and kernel together: `utime` and
    /// `stime` in `/proc/<pid>/stat`, fields 14 and 15.
    pub ticks: Number,

    /// Bytes read through the process's open files: `rchar` in
    /// `/proc/<pid>/io`.
    pub read: Number,

    /// Bytes written through the process's open files: `wchar` in
    /// `/proc/<pid>/io`.
    pub written: Number,

    /// Bytes received on every interface but loopback, in the process's
    /// network namespace: `/proc/<pid>/net/dev`.
    pub received: Number,

    /// Bytes sent on every interface but loopback, in the process's network
    /// namespace: `/proc/<pid>/net/dev`.
    pub sent: Number,

    /// Which network namespace the two counts above are for: the number in
    /// `/proc/<pid>/ns/net`, or [`None`] where the kernel would not say.
    pub namespace: Option<u64>,
}

/// The machine at one moment: what a process's share is a share of, and how
/// much memory there is to use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Machine {
    /// Processor time across every processor in every state, in clock ticks:
    /// the sum of the `cpu` line of `/proc/stat`.
    ///
    /// Always known: a machine whose `/proc/stat` cannot be read gives no
    /// reading at all rather than a reading with a gap here.
    pub ticks: Known,

    /// Memory in the machine, in bytes: `MemTotal` in `/proc/meminfo`.
    pub memory_total: Number,

    /// Memory that could be given to a program without swapping, in bytes:
    /// `MemAvailable` in `/proc/meminfo`.
    pub memory_available: Number,
}
