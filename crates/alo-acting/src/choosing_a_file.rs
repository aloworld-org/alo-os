//! The words on the compositor's door for **the person choosing a file** — the
//! file chooser the owner decided on 2026-11-17 (*"Yes, build it"*): one sheet,
//! the same for every application, shown when an application asks to open a
//! file or to save one.
//!
//! ```text
//! → choose-file org.gnome.TextEditor open /home/anna/Documents
//! → choose-file org.gnome.TextEditor save /home/anna/Documents notes%20for%20March.txt
//! ← chosen /home/anna/Documents/notes%20for%20March.txt   (or: cancelled, unanswered, cannot-show)
//! ```
//!
//! `docs/contracts/application-door.md`, *The person chooses a file*, is the
//! published surface; this is it as working code.
//!
//! # A path is written percent-encoded
//!
//! A folder and a file name may each hold a space, and this act carries two of
//! them on one line, so neither can be *everything after the identifier* as
//! `open-file`'s one path is. Every byte outside `A`–`Z`, `a`–`z`, `0`–`9`,
//! `-`, `.`, `_`, `~` and — in a path, never in a name — `/` is written `%XX`,
//! as a `file:` address writes it. What is read back must decode to text with no
//! control character in it, so a name that would end the line, or hide part of
//! itself, is never a name.
//!
//! # The desktop asks; the person decides
//!
//! The folder the sheet starts in and the name it suggests are the asking
//! application's, through the portal, and are only where the sheet **starts**:
//! the person may go anywhere their own account can, and what comes back is
//! where they went. Nothing on this line limits them. It is the portal that
//! decides what an application is then given (`docs/contracts/portals.md`).

use std::path::{Component, Path, PathBuf};

use alo_applications::Application;

/// The word that begins this act.
pub const CHOOSE_FILE: &str = "choose-file";

/// The most a line of this act may be, in bytes — the most the door reads of
/// any line (`crate::choosing_a_link::THE_MOST`).
pub const THE_MOST: usize = crate::choosing_a_link::THE_MOST;

/// The longest file name, in bytes, as Linux filesystems hold one.
pub const THE_LONGEST_NAME: usize = 255;

/// What the application asked the person for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToChoose {
    /// A file that is already there, to open.
    Open,
    /// A place to save a file, and the name the application suggests.
    Save {
        /// The name suggested, which the person may change.
        name: String,
    },
}

/// A request for the person to choose a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToChooseAFile {
    /// The application that asked, as its sandbox named it.
    asking: Application,
    /// Opening or saving.
    to: ToChoose,
    /// The folder the sheet starts in.
    starting_in: PathBuf,
}

impl ToChooseAFile {
    /// Ask the person to choose a file for `asking`, starting in `starting_in`.
    ///
    /// # Errors
    ///
    /// [`NotAFileToChoose::NotAFolder`] for a folder that is not a whole path
    /// from the root with only ordinary names in it, or that holds a character
    /// no line can carry; [`NotAFileToChoose::NotAName`] for a suggested name
    /// that is empty, `.`, `..`, holds a `/` or a control character, or is
    /// longer than [`THE_LONGEST_NAME`]; and [`NotAFileToChoose::TooLong`] for
    /// a request longer than [`THE_MOST`].
    pub fn of(
        asking: Application,
        to: ToChoose,
        starting_in: PathBuf,
    ) -> Result<Self, NotAFileToChoose> {
        if !is_a_whole_path(&starting_in) {
            return Err(NotAFileToChoose::NotAFolder);
        }
        if let ToChoose::Save { name } = &to
            && !is_a_name(name)
        {
            return Err(NotAFileToChoose::NotAName);
        }
        let asked = Self {
            asking,
            to,
            starting_in,
        };
        if asked.written().len() > THE_MOST {
            return Err(NotAFileToChoose::TooLong);
        }
        Ok(asked)
    }

    /// The application that asked.
    #[must_use]
    pub const fn asking(&self) -> &Application {
        &self.asking
    }

    /// Opening or saving, and the name suggested.
    #[must_use]
    pub const fn to(&self) -> &ToChoose {
        &self.to
    }

    /// The folder the sheet starts in.
    #[must_use]
    pub fn starting_in(&self) -> &Path {
        &self.starting_in
    }

    /// The line, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        let folder = encoded(&self.starting_in.to_string_lossy(), true);
        match &self.to {
            ToChoose::Open => format!("{CHOOSE_FILE} {} {OPEN} {folder}", self.asking.identifier()),
            ToChoose::Save { name } => format!(
                "{CHOOSE_FILE} {} {SAVE} {folder} {}",
                self.asking.identifier(),
                encoded(name, false)
            ),
        }
    }

    /// One line, read as this act — [`None`] when it is not this act at all.
    ///
    /// A line that begins with [`CHOOSE_FILE`] and is not exactly this act's
    /// words is `Some(Err(…))`: a caller with a bug.
    #[must_use]
    pub fn read(line: &str) -> Option<Result<Self, NotAFileToChoose>> {
        let line = line.trim_end_matches(['\r', '\n']);
        let mut words = line.split(' ');
        if words.next() != Some(CHOOSE_FILE) {
            return None;
        }
        if line.len() > THE_MOST {
            return Some(Err(NotAFileToChoose::TooLong));
        }
        let (Some(asking), Some(how), Some(folder)) = (words.next(), words.next(), words.next())
        else {
            return Some(Err(NotAFileToChoose::NotEnoughWords));
        };
        let Some(asking) = an_identifier(asking) else {
            return Some(Err(NotAFileToChoose::NotAnApplication));
        };
        let Some(folder) = decoded(folder) else {
            return Some(Err(NotAFileToChoose::NotAFolder));
        };
        let to = match (how, words.next()) {
            (OPEN, None) => ToChoose::Open,
            (SAVE, Some(name)) => match decoded(name) {
                Some(name) => ToChoose::Save { name },
                None => return Some(Err(NotAFileToChoose::NotAName)),
            },
            (OPEN | SAVE, _) => return Some(Err(NotAFileToChoose::NotEnoughWords)),
            _ => return Some(Err(NotAFileToChoose::NeitherOpenNorSave)),
        };
        if words.next().is_some() {
            return Some(Err(NotAFileToChoose::NotEnoughWords));
        }
        Some(Self::of(asking, to, PathBuf::from(folder)))
    }
}

/// The word for opening.
const OPEN: &str = "open";
/// The word for saving.
const SAVE: &str = "save";

/// An identifier exactly as written, or [`None`].
fn an_identifier(word: &str) -> Option<Application> {
    let application = Application::identified(word).ok()?;
    (application.identifier() == word).then_some(application)
}

/// Why a line that began as this act is not one. English, for whoever is
/// fixing the caller; a person never reads one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotAFileToChoose {
    /// Longer than [`THE_MOST`].
    #[error("a choose-file line is at most {THE_MOST} bytes")]
    TooLong,
    /// A word missing, or one too many.
    #[error(
        "choose-file is followed by the asking application, open or save, a folder, and for save a name"
    )]
    NotEnoughWords,
    /// The asking application is not an identifier.
    #[error("choose-file names the asking application by its identifier")]
    NotAnApplication,
    /// Neither `open` nor `save`.
    #[error("choose-file asks to open or to save")]
    NeitherOpenNorSave,
    /// The folder is not a whole, ordinary path.
    #[error("choose-file starts in a whole path from the root, percent-encoded")]
    NotAFolder,
    /// The suggested name is not one name.
    #[error("choose-file suggests one file name, percent-encoded, with no slash in it")]
    NotAName,
}

/// What the person did with the sheet, as the desktop answers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoseAFile {
    /// The person chose this file — to open it, or to save there.
    Chosen(PathBuf),
    /// The person pressed *Cancel* or Escape.
    Cancelled,
    /// Nobody answered while the sheet was up, and the desktop took it down.
    Unanswered,
    /// This desktop cannot ask the person now.
    CannotShow,
}

/// The word for [`ChoseAFile::Chosen`].
const CHOSEN: &str = "chosen";
/// The word for [`ChoseAFile::Cancelled`].
const CANCELLED: &str = "cancelled";
/// The word for [`ChoseAFile::Unanswered`].
const UNANSWERED: &str = "unanswered";
/// The word for [`ChoseAFile::CannotShow`].
const CANNOT_SHOW: &str = "cannot-show";

impl ChoseAFile {
    /// The answer, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        match self {
            Self::Chosen(path) => format!("{CHOSEN} {}", encoded(&path.to_string_lossy(), true)),
            Self::Cancelled => CANCELLED.to_owned(),
            Self::Unanswered => UNANSWERED.to_owned(),
            Self::CannotShow => CANNOT_SHOW.to_owned(),
        }
    }

    /// An answer heard, or [`None`] when it is not one — an unknown word, or
    /// `chosen` of something that is not a whole path to a file. **An unknown
    /// answer is never read as a choice.**
    #[must_use]
    pub fn read(heard: &str) -> Option<Self> {
        let heard = heard.trim_end_matches(['\r', '\n']);
        match heard {
            CANCELLED => return Some(Self::Cancelled),
            UNANSWERED => return Some(Self::Unanswered),
            CANNOT_SHOW => return Some(Self::CannotShow),
            _ => {}
        }
        let path = PathBuf::from(decoded(heard.strip_prefix(CHOSEN)?.strip_prefix(' ')?)?);
        (is_a_whole_path(&path) && path.file_name().is_some()).then_some(Self::Chosen(path))
    }
}

/// Whether `path` is a whole path from the root with only ordinary names in it
/// — no `.`, no `..`, nothing a line cannot carry.
#[must_use]
pub fn is_a_whole_path(path: &Path) -> bool {
    let Some(text) = path.to_str() else {
        return false;
    };
    if !text.starts_with('/') || text.chars().any(char::is_control) {
        return false;
    }
    if text.len() > 1 && (text.ends_with('/') || text.contains("//")) {
        return false;
    }
    text.split('/')
        .skip(1)
        .all(|name| text == "/" || is_a_name(name))
        && path
            .components()
            .all(|part| matches!(part, Component::RootDir | Component::Normal(_)))
}

/// Whether `name` is one file name: not empty, not `.` or `..`, no `/`, no
/// control character, and at most [`THE_LONGEST_NAME`] bytes.
#[must_use]
pub fn is_a_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && name.len() <= THE_LONGEST_NAME
        && !name.contains('/')
        && !name.chars().any(char::is_control)
}

/// `text` with every byte outside the unreserved set — and `/`, where `slash`
/// keeps it — written `%XX`.
fn encoded(text: &str, slash: bool) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'.' | b'_' | b'~')
            || (slash && byte == b'/')
        {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// `word` with every `%XX` read back — [`None`] for an empty word, a `%` not
/// followed by two hexadecimal digits, a byte that should have been written
/// `%XX` and was not, or bytes that are not text.
fn decoded(word: &str) -> Option<String> {
    if word.is_empty() {
        return None;
    }
    let mut bytes = Vec::with_capacity(word.len());
    let mut at = word.bytes();
    while let Some(byte) = at.next() {
        if byte == b'%' {
            let high = char::from(at.next()?).to_digit(16)?;
            let low = char::from(at.next()?).to_digit(16)?;
            bytes.push(u8::try_from(high * 16 + low).ok()?);
        } else if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/') {
            bytes.push(byte);
        } else {
            return None;
        }
    }
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "choosing_a_file_tests.rs"]
mod tests;
