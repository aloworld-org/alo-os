//! What a person missed is kept on this machine until they dismiss it, and is
//! never synced.
//!
//! *Never synced* is a promise a settings default cannot keep. A switch that is
//! off ships off and gets turned on; a server nobody configured gets
//! configured. The way it is kept here is that **there is nothing to sync**: a
//! notification never reaches a disk, a socket or a serialiser anywhere in this
//! crate, so there is no file to copy and no shape to send.
//!
//! That is a stronger promise than the plan asked for and a narrower one in
//! exactly one place, which this file states rather than leaves to be
//! discovered: the list lives in the session's own memory, so signing out
//! empties it as dismissing does. A machine nobody is signed into holds nothing
//! about who was trying to reach the person who owns it.
//!
//! The settings file is the only thing this crate writes, and it can hold two
//! keys: whether notifications are held, and the hours set aside. There is no
//! shape of `notifying.toml` that carries a title, a body or a sender —
//! `keeping.rs`'s own tests hold that, and this one holds the other half: the
//! crate opens nothing else at all.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod common;

use std::fs;
use std::path::Path;

use alo_notifying::{Missed, Waiting};

use common::{a_notification_titled, the_machines_words};

/// Every source file this crate ships, with its test modules cut off.
fn the_shipped_source() -> Vec<(String, String)> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    for entry in fs::read_dir(&src).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "testing.rs" {
            continue;
        }
        let text = fs::read_to_string(entry.path()).unwrap();
        let shipped = text.split("#[cfg(test)]").next().unwrap_or("").to_owned();
        found.push((name, shipped));
    }
    assert!(found.len() > 12, "this crate's source was not found");
    found
}

/// The lines of a file that are code rather than what somebody wrote about it.
fn the_code_of(text: &str) -> String {
    text.lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// **What was missed waits until the person dismisses it**, in the order it
/// arrived, one at a time or all at once.
#[test]
fn what_was_missed_waits_until_the_person_dismisses_it() {
    let mut missed = Missed::nothing();
    let first = missed.keeps(a_notification_titled("Anna Pärt"));
    missed.keeps(a_notification_titled("Bertrand"));
    missed.keeps(a_notification_titled("Clara"));
    assert_eq!(missed.how_many(), 3);

    assert!(missed.dismiss(first));
    assert_eq!(missed.how_many(), 2);
    let titles: Vec<&str> = missed
        .waiting()
        .iter()
        .map(|waiting| waiting.notification().title())
        .collect();
    assert_eq!(titles, ["Bertrand", "Clara"]);

    assert_eq!(missed.dismiss_everything(), 2);
    assert!(missed.is_empty());
    assert_eq!(
        missed.reads_as(&the_machines_words()).len(),
        1,
        "an empty list says so rather than saying nothing"
    );
}

/// **Nothing about the list is written down anywhere.** No serialiser, no
/// deserialiser, no path, no file, no socket: the shipped code of the file that
/// holds it names none of them, so there is nothing that could be copied off
/// this machine and nothing that survives a sign-out.
#[test]
fn nothing_about_what_was_missed_is_written_down() {
    let mut found = false;
    for (name, shipped) in the_shipped_source() {
        if name != "missed.rs" {
            continue;
        }
        found = true;
        let code = the_code_of(&shipped);
        for what in [
            "Serialize",
            "Deserialize",
            "serde",
            "Path",
            "PathBuf",
            "File",
            "fs::",
            "alo_kept",
            "TcpStream",
            "Socket",
            "http",
        ] {
            assert!(
                !code.contains(what),
                "src/missed.rs names `{what}`: what a person missed is never written down and \
                 never synced"
            );
        }
    }
    assert!(found, "src/missed.rs was not read");
}

/// **The whole crate opens nothing but the one settings file it keeps.** Every
/// byte this crate puts on a disk goes through `alo-kept`, at a path it is
/// handed, and the only question it asks a disk of its own is whether the
/// person's file is there.
#[test]
fn the_crate_opens_nothing_but_the_one_settings_file_it_keeps() {
    let mut asked_the_disk = Vec::new();
    for (name, shipped) in the_shipped_source() {
        let code = the_code_of(&shipped);
        for what in [
            "File::",
            "OpenOptions",
            "fs::write",
            "fs::read",
            "fs::remove",
            "fs::rename",
            "fs::create",
            "read_dir",
            "read_to_string",
        ] {
            assert!(
                !code.contains(what),
                "src/{name} names `{what}`: every file this crate keeps goes through alo-kept"
            );
        }
        if code.contains(".exists()") {
            asked_the_disk.push(name);
        }
    }
    assert!(
        asked_the_disk.is_empty() || asked_the_disk == vec!["keeping.rs".to_owned()],
        "the only place this crate asks a disk anything of its own is keeping.rs, and it is \
         {asked_the_disk:?}"
    );
}

/// **This crate reaches nothing that could carry a notification off the
/// machine.** Its whole dependency list is other parts of alo OS, `serde` for
/// two settings and `thiserror`; there is no client, no socket and no clock.
#[test]
fn it_depends_on_nothing_that_could_carry_a_notification_anywhere() {
    let at = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = fs::read_to_string(&at).unwrap();
    let mut inside = false;
    let mut named = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == "[dependencies]";
            continue;
        }
        if !inside || line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(name) = line.split_whitespace().next() {
            named.push(name.to_owned());
        }
    }
    for carrier in ["ureq", "socket2", "rustix", "zbus", "reqwest", "tokio"] {
        assert!(
            !named.iter().any(|it| it == carrier),
            "this crate depends on {carrier}, which could carry a notification off the machine"
        );
    }
    assert!(!named.is_empty(), "no dependency was read at all");
}

/// **A handle is this machine's and is never a sender's.** Two applications
/// that both called their notification `1` cannot dismiss each other's,
/// because nothing a sender wrote is ever what a dismissal names.
#[test]
fn a_handle_is_this_machines_and_never_a_senders() {
    let mut missed = Missed::nothing();
    let first = missed.keeps(a_notification_titled("Anna Pärt"));
    let second = missed.keeps(a_notification_titled("Bertrand"));
    assert_ne!(first, second);
    assert!(missed.dismiss(second));
    assert_eq!(missed.waiting().first().map(Waiting::id), Some(first));

    let third = missed.keeps(a_notification_titled("Clara"));
    assert_ne!(third, second, "a handle was reused after a dismissal");
    assert_eq!(missed.how_many(), 2);
}
