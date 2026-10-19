//! Where this machine's daemon listens, which is part of the protocol rather
//! than of any one client.
//!
//! The shape is ADR 0017's: **`/run/alo/<uid>/agentd.sock`**, one socket per
//! signed-in person, with two doors behind it told apart by `SO_PEERCRED` —
//! `docs/contracts/daemon-protocol.md` carries both halves.
//!
//! # Why it is here and not only in the daemon
//!
//! It was only in `alo-agentd::place`, which is right while every client is
//! handed the path by whoever started it: `alo-changing::TheDaemonsDoor`'s own
//! header says *the path is the caller's*, and `alo-shell` is the caller.
//!
//! `alo-ask` has no such caller. It is a command a person types, and nothing
//! hands it anything — and **nothing in this repository depends on
//! `alo-agentd`** (measured 2026-10-19: 0 of 122 crates), because it is the
//! service. So the choice was a second copy of the shape in the command or one
//! copy somewhere both can reach, and a second copy is the fault
//! `docs/quirks.md` keeps finding: a declared place and an actual place that
//! drift, which no green suite catches.
//!
//! This crate is where both can reach, and it is not an arbitrary choice —
//! `alo-agentd::place`'s own doc already said the socket's name *is part of
//! `docs/contracts/daemon-protocol.md`*, which is this crate's contract.
//!
//! # Nothing here touches a disk
//!
//! Three paths joined. Whether a socket is there, who owns it and what mode it
//! has are the daemon's to decide and `alo_agentd::place::Place` is what
//! decides them; a client learns only that nobody answered.

use std::path::{Path, PathBuf};

/// The directory every person's door goes in, which the image makes at boot.
pub const THE_ROOT: &str = "/run/alo";

/// The socket's name, inside that person's directory.
pub const THE_SOCKET: &str = "agentd.sock";

/// This person's door beneath this root.
///
/// `person` is a login number as the kernel reports it. Nothing is made or
/// looked at.
#[must_use]
pub fn the_door_beneath(root: &Path, person: u32) -> PathBuf {
    root.join(person.to_string()).join(THE_SOCKET)
}

/// This person's door on a real machine.
#[must_use]
pub fn the_door_of(person: u32) -> PathBuf {
    the_door_beneath(Path::new(THE_ROOT), person)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shape ADR 0017 settled, and the one the contract documents.
    #[test]
    fn a_persons_door_is_their_login_number_under_the_root() {
        assert_eq!(
            the_door_of(1000),
            Path::new("/run/alo/1000/agentd.sock"),
            "the shape in docs/contracts/daemon-protocol.md"
        );
    }

    /// **The root is a parameter**, because a test cannot bind in `/run` and a
    /// rule about a socket is only a rule with a test if a test can make one.
    #[test]
    fn the_same_shape_holds_somewhere_a_test_may_write() {
        assert_eq!(
            the_door_beneath(Path::new("/tmp/alo-test"), 1000),
            Path::new("/tmp/alo-test/1000/agentd.sock")
        );
    }

    /// Two people have two doors, which is what *one per signed-in person*
    /// means.
    #[test]
    fn two_logins_are_two_doors() {
        assert_ne!(the_door_of(1000), the_door_of(1001));
    }
}
