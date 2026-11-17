//! The file chooser as a reader is told about it: the title, the two lines,
//! every row with its kind as description and the keyboard's row selected,
//! the name when saving, what just happened, then Cancel and the button — each
//! pressable, and every press held to this sheet.

use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Instant;

use alo_access::{Surface, TurnedOn};
use alo_acting::{ToChoose, ToChooseAFile};
use alo_applications::Application;
use alo_file_chooser::{Folders, Key, Listing, NotListed};

use super::*;
use crate::the_file_chooser::{AskedForAFile, TheFileChooser};

/// The sheet's words.
fn words() -> Strings {
    let mut vocabulary = alo_strings::Vocabulary::empty();
    crate::the_file_choosers_words::declare_the_file_choosers_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A home with a folder and a file in it.
struct AHome;

impl Folders for AHome {
    fn listed(&self, at: &Path) -> Result<Listing, NotListed> {
        if at == Path::new("/home/anna") {
            Ok(Listing::of(vec!["Letters".to_owned()], vec!["march.txt".to_owned()]))
        } else {
            Err(NotListed::NotThere)
        }
    }

    fn holds(&self, at: &Path, name: &str) -> Option<Kind> {
        (at == Path::new("/home/anna") && name == "march.txt").then_some(Kind::File)
    }
}

/// A sheet up for Text Editor, with its portal's end held.
fn a_sheet(to: ToChoose) -> (TheFileChooser, UnixStream) {
    let (portal, desktop) = UnixStream::pair().unwrap();
    desktop.set_nonblocking(true).unwrap();
    let asked = ToChooseAFile::of(
        Application::identified("org.gnome.TextEditor").unwrap(),
        to,
        PathBuf::from("/home/anna"),
    )
    .unwrap();
    let mut sheet = TheFileChooser::none();
    sheet.asked(vec![AskedForAFile::of(asked, desktop)], true, Instant::now(), &AHome);
    (sheet, portal)
}

/// The desktop's tree with `sheet` on it.
fn read(strings: &Strings, sheet: &TheFileChooser) -> ReadAloudTree {
    ReadAloudTree::of(strings, &[Surface::Desktop], &TurnedOn::nothing()).with_the_file_chooser(
        strings,
        sheet.shows(),
        &|identifier: &str| identifier.to_owned(),
    )
}

/// What hangs under `at`.
fn under(tree: &ReadAloudTree, at: usize) -> Vec<&Node> {
    tree.nodes()[at]
        .children
        .iter()
        .map(|child| &tree.nodes()[*child])
        .collect()
}

fn role(of: alo_access::Role) -> u32 {
    access_roles::number_of(of)
}

/// **Opening is read as it is drawn**, every row with its kind, the
/// keyboard's row holding the keyboard, and each press numbered.
#[test]
fn opening_is_read_as_it_is_drawn() {
    let strings = words();
    let (sheet, _portal) = a_sheet(ToChoose::Open);
    let tree = read(&strings, &sheet);
    let in_front = tree.in_front().unwrap();
    let dialogue = &tree.nodes()[in_front.at];
    assert_eq!(dialogue.role, role(alo_access::Role::Dialogue));
    assert_eq!(dialogue.name, "Open a file");
    let parts = under(&tree, in_front.at);
    let roles: Vec<u32> = parts.iter().map(|it| it.role).collect();
    assert_eq!(
        roles,
        [
            role(alo_access::Role::Label),
            role(alo_access::Role::Label),
            role(alo_access::Role::List),
            role(alo_access::Role::Button),
            role(alo_access::Role::Button),
        ]
    );
    assert_eq!(parts[0].name, "org.gnome.TextEditor asked to open a file");
    assert_eq!(parts[1].name, "In /home/anna");
    assert_eq!(parts[3].name, "Cancel");
    assert_eq!(parts[4].name, "Open");
    let list = tree.nodes()[in_front.at].children[2];
    let rows = under(&tree, list);
    let read: Vec<(&str, &str)> = rows
        .iter()
        .map(|row| (row.name.as_str(), row.description.as_str()))
        .collect();
    assert_eq!(
        read,
        [
            ("home", "The folder above"),
            ("Letters", "Folder"),
            ("march.txt", "File")
        ]
    );
    let keyboard = &tree.nodes()[in_front.keyboard];
    assert_eq!(keyboard.name, "Letters", "the keyboard starts inside the folder");
    let number = sheet.shows().unwrap().number;
    assert_eq!(
        rows[2].press,
        Some(Pressed::OnTheFileChooser {
            sheet: number,
            on: OnTheFileChooser::TheRow(2)
        })
    );
}

/// **Saving reads the name as a field with the keyboard**, and a sentence
/// about what just happened is announced.
#[test]
fn saving_reads_the_name_and_what_happened() {
    let strings = words();
    let (mut sheet, _portal) = a_sheet(ToChoose::Save {
        name: "march.txt".to_owned(),
    });
    sheet.pressed(Key::Enter, Instant::now(), &AHome);
    assert!(sheet.is_up(), "a file of that name is there");
    let tree = read(&strings, &sheet);
    let in_front = tree.in_front().unwrap();
    let parts = under(&tree, in_front.at);
    let field = parts
        .iter()
        .find(|it| it.role == role(alo_access::Role::Entry))
        .unwrap();
    assert_eq!(field.name, "Name");
    assert_eq!(field.description, "march.txt");
    assert_eq!(tree.nodes()[in_front.keyboard].name, "Name");
    let note = parts
        .iter()
        .find(|it| it.announced)
        .unwrap();
    assert_eq!(
        note.name,
        "A file with this name is here already. Save again to replace it."
    );
    assert_eq!(parts.last().unwrap().name, "Save");
}

/// **No sheet, no change to the tree.**
#[test]
fn no_sheet_leaves_the_tree_alone() {
    let strings = words();
    let sheet = TheFileChooser::none();
    let tree = read(&strings, &sheet);
    assert!(tree.in_front().is_none());
}
