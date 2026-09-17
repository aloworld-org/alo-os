//! What applications were answered, kept on the disk after the backend stops.
//!
//! [`AnswersFile`] is the [`Recording`] a machine runs. **One file per login**,
//! in that person's own state directory (`crate::where_the_answers_are`): its
//! first line says which format it is in, and every line after it is one answer
//! ([`KeptAnswer`]), in the order given.
//! `docs/contracts/portal-answers-file.md` is the shape for people reading it
//! without this crate.
//!
//! # Whose it is
//!
//! [ADR 0052](../../../docs/decisions/0052-what-a-persons-applications-asked-for-is-the-persons-record.md):
//! the record of what somebody's applications asked for and were told is
//! **theirs**. It is not the agent's record, which is the machine's and which an
//! organisation may export (ADR 0004); a list of every time a person's calendar
//! asked for their microphone is a description of their working day.
//!
//! # Appended to, never rewritten
//!
//! As the agent's record is (`docs/contracts/record-file.md`), for that
//! contract's reasons: keeping an answer does not read the file, a write the
//! machine interrupts costs the answer being written and nothing before it, and
//! a reader needs a JSON parser and nothing else. **Each answer is synced
//! before [`Recording::keep`] returns**, so the application is never answered
//! with something a power cut takes out of the record.
//!
//! # A backend started again finds what the last one wrote
//!
//! [`AnswersFile::opened`] adds to the file that is there. A file that ends in
//! the middle of a line — the machine stopped while one was written — has a
//! line ending put after it before the next answer, so the torn line stays one
//! line that did not read and the next answer is whole. A file whose first line
//! is not a format line, or names a format newer than this, is refused rather
//! than added to.
//!
//! # Kept as long as the machine's record, and no longer
//!
//! The one exception to *never rewritten* is [`AnswersFile::shortened`]
//! (`crate::shortening`), under the same `[record].keeping` rule the agent's
//! record is shortened under. It says so in the first line (`crate::answers_head`).
//!
//! # Who may have written it
//!
//! `crate::believed_file`: not a link, a regular file, **this login's**, and
//! nobody else can write it. The folder is made once, by
//! `crate::where_the_answers_are`, inside a directory that is already there.

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::SystemTime;

use alo_keeping::Keeping;

use crate::answered::Answered;
use crate::answers_head::AnswersHead;
use crate::believed_file::{opened_to_add_to, opened_to_read};
use crate::kept_answer::KeptAnswer;
use crate::not_recorded::NotRecorded;
use crate::recording::Recording;

/// Where a machine kept what applications were answered before ADR 0052 moved
/// it into each person's own state directory.
///
/// Kept as a name so that a machine upgraded from a pre-release image can say
/// what the file it is no longer reading was. **Nothing opens it**: it may hold
/// two people's answers, there is no honest way to split one, and deleting
/// somebody's record is a thing a person does rather than a thing an upgrade
/// does. `crate::where_the_answers_are` is where the file is now.
pub const WHERE_IT_USED_TO_BE: &str = "/var/lib/alo/portal-answers.jsonl";

/// The longest first line read looking for the format. A format line is a
/// dozen bytes; anything this long is not one.
const LONGEST_HEAD: u64 = 4096;

/// Every answer, kept in a file.
#[derive(Debug)]
pub struct AnswersFile {
    /// Where it is, for what a refusal names.
    pub(crate) at: PathBuf,
    /// The file, and whether the last write may have left half a line.
    ///
    /// Held by every answer while it is written and by a shortening for the
    /// whole of its replacing the file, so no answer is kept — and so none is
    /// sent — while the file is replaced.
    pub(crate) open: Mutex<Open>,
}

/// The file as it is held between answers.
#[derive(Debug)]
pub(crate) struct Open {
    /// The file, opened to append.
    pub(crate) file: File,
    /// Whether the file may end in the middle of a line.
    pub(crate) torn: bool,
}

/// Every answer read back off the disk, oldest first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadBack {
    /// The answers that read, in the order they were given.
    pub answers: Vec<KeptAnswer>,
    /// The number of each line that did not read, counting the format line as
    /// line 1 — reported beside what did, never in place of it.
    pub unreadable: Vec<usize>,
    /// The moment the file now starts at, once a shortening has removed
    /// anything — so an answer missing from before it is not read as a request
    /// nobody made. [`None`] for a file nothing has been removed from.
    pub since: Option<SystemTime>,
    /// The rule the last shortening that removed anything was under.
    pub under: Option<Keeping>,
}

impl AnswersFile {
    /// The answers file at `at`, made with its format line when it is not
    /// there, and added to when it is.
    ///
    /// # Errors
    /// Everything `crate::believed_file` refuses about who could have written
    /// it; [`NotRecorded::NotAnAnswersFile`] for a file whose first line is not
    /// a format line; [`NotRecorded::ANewerFormat`]; and
    /// [`NotRecorded::NotRead`] or [`NotRecorded::NotWritten`] for what the
    /// machine said — a folder that is not there among them.
    pub fn opened(at: &Path) -> Result<Self, NotRecorded> {
        let mut file = opened_to_add_to(at)?;
        let not_read = |why: std::io::Error| NotRecorded::NotRead {
            at: at.to_owned(),
            why: why.to_string(),
        };
        let length = file.metadata().map_err(not_read)?.len();
        let torn = if length == 0 {
            let head = AnswersHead::new()
                .line()
                .map_err(|why| NotRecorded::NotWritten {
                    at: at.to_owned(),
                    why: why.to_string(),
                })?;
            file.write_all(head.as_bytes())
                .and_then(|()| file.sync_data())
                .map_err(|why| NotRecorded::NotWritten {
                    at: at.to_owned(),
                    why: why.to_string(),
                })?;
            false
        } else {
            let mut first = String::new();
            BufReader::new((&file).take(LONGEST_HEAD))
                .read_line(&mut first)
                .map_err(not_read)?;
            AnswersHead::read(at, &first)?;
            file.seek(SeekFrom::End(-1)).map_err(not_read)?;
            let mut last = [0_u8; 1];
            file.read_exact(&mut last).map_err(not_read)?;
            last != *b"\n"
        };
        Ok(Self {
            at: at.to_owned(),
            open: Mutex::new(Open { file, torn }),
        })
    }

    /// Every answer kept in the file at `at`, in the order given.
    ///
    /// # Errors
    /// [`NotRecorded::NotThere`] when no file is there — nothing has been
    /// answered yet, told apart from every failure; everything
    /// `crate::believed_file` refuses; [`NotRecorded::NotAnAnswersFile`] and
    /// [`NotRecorded::ANewerFormat`] for its first line; and
    /// [`NotRecorded::NotRead`].
    pub fn read_back(at: &Path) -> Result<ReadBack, NotRecorded> {
        let mut bytes = Vec::new();
        opened_to_read(at)?
            .read_to_end(&mut bytes)
            .map_err(|why| NotRecorded::NotRead {
                at: at.to_owned(),
                why: why.to_string(),
            })?;
        let mut lines = bytes.split(|byte| *byte == b'\n');
        let first = lines.next().and_then(|line| std::str::from_utf8(line).ok());
        let head = AnswersHead::read(at, first.unwrap_or_default())?;
        let mut read = ReadBack {
            answers: Vec::new(),
            unreadable: Vec::new(),
            since: head.since(),
            under: head.under(),
        };
        for (number, line) in lines.enumerate() {
            if line.is_empty() {
                continue;
            }
            match std::str::from_utf8(line).ok().and_then(KeptAnswer::read) {
                Some(answer) => read.answers.push(answer),
                None => read.unreadable.push(number + 2),
            }
        }
        Ok(read)
    }

    /// Where the file is.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }
}

impl Recording for AnswersFile {
    fn keep(&self, answered: Answered) -> Result<(), NotRecorded> {
        let not_written = |why: String| NotRecorded::NotWritten {
            at: self.at.clone(),
            why,
        };
        let line = KeptAnswer::from(&answered)
            .line()
            .map_err(|why| not_written(why.to_string()))?;
        // A panic elsewhere while the file was held leaves the file, and
        // `torn` says what the last write may have left in it.
        let mut open = self.open.lock().unwrap_or_else(PoisonError::into_inner);
        let mut written = String::with_capacity(line.len() + 2);
        if open.torn {
            written.push('\n');
        }
        written.push_str(&line);
        written.push('\n');
        // Torn until the whole line is down and synced: a write that stops
        // part of the way leaves half a line for the next answer to end.
        open.torn = true;
        open.file
            .write_all(written.as_bytes())
            .and_then(|()| open.file.sync_data())
            .map_err(|why| not_written(why.to_string()))?;
        open.torn = false;
        Ok(())
    }
}
