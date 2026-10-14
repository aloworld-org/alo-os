//! Why a command that looks stuck is waiting, from evidence rather than from a
//! guess.
//!
//! `docs/features.md`: *a command looks stuck; ask, and get **waiting for the
//! keyboard**, **for the network**, **for another process or a lock**, **still
//! computing**, or **unknown with the evidence available** — instead of killing
//! something that was nearly done.*
//!
//! This module is the reading. [`crate::what_the_kernel_says`] is where the
//! evidence comes from on a machine; everything here is arithmetic over
//! [`TheEvidence`], so the whole table can be held to its cases on any
//! platform and the part that needs Linux is only the part that reads.
//!
//! # What was measured, and how it differs from what was written down
//!
//! Measured on 2026-10-14, kernel **7.0.0-31-generic**, each case with a
//! control:
//!
//! | what the process was doing | state | `wchan` | its stdin |
//! |---|---|---|---|
//! | `sleep 30` | `S` | `hrtimer_nanosleep` | a pipe |
//! | `cat` on an idle pipe | `S` | `anon_pipe_read` | a pipe |
//! | `cat` on a terminal | `S` | `wait_woken` | `/dev/pts/0` |
//! | reading a socket nothing writes to | `S` | `wait_woken` | — |
//! | waiting on a file lock another process holds | `S` | `locks_lock_inode_wait` | a pipe |
//! | waiting for a child | `S` | `do_wait` | a pipe |
//! | a busy loop *(control)* | `R` | `0` | a pipe |
//!
//! **Two of those overturn the design this was going to have.**
//!
//! **A socket read and a terminal read are the same `wchan`.** Both are
//! `wait_woken`. So the kernel function cannot tell *waiting for the keyboard*
//! from *waiting for the network*, and a table keyed on it alone would answer
//! one of them wrongly every time. **What the process reads from is what tells
//! them apart**, and that is why [`TheEvidence`] carries it.
//!
//! **And the symbol names are not stable across kernels.** `docs/features.md`
//! records a pipe reader as `wait_for_partner`; this kernel says
//! `anon_pipe_read`. Both are true, of different kernels. So this reads `wchan`
//! only where the name *describes* what is being waited for — a lock, a pipe, a
//! child — and never to identify the keyboard or the network, which rest on the
//! file instead. `docs/quirks.md` carries both findings.
//!
//! # Why nothing in the shell calls this yet, and why that is deliberate
//!
//! `docs/features.md` promises the five answers **and** four things to do about
//! them — *show the prompt*, *inspect the connection*, *reveal what is blocking
//! it*, *stop*. Those are a surface, and no surface asks this question yet.
//!
//! A `pub(crate)` method on the shell's terminal with nothing calling it was
//! written and then removed, because clippy refuses it and clippy is right to:
//! an uncalled private method is a claim that something is wired when it is
//! not. What is here instead is **public API of this crate** — complete,
//! measured against real processes, and reachable the moment something asks.
//!
//! The alternative was inventing a surface nobody designed in order to have a
//! caller, which is the failure `CLAUDE.md` names: *the cost of asking is a
//! message, and the cost of guessing wrong is a release nobody decided.*
//!
//! # The control is what makes *still computing* a measurement
//!
//! A running process is `R` with `wchan` `0`, observed twice from two different
//! programs. So **still computing is positive evidence**, not the absence of a
//! reason — which matters, because *we found nothing* and *it is working* are
//! the two readings a person must never have blended for them.

use crate::words;

/// What the thing a command is reading from turns out to be.
///
/// **The evidence that distinguishes the keyboard from the network**, which
/// `wchan` cannot: both are a generic wait. Taken from what `fd 0` points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatItReadsFrom {
    /// A terminal — so a person typing is what would release it.
    ATerminal,
    /// A socket: something on a network, or another machine.
    ASocket,
    /// A pipe: another program's output.
    APipe,
    /// An ordinary file, which does not block for long.
    AFile,
    /// Something else, or nothing that could be read.
    SomethingElse,
}

/// What a machine could find out about a command that looks stuck.
///
/// **Three readings, all unprivileged**, which is what makes this answerable
/// without a kernel boundary and why `docs/features.md` calls it the first
/// buildable of its group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheEvidence {
    /// Whether the kernel has it on a processor rather than parked.
    ///
    /// `R` in `/proc/PID/stat`. True is the positive evidence behind
    /// [`WhyItIsWaiting::StillComputing`].
    pub running: bool,
    /// The kernel function it is parked in, where one could be read.
    ///
    /// [`None`] when the kernel would not say — which is a real answer and not
    /// a failure, and is why this is an option rather than an empty string.
    pub kernel_says: Option<String>,
    /// What it reads from.
    pub reads_from: WhatItReadsFrom,
}

/// Why a command that looks stuck is waiting.
///
/// The five `docs/features.md` promises, and no sixth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyItIsWaiting {
    /// Waiting for somebody to type.
    ForTheKeyboard,
    /// Waiting for a network, or for another machine.
    ForTheNetwork,
    /// Waiting for another process, or for a lock one of them holds.
    ForAnotherProcess,
    /// Not waiting at all: it is working.
    StillComputing,
    /// Nothing here settles it, and saying so is the honest answer.
    ///
    /// **This exists because the mechanism produces it, not for appearances.**
    /// A `wchan` is a kernel symbol and any table written here is finite, so a
    /// kernel naming something this one has never seen lands here. `sleep` does
    /// too: a timer is none of the four, and calling it *still computing*
    /// would be a lie a person could act on.
    NobodyCanTell,
}

impl WhyItIsWaiting {
    /// The sentence a person reads.
    #[must_use]
    pub const fn the_word(self) -> alo_strings::Word {
        match self {
            Self::ForTheKeyboard => words::WAITING_FOR_THE_KEYBOARD,
            Self::ForTheNetwork => words::WAITING_FOR_THE_NETWORK,
            Self::ForAnotherProcess => words::WAITING_FOR_ANOTHER_PROCESS,
            Self::StillComputing => words::STILL_COMPUTING,
            Self::NobodyCanTell => words::NOBODY_CAN_TELL,
        }
    }

    /// Every answer, for a test that must cover all of them.
    pub const ALL: [Self; 5] = [
        Self::ForTheKeyboard,
        Self::ForTheNetwork,
        Self::ForAnotherProcess,
        Self::StillComputing,
        Self::NobodyCanTell,
    ];
}

/// The kernel functions whose **names describe what is being waited for**.
///
/// Only these, and deliberately only these. A symbol like `wait_woken` says a
/// process is parked and nothing about what would wake it, so it is not here —
/// it is answered by what the command reads from instead. A name that
/// describes a lock, a pipe or a child is evidence; a name that describes
/// *waiting* is not.
///
/// Matched by prefix, because kernels rename around the edges and
/// `locks_lock_inode_wait` is the same fact as a later `locks_lock_inode_wait2`
/// would be. The list is short so that a reader can check every entry against
/// the measurement in this file's header.
const WAITING_ON_ANOTHER_PROCESS: [&str; 4] = [
    // A file lock another process holds.
    "locks_lock_inode_wait",
    // Waiting for a child to finish.
    "do_wait",
    // A pipe with nothing in it, on this kernel.
    "anon_pipe_read",
    // The same fact under the name an older kernel used, kept because the
    // symbol is what changed and not the situation.
    "wait_for_partner",
];

/// The kernel functions that say a process is **parked and nothing more**.
///
/// These are the ones where what the command reads from is the only evidence
/// there is: `wait_woken` is a process asleep on a wait queue, and both a
/// terminal read and a socket read were measured in it. A name here means
/// *look at the file*; a name in neither list means *do not guess*.
const PARKED_AND_NOTHING_MORE: [&str; 2] = [
    // What a terminal read and a socket read both park in, on 7.0.0.
    "wait_woken",
    // The older spelling of the same situation.
    "wait_for_common",
];

/// Why this command is waiting, from what could be found out about it.
///
/// # The order of the questions, which is the whole of the logic
///
/// 1. **Running beats everything.** A process the kernel has on a processor is
///    not waiting for anything, whatever else is true of it.
/// 2. **A kernel name that describes what it waits for.** A lock, a pipe or a
///    child names itself; see [`WAITING_ON_ANOTHER_PROCESS`]. This comes before
///    the file deliberately: a command blocked on a lock usually still has a
///    terminal for its input, and *waiting for the keyboard* would send a
///    person to press a key that changes nothing.
/// 3. **A kernel name that says only *parked*, or no name at all** — then what
///    it reads from is the evidence, because the kernel has given none.
/// 4. **Otherwise nobody can tell.**
///
/// # Why step 4 is not *fall back to the file*
///
/// Because the file is then evidence of nothing. `sleep 30` is parked in
/// `hrtimer_nanosleep` and happens to have a pipe for its input; reading the
/// pipe would answer *waiting for another program*, which is a confident wrong
/// answer a person would act on by going to look for a program that does not
/// exist. The command is waiting for a **clock**, which is not one of the five,
/// so the fifth is the true one.
///
/// *This replaced a version that did fall through to the file, which answered
/// exactly that about `sleep`. The table was right about every case that had
/// been measured and wrong about the one that had.*
#[must_use]
pub fn why_it_is_waiting(evidence: &TheEvidence) -> WhyItIsWaiting {
    if evidence.running {
        return WhyItIsWaiting::StillComputing;
    }

    let names = |list: &[&str]| {
        evidence
            .kernel_says
            .as_deref()
            .is_some_and(|said| list.iter().any(|known| said.starts_with(known)))
    };

    if names(&WAITING_ON_ANOTHER_PROCESS) {
        return WhyItIsWaiting::ForAnotherProcess;
    }

    // The kernel said nothing, or said only that it is parked. Either way the
    // file is now the only evidence there is.
    if evidence.kernel_says.is_none() || names(&PARKED_AND_NOTHING_MORE) {
        return match evidence.reads_from {
            WhatItReadsFrom::ATerminal => WhyItIsWaiting::ForTheKeyboard,
            WhatItReadsFrom::ASocket => WhyItIsWaiting::ForTheNetwork,
            WhatItReadsFrom::APipe => WhyItIsWaiting::ForAnotherProcess,
            WhatItReadsFrom::AFile | WhatItReadsFrom::SomethingElse => {
                WhyItIsWaiting::NobodyCanTell
            }
        };
    }

    WhyItIsWaiting::NobodyCanTell
}

#[cfg(test)]
#[path = "why_a_command_is_waiting_tests.rs"]
mod tests;
