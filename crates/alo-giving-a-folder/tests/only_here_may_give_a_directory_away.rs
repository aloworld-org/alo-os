//! **This crate is the only place in the repository that changes a file's
//! owner**, and this is what holds it to that.
//!
//! ADR 0119 promised this test in the sentence that chose a component: *one
//! component must gain it, narrowly, **with a test holding it to that**.* A
//! capability granted narrowly and then used from a second place is a capability
//! granted widely, with a decision document saying otherwise.
//!
//! # What it reads, and what it cannot
//!
//! It walks every crate's `src` and refuses a `chown` that changes an **owner**
//! anywhere but here.
//!
//! **A bare `chown` cannot be what is looked for**, because this repository
//! makes a legitimate one: `alo-sessiond`'s `unix.rs` does
//! `chown(path, None, Some(group))` and explains why it may — changing only the
//! group is permitted without any capability when the process is already in
//! that group, which is how a socket root created becomes one the sign-in
//! surface can connect to. Forbidding `chown` outright would fail on correct
//! code, which is how a guard gets weakened by whoever meets it next.
//!
//! So what is looked for is the **owner argument**, which is the same shape
//! `alo-sessiond`'s own `this_crate_cannot_do_what_its_unit_forbids.rs` uses
//! after it missed exactly this on 2026-10-16 — it looked for `.uid(`, `.gid(`
//! and `setuid`, and caught an owner-changing `chown` only by accident.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The one crate that may.
const THE_ONE_PLACE: &str = "alo-giving-a-folder";

/// Whether a line asks the kernel to change a file's **owner**.
///
/// The permitted shape anywhere in this repository is
/// `chown(path, None, Some(group))` — the group only. A call whose owner
/// argument is anything but `None` is asking for `CAP_CHOWN`, which ADR 0119
/// granted to exactly one unit.
fn changes_an_owner(code: &str) -> bool {
    code.contains("chown(") && !code.contains(", None,")
}

/// Whether a line says something to a reader rather than to the machine.
fn is_a_comment(line: &str) -> bool {
    line.trim_start().starts_with("//")
}

/// Which lines of a file are inside a `#[cfg(test)]` module.
///
/// **The subject of this test is what ships, and a test fixture is not that.**
/// `alo-agentd` chowns a socket to the agent in two `#[cfg(test)]` modules, as
/// a fixture, and those tests run as root — it claims no capability in anything
/// installed on a machine. A guard that counted them would be asking a crate to
/// justify a power it does not hold, and the only way to satisfy it would be to
/// make the test worse.
///
/// *The first version of this test did count them, and that is how they were
/// found.* Checking whether they were production code took one look and is the
/// difference between a hole in ADR 0119 and a false positive here.
///
/// Counted by braces from the `cfg(test)` marker rather than by skipping to the
/// end of the file: a module is a region, and production code after one is
/// still production code.
fn inside_a_test_module(written: &str) -> Vec<bool> {
    let mut inside = Vec::with_capacity(written.lines().count());
    let mut depth = 0_i32;
    let mut marked = false;

    for line in written.lines() {
        if !is_a_comment(line) && line.contains("cfg(test)") {
            marked = true;
        }
        let opens = i32::try_from(line.matches('{').count()).unwrap_or(0);
        let closes = i32::try_from(line.matches('}').count()).unwrap_or(0);

        if marked && depth == 0 && opens > 0 {
            // The module's own brace: everything until it closes is a test.
            marked = false;
            depth = opens - closes;
            inside.push(true);
            continue;
        }
        if depth > 0 {
            inside.push(true);
            depth += opens - closes;
            continue;
        }
        inside.push(false);
    }
    inside
}

/// The repository's own root.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("this crate sits in the repository it is part of")
}

/// Every `.rs` file under a directory.
fn every_source(at: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            every_source(&path, into);
        } else if path.extension().is_some_and(|it| it == "rs") {
            into.push(path);
        }
    }
}

/// **The matcher catches an owner change and permits a group change.**
///
/// A source test that finds nothing has two indistinguishable outcomes, and
/// here the permitted form is the same function as the forbidden one with a
/// different argument — so this control matters more than usual.
#[test]
fn the_matcher_tells_the_two_chowns_apart() {
    for forbidden in [
        "rustix::fs::chown(at, Some(rustix::fs::Uid::from_raw(owner)), None)",
        "chown(path, Some(uid), Some(gid))?;",
        "        fs::chown(&at, Some(person), None).map_err(…)",
    ] {
        assert!(
            changes_an_owner(forbidden),
            "a call that changes an owner was not caught: {forbidden}"
        );
    }
    for permitted in [
        // What `alo-sessiond`'s `unix.rs` really does, and may.
        "rustix::fs::chown(path, None, Some(rustix::fs::Gid::from_raw(group)))?;",
        "chown(&socket, None, Some(ours))",
    ] {
        assert!(
            !changes_an_owner(permitted),
            "a group-only chown was called an owner change, which would fail correct code: \
             {permitted}"
        );
    }
    for ordinary in [
        "/// The group this process is in, which is the group its unit named.",
        "let owner = fs::metadata(&at)?.uid();",
        "pub fn what_to_make(under: &Path, person: u32) -> [ToMake; 2] {",
    ] {
        assert!(
            !changes_an_owner(ordinary),
            "an ordinary line was called an owner change: {ordinary}"
        );
    }
}

/// **Only this crate changes a file's owner.**
#[test]
fn only_this_crate_changes_a_files_owner() {
    let root = the_repository();
    let crates = root.join("crates");

    let mut elsewhere = Vec::new();
    let mut read = 0_usize;
    let mut here = 0_usize;

    for entry in std::fs::read_dir(&crates).expect("this repository has crates") {
        let at = entry.expect("a directory entry").path();
        if !at.is_dir() {
            continue;
        }
        let whose = at
            .file_name()
            .expect("a directory has a name")
            .to_string_lossy()
            .into_owned();

        let mut sources = Vec::new();
        every_source(&at.join("src"), &mut sources);
        for file in sources {
            let Ok(written) = std::fs::read_to_string(&file) else {
                continue;
            };
            read += 1;
            let in_a_test = inside_a_test_module(&written);
            for (number, line) in written.lines().enumerate() {
                if is_a_comment(line) || !changes_an_owner(line) {
                    continue;
                }
                if in_a_test.get(number).copied().unwrap_or(false) {
                    continue;
                }
                if whose == THE_ONE_PLACE {
                    here += 1;
                    continue;
                }
                elsewhere.push(format!(
                    "{}:{}  {}",
                    file.strip_prefix(&root).unwrap_or(&file).display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }

    // **The control**: a walk that read nothing would find nothing.
    assert!(
        read > 500,
        "only {read} source files were read, so this test is reading less of the repository \
         than it thinks"
    );

    // **And the other control**: this crate really does change an owner, so the
    // matcher is known to fire on the code it is written for.
    assert_eq!(
        here, 1,
        "{THE_ONE_PLACE} changes an owner on {here} lines and should on exactly one — if it is \
         zero this test cannot fail for the right reason"
    );

    assert!(
        elsewhere.is_empty(),
        "a crate other than {THE_ONE_PLACE} changes a file's owner:\n  {}\n\n\
         **Changing an owner needs `CAP_CHOWN`**, and ADR 0119 granted it to exactly one unit, \
         `alo-giving-a-folder@.service`, chosen by measurement: `CAP_CHOWN` is necessary and on \
         its own sufficient, and the two components already running as root are 113,587 and \
         2,160 lines against this one's hundred. A second place that changes an owner is either \
         a second unit that needs the capability — which is a decision and a new record — or \
         code that will fail on a booted machine with `EPERM`, the way `Command::uid()` did on \
         2026-10-14.",
        elsewhere.join("\n  ")
    );
}

/// **The test-module skip finds the region and nothing more.**
///
/// A skip that was too greedy would blind this guard quietly, which is the
/// worst way for a source test to fail — so it is measured on a file shaped
/// like the ones it reads: production code, a test module that chowns, and more
/// production code after it.
#[test]
fn the_skip_covers_the_test_module_and_not_what_follows() {
    let written = "\
pub fn ships(at: &Path) -> std::io::Result<()> {
    chown(at, Some(uid), None)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_fixture() {
        chown(&at, Some(AN_AGENT), Some(AN_AGENT)).unwrap();
    }
}

pub fn also_ships(at: &Path) -> std::io::Result<()> {
    chown(at, Some(uid), None)
}
";
    let inside = inside_a_test_module(written);
    let lines: Vec<&str> = written.lines().collect();

    for (number, line) in lines.iter().enumerate() {
        let is_inside = inside.get(number).copied().unwrap_or(false);
        if line.contains("AN_AGENT") {
            assert!(is_inside, "the fixture's chown was not skipped: {line}");
        }
        if line.contains("Some(uid)") {
            assert!(
                !is_inside,
                "a shipping chown was skipped as though it were a test: {line}"
            );
        }
    }

    // And both shipping ones are found, so the skip did not swallow the file.
    let found = lines
        .iter()
        .enumerate()
        .filter(|(number, line)| {
            changes_an_owner(line) && !inside.get(*number).copied().unwrap_or(false)
        })
        .count();
    assert_eq!(found, 2, "the two shipping chowns were not both found");
}
