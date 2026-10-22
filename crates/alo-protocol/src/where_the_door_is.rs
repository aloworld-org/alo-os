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

/// Where a person's own words reach the agent, which is a different door from
/// the one above and is not one per person.
///
/// **Two doors, and the difference is who is behind them.** [`the_door_of`] is
/// the daemon's, one per signed-in person, and what arrives on it is the
/// protocol in this crate. This one is ADR 0109's: `alo-asking.socket` with
/// `Accept=yes`, which starts `alo-asking@.service` as `User=alo-agent` for each
/// connection, and what arrives on it is **a sentence a person wrote**, answered
/// with whatever the agent has to say. No framing, no verbs, one line in.
///
/// # Why its name is here
///
/// For the reason this module exists at all, which its header gives: a command a
/// person types has no caller to hand it a path. It was declared in
/// `alo-shell`, which is the compositor — and a command must not depend on the
/// compositor to learn where a socket is.
///
/// **`alo-shell` keeps its own `WHERE_WORDS_GO` and that is not a second copy
/// drifting.** Both are held to the same third thing, which is the unit that
/// actually opens the socket: the compositor's by
/// `crates/alo-shell/tests/the_composer_reaches_the_agent.rs`, the image's by
/// `crates/alo-image/tests/what_the_image_owes_the_daemons.rs`, and this one by
/// the test below, which reads the unit off the disk rather than restating the
/// path. Pointing the compositor at this constant instead would give
/// `alo-shell` a dependency on the protocol crate it has never had, which is a
/// change to the dependency graph and not this decision's to make.
pub const WHERE_A_PERSONS_WORDS_GO: &str = "/run/alo-asking/socket";

#[cfg(test)]
mod tests {
    use super::*;

    /// **The agent's door is the one the unit actually opens**, read off the
    /// unit rather than restated here.
    ///
    /// A constant asserted against a string somebody typed beside it proves only
    /// that two lines of one file agree. This reads
    /// `image/usr/lib/systemd/system/alo-asking.socket` and takes its
    /// `ListenStream`, so a unit that moved its socket fails this — which is the
    /// case that matters, because a command would then knock where nothing
    /// listens.
    #[test]
    fn a_persons_words_reach_the_agent_where_the_unit_listens() {
        let unit = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../image/usr/lib/systemd/system/alo-asking.socket");
        let text = std::fs::read_to_string(&unit).unwrap_or_default();
        assert!(
            !text.is_empty(),
            "no unit to read at {} — this assertion's subject has gone, which is the case it \
             exists to catch",
            unit.display()
        );
        let listens: Vec<&str> = text
            .lines()
            .filter_map(|line| line.trim().strip_prefix("ListenStream="))
            .collect();
        assert_eq!(
            listens,
            [WHERE_A_PERSONS_WORDS_GO],
            "the unit at {} opens somewhere this constant does not name",
            unit.display()
        );
        // **A component, not a substring.** `contains` was written here first and
        // failed on a correct constant: `/run/alo-asking/socket` contains
        // `/run/alo` because one path begins with the other's spelling, and
        // neither is beneath the other.
        assert!(
            !std::path::Path::new(WHERE_A_PERSONS_WORDS_GO).starts_with(THE_ROOT),
            "the two doors are different places: one per person under {THE_ROOT}, and the \
             agent's, which is not per person"
        );
    }

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
