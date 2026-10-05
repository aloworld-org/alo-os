//! Where this person's daemon listens, named once.
//!
//! [`crate::TheDaemonsDoor`] deliberately holds a path it is handed and names
//! none: *a rule about a socket is only a rule with a test if a test can bind
//! one somewhere it may write, and `/run` is not that.* That argument is about
//! the type, and it leaves a gap this module fills — **somebody still has to
//! know where the real one is**, and until this file existed nothing in
//! production did, so no surface could build a door at all.
//!
//! The shape is ADR 0017's and `alo-agentd`'s `Place` is where a daemon gets
//! it from: `/run/alo` is every person's door, and beneath it a directory per
//! person holding `agentd.sock`. It is written again here rather than taken
//! from that crate because `alo-agentd` is a Linux-only service crate that
//! brings a kernel-credentials dependency with it, and a surface that wants one
//! path should not link a daemon to get it. [`the_door_beneath`] is the whole
//! of the duplication, and `the_shape_is_the_daemons_own` in this file's own
//! test module is what holds the two spellings to each other — named in prose
//! rather than linked, because a `cfg(test)` item is not there to be linked to
//! when the documentation is built.
//!
//! # The person is asked of the kernel, in the one place that asks
//!
//! `alo_remembering::this_person` is that place, and its own note says why a
//! second `geteuid` would be wrong: every per-person path on a machine is keyed
//! by this number, and two readings of it are two different people as far as a
//! `0700` directory is concerned. So [`this_persons_door`] asks that, and this
//! file contains no uid of its own.

use std::path::{Path, PathBuf};

/// The directory every person's door goes in (ADR 0017), which the image makes.
pub const THE_ROOT: &str = "/run/alo";

/// The socket inside a person's own directory.
pub const THE_SOCKET: &str = "agentd.sock";

/// The door of the person whose number this is, beneath this root.
///
/// Three paths joined and nothing looked at, so a test can be given a root it
/// may write in — which is the same reason [`crate::TheDaemonsDoor`] takes a
/// path rather than finding one.
#[must_use]
pub fn the_door_beneath(root: &Path, uid: u32) -> PathBuf {
    root.join(uid.to_string()).join(THE_SOCKET)
}

/// **This** process's own door, on a real machine.
///
/// [`the_door_beneath`] with the root the image makes and the person **asked of
/// the kernel** through `alo_remembering::this_person`. A caller that built this
/// path out of an `Environment=` line would be answering *whose session is
/// this* from a variable, which `crates/alo-desktop`'s header forbids.
#[must_use]
pub fn this_persons_door() -> PathBuf {
    the_door_beneath(Path::new(THE_ROOT), alo_remembering::this_person())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The spelling is the daemon's own**, written out so that a change to
    /// either side is a failing test rather than a surface knocking at a socket
    /// nothing binds.
    ///
    /// `alo-agentd`'s `place.rs` holds `THE_ROOT = "/run/alo"` and
    /// `THE_SOCKET = "agentd.sock"`, and joins them as
    /// `root/<uid>/agentd.sock`. This crate does not depend on it — see this
    /// module's header — so the agreement is asserted rather than compiled.
    #[test]
    fn the_shape_is_the_daemons_own() {
        assert_eq!(THE_ROOT, "/run/alo");
        assert_eq!(THE_SOCKET, "agentd.sock");
        assert_eq!(
            the_door_beneath(Path::new(THE_ROOT), 1000),
            Path::new("/run/alo/1000/agentd.sock")
        );
    }

    /// **Two people are two doors**, which is the whole reason the uid is in
    /// the path: one socket for a machine would be the fault
    /// `alo_remembering::whose` exists to have fixed, in another file.
    #[test]
    fn two_people_do_not_share_a_door() {
        let root = Path::new("/run/alo");
        assert_ne!(the_door_beneath(root, 1000), the_door_beneath(root, 1001));
    }
}
