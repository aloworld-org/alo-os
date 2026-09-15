//! One build, kept in one file across a restart: written whole or not at all,
//! and read strictly.
//!
//! The machine keeps two such facts — the build it last knew it was running
//! ([`crate::last_known`]) and the build the person chose to go back to
//! ([`crate::going_back`]) — and both are exactly this: one digest, one line.
//! What differs is what a failure means, which each of them says in its own
//! refusal; how the line is written and read is here, once.
//!
//! **Written whole or not at all.** A new value goes to a file beside it,
//! synced, then renamed over the old one, and the folder is synced after, so a
//! machine that loses power while writing keeps either the old build or the new
//! one and never half of either.
//!
//! **Read strictly.** What is there has to be a whole digest ending in a
//! newline; anything else is refused rather than read as *nothing kept*.

use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use alo_keeping_up::Digest;

/// Why a build could not be read from, kept in, or cleared from its file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Trouble {
    /// The file.
    pub(crate) path: String,
    /// What was wrong, or what the machine said.
    pub(crate) why: String,
}

/// The build kept at `path`, or [`None`] if nothing is kept there.
pub(crate) fn read(path: &Path) -> Result<Option<Digest>, Trouble> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(why) if why.kind() == ErrorKind::NotFound => return Ok(None),
        Err(why) => return Err(trouble(path, &why.to_string())),
    };
    let Some(line) = text.strip_suffix('\n') else {
        return Err(trouble(path, "it does not end in a newline"));
    };
    Digest::read(line)
        .map(Some)
        .map_err(|why| trouble(path, &format!("{why:?}")))
}

/// Keep `digest` at `path`, whole or not at all.
pub(crate) fn keep(path: &Path, digest: &Digest) -> Result<(), Trouble> {
    let beside = beside(path);
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&beside)
        .and_then(|mut file| {
            file.write_all(format!("{}\n", digest.as_str()).as_bytes())?;
            file.sync_all()
        })
        .and_then(|()| std::fs::rename(&beside, path))
        .and_then(|()| synced_folder(path))
        .map_err(|why| trouble(path, &why.to_string()))
}

/// Keep nothing at `path`; nothing kept already is not a failure.
pub(crate) fn clear(path: &Path) -> Result<(), Trouble> {
    match std::fs::remove_file(path) {
        Ok(()) => synced_folder(path).map_err(|why| trouble(path, &why.to_string())),
        Err(why) if why.kind() == ErrorKind::NotFound => Ok(()),
        Err(why) => Err(trouble(path, &why.to_string())),
    }
}

/// Sync the folder `path` is in, so a rename or removal survives power loss.
fn synced_folder(path: &Path) -> std::io::Result<()> {
    match path.parent() {
        Some(folder) if !folder.as_os_str().is_empty() => File::open(folder)?.sync_all(),
        _ => Ok(()),
    }
}

/// Where a new value is written before it replaces the old one.
fn beside(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".new");
    PathBuf::from(name)
}

/// The trouble with `path`, in words.
fn trouble(path: &Path, why: &str) -> Trouble {
    Trouble {
        path: path.display().to_string(),
        why: why.to_owned(),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn a_folder(named: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!(
            "alo-updating-one-build-{named}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    fn whole(pair: &str) -> Digest {
        Digest::read(&format!("sha256:{}", pair.repeat(32))).unwrap()
    }

    /// A build kept is read back, cleared is gone, and clearing twice is not a
    /// failure.
    #[test]
    fn a_build_kept_is_read_back_and_a_build_cleared_is_gone() {
        let folder = a_folder("round");
        let path = folder.join("build");
        assert_eq!(read(&path), Ok(None));
        keep(&path, &whole("aa")).unwrap();
        assert_eq!(read(&path), Ok(Some(whole("aa"))));
        keep(&path, &whole("bb")).unwrap();
        assert_eq!(read(&path), Ok(Some(whole("bb"))));
        clear(&path).unwrap();
        assert_eq!(read(&path), Ok(None));
        clear(&path).unwrap();
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// **Half a build, or a line with no end, is refused** rather than read as
    /// nothing kept.
    #[test]
    fn half_a_build_is_refused_and_not_read_as_nothing() {
        let folder = a_folder("half");
        let path = folder.join("build");
        std::fs::write(&path, "sha256:beef\n").unwrap();
        assert!(read(&path).is_err());
        std::fs::write(&path, whole("aa").as_str()).unwrap();
        assert!(read(&path).is_err());
        let _ = std::fs::remove_dir_all(&folder);
    }
}
