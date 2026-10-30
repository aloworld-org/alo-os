//! What this privileged component may say to systemd, read off its own source.
//!
//! `crates/alo-sessiond/src/starting_their_units.rs` declares `EVERY_METHOD` as the
//! whole list of what it calls, and its own `call` refuses anything not on it.
//! That guard is real at run time; **this is the check that the list stays
//! short**, because a later change could add a method to the constant as easily
//! as to the code beneath it.
//!
//! [ADR 0121](../../../docs/decisions/0121-the-thing-that-opens-a-session-makes-the-persons-folder.md)
//! is where this component was allowed to start a unit at all, and this file is
//! the bound that decision was granted under: one unit, named by a constant,
//! instanced by a number, and three read-or-start methods. [ADR 0135](../../../docs/decisions/0135-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md)
//! added a second unit under the same bound, and nothing else.
//!
//! # Why read the source rather than test the behaviour
//!
//! Because the fault this guards cannot be reached from a test. `StopUnit`
//! would work perfectly if somebody added it; nothing would fail; a privileged
//! component would simply have grown the ability to stop any unit on the
//! machine. There is no input that catches that, so the thing to read is the
//! file.
//!
//! This is `alo-brokerd`'s
//! `tests/the_updates_are_carried_by_a_unit.rs` shape, deliberately — the same
//! question asked of the other component that talks to systemd.

#![cfg(target_os = "linux")]
#![expect(
    clippy::panic,
    reason = "in a test, a panic on a file that no longer says what this crate does is the \
              failure being reported"
)]

use std::path::{Path, PathBuf};

/// The file that does the talking.
fn the_source() -> String {
    let at = where_it_is();
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// Where that file is.
fn where_it_is() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/starting_their_units.rs")
}

/// Every systemd method that changes something, and none of them may appear.
///
/// Not an exhaustive list of systemd's interface — that would be a list nobody
/// maintains — but every verb that **stops, kills, masks, reloads, enables or
/// writes**, which is the set that would turn this component into a general
/// lever on the machine.
const NOT_FROM_HERE: [&str; 12] = [
    "StopUnit",
    "RestartUnit",
    "KillUnit",
    "MaskUnitFiles",
    "UnmaskUnitFiles",
    "EnableUnitFiles",
    "DisableUnitFiles",
    "Reload",
    "Reexecute",
    "SetUnitProperties",
    "StartTransientUnit",
    "ResetFailedUnit",
];

/// **None of the changing verbs is in this file.**
#[test]
fn this_component_says_nothing_to_systemd_that_changes_it() {
    let source = the_source();
    let found: Vec<&str> = NOT_FROM_HERE
        .into_iter()
        .filter(|verb| source.contains(verb))
        .collect();
    assert!(
        found.is_empty(),
        "{} names systemd verbs this component may not use: {found:?}",
        where_it_is().display()
    );
}

/// **The control**: this check can fail.
///
/// A test that only ever reports an empty list is a test that would pass
/// against an empty file, against a renamed module, and against a `NOT_FROM_HERE`
/// somebody emptied. So the same filter is run over a line that does contain
/// one, and must find it.
#[test]
fn the_check_finds_a_changing_verb_when_there_is_one() {
    let pretend = "let _ = call(&bus, at, i, \"StopUnit\", &());";
    let found: Vec<&str> = NOT_FROM_HERE
        .into_iter()
        .filter(|verb| pretend.contains(verb))
        .collect();
    assert_eq!(
        found,
        vec!["StopUnit"],
        "the filter cannot find a verb that is there, so finding none above meant nothing"
    );
}

/// **And the file it reads is the file that does the talking.**
///
/// The other half of the control. A path that had gone stale would make the
/// test above read something irrelevant and pass — so this asserts the file
/// names the bus and the method it is supposed to be about.
#[test]
fn the_file_this_reads_is_the_one_that_talks_to_systemd() {
    let source = the_source();
    for must in [
        "org.freedesktop.systemd1",
        "StartUnit",
        "EVERY_METHOD",
        "THE_UNIT",
    ] {
        assert!(
            source.contains(must),
            "{} does not mention {must}, so it is not the file this test is about",
            where_it_is().display()
        );
    }
}

/// **Each unit's name is built from a constant and a number, and nothing else.**
///
/// `CLAUDE.md` law 2: no spelling of *start whatever unit this names*. Each
/// instance comes from a `u32` parameter, so the only text in a name is a
/// constant of this repository. A `&str` parameter arriving here later is what
/// this catches.
#[test]
fn each_unit_is_named_by_a_constant_and_instanced_by_a_number() {
    let source = the_source();
    for (built, by) in [
        (
            r#"format!("{THE_UNIT}@{person}.service")"#,
            "fn made_for(&self, person: u32)",
        ),
        (
            r#"format!("{THE_NAMES_UNIT}@{person}.service")"#,
            "fn given_to(&self, person: u32)",
        ),
        (
            r#"format!("{THE_DESKTOPS_UNIT}@{person}.service")"#,
            "fn the_desktop_of(person: u32)",
        ),
    ] {
        assert!(
            source.contains(built),
            "a unit's name is no longer a constant plus a number — {built} is missing from {}",
            where_it_is().display()
        );
        assert!(
            source.contains(by),
            "the number is no longer a u32 in `{by}`, so something other than a number can reach \
             a unit name"
        );
    }
}

/// **And there are three units, not a fourth.**
///
/// A unit name is built in exactly three places, and the functions that start
/// one are private — so the set of things a sign-in can start is the three
/// constants, and a fourth arriving is a decision this test makes somebody take
/// on purpose.
#[test]
fn this_component_builds_three_unit_names_and_no_more() {
    let source = the_source();
    let built = source.matches(r#"@{person}.service")"#).count();
    assert_eq!(
        built,
        3,
        "{} builds {built} unit names, and ADR 0121, ADR 0135 and ADR 0141 allow three",
        where_it_is().display()
    );
    for private in ["started_and_finished", "started_and_running", "the_desktop_of"] {
        assert!(
            source.contains(&format!("\nfn {private}("))
                && !source.contains(&format!("pub fn {private}(")),
            "{private}, which starts or names a unit, is public, or gone, in {}",
            where_it_is().display()
        );
    }
}

/// **Nothing in this crate asks `logind` for a session.**
///
/// It used to: `CreateSession`, with this process as the leader, on the
/// person's seat. Measured inside the image on 2026-10-30, `logind` then moved
/// `alo-sessiond` into the person's session scope and gave it their seat, and
/// the desktop's own session was never made — so the person's terminal was in
/// no session and restart, power off and the time zone asked for a password
/// (ADR 0141). The desktop's unit opens the session now.
#[test]
fn nothing_in_this_crate_opens_a_session_of_its_own() {
    let mut found = Vec::new();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let entries = std::fs::read_dir(&source)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", source.display()));
    let mut read = 0;
    for entry in entries.flatten() {
        let at = entry.path();
        if at.extension().is_none_or(|it| it != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&at)
            .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()));
        read += 1;
        for (number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or(line);
            if code.contains("CreateSession") || code.contains("org.freedesktop.login1") {
                found.push(format!("{}:{}", at.display(), number + 1));
            }
        }
    }
    // The control: the directory read is this crate's source.
    assert!(read > 5, "only {read} files were read from {}", source.display());
    assert!(
        found.is_empty(),
        "this crate asks logind for a session again, which makes alo-sessiond the leader of the \
         person's session and leaves their desktop outside it: {found:?}"
    );
}
