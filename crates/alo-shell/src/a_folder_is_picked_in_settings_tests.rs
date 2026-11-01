//! What a person meets while giving alo a folder, against a disk written down
//! rather than created.
//!
//! The whole of what this file learns about a disk arrives through
//! `alo_picking::Folders`, so a tree in a map exercises every rule — on every
//! platform, with no privilege and no temporary directory. The refusals are
//! the point: a folder that went away between being listed and being opened is
//! the case a real filesystem will not reproduce on demand.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use alo_picking::{Inside, NotShown};
use alo_strings::{Strings, Vocabulary};

use super::*;

/// A disk written down: which folder holds which folders.
struct SomeFolders(BTreeMap<PathBuf, Vec<String>>);

impl SomeFolders {
    /// A person's home with three folders in it, one of which has a folder.
    fn a_persons_home() -> Self {
        let mut tree = BTreeMap::new();
        tree.insert(PathBuf::from("/"), vec!["var".to_owned()]);
        tree.insert(PathBuf::from("/var"), vec!["home".to_owned()]);
        tree.insert(PathBuf::from("/var/home"), vec!["alo".to_owned()]);
        tree.insert(
            PathBuf::from("/var/home/alo"),
            vec![
                "Documents".to_owned(),
                "Downloads".to_owned(),
                "Pictures".to_owned(),
            ],
        );
        tree.insert(
            PathBuf::from("/var/home/alo/Documents"),
            vec!["Invoices".to_owned()],
        );
        tree.insert(PathBuf::from("/var/home/alo/Documents/Invoices"), vec![]);
        tree.insert(PathBuf::from("/var/home/alo/Downloads"), vec![]);
        tree.insert(PathBuf::from("/var/home/alo/Pictures"), vec![]);
        Self(tree)
    }

    /// The same disk with one folder taken away, the way a folder listed a
    /// moment ago and deleted since has been.
    fn without(mut self, gone: &str) -> Self {
        self.0.remove(Path::new(gone));
        self
    }
}

impl Folders for SomeFolders {
    fn inside(&self, folder: &Path) -> Result<Inside, NotShown> {
        self.0
            .get(folder)
            .map(|names| Inside::these(names.clone()))
            .ok_or(NotShown::WentAway)
    }
}

/// Everything this surface can say: the picker's own words and this window's.
fn what_it_can_say() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_picking::declare_into(&mut vocabulary).unwrap();
    crate::the_pickers_words::declare_the_pickers_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The picker, opened in the person's home.
fn in_their_home(folders: &SomeFolders) -> AFolderBeingPicked {
    AFolderBeingPicked::standing_in(Path::new("/var/home/alo"), folders).unwrap()
}

/// What a person sees when they open it: their own folders, and the folder
/// they are in offered first.
#[test]
fn the_picker_opens_in_their_home_and_offers_it_first() {
    let folders = SomeFolders::a_persons_home();
    let picking = in_their_home(&folders);

    assert_eq!(
        picking.rows(),
        vec![
            PickingRow::PickThisFolder,
            PickingRow::UpOneFolder,
            PickingRow::IntoFolder("Documents".to_owned()),
            PickingRow::IntoFolder("Downloads".to_owned()),
            PickingRow::IntoFolder("Pictures".to_owned()),
        ]
    );
}

/// **A row that could only ever refuse is not shown.** At the top of the disk
/// there is nothing above, so *up one* is absent rather than present and
/// useless.
#[test]
fn there_is_no_way_up_from_the_top_of_the_disk() {
    let folders = SomeFolders::a_persons_home();
    let picking = AFolderBeingPicked::standing_in(Path::new("/"), &folders).unwrap();

    assert!(!picking.rows().contains(&PickingRow::UpOneFolder));
    assert_eq!(picking.rows().first(), Some(&PickingRow::PickThisFolder));
}

/// Opening a folder shows what is in it.
#[test]
fn going_into_a_folder_shows_what_is_in_it() {
    let folders = SomeFolders::a_persons_home();
    let mut picking = in_their_home(&folders);
    let strings = what_it_can_say();

    let did = picking.act(
        &PickingRow::IntoFolder("Documents".to_owned()),
        &folders,
        &strings,
    );

    assert_eq!(did, WhatPickingDid::ItChanged);
    assert_eq!(picking.at(), Path::new("/var/home/alo/Documents"));
    assert!(
        picking
            .rows()
            .contains(&PickingRow::IntoFolder("Invoices".to_owned()))
    );
    assert_eq!(picking.told(), None);
}

/// And going up comes back.
#[test]
fn going_up_comes_back_to_where_they_were() {
    let folders = SomeFolders::a_persons_home();
    let mut picking = in_their_home(&folders);
    let strings = what_it_can_say();

    picking.act(
        &PickingRow::IntoFolder("Documents".to_owned()),
        &folders,
        &strings,
    );
    let did = picking.act(&PickingRow::UpOneFolder, &folders, &strings);

    assert_eq!(did, WhatPickingDid::ItChanged);
    assert_eq!(picking.at(), Path::new("/var/home/alo"));
}

/// **Picking does not grant.** It asks how long, which is the owner's decision
/// of 2026-10-24: the person is asked every time rather than given a default.
#[test]
fn picking_a_folder_asks_how_long_rather_than_granting_it() {
    let folders = SomeFolders::a_persons_home();
    let mut picking = in_their_home(&folders);
    let strings = what_it_can_say();

    picking.act(
        &PickingRow::IntoFolder("Documents".to_owned()),
        &folders,
        &strings,
    );
    let did = picking.act(&PickingRow::PickThisFolder, &folders, &strings);

    assert_eq!(did, WhatPickingDid::ItChanged);
    assert!(picking.is_saying_how_long());
    // **Every length, derived rather than listed.** This spelled out three
    // by hand and broke on 2026-10-25 when a fourth was added — which is
    // exactly what listing them again would set up for the next person. The
    // order is held by `how_long_a_grant_lasts`' own test (shortest first,
    // the endless one last), so this one asks only that the picker offers
    // the whole list and nothing beside it.
    assert_eq!(
        picking.rows(),
        HowLong::ALL
            .iter()
            .copied()
            .map(PickingRow::For)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        picking.rows().len(),
        4,
        "a length was added and not offered"
    );
}

/// And the heading changes with the step, so the list is never the lengths
/// under a heading asking for a folder.
#[test]
fn the_heading_says_which_of_the_two_questions_is_being_asked() {
    let folders = SomeFolders::a_persons_home();
    let mut picking = in_their_home(&folders);
    let strings = what_it_can_say();

    let asking_for_a_folder = picking.asking(&strings).into_text();
    picking.act(&PickingRow::PickThisFolder, &folders, &strings);
    let asking_how_long = picking.asking(&strings).into_text();

    assert_ne!(asking_for_a_folder, asking_how_long);
    assert!(!asking_how_long.is_empty());
}

/// Answering *how long* hands the caller the folder and the length, and makes
/// no grant here.
#[test]
fn saying_how_long_hands_the_folder_and_the_length_to_the_caller() {
    let folders = SomeFolders::a_persons_home();
    let mut picking = in_their_home(&folders);
    let strings = what_it_can_say();

    picking.act(
        &PickingRow::IntoFolder("Documents".to_owned()),
        &folders,
        &strings,
    );
    picking.act(&PickingRow::PickThisFolder, &folders, &strings);
    let did = picking.act(&PickingRow::For(HowLong::ADay), &folders, &strings);

    let WhatPickingDid::GiveItThisFolder { chosen, how_long } = did else {
        panic!("a length was chosen and nothing was handed over: {did:?}")
    };
    assert_eq!(how_long, HowLong::ADay);
    assert_eq!(chosen.folder(), Some(Path::new("/var/home/alo/Documents")));
}

/// **The whole machine is never granted**, and the refusal is a rule in the
/// person's own words rather than a failure.
#[test]
fn picking_the_top_of_the_disk_is_refused_in_the_persons_own_words() {
    let folders = SomeFolders::a_persons_home();
    let mut picking = AFolderBeingPicked::standing_in(Path::new("/"), &folders).unwrap();
    let strings = what_it_can_say();

    let did = picking.act(&PickingRow::PickThisFolder, &folders, &strings);

    assert_eq!(did, WhatPickingDid::ItChanged);
    assert!(!picking.is_saying_how_long());
    let told = picking.told().expect("a sentence");
    assert!(told.contains("whole machine"), "{told}");
}

/// A folder listed a moment ago and gone since: the person is told, and the
/// picker is still where it was and still works.
#[test]
fn a_folder_that_went_away_is_said_and_leaves_the_picker_usable() {
    let folders = SomeFolders::a_persons_home().without("/var/home/alo/Downloads");
    let mut picking = in_their_home(&folders);
    let strings = what_it_can_say();

    let did = picking.act(
        &PickingRow::IntoFolder("Downloads".to_owned()),
        &folders,
        &strings,
    );

    assert_eq!(did, WhatPickingDid::ItChanged);
    assert_eq!(picking.at(), Path::new("/var/home/alo"));
    assert!(picking.told().is_some());

    // And the picker still works afterwards.
    picking.act(
        &PickingRow::IntoFolder("Documents".to_owned()),
        &folders,
        &strings,
    );
    assert_eq!(picking.at(), Path::new("/var/home/alo/Documents"));
    assert_eq!(picking.told(), None, "a refusal outlived the move after it");
}

/// A row that is not in the list in front of them does nothing at all.
#[test]
fn a_row_that_is_not_in_the_list_does_nothing() {
    let folders = SomeFolders::a_persons_home();
    let mut picking = AFolderBeingPicked::standing_in(Path::new("/"), &folders).unwrap();
    let strings = what_it_can_say();

    let did = picking.act(&PickingRow::UpOneFolder, &folders, &strings);

    assert_eq!(did, WhatPickingDid::Nothing);
    assert_eq!(picking.at(), Path::new("/"));
    assert_eq!(picking.told(), None);
}

/// A length cannot be chosen before a folder is, so nothing can be granted
/// with nothing to grant it over.
#[test]
fn a_length_does_nothing_before_a_folder_has_been_chosen() {
    let folders = SomeFolders::a_persons_home();
    let mut picking = in_their_home(&folders);
    let strings = what_it_can_say();

    let did = picking.act(&PickingRow::For(HowLong::AMonth), &folders, &strings);

    assert_eq!(did, WhatPickingDid::Nothing);
    assert!(!picking.is_saying_how_long());
}

/// Every row says something a person can read, and a folder says its own name.
#[test]
fn every_row_says_something_and_a_folder_says_its_own_name() {
    let folders = SomeFolders::a_persons_home();
    let mut picking = in_their_home(&folders);
    let strings = what_it_can_say();

    for row in picking.rows() {
        let said = picking.said(&row, &strings);
        assert!(!said.is_empty(), "{row:?}");
        assert!(!said.contains('«'), "a key reached a person: {said}");
        if let PickingRow::IntoFolder(name) = &row {
            assert_eq!(&said, name);
        }
    }

    picking.act(&PickingRow::PickThisFolder, &folders, &strings);
    for row in picking.rows() {
        let said = picking.said(&row, &strings);
        assert!(!said.is_empty(), "{row:?}");
        assert!(!said.contains('«'), "a key reached a person: {said}");
    }
}

/// A home with the hidden folders every real home has, as a booted machine
/// listed them on 2026-11-01.
fn a_home_with_hidden_folders() -> SomeFolders {
    let mut folders = SomeFolders::a_persons_home();
    folders.0.insert(
        PathBuf::from("/var/home/alo"),
        vec![
            ".cache".to_owned(),
            ".config".to_owned(),
            "Documents".to_owned(),
            "Downloads".to_owned(),
            "Pictures".to_owned(),
        ],
    );
    folders
        .0
        .insert(PathBuf::from("/var/home/alo/.config"), vec![]);
    folders
}

/// **Hidden folders are not listed until the person asks**, and the row that
/// shows them is at the end.
#[test]
fn hidden_folders_are_hidden_and_one_row_shows_them() {
    let folders = a_home_with_hidden_folders();
    let picking = in_their_home(&folders);
    assert_eq!(
        picking.rows(),
        [
            PickingRow::PickThisFolder,
            PickingRow::UpOneFolder,
            PickingRow::IntoFolder("Documents".to_owned()),
            PickingRow::IntoFolder("Downloads".to_owned()),
            PickingRow::IntoFolder("Pictures".to_owned()),
            PickingRow::HiddenFoldersShown(true),
        ]
    );
}

/// **Showing them lists them, and a hidden folder can still be given**: a
/// person could give alo `.config` before hidden folders were hidden, and
/// still can (law 5).
#[test]
fn shown_hidden_folders_can_be_walked_into_and_hidden_again() {
    let folders = a_home_with_hidden_folders();
    let strings = what_it_can_say();
    let mut picking = in_their_home(&folders);
    let into_config = PickingRow::IntoFolder(".config".to_owned());
    assert_eq!(
        picking.act(&into_config, &folders, &strings),
        WhatPickingDid::Nothing,
        "the control: a hidden folder was reachable while hidden"
    );

    assert_eq!(
        picking.act(&PickingRow::HiddenFoldersShown(true), &folders, &strings),
        WhatPickingDid::ItChanged
    );
    assert!(picking.rows().contains(&into_config));
    assert_eq!(
        picking.rows().last(),
        Some(&PickingRow::HiddenFoldersShown(false)),
        "the row does not offer to hide them again"
    );

    assert_eq!(
        picking.act(&into_config, &folders, &strings),
        WhatPickingDid::ItChanged
    );
    assert_eq!(picking.at(), Path::new("/var/home/alo/.config"));
    assert!(matches!(
        picking.act(&PickingRow::PickThisFolder, &folders, &strings),
        WhatPickingDid::ItChanged
    ));
    assert!(
        picking.is_saying_how_long(),
        "a hidden folder could not be given"
    );
}

/// **No row about hidden folders where there are none.**
#[test]
fn a_folder_with_nothing_hidden_offers_nothing_to_show() {
    let folders = SomeFolders::a_persons_home();
    let picking = in_their_home(&folders);
    assert!(
        !picking
            .rows()
            .iter()
            .any(|row| matches!(row, PickingRow::HiddenFoldersShown(_)))
    );
}

/// **The page says which folder this is**: its name and its full path, in
/// the home and inside a folder.
#[test]
fn the_picker_says_which_folder_the_person_is_in() {
    let folders = SomeFolders::a_persons_home();
    let strings = what_it_can_say();
    let mut picking = in_their_home(&folders);
    assert_eq!(
        picking.where_they_are(&strings),
        "You are in alo, at /var/home/alo"
    );
    picking.act(
        &PickingRow::IntoFolder("Documents".to_owned()),
        &folders,
        &strings,
    );
    assert_eq!(
        picking.where_they_are(&strings),
        "You are in Documents, at /var/home/alo/Documents"
    );
}

/// **At the top of the disk the folder has no name of its own**, so it is
/// called by its path rather than by nothing.
#[test]
fn the_top_of_the_disk_is_called_by_its_path() {
    let folders = SomeFolders::a_persons_home();
    let strings = what_it_can_say();
    let picking = AFolderBeingPicked::standing_in(Path::new("/"), &folders).unwrap();
    assert_eq!(picking.where_they_are(&strings), "You are in /, at /");
}
