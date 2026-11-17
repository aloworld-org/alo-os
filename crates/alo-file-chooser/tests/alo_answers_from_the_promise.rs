//! **alo answering the file chooser, tested from the promise and not from the
//! code.** Written by a second reader who read `docs/contracts/application-door.md`'s
//! *alo answers the file chooser* and *The person chooses a file*, and
//! `agent-verbs.md`'s `choose_on_the_file_chooser`, and none of the
//! implementation. alo's answer is believed only where the person's own press
//! could have given it: to open, a file that is there; to save, a name in a
//! folder that is there which is not a folder's name, a file already there
//! being one that may be saved over. Every refusal — a file not there, a
//! folder's name, a folder not there or not readable, a name that is not one
//! name — is tested as carefully as the answer, and so is the promise that
//! *the sheet stays up for the person, as it was*: nothing alo answers moves
//! a row, the focus, the folder, the name or the note.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use alo_file_chooser::{Chooser, Folders, Key, Kind, Listing, NotBelieved, NotListed};

/// A disk held in memory: what each folder holds, and the folders that are
/// there but would not be read.
#[derive(Default)]
struct Disk {
    /// Each folder, with its folders and its files.
    held: BTreeMap<PathBuf, (Vec<String>, Vec<String>)>,
    /// Folders the person's account may not read.
    unreadable: BTreeSet<PathBuf>,
}

impl Disk {
    /// This disk, with `at` holding these folders and files.
    fn with(mut self, at: &str, folders: &[&str], files: &[&str]) -> Self {
        self.held.insert(
            PathBuf::from(at),
            (
                folders.iter().map(|name| (*name).to_owned()).collect(),
                files.iter().map(|name| (*name).to_owned()).collect(),
            ),
        );
        self
    }

    /// This disk, with `at` there but not readable by the person.
    fn unreadable(mut self, at: &str) -> Self {
        self.unreadable.insert(PathBuf::from(at));
        self
    }
}

impl Folders for Disk {
    fn listed(&self, folder: &Path) -> Result<Listing, NotListed> {
        if self.unreadable.contains(folder) {
            return Err(NotListed::WouldNotBeRead);
        }
        self.held
            .get(folder)
            .map(|(folders, files)| Listing::of(folders.clone(), files.clone()))
            .ok_or(NotListed::NotThere)
    }

    /// Answered truthfully even for a folder the person cannot list, as
    /// asking the disk about one name may be: so a refusal for an unreadable
    /// folder cannot come from this answer alone.
    fn holds(&self, folder: &Path, name: &str) -> Option<Kind> {
        let (folders, files) = self.held.get(folder)?;
        if folders.iter().any(|one| one == name) {
            Some(Kind::Folder)
        } else {
            files.iter().any(|one| one == name).then_some(Kind::File)
        }
    }
}

/// Alo's machine: a home with documents, one folder the person cannot read,
/// and a folder with a space and non-ASCII in its name.
fn alos_disk() -> Disk {
    Disk::default()
        .with("/", &["var"], &[])
        .with("/var", &["home"], &[])
        .with("/var/home", &["alo"], &[])
        .with(
            "/var/home/alo",
            &["Documents", "Private", "Bücher Neu"],
            &["todo.md"],
        )
        .with(
            "/var/home/alo/Documents",
            &["Letters", "notes.txt.d"],
            &["notes.txt", "to Rui.txt"],
        )
        .with("/var/home/alo/Documents/Letters", &[], &["march.txt"])
        .with("/var/home/alo/Documents/notes.txt.d", &[], &[])
        .with("/var/home/alo/Bücher Neu", &[], &["für Rui.md"])
        .with("/var/home/alo/Private", &[], &["secret.txt"])
        .unreadable("/var/home/alo/Private")
}

/// The documents folder.
const DOCUMENTS: &str = "/var/home/alo/Documents";

/// A sheet to open a file, standing in the documents folder.
fn to_open(disk: &Disk) -> Chooser {
    Chooser::to_open(Path::new(DOCUMENTS), disk)
}

/// A sheet to save, standing in the documents folder and suggesting
/// `draft.txt`.
fn to_save(disk: &Disk) -> Chooser {
    Chooser::to_save(Path::new(DOCUMENTS), "draft.txt", disk)
}

/// Names that are not one name, in any sheet.
const NOT_ONE_NAME: [&str; 7] = [
    "",
    ".",
    "..",
    "a/b",
    "/notes.txt",
    "notes.txt/",
    "../todo.md",
];

/// To open: a file that is there is believed, and what is chosen is that file.
#[test]
fn to_open_a_file_that_is_there_is_believed() {
    let disk = alos_disk();
    let sheet = to_open(&disk);
    assert_eq!(
        sheet.alo_chose(Path::new(DOCUMENTS), "notes.txt", &disk),
        Ok(PathBuf::from(DOCUMENTS).join("notes.txt"))
    );
    assert_eq!(
        sheet.alo_chose(Path::new(DOCUMENTS), "to Rui.txt", &disk),
        Ok(PathBuf::from(DOCUMENTS).join("to Rui.txt"))
    );
}

/// Where the sheet starts is only where it starts: a file in another folder
/// the person could reach, and a name with non-ASCII in it, are believed too.
#[test]
fn to_open_a_file_in_another_folder_is_believed() {
    let disk = alos_disk();
    let sheet = to_open(&disk);
    assert_eq!(
        sheet.alo_chose(
            Path::new("/var/home/alo/Documents/Letters"),
            "march.txt",
            &disk
        ),
        Ok(PathBuf::from("/var/home/alo/Documents/Letters").join("march.txt"))
    );
    assert_eq!(
        sheet.alo_chose(Path::new("/var/home/alo"), "todo.md", &disk),
        Ok(PathBuf::from("/var/home/alo").join("todo.md"))
    );
    assert_eq!(
        sheet.alo_chose(Path::new("/var/home/alo/Bücher Neu"), "für Rui.md", &disk),
        Ok(PathBuf::from("/var/home/alo/Bücher Neu").join("für Rui.md"))
    );
}

/// `not-believed`: a file not there to open.
#[test]
fn to_open_a_file_that_is_not_there_is_not_believed() {
    let disk = alos_disk();
    let sheet = to_open(&disk);
    assert_eq!(
        sheet.alo_chose(Path::new(DOCUMENTS), "missing.txt", &disk),
        Err(NotBelieved)
    );
    // There, but in another folder than the one named.
    assert_eq!(
        sheet.alo_chose(Path::new(DOCUMENTS), "march.txt", &disk),
        Err(NotBelieved)
    );
    // A name differing only in case is another name.
    assert_eq!(
        sheet.alo_chose(Path::new(DOCUMENTS), "Notes.txt", &disk),
        Err(NotBelieved)
    );
}

/// To open is to open a file: a folder's name is not one, even one that looks
/// like a file's.
#[test]
fn to_open_a_folders_name_is_not_believed() {
    let disk = alos_disk();
    let sheet = to_open(&disk);
    for name in ["Letters", "notes.txt.d"] {
        assert_eq!(
            sheet.alo_chose(Path::new(DOCUMENTS), name, &disk),
            Err(NotBelieved),
            "{name}"
        );
    }
}

/// `not-believed`: a folder that is not one — not there at all.
#[test]
fn to_open_in_a_folder_that_is_not_there_is_not_believed() {
    let disk = alos_disk();
    let sheet = to_open(&disk);
    assert_eq!(
        sheet.alo_chose(Path::new("/var/home/alo/Gone"), "notes.txt", &disk),
        Err(NotBelieved)
    );
    // A file's path given as the folder is not a folder.
    assert_eq!(
        sheet.alo_chose(
            Path::new("/var/home/alo/Documents/notes.txt"),
            "notes.txt",
            &disk
        ),
        Err(NotBelieved)
    );
}

/// The person's own press could not reach into a folder their account cannot
/// read, so alo's answer from inside one is not believed either.
#[test]
fn to_open_in_a_folder_that_would_not_be_read_is_not_believed() {
    let disk = alos_disk();
    let sheet = to_open(&disk);
    assert_eq!(
        sheet.alo_chose(Path::new("/var/home/alo/Private"), "secret.txt", &disk),
        Err(NotBelieved)
    );
}

/// A name is one name: nothing that is not one is believed to open.
#[test]
fn to_open_a_name_that_is_not_one_name_is_not_believed() {
    let disk = alos_disk();
    let sheet = to_open(&disk);
    for name in NOT_ONE_NAME {
        assert_eq!(
            sheet.alo_chose(Path::new(DOCUMENTS), name, &disk),
            Err(NotBelieved),
            "{name:?}"
        );
    }
    // `a/b` that would name a real file through the folder below is still not
    // one name.
    assert_eq!(
        sheet.alo_chose(Path::new(DOCUMENTS), "Letters/march.txt", &disk),
        Err(NotBelieved)
    );
}

/// To save: a new name in a folder that is there is believed, and the place
/// chosen is that name in that folder.
#[test]
fn to_save_a_new_name_in_a_folder_that_is_there_is_believed() {
    let disk = alos_disk();
    let sheet = to_save(&disk);
    assert_eq!(
        sheet.alo_chose(Path::new(DOCUMENTS), "final.txt", &disk),
        Ok(PathBuf::from(DOCUMENTS).join("final.txt"))
    );
    assert_eq!(
        sheet.alo_chose(
            Path::new("/var/home/alo/Bücher Neu"),
            "Kapitel 1 – Entwurf.md",
            &disk
        ),
        Ok(PathBuf::from("/var/home/alo/Bücher Neu").join("Kapitel 1 – Entwurf.md"))
    );
}

/// The name alo chooses is the one chosen, not the one the application
/// suggested — and it is chosen in the folder named, wherever the sheet is.
#[test]
fn to_save_the_name_alo_gives_is_the_one_chosen() {
    let disk = alos_disk();
    let sheet = to_save(&disk);
    assert_eq!(
        sheet.alo_chose(Path::new("/var/home/alo"), "final.txt", &disk),
        Ok(PathBuf::from("/var/home/alo").join("final.txt"))
    );
}

/// A file already there may be saved over: the sheet's second press is the
/// person's safeguard, and the replacing is the application's.
#[test]
fn to_save_over_a_file_that_is_there_is_believed() {
    let disk = alos_disk();
    let sheet = to_save(&disk);
    assert_eq!(
        sheet.alo_chose(Path::new(DOCUMENTS), "notes.txt", &disk),
        Ok(PathBuf::from(DOCUMENTS).join("notes.txt"))
    );
}

/// `not-believed`: a folder's name to save under.
#[test]
fn to_save_under_a_folders_name_is_not_believed() {
    let disk = alos_disk();
    let sheet = to_save(&disk);
    for name in ["Letters", "notes.txt.d"] {
        assert_eq!(
            sheet.alo_chose(Path::new(DOCUMENTS), name, &disk),
            Err(NotBelieved),
            "{name}"
        );
    }
}

/// `not-believed`: a folder that is not one, to save in.
#[test]
fn to_save_in_a_folder_that_is_not_there_is_not_believed() {
    let disk = alos_disk();
    let sheet = to_save(&disk);
    assert_eq!(
        sheet.alo_chose(Path::new("/var/home/alo/Gone"), "final.txt", &disk),
        Err(NotBelieved)
    );
    assert_eq!(
        sheet.alo_chose(
            Path::new("/var/home/alo/Documents/notes.txt"),
            "final.txt",
            &disk
        ),
        Err(NotBelieved)
    );
}

/// A folder the person cannot read is not one they could have saved in from
/// the sheet.
#[test]
fn to_save_in_a_folder_that_would_not_be_read_is_not_believed() {
    let disk = alos_disk();
    let sheet = to_save(&disk);
    assert_eq!(
        sheet.alo_chose(Path::new("/var/home/alo/Private"), "final.txt", &disk),
        Err(NotBelieved)
    );
}

/// A name to save under is one name: not empty, not `.` or `..`, no `/`.
#[test]
fn to_save_a_name_that_is_not_one_name_is_not_believed() {
    let disk = alos_disk();
    let sheet = to_save(&disk);
    for name in NOT_ONE_NAME {
        assert_eq!(
            sheet.alo_chose(Path::new(DOCUMENTS), name, &disk),
            Err(NotBelieved),
            "{name:?}"
        );
    }
}

/// A name is at most 255 bytes, and not one with a control character in it.
#[test]
fn to_save_a_name_too_long_or_with_a_control_character_is_not_believed() {
    let disk = alos_disk();
    let sheet = to_save(&disk);
    let longest = "a".repeat(255);
    assert_eq!(
        sheet.alo_chose(Path::new(DOCUMENTS), &longest, &disk),
        Ok(PathBuf::from(DOCUMENTS).join(&longest))
    );
    for name in [
        "a".repeat(256),
        "ü".repeat(128),
        "no\ntes.txt".to_owned(),
        "\0".to_owned(),
    ] {
        assert_eq!(
            sheet.alo_chose(Path::new(DOCUMENTS), &name, &disk),
            Err(NotBelieved),
            "{name:?}"
        );
    }
}

/// What the sheet shows, all of it the person can see.
#[derive(Debug, PartialEq)]
struct Shown {
    /// Each row's name and kind.
    rows: Vec<(String, Kind)>,
    /// The row with focus.
    focus: usize,
    /// The folder the sheet stands in.
    at: PathBuf,
    /// The name to save under.
    name: String,
    /// The note under the name, as written for a reader.
    note: Option<String>,
}

/// What `sheet` shows now.
fn shown(sheet: &Chooser) -> Shown {
    Shown {
        rows: sheet
            .rows()
            .iter()
            .map(|row| (row.name.to_owned(), row.kind))
            .collect(),
        focus: sheet.focus(),
        at: sheet.at().to_path_buf(),
        name: sheet.name().to_owned(),
        note: sheet.note().map(|note| format!("{note:?}")),
    }
}

/// Every answer alo could give, believed or not, against a sheet.
fn every_answer(sheet: &Chooser, disk: &Disk) {
    for (folder, name) in [
        (DOCUMENTS, "notes.txt"),
        (DOCUMENTS, "final.txt"),
        (DOCUMENTS, "Letters"),
        (DOCUMENTS, ".."),
        (DOCUMENTS, ""),
        (DOCUMENTS, "a/b"),
        ("/var/home/alo/Documents/Letters", "march.txt"),
        ("/var/home/alo/Gone", "notes.txt"),
        ("/var/home/alo/Private", "secret.txt"),
    ] {
        let before = shown(sheet);
        let whole = sheet.clone();
        let _ = sheet.alo_chose(Path::new(folder), name, disk);
        assert_eq!(shown(sheet), before, "{folder} / {name:?}");
        assert_eq!(*sheet, whole, "{folder} / {name:?}");
    }
}

/// The sheet stays up for the person, as it was: no answer alo gives, to open,
/// moves a row, the focus or the folder.
#[test]
fn no_answer_changes_a_sheet_to_open() {
    let disk = alos_disk();
    let mut sheet = to_open(&disk);
    // The person has moved the focus away from where it started.
    let _ = sheet.key(Key::Down, &disk);
    let _ = sheet.key(Key::Down, &disk);
    every_answer(&sheet, &disk);
}

/// The sheet stays up for the person, as it was: no answer alo gives, to save,
/// changes the name they were typing or the note under it.
#[test]
fn no_answer_changes_a_sheet_to_save() {
    let disk = alos_disk();
    let mut sheet = to_save(&disk);
    every_answer(&sheet, &disk);

    // The person has typed a name that is there and pressed Save once, so the
    // sheet is saying the name will be replaced; alo's answers leave that too.
    sheet.pressed_the_name();
    for _ in 0.."draft.txt".len() {
        let _ = sheet.key(Key::Erase, &disk);
    }
    for letter in "notes.txt".chars() {
        let _ = sheet.key(Key::Letter(letter), &disk);
    }
    let _ = sheet.pressed_the_button(&disk);
    every_answer(&sheet, &disk);
}

/// After an answer that is not believed, the sheet is still the person's: their
/// own press still chooses what it would have.
#[test]
fn after_an_answer_not_believed_the_person_can_still_choose() {
    let disk = alos_disk();
    let sheet = to_save(&disk);
    let mut untouched = sheet.clone();
    let mut answered = sheet;
    assert_eq!(
        answered.alo_chose(Path::new(DOCUMENTS), "Letters", &disk),
        Err(NotBelieved)
    );
    assert_eq!(
        answered.pressed_the_button(&disk),
        untouched.pressed_the_button(&disk)
    );
}

/// alo is answering the sheet, not walking it: a sheet standing in a folder
/// that would not open still believes a file elsewhere that is there.
#[test]
fn a_sheet_where_its_folder_did_not_open_still_believes_a_file_that_is_there() {
    let disk = alos_disk();
    let sheet = Chooser::to_open(Path::new("/var/home/alo/Gone"), &disk);
    assert_eq!(
        sheet.alo_chose(Path::new(DOCUMENTS), "notes.txt", &disk),
        Ok(PathBuf::from(DOCUMENTS).join("notes.txt"))
    );
}
