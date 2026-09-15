//! The one fact kept across a restart: which build this machine was running
//! the last time it looked.
//!
//! The base says which build it booted, and not whether this start is the
//! first on it, so this file holds the build last seen — one digest, one line,
//! in a file whose path the caller hands in. `alo_keeping_up::Since` compares
//! it with what boots.
//!
//! **Written whole or not at all.** A new value goes to a file beside it,
//! synced, then renamed over the old one, so a machine that loses power while
//! writing it keeps either the old build or the new one and never half of
//! either.
//!
//! **Read strictly.** What is there has to be a whole digest; anything else is
//! refused rather than read as *nothing known*, for the reason
//! [`crate::NotRecorded::LastKnownNotRead`] gives.

use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use alo_keeping_up::Digest;

use crate::refusing::NotRecorded;

/// The build last known, or [`None`] if nothing has been kept at `path`.
///
/// # Errors
/// [`NotRecorded::LastKnownNotRead`] when something is there that cannot be
/// read, or is not a whole digest.
pub fn read(path: &Path) -> Result<Option<Digest>, NotRecorded> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(why) if why.kind() == ErrorKind::NotFound => return Ok(None),
        Err(why) => return Err(not_read(path, &why.to_string())),
    };
    let Some(line) = text.strip_suffix('\n') else {
        return Err(not_read(path, "it does not end in a newline"));
    };
    Digest::read(line)
        .map(Some)
        .map_err(|why| not_read(path, &format!("{why:?}")))
}

/// Keep `digest` as the build last known, whole or not at all.
///
/// # Errors
/// [`NotRecorded::LastKnownNotKept`] with what the machine said.
pub fn keep(path: &Path, digest: &Digest) -> Result<(), NotRecorded> {
    let beside = beside(path);
    let written = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&beside)
        .and_then(|mut file| {
            file.write_all(format!("{}\n", digest.as_str()).as_bytes())?;
            file.sync_all()
        })
        .and_then(|()| std::fs::rename(&beside, path))
        .and_then(|()| match path.parent() {
            Some(folder) if !folder.as_os_str().is_empty() => File::open(folder)?.sync_all(),
            _ => Ok(()),
        });
    written.map_err(|why| NotRecorded::LastKnownNotKept {
        path: path.display().to_string(),
        why: why.to_string(),
    })
}

/// Where a new value is written before it replaces the old one.
fn beside(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".new");
    PathBuf::from(name)
}

/// The refusal for something at `path` that could not be read as a build.
fn not_read(path: &Path, why: &str) -> NotRecorded {
    NotRecorded::LastKnownNotRead {
        path: path.display().to_string(),
        why: why.to_owned(),
    }
}
