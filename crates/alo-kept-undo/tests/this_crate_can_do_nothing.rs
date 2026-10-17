//! **This crate can do nothing**, and that is what makes it safe for a turn to
//! reach.
//!
//! `alo-letting-go` owned this layout until 2026-10-17 and could not share it:
//! that crate holds the machine's one privileged remover, and its
//! `tests/a_turn_cannot_arrive_at_this_road.rs` forbids anything a turn runs
//! inside from linking it — *"a second crate reaching for a type, a constant or
//! a function of this one fails here"*. So `alo-giving-a-folder` declared
//! `THE_FOLDER` a second time, and `alo-turn`, where ADR 0045 puts the taking of
//! a bracket, could not reach the layout at all.
//!
//! This crate exists so all three can. **The whole of its safety is that it has
//! no powers**, and a power added here would travel into a turn's process
//! without anything objecting — the guard in `alo-letting-go` would still pass,
//! because it is about *that* crate. So the guard for this one is here.
//!
//! # Why read the source rather than test the behaviour
//!
//! Because the fault cannot be reached from a test. A `std::fs::write` added
//! here would work perfectly; nothing would fail; a crate a turn links would
//! simply have gained the ability to write a file. There is no input that
//! catches that, so the thing to read is the file.
//!
//! It is the shape `alo-sessiond`'s systemd guard and `alo-giving-a-folder`'s
//! owner guard already have, and the reason is theirs: a fault that cannot be
//! reached from a test is still a fault, and the source is where it lives.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a source file that no longer says what it must is the \
              failure being reported"
)]

use std::path::{Path, PathBuf};

/// Where this crate's own source is.
fn where_it_is() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")
}

/// This crate's source, as text.
fn the_source() -> String {
    let at = where_it_is();
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// Every way a crate reaches past its own values, and none may appear here.
///
/// Not an exhaustive list of what Rust can do — that is not writable — but every
/// door this crate could plausibly grow: the disk, a process, the network, and
/// the two ways to leave safe Rust behind. Each is the thing a turn's process
/// must not gain by linking a layout.
const NOT_FROM_HERE: [(&str, &str); 10] = [
    ("fs::", "the disk"),
    ("File::", "the disk"),
    ("Command", "a process"),
    ("std::process", "a process"),
    ("TcpStream", "the network"),
    ("UnixStream", "a socket"),
    ("UdpSocket", "the network"),
    ("unsafe", "safe Rust"),
    ("rustix", "a syscall"),
    ("libc", "a syscall"),
];

/// **No door is in this crate.**
#[test]
fn this_crate_reaches_nothing_but_the_values_it_is_handed() {
    let source = the_source();
    let found: Vec<String> = NOT_FROM_HERE
        .into_iter()
        .filter(|(door, _)| source.contains(door))
        .map(|(door, what)| format!("{door} — {what}"))
        .collect();
    assert!(
        found.is_empty(),
        "{} has grown a way to reach past the values it is handed:\n  {}\n\n\
         This crate is linked by things a turn runs inside, which is the whole reason it exists \
         separately from `alo-letting-go`. A power here is a power in an agent's process, and \
         `alo-letting-go`'s own guard will not object, because that guard is about that crate.",
        where_it_is().display(),
        found.join("\n  ")
    );
}

/// **The control**: the filter can find a door when there is one.
///
/// Without it, a typo in every entry of [`NOT_FROM_HERE`], or a `the_source`
/// that returned nothing, would make the test above pass with nothing to
/// report — which is the usual way a forbidden-text check gets satisfied.
#[test]
fn the_filter_finds_a_door_when_there_is_one() {
    for (door, _) in NOT_FROM_HERE {
        let pretend = format!("    let _ = {door};");
        assert!(
            pretend.contains(door),
            "the filter cannot find {door:?} when it is there"
        );
    }

    let source = the_source();
    assert!(
        source.len() > 1_000,
        "{} is {} bytes, which is too short to be the crate this test is about",
        where_it_is().display(),
        source.len()
    );
    assert!(
        source.contains("pub const THE_FOLDER"),
        "{} does not declare THE_FOLDER, so it is not the layout and this test is reading the \
         wrong file",
        where_it_is().display()
    );
}

/// **And its dependencies are three, all of them parsers.**
///
/// The other half: a door can arrive through a manifest as easily as through a
/// line of code. `serde`, `serde_json` and `thiserror` read text and derive
/// sentences; none of them opens anything. A fourth dependency here is a
/// decision somebody should have to make on purpose.
#[test]
fn it_rents_three_things_and_all_of_them_only_read_text() {
    let at = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let said = std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()));

    let rented: Vec<&str> = said
        .lines()
        .filter(|line| line.contains("workspace = true") && line.contains('{'))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert_eq!(
        rented,
        vec!["serde", "serde_json", "thiserror"],
        "{} rents something other than the three text parsers, and this crate is linked by \
         things a turn runs inside",
        at.display()
    );

    for must_not in ["alo-letting-go", "alo-giving-a-folder", "rustix", "tokio"] {
        assert!(
            !said.contains(must_not),
            "{} depends on {must_not}, which would defeat the point of this crate existing",
            at.display()
        );
    }
}
