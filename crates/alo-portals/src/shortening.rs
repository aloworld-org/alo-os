//! The answers file shortened under the machine's record rule, and no further.
//!
//! `[record].keeping` in the machine description
//! (`docs/contracts/machine-description.md`) is how long what happened on this
//! machine is kept — the organisation's rule on a managed machine (ADR 0004),
//! the person's on their own. `alo-keeping` shortens the agent's record under
//! it. An application's requests are as much a record of what happened on the
//! machine as an agent's verbs, so [`AnswersFile::shortened`] shortens the
//! answers file under **the same [`Keeping`]**, asking it
//! [`Keeping::oldest_kept`] and [`Keeping::keeps`] rather than restating what a
//! day is or what a clock put back means.
//!
//! # What goes, and what never does
//!
//! - **It takes a rule and a moment, and nothing else.** No answer, application
//!   or portal can be named for removal.
//! - **Only an answer that reads, and is older than the rule keeps, goes.** A
//!   line that did not read — torn by a power cut, or not one this backend could
//!   have written — stays, byte for byte and in its place, because nobody can
//!   say how old it is. This is where the answers file parts from the agent's
//!   record, which refuses to shorten at all around a damaged line: task 9 of
//!   the applications plan asks that the rule hold over the answers that read
//!   *and* that the unreadable line survive, and keeping the line does both.
//! - **A file that loses nothing is not rewritten.** `"forever"`, and a rule
//!   nothing is old enough for yet, leave the file untouched — its first line
//!   too, which a shortening that removed nothing has no business marking.
//! - **It leaves a mark that cannot age out.** Where the file now starts, and
//!   the rule, go into the first line (`crate::answers_head`), which no
//!   shortening walks; `format` stays `1`.
//!
//! # Whole or not at all, and nothing answered meanwhile
//!
//! The shortened file is written beside the file, synced, and renamed over it.
//! A machine that stops partway leaves the file exactly as long as it was, and
//! a file beside it the next shortening removes. **The file's lock is held from
//! the first byte read to the rename**, and every answer is kept under that
//! lock before it is sent (`crate::recording`), so no answer is kept, and none
//! sent, while the file is replaced — and none is kept into the file renamed
//! away. The file written beside is opened to append and becomes the file
//! answers are kept into, so there is no moment after the rename when the
//! backend holds the old one.
//!
//! # What is not here
//!
//! **When a shortening runs, and reading `[record].keeping` off the machine.**
//! Both belong to the session that starts the backend, as they do for the
//! agent's record; this is handed a rule and a moment.

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::PoisonError;
use std::time::SystemTime;

use alo_keeping::Keeping;

use crate::answers_file::AnswersFile;
use crate::answers_head::AnswersHead;
use crate::believed_file::made_to_replace;
use crate::kept_answer::KeptAnswer;
use crate::not_recorded::NotRecorded;

/// What a shortening did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortened {
    /// How many answers were removed.
    removed: usize,
    /// How many answers that read are still there.
    kept: usize,
    /// The moment the file now starts at, where anything has ever been removed.
    since: Option<SystemTime>,
}

impl Shortened {
    /// How many answers were removed.
    #[must_use]
    pub const fn removed(self) -> usize {
        self.removed
    }

    /// How many answers that read are still there. Lines that did not read are
    /// not counted, and are all still there.
    #[must_use]
    pub const fn kept(self) -> usize {
        self.kept
    }

    /// The moment the file now starts at, where anything has ever been removed
    /// from it — by this shortening or an earlier one.
    #[must_use]
    pub const fn since(self) -> Option<SystemTime> {
        self.since
    }

    /// Whether this shortening removed anything.
    #[must_use]
    pub const fn anything_removed(self) -> bool {
        self.removed > 0
    }
}

impl AnswersFile {
    /// Remove every answer `keeping` no longer keeps at `now`, and nothing
    /// else.
    ///
    /// No answer is kept, and so none is sent, until this returns.
    ///
    /// # Errors
    /// In every one, **the file is as it was**:
    /// [`NotRecorded::Replaced`] when the path no longer holds the file this
    /// backend keeps answers in; [`NotRecorded::NotAnAnswersFile`] and
    /// [`NotRecorded::ANewerFormat`] for its first line;
    /// [`NotRecorded::NotRead`]; and [`NotRecorded::NotWritten`] or
    /// [`NotRecorded::ALink`] when the shortened file could not be written
    /// beside it or renamed over it.
    pub fn shortened(&self, keeping: Keeping, now: SystemTime) -> Result<Shortened, NotRecorded> {
        // A panic elsewhere while the file was held leaves the file, as it does
        // for `keep`.
        let mut open = self.open.lock().unwrap_or_else(PoisonError::into_inner);
        let at = self.at.as_path();
        let bytes = everything_in(at, &mut open.file)?;
        still_the_file(at, &open.file)?;

        let mut lines = bytes.split(|byte| *byte == b'\n');
        let first = lines
            .next()
            .and_then(|line| std::str::from_utf8(line).ok())
            .unwrap_or_default();
        let head = AnswersHead::read(at, first)?;

        let mut staying: Vec<&[u8]> = Vec::new();
        let mut removed = 0;
        let mut kept = 0;
        for line in lines.filter(|line| !line.is_empty()) {
            match std::str::from_utf8(line).ok().and_then(KeptAnswer::read) {
                Some(answer) if !keeping.keeps(answer.at(), now) => removed += 1,
                Some(_) => {
                    kept += 1;
                    staying.push(line);
                }
                None => staying.push(line),
            }
        }
        let unchanged = Shortened {
            removed: 0,
            kept,
            since: head.since(),
        };
        let Some(oldest) = keeping.oldest_kept(now).filter(|_| removed > 0) else {
            return Ok(unchanged);
        };

        let head = head.shortened_to(oldest, keeping);
        let not_written = |why: String| NotRecorded::NotWritten {
            at: at.to_owned(),
            why,
        };
        let mut text = head
            .line()
            .map_err(|why| not_written(why.to_string()))?
            .into_bytes();
        for line in staying {
            text.extend_from_slice(line);
            text.push(b'\n');
        }

        let beside = beside(at);
        let replacement = written_beside(&beside, &text)?;
        if let Err(why) = fs::rename(&beside, at) {
            drop(fs::remove_file(&beside));
            return Err(not_written(why.to_string()));
        }
        // From here the file answers are kept into is the one that was renamed
        // into place, and it ends in a whole line.
        open.file = replacement;
        open.torn = false;
        Ok(Shortened {
            removed,
            kept,
            since: head.since(),
        })
    }
}

/// Every byte of the file this backend holds, read from its start.
fn everything_in(at: &Path, file: &mut File) -> Result<Vec<u8>, NotRecorded> {
    let mut bytes = Vec::new();
    file.seek(SeekFrom::Start(0))
        .and_then(|_| file.read_to_end(&mut bytes))
        .map_err(|why| NotRecorded::NotRead {
            at: at.to_owned(),
            why: why.to_string(),
        })?;
    Ok(bytes)
}

/// Whether the path still names the file this backend holds, without following
/// a link — a shortening renames over the path, so it must be over this file.
fn still_the_file(at: &Path, held: &File) -> Result<(), NotRecorded> {
    let replaced = || NotRecorded::Replaced { at: at.to_owned() };
    let there = fs::symlink_metadata(at).map_err(|_| replaced())?;
    let ours = held.metadata().map_err(|why| NotRecorded::NotRead {
        at: at.to_owned(),
        why: why.to_string(),
    })?;
    if there.dev() == ours.dev() && there.ino() == ours.ino() {
        Ok(())
    } else {
        Err(replaced())
    }
}

/// `text`, written into a new file at `beside` and synced, and that file held
/// open to be appended to once it is renamed into place.
///
/// Whatever an interrupted shortening left at `beside` is removed first. A
/// folder or anything else that will not go stays, and the shortening is
/// refused rather than written through it.
fn written_beside(beside: &Path, text: &[u8]) -> Result<File, NotRecorded> {
    // Removing a link removes the link and never what it points at.
    drop(fs::remove_file(beside));
    let mut file = made_to_replace(beside)?;
    if let Err(why) = file.write_all(text).and_then(|()| file.sync_all()) {
        drop(fs::remove_file(beside));
        return Err(NotRecorded::NotWritten {
            at: beside.to_owned(),
            why: why.to_string(),
        });
    }
    Ok(file)
}

/// Where the shortened file is written: beside the file and named after it, so
/// the rename that replaces the file never crosses a filesystem.
fn beside(at: &Path) -> PathBuf {
    let mut beside = at.as_os_str().to_owned();
    beside.push(".shortening");
    PathBuf::from(beside)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The replacement is written in the file's own folder, under a name no
    /// answers file is given.
    #[test]
    fn the_replacement_is_written_beside_the_file() {
        assert_eq!(
            beside(Path::new("/var/lib/alo/portal-answers.jsonl")),
            Path::new("/var/lib/alo/portal-answers.jsonl.shortening")
        );
    }
}
