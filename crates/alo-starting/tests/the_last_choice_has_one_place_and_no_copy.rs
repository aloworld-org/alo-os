//! **The last choice is kept in one place, and nothing anywhere keeps a
//! second.**
//!
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)'s
//! third term, and the reason it is a term: *two copies drift, and a menu that
//! preselects one thing while a setting says another is the bug this term
//! exists to prevent.*
//!
//! A unit test can show that this crate reads and writes the loader's own
//! file. It cannot show that nothing **else** does, and that is the half the
//! term is about — so this reads the workspace's own crates and the image's own
//! files off the disk, and fails if anything outside this crate names the file,
//! the setting in it, or the menu the loader reads.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what could not be read is the failure being reported"
)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use alo_starting::{
    EnvironmentBlock, Menu, SAVED_ENTRY, System, THE_COUNTDOWN, THE_ENVIRONMENT_BLOCK, THE_MENU,
    TheStartingChoice,
};

/// The one crate any of this may be named in.
const THE_ONE_CRATE: &str = "crates/alo-starting/";

/// The words that would be a second copy of the last choice, wherever they are
/// written.
const A_SECOND_COPY: [&str; 4] = [
    "grubenv",
    "saved_entry",
    "custom.cfg",
    "GRUB Environment Block",
];

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// What is not this repository's own text: what a build wrote, and what git
/// keeps.
fn is_not_ours(name: &str) -> bool {
    matches!(name, ".git" | "target" | "node_modules") || name.starts_with("target-")
}

/// Every file under `directory`, as a repository-relative path with forward
/// slashes and the text in it.
fn everything_under(directory: &Path, below: &str, into: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = format!("{below}/{name}");
        if entry.path().is_dir() {
            if !is_not_ours(&name) {
                everything_under(&entry.path(), &path, into);
            }
        } else if let Ok(text) = fs::read_to_string(entry.path()) {
            into.push((path, text));
        }
    }
}

/// Every file of the workspace's crates and of the image, read.
fn what_this_machine_ships() -> Vec<(String, String)> {
    let mut written = Vec::new();
    for directory in ["crates", "image"] {
        everything_under(
            &the_repository().join(directory),
            directory.trim_end_matches('/'),
            &mut written,
        );
    }
    assert!(
        written.len() > 500,
        "{} file(s) were read, which is not this repository's crates and image",
        written.len()
    );
    written
}

/// **Nothing outside this crate names the last choice, the file it is in, or
/// the menu that reads it.**
///
/// The failure this refuses is not a duplicated constant. It is a second place
/// that decides which system a machine starts — a setting of ours beside the
/// loader's, a menu written somewhere else, a reader that caches the answer —
/// each of which is correct on the day it is written and wrong the first time
/// somebody chooses at the menu instead.
#[test]
fn nothing_outside_this_crate_keeps_a_second_copy_of_the_last_choice() {
    let mut elsewhere: BTreeSet<String> = BTreeSet::new();
    for (path, text) in what_this_machine_ships() {
        if path.contains(THE_ONE_CRATE) {
            continue;
        }
        for word in A_SECOND_COPY {
            if text.contains(word) {
                elsewhere.insert(format!("{path} names {word}"));
            }
        }
    }
    assert!(
        elsewhere.is_empty(),
        "the last choice is kept in more than one place: {elsewhere:?}"
    );
}

/// **The menu and the setting agree about what Windows is**, which is the bug
/// ADR 0062's third term names: *a menu that preselects one thing while a
/// setting says another.*
///
/// Not a comparison of two constants — a comparison of the **generated menu**
/// with the **setting's own reader**. What the menu writes into the environment
/// block when a person chooses Windows is read out of the configuration text
/// itself, and that value is then put in front of the reader a surface in
/// Settings uses. If they ever stopped meaning the same thing, a person would
/// choose Windows at the menu and Settings would tell them the machine starts
/// alo OS.
#[test]
fn the_menu_and_the_setting_agree_about_what_windows_is() {
    let menu = Menu::offering("Windows", THE_COUNTDOWN).expect("a menu is generated");
    let written = menu.written();

    let saved_by_the_menu = written
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("set "))
        .and_then(|line| line.split_once('='))
        .map(|(name, _)| name.to_owned());
    assert_eq!(
        saved_by_the_menu.as_deref(),
        Some("default"),
        "the menu's first setting is not its default: {written}"
    );

    let chosen = written
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(&format!("set {SAVED_ENTRY}=")))
        .map(|value| value.trim_matches('\'').to_owned())
        .expect("the menu says what it saves when Windows is chosen");

    let mut block = EnvironmentBlock::empty();
    block
        .keep(SAVED_ENTRY, &chosen)
        .expect("what the menu saves is a value a block can hold");
    assert_eq!(
        TheStartingChoice::read(&block),
        System::Windows,
        "the menu saves {chosen:?} and the setting does not read it as Windows"
    );

    assert!(
        written.contains(&format!("set default=\"${{{SAVED_ENTRY}}}\"")),
        "the menu does not take its default from the setting: {written}"
    );
}

/// **A choice written and read back is the choice**, through the loader's own
/// file and nothing beside it — and the file it leaves behind holds exactly one
/// line about it.
#[test]
fn a_choice_written_reads_back_from_the_loaders_own_file() {
    for system in System::BOTH {
        let mut block = EnvironmentBlock::empty();
        block
            .keep("boot_success", "1")
            .expect("a block takes a setting of the base's");
        TheStartingChoice::write(&mut block, system).expect("a choice is written");
        let bytes = block.written().expect("a block is written");

        let read = EnvironmentBlock::read(&bytes).expect("what was written reads back");
        assert_eq!(TheStartingChoice::read(&read), system);

        let text = String::from_utf8(bytes).expect("an environment block is text");
        let about_it: Vec<&str> = text
            .lines()
            .filter(|line| line.contains(SAVED_ENTRY))
            .collect();
        assert_eq!(about_it.len(), 1, "{about_it:?}");
    }
}

/// **The file the choice is kept in is the loader's own, beside the menu**, and
/// neither is somewhere alo OS invented.
#[test]
fn the_file_is_the_loaders_own_and_the_menu_is_beside_it() {
    let block = Path::new(THE_ENVIRONMENT_BLOCK);
    let menu = Path::new(THE_MENU);
    assert_eq!(block.parent(), menu.parent(), "{block:?} {menu:?}");
    assert!(block.is_absolute());
    assert!(menu.is_absolute());
}
