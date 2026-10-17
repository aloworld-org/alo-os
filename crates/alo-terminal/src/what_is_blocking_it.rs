//! Which process is holding the thing a command is waiting for.
//!
//! `docs/features.md`: *ask, and get [the five answers] — then show the prompt,
//! inspect the connection, **reveal what is blocking it**, or stop.* This is the
//! third of those four, and it is the one that turns *waiting for another
//! process or a lock* from a category into a name.
//!
//! [`crate::WhyItIsWaiting::ForAnotherProcess`] is the answer this exists to
//! follow, and the three ways a command earns it are the three readers here: a
//! file lock somebody holds, a pipe nobody is writing to, and a child of its
//! own.
//!
//! # Everything here is a read, and every read can fail politely
//!
//! Three unprivileged reads under `/proc`, and [`None`] wherever the machine
//! will not say. **A machine that would not answer is not a fault** — a command
//! that finished between two questions, a process belonging to somebody else, a
//! kernel built without `/proc/locks`. The honest answer is then *this machine
//! will not name it*, which is what a caller shows.
//!
//! # What it cannot see, said here rather than discovered
//!
//! **Another person's process.** Reading `/proc/PID/fd` needs to be that
//! process's owner, so a pipe fed by somebody else's program is a pipe this
//! cannot name. The command being asked about is the person's own, and so is
//! whatever it is usually waiting for; where it is not, this answers [`None`]
//! rather than guessing.

use std::path::Path;

/// What is holding the thing a command is waiting for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatIsBlockingIt {
    /// Another process holds a lock on a file this command is waiting for.
    ALockHeldBy(u32),
    /// This command is reading a pipe, and this is the process at the other end.
    APipeFedBy(u32),
    /// This command is waiting for a child of its own.
    AChildOfItsOwn(u32),
}

impl WhatIsBlockingIt {
    /// The process that is holding it up.
    #[must_use]
    pub const fn process(&self) -> u32 {
        match self {
            Self::ALockHeldBy(who) | Self::APipeFedBy(who) | Self::AChildOfItsOwn(who) => *who,
        }
    }
}

/// Where the kernel lists who holds which lock.
const THE_LOCKS: &str = "/proc/locks";

/// What is blocking `process`, if this machine will say.
///
/// # Nothing is blocking a process that is working
///
/// Asked first, before any of the three readers below, because each of them
/// answers *who holds this thing* and that is not the same question as *what is
/// holding this process up*. A command on a processor is held up by nothing, and
/// its descriptors are not evidence about anything.
///
/// # The order of the questions, and why it is this order
///
/// 1. **A lock**, because the kernel says so outright: a blocked waiter appears
///    in `/proc/locks` beside the holder, and there is nothing to infer.
/// 2. **A pipe**, which takes a walk of every process's descriptors and is
///    therefore the expensive one.
/// 3. **A child**, last because a process waiting for a child is usually a
///    shell or a build tool doing exactly what it is for, which is the least
///    interesting answer of the three to a person asking why nothing is
///    happening.
#[must_use]
pub fn what_is_blocking(process: u32) -> Option<WhatIsBlockingIt> {
    // **A process the kernel has on a processor is not blocked by anything**,
    // whatever its descriptors happen to be.
    //
    // *This was missing, and a control test found it.* A busy loop whose
    // standard input happened to be an inherited pipe was reported as blocked
    // by whoever held the other end — a confident, specific, wrong answer about
    // a process working flat out. The three readers below each answer *who
    // holds this thing*, which is a different question from *what is holding
    // this process up*, and nothing joined the two until here.
    //
    // `running` is positive evidence rather than the absence of a reason, which
    // is `crate::why_a_command_is_waiting`'s own argument about the same letter.
    if crate::what_the_kernel_says::what_can_be_found_out_about(process)
        .is_none_or(|evidence| evidence.running)
    {
        return None;
    }

    if let Some(holder) = a_lock_held_by(process) {
        return Some(WhatIsBlockingIt::ALockHeldBy(holder));
    }
    if let Some(writer) = a_pipe_fed_by(process) {
        return Some(WhatIsBlockingIt::APipeFedBy(writer));
    }
    a_child_of_its_own(process).map(WhatIsBlockingIt::AChildOfItsOwn)
}

/// One line of `/proc/locks`, as far as this needs it.
///
/// **The fields are read by skipping the arrow and not by counting**, which is
/// the whole of why this is a function. Measured on 2026-10-17:
///
/// ```text
/// 10: FLOCK  ADVISORY  WRITE 2142693 fd:01:4311 0 EOF
/// 10: -> FLOCK  ADVISORY  WRITE 2142696 fd:01:4311 0 EOF
/// ```
///
/// The two lines are the holder and the process blocked behind it, they share
/// the index, and **the arrow shifts every field by one**. Reading the fifth
/// token would give the pid on one line and the lock's mode on the other — a
/// number that is not a process and a comparison that quietly never matches.
struct ALock {
    /// The index, which a holder and its waiters share.
    index: String,
    /// Whether this line is a process blocked behind the holder.
    blocked: bool,
    /// Whose it is.
    process: u32,
}

impl ALock {
    /// Read one line, or nothing if it is not one this understands.
    fn read(line: &str) -> Option<Self> {
        let mut said = line.split_whitespace();
        let index = said.next()?.trim_end_matches(':').to_owned();

        // The token after the index is the arrow on a blocked line and the kind
        // of lock on a holder's. That one token is the whole difference, and it
        // is why the count below is computed rather than written.
        let first = said.next()?;
        let blocked = first == "->";

        // What is left before the process number:
        //   a holder   MODE ACCESS PID        -> skip two
        //   a waiter   KIND MODE ACCESS PID   -> skip three
        let skip = if blocked { 3 } else { 2 };
        let process = said.nth(skip)?.parse().ok()?;

        Some(Self {
            index,
            blocked,
            process,
        })
    }
}

/// The process holding a lock this one is blocked behind, if any.
///
/// Found through the kernel's own pairing rather than by matching inodes: the
/// blocked line and the holder share an index, so there is nothing to work out
/// and no chance of naming a holder of a different file that happens to have the
/// same inode on another device.
#[must_use]
pub fn a_lock_held_by(process: u32) -> Option<u32> {
    let locks = std::fs::read_to_string(THE_LOCKS).ok()?;
    let lines: Vec<ALock> = locks.lines().filter_map(ALock::read).collect();

    let waiting = lines
        .iter()
        .find(|lock| lock.blocked && lock.process == process)?;
    lines
        .iter()
        .find(|lock| !lock.blocked && lock.index == waiting.index)
        .map(|holder| holder.process)
}

/// The process at the other end of the pipe this one is reading, if any.
///
/// What `fd 0` points at is `pipe:[N]`, and the other end is whichever process
/// has a descriptor pointing at the same `pipe:[N]`. Measured: `sleep 60 | cat`
/// gives the writer that link as `fd 1` and the reader as `fd 0`.
#[must_use]
pub fn a_pipe_fed_by(process: u32) -> Option<u32> {
    let reading = std::fs::read_link(format!("/proc/{process}/fd/0")).ok()?;
    let pipe = reading.to_str()?;
    if !pipe.starts_with("pipe:[") {
        return None;
    }

    for entry in std::fs::read_dir("/proc").ok()? {
        let at = entry.ok()?.path();
        let Some(who) = whose_directory(&at) else {
            continue;
        };
        if who == process {
            continue;
        }
        let Ok(descriptors) = std::fs::read_dir(at.join("fd")) else {
            // Somebody else's process, or one that has gone. Neither is a fault.
            continue;
        };
        for descriptor in descriptors.flatten() {
            if std::fs::read_link(descriptor.path())
                .is_ok_and(|points| points.to_str() == Some(pipe))
            {
                return Some(who);
            }
        }
    }
    None
}

/// A child of this process, if it has one.
///
/// Read from every process's own `stat`, because `/proc/PID/task/TID/children`
/// needs a kernel built with `CONFIG_PROC_CHILDREN` and this does not.
#[must_use]
pub fn a_child_of_its_own(process: u32) -> Option<u32> {
    for entry in std::fs::read_dir("/proc").ok()? {
        let at = entry.ok()?.path();
        let Some(who) = whose_directory(&at) else {
            continue;
        };
        if who == process {
            continue;
        }
        if whose_child(&at) == Some(process) {
            return Some(who);
        }
    }
    None
}

/// Which process a `/proc` entry is about, if it is about one at all.
fn whose_directory(at: &Path) -> Option<u32> {
    at.file_name()?.to_str()?.parse().ok()
}

/// Which process is this one's parent, from its `stat`.
///
/// **Read from the last `)` and not by splitting from the left**, because a
/// program may be called `my (odd) name` and the second field is its name in
/// brackets. `crate::what_the_kernel_says` reads the state letter the same way
/// and for the same reason.
fn whose_child(at: &Path) -> Option<u32> {
    let stat = std::fs::read_to_string(at.join("stat")).ok()?;
    let after = stat.rfind(')')?;
    // After the name come the state letter and then the parent.
    stat.get(after + 1..)?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "what_is_blocking_it_tests.rs"]
mod tests;
