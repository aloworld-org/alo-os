//! **The file a person's divisions are kept in.**
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md):
//! a person's settings are kept by the crate that owns each, in the person's own
//! folder. So this is `dividing.toml` beside `access.toml` and
//! `appearance.toml`, written by this crate and by nothing else.
//!
//! **This crate does not know where the folder is.** It is handed the path, as
//! `alo-access`'s keeping is, because a crate that resolved a person's home
//! folder would be a second answer to a question ADR 0016 already settled.
//!
//! # There is no machine-wide copy, and that is deliberate
//!
//! `alo-access` keeps a second copy for the sign-in screen, because somebody who
//! needs the screen read aloud cannot turn that on by reading the screen. No such
//! argument exists here: nobody divides a display before they have signed in, and
//! a machine-wide arrangement of somebody's windows would be one person's desk
//! shown to the next person who sits down.
//!
//! # What reaches the disk
//!
//! Applications and shares. **No window, no title, no document, no path** — see
//! `crate::remembering`, whose own test reads the written bytes back and fails
//! if a window ever reaches them.

use std::io;
use std::path::Path;

use crate::remembering::Divisions;

/// What the file is called, in the person's folder.
pub const THE_FILE: &str = "dividing.toml";

/// The shape of the file, so a later one can be read and this one still is.
pub const FORMAT: i64 = 1;

/// Why the divisions could not be read.
#[derive(Debug, thiserror::Error)]
pub enum FileNotRead {
    /// Nothing could be read from that path.
    #[error("the divisions could not be read: {0}")]
    NotReadable(#[from] io::Error),
    /// It is there and it is not these settings.
    #[error("the divisions are there and are not settings this alo OS reads: {0}")]
    NotTheseSettings(#[from] toml::de::Error),
}

/// Why the divisions could not be written.
#[derive(Debug, thiserror::Error)]
pub enum FileNotWritten {
    /// The folder is not writable, or the disk is full.
    #[error("the divisions could not be written: {0}")]
    NotWritable(#[from] io::Error),
    /// They could not be turned into a file at all.
    #[error("the divisions could not be written down: {0}")]
    NotWritableAsSettings(#[from] toml::ser::Error),
}

/// What the file holds: the format, and the divisions.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Written {
    /// Which shape this file is in.
    format: i64,
    /// Every screen's remembered division.
    #[serde(flatten)]
    divisions: Divisions,
}

/// **Read the divisions a person has**, from the folder they keep settings in.
///
/// A machine with no file is a person who has divided nothing yet, which is
/// [`Divisions::none`] and never an error.
///
/// # Errors
/// [`FileNotRead`] where a file is there and cannot be read, or is there and is
/// not these settings — half a settings file is the machine choosing the other
/// half (ADR 0016).
pub fn read(at: &Path) -> Result<Divisions, FileNotRead> {
    let path = at.join(THE_FILE);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(why) if why.kind() == io::ErrorKind::NotFound => return Ok(Divisions::none()),
        Err(why) => return Err(why.into()),
    };
    let written: Written = toml::from_str(&text)?;
    Ok(written.divisions)
}

/// **Keep a person's divisions**, in their own folder.
///
/// # Errors
/// [`FileNotWritten`] where the folder cannot be written to.
pub fn keep(at: &Path, divisions: &Divisions) -> Result<(), FileNotWritten> {
    let written = Written {
        format: FORMAT,
        divisions: divisions.clone(),
    };
    std::fs::write(at.join(THE_FILE), toml::to_string_pretty(&written)?)?;
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::remembering::{HeldBy, Remembered};
    use crate::side::Axis;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A folder of this test's own, made rather than made-if-needed, so two
    /// tests can never be handed each other's (`docs/quirks.md`, 2026-09-17).
    fn a_folder() -> std::path::PathBuf {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        loop {
            let at = std::env::temp_dir().join(format!(
                "alo-dividing-keeping-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::create_dir(&at) {
                Ok(()) => return at,
                Err(why) => {
                    assert!(
                        why.kind() == io::ErrorKind::AlreadyExists,
                        "a folder for this test: {why}"
                    );
                }
            }
        }
    }

    /// An application, named.
    fn app(name: &str) -> HeldBy {
        HeldBy::named(name).expect("a usable name")
    }

    /// **A person who has divided nothing has divided nothing**, and that is
    /// not an error.
    #[test]
    fn no_file_is_nothing_remembered() {
        let at = a_folder();
        assert_eq!(read(&at).expect("an unwritten folder"), Divisions::none());
    }

    /// **What is kept is what is read back.**
    #[test]
    fn what_is_kept_comes_back() {
        let at = a_folder();
        let mut divisions = Divisions::none();
        divisions.remember(
            "panel:ACME/A1/SER",
            Remembered::Cut {
                axis: Axis::SideBySide,
                first_length: 960,
                first: Box::new(Remembered::HeldBy(app("org.example.Mail"))),
                second: Box::new(Remembered::HeldBy(app("org.example.Editor"))),
            },
        );
        keep(&at, &divisions).expect("a writable folder");
        assert_eq!(read(&at).expect("what was just written"), divisions);
    }

    /// **A file that is there and is not these settings is a refusal**, never
    /// half an answer.
    #[test]
    fn a_file_that_is_not_these_settings_is_refused() {
        let at = a_folder();
        std::fs::write(at.join(THE_FILE), "this is not a settings file\n")
            .expect("a writable folder");
        assert!(matches!(read(&at), Err(FileNotRead::NotTheseSettings(_))));
    }

    /// **The file says which shape it is in**, so a later one can be read and
    /// this one still is.
    #[test]
    fn the_file_says_its_format() {
        let at = a_folder();
        keep(&at, &Divisions::none()).expect("a writable folder");
        let text = std::fs::read_to_string(at.join(THE_FILE)).expect("what was written");
        assert!(text.contains(&format!("format = {FORMAT}")), "{text}");
    }
}
