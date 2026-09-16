//! **The file this crate keeps, and the machine's own copy for sign-in.**
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md):
//! a person's settings are kept by the crate that owns each, in the person's own
//! folder. So this file is `access.toml` beside `appearance.toml`, written by
//! this crate and by nothing else.
//!
//! **This crate does not know where the folder is.** It is handed the path, as
//! `alo-appearance`'s keeping is, because a crate that resolved a person's home
//! folder would be a second answer to a question ADR 0016 already settled.
//!
//! # Two copies, and why the second is not a duplicate
//!
//! A person who needs the screen read aloud cannot turn that on if turning it on
//! requires reading the screen. So the settings here are also kept **machine
//! wide**, where sign-in and setup read them before any account exists — and
//! that copy belongs to no account: it is what this machine does at the sign-in
//! screen, for whoever is standing in front of it.
//!
//! The person's copy is not the machine's, and neither overwrites the other:
//! [`at_sign_in`] reads the machine's, [`read`] reads a person's, and a person
//! who turns something on for themselves has not changed what the sign-in
//! screen does. [`keep_for_this_machine`] is the deliberate act of saying *and
//! at sign-in too*.

use std::io;
use std::path::Path;

use crate::turned_on::TurnedOn;

/// What the file is called, in a person's folder and machine-wide alike.
pub const THE_FILE: &str = "access.toml";

/// The shape of the file, so a later one can be read and this one still is.
pub const FORMAT: i64 = 1;

/// Why a file could not be read.
#[derive(Debug, thiserror::Error)]
pub enum FileNotRead {
    /// Nothing could be read from that path.
    #[error("the settings could not be read: {0}")]
    NotReadable(#[from] io::Error),
    /// It is there and it is not these settings.
    #[error("the settings are there and are not settings this alo OS reads: {0}")]
    NotTheseSettings(#[from] toml::de::Error),
}

/// Why a file could not be written.
#[derive(Debug, thiserror::Error)]
pub enum FileNotWritten {
    /// The folder is not writable, or the disk is full.
    #[error("the settings could not be written: {0}")]
    NotWritable(#[from] io::Error),
    /// The settings could not be turned into a file at all.
    #[error("the settings could not be written down: {0}")]
    NotWritableAsSettings(#[from] toml::ser::Error),
}

/// What a file holds: the format, and what is turned on.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Written {
    /// Which shape this file is in.
    format: i64,
    /// What is turned on.
    #[serde(flatten)]
    turned_on: TurnedOn,
}

/// **Read what a person has turned on**, from the folder they keep settings in.
///
/// A machine with no file is a person who has not been asked, which is
/// [`TurnedOn::nothing`] and never an error.
///
/// # Errors
/// [`FileNotRead`] where a file is there and cannot be read, or is there and is
/// not these settings — half a settings file is the machine choosing the other
/// half (ADR 0016).
pub fn read(at: &Path) -> Result<TurnedOn, FileNotRead> {
    let path = at.join(THE_FILE);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(why) if why.kind() == io::ErrorKind::NotFound => return Ok(TurnedOn::nothing()),
        Err(why) => return Err(why.into()),
    };
    let written: Written = toml::from_str(&text)?;
    Ok(written.turned_on)
}

/// **Keep what a person has turned on**, in their own folder.
///
/// # Errors
/// [`FileNotWritten`] where the folder cannot be written to.
pub fn keep(at: &Path, turned_on: &TurnedOn) -> Result<(), FileNotWritten> {
    let written = Written {
        format: FORMAT,
        turned_on: turned_on.clone(),
    };
    std::fs::write(at.join(THE_FILE), toml::to_string_pretty(&written)?)?;
    Ok(())
}

/// **Keep what this machine does at the sign-in screen**, for whoever is in
/// front of it and has no account yet.
///
/// The same file in another folder, and a deliberate act: turning something on
/// for oneself does not change what the machine does before anybody signs in.
///
/// # Errors
/// [`FileNotWritten`], as [`keep`].
pub fn keep_for_this_machine(at: &Path, turned_on: &TurnedOn) -> Result<(), FileNotWritten> {
    keep(at, turned_on)
}

/// **What the sign-in screen turns on**, and what went wrong reading it.
///
/// Never fails: a sign-in screen that refused to draw because a settings file
/// was malformed would lock somebody out of their own machine. What it could not
/// read is handed back beside the settings, for whoever can say so.
#[must_use]
pub fn at_sign_in(at: &Path) -> (TurnedOn, Option<FileNotRead>) {
    match read(at) {
        Ok(turned_on) => (turned_on, None),
        Err(why) => (TurnedOn::nothing(), Some(why)),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::setting::Setting;

    /// A folder of this test's own.
    fn a_folder() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        // Counted rather than clocked: two folders named in the same
        // nanosecond are one folder, and a test sharing a folder with another
        // fails for a reason that has nothing to do with what it tests.
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let path = std::env::temp_dir().join(format!(
            "alo-access-unit-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// **A machine nobody has changed has no file**, and reads as nothing on.
    #[test]
    fn no_file_is_a_person_who_has_not_been_asked() {
        let folder = a_folder();
        assert_eq!(read(&folder).unwrap(), TurnedOn::nothing());
        let (turned_on, why) = at_sign_in(&folder);
        assert_eq!(turned_on, TurnedOn::nothing());
        assert!(why.is_none());
    }

    /// **What is kept is what comes back.**
    #[test]
    fn what_is_turned_on_is_what_is_read_back() {
        let folder = a_folder();
        let mut turned_on = TurnedOn::nothing();
        turned_on.turn_on(Setting::ScreenReader);
        turned_on.turn_on(Setting::HighContrast);
        keep(&folder, &turned_on).unwrap();
        let read_back = read(&folder).unwrap();
        assert_eq!(read_back, turned_on);
        assert!(read_back.has(Setting::ScreenReader));
        assert!(!read_back.has(Setting::Magnifier));
    }

    /// **The sign-in screen draws whatever happens to the file.** A person
    /// cannot be locked out of their own machine by a settings file.
    #[test]
    fn a_broken_file_does_not_stop_the_sign_in_screen() {
        let folder = a_folder();
        std::fs::write(folder.join(THE_FILE), "this is not a settings file {{").unwrap();
        let (turned_on, why) = at_sign_in(&folder);
        assert_eq!(turned_on, TurnedOn::nothing());
        assert!(matches!(why, Some(FileNotRead::NotTheseSettings(_))));
    }

    /// **The machine's copy is not the person's**: turning something on for
    /// oneself leaves the sign-in screen as it was.
    #[test]
    fn what_a_person_turns_on_is_not_what_the_machine_does_before_anybody_signs_in() {
        let machine = a_folder();
        let person = a_folder();
        let mut for_the_machine = TurnedOn::nothing();
        for_the_machine.turn_on(Setting::ScreenReader);
        keep_for_this_machine(&machine, &for_the_machine).unwrap();

        let mut theirs = TurnedOn::nothing();
        theirs.turn_on(Setting::LargerText);
        keep(&person, &theirs).unwrap();

        let (at_the_screen, _) = at_sign_in(&machine);
        assert!(at_the_screen.has(Setting::ScreenReader));
        assert!(
            !at_the_screen.has(Setting::LargerText),
            "a person's own setting reached the sign-in screen"
        );
        assert!(read(&person).unwrap().has(Setting::LargerText));
    }
}
