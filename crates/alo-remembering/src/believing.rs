//! A file on the disk that this machine believes: who may have written it,
//! and how it is replaced without ever being half a file.
//!
//! Two files a machine keeps say what its agent may do — the grants
//! (`keeping.rs`) and the pairings (`pairings.rs`) — and whoever can rewrite
//! either says what this machine's agent may reach or which machines may ask
//! it. So both are held to one rule, written once here, and neither file's
//! module names a mode or an owner of its own.
//!
//! # Who may write it
//!
//! Before a byte of it is parsed, and with exactly the accounts store's three
//! questions —
//!
//! - the path is **not a symbolic link** (`O_NOFOLLOW`);
//! - it belongs to **root or to the login reading it**, and to nobody else;
//! - **nobody else can write it** — group- or world-writable is refused.
//!
//! Asked of the open file rather than of the path, so the file that was checked
//! and the file that is read cannot be two different files.
//!
//! # Replaced whole or not at all
//!
//! [`replaced_whole`] writes a sibling file (`<name>.new`, mode `0600`), syncs
//! it and renames it over the real one. A machine that loses power mid-write
//! keeps the file it had — a torn file is refused whole by whichever reader
//! owns its shape — and a person who granted a folder or paired a machine this
//! morning would find nothing this afternoon with nothing able to say why.
//!
//! The folder is **not** created here, for `alo-accounts`' reason and
//! `alo-agentd`'s: `/var/lib/alo` is the image's, a missing one means this is
//! not an alo OS machine, and making one would turn a typo in a path into a
//! second list nobody is reading.

use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use crate::refusing::NotRemembered;

/// The mode bits that let the group or the world write.
const OTHERS_MAY_WRITE: u32 = 0o022;

/// The mode a file here is created with: the owner and nobody else.
pub(crate) const OURS_ALONE: u32 = 0o600;

/// The text at this path, believed and read.
///
/// # Errors
///
/// [`NotRemembered::NotThere`] when there is no file — told apart from every
/// failure; [`NotRemembered::ALink`] for a symbolic link;
/// [`NotRemembered::SomebodyElses`] and [`NotRemembered::WritableByOthers`]
/// for a file somebody else could have written; [`NotRemembered::NotRead`]
/// for everything else the machine said.
pub(crate) fn read_believed(at: &Path) -> Result<String, NotRemembered> {
    let mut file = match std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(o_nofollow())
        .open(at)
    {
        Ok(file) => file,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
            return Err(NotRemembered::NotThere { at: at.to_owned() });
        }
        // `ELOOP` is what `O_NOFOLLOW` answers a link with; the named
        // `ErrorKind` for it is not yet stable, so the number is compared.
        Err(why) if why.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error()) => {
            return Err(NotRemembered::ALink { at: at.to_owned() });
        }
        Err(why) => {
            return Err(NotRemembered::NotRead {
                at: at.to_owned(),
                why: why.to_string(),
            });
        }
    };
    let seen = file.metadata().map_err(|why| NotRemembered::NotRead {
        at: at.to_owned(),
        why: why.to_string(),
    })?;
    believed(at, seen.uid(), seen.mode(), us())?;
    let mut text = String::new();
    file.read_to_string(&mut text)
        .map_err(|why| NotRemembered::NotRead {
            at: at.to_owned(),
            why: why.to_string(),
        })?;
    Ok(text)
}

/// This text, written whole to this path, owner-only.
///
/// # Errors
///
/// [`NotRemembered::NotWritten`] naming what the machine said — including a
/// folder that is not there, which is refused rather than made.
pub(crate) fn replaced_whole(at: &Path, text: &str) -> Result<(), NotRemembered> {
    let fresh = a_sibling_of(at);
    // A stale sibling from a write that died is removed so `create_new` below
    // can insist the one being written is ours alone.
    if let Err(why) = std::fs::remove_file(&fresh)
        && why.kind() != std::io::ErrorKind::NotFound
    {
        return Err(NotRemembered::NotWritten {
            at: at.to_owned(),
            why: why.to_string(),
        });
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(OURS_ALONE)
        .custom_flags(o_nofollow())
        .open(&fresh)
        .map_err(|why| NotRemembered::NotWritten {
            at: at.to_owned(),
            why: why.to_string(),
        })?;
    file.write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|why| NotRemembered::NotWritten {
            at: at.to_owned(),
            why: why.to_string(),
        })?;
    drop(file);
    std::fs::rename(&fresh, at).map_err(|why| NotRemembered::NotWritten {
        at: at.to_owned(),
        why: why.to_string(),
    })
}

/// Whether a file with this owner and mode is one to believe, as a rule of its
/// own so every branch of it is testable without root.
fn believed(at: &Path, owner: u32, mode: u32, us: u32) -> Result<(), NotRemembered> {
    if owner != 0 && owner != us {
        return Err(NotRemembered::SomebodyElses {
            at: at.to_owned(),
            owner,
        });
    }
    if mode & OTHERS_MAY_WRITE != 0 {
        return Err(NotRemembered::WritableByOthers {
            at: at.to_owned(),
            mode: mode & 0o777,
        });
    }
    Ok(())
}

/// The user this process runs as, asked of the kernel rather than of an
/// environment.
fn us() -> u32 {
    rustix::process::geteuid().as_raw()
}

/// The flag that refuses to open a symbolic link, as `OpenOptions` takes it.
#[expect(
    clippy::cast_possible_wrap,
    reason = "O_NOFOLLOW is a flag bit pattern; the kernel reads it as bits either way"
)]
fn o_nofollow() -> i32 {
    rustix::fs::OFlags::NOFOLLOW.bits() as i32
}

/// The path a file is staged at before it replaces the real one.
fn a_sibling_of(at: &Path) -> PathBuf {
    let mut named = at.as_os_str().to_owned();
    named.push(".new");
    PathBuf::from(named)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The ownership rule, every branch** — a test cannot chown a file to
    /// somebody else without root, so the rule is a function and this walks it:
    /// root's file is believed, ours is believed, anybody else's is not, and a
    /// believable owner does not excuse a writable mode.
    #[test]
    fn only_roots_file_or_our_own_is_believed() {
        let at = Path::new(crate::THE_GRANTS);
        assert!(believed(at, 0, OURS_ALONE, 1000).is_ok());
        assert!(believed(at, 1000, OURS_ALONE, 1000).is_ok());
        assert!(matches!(
            believed(at, 1001, OURS_ALONE, 1000),
            Err(NotRemembered::SomebodyElses { owner: 1001, .. })
        ));
        assert!(matches!(
            believed(at, 0, 0o620, 1000),
            Err(NotRemembered::WritableByOthers { .. })
        ));
        assert!(matches!(
            believed(at, 1000, 0o602, 1000),
            Err(NotRemembered::WritableByOthers { .. })
        ));
    }
}
