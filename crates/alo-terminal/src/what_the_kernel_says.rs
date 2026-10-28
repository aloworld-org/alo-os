//! What a machine can find out about a process that looks stuck, read from
//! `/proc`.
//!
//! **Three unprivileged reads and nothing else.** No `ptrace`, no capability,
//! no kernel boundary — which is why `docs/features.md` calls this the first
//! buildable of its group, and why it is honest to ship before anything that
//! needs a boundary exists.
//!
//! The reading of this evidence is [`crate::why_a_command_is_waiting`], which
//! is deliberately somewhere else: the table is the part that can be wrong in
//! an interesting way, and it is held to its cases on any platform. This file
//! is the part that needs Linux, and it does as little as possible.
//!
//! # Every read here answers [`None`] rather than failing
//!
//! A process that exits between two of these reads is the ordinary case, not an
//! error: `/proc/PID` disappears the moment it does. So a missing file means
//! *there is nothing to say about this*, which is exactly what
//! [`crate::WhyItIsWaiting::NobodyCanTell`] is for, and an `Err` here would
//! make a routine race look like a fault.

use std::fs;
use std::path::{Path, PathBuf};

use crate::why_a_command_is_waiting::{TheEvidence, WhatItReadsFrom};

/// Everything that can be found out about this process, from `/proc`.
///
/// Answers [`None`] only when the process is not there at all — when even its
/// state could not be read. A process whose `wchan` is unreadable still has
/// evidence worth having, so that is a [`None`] *inside* the answer rather than
/// instead of it.
#[must_use]
pub fn what_can_be_found_out_about(process: u32) -> Option<TheEvidence> {
    let about = Path::new("/proc").join(process.to_string());
    Some(TheEvidence {
        running: is_running(&about)?,
        kernel_says: what_it_is_parked_in(&about),
        reads_from: what_it_reads_from(&about),
    })
}

/// What this process is called, as the kernel records it.
///
/// `/proc/PID/comm` — the executable's own name, **not** what `$SHELL` says it
/// should be. A person's shell is whatever is really running on the other end
/// of the pseudoterminal, and those two can differ: `$SHELL` is a preference
/// the environment carries and may be stale, inherited, or set by something
/// other than the thing that actually started.
///
/// Truncated by the kernel to fifteen characters, which is a fact about
/// `comm` rather than about any shell, and is why this is not used for
/// anything that must be a whole path.
///
/// [`None`] where the process is gone, which is the ordinary case rather than
/// an error.
#[must_use]
pub fn what_this_process_is_called(process: u32) -> Option<String> {
    let said =
        fs::read_to_string(Path::new("/proc").join(process.to_string()).join("comm")).ok()?;
    let said = said.trim();
    if said.is_empty() {
        return None;
    }
    Some(said.to_owned())
}

/// The folder this process is in, as the kernel records it.
///
/// `/proc/PID/cwd` — a link the kernel keeps to the process's own working
/// folder, so a shell that has just run `cd` answers with where it went. **Read
/// when it is asked and never watched**: nothing here keeps it, polls it or is
/// told when it changes, and a caller decides when the question is worth
/// asking.
///
/// A folder that has been deleted out from under the process comes back as the
/// kernel writes it, with ` (deleted)` after the path. That is left as it is:
/// it is true, and it is what a person in that folder most needs to know.
///
/// [`None`] where the process is gone or the link cannot be read, which is the
/// ordinary case rather than an error — the same as [`what_this_process_is_called`].
#[must_use]
pub fn where_this_process_is(process: u32) -> Option<PathBuf> {
    fs::read_link(Path::new("/proc").join(process.to_string()).join("cwd")).ok()
}

/// Whether the kernel has this process on a processor.
///
/// Field 3 of `/proc/PID/stat`, and **read from the last `)` rather than by
/// splitting the line**. The second field is the executable's name in
/// parentheses and a program may be called `my program (2)`, so counting
/// spaces from the left finds the wrong field for exactly the processes
/// somebody chose an awkward name for. The name cannot contain a `)` after the
/// kernel's own escaping, so the last one is the end of field 2.
fn is_running(about: &Path) -> Option<bool> {
    let said = fs::read_to_string(about.join("stat")).ok()?;
    let after_the_name = said.rfind(')').map(|at| at + 2)?;
    let state = said.get(after_the_name..)?.split_whitespace().next()?;
    Some(state == "R")
}

/// The kernel function this process is parked in, where the kernel will say.
///
/// `0` means *not parked*, which is what a running process reads, and is
/// answered as [`None`] rather than as a function called zero. The file may
/// also be unreadable on a hardened kernel; that is the same [`None`], because
/// the two are the same fact to a reader — **nothing was learned here** — and
/// telling them apart would need a privilege this is built to avoid.
fn what_it_is_parked_in(about: &Path) -> Option<String> {
    let said = fs::read_to_string(about.join("wchan")).ok()?;
    let said = said.trim();
    if said.is_empty() || said == "0" {
        return None;
    }
    Some(said.to_owned())
}

/// What this process reads from, from where its first file points.
///
/// **The evidence `wchan` cannot give.** Measured 2026-10-14: a process reading
/// a terminal and a process reading a socket are both parked in `wait_woken`,
/// so the kernel function says only *parked* and the file says *parked on
/// what*.
///
/// A link that cannot be read is [`WhatItReadsFrom::SomethingElse`] rather than
/// an error: a process may have closed its input, and a command with no input
/// at all is not waiting for the keyboard.
fn what_it_reads_from(about: &Path) -> WhatItReadsFrom {
    let Ok(points_at) = fs::read_link(about.join("fd").join("0")) else {
        return WhatItReadsFrom::SomethingElse;
    };
    let points_at = points_at.to_string_lossy();

    // The shapes `/proc` uses for a file that is not a file on a disk.
    if points_at.starts_with("socket:") {
        WhatItReadsFrom::ASocket
    } else if points_at.starts_with("pipe:") {
        WhatItReadsFrom::APipe
    } else if is_a_terminal(&points_at) {
        WhatItReadsFrom::ATerminal
    } else if points_at.starts_with('/') {
        WhatItReadsFrom::AFile
    } else {
        WhatItReadsFrom::SomethingElse
    }
}

/// Whether this path is a terminal.
///
/// The two shapes Linux gives one: a pseudoterminal under `/dev/pts/`, which is
/// what every terminal emulator opens, and the older `/dev/tty*` for a console.
///
/// A separate function because it is the one question here with two right
/// answers, and because a reader checking *does this catch the console as well*
/// should find both in one place.
fn is_a_terminal(points_at: &str) -> bool {
    points_at.starts_with("/dev/pts/") || points_at.starts_with("/dev/tty")
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "what_the_kernel_says_tests.rs"]
mod tests;
