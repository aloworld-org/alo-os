//! Where a machine keeps what an undo would put back, and the two small files
//! that say what is in it.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) chose the
//! base's own read-only snapshots, *kept outside every grant under
//! `/var/lib/alo`*, and named who takes them — `alo-turn`, at the moment a
//! changing turn begins and again when it ends. It did not say how they are laid
//! out, because nothing had to read them yet. **Something does now**, and this
//! module is that layout, written down as
//! [the contract](../../../docs/contracts/kept-undo-folder.md) and held to it by
//! `tests/the_contract_describes_this_folder.rs`.
//!
//! ```text
//! /var/lib/alo/undo/                 the machine's, 0700 root, outside every grant
//!   ada/                             one directory per person
//!     theirs.json                    where that person's settings folder is
//!     4f1c…/                         one directory per kept changing turn
//!       kept.json                    when it ran, and what it did in their words
//!       before/                      a read-only snapshot, as the turn found the home
//!       after/                       a read-only snapshot, as the turn left it
//! ```
//!
//! # Why the moment is in a file rather than in the directory's name
//!
//! A name is read by everything and checked by nothing. A directory whose name
//! is a moment invites a reader to parse one, and the first reader that parses
//! it differently — a different timezone, a different separator, a name
//! somebody renamed by hand — is a machine removing the wrong snapshot. So the
//! name is opaque, nothing here reads it, and the truth is
//! [`TheTurn::taken`] in a file that is refused whole when it does not read.
//!
//! # Why the sentence is kept beside the snapshot
//!
//! ADR 0045's second term: when an undo is let go the record *names the turns
//! that lost it rather than a number*. The unit that removes a snapshot runs as
//! root on a timer, long after the turn, and the person's own record is not
//! its to read. So what the person approved at the time is copied here when the
//! bracket is taken, by the process that already has it — the same *a copy, not
//! a pointer* the record keeps for its own reason.
//!
//! # Why where the person's settings are is kept here too
//!
//! The window is **the person's setting** (ADR 0045's first term), in
//! `undo.toml` in their own folder, and that folder is
//! `$XDG_CONFIG_HOME/alo/` — a variable of the person's *session*, which a root
//! unit on a timer has no way to know and must not guess. Guessing
//! `$HOME/.config` would quietly give the shipped window to exactly the people
//! who had moved theirs. So the session that takes the bracket, which knows,
//! writes [`Theirs::settings`] down; and the unit reads the person's own file at
//! the path the person's own session named.
//!
//! # Nothing here touches a disk
//!
//! This module is the shapes and their refusals. [`crate::found`] is what reads
//! them, and it is the only place in this crate that opens one.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// Where a machine keeps what an undo would put back.
pub const THE_FOLDER: &str = "/var/lib/alo/undo";

/// What says where one person's settings folder is, inside their directory.
pub const THEIRS: &str = "theirs.json";

/// What says when one kept turn ran and what it did, inside its directory.
pub const THE_TURN: &str = "kept.json";

/// The read-only snapshot of the home as the turn found it.
pub const BEFORE: &str = "before";

/// The read-only snapshot of the home as the turn left it.
pub const AFTER: &str = "after";

/// Both snapshots of one kept turn, in the order they are removed.
pub const BOTH_SNAPSHOTS: [&str; 2] = [AFTER, BEFORE];

/// The shape of both files, written at the top of each.
///
/// One number for the folder rather than one per file: they are written
/// together, by one process, in one act, and a machine that could hold a
/// `theirs.json` of one shape beside a `kept.json` of another is a machine with
/// a state nobody designed.
pub const THE_FORMAT: u32 = 1;

/// How long a day is where this crate counts them.
const A_DAY: Duration = Duration::from_secs(24 * 60 * 60);

/// `theirs.json`: where one person's own settings folder is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Theirs {
    /// [`THE_FORMAT`].
    format: u32,
    /// The person's folder — `$XDG_CONFIG_HOME/alo` as their session really has
    /// it, absolute, where `undo.toml` sits beside `settings.toml`.
    settings: PathBuf,
}

/// `kept.json`: when one changing turn ran, and what it did in the person's
/// own words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TheTurn {
    /// [`THE_FORMAT`].
    format: u32,
    /// The moment the turn ran, in seconds since the epoch.
    taken: u64,
    /// What it did, in the sentence the person approved at the time.
    did: String,
}

/// Why one of the two files is not what it says it is.
///
/// A file this says no about is one nothing removes: the snapshot beside it
/// stays on the disk, because removing an undo the machine cannot describe
/// would leave a person unable to be told which turn lost it — which is the
/// whole of ADR 0045's second term.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotWhatItSays {
    /// It is not JSON, or not this file's shape.
    #[error("{at} is not {what} as alo OS writes it: {why}")]
    NotItsShape {
        /// The file.
        at: PathBuf,
        /// Which of the two it should have been.
        what: &'static str,
        /// What the reader said.
        why: String,
    },
    /// It says it is a shape this alo OS does not read.
    #[error("{at} says it is format {found}, and this alo OS writes format 1")]
    AnotherFormat {
        /// The file.
        at: PathBuf,
        /// What it said.
        found: u32,
    },
    /// A person's settings folder that is not an absolute path.
    #[error("{at} names {} as a settings folder, which is not an absolute path", named.display())]
    NotAbsolute {
        /// The file.
        at: PathBuf,
        /// What it named.
        named: PathBuf,
    },
    /// A turn with nothing said about it.
    #[error("{at} says nothing about what the turn did, so nobody could be told it was let go")]
    NothingSaid {
        /// The file.
        at: PathBuf,
    },
}

impl Theirs {
    /// A person whose settings folder is `settings`, as `written_in` says.
    ///
    /// # Errors
    /// [`NotWhatItSays::NotAbsolute`] for anything but an absolute path: a
    /// relative one would be followed from wherever the unit happened to be
    /// started, which is `docs/contracts/person-settings.md`'s own rule about
    /// `$XDG_CONFIG_HOME` and is no different here.
    pub fn at(settings: &Path, written_in: &Path) -> Result<Self, NotWhatItSays> {
        if !settings.is_absolute() {
            return Err(NotWhatItSays::NotAbsolute {
                at: written_in.to_owned(),
                named: settings.to_owned(),
            });
        }
        Ok(Self {
            format: THE_FORMAT,
            settings: settings.to_owned(),
        })
    }

    /// The person's own settings folder.
    #[must_use]
    pub fn settings(&self) -> &Path {
        &self.settings
    }

    /// This file's text, read — the one door, so nothing reaches a reader
    /// without having been through every refusal above.
    ///
    /// # Errors
    /// [`NotWhatItSays`], and nothing in the folder it names is touched.
    pub fn read(text: &str, at: &Path) -> Result<Self, NotWhatItSays> {
        format_is(text, at, THEIRS)?;
        let read: Self = serde_json::from_str(text).map_err(|why| NotWhatItSays::NotItsShape {
            at: at.to_owned(),
            what: THEIRS,
            why: why.to_string(),
        })?;
        Self::at(&read.settings, at)
    }
}

impl TheTurn {
    /// A turn that ran at `taken` and did what `did` says, in the words the
    /// person approved, as the file at `written_in` says.
    ///
    /// # Errors
    /// [`NotWhatItSays::NothingSaid`] for a sentence with nothing in it.
    pub fn done(taken: SystemTime, did: &str, written_in: &Path) -> Result<Self, NotWhatItSays> {
        let did = did.trim();
        if did.is_empty() {
            return Err(NotWhatItSays::NothingSaid {
                at: written_in.to_owned(),
            });
        }
        Ok(Self {
            format: THE_FORMAT,
            taken: taken
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_or(0, |since| since.as_secs()),
            did: did.to_owned(),
        })
    }

    /// The moment the turn ran.
    #[must_use]
    pub fn taken(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(self.taken)
    }

    /// What it did, in the words the person approved.
    #[must_use]
    pub fn did(&self) -> &str {
        &self.did
    }

    /// This file's text, read.
    ///
    /// # Errors
    /// [`NotWhatItSays`], and the snapshots beside it are left alone.
    pub fn read(text: &str, at: &Path) -> Result<Self, NotWhatItSays> {
        format_is(text, at, THE_TURN)?;
        let read: Self = serde_json::from_str(text).map_err(|why| NotWhatItSays::NotItsShape {
            at: at.to_owned(),
            what: THE_TURN,
            why: why.to_string(),
        })?;
        Self::done(read.taken(), &read.did, at)
    }

    /// How many whole days ago this was, counted **down** from `now`, the way
    /// [`alo_keeping_up::HowFarBack`] counts: a turn made six days and
    /// twenty-three hours ago is six days ago.
    ///
    /// A moment in the future is nought days ago rather than an error. A clock
    /// that went backwards is not a reason to keep a snapshot for ever, and it
    /// is not a reason to remove one either: nought is the answer that leaves
    /// the window to decide.
    #[must_use]
    pub fn days_ago(&self, now: SystemTime) -> u32 {
        let since = now
            .duration_since(self.taken())
            .unwrap_or(Duration::ZERO)
            .as_secs()
            / A_DAY.as_secs();
        u32::try_from(since).unwrap_or(u32::MAX)
    }
}

/// Refuse a format this alo OS does not write, **before anything else is read**.
///
/// The order matters and is `alo_kept`'s: a file a later release wrote is
/// refused for being a later release's, not for missing a field this one
/// happens to expect. The second sentence is one nobody can act on.
fn format_is(text: &str, at: &Path, what: &'static str) -> Result<(), NotWhatItSays> {
    let said: serde_json::Value =
        serde_json::from_str(text).map_err(|why| NotWhatItSays::NotItsShape {
            at: at.to_owned(),
            what,
            why: why.to_string(),
        })?;
    match said.get("format").and_then(serde_json::Value::as_u64) {
        Some(found) if found == u64::from(THE_FORMAT) => Ok(()),
        Some(found) => Err(NotWhatItSays::AnotherFormat {
            at: at.to_owned(),
            found: u32::try_from(found).unwrap_or(u32::MAX),
        }),
        None => Err(NotWhatItSays::NotItsShape {
            at: at.to_owned(),
            what,
            why: "it does not say which format it is".to_owned(),
        }),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The file a test pretends it read.
    fn at() -> PathBuf {
        PathBuf::from("/var/lib/alo/undo/ada/4f1c/kept.json")
    }

    /// A moment far enough from the epoch to read as a real one.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// **Both files are written and read back as themselves**, which is the
    /// whole of what the contract promises anybody outside this repository.
    #[test]
    fn both_files_read_back_as_what_was_written() {
        let theirs = Theirs::at(Path::new("/var/home/ada/.config/alo"), &at()).unwrap();
        let written = serde_json::to_string(&theirs).unwrap();
        assert_eq!(Theirs::read(&written, &at()).unwrap(), theirs);
        assert_eq!(theirs.settings(), Path::new("/var/home/ada/.config/alo"));

        let turn = TheTurn::done(noon(), "  move March.pdf into Invoices ", &at()).unwrap();
        let written = serde_json::to_string(&turn).unwrap();
        let read = TheTurn::read(&written, &at()).unwrap();
        assert_eq!(read, turn);
        assert_eq!(read.taken(), noon());
        assert_eq!(read.did(), "move March.pdf into Invoices");
    }

    /// **A settings folder that is not absolute is refused**, on the way in and
    /// when read back off a disk, because following one would put a person's
    /// window wherever the unit was started from.
    #[test]
    fn a_settings_folder_that_is_not_absolute_is_refused() {
        assert!(matches!(
            Theirs::at(Path::new(".config/alo"), &at()),
            Err(NotWhatItSays::NotAbsolute { .. })
        ));
        assert!(matches!(
            Theirs::read(r#"{"format":1,"settings":".config/alo"}"#, &at()),
            Err(NotWhatItSays::NotAbsolute { .. })
        ));
    }

    /// **A turn with nothing said about it is refused**, so that nothing is
    /// ever removed that a person could not be told they had lost.
    #[test]
    fn a_turn_with_nothing_said_about_it_is_refused() {
        assert!(matches!(
            TheTurn::done(noon(), "   ", &at()),
            Err(NotWhatItSays::NothingSaid { .. })
        ));
        assert!(matches!(
            TheTurn::read(r#"{"format":1,"taken":1760000000,"did":""}"#, &at()),
            Err(NotWhatItSays::NothingSaid { .. })
        ));
    }

    /// **Another format is refused whole**, and so is text that is not the
    /// file's shape at all — including each file read as the other.
    #[test]
    fn another_format_and_anything_that_is_not_the_shape_are_refused() {
        assert!(matches!(
            TheTurn::read(r#"{"format":2,"taken":1760000000,"did":"x"}"#, &at()),
            Err(NotWhatItSays::AnotherFormat { found: 2, .. })
        ));
        assert!(matches!(
            Theirs::read(
                r#"{"format":9,"settings":"/var/home/ada/.config/alo"}"#,
                &at()
            ),
            Err(NotWhatItSays::AnotherFormat { found: 9, .. })
        ));
        assert!(matches!(
            TheTurn::read("not json at all", &at()),
            Err(NotWhatItSays::NotItsShape { .. })
        ));
        assert!(matches!(
            Theirs::read(r#"{"format":1,"taken":1760000000,"did":"x"}"#, &at()),
            Err(NotWhatItSays::NotItsShape { .. })
        ));
    }

    /// **Days are counted down**, the way the window counts them: six days and
    /// twenty-three hours ago is six days ago and inside a window of seven.
    #[test]
    fn days_are_counted_down_the_way_the_window_counts_them() {
        let turn = TheTurn::done(noon(), "x", &at()).unwrap();
        assert_eq!(turn.days_ago(noon()), 0);
        assert_eq!(
            turn.days_ago(noon() + A_DAY * 6 + Duration::from_secs(82_800)),
            6
        );
        assert_eq!(turn.days_ago(noon() + A_DAY * 7), 7);
    }

    /// **A moment in the future is nought days ago, not a refusal.** A clock
    /// that went backwards is not a reason to keep a snapshot for ever, and the
    /// window decides either way.
    #[test]
    fn a_moment_in_the_future_is_nought_days_ago() {
        let turn = TheTurn::done(noon() + A_DAY * 3, "x", &at()).unwrap();
        assert_eq!(turn.days_ago(noon()), 0);
    }
}
