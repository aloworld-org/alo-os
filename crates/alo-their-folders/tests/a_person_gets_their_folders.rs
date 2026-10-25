//! A person's folders, made against the vocabulary the machine really has.
//!
//! The tests inside this crate build their own small vocabulary, which proves
//! the rules and cannot prove the one thing that breaks silently: **whether
//! this crate's words are collected at all.** `alo-saying` is the one place
//! that gathers every crate's list, and a crate missing from it gets a
//! `Strings` that answers each of its keys with the key.
//!
//! That failure is invisible everywhere else in this repository — a sentence
//! comes out as `«their-folders.documents»` and somebody notices. Here it
//! would be refused by `crate::naming` and a person would get **no folders at
//! all**, with the reason in a log nobody reads. So this file asks the
//! question the unit tests cannot: assembled the way the shell assembles it,
//! does this work?

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_strings::Strings;
use alo_their_folders::{Folder, OnThisDisk, made_for, the_record};

/// Everything this machine can say, the way the shell builds it.
fn as_the_shell_has_it() -> Strings {
    Strings::of(
        alo_saying::everything_this_machine_can_say().expect("the machine's own vocabulary"),
    )
}

/// A folder for one test, removed when the test ends.
struct Somewhere(PathBuf);

impl Somewhere {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let path = std::env::temp_dir().join(format!(
            "alo-their-folders-acceptance-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("a folder for this test");
        Self(path)
    }
}

impl Drop for Somewhere {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// **This crate's words are collected with everybody else's.**
///
/// The one that fails if `alo-saying` was never told about this crate, and the
/// reason this file exists.
#[test]
fn the_folder_names_are_in_the_vocabulary_the_machine_assembles() {
    let strings = as_the_shell_has_it();
    for folder in Folder::ALL {
        let said = strings.say(&folder.word().key(), &alo_strings::Filling::nothing());
        assert!(
            !said.is_a_bug(),
            "{} is not in the machine's vocabulary: {}",
            folder.word().key(),
            said.text()
        );
    }
}

/// And the names are the ones a person would recognise, not keys.
#[test]
fn a_person_on_this_machine_gets_folders_with_real_names() {
    let home = Somewhere::new();
    let made = made_for(&home.0, &as_the_shell_has_it(), &OnThisDisk);

    assert!(made.everything_was_made(), "{made:?}");
    assert!(home.0.join("Documents").is_dir());
    assert!(home.0.join("Downloads").is_dir());
    assert!(home.0.join("Pictures").is_dir());
}

/// Nothing in this crate's list collides with another crate's.
///
/// `alo-saying` refuses a duplicate key rather than replacing one, so a
/// collision is a machine with no vocabulary at all — every crate's words
/// lost, not just these.
#[test]
fn no_word_here_takes_a_key_another_crate_already_had() {
    assert!(
        alo_saying::everything_this_machine_can_say().is_ok(),
        "a key in this crate collides with one already declared"
    );
}

/// The record is written in the format other software reads, from the names
/// this machine really produced.
#[test]
fn the_record_names_the_folders_this_machine_actually_made() {
    let home = Somewhere::new();
    let made = made_for(&home.0, &as_the_shell_has_it(), &OnThisDisk);
    let written = std::fs::read_to_string(made.recorded().expect("a record")).expect("readable");

    for (_, at) in made.folders() {
        let name = at.file_name().expect("a name").to_string_lossy();
        assert!(
            written.contains(&format!("\"$HOME/{name}\"")),
            "{name} was made and is not in the record:\n{written}"
        );
    }
    assert_eq!(written.lines().filter(|it| !it.starts_with('#')).count(), 3);
}

/// The record built from nothing is still a file somebody can read, and names
/// no folder — the shape `made_for` relies on when it declines to write one.
#[test]
fn a_record_of_no_folders_names_no_folder() {
    let text = the_record(&[]);
    assert!(text.lines().all(|it| it.starts_with('#')), "{text}");
}
