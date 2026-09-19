//! Where the answer is kept, so that a surface reads it back without asking
//! again.
//!
//! A check is a departure. A settings panel that asked again every time
//! somebody opened it would put a line on the indicator for a question already
//! answered a minute ago, which is the opposite of what law 1 makes the
//! indicator for. So one check writes one file, under `/var` beside the two
//! facts `alo-updating` already keeps across a restart, and every surface reads
//! it.
//!
//! # What is kept is not an offer
//!
//! `alo_keeping_up::Offered` deliberately does not deserialise: *one read back
//! off a disk would be an offer nobody was shown being asked for.* That holds
//! here, so what this file writes is **not** an offer and what it reads back is
//! **not** one. [`TheAnswer`] can say what a person reads and can say whether
//! an update was ready; it cannot be turned into the `Ready` that
//! `alo_keeping_up::Staging` needs, and nothing in this crate will give it one.
//!
//! Which means a person choosing *apply it* is a check away from applying it,
//! and that is correct rather than a gap: the update is staged against the
//! machine **as it is at that moment** (`alo_updating::apply`), and the thing
//! that makes that safe is that the offer was heard while it was fetched.
//! Carrying a decided update across a process is
//! [ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)'s
//! open question, not this file's.
//!
//! # A kept answer names the build it was about
//!
//! And is refused when that is no longer the build running — see
//! [`TheAnswer::said`]. *An update is ready* on a machine that has updated
//! since, gone back since, or been changed by somebody at a root shell is a
//! sentence about a machine that no longer exists.
//!
//! # Written whole or not at all, and read strictly
//!
//! The same two rules `alo_updating::one_build` is written to, for the same
//! reasons: a new answer goes to a file beside this one, is synced, renamed
//! over it and the folder synced after, so a machine that loses power mid-write
//! keeps the old answer or the new one and never half of either; and text that
//! is not an answer is refused rather than read as *nothing kept*, because
//! *nothing kept* is what a surface takes as *this machine has never looked*.

use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use alo_keeping_up::{Digest, Running, Standing, words as update_words};
use alo_strings::{Filling, Said, Strings};

use crate::because::Because;
use crate::found::Found;

/// Where the answer is kept on an alo OS machine, beside the record and the
/// two facts an update already keeps across a restart.
pub const THE_ANSWER: &str = "/var/lib/alo/an-update-was-found";

/// The answer, as it is written down.
///
/// Its own shape rather than `alo_keeping_up::Standing`'s, because that type
/// does not read back and must not start: what is written here is a note about
/// a past check, and what that type means is *this machine may stage this*.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    /// The build this machine was running when it asked.
    about: Digest,
    /// The build offered, where one was and it differed.
    offered: Option<Digest>,
    /// Why it asked.
    because: Because,
    /// When it asked, in seconds since the epoch.
    at: u64,
}

/// Why a kept answer could not be read, written or forgotten.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotKept {
    /// Something is there and it is not an answer this machine wrote.
    ///
    /// Refused rather than read as nothing kept, which a surface would take as
    /// *this machine has never looked*.
    #[error("what is kept at {path} is not an answer about updates: {why}")]
    NotUnderstood {
        /// Where it is kept.
        path: String,
        /// What was wrong with it.
        why: String,
    },
    /// It could not be read, written or removed.
    #[error("the answer about updates at {path} could not be kept: {why}")]
    NotWritten {
        /// Where it is kept.
        path: String,
        /// What the machine said.
        why: String,
    },
}

/// Where this machine keeps what its last check answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    /// The file.
    path: PathBuf,
}

/// A kept answer, read back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheAnswer {
    /// The build this machine was running when it asked.
    about: Digest,
    /// The build offered, where one was and it differed.
    offered: Option<Digest>,
    /// Why it asked.
    because: Because,
    /// When it asked.
    at: SystemTime,
}

/// A kept answer that is about a build this machine is no longer running.
///
/// Its sentence is `alo-keeping-up`'s own — *this machine changed after the
/// update was found* — because that is exactly what has happened and it is
/// already in the machine's vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the kept answer is about a version this machine is no longer running")]
pub struct NoLongerTrue;

impl Kept {
    /// Where an alo OS machine keeps it, under `/var`, which no update and no
    /// return replaces.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self {
            path: PathBuf::from(THE_ANSWER),
        }
    }

    /// In this folder, under the name an alo OS machine gives it — a test's.
    #[must_use]
    pub fn in_folder(folder: &Path) -> Self {
        Self {
            path: folder.join("an-update-was-found"),
        }
    }

    /// Where it is kept.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Keep what a check answered, whole or not at all.
    ///
    /// # Errors
    /// [`NotKept::NotWritten`].
    pub fn keep(&self, found: &Found) -> Result<(), NotKept> {
        let written = Written {
            about: found.about().clone(),
            offered: match found.standing() {
                Standing::UpToDate => None,
                Standing::Ready(ready) => Some(ready.offered().clone()),
            },
            because: found.because(),
            at: found
                .at()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |since| since.as_secs()),
        };
        let text = serde_json::to_string(&written).map_err(|why| self.not_written(&why))?;
        self.written_whole_or_not_at_all(&format!("{text}\n"))
    }

    /// What the last check answered, or [`None`] if this machine has never
    /// looked.
    ///
    /// # Errors
    /// [`NotKept`].
    pub fn read(&self) -> Result<Option<TheAnswer>, NotKept> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(why) if why.kind() == ErrorKind::NotFound => return Ok(None),
            Err(why) => return Err(self.not_written(&why)),
        };
        let written: Written =
            serde_json::from_str(text.trim_end()).map_err(|why| NotKept::NotUnderstood {
                path: self.path.display().to_string(),
                why: why.to_string(),
            })?;
        Ok(Some(TheAnswer {
            about: written.about,
            offered: written.offered,
            because: written.because,
            at: UNIX_EPOCH + Duration::from_secs(written.at),
        }))
    }

    /// Keep nothing; nothing kept already is not a failure.
    ///
    /// # Errors
    /// [`NotKept::NotWritten`].
    pub fn forget(&self) -> Result<(), NotKept> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => self.synced_folder().map_err(|why| self.not_written(&why)),
            Err(why) if why.kind() == ErrorKind::NotFound => Ok(()),
            Err(why) => Err(self.not_written(&why)),
        }
    }

    /// Write `text` beside the file, sync it, rename it over, sync the folder.
    fn written_whole_or_not_at_all(&self, text: &str) -> Result<(), NotKept> {
        let mut beside = self.path.as_os_str().to_owned();
        beside.push(".new");
        let beside = PathBuf::from(beside);
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&beside)
            .and_then(|mut file| {
                file.write_all(text.as_bytes())?;
                file.sync_all()
            })
            .and_then(|()| std::fs::rename(&beside, &self.path))
            .and_then(|()| self.synced_folder())
            .map_err(|why| self.not_written(&why))
    }

    /// Sync the folder the answer is in, so a rename survives power loss.
    fn synced_folder(&self) -> std::io::Result<()> {
        match self.path.parent() {
            Some(folder) if !folder.as_os_str().is_empty() => File::open(folder)?.sync_all(),
            _ => Ok(()),
        }
    }

    /// What went wrong, in words, for whoever administers the machine.
    fn not_written(&self, why: &impl std::fmt::Display) -> NotKept {
        NotKept::NotWritten {
            path: self.path.display().to_string(),
            why: why.to_string(),
        }
    }
}

impl TheAnswer {
    /// The build this machine was running when it asked.
    #[must_use]
    pub fn about(&self) -> &Digest {
        &self.about
    }

    /// Why it asked.
    #[must_use]
    pub fn because(&self) -> Because {
        self.because
    }

    /// When it asked.
    #[must_use]
    pub fn at(&self) -> SystemTime {
        self.at
    }

    /// Whether this answer is still about the machine in front of the person.
    #[must_use]
    pub fn is_still_about(&self, running: &Running) -> bool {
        running.digest() == &self.about
    }

    /// Whether an update was ready — asked only of an answer that is still
    /// about this machine.
    ///
    /// # Errors
    /// [`NoLongerTrue`] when this machine is running a different version from
    /// the one the answer was about.
    pub fn is_ready(&self, running: &Running) -> Result<bool, NoLongerTrue> {
        if self.is_still_about(running) {
            Ok(self.offered.is_some())
        } else {
            Err(NoLongerTrue)
        }
    }

    /// What a person reads, when the answer is still about their machine.
    ///
    /// The sentence is `alo_keeping_up::Standing`'s, unchanged — this reads an
    /// answer back rather than wording one.
    ///
    /// # Errors
    /// [`NoLongerTrue`], whose own sentence says the machine changed after the
    /// update was found and that checking again finds the right one.
    pub fn said(&self, running: &Running, strings: &Strings) -> Result<Said, NoLongerTrue> {
        let word = if self.is_ready(running)? {
            update_words::READY
        } else {
            update_words::UP_TO_DATE
        };
        Ok(strings.say(&word.key(), &Filling::nothing()))
    }
}

impl NoLongerTrue {
    /// What a person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(
            &update_words::CHANGED_SINCE_IT_WAS_FOUND.key(),
            &Filling::nothing(),
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_folder, a_moment, an_update, build, in_english, up_to_date};

    /// **It is kept under `/var/lib/alo`**, beside what an update already keeps
    /// across a restart, which is what an update and a return both leave alone.
    #[test]
    fn it_is_kept_beside_what_an_update_already_keeps() {
        assert_eq!(
            Kept::on_this_machine().path(),
            Path::new("/var/lib/alo/an-update-was-found")
        );
        assert_eq!(
            Kept::in_folder(Path::new("/tmp/x")).path().file_name(),
            Kept::on_this_machine().path().file_name()
        );
    }

    /// An answer kept is read back, and a machine that has never looked says
    /// so rather than saying it is up to date.
    #[test]
    fn an_answer_kept_is_read_back_and_never_looked_is_not_up_to_date() {
        let folder = a_folder("round");
        let kept = Kept::in_folder(&folder);
        assert_eq!(kept.read().unwrap(), None);

        kept.keep(&an_update(build("aa"), build("bb"))).unwrap();
        let read = kept.read().unwrap().unwrap();
        assert_eq!(read.about(), &build("aa"));
        assert_eq!(read.at(), a_moment());
        assert!(read.is_ready(&Running::reported(build("aa"))).unwrap());

        kept.keep(&up_to_date(build("aa"))).unwrap();
        assert!(
            !kept
                .read()
                .unwrap()
                .unwrap()
                .is_ready(&Running::reported(build("aa")))
                .unwrap()
        );

        kept.forget().unwrap();
        assert_eq!(kept.read().unwrap(), None);
        kept.forget().unwrap();
        drop(std::fs::remove_dir_all(&folder));
    }

    /// **A kept answer about a build this machine no longer runs is refused**,
    /// rather than being shown stale — and what the person reads is that the
    /// machine changed and to check again.
    #[test]
    fn an_answer_about_a_build_this_machine_no_longer_runs_is_refused() {
        let folder = a_folder("stale");
        let kept = Kept::in_folder(&folder);
        kept.keep(&an_update(build("aa"), build("bb"))).unwrap();
        let read = kept.read().unwrap().unwrap();

        let now = Running::reported(build("bb"));
        assert!(!read.is_still_about(&now));
        assert_eq!(read.is_ready(&now), Err(NoLongerTrue));
        assert_eq!(read.said(&now, &in_english()), Err(NoLongerTrue));
        assert_eq!(
            NoLongerTrue.said(&in_english()).text(),
            "This machine changed after the update was found, so nothing was changed. Check for \
             the update again"
        );
        drop(std::fs::remove_dir_all(&folder));
    }

    /// The sentence a surface reads back is the one a person already met.
    #[test]
    fn the_sentence_read_back_is_the_one_that_already_existed() {
        let folder = a_folder("sentence");
        let kept = Kept::in_folder(&folder);
        let running = Running::reported(build("aa"));

        kept.keep(&an_update(build("aa"), build("bb"))).unwrap();
        let said = kept
            .read()
            .unwrap()
            .unwrap()
            .said(&running, &in_english())
            .unwrap();
        assert!(said.text().starts_with("An update is ready"), "{said}");

        kept.keep(&up_to_date(build("aa"))).unwrap();
        assert_eq!(
            kept.read()
                .unwrap()
                .unwrap()
                .said(&running, &in_english())
                .unwrap()
                .text(),
            "This machine is up to date"
        );
        drop(std::fs::remove_dir_all(&folder));
    }

    /// **Something there that is not an answer is refused**, rather than read
    /// as a machine that has never looked — half a build included.
    #[test]
    fn what_is_there_and_is_not_an_answer_is_refused() {
        let folder = a_folder("nonsense");
        let kept = Kept::in_folder(&folder);
        for text in [
            "not json at all",
            "{}",
            "{\"about\":\"sha256:beef\",\"offered\":null,\"because\":\"the-person-asked\",\"at\":1}",
            "{\"about\":\"sha256:aa\",\"nothing\":1}",
        ] {
            std::fs::write(kept.path(), text).unwrap();
            let refused = kept.read().unwrap_err();
            assert!(
                matches!(refused, NotKept::NotUnderstood { .. }),
                "`{text}`: {refused}"
            );
        }
        drop(std::fs::remove_dir_all(&folder));
    }

    /// **A folder that is not there is a refusal**, not a silently lost answer.
    #[test]
    fn an_answer_that_could_not_be_written_says_so() {
        let kept = Kept::in_folder(Path::new("/nowhere/at/all/alo-looking"));
        let refused = kept.keep(&up_to_date(build("aa"))).unwrap_err();
        assert!(matches!(refused, NotKept::NotWritten { .. }), "{refused}");
    }

    /// What is written names no person and no agent: a check is something the
    /// machine did.
    #[test]
    fn what_is_written_names_no_agent() {
        let folder = a_folder("written");
        let kept = Kept::in_folder(&folder);
        kept.keep(&an_update(build("aa"), build("bb"))).unwrap();
        let text = std::fs::read_to_string(kept.path()).unwrap();
        assert!(text.contains("the-person-asked"), "{text}");
        assert!(!text.contains("agent"), "{text}");
        assert!(text.ends_with('\n'), "{text}");
        drop(std::fs::remove_dir_all(&folder));
    }
}
