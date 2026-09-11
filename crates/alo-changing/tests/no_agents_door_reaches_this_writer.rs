//! Nothing new can write the grants from an agent's door.
//!
//! `alo-agentd/src/rereading.rs` already shows the daemon's half of this:
//! nothing an agent can send over the socket writes a byte of the grants
//! file — the knock carries nothing, the daemon's answer to it only reads,
//! and an agent sending the knock is refused in words with the refusal in
//! the record. This crate is the writer that half was counting on nobody
//! adding behind its back, so it owes the same argument from the other side.
//!
//! What a behaviour test cannot show, a manifest can — the same way
//! `alo-clipboard` keeps a turn out of the clipboard. **The daemon cannot
//! name this crate**: no `alo-changing` in `alo-agentd`'s dependencies means
//! no road from anything the socket carries to [`alo_changing::Changing`],
//! whatever anybody writes inside the daemon tomorrow. And **this crate
//! cannot name the agent's side**: no daemon, no turn, no record, no
//! context, no asking — the person's half is the person's, and the one thing
//! it shares with the daemon is the file, which the daemon reads under its
//! own rules about who may have written it.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// A manifest of this repository, read.
fn manifest(crate_named: &str) -> String {
    let at = the_repository()
        .join("crates")
        .join(crate_named)
        .join("Cargo.toml");
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// Whether this manifest names this crate as a dependency — a line that opens
/// its table, rather than any mention of the name, because both manifests
/// argue in prose about the other side and a check should not be confused by
/// an argument.
fn depends_on(manifest: &str, crate_named: &str) -> bool {
    manifest
        .lines()
        .any(|line| line.trim_start().starts_with(&format!("{crate_named} = ")))
}

/// **The daemon cannot name this writer.** An agent's door that could reach
/// [`alo_changing::Changing`] would be an agent one bug away from writing its
/// own grants; there is no such road, and this is what fails the day somebody
/// opens one.
#[test]
fn the_daemon_does_not_depend_on_the_persons_writer() {
    assert!(
        !depends_on(&manifest("alo-agentd"), "alo-changing"),
        "alo-agentd depends on alo-changing: the socket now has a road to the \
         writer of the grants file, which rereading.rs promises it does not"
    );
}

/// **This writer cannot name the agent's side.** Nothing that ships here
/// reaches a daemon, a turn, a record, a context or a question — the person's
/// half is the person's, and the file is the only thing the two halves share.
#[test]
fn the_persons_writer_reaches_no_agents_door() {
    let ours = manifest("alo-changing");
    for elsewhere in [
        "alo-agentd",
        "alo-turn",
        "alo-record",
        "alo-context",
        "alo-asking",
        "alo-answering",
    ] {
        assert!(
            !depends_on(&ours, elsewhere),
            "alo-changing depends on {elsewhere}, and a person changing their \
             own grants is not an agent doing something"
        );
    }
}

/// **The knock this crate sends is the empty one.** The wire word is
/// `alo-protocol`'s `FromAPerson::Granted`, which has no field for a grant, a
/// path or a duration — so even the message this crate does send could not
/// widen anything if it arrived from somewhere else. Measured here rather
/// than believed: the line on the wire is exactly the one with nothing in it.
#[test]
fn the_knock_carries_nothing() {
    #[cfg(unix)]
    {
        let line = alo_protocol::FromAPerson::Granted
            .written()
            .expect("the knock writes");
        assert!(
            line.contains(r#""granted":{}"#),
            "the knock grew a payload: {line}"
        );
        assert_eq!(
            alo_protocol::FromAPerson::read(&line).expect("the knock reads"),
            alo_protocol::FromAPerson::Granted
        );
    }
}
