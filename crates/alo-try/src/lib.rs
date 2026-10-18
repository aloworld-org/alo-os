//! The word a person puts before a command to run it in a sealed box.
//!
//! `docs/features.md`, starred: *Try a command without letting it touch
//! anything. Paste the install script from a README and **run it**… A process
//! cannot credibly restrict itself — the kernel has to.*
//!
//! The owner chose the gesture on 2026-10-18: **a word before the command**,
//! plus alo offering when it sees a risk. This crate is the word. The offering
//! is the terminal's and is not here.
//!
//! ```text
//! $ try ./install.sh
//! ```
//!
//! | | |
//! |---|---|
//! | [`where_the_box_goes`] | the control group a person's box is made under, which `docs/quirks.md` had open |
//! | [`the_cgroup_this_process_is_in`] | the one it is made under, read from `/proc/self/cgroup` |
//! | [`the_record_a_person_keeps`] | where a person's own runs are written down |
//! | [`what_was_asked_for`] | the flags in front of the command, and the command after them |
//! | [`found_on_the_path`] | the command, resolved the way a shell resolves one |
//! | `src/main.rs` | the order: resolve, places, record, boundary, box, run |
//!
//! # It declares no words of its own
//!
//! Every sentence a person reads here is `alo_capability`'s — `NotRun::the_word`
//! already maps each refusal to one of four, and `failing.rs` explains at length
//! why those words are not `alo-running`'s either. This crate joins things up
//! for one process and says nothing a vocabulary has not got.
//!
//! # Why the box is made by the person's own process
//!
//! [ADR 0125](../../../docs/decisions/0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md)
//! decides that a short-lived process asks a long-lived service for the part
//! only a service can do. **Running a person's own command is not that part**,
//! and its amendment of 2026-10-18 says so with the measurements: the command's
//! output has to reach the terminal it was typed in, and a box the daemon
//! spawned inherits the *daemon's* stdio, so the person would see nothing. A
//! child of their own shell inherits their terminal and needs nothing.
//!
//! What the person's process needs and has, measured against `image/`:
//!
//! | needed | how it is had |
//! |---|---|
//! | the kernel boundary | `alo-boundaryd` runs as root and leaves it pinned under [`alo_bounding::THE_ROOT`]; the map of turns is mode `0660` to group `alo-agent`, and `sysusers.d/alo.conf` puts `alo` in that group. **Writing a grant needs permission on a file, which a person has** |
//! | somewhere to make a `domain` control group | a child of its own cgroup, which is theirs to write wherever systemd delegates — see [`where_the_box_goes`], whose first version got this wrong |
//! | somewhere to write the record | their own state folder, which is theirs to write |
//!
//! # And law 2 is not bent by any of it
//!
//! *The law binds the agent, never the person.* Nothing here is a verb, nothing
//! here is reachable by an agent, and the broker's list does not gain a name.
//! This is the person's own `exec`, which is the second half of law 2 — *running
//! code is a separate grant that only the person gives* — taken up by the
//! person themselves.

use std::path::{Path, PathBuf};

/// Where a person's own boxes are made: a child of the cgroup this process is
/// already in.
///
/// # This answers a question `docs/quirks.md` had left open, and the first
/// answer was wrong
///
/// That file says of the folder a box lives in: *where that folder is on a
/// machine running alo OS is still open — it depends on what systemd delegates
/// to the service, and nothing has measured it on a booted image yet.*
///
/// **Measured on a real kernel on 2026-10-18, by running it.** The first
/// version of this function composed a fixed path from the uid —
/// `user.slice/user-<uid>.slice/user@<uid>.service/alo-try` — on the strength of
/// a `mkdir` probe that succeeded there. It was wrong, and `alo-try` failed with
/// `Permission denied` from the helper, writing `cgroup.procs`.
///
/// **cgroup v2 does not ask whether you may write the destination. It asks
/// whether you may write the *common ancestor* of where the process is and
/// where it is going.** A command typed in a login session lives in
/// `session-N.scope`; the destination was under `user@<uid>.service`; their
/// common ancestor is `user-<uid>.slice`, which is `root root`. So the move was
/// refused even though both ends looked writable — which is why a `mkdir` probe
/// proved nothing.
///
/// A child of **this process's own** cgroup makes the common ancestor that
/// cgroup itself, so the question becomes the one delegation actually answers:
/// *is my own cgroup mine to write?* Measured inside a delegated subtree:
/// `mkdir` succeeds, `cgroup.type` is `domain` — which is what
/// `alo_running::WhereCommandsRun` needs — and writing `0` to the child's
/// `cgroup.procs` succeeds.
///
/// # What that asks of the machine
///
/// The unit a person's shell descends from has to be `Delegate=`, or its cgroup
/// is not theirs to write and `try` refuses. On alo OS that is
/// `alo-desktop.service`, which the image now says so for.
#[must_use]
pub fn where_the_box_goes(this_processes_cgroup: &Path) -> PathBuf {
    this_processes_cgroup.join("alo-try")
}

/// The cgroup this process is in, as a path under the cgroup filesystem.
///
/// Read from `/proc/self/cgroup`, whose cgroup v2 line is `0::<path>` with the
/// path rooted at the hierarchy. [`None`] on a machine with no cgroup v2 line,
/// which is a machine that cannot hold a box at all.
#[must_use]
pub fn the_cgroup_this_process_is_in(said: &str) -> Option<PathBuf> {
    let under = said
        .lines()
        .find_map(|line| line.strip_prefix("0::"))?
        .trim();
    // A process in the root cgroup answers `/`, and a box there is not a box
    // anybody delegated — so it is refused rather than made at the top of the
    // hierarchy.
    (under.starts_with('/') && under.len() > 1)
        .then(|| Path::new(THE_CGROUP_FILESYSTEM).join(under.trim_start_matches('/')))
}

/// Where the cgroup filesystem is mounted, which systemd fixes.
pub const THE_CGROUP_FILESYSTEM: &str = "/sys/fs/cgroup";

/// Where a person's own runs are written down.
///
/// **Their own file, under their own state folder**, which ADR 0125's amendment
/// settles and `docs/contracts/record-file.md` already prescribes the shape of:
/// *two files, because a record file has one writer.* That contract's own answer
/// to a new writer is a new file, and this is the fourth — written by the
/// person's own command rather than by a service.
///
/// **Not `/var/lib/alo/record.jsonl`.** That file has one writer and it is
/// `alo-agentd`. Appending to it from a second process is the thing the rule
/// forbids, and it is forbidden for a reason that has nothing to do with
/// permission: on a shipped machine the person and the daemon are the same
/// account, so the permission is there and the rule is still right.
///
/// The format is the same line-per-entry shape, so the tooling that reads a
/// record can read this one when it is pointed at it.
#[must_use]
pub fn the_record_a_person_keeps(home: &Path) -> PathBuf {
    home.join(".local")
        .join("state")
        .join("alo")
        .join("commands.jsonl")
}

/// What a person asked for in front of the command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowNarrow {
    /// `try ./install.sh` — the command and nothing of the person's.
    NothingOfTheirs,
    /// `try --only-here ./build.sh` — and the folder they are standing in.
    ///
    /// **A loosening**, which the flag's name does not suggest: it adds the
    /// folder to a box that had none of their files. What they are narrowing is
    /// their session down to one folder, not the box down from something.
    AndTheFolderTheyAreIn,
}

/// What the person asked for, and the command left after the flags.
///
/// **Flags are read only before the command.** `try --only-here ./build.sh
/// --only-here` passes the second one to the script, because everything after
/// the command is the command's — the rule every `--` taking program keeps, and
/// the one a person relies on when the thing they are running has flags of its
/// own.
///
/// [`None`] when nothing is left after the flags, which is a person who typed
/// only a flag.
#[must_use]
pub fn what_was_asked_for<A: AsRef<std::ffi::OsStr>>(
    arguments: &[A],
) -> Option<(HowNarrow, Vec<std::ffi::OsString>)> {
    let mut how = HowNarrow::NothingOfTheirs;
    let mut rest = arguments
        .iter()
        .map(|one| one.as_ref().to_owned())
        .peekable();
    while let Some(next) = rest.peek() {
        match next.to_str() {
            Some(ONLY_HERE) => {
                how = HowNarrow::AndTheFolderTheyAreIn;
                rest.next();
            }
            // **`--` ends the flags and is not passed on**, so a command whose
            // own name begins with a dash can still be run.
            Some("--") => {
                rest.next();
                break;
            }
            _ => break,
        }
    }
    let left: Vec<std::ffi::OsString> = rest.collect();
    (!left.is_empty()).then_some((how, left))
}

/// The flag a person types to be allowed to work where they are standing.
pub const ONLY_HERE: &str = "--only-here";

/// The command, resolved to a full path the way a shell resolves one.
///
/// A name with a separator in it is taken as a path and made absolute against
/// `from`, the folder the person was standing in. A bare name is looked for in
/// `path`, in order, and the first entry that holds a file wins.
///
/// [`None`] when there is nothing of that name, which is a person's own typo
/// and is answered with `capability.running.not-a-command`.
///
/// **Resolved here and not in `alo-running`**, because only this process knows
/// which folder the person was in — `alo_running::what_a_try_may_reach` refuses
/// a relative path for exactly that reason.
#[must_use]
pub fn found_on_the_path(named: &str, from: &Path, path: Option<&str>) -> Option<PathBuf> {
    let asked = Path::new(named);
    if named.contains('/') {
        let full = if asked.is_absolute() {
            asked.to_path_buf()
        } else {
            from.join(asked)
        };
        return full.is_file().then_some(full);
    }
    path?
        .split(':')
        .filter(|entry| !entry.is_empty())
        .map(|entry| Path::new(entry).join(named))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "lib_tests.rs"]
mod tests;
