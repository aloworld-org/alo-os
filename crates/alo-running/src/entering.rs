//! Putting this process inside a box, and then becoming the command.
//!
//! This is what `alo-enter-the-box` does, as a function so that it can be
//! tested without being a program. ADR 0117 is why the admission is an ordinary
//! program at all: `pre_exec` would do it in one process, and it is `unsafe`,
//! which `CLAUDE.md` forbids and the workspace sets to `forbid` so that no
//! attribute can lift it. `CommandExt::exec` is safe, because it replaces the
//! running program rather than running inside a forked copy of one.
//!
//! # The order is the whole of the guarantee
//!
//! ```text
//! 1. write "0" into the box's cgroup.procs   <- this process is now inside
//! 2. exec the command                        <- the command starts inside
//! ```
//!
//! Between 1 and 2 the only thing running is this program, which does nothing
//! but step 2. **The command a person approved starts after the admission and
//! never before it**, and that is the sentence the whole design exists to be
//! able to say.
//!
//! Nothing here runs after a successful `exec`, because there is no *here* left
//! to run in — the process is the command. So the only paths out of
//! [`enter_the_box_and_become`] are failures, and its return type says so.
//!
//! # `"0"` rather than this process's own number
//!
//! cgroup v2 reads `"0"` in `cgroup.procs` as *the process writing this*.
//! Measured on 2026-10-16 on `cgroup2fs`: a process that writes it is
//! afterwards in that group, read back from `/proc/self/cgroup`. It needs no
//! buffer to format and no second call to learn, which is worth having even
//! here where allocating would be allowed.
//!
//! # The box it is handed is checked, and comparing the paths as written is not
//! enough
//!
//! One argument decides where this process puts itself, so
//! [`enter_the_box_and_become`] refuses a box that is not under `inside`.
//!
//! **`Path::starts_with` is the wrong way to ask.** It compares components as
//! they are written, so `inside/../somewhere-else` starts with `inside` and
//! passes — and so does any symbolic link under `inside` that points out of it.
//! This was written that way first and the test
//! `a_box_that_climbs_out_is_refused` caught it: the refusal it expected never
//! happened, and the write was attempted on a path outside the service.
//!
//! So both paths are resolved by the machine before they are compared, which
//! answers `..` and symbolic links together because neither survives being
//! resolved. It is `alo-bounding`'s rule about `place_of` arriving in a
//! different crate for the same reason: *a granted folder reached through a
//! symbolic link is the folder rather than the link*, and anything that decides
//! about a path by reading it has not asked the machine.

use std::{ffi::OsStr, fs, os::unix::process::CommandExt as _, path::Path, process::Command};

/// What `cgroup.procs` is called, which is the file a process is admitted
/// through.
///
/// Named here rather than written twice, because the whole mechanism is one
/// write into this one file.
pub const THE_PROCESSES: &str = "cgroup.procs";

/// What cgroup v2 reads as *the process writing this*.
pub const MEANING_ME: &str = "0";

/// Puts this process in `box_at`, then becomes `program`.
///
/// `inside` is the folder every box of this service's lives under, and `box_at`
/// has to be beneath it. Both are taken so that the check is made here, by the
/// program that acts on the answer, rather than trusted from a caller.
///
/// # Returns
///
/// Only ever an error. On success this process **is** `program` and there is
/// nothing left to return to — which is why this returns
/// [`std::io::Error`] rather than a result, in the shape
/// [`std::os::unix::process::CommandExt::exec`] already has.
pub fn enter_the_box_and_become(
    inside: &Path,
    box_at: &Path,
    program: &Path,
    arguments: &[impl AsRef<OsStr>],
) -> std::io::Error {
    // Before anything is written: the one argument that decides where this
    // process goes has to name somewhere this service owns. Both sides are
    // resolved first, because `..` and a symbolic link both survive a
    // comparison of the paths as written.
    let home = match inside.canonicalize() {
        Ok(home) => home,
        Err(why) => return why,
    };
    let here = match box_at.canonicalize() {
        Ok(here) => here,
        Err(why) => return why,
    };
    if !here.starts_with(&home) {
        return std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "{} is {}, which is not a box under {}, and nothing else may be entered",
                box_at.display(),
                here.display(),
                home.display()
            ),
        );
    }

    // Step one. After this write, every open this process makes is the
    // kernel's decision — including the open `exec` is about to make.
    //
    // **Each step says which it was.** On a booted machine on 2026-10-31 every
    // `try` ended *did not start inside …: Permission denied*, and that one
    // sentence could not say whether the box refused this process or the box,
    // once joined, refused the command. They are different faults in different
    // crates. The kind is kept, so a caller deciding on it decides as before.
    if let Err(why) = fs::write(here.join(THE_PROCESSES), MEANING_ME) {
        return std::io::Error::new(why.kind(), the_box_would_not_be_joined(&here, &why));
    }

    // Step two, and there is no step three. `exec` returns only on failure.
    let why = Command::new(program).args(arguments).exec();
    std::io::Error::new(why.kind(), the_command_would_not_start(program, &why))
}

/// What a refusal to join the box says: this process never got in.
pub(crate) fn the_box_would_not_be_joined(here: &Path, why: &std::io::Error) -> String {
    format!(
        "the box at {} would not take this process, so nothing was run: {why}",
        here.display()
    )
}

/// What a refusal after joining says: the box was entered, and from inside it
/// the command could not be started.
pub(crate) fn the_command_would_not_start(program: &Path, why: &std::io::Error) -> String {
    format!(
        "the box was joined, and from inside it {} could not be started: {why}",
        program.display()
    )
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "entering_tests.rs"]
mod tests;
