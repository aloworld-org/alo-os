//! Private bookkeeping for document permissions derived from the one grants list.

use alo_capability::GrantId;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

/// Maximum private journal size, including entries awaiting revocation.
const MAX_BYTES: u64 = 4 * 1024 * 1024;
/// Versioned, replace-whole bookkeeping, never an authority for access.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Journal {
    /// A future bookkeeping shape must never be read as an empty old one.
    format: u32,
    /// Document exports whose grants must still stand.
    pub(crate) exports: Vec<Export>,
}

impl Default for Journal {
    fn default() -> Self {
        Self {
            format: 1,
            exports: Vec::new(),
        }
    }
}
/// A rented document permission and the alo grant it materialises.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Export {
    /// The sole authority, visible and revocable in Settings.
    pub(crate) grant: GrantId,
    /// The sandbox authenticated from the bus caller.
    pub(crate) application: String,
    /// The opaque identifier returned by the Documents portal.
    pub(crate) document: String,
}

/// Read only an owner-controlled regular file, refusing a corrupt journal whole.
pub(crate) fn read(at: &Path) -> io::Result<Journal> {
    let fd = match rustix::fs::open(
        at,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    ) {
        Ok(fd) => fd,
        Err(rustix::io::Errno::NOENT) => return Ok(Journal::default()),
        Err(why) => return Err(why.into()),
    };
    let file = File::from(fd);
    let about = file.metadata()?;
    if !about.is_file()
        || about.uid() != rustix::process::geteuid().as_raw()
        || about.mode() & 0o077 != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "document bookkeeping must be an owner-only regular file",
        ));
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "document bookkeeping is too large",
        ));
    }
    let journal: Journal = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    if journal.format != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported document bookkeeping format",
        ));
    }
    Ok(journal)
}

/// Keep bookkeeping before a new document URI is handed to an application.
pub(crate) fn write(at: &Path, journal: &Journal) -> io::Result<()> {
    let bytes = serde_json::to_vec(journal).map_err(io::Error::other)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "document bookkeeping is too large",
        ));
    }
    let fresh = at.with_extension(format!("{}-new", rustix::process::getpid().as_raw()));
    // Only this frontend writes the journal. A stale sibling is its own failed
    // replacement; create_new still refuses links and other competing objects.
    match std::fs::remove_file(&fresh) {
        Ok(()) => {}
        Err(why) if why.kind() == io::ErrorKind::NotFound => {}
        Err(why) => return Err(why),
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&fresh)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(fresh, at)
}
