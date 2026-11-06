//! **Whether the sealed box can run programs on this computer yet.**
//!
//! Measured on a booted alo OS on 2026-11-02: on a machine whose `/` is a
//! composefs image, the kernel boundary refuses the system layer's own checks
//! — following a link, reading a program's capability attribute when it is
//! started, listing a folder, looking a name up for the first time — so a
//! boxed command cannot start any program from `/usr`, or anything reached
//! through the `/home` link. A static program at its real `/var/home/…` path
//! ran boxed; nothing from the system layer did.
//!
//! **The owner's decision of 2026-11-06: "say so plainly, keep going."** The
//! box will be allowed to *read* the system layer, read-only, and that is
//! kernel work planned on its own. Until then a person is told in plain words
//! — `alo-try` when a boxed start is refused after joining, Settings under the
//! sealed box — and **nothing chosen is changed**: the sealed box stays the
//! default and stays the person's to choose (law 5).
//!
//! What this file answers is the half that can be read without running
//! anything: whether `/` is a composefs image. It is never the whole reason on
//! its own in `alo-try`, which also needs the box to have refused the start
//! after it was joined; Settings has no start to look at, and says *not on
//! this computer yet* of every machine whose root is composefs, which is every
//! installed alo OS.

/// Where this process's mounts are read from.
pub const THE_MOUNTS: &str = "/proc/self/mountinfo";

/// Whether the root in `mountinfo` — the text of [`THE_MOUNTS`] — is a
/// composefs image.
///
/// composefs mounts `/` as an overlay whose source is `composefs`, and a
/// kernel that mounts composefs as its own filesystem names it so; both are
/// read as yes. Anything else, including a mountinfo with no root at all, is
/// no.
#[must_use]
pub fn the_root_is_composefs(mountinfo: &str) -> bool {
    mountinfo.lines().any(|line| {
        let Some((mounted, described)) = line.split_once(" - ") else {
            return false;
        };
        // The fifth field is where it is mounted.
        if mounted.split(' ').nth(4) != Some("/") {
            return false;
        }
        let mut description = described.split(' ');
        let kind = description.next();
        let source = description.next();
        kind == Some("composefs") || (kind == Some("overlay") && source == Some("composefs"))
    })
}

/// Whether the sealed box can run programs on this computer yet, read from
/// [`THE_MOUNTS`]. A machine whose mounts cannot be read is not said to be one
/// where it cannot: the plain line is for what was read, never a guess.
#[must_use]
pub fn the_sealed_box_runs_programs_here() -> bool {
    match std::fs::read_to_string(THE_MOUNTS) {
        Ok(mounts) => !the_root_is_composefs(&mounts),
        Err(_) => true,
    }
}

#[cfg(test)]
#[path = "the_sealed_box_here_tests.rs"]
mod tests;
