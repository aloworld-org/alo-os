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
    clippy::panic,
    reason = "in a test, a panic naming what could not be read is the failure being reported"
)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use alo_starting::{
    EnvironmentBlock, Menu, SAVED_ENTRY, System, THE_BLOCK_ON_THE_ESP, THE_COUNTDOWN,
    THE_ENVIRONMENT_BLOCK, THE_MENU, TheLoader, TheLoadersFiles, TheStartingChoice,
    start_by_default, the_identity_of, to_start_by_default,
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

/// A directory holding a machine's loader files, made for one test.
fn a_machines_loader(named: &str) -> TheLoadersFiles {
    let at = std::env::temp_dir().join(format!(
        "alo-starting-one-place-{}-{named}",
        std::process::id()
    ));
    drop(fs::remove_dir_all(&at));
    fs::create_dir_all(&at).expect("a directory for this test's files");
    let loader = TheLoadersFiles::at(&at.join("menu.cfg"), &at.join("block"));
    fs::write(
        loader.menu(),
        Menu::offering("Windows", THE_COUNTDOWN)
            .expect("a menu is generated")
            .written(),
    )
    .expect("the menu is written");
    fs::write(
        loader.block(),
        EnvironmentBlock::empty()
            .written()
            .expect("a block is written"),
    )
    .expect("the block is written");
    loader
}

/// **The road a person's change travels ends in that one file, and adds no
/// second copy of the answer on the way.**
///
/// The whole of it, over files on a disk: a person picks a system in Settings,
/// that becomes the broker's verb, the verb's argument is carried out where the
/// privilege is, and what the menu will read at the next start is the system
/// they picked. Nothing between them keeps the answer — the verb carries
/// thirty-two bytes naming a system, and the only thing that changes on the
/// disk is the loader's own file.
#[test]
fn the_road_a_persons_change_travels_ends_in_that_one_file() {
    for system in System::BOTH {
        let loader = a_machines_loader("the-road");
        let menu_before = fs::read(loader.menu()).expect("the menu is there");

        let verb = to_start_by_default(system);
        let alo_broker::Argument::Identity(identity) = verb.argument() else {
            panic!("{} does not name a system", verb.name());
        };
        assert_eq!(verb.name(), "starting.default");
        assert_eq!(identity, the_identity_of(system));

        assert_eq!(start_by_default(&loader, identity), Ok(system));

        let block = EnvironmentBlock::read(&loader.saved().expect("the block is read"))
            .expect("what is there is a block");
        assert_eq!(TheStartingChoice::read(&block), system);
        assert_eq!(block.how_many(), 1, "the file grew a second answer");
        assert_eq!(
            fs::read(loader.menu()).expect("the menu is still there"),
            menu_before,
            "changing which system starts rewrote the menu"
        );

        let beside_it: Vec<String> = fs::read_dir(
            loader
                .block()
                .parent()
                .expect("the files are in a directory"),
        )
        .expect("the directory is read")
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
        assert_eq!(
            beside_it.len(),
            2,
            "something was written beside the loader's two files: {beside_it:?}"
        );
    }
}

/// **A machine that offers one system is not set to start the other**, and the
/// loader's file is left exactly as it was.
///
/// The refusal that keeps the one place honest: the answer in that file is only
/// ever a system this machine has, so a person reading it is never told their
/// computer starts something that is not on it.
#[test]
fn a_machine_that_offers_one_system_is_not_set_to_start_the_other() {
    let loader = a_machines_loader("replaced");
    fs::remove_file(loader.menu()).expect("the menu goes, as on a machine alo OS replaced");
    let before = fs::read(loader.block()).expect("the block is there");

    assert_eq!(loader.offering(), Ok(vec![System::AloOs]));
    assert!(
        start_by_default(&loader, the_identity_of(System::Windows)).is_err(),
        "a machine with no Windows on it was set to start Windows"
    );
    assert_eq!(
        fs::read(loader.block()).expect("the block is there"),
        before
    );
}

/// **The file the choice is kept in is on the EFI system partition, and the
/// menu is not.**
///
/// ADR 0066 term 1: the last choice is kept on *the one filesystem both systems
/// can read and write*. The menu is alo OS's own file and stays on alo OS's own
/// filesystem — the two are deliberately not beside each other, and a change
/// that put them back together would take the default somewhere Windows cannot
/// reach while every test about verbs, refusals and records went on passing.
#[test]
fn the_choice_is_on_the_partition_both_systems_share_and_the_menu_is_not() {
    let block = Path::new(THE_ENVIRONMENT_BLOCK);
    let menu = Path::new(THE_MENU);
    assert!(block.is_absolute(), "{block:?}");
    assert!(menu.is_absolute(), "{menu:?}");
    assert_ne!(
        block.parent(),
        menu.parent(),
        "the last choice is back beside the menu, on a filesystem Windows cannot write"
    );
    assert!(
        THE_ENVIRONMENT_BLOCK.ends_with(THE_BLOCK_ON_THE_ESP),
        "{THE_ENVIRONMENT_BLOCK} is not {THE_BLOCK_ON_THE_ESP} on a mounted partition"
    );
}

/// **The file alo OS opens is the file the loader saves into.**
///
/// The two are written in different languages — a path from `/` on a machine
/// running alo OS, and a path from the partition's own root in the loader's
/// configuration — and they are the same file. There is no test a machine could
/// run to notice them drifting apart, because by then the person's change would
/// be landing in a file nothing reads; so they are held together here, in the
/// repository, where the drift would have to pass a reader first.
#[test]
fn the_file_alo_os_opens_is_the_file_the_loader_saves_into() {
    let written = Menu::offering("Windows", THE_COUNTDOWN)
        .expect("a menu with a plain title")
        .written();
    let named_by_the_loader: BTreeSet<&str> = written
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("load_env") || line.starts_with("save_env"))
        .filter_map(|line| line.split_whitespace().nth(2))
        .collect();

    assert_eq!(
        named_by_the_loader,
        BTreeSet::from([&*format!("(${{esp}}){THE_BLOCK_ON_THE_ESP}")]),
        "the loader reads and writes somewhere other than the one file alo OS opens"
    );
    assert_eq!(
        Path::new(THE_ENVIRONMENT_BLOCK).file_name(),
        Path::new(THE_BLOCK_ON_THE_ESP).file_name()
    );
}
