//! **The file chooser, tested from its promise and not from its code.** Written
//! by a second reader who read `docs/design/the-file-chooser.md` and the
//! `choose-file` section of `docs/contracts/application-door.md`, and none of
//! the implementation. Every test holds one sentence of those two: what rows
//! the person sees and in what order, what each key and press does, when a
//! name may be saved and why it may not, and that the person may go anywhere
//! their own account can read, up to the root. The refusals — a folder that
//! will not open, a name that is not one, a name a folder has, a file that is
//! there already — are tested as carefully as the choosing.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use alo_file_chooser::{
    Chooser, Ending, Folders, How, Key, Kind, Listing, MOST_SHOWN, NotListed, Note, Where,
};

/// A disk held in memory: what each folder lists, and the folders that are
/// there but would not be read.
#[derive(Default)]
struct Disk {
    /// Each folder, with its folders and its files, unsorted and with hidden
    /// names in, as a disk gives them.
    listed: BTreeMap<PathBuf, (Vec<String>, Vec<String>)>,
    /// Folders the person's account may not read.
    unreadable: BTreeSet<PathBuf>,
}

impl Disk {
    /// This disk, with `at` holding these folders and files.
    fn with(mut self, at: &str, folders: &[&str], files: &[&str]) -> Self {
        self.listed.insert(
            PathBuf::from(at),
            (
                folders.iter().map(|name| (*name).to_owned()).collect(),
                files.iter().map(|name| (*name).to_owned()).collect(),
            ),
        );
        self
    }

    /// This disk, with `at` there but not readable.
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
        self.listed
            .get(folder)
            .map(|(folders, files)| Listing::of(folders.clone(), files.clone()))
            .ok_or(NotListed::NotThere)
    }

    /// Added by the author after this file's own finding 1: the disk is
    /// asked, hidden names and all, never the listing.
    fn holds(&self, folder: &Path, name: &str) -> Option<Kind> {
        let (folders, files) = self.listed.get(folder)?;
        if folders.iter().any(|one| one == name) {
            Some(Kind::Folder)
        } else {
            files.iter().any(|one| one == name).then_some(Kind::File)
        }
    }
}

/// Anna's machine: her home holds three folders (one she may not read, and
/// one that has gone), two files and a hidden file; everything above it, up to
/// the root, can be read.
fn annas_disk() -> Disk {
    Disk::default()
        .with("/", &["home", "etc"], &[])
        .with("/home", &["anna"], &[])
        .with("/etc", &[], &["hostname"])
        .with(
            "/home/anna",
            &["Music", "Letters", "Private", "Gone", ".config"],
            &["notes.md", "march.txt", ".bashrc"],
        )
        .with("/home/anna/Letters", &[], &["to Rui.txt"])
        .with("/home/anna/Music", &[], &[])
        .with("/home/anna/.config", &[], &[])
        .unreadable("/home/anna/Private")
}

/// Anna's home.
fn home() -> &'static Path {
    Path::new("/home/anna")
}

/// The name of the row the keyboard is on.
fn focused(chooser: &Chooser) -> Option<String> {
    chooser
        .rows()
        .get(chooser.focus())
        .map(|row| row.name.to_owned())
}

/// Where in the rows the row called `name` is.
fn row_named(chooser: &Chooser, name: &str) -> usize {
    chooser
        .rows()
        .iter()
        .position(|row| row.name == name)
        .unwrap()
}

/// Moves the keyboard onto the row called `name` with one click, which only
/// moves it.
fn onto(chooser: &mut Chooser, name: &str, disk: &Disk) {
    let row = row_named(chooser, name);
    if chooser.focus() != row {
        assert_eq!(chooser.pressed(row, disk), None);
    }
    assert_eq!(chooser.focus(), row);
}

/// Types `words` into the name, one letter at a time.
fn typed(chooser: &mut Chooser, words: &str, disk: &Disk) {
    for letter in words.chars() {
        assert_eq!(chooser.key(Key::Letter(letter), disk), None);
    }
}

/// Erases the whole name.
fn erased(chooser: &mut Chooser, disk: &Disk) {
    while !chooser.name().is_empty() {
        let before = chooser.name().len();
        assert_eq!(chooser.key(Key::Erase, disk), None);
        assert!(chooser.name().len() < before, "Erase did not erase");
    }
}

/// The folder above first, then folders, then files, each by name.
#[test]
fn the_rows_are_the_folder_above_then_folders_then_files() {
    let disk = annas_disk();
    let chooser = Chooser::to_open(home(), &disk);
    assert!(matches!(chooser.how(), How::Open));
    assert_eq!(chooser.at(), home());
    let rows = chooser.rows();
    let seen: Vec<(&str, &str)> = rows
        .iter()
        .map(|row| {
            let kind = match row.kind {
                Kind::Above => "above",
                Kind::Folder => "folder",
                Kind::File => "file",
            };
            (kind, row.name)
        })
        .collect();
    assert_eq!(seen.len(), 7, "{seen:?}");
    assert_eq!(seen.first().map(|row| row.0), Some("above"));
    assert_eq!(
        seen.get(1..).unwrap(),
        [
            ("folder", "Gone"),
            ("folder", "Letters"),
            ("folder", "Music"),
            ("folder", "Private"),
            ("file", "march.txt"),
            ("file", "notes.md"),
        ]
    );
}

/// Folders before files, ordered by name with case set aside.
#[test]
fn names_are_ordered_with_case_set_aside() {
    let listing = Listing::of(
        vec!["crates".into(), "Archive".into(), "beta".into()],
        vec!["Zed.txt".into(), "apple.txt".into(), "Banana.txt".into()],
    );
    assert_eq!(listing.folders(), ["Archive", "beta", "crates"]);
    assert_eq!(listing.files(), ["apple.txt", "Banana.txt", "Zed.txt"]);

    let disk = Disk::default().with(
        "/srv",
        &["crates", "Archive", "beta"],
        &["Zed.txt", "apple.txt", "Banana.txt"],
    );
    let chooser = Chooser::to_open(Path::new("/srv"), &disk);
    let names: Vec<&str> = chooser.rows().iter().map(|row| row.name).collect();
    assert_eq!(
        names.get(1..).unwrap(),
        [
            "Archive",
            "beta",
            "crates",
            "apple.txt",
            "Banana.txt",
            "Zed.txt"
        ]
    );
}

/// A name beginning with a dot is hidden, and counted as not shown.
#[test]
fn hidden_names_are_not_shown_but_counted() {
    let listing = Listing::of(
        vec![".config".into(), "Music".into()],
        vec![".bashrc".into(), ".profile".into(), "notes.md".into()],
    );
    assert_eq!(listing.folders(), ["Music"]);
    assert_eq!(listing.files(), ["notes.md"]);
    assert_eq!(listing.not_shown(), 3);

    let disk = annas_disk();
    let chooser = Chooser::to_open(home(), &disk);
    assert!(chooser.rows().iter().all(|row| !row.name.starts_with('.')));
    assert_eq!(chooser.listing().not_shown(), 2);
}

/// A folder shows at most `MOST_SHOWN` names, and says there are more.
#[test]
fn a_large_folder_shows_at_most_the_most_shown_and_says_there_are_more() {
    assert_eq!(MOST_SHOWN, 1000);
    let many: Vec<String> = (0..=MOST_SHOWN).map(|n| format!("f{n:04}.txt")).collect();
    let listing = Listing::of(vec![], many);
    assert!(listing.folders().len() + listing.files().len() <= MOST_SHOWN);
    assert!(listing.there_are_more());

    let few = Listing::of(vec!["a".into()], vec!["b.txt".into()]);
    assert!(!few.there_are_more());
}

/// At the root there is no folder above, so its first row is a folder.
#[test]
fn at_the_root_there_is_no_folder_above() {
    let disk = annas_disk();
    let chooser = Chooser::to_open(Path::new("/"), &disk);
    assert_eq!(chooser.at(), Path::new("/"));
    assert!(
        chooser
            .rows()
            .iter()
            .all(|row| !matches!(row.kind, Kind::Above))
    );
    let names: Vec<&str> = chooser.rows().iter().map(|row| row.name).collect();
    assert_eq!(names, ["etc", "home"]);
}

/// "The person may go anywhere their own account can read, up to the root":
/// Backspace goes up, all the way, and at the root it goes nowhere and ends
/// nothing.
#[test]
fn the_person_can_go_up_all_the_way_to_the_root() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_open(home(), &disk);
    assert_eq!(chooser.key(Key::Erase, &disk), None);
    assert_eq!(chooser.at(), Path::new("/home"));
    assert_eq!(chooser.key(Key::Erase, &disk), None);
    assert_eq!(chooser.at(), Path::new("/"));
    assert_eq!(chooser.key(Key::Erase, &disk), None);
    assert_eq!(chooser.at(), Path::new("/"));
    assert!(
        chooser
            .rows()
            .iter()
            .all(|row| !matches!(row.kind, Kind::Above))
    );

    // And from the root, into a folder that is not the person's home at all.
    onto(&mut chooser, "etc", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert_eq!(chooser.at(), Path::new("/etc"));
    onto(&mut chooser, "hostname", &disk);
    assert_eq!(
        chooser.key(Key::Enter, &disk),
        Some(Ending::Chosen(PathBuf::from("/etc/hostname")))
    );
}

/// Enter on the folder above goes up, as Backspace does.
#[test]
fn enter_on_the_folder_above_goes_up() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_open(Path::new("/home/anna/Letters"), &disk);
    assert_eq!(chooser.key(Key::First, &disk), None);
    assert!(matches!(
        chooser.rows().get(chooser.focus()),
        Some(row) if matches!(row.kind, Kind::Above)
    ));
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert_eq!(chooser.at(), home());
}

/// Enter on a folder goes in; Enter on a file chooses it, as a whole path.
#[test]
fn enter_on_a_folder_goes_in_and_on_a_file_chooses_it() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_open(home(), &disk);
    onto(&mut chooser, "Letters", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert_eq!(chooser.at(), Path::new("/home/anna/Letters"));
    assert!(chooser.note().is_none());
    onto(&mut chooser, "to Rui.txt", &disk);
    assert_eq!(
        chooser.key(Key::Enter, &disk),
        Some(Ending::Chosen(PathBuf::from(
            "/home/anna/Letters/to Rui.txt"
        )))
    );
}

/// Up and Down move, Home and End go to the ends, and the keyboard never
/// leaves the rows.
#[test]
fn up_down_home_and_end_move_within_the_rows() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_open(home(), &disk);
    let last = chooser.rows().len() - 1;

    assert_eq!(chooser.key(Key::First, &disk), None);
    assert_eq!(chooser.focus(), 0);
    assert_eq!(chooser.key(Key::Up, &disk), None);
    assert_eq!(chooser.focus(), 0);
    assert_eq!(chooser.key(Key::Down, &disk), None);
    assert_eq!(chooser.focus(), 1);
    assert_eq!(chooser.key(Key::Last, &disk), None);
    assert_eq!(chooser.focus(), last);
    assert_eq!(chooser.key(Key::Down, &disk), None);
    assert!(chooser.focus() <= last);
    assert_eq!(chooser.key(Key::Up, &disk), None);
    assert!(chooser.focus() < last);
    assert_eq!(chooser.at(), home(), "moving went somewhere");
}

/// A letter jumps to the next name that begins with it.
#[test]
fn a_letter_jumps_to_the_next_name_beginning_with_it() {
    let disk = Disk::default().with(
        "/fruit",
        &["Archive", "beta"],
        &["apple.txt", "banana.txt", "berry.txt"],
    );
    let mut chooser = Chooser::to_open(Path::new("/fruit"), &disk);
    assert_eq!(chooser.key(Key::First, &disk), None);
    assert_eq!(chooser.key(Key::Letter('b'), &disk), None);
    assert_eq!(focused(&chooser).as_deref(), Some("beta"));
    assert_eq!(chooser.key(Key::Letter('b'), &disk), None);
    assert_eq!(focused(&chooser).as_deref(), Some("banana.txt"));
    assert_eq!(chooser.key(Key::Letter('b'), &disk), None);
    assert_eq!(focused(&chooser).as_deref(), Some("berry.txt"));
    assert_eq!(chooser.at(), Path::new("/fruit"));
}

/// Escape cancels an open, wherever the person has gone.
#[test]
fn escape_cancels_an_open() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_open(home(), &disk);
    onto(&mut chooser, "Letters", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert_eq!(chooser.key(Key::Escape, &disk), Some(Ending::Cancelled));
}

/// Escape cancels a save, from the name and from the rows alike, and never
/// saves the name it holds.
#[test]
fn escape_cancels_a_save() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "new.txt", &disk);
    assert!(matches!(chooser.on(), Where::TheName));
    assert_eq!(chooser.key(Key::Escape, &disk), Some(Ending::Cancelled));

    let mut chooser = Chooser::to_save(home(), "new.txt", &disk);
    assert_eq!(chooser.key(Key::Tab, &disk), None);
    assert!(matches!(chooser.on(), Where::TheRows));
    assert_eq!(chooser.key(Key::Escape, &disk), Some(Ending::Cancelled));
}

/// A folder the person's account may not read keeps them where they were,
/// and the sheet says it could not open it.
#[test]
fn a_folder_that_would_not_be_read_keeps_the_person_where_they_were() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_open(home(), &disk);
    let rows_before: Vec<String> = chooser
        .rows()
        .iter()
        .map(|row| row.name.to_owned())
        .collect();
    onto(&mut chooser, "Private", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert_eq!(chooser.at(), home());
    assert!(matches!(chooser.note(), Some(Note::CouldNotOpen(_))));
    let rows_after: Vec<String> = chooser
        .rows()
        .iter()
        .map(|row| row.name.to_owned())
        .collect();
    assert_eq!(rows_after, rows_before);
}

/// A folder that has gone since it was listed is the same: the person stays
/// where they were and is told.
#[test]
fn a_folder_that_is_no_longer_there_keeps_the_person_where_they_were() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_open(home(), &disk);
    onto(&mut chooser, "Gone", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert_eq!(chooser.at(), home());
    assert!(matches!(chooser.note(), Some(Note::CouldNotOpen(_))));
    assert_eq!(focused(&chooser).as_deref(), Some("Gone"));
}

/// A reader's press or a second click on a folder that will not open is
/// refused the same way as Enter.
#[test]
fn every_way_into_a_folder_that_will_not_open_is_refused() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_open(home(), &disk);
    let private = row_named(&chooser, "Private");
    assert_eq!(chooser.a_reader_pressed(private, &disk), None);
    assert_eq!(chooser.at(), home());
    assert!(matches!(chooser.note(), Some(Note::CouldNotOpen(_))));

    let mut chooser = Chooser::to_open(home(), &disk);
    onto(&mut chooser, "Private", &disk);
    let private = chooser.focus();
    assert_eq!(chooser.pressed(private, &disk), None);
    assert_eq!(chooser.at(), home());
    assert!(matches!(chooser.note(), Some(Note::CouldNotOpen(_))));
}

/// Saving starts in the name, holding the name the application suggested.
#[test]
fn a_save_starts_in_the_name_holding_the_suggestion() {
    let disk = annas_disk();
    let chooser = Chooser::to_save(home(), "Untitled.txt", &disk);
    assert!(matches!(chooser.how(), How::Save));
    assert!(matches!(chooser.on(), Where::TheName));
    assert_eq!(chooser.name(), "Untitled.txt");
    assert_eq!(chooser.at(), home());
    assert!(chooser.note().is_none());
}

/// Tab and Shift-Tab move between the name and the rows, and change nothing
/// else.
#[test]
fn tab_moves_between_the_name_and_the_rows() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "Untitled.txt", &disk);
    assert_eq!(chooser.key(Key::Tab, &disk), None);
    assert!(matches!(chooser.on(), Where::TheRows));
    assert_eq!(chooser.key(Key::Tab, &disk), None);
    assert!(matches!(chooser.on(), Where::TheName));
    assert_eq!(chooser.key(Key::BackTab, &disk), None);
    assert!(matches!(chooser.on(), Where::TheRows));
    assert_eq!(chooser.key(Key::BackTab, &disk), None);
    assert!(matches!(chooser.on(), Where::TheName));
    assert_eq!(chooser.name(), "Untitled.txt");
    assert_eq!(chooser.at(), home());
}

/// In the name, a letter is typed and Backspace erases one letter, not one
/// byte — and neither moves the person.
#[test]
fn in_the_name_letters_are_typed_and_erased() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "a", &disk);
    typed(&mut chooser, "bü", &disk);
    assert_eq!(chooser.name(), "abü");
    assert_eq!(chooser.key(Key::Erase, &disk), None);
    assert_eq!(chooser.name(), "ab");
    assert_eq!(chooser.at(), home());
    assert!(matches!(chooser.on(), Where::TheName));
}

/// Enter in the name saves, in the folder the person is in.
#[test]
fn enter_in_the_name_saves_where_the_person_is() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "new letter.txt", &disk);
    assert_eq!(
        chooser.key(Key::Enter, &disk),
        Some(Ending::Chosen(PathBuf::from("/home/anna/new letter.txt")))
    );
}

/// "Where it starts is only where it starts": a save lands where the person
/// went, the root included.
#[test]
fn a_save_lands_where_the_person_went() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "new.txt", &disk);
    let letters = row_named(&chooser, "Letters");
    assert_eq!(chooser.a_reader_pressed(letters, &disk), None);
    assert_eq!(chooser.at(), Path::new("/home/anna/Letters"));
    assert_eq!(chooser.name(), "new.txt");
    assert_eq!(
        chooser.pressed_the_button(&disk),
        Some(Ending::Chosen(PathBuf::from("/home/anna/Letters/new.txt")))
    );

    let mut chooser = Chooser::to_save(home(), "new.txt", &disk);
    assert_eq!(chooser.key(Key::Tab, &disk), None);
    assert_eq!(chooser.key(Key::Erase, &disk), None);
    assert_eq!(chooser.key(Key::Erase, &disk), None);
    assert_eq!(chooser.at(), Path::new("/"));
    chooser.pressed_the_name();
    assert!(matches!(chooser.on(), Where::TheName));
    assert_eq!(
        chooser.key(Key::Enter, &disk),
        Some(Ending::Chosen(PathBuf::from("/new.txt")))
    );
}

/// A save with no name saves nothing and says there is no name, whether it
/// began empty or was erased.
#[test]
fn a_save_with_no_name_saves_nothing_and_says_so() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert!(matches!(chooser.note(), Some(Note::NoName)));
    assert_eq!(chooser.pressed_the_button(&disk), None);
    assert!(matches!(chooser.note(), Some(Note::NoName)));

    let mut chooser = Chooser::to_save(home(), "ab", &disk);
    erased(&mut chooser, &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert!(matches!(chooser.note(), Some(Note::NoName)));
}

/// `.` and `..` are not names, and saving under either saves nothing.
#[test]
fn a_save_named_dot_or_dot_dot_saves_nothing_and_says_so() {
    let disk = annas_disk();
    for name in [".", ".."] {
        let mut chooser = Chooser::to_save(home(), "", &disk);
        typed(&mut chooser, name, &disk);
        assert_eq!(chooser.key(Key::Enter, &disk), None, "{name:?} was saved");
        assert!(matches!(chooser.note(), Some(Note::NotAName)), "{name:?}");
        assert_eq!(
            chooser.pressed_the_button(&disk),
            None,
            "{name:?} was saved"
        );
        assert_eq!(chooser.at(), home(), "{name:?} moved the person");
    }
}

/// A name holding `/` is not one name, and saving under it saves nothing —
/// never somewhere the `/` leads.
#[test]
fn a_save_name_with_a_slash_saves_nothing() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "", &disk);
    typed(&mut chooser, "Letters/x.txt", &disk);
    let ending = chooser.key(Key::Enter, &disk);
    if chooser.name().contains('/') {
        assert_eq!(ending, None);
        assert!(matches!(chooser.note(), Some(Note::NotAName)));
    } else if let Some(Ending::Chosen(path)) = ending {
        assert_eq!(path.parent(), Some(home()), "the slash led somewhere");
    }

    let mut chooser = Chooser::to_save(home(), "../../etc/passwd", &disk);
    let ending = chooser.key(Key::Enter, &disk);
    if chooser.name().contains('/') {
        assert_eq!(ending, None);
        assert!(matches!(chooser.note(), Some(Note::NotAName)));
    } else if let Some(Ending::Chosen(path)) = ending {
        assert_eq!(path.parent(), Some(home()), "the slash led somewhere");
    }
}

/// A name of more than 255 bytes is not one, and saves nothing.
#[test]
fn a_name_longer_than_the_longest_saves_nothing() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "", &disk);
    typed(&mut chooser, &"a".repeat(256), &disk);
    let ending = chooser.key(Key::Enter, &disk);
    if chooser.name().len() > 255 {
        assert_eq!(ending, None);
        assert!(matches!(chooser.note(), Some(Note::NotAName)));
    } else {
        assert!(matches!(ending, Some(Ending::Chosen(_))));
    }
}

/// A name a folder has saves nothing, on the first press or any after it.
#[test]
fn a_name_a_folder_has_saves_nothing() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "Letters", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert!(matches!(chooser.note(), Some(Note::AFolderHasThisName)));
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert!(matches!(chooser.note(), Some(Note::AFolderHasThisName)));
    assert_eq!(chooser.pressed_the_button(&disk), None);
    assert_eq!(
        chooser.at(),
        home(),
        "saving under a folder's name went into it"
    );
}

/// A name that is there already is not replaced on the first Enter; the sheet
/// says so, and a second Enter with the name unchanged replaces it.
#[test]
fn a_name_that_is_there_already_is_replaced_only_on_the_second_enter() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "march.txt", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert!(matches!(chooser.note(), Some(Note::ReplacesWhatIsThere)));
    assert_eq!(
        chooser.key(Key::Enter, &disk),
        Some(Ending::Chosen(PathBuf::from("/home/anna/march.txt")))
    );
}

/// The Save button asks twice the same way.
#[test]
fn the_save_button_also_asks_before_replacing() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "notes.md", &disk);
    assert_eq!(chooser.pressed_the_button(&disk), None);
    assert!(matches!(chooser.note(), Some(Note::ReplacesWhatIsThere)));
    assert_eq!(
        chooser.pressed_the_button(&disk),
        Some(Ending::Chosen(PathBuf::from("/home/anna/notes.md")))
    );
}

/// Changing the name withdraws the warning: the next press asks again, even
/// when the name was changed back.
#[test]
fn changing_the_name_withdraws_the_warning() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "march.txt", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert!(matches!(chooser.note(), Some(Note::ReplacesWhatIsThere)));

    // Renamed to another file that is there: that one is asked about first.
    erased(&mut chooser, &disk);
    typed(&mut chooser, "notes.md", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert!(matches!(chooser.note(), Some(Note::ReplacesWhatIsThere)));

    // Changed and changed back: the name was not unchanged between the presses.
    typed(&mut chooser, "x", &disk);
    assert_eq!(chooser.key(Key::Erase, &disk), None);
    assert_eq!(chooser.name(), "notes.md");
    assert_eq!(
        chooser.key(Key::Enter, &disk),
        None,
        "a changed name replaced a file"
    );
    assert_eq!(
        chooser.key(Key::Enter, &disk),
        Some(Ending::Chosen(PathBuf::from("/home/anna/notes.md")))
    );
}

/// Going to another folder withdraws the warning too: the same name there is
/// a different file.
#[test]
fn going_elsewhere_withdraws_the_warning() {
    let disk = annas_disk().with("/home/anna/Letters", &[], &["march.txt"]);
    let mut chooser = Chooser::to_save(home(), "march.txt", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert!(matches!(chooser.note(), Some(Note::ReplacesWhatIsThere)));
    let letters = row_named(&chooser, "Letters");
    assert_eq!(chooser.a_reader_pressed(letters, &disk), None);
    assert_eq!(chooser.at(), Path::new("/home/anna/Letters"));
    assert_eq!(chooser.name(), "march.txt");
    assert_eq!(
        chooser.pressed_the_button(&disk),
        None,
        "a file in another folder was replaced without asking"
    );
    assert!(matches!(chooser.note(), Some(Note::ReplacesWhatIsThere)));
}

/// A hidden name may still be saved under by typing it.
#[test]
fn a_hidden_name_may_be_typed_and_saved() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), ".profile", &disk);
    assert_eq!(
        chooser.key(Key::Enter, &disk),
        Some(Ending::Chosen(PathBuf::from("/home/anna/.profile")))
    );
}

/// A hidden file is still there: saving under its name asks first, though it
/// is not shown.
#[test]
fn a_hidden_file_that_is_there_is_not_replaced_on_the_first_enter() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), ".bashrc", &disk);
    assert_eq!(
        chooser.key(Key::Enter, &disk),
        None,
        "a hidden file was replaced without asking"
    );
    assert!(matches!(chooser.note(), Some(Note::ReplacesWhatIsThere)));
}

/// A hidden folder is still a folder: its name saves nothing.
#[test]
fn a_hidden_folders_name_saves_nothing() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), ".config", &disk);
    assert_eq!(
        chooser.key(Key::Enter, &disk),
        None,
        "a save was chosen under a hidden folder's name"
    );
    assert!(matches!(chooser.note(), Some(Note::AFolderHasThisName)));
}

/// A file past the most shown is still there: saving under its name asks
/// first.
#[test]
fn a_file_past_the_most_shown_is_not_replaced_on_the_first_enter() {
    let many: Vec<String> = (0..=MOST_SHOWN).map(|n| format!("f{n:04}.txt")).collect();
    let names: Vec<&str> = many.iter().map(String::as_str).collect();
    let disk = Disk::default().with("/big", &[], &names);
    let last = format!("f{MOST_SHOWN:04}.txt");
    let mut chooser = Chooser::to_save(Path::new("/big"), &last, &disk);
    assert_eq!(
        chooser.key(Key::Enter, &disk),
        None,
        "a file nobody was shown was replaced without asking"
    );
    assert!(matches!(chooser.note(), Some(Note::ReplacesWhatIsThere)));
}

/// Enter on a file, when saving, takes its name and saves nothing yet.
#[test]
fn enter_on_a_file_when_saving_takes_its_name() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "Untitled.txt", &disk);
    assert_eq!(chooser.key(Key::Tab, &disk), None);
    onto(&mut chooser, "march.txt", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert_eq!(chooser.name(), "march.txt");
    assert_eq!(chooser.at(), home());
}

/// Enter on a folder, when saving, goes in and keeps the name.
#[test]
fn enter_on_a_folder_when_saving_goes_in() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "Untitled.txt", &disk);
    assert_eq!(chooser.key(Key::Tab, &disk), None);
    onto(&mut chooser, "Music", &disk);
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert_eq!(chooser.at(), Path::new("/home/anna/Music"));
    assert_eq!(chooser.name(), "Untitled.txt");
}

/// A click on a row the keyboard is not on only moves the keyboard there; a
/// click on the row it is on uses it, as Enter does.
#[test]
fn a_first_click_moves_the_keyboard_and_a_second_uses_the_row() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_open(home(), &disk);
    let march = row_named(&chooser, "march.txt");
    assert_ne!(chooser.focus(), march);
    assert_eq!(chooser.pressed(march, &disk), None);
    assert_eq!(chooser.focus(), march);
    assert_eq!(
        chooser.pressed(march, &disk),
        Some(Ending::Chosen(PathBuf::from("/home/anna/march.txt")))
    );

    let mut chooser = Chooser::to_open(home(), &disk);
    let music = row_named(&chooser, "Music");
    assert_ne!(chooser.focus(), music);
    assert_eq!(chooser.pressed(music, &disk), None);
    assert_eq!(chooser.at(), home(), "a first click went in");
    assert_eq!(chooser.pressed(music, &disk), None);
    assert_eq!(chooser.at(), Path::new("/home/anna/Music"));
}

/// When saving, a click on a row takes the keyboard out of the name, and a
/// click on the name puts it back.
#[test]
fn a_click_when_saving_moves_the_keyboard_between_name_and_rows() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_save(home(), "Untitled.txt", &disk);
    let notes = row_named(&chooser, "notes.md");
    assert_eq!(chooser.pressed(notes, &disk), None);
    assert!(matches!(chooser.on(), Where::TheRows));
    assert_eq!(chooser.name(), "Untitled.txt");
    chooser.pressed_the_name();
    assert!(matches!(chooser.on(), Where::TheName));
    assert_eq!(chooser.name(), "Untitled.txt");
}

/// A screen reader's press on a row uses it at once.
#[test]
fn a_readers_press_uses_the_row_at_once() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_open(home(), &disk);
    let march = row_named(&chooser, "march.txt");
    assert_ne!(chooser.focus(), march);
    assert_eq!(
        chooser.a_reader_pressed(march, &disk),
        Some(Ending::Chosen(PathBuf::from("/home/anna/march.txt")))
    );

    let mut chooser = Chooser::to_open(home(), &disk);
    let letters = row_named(&chooser, "Letters");
    assert_eq!(chooser.a_reader_pressed(letters, &disk), None);
    assert_eq!(chooser.at(), Path::new("/home/anna/Letters"));

    let mut chooser = Chooser::to_open(Path::new("/home/anna/Letters"), &disk);
    let above = chooser
        .rows()
        .iter()
        .position(|row| matches!(row.kind, Kind::Above))
        .unwrap();
    assert_eq!(chooser.a_reader_pressed(above, &disk), None);
    assert_eq!(chooser.at(), home());
}

/// A press past the last row does nothing at all: no ending, no move, no
/// note.
#[test]
fn a_press_outside_the_rows_does_nothing() {
    let disk = annas_disk();
    for saving in [false, true] {
        let mut chooser = if saving {
            Chooser::to_save(home(), "Untitled.txt", &disk)
        } else {
            Chooser::to_open(home(), &disk)
        };
        let focus = chooser.focus();
        let rows = chooser.rows().len();
        for outside in [rows, rows + 1, usize::MAX] {
            assert_eq!(chooser.pressed(outside, &disk), None);
            assert_eq!(chooser.pressed(outside, &disk), None);
            assert_eq!(chooser.a_reader_pressed(outside, &disk), None);
            assert_eq!(chooser.focus(), focus);
            assert_eq!(chooser.at(), home());
            assert!(chooser.note().is_none());
        }
        if saving {
            assert_eq!(chooser.name(), "Untitled.txt");
        }
    }
}

/// The Open button opens the file the keyboard is on.
#[test]
fn the_open_button_opens_the_file_the_keyboard_is_on() {
    let disk = annas_disk();
    let mut chooser = Chooser::to_open(home(), &disk);
    onto(&mut chooser, "notes.md", &disk);
    assert_eq!(
        chooser.pressed_the_button(&disk),
        Some(Ending::Chosen(PathBuf::from("/home/anna/notes.md")))
    );
}

/// "So the row the keyboard is on is always shown": for every page size the
/// display may hold, the rows shown hold the focus, fit the page and fit the
/// rows.
#[test]
fn the_rows_shown_always_hold_the_row_the_keyboard_is_on() {
    let many: Vec<String> = (0..50).map(|n| format!("f{n:02}.txt")).collect();
    let names: Vec<&str> = many.iter().map(String::as_str).collect();
    let disk = Disk::default().with("/big", &["a", "b", "c"], &names);
    let moves = || {
        std::iter::repeat_with(|| Key::Down).take(10).chain([
            Key::Last,
            Key::Up,
            Key::Up,
            Key::First,
            Key::Last,
            Key::Letter('f'),
            Key::Down,
            Key::Up,
        ])
    };
    for page in 3..=8 {
        let mut chooser = Chooser::to_open(Path::new("/big"), &disk);
        for key in moves() {
            assert_eq!(chooser.key(key, &disk), None);
            let shown = chooser.shown(page);
            assert!(
                shown.contains(&chooser.focus()),
                "page {page}: {shown:?} does not hold {}",
                chooser.focus()
            );
            assert!(shown.len() <= page, "page {page}: {shown:?}");
            assert!(shown.end <= chooser.rows().len(), "page {page}: {shown:?}");
        }
        onto(&mut chooser, "f30.txt", &disk);
        assert!(chooser.shown(page).contains(&chooser.focus()));
    }
}

/// Fewer rows than the page holds are all shown.
#[test]
fn fewer_rows_than_the_page_are_all_shown() {
    let disk = annas_disk();
    let chooser = Chooser::to_open(Path::new("/home/anna/Letters"), &disk);
    assert_eq!(chooser.rows().len(), 2);
    assert_eq!(chooser.shown(8), 0..2);
}
