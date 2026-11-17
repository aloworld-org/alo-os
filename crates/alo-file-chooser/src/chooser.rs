//! Where a person is standing in the file chooser, and everything they may do
//! there.
//!
//! # The rows
//!
//! The folder above comes first ([`Kind::Above`]) — except at the root, which
//! has none — then the folders in this one, then its files. The keyboard is on
//! one row at a time ([`Chooser::focus`]).
//!
//! # Opening
//!
//! - **Up and Down** move the keyboard; Home and End go to the first and last.
//! - **Enter** on a folder goes into it, on the folder above goes up, and on a
//!   file **chooses it**.
//! - **Backspace** goes up.
//! - **A letter** moves to the next row whose name begins with it.
//! - **Escape** cancels.
//!
//! # Saving
//!
//! The keyboard starts in **the name**, holding the name the application
//! suggested. Tab and Shift+Tab move between the name and the rows.
//!
//! - In the name, letters and Backspace change it; **Enter saves** it in the
//!   folder the person is standing in.
//! - On the rows, the keys are opening's, except that Enter on a file takes its
//!   name into the name and goes back there — the person still saves.
//! - **A name that is there already is not replaced on the first Enter**: the
//!   sheet says so ([`Note::ReplacesWhatIsThere`]) and a second Enter, with the
//!   name unchanged, replaces it. A name that is a folder's, or that is not one
//!   name at all, saves nothing and says why.
//!
//! # A person is never limited here
//!
//! The person may go anywhere their own account can read, up to the root.
//! Where the sheet started is only where it started. What alo may do on this
//! sheet is decided elsewhere and never narrows what the person may.

use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::listing::{Folders, Kind, Listing, NotListed};

/// What the application asked the person for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum How {
    /// A file to open.
    Open,
    /// A place and a name to save under.
    Save,
}

/// Where the keyboard is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Where {
    /// On the rows.
    TheRows,
    /// In the name, when saving.
    TheName,
}

/// One key, as the chooser understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// Up: the row before.
    Up,
    /// Down: the row after.
    Down,
    /// Home: the first row.
    First,
    /// End: the last row.
    Last,
    /// Enter.
    Enter,
    /// Backspace.
    Erase,
    /// Tab: from the name to the rows, or back.
    Tab,
    /// Shift+Tab: the same, the other way.
    BackTab,
    /// Escape: cancel.
    Escape,
    /// A letter typed.
    Letter(char),
}

/// A sentence the sheet says about what just happened, for the shell to put
/// in the person's language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Note {
    /// The folder pressed could not be opened, and the person is where they
    /// were.
    CouldNotOpen(String),
    /// The name is empty.
    NoName,
    /// The name is not one name: `.`, `..`, or with a `/` in it.
    NotAName,
    /// A folder in this folder already has this name.
    AFolderHasThisName,
    /// A file with this name is here; a second Enter replaces it.
    ReplacesWhatIsThere,
}

/// alo's answer could not have been the sheet's: a file not there to open,
/// or a folder's name to save under, or a folder that is not one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotBelieved;

/// How the sheet ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ending {
    /// The person chose this file — to open it, or to save there.
    Chosen(PathBuf),
    /// The person cancelled.
    Cancelled,
}

/// One row, as it is drawn and read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row<'a> {
    /// What it is called: a folder's or a file's name, or the folder above's.
    pub name: &'a str,
    /// What it is.
    pub kind: Kind,
}

/// The file chooser: where the person stands, and what is in front of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chooser {
    /// Opening or saving.
    how: How,
    /// The folder the person is standing in.
    at: PathBuf,
    /// What is in it.
    listing: Listing,
    /// The row the keyboard is on.
    focus: usize,
    /// Where the keyboard is.
    on: Where,
    /// The name, when saving.
    name: String,
    /// Whether the next Enter replaces the file the name names.
    replacing: bool,
    /// What the sheet says about what just happened.
    note: Option<Note>,
}

impl Chooser {
    /// A chooser to open a file, standing in `at`.
    ///
    /// A folder that cannot be listed is stood in with nothing shown and
    /// [`Note::CouldNotOpen`] said, so the person can still go up.
    #[must_use]
    pub fn to_open(at: &Path, folders: &dyn Folders) -> Self {
        Self::standing_in(How::Open, at, String::new(), folders)
    }

    /// A chooser to save a file, standing in `at`, with `name` suggested.
    #[must_use]
    pub fn to_save(at: &Path, name: &str, folders: &dyn Folders) -> Self {
        Self::standing_in(How::Save, at, name.to_owned(), folders)
    }

    /// Standing in `at`, either way.
    fn standing_in(how: How, at: &Path, name: String, folders: &dyn Folders) -> Self {
        let (listing, note) = match folders.listed(at) {
            Ok(listing) => (listing, None),
            Err(_) => (Listing::default(), Some(Note::CouldNotOpen(called(at)))),
        };
        let mut chooser = Self {
            how,
            at: at.to_path_buf(),
            listing,
            focus: 0,
            on: match how {
                How::Open => Where::TheRows,
                How::Save => Where::TheName,
            },
            name,
            replacing: false,
            note,
        };
        chooser.focus = chooser.the_first_inside();
        chooser
    }

    /// Opening or saving.
    #[must_use]
    pub const fn how(&self) -> How {
        self.how
    }

    /// The folder the person is standing in.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// What is in it.
    #[must_use]
    pub const fn listing(&self) -> &Listing {
        &self.listing
    }

    /// The row the keyboard is on.
    #[must_use]
    pub const fn focus(&self) -> usize {
        self.focus
    }

    /// Where the keyboard is.
    #[must_use]
    pub const fn on(&self) -> Where {
        self.on
    }

    /// The name, when saving; empty when opening.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What the sheet says about what just happened.
    #[must_use]
    pub const fn note(&self) -> Option<&Note> {
        self.note.as_ref()
    }

    /// Every row, in order: the folder above, the folders, the files.
    #[must_use]
    pub fn rows(&self) -> Vec<Row<'_>> {
        let mut rows = Vec::with_capacity(self.how_many_rows());
        if let Some(above) = self.above() {
            rows.push(Row {
                name: above,
                kind: Kind::Above,
            });
        }
        rows.extend(self.listing.folders().iter().map(|name| Row {
            name,
            kind: Kind::Folder,
        }));
        rows.extend(self.listing.files().iter().map(|name| Row {
            name,
            kind: Kind::File,
        }));
        rows
    }

    /// The rows to show when `how_many` fit: the page the keyboard is on, so
    /// the row it is on is always shown.
    #[must_use]
    pub fn shown(&self, how_many: usize) -> Range<usize> {
        let how_many = how_many.max(1);
        let from = (self.focus / how_many) * how_many;
        from..(from + how_many).min(self.how_many_rows())
    }

    /// A key.
    pub fn key(&mut self, key: Key, folders: &dyn Folders) -> Option<Ending> {
        let last = self.how_many_rows().saturating_sub(1);
        match key {
            Key::Escape => return Some(Ending::Cancelled),
            Key::Tab | Key::BackTab => {
                if self.how == How::Save {
                    self.on = match self.on {
                        Where::TheName => Where::TheRows,
                        Where::TheRows => Where::TheName,
                    };
                }
            }
            Key::Up => self.moved_to(self.focus.saturating_sub(1)),
            Key::Down => self.moved_to((self.focus + 1).min(last)),
            Key::First => self.moved_to(0),
            Key::Last => self.moved_to(last),
            Key::Enter => {
                return match self.on {
                    Where::TheRows => self.used(self.focus, folders),
                    Where::TheName => self.saved(folders),
                };
            }
            Key::Erase => match self.on {
                Where::TheRows => self.went_up(folders),
                Where::TheName => {
                    if self.name.pop().is_some() {
                        self.name_changed();
                    }
                }
            },
            Key::Letter(letter) => match self.on {
                Where::TheRows => self.jumped_to(letter),
                Where::TheName => {
                    if !letter.is_control()
                        && self.name.len() + letter.len_utf8() <= alo_acting::THE_LONGEST_NAME
                    {
                        self.name.push(letter);
                        self.name_changed();
                    }
                }
            },
        }
        None
    }

    /// A press on `row` — a click: the keyboard goes there, and a press on the
    /// row it is already on uses it, as Enter does.
    pub fn pressed(&mut self, row: usize, folders: &dyn Folders) -> Option<Ending> {
        if row >= self.how_many_rows() {
            return None;
        }
        if self.focus == row && self.on == Where::TheRows {
            return self.used(row, folders);
        }
        self.moved_to(row);
        None
    }

    /// A screen reader's press on `row`: it is used at once, as Enter on it is.
    pub fn a_reader_pressed(&mut self, row: usize, folders: &dyn Folders) -> Option<Ending> {
        if row >= self.how_many_rows() {
            return None;
        }
        self.moved_to(row);
        self.used(row, folders)
    }

    /// A press on the name: the keyboard goes there, when saving.
    pub fn pressed_the_name(&mut self) {
        if self.how == How::Save {
            self.on = Where::TheName;
        }
    }

    /// *Open* or *Save*, pressed.
    pub fn pressed_the_button(&mut self, folders: &dyn Folders) -> Option<Ending> {
        match self.how {
            How::Open => self.used(self.focus, folders),
            How::Save => self.saved(folders),
        }
    }

    /// **alo answers the sheet with `name` in `folder`**, which the person
    /// approved (`choose_on_the_file_chooser`): the file it ends with, or
    /// [`NotBelieved`] when the sheet could not have been answered with it.
    ///
    /// Held to what a person's own answer could be — to open, a file that is
    /// there; to save, a name in a folder that is there that no folder in it
    /// has — and **nothing else**: which folders alo may reach is the turn's
    /// question, asked against the grants before this is ever reached. A file
    /// already there may be saved over, because the person approved that
    /// sentence; the sheet's own second press is the person's way of saying so.
    ///
    /// # Errors
    /// [`NotBelieved`], and the sheet is left as it was.
    pub fn alo_chose(
        &self,
        folder: &Path,
        name: &str,
        folders: &dyn Folders,
    ) -> Result<PathBuf, NotBelieved> {
        if !alo_acting::is_a_name(name) || folders.listed(folder).is_err() {
            return Err(NotBelieved);
        }
        let there = folders.holds(folder, name);
        let could = match self.how {
            How::Open => there == Some(Kind::File),
            How::Save => there != Some(Kind::Folder),
        };
        if could {
            Ok(folder.join(name))
        } else {
            Err(NotBelieved)
        }
    }

    /// The keyboard to `row`, on the rows.
    fn moved_to(&mut self, row: usize) {
        self.focus = row;
        self.on = Where::TheRows;
    }

    /// Enter on `row`.
    fn used(&mut self, row: usize, folders: &dyn Folders) -> Option<Ending> {
        let rows = self.rows();
        let chosen = rows.get(row).copied()?;
        match chosen.kind {
            Kind::Above => {
                self.went_up(folders);
                None
            }
            Kind::Folder => {
                let into = self.at.join(chosen.name);
                self.went_to(into, None, folders);
                None
            }
            Kind::File => match self.how {
                How::Open => Some(Ending::Chosen(self.at.join(chosen.name))),
                How::Save => {
                    self.name = chosen.name.to_owned();
                    self.name_changed();
                    self.on = Where::TheName;
                    None
                }
            },
        }
    }

    /// Enter in the name.
    fn saved(&mut self, folders: &dyn Folders) -> Option<Ending> {
        if self.name.is_empty() {
            self.note = Some(Note::NoName);
            return None;
        }
        if !alo_acting::is_a_name(&self.name) {
            self.note = Some(Note::NotAName);
            return None;
        }
        // **Asked of the disk now**, not of what is shown: a hidden name, one
        // past the most shown, or one made since the folder was opened is not
        // replaced without the person being told.
        let there = folders.holds(&self.at, &self.name);
        if there == Some(Kind::Folder) {
            self.note = Some(Note::AFolderHasThisName);
            return None;
        }
        if there.is_some() && !self.replacing {
            self.replacing = true;
            self.note = Some(Note::ReplacesWhatIsThere);
            return None;
        }
        Some(Ending::Chosen(self.at.join(&self.name)))
    }

    /// The name changed, so a replacement agreed to is not agreed any more.
    fn name_changed(&mut self) {
        self.replacing = false;
        self.note = None;
    }

    /// Up to the folder above, the keyboard on the folder just left.
    fn went_up(&mut self, folders: &dyn Folders) {
        let Some(above) = self.at.parent().map(Path::to_path_buf) else {
            return;
        };
        let left = self
            .at
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        self.went_to(above, left, folders);
    }

    /// Into `folder`, the keyboard on the folder named `on` when it is there.
    fn went_to(&mut self, folder: PathBuf, on: Option<String>, folders: &dyn Folders) {
        match folders.listed(&folder) {
            Ok(listing) => {
                self.at = folder;
                self.listing = listing;
                self.replacing = false;
                self.note = None;
                let above = usize::from(self.above().is_some());
                self.focus = on
                    .and_then(|on| self.listing.folders().iter().position(|one| *one == on))
                    .map_or_else(|| self.the_first_inside(), |at| at + above);
            }
            Err(NotListed::NotThere | NotListed::WouldNotBeRead) => {
                self.note = Some(Note::CouldNotOpen(called(&folder)));
            }
        }
    }

    /// The next row after the keyboard's whose name begins with `letter`.
    fn jumped_to(&mut self, letter: char) {
        let rows = self.rows();
        let wanted: String = letter.to_lowercase().collect();
        let count = rows.len();
        for step in 1..=count {
            let at = (self.focus + step) % count;
            if let Some(row) = rows.get(at)
                && row.kind != Kind::Above
                && row.name.to_lowercase().starts_with(&wanted)
            {
                self.focus = at;
                return;
            }
        }
    }

    /// The name of the folder above, or [`None`] at the root.
    fn above(&self) -> Option<&str> {
        let above = self.at.parent()?;
        Some(
            above
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("/"),
        )
    }

    /// How many rows there are.
    fn how_many_rows(&self) -> usize {
        usize::from(self.above().is_some())
            + self.listing.folders().len()
            + self.listing.files().len()
    }

    /// The first row inside this folder — past the folder above, where there
    /// is anything past it.
    fn the_first_inside(&self) -> usize {
        let above = usize::from(self.above().is_some());
        if self.how_many_rows() > above {
            above
        } else {
            0
        }
    }
}

/// What a folder is called in a sentence: its own name, or `/` for the root.
fn called(folder: &Path) -> String {
    folder.file_name().map_or_else(
        || folder.to_string_lossy().into_owned(),
        |name| name.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    reason = "in a test, a panic on a row that is not there is the failure being reported"
)]
#[path = "chooser_tests.rs"]
mod tests;
