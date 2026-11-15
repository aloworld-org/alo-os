//! The words on the compositor's door for **opening a file in an application
//! the person named** — `open_file_in_application`, added 2026-11-14 by the
//! owner's decision, *"Yes, add it"*.
//!
//! ```text
//! → open-file com.vscodium.codium /var/home/anna/hello.txt
//! ← opened-in-alos-copy        (or: opened-in-their-window, not-installed,
//!                               takes-no-file, no-copy-of-its-own,
//!                               could-not-start)
//! ```
//!
//! # Where it opens is the person's, read by the desktop
//!
//! The line says which application and which file, and **never where**: at the
//! default the desktop opens it in alo's own copy of the application (ADR 0143
//! Q6, ADR 0153), so what alo types into it next is typed there; where the
//! person chose *alo may work in your own window* in Settings, it opens as their
//! own press would. Which, is read by the desktop from the person's own file at
//! this ask (`alo_capability::AloInApplications`), as it is for working in an
//! application — never taken from a word alo sent. The answer says which it
//! was, so the record can.
//!
//! # Nothing in it can be a command
//!
//! The identifier is an [`Application`]'s, exactly. The path is **everything
//! after it**, so a file name with spaces in it is still one path; it begins
//! with `/`, so it can never be read as an option, and it holds no control
//! character, so it is one line. The desktop hands it to the application as
//! one argument, where its entry's own `%f`, `%F`, `%u` or `%U` stands, or as
//! the one `file://` address of a bus `Open` — never to a shell.

use std::path::{Path, PathBuf};

use alo_applications::Application;

/// The word that begins this act.
pub const OPEN_FILE: &str = "open-file";

/// The most a line of this act may be, in bytes — the most the door reads of
/// any line (`choosing_a_link::THE_MOST`), so a line this side writes is one
/// the door will read.
pub const THE_MOST: usize = crate::choosing_a_link::THE_MOST;

/// A request to open a file in an application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToOpenAFile {
    /// The application that opens it, by its identifier.
    application: Application,
    /// The file, an absolute path.
    file: PathBuf,
}

impl ToOpenAFile {
    /// Open `file` in `application` — [`None`] when the file is not a path
    /// this line can carry: absolute, one line, text, and short enough.
    #[must_use]
    pub fn with(application: Application, file: &Path) -> Option<Self> {
        let asked = Self {
            application,
            file: file.to_path_buf(),
        };
        let line = asked.written()?;
        Self::read(&line).and_then(Result::ok).map(|_| asked)
    }

    /// The application to open it in.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// The file to open.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }

    /// The line, without its newline — [`None`] for a file that is not text.
    #[must_use]
    pub fn written(&self) -> Option<String> {
        Some(format!(
            "{OPEN_FILE} {} {}",
            self.application.identifier(),
            self.file.to_str()?
        ))
    }

    /// One line, read as this act — [`None`] when it is not this act at all,
    /// so the door can read it as one of the others.
    ///
    /// A line that begins with [`OPEN_FILE`] and is not exactly an identifier
    /// and an absolute path after it is `Some(Err(…))`: it was meant as this
    /// act and is a caller with a bug.
    #[must_use]
    pub fn read(line: &str) -> Option<Result<Self, NotAFileToOpen>> {
        let line = line.trim_end_matches(['\r', '\n']);
        let rest = line.strip_prefix(OPEN_FILE)?;
        let Some(rest) = rest.strip_prefix(' ') else {
            return rest.is_empty().then_some(Err(NotAFileToOpen::NotTwoWords));
        };
        if line.len() > THE_MOST {
            return Some(Err(NotAFileToOpen::TooLong));
        }
        let Some((named, file)) = rest.split_once(' ') else {
            return Some(Err(NotAFileToOpen::NotTwoWords));
        };
        let Ok(application) = Application::identified(named) else {
            return Some(Err(NotAFileToOpen::NotAnApplication));
        };
        if application.identifier() != named {
            return Some(Err(NotAFileToOpen::NotAnApplication));
        }
        // **Absolute, so never an option**: on Linux, where the door is, that
        // is a path beginning with `/`. A host whose absolute paths begin
        // otherwise — a drive letter — still never begins one with `-`.
        let absolute = file.starts_with('/') || Path::new(file).is_absolute();
        if !absolute || file.chars().any(char::is_control) {
            return Some(Err(NotAFileToOpen::NotAFile));
        }
        Some(Ok(Self {
            application,
            file: PathBuf::from(file),
        }))
    }
}

/// Why a line that began as this act is not one.
///
/// English, read by whoever is fixing the caller. A person never reads one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotAFileToOpen {
    /// Longer than [`THE_MOST`].
    #[error("an open-file line is at most {THE_MOST} bytes")]
    TooLong,
    /// Not an identifier and a path after the act.
    #[error("open-file is followed by an application and a file")]
    NotTwoWords,
    /// The second word is not an application's identifier.
    #[error("open-file names an application by its identifier")]
    NotAnApplication,
    /// What follows is not an absolute path on one line.
    #[error("open-file carries an absolute path on one line")]
    NotAFile,
}

/// What the desktop did with a file it was asked to open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opened {
    /// Opened in alo's own copy of the application — the default.
    InAlosCopy,
    /// Opened in the person's own window, where they chose that.
    InTheirWindow,
    /// No application by that identifier is installed.
    NotInstalled,
    /// Its entry has nowhere to put a file, so nothing was started.
    TakesNoFile,
    /// At the default, and it cannot open a separate copy for alo.
    NoCopyOfItsOwn,
    /// It could not be started with the file.
    CouldNotStart,
}

impl Opened {
    /// Every answer, for a test that each reads back as itself.
    pub const ALL: [Self; 6] = [
        Self::InAlosCopy,
        Self::InTheirWindow,
        Self::NotInstalled,
        Self::TakesNoFile,
        Self::NoCopyOfItsOwn,
        Self::CouldNotStart,
    ];

    /// The word on the wire.
    #[must_use]
    pub const fn written(self) -> &'static str {
        match self {
            Self::InAlosCopy => "opened-in-alos-copy",
            Self::InTheirWindow => "opened-in-their-window",
            Self::NotInstalled => "not-installed",
            Self::TakesNoFile => "takes-no-file",
            Self::NoCopyOfItsOwn => "no-copy-of-its-own",
            Self::CouldNotStart => "could-not-start",
        }
    }

    /// One answer read back — [`None`] for any word this machine does not
    /// know, which is **never read as opened**.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        let word = line.trim_end_matches(['\r', '\n']);
        Self::ALL.into_iter().find(|it| it.written() == word)
    }

    /// Whether the file was opened.
    #[must_use]
    pub const fn was_opened(self) -> bool {
        matches!(self, Self::InAlosCopy | Self::InTheirWindow)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The application every test here opens a file in.
    fn vscodium() -> Application {
        Application::identified("com.vscodium.codium").unwrap()
    }

    /// **What is written is what is read**, a file name with spaces in it as
    /// one path.
    #[test]
    fn a_line_written_reads_back_as_itself() {
        for file in [
            "/var/home/anna/hello.txt",
            "/var/home/anna/My Notes/a b.txt",
        ] {
            let asked = ToOpenAFile::with(vscodium(), Path::new(file)).unwrap();
            let line = asked.written().unwrap();
            assert_eq!(ToOpenAFile::read(&line), Some(Ok(asked.clone())), "{file}");
            assert_eq!(asked.file(), Path::new(file));
        }
    }

    /// **Another act is not this one**, so the door reads it as the other.
    #[test]
    fn another_act_is_not_this_one() {
        for line in [
            "open com.vscodium.codium",
            "open-link org.mozilla.firefox https://example.com/",
            "open-files com.vscodium.codium /a",
        ] {
            assert_eq!(ToOpenAFile::read(line), None, "{line}");
        }
    }

    /// **Anything that is not an application and an absolute path is
    /// refused**, and a refusal is never a shorter, different request.
    #[test]
    fn what_is_not_an_application_and_a_path_is_refused() {
        for (line, refused) in [
            ("open-file", NotAFileToOpen::NotTwoWords),
            ("open-file com.vscodium.codium", NotAFileToOpen::NotTwoWords),
            ("open-file  /a", NotAFileToOpen::NotAnApplication),
            (
                "open-file com.vscodium.codium hello.txt",
                NotAFileToOpen::NotAFile,
            ),
            (
                "open-file com.vscodium.codium --user-data-dir=/tmp",
                NotAFileToOpen::NotAFile,
            ),
            (
                "open-file com.vscodium.codium /a\tb",
                NotAFileToOpen::NotAFile,
            ),
            (
                "open-file /usr/bin/codium /var/home/anna/hello.txt",
                NotAFileToOpen::NotAnApplication,
            ),
        ] {
            assert_eq!(ToOpenAFile::read(line), Some(Err(refused)), "{line}");
        }
        let long = format!("open-file com.vscodium.codium /{}", "a".repeat(THE_MOST));
        assert_eq!(ToOpenAFile::read(&long), Some(Err(NotAFileToOpen::TooLong)));
        assert_eq!(
            ToOpenAFile::with(vscodium(), Path::new(&format!("/{}", "a".repeat(THE_MOST)))),
            None,
            "a file too long for the line is not made into one"
        );
        assert_eq!(ToOpenAFile::with(vscodium(), Path::new("hello.txt")), None);
    }

    /// **Each answer reads back as itself, and an unknown word is never
    /// opened.**
    #[test]
    fn each_answer_reads_back_and_an_unknown_one_is_not_opened() {
        for answer in Opened::ALL {
            assert_eq!(
                Opened::read(&format!("{}\n", answer.written())),
                Some(answer)
            );
        }
        for unknown in ["opened", "acted", "", "OPENED-IN-ALOS-COPY"] {
            assert_eq!(Opened::read(unknown), None, "{unknown}");
        }
        assert!(Opened::InAlosCopy.was_opened() && Opened::InTheirWindow.was_opened());
        assert!(!Opened::TakesNoFile.was_opened());
    }
}
