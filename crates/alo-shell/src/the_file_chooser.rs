//! **The file chooser sheet**: an application asked the person to choose a
//! file to open, or a place to save one, and the desktop asks them — the
//! owner's decision of 2026-11-17 (*"Yes, build it"*,
//! `docs/design/the-file-chooser.md`).
//!
//! The portal asks through the door's `choose-file` act and holds the
//! connection (`docs/contracts/application-door.md`); the door hands the
//! connection here ([`crate::the_file_chooser::AskedForAFile`]) and this
//! answers it when the person has.
//! What the sheet knows and decides — the rows, each key, when a name may be
//! saved — is `alo_file_chooser::Chooser`'s; this holds the sheets and their
//! connections.
//!
//! # Parked until the portal asks
//!
//! **Nothing writes `choose-file` yet.** The portal service writes it once it
//! serves `org.freedesktop.portal.FileChooser`, which is the next change; until
//! then an application's file dialog is its own, as it always was.
//!
//! # One at a time, and never for ever
//!
//! *Open with*'s rules (`crate::the_open_with_sheet`), for the same reasons:
//! one sheet is up, up to `AS_MANY_AS_WAIT` wait behind it, one more is
//! answered `cannot-show`; a sheet nobody answers in `THE_LONGEST` is
//! answered `unanswered`; a portal that hangs up takes its sheet down.

use std::collections::VecDeque;
use std::io::{ErrorKind, Read as _, Write as _};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use alo_acting::{ChoseAFile, ToChoose, ToChooseAFile};
use alo_applications::Application;
use alo_file_chooser::{Chooser, Ending, Folders, Key, OnThisDisk};

/// The longest a sheet stays up unanswered.
pub(crate) const THE_LONGEST: Duration = Duration::from_secs(5 * 60);

/// How many sheets may wait behind the one that is up.
pub(crate) const AS_MANY_AS_WAIT: usize = 4;

/// The portal asking the person to choose a file, with its connection held.
#[derive(Debug)]
pub struct AskedForAFile {
    /// What it asked.
    asked: ToChooseAFile,
    /// Its connection, not blocking.
    talking: UnixStream,
}

impl AskedForAFile {
    /// The door read `asked` on `talking`.
    pub(crate) const fn of(asked: ToChooseAFile, talking: UnixStream) -> Self {
        Self { asked, talking }
    }

    /// Answer it and let it go. A portal that has gone is not told.
    fn answered(mut self, chose: &ChoseAFile) {
        let _ = self.talking.write_all(chose.written().as_bytes());
        let _ = self.talking.write_all(b"\n");
        let _ = self.talking.flush();
    }

    /// Whether the portal has hung up.
    fn has_hung_up(&mut self) -> bool {
        let mut said = [0_u8; 64];
        loop {
            match self.talking.read(&mut said) {
                Ok(0) => return true,
                Ok(_) => {}
                Err(why) if why.kind() == ErrorKind::WouldBlock => return false,
                Err(why) if why.kind() == ErrorKind::Interrupted => {}
                Err(_) => return true,
            }
        }
    }
}

/// A key at the seat, as the chooser understands it — read as the password
/// sheet reads keys, because the name is typed.
pub(crate) const fn the_key(key: crate::SignInKey) -> Option<Key> {
    use crate::SignInKey as K;
    match key {
        K::Letter(letter) => Some(Key::Letter(letter)),
        K::Erase => Some(Key::Erase),
        K::OtherField => Some(Key::Tab),
        K::FieldBefore => Some(Key::BackTab),
        K::Enter => Some(Key::Enter),
        K::Clear => Some(Key::Escape),
        K::Earlier => Some(Key::Up),
        K::Later => Some(Key::Down),
        _ => None,
    }
}

/// The sheet that is up.
#[derive(Debug)]
struct Shown {
    /// What was asked, and where to answer.
    asked: AskedForAFile,
    /// Where the person stands, and what they see.
    chooser: Chooser,
    /// When it went up.
    since: Instant,
    /// Which sheet this is, counted from the first to go up: what a screen
    /// reader's press is held to.
    number: u64,
}

/// What the sheet shows, for drawing it and reading it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FileChooserShows<'a> {
    /// The application that asked.
    pub(crate) asking: &'a Application,
    /// Where the person stands, and what they see.
    pub(crate) chooser: &'a Chooser,
    /// Which sheet this is.
    pub(crate) number: u64,
}

/// What a press on the sheet is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnTheFileChooser {
    /// The row at this place in the list.
    TheRow(usize),
    /// The name, when saving.
    TheName,
    /// *Open* or *Save*.
    TheButton,
    /// *Cancel*.
    Cancel,
}

/// Every file chooser the desktop holds: the one up, and those waiting.
#[derive(Debug, Default)]
pub(crate) struct TheFileChooser {
    /// The one up.
    shown: Option<Shown>,
    /// Those waiting behind it, oldest first.
    waiting: VecDeque<AskedForAFile>,
    /// How many sheets have gone up.
    went_up: u64,
}

impl TheFileChooser {
    /// No sheet.
    pub(crate) fn none() -> Self {
        Self::default()
    }

    /// Whether a sheet is up, so every key is its.
    pub(crate) const fn is_up(&self) -> bool {
        self.shown.is_some()
    }

    /// What the sheet that is up shows.
    pub(crate) fn shows(&self) -> Option<FileChooserShows<'_>> {
        self.shown.as_ref().map(|shown| FileChooserShows {
            asking: shown.asked.asked.asking(),
            chooser: &shown.chooser,
            number: shown.number,
        })
    }

    /// The portal asked these. `can_show` is whether this desktop can ask the
    /// person now; where it cannot, each is answered `cannot-show` at once.
    pub(crate) fn asked(
        &mut self,
        asks: Vec<AskedForAFile>,
        can_show: bool,
        now: Instant,
        folders: &dyn Folders,
    ) {
        for ask in asks {
            if !can_show || self.waiting.len() >= AS_MANY_AS_WAIT {
                ask.answered(&ChoseAFile::CannotShow);
                continue;
            }
            self.waiting.push_back(ask);
            self.the_next_goes_up(now, folders);
        }
    }

    /// Once a frame: a portal that hung up takes its sheet down, a sheet up
    /// for `THE_LONGEST` is answered `unanswered`, and the next goes up.
    pub(crate) fn waited(&mut self, now: Instant, folders: &dyn Folders) {
        self.waiting.retain_mut(|ask| !ask.has_hung_up());
        if let Some(shown) = self.shown.as_mut() {
            if shown.asked.has_hung_up() {
                self.shown = None;
            } else if now.saturating_duration_since(shown.since) >= THE_LONGEST
                && let Some(shown) = self.shown.take()
            {
                shown.asked.answered(&ChoseAFile::Unanswered);
            }
        }
        self.the_next_goes_up(now, folders);
    }

    /// A key while the sheet is up.
    pub(crate) fn pressed(&mut self, key: Key, now: Instant, folders: &dyn Folders) {
        let ending = self
            .shown
            .as_mut()
            .and_then(|shown| shown.chooser.key(key, folders));
        self.ended(ending, now, folders);
    }

    /// A click on `on`.
    pub(crate) fn clicked(&mut self, on: OnTheFileChooser, now: Instant, folders: &dyn Folders) {
        let ending = self.shown.as_mut().and_then(|shown| match on {
            OnTheFileChooser::TheRow(row) => shown.chooser.pressed(row, folders),
            OnTheFileChooser::TheName => {
                shown.chooser.pressed_the_name();
                None
            }
            OnTheFileChooser::TheButton => shown.chooser.pressed_the_button(folders),
            OnTheFileChooser::Cancel => Some(Ending::Cancelled),
        });
        self.ended(ending, now, folders);
    }

    /// **A screen reader pressed `on`, on the sheet numbered `sheet`** — held
    /// to the sheet it was heard on, so a press heard on a sheet since
    /// answered never answers the next. `false` when it was for another sheet
    /// or a row the sheet does not have.
    pub(crate) fn a_reader_pressed(
        &mut self,
        sheet: u64,
        on: OnTheFileChooser,
        now: Instant,
        folders: &dyn Folders,
    ) -> bool {
        let Some(shown) = self.shown.as_mut() else {
            return false;
        };
        if shown.number != sheet {
            return false;
        }
        let ending = match on {
            OnTheFileChooser::TheRow(row) => {
                if row >= shown.chooser.rows().len() {
                    return false;
                }
                shown.chooser.a_reader_pressed(row, folders)
            }
            OnTheFileChooser::TheName => {
                shown.chooser.pressed_the_name();
                None
            }
            OnTheFileChooser::TheButton => shown.chooser.pressed_the_button(folders),
            OnTheFileChooser::Cancel => Some(Ending::Cancelled),
        };
        self.ended(ending, now, folders);
        true
    }

    /// The desktop could not draw the sheet on this display: its question is
    /// answered `cannot-show` rather than every key held with nothing on the
    /// screen.
    pub(crate) fn could_not_be_drawn(&mut self, now: Instant, folders: &dyn Folders) {
        if let Some(shown) = self.shown.take() {
            shown.asked.answered(&ChoseAFile::CannotShow);
        }
        self.the_next_goes_up(now, folders);
    }

    /// **The sheet is answered with `file`**, as the person's own choice
    /// would answer it — what alo's approved answer comes to
    /// (`crate::alo_answers_the_file_chooser`).
    pub(crate) fn chosen_for_the_person(
        &mut self,
        file: std::path::PathBuf,
        now: Instant,
        folders: &dyn Folders,
    ) {
        self.ended(Some(Ending::Chosen(file)), now, folders);
    }

    /// The sheet ended, when it did: answered, and the next goes up.
    fn ended(&mut self, ending: Option<Ending>, now: Instant, folders: &dyn Folders) {
        let Some(ending) = ending else {
            return;
        };
        if let Some(shown) = self.shown.take() {
            shown.asked.answered(&match ending {
                Ending::Chosen(path) => ChoseAFile::Chosen(path),
                Ending::Cancelled => ChoseAFile::Cancelled,
            });
        }
        self.the_next_goes_up(now, folders);
    }

    /// The oldest waiting goes up, when none is.
    fn the_next_goes_up(&mut self, now: Instant, folders: &dyn Folders) {
        if self.shown.is_some() {
            return;
        }
        if let Some(asked) = self.waiting.pop_front() {
            self.went_up = self.went_up.wrapping_add(1);
            let at = asked.asked.starting_in().to_path_buf();
            let chooser = match asked.asked.to() {
                ToChoose::Open => Chooser::to_open(&at, folders),
                ToChoose::Save { name } => Chooser::to_save(&at, name, folders),
            };
            self.shown = Some(Shown {
                asked,
                chooser,
                since: now,
                number: self.went_up,
            });
        }
    }
}

/// The disk the desktop's sheet reads: the person's own, as they can read it.
pub(crate) const THE_DISK: OnThisDisk = OnThisDisk;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_file_chooser_tests.rs"]
mod tests;
