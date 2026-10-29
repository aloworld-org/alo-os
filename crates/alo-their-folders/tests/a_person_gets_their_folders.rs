//! A person's folders, made on a real disk and reached the way a person
//! reaches them: through the picker, by the name that is really there.
//!
//! The tests inside this crate prove the rules against a disk that is two
//! maps. This file asks what only a real one can answer — and the one question
//! the owner put on 2026-10-28 that no unit test reaches: **does a German
//! person's `Dokumente` work everywhere alo would otherwise have assumed
//! `Documents`?** The picker lists what is on the disk and a grant is made
//! from what the picker picked, so the honest test is to walk both.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::time::SystemTime;

use alo_their_folders::{Catalogue, OnThisDisk, made_for, the_record};

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

/// **German, as `xdg-user-dirs` ships it on the machine**: a catalogue on a
/// real disk under a locale tree, found by language the way the program finds
/// it.
fn german_on_the_disk(locales: &Somewhere) -> Catalogue {
    let at = alo_their_folders::catalogue::the_file(&locales.0, "de");
    std::fs::create_dir_all(at.parent().expect("a folder")).expect("the locale tree");
    std::fs::write(
        &at,
        msgfmt(&[("Documents", "Dokumente"), ("Pictures", "Bilder")]),
    )
    .expect("the catalogue");
    Catalogue::for_language(
        &locales.0,
        &alo_strings::Language::written("de").expect("German"),
    )
    .expect("the German catalogue is found")
}

/// A GNU catalogue's bytes, little-endian, as `msgfmt` writes one.
fn msgfmt(pairs: &[(&str, &str)]) -> Vec<u8> {
    let count = u32::try_from(pairs.len()).expect("a few");
    let mut at = 28 + 16 * count;
    let mut head: Vec<u8> = [0x9504_12de_u32, 0, count, 28, 28 + 8 * count, 0, 0]
        .iter()
        .flat_map(|it| it.to_le_bytes())
        .collect();
    let mut tables = Vec::new();
    let mut text = Vec::new();
    for column in [0, 1] {
        for (key, name) in pairs {
            let it = if column == 0 { key } else { name };
            let length = u32::try_from(it.len()).expect("short");
            tables.extend(length.to_le_bytes());
            tables.extend(at.to_le_bytes());
            text.extend(it.as_bytes());
            text.push(0);
            at += length + 1;
        }
    }
    head.extend(tables);
    head.extend(text);
    head
}

/// **A German person gets `Dokumente`, and alo lists and grants it by that
/// name.** The picker, standing in their home, shows `Dokumente` and no
/// `Documents`; picking it and granting it reaches the folder that is really
/// there.
#[test]
fn a_german_persons_dokumente_is_listed_and_granted_by_its_real_name() {
    let locales = Somewhere::new();
    let home = Somewhere::new();
    let catalogue = german_on_the_disk(&locales);

    let made = made_for(&home.0, Some(&catalogue), &OnThisDisk);
    assert!(made.everything_was_made(), "{made:?}");
    assert!(home.0.join("Dokumente").is_dir());
    assert!(!home.0.join("Documents").exists());

    let mut picker = alo_picking::Picker::standing_in(&home.0, &alo_picking::OnThisDisk)
        .expect("the picker stands in the home");
    assert!(
        picker.showing().iter().any(|it| it == "Dokumente"),
        "{:?}",
        picker.showing()
    );
    assert!(!picker.showing().iter().any(|it| it == "Documents"));
    picker
        .go_into("Dokumente", &alo_picking::OnThisDisk)
        .expect("the picker goes into Dokumente");
    let chosen = picker.pick().expect("Dokumente is picked");
    assert_eq!(chosen.folder(), Some(home.0.join("Dokumente").as_path()));

    let mut grants = alo_capability::Grants::default();
    let granted = alo_picking::Granting::to_until_removed("alo")
        .of(&chosen, &mut grants, SystemTime::now())
        .expect("the grant is made");
    assert!(granted.is_some());
    assert_eq!(grants.len(), 1);
}

/// **An existing home is never renamed without asking**, on a real disk: made
/// in English, then signed into in German. `Documents` stays with what is in
/// it, no `Dokumente` appears, and what a rename would be is said.
#[test]
fn an_english_home_signed_into_in_german_keeps_its_folders() {
    let locales = Somewhere::new();
    let home = Somewhere::new();
    let first = made_for(&home.0, None, &OnThisDisk);
    assert!(first.everything_was_made(), "{first:?}");
    std::fs::write(home.0.join("Documents").join("notes.txt"), "theirs").expect("a file");
    let record =
        std::fs::read_to_string(home.0.join(alo_their_folders::THE_RECORD)).expect("the record");

    let again = made_for(&home.0, Some(&german_on_the_disk(&locales)), &OnThisDisk);

    assert!(again.everything_was_made(), "{again:?}");
    assert!(
        !home.0.join("Dokumente").exists(),
        "a second folder was made"
    );
    assert_eq!(
        std::fs::read_to_string(home.0.join("Documents").join("notes.txt")).expect("kept"),
        "theirs"
    );
    assert_eq!(
        std::fs::read_to_string(home.0.join(alo_their_folders::THE_RECORD)).expect("the record"),
        record,
        "the record was rewritten"
    );
    assert!(
        again
            .named_differently()
            .iter()
            .any(|it| it.now == "Dokumente"),
        "{again:?}"
    );
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

/// The record built from nothing is still a file somebody can read, and names
/// no folder.
#[test]
fn a_record_of_no_folders_names_no_folder() {
    let text = the_record(&[], &alo_their_folders::AlreadyRecorded::default());
    assert!(text.lines().all(|it| it.starts_with('#')), "{text}");
}
