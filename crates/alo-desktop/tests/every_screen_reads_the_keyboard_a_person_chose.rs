//! **Every binary that puts a screen up reads the keyboard a person chose, and
//! a test keeps it that way.**
//!
//! # What went wrong, twice
//!
//! `/etc/alo/keyboards.toml` is where the language sheet writes the keyboard a
//! person picked. On 2026-10-14 a booted machine showed the whole of it going
//! right and then being thrown away:
//!
//! ```text
//! alo-compositor: the keyboard is now fr, chosen on the language sheet
//! alo-desktop:    the desktop is going up on /dev/dri/card1, on a gb keyboard
//! ```
//!
//! The greeter read the file. The desktop read `ALO_KEYBOARD` out of its unit
//! file and nothing else, so a person chose French, typed French to sign in,
//! and then worked all day on a British keyboard.
//!
//! **It is the second time this exact pair has happened.** The card to open was
//! fixed in the greeter and not here, under a comment that says *the greeter had
//! the fix and this did not, because the greeter met the fault first*. Two
//! processes read the same machine, one learns something about it, and the other
//! is never told.
//!
//! # Why the assertion is shaped like this
//!
//! A test that only asserted `alo-desktop` calls
//! `the_keyboard_this_machine_has` would refuse today's bug and miss the next
//! one, which will be the third binary somebody adds. So the subject is the
//! **class**: anything in this repository that builds a keymap or stands a
//! screen up is a thing that must ask where the keyboard came from.
//!
//! A binary that puts a screen up and does not read that file is refused by
//! name, and the sentence tells whoever lands it what to call.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The repository root, from this crate's own manifest directory.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("the repository root")
        .to_path_buf()
}

/// What makes a file one that puts a screen up.
///
/// Each of these is a thing only a process standing a compositor up does:
/// building an XKB keymap, or filling in one of the two structs the shell takes
/// a display by.
const PUTS_A_SCREEN_UP: [&str; 3] = ["XkbConfig {", "AMachineToStandOn {", "ADisplayToStandOn {"];

/// The one call that answers *which keyboard does this machine have*.
const ASKS_WHERE_THE_KEYBOARD_CAME_FROM: &str = "the_keyboard_this_machine_has";

/// Every production binary in the workspace: `src/main.rs` and `src/bin/*.rs`.
///
/// Tests are not included and should not be: a test naming a layout outright is
/// a test choosing its own fixture, which is the right thing for it to do.
fn every_binary(repository: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let crates = std::fs::read_dir(repository.join("crates")).expect("crates/");
    for entry in crates {
        let src = entry.expect("a crate directory").path().join("src");
        let main = src.join("main.rs");
        if main.is_file() {
            found.push(main);
        }
        if let Ok(bins) = std::fs::read_dir(src.join("bin")) {
            for bin in bins {
                let path = bin.expect("a binary").path();
                if path.extension().is_some_and(|it| it == "rs") {
                    found.push(path);
                }
            }
        }
    }
    found.sort();
    found
}

#[test]
fn a_binary_that_puts_a_screen_up_reads_the_keyboard_a_person_chose() {
    let repository = the_repository();
    let mut screens: Vec<PathBuf> = Vec::new();
    let mut deaf: Vec<String> = Vec::new();

    for binary in every_binary(&repository) {
        let text = std::fs::read_to_string(&binary).expect("a binary this workspace builds");
        if !PUTS_A_SCREEN_UP.iter().any(|it| text.contains(it)) {
            continue;
        }
        screens.push(binary.clone());
        if !text.contains(ASKS_WHERE_THE_KEYBOARD_CAME_FROM) {
            deaf.push(
                binary
                    .strip_prefix(&repository)
                    .unwrap_or(&binary)
                    .display()
                    .to_string(),
            );
        }
    }

    // **The positive control.** A directory walk that finds nothing passes every
    // assertion below it, so the count is checked before the verdict is
    // believed: two binaries put a screen up today, and if that becomes zero
    // this test has stopped reading the repository rather than started
    // approving of it.
    assert!(
        screens.len() >= 2,
        "this test found only {} binaries that put a screen up, and there are two: the greeter \
         and the desktop. Either they were renamed or moved out of `src/bin`, or this walk has \
         stopped finding them — and a walk that finds nothing passes. Point it at them again \
         before trusting a green run.\n  found: {:?}",
        screens.len(),
        screens
    );

    assert!(
        deaf.is_empty(),
        "these binaries put a screen up without asking which keyboard this machine has:\n  {}\n\n\
         So a person can choose a language at the sign-in screen, watch it take, and then meet a \
         different keyboard one process later — which is what happened on a booted machine on \
         2026-10-14, with nothing failing anywhere. Call \
         `alo_shell::the_machines_keyboard::the_keyboard_this_machine_has`, which reads \
         `/etc/alo/keyboards.toml` first and falls back to the unit file's `ALO_KEYBOARD` only on \
         a machine where nobody has chosen yet, and carry **both** the layout and the variant: \
         dropping the variant is the same bug one layer down, and it was dropped here until the \
         same day.",
        deaf.join("\n  ")
    );
}
