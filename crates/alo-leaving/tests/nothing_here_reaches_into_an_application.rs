//! Nothing here kills an application, and nothing here reads one's own files.
//!
//! *Never kills one silently* is a promise that is easy to keep badly: a
//! countdown, then a signal, and a log sentence nobody reads. The way it is kept
//! here is that **this crate cannot kill anything at all** — it has no road to a
//! process, no signal, no child, and no dependency that could lend it one. What
//! happens to an application that would not close is that the person is shown
//! its name and decides, and the session then ends over it if they say so.
//!
//! So this reads the crate's own shipped source and its own manifest, which is
//! the only form of that promise a later change cannot quietly undo.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::Path;

/// Everything this crate depends on, exactly.
///
/// A closed list rather than a search for a name: the way a crate about logging
/// out gains the ability to kill a process is by gaining a dependency that can,
/// and every one of those has a different name. Adding anything here is a
/// deliberate act with this test in front of it.
const EVERYTHING_IT_DEPENDS_ON: [&str; 13] = [
    "alo-locking",
    "alo-overlay",
    "alo-accounts",
    "alo-greeting",
    "alo-applications",
    "alo-appearance",
    "alo-dividing",
    "alo-kept",
    "alo-strings",
    "serde",
    "thiserror",
    "alo-saying",
    "toml",
];

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
    assert!(found.len() > 8, "this crate's source was not found");
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

/// This crate's manifest, and every dependency line in it that is not a
/// comment.
fn what_it_depends_on() -> Vec<String> {
    let at = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = fs::read_to_string(&at).unwrap();
    let mut named = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line.contains("dependencies");
            continue;
        }
        if !inside || line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(name) = line.split_whitespace().next() {
            named.push(name.to_owned());
        }
    }
    assert!(!named.is_empty(), "no dependency was read at all");
    named
}

/// **Nothing in this crate's code can reach a process.** Not a command, not a
/// signal, not a child, not a kill — in any spelling.
#[test]
fn nothing_in_this_crate_can_reach_a_process() {
    let never = [
        "std::process",
        "Command",
        "kill",
        "Kill",
        "SIGTERM",
        "SIGKILL",
        "signal",
        "Signal",
        "libc",
        "abort",
        "exit(",
        "Child",
        "terminate",
    ];
    for (name, shipped) in the_shipped_source() {
        let code = the_code_of(&shipped);
        for what in never {
            assert!(
                !code.contains(what),
                "src/{name} names `{what}` in its code: logging out asks, and never kills"
            );
        }
    }
}

/// **It depends on nothing that could kill a process for it**, and the list of
/// what it depends on is closed.
#[test]
fn it_depends_on_nothing_that_could_kill_a_process() {
    let mut depends = what_it_depends_on();
    let mut expected: Vec<String> = EVERYTHING_IT_DEPENDS_ON
        .iter()
        .map(|it| (*it).to_owned())
        .collect();
    depends.sort();
    expected.sort();
    assert_eq!(
        depends, expected,
        "this crate's dependencies changed: a crate about leaving that can reach a process is how \
         *never kills one silently* stops being true"
    );
}

/// **It opens nothing but the one file it keeps.** No application's own session
/// file is read or written here: everything on a disk goes through `alo-kept`,
/// at a path this crate is handed, and the only place in the shipped code that
/// asks the disk anything of its own is the one question *is this person's file
/// there* — which decides whether a log-out leaves them without one.
#[test]
fn it_opens_nothing_but_the_one_file_it_keeps() {
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
    assert_eq!(
        asked_the_disk,
        vec!["keeping.rs".to_owned()],
        "the one question this crate asks a disk of its own is in keeping.rs"
    );
}
