//! A folder of pictures, one at a time.
//!
//! **This crate does not read the folder and does not read the clock**, which is
//! the same rule item 1 set for grants and for the same reason: what is on the
//! disk and what time it is are the compositor's to know, and a model that went
//! and looked could not be tested without a disk and a wait.
//! [`Rotating::showing`] is therefore asked *how many pictures the folder holds*
//! and *how long the rotation has been running*, and answers with a position.
//! Which file is at that position is decided where the folder is read.
//!
//! **The order is the folder's own**, sorted by name, rather than shuffled.
//! Shuffling needs a source of randomness, and a background that cannot say
//! which picture comes next is a background nobody can predict — including the
//! person who put a photograph they would rather not have on the screen when a
//! colleague walks past.
//!
//! **A rotation is at least a minute.** Faster than that is not a background
//! changing, it is a screen flickering, and a flickering screen is an
//! accessibility problem rather than a preference.

use std::path::PathBuf;
use std::time::Duration;

use alo_strings::{Filling, Said, Strings, Word};
use serde::{Deserialize, Serialize};

use crate::picture::Fitting;
use crate::unreadable::NotRead;
use crate::words;

/// The shortest a picture may stay on the screen.
const AT_LEAST: Duration = Duration::from_secs(60);

/// Why a rotating background cannot be used.
///
/// There is no `Display`: the only road to words is [`RotatingError::said`], and
/// what a settings file that did not read writes is [`NotRead`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RotatingError {
    /// A folder given as a relative path.
    NotAWholePath(PathBuf),
    /// A rotation faster than a minute.
    TooQuick,
}

impl RotatingError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> Word {
        match *self {
            Self::NotAWholePath(_) => words::FOLDER_NOT_A_WHOLE_PATH,
            Self::TooQuick => words::TOO_QUICK,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NotAWholePath(folder) => Filling::of("folder", folder.display().to_string()),
            Self::TooQuick => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// How often the picture changes.
///
/// Reads back through [`Every::checked`], so a hand-edited file cannot ask for
/// a background that changes every second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Duration", into = "Duration")]
pub struct Every {
    /// How long one picture stays up, at least [`AT_LEAST`].
    long: Duration,
}

impl Every {
    /// A rotation this long.
    ///
    /// # Errors
    /// [`RotatingError::TooQuick`] for anything under a minute.
    pub fn checked(long: Duration) -> Result<Self, RotatingError> {
        if long < AT_LEAST {
            return Err(RotatingError::TooQuick);
        }
        Ok(Self { long })
    }

    /// This many minutes.
    ///
    /// # Errors
    /// [`RotatingError::TooQuick`] for none of them.
    pub fn minutes(count: u32) -> Result<Self, RotatingError> {
        Self::checked(Duration::from_secs(u64::from(count).saturating_mul(60)))
    }

    /// This many hours.
    ///
    /// # Errors
    /// [`RotatingError::TooQuick`] for none of them.
    pub fn hours(count: u32) -> Result<Self, RotatingError> {
        Self::minutes(count.saturating_mul(60))
    }

    /// How long one picture stays up.
    #[must_use]
    pub fn long(self) -> Duration {
        self.long
    }
}

impl TryFrom<Duration> for Every {
    type Error = NotRead;

    fn try_from(long: Duration) -> Result<Self, Self::Error> {
        Self::checked(long).map_err(|refused| NotRead::about(refused.word()))
    }
}

impl From<Every> for Duration {
    fn from(every: Every) -> Self {
        every.long
    }
}

/// A folder whose pictures take turns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rotating {
    /// Where the pictures are.
    folder: PathBuf,
    /// How often the picture changes.
    every: Every,
    /// How each of them meets the edges.
    fitting: Fitting,
}

impl Rotating {
    /// This folder, changing this often.
    ///
    /// # Errors
    /// [`RotatingError::NotAWholePath`] for a folder that is not named in full.
    pub fn folder(folder: PathBuf, every: Every) -> Result<Self, RotatingError> {
        if !folder.is_absolute() {
            return Err(RotatingError::NotAWholePath(folder));
        }
        Ok(Self {
            folder,
            every,
            fitting: Fitting::default(),
        })
    }

    /// The same folder, with its pictures meeting the edges differently.
    #[must_use]
    pub fn fitted(mut self, fitting: Fitting) -> Self {
        self.fitting = fitting;
        self
    }

    /// Where the pictures are.
    #[must_use]
    pub fn where_they_are(&self) -> &PathBuf {
        &self.folder
    }

    /// How often the picture changes.
    #[must_use]
    pub fn every(&self) -> Every {
        self.every
    }

    /// How each picture meets the edges.
    #[must_use]
    pub fn fitting(&self) -> Fitting {
        self.fitting
    }

    /// Which of the folder's pictures is up, counting from the first, given how
    /// many it holds and how long the rotation has been running.
    ///
    /// `None` when the folder holds nothing: a folder that has been emptied
    /// shows no picture, and what to put there instead is the caller's — the
    /// shipped wallpaper, in [`crate::appearance`].
    #[must_use]
    pub fn showing(&self, holding: usize, running: Duration) -> Option<usize> {
        let holding = u64::try_from(holding).ok().filter(|held| *held > 0)?;
        // `Every` cannot be shorter than a minute, so the division has a
        // divisor; `max(1)` is here so that a later change to that rule cannot
        // turn this into a panic in the middle of drawing a screen.
        let turns = running.as_secs() / self.every.long().as_secs().max(1);
        usize::try_from(turns % holding).ok()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// A folder that exists on no machine: nothing here reads a disk.
    fn folder() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(r"C:\Users\a\Pictures\Rotating")
        } else {
            PathBuf::from("/home/a/Pictures/Rotating")
        }
    }

    /// Ten minutes, which every test below rotates at.
    fn ten_minutes() -> Every {
        Every::minutes(10).unwrap()
    }

    /// The picture advances one place per turn and comes back round, and the
    /// answer depends on nothing but the two things it was told.
    #[test]
    fn the_pictures_take_turns_in_order() {
        let rotating = Rotating::folder(folder(), ten_minutes()).unwrap();
        let at =
            |minutes: u64| rotating.showing(3, Duration::from_secs(minutes.saturating_mul(60)));

        assert_eq!(at(0), Some(0));
        assert_eq!(at(9), Some(0), "it stays up for the whole ten minutes");
        assert_eq!(at(10), Some(1));
        assert_eq!(at(20), Some(2));
        assert_eq!(at(30), Some(0), "and comes back round");
        assert_eq!(at(1_000_000), Some(1), "a machine left on for two years");
    }

    /// **An empty folder shows nothing**, and says so rather than answering
    /// with a position that does not exist.
    #[test]
    fn an_empty_folder_shows_nothing() {
        let rotating = Rotating::folder(folder(), ten_minutes()).unwrap();
        assert_eq!(rotating.showing(0, Duration::from_secs(0)), None);
        assert_eq!(rotating.showing(0, Duration::from_secs(9_999)), None);
        assert_eq!(
            rotating.showing(1, Duration::from_secs(9_999)),
            Some(0),
            "and one picture is on the screen whatever the time is"
        );
    }

    /// **A flicker is refused where it is set.** A background changing faster
    /// than a minute is an accessibility problem, not a preference.
    #[test]
    fn a_rotation_faster_than_a_minute_is_refused() {
        assert_eq!(
            Every::checked(Duration::from_secs(59)),
            Err(RotatingError::TooQuick)
        );
        assert_eq!(Every::minutes(0), Err(RotatingError::TooQuick));
        assert_eq!(
            Every::checked(Duration::from_secs(60)).unwrap().long(),
            Duration::from_secs(60),
            "a minute exactly is allowed"
        );
        assert_eq!(
            Every::hours(2).unwrap().long(),
            Duration::from_secs(2 * 60 * 60)
        );
    }

    /// A file is a thing a person edits, so the minute is checked again where
    /// the file is read.
    #[test]
    fn a_file_cannot_ask_for_a_flicker_either() {
        let written = serde_json::to_string(&ten_minutes()).unwrap();
        assert_eq!(
            serde_json::from_str::<Every>(&written).unwrap(),
            ten_minutes()
        );
        assert!(
            serde_json::from_str::<Every>(r#"{"secs":1,"nanos":0}"#).is_err(),
            "a second is a flicker however it arrives"
        );
    }

    /// A folder is named in full, for the same reason a picture is.
    #[test]
    fn a_folder_is_named_in_full() {
        let relative = PathBuf::from("Pictures/Rotating");
        assert_eq!(
            Rotating::folder(relative.clone(), ten_minutes()),
            Err(RotatingError::NotAWholePath(relative))
        );
    }

    /// **Both refusals are read in the reader's language**, and the one about a
    /// folder names the folder — which came off their disk and is not
    /// translated. The flicker one names nothing, because what is wrong is the
    /// rule rather than the value.
    #[test]
    fn the_refusals_are_read_in_the_readers_language() {
        let strings = in_english();
        let flicker = Every::minutes(0).unwrap_err().said(&strings);
        assert_eq!(
            flicker.text(),
            "leave each picture up for at least a minute — anything quicker is a flicker"
        );
        assert!(flicker.unfilled().is_empty());

        let relative = Rotating::folder(PathBuf::from("Pictures"), ten_minutes())
            .unwrap_err()
            .said(&strings);
        assert!(relative.text().contains("Pictures"), "{relative}");
        assert!(relative.unfilled().is_empty(), "{relative}");

        let auf_deutsch = translated(&[(
            words::TOO_QUICK,
            "lassen Sie jedes Bild mindestens eine Minute stehen — alles Schnellere flackert",
        )]);
        let said = Every::minutes(0).unwrap_err().said(&auf_deutsch);
        assert_eq!(
            said.text(),
            "lassen Sie jedes Bild mindestens eine Minute stehen — alles Schnellere flackert"
        );
        assert!(said.is_translated());
    }

    /// A settings file that asked for a flicker writes the key of the refusal,
    /// because a deserialiser has no `Strings` to ask.
    #[test]
    fn a_file_that_did_not_read_names_the_string_rather_than_saying_it() {
        let refused = serde_json::from_str::<Every>(r#"{"secs":1,"nanos":0}"#).unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("appearance.rotating.too-quick"),
            "{refused}"
        );
    }

    /// A rotating background survives a settings file unchanged.
    #[test]
    fn a_rotation_survives_being_written_down() {
        let rotating = Rotating::folder(folder(), ten_minutes())
            .unwrap()
            .fitted(Fitting::Fit);
        let written = serde_json::to_string(&rotating).unwrap();
        assert_eq!(
            serde_json::from_str::<Rotating>(&written).unwrap(),
            rotating
        );
        assert_eq!(rotating.where_they_are(), &folder());
        assert_eq!(rotating.every(), ten_minutes());
        assert_eq!(rotating.fitting(), Fitting::Fit);
    }
}
