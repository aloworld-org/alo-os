//! Where a person's indexes are, and how that is worked out.
//!
//! The base directory specification puts data a program keeps for a person
//! under `$XDG_DATA_HOME`, and `$HOME/.local/share` when that says nothing.
//! An index is exactly that: not a setting, which is `$XDG_CONFIG_HOME` and
//! `alo-choosing`'s, and not a cache, because a cache is something a machine
//! may throw away unasked and an index of ten thousand documents is an
//! afternoon's reading. Per person, because a machine may have several
//! people on it and one person's documents are not another's.
//!
//! # Nothing here reads the environment
//!
//! The two variables arrive as arguments, the way `alo_choosing::place` takes
//! them, so that a session with `$XDG_DATA_HOME` set to a relative path, one
//! with neither variable, and a service started with an empty environment are
//! three tests rather than three things somebody has to arrange on a login.
//! The rule is the specification's: a relative `$XDG_DATA_HOME` is invalid
//! and ignored, and a session with no usable `$HOME` has nowhere for an index
//! to be, which is [`None`] rather than a guess.
//!
//! # One file per folder, named by the folder
//!
//! Each indexed folder is one file, named by a hash of the folder's path
//! rather than by the path itself, because a path spelled into a file name
//! runs past what a filesystem allows for a name long before a person's
//! folders do. The folder is written into the file's first line, so a reader
//! listing the directory can tell which is which by reading each head, and a
//! file whose head names another folder is refused rather than trusted. The
//! hash is FNV-1a, written out here in eight lines rather than rented,
//! because nothing about it is secret: it is a stable name, not a signature.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The variable that names where a person's data is kept.
pub const DATA_HOME: &str = "XDG_DATA_HOME";

/// The variable that names a person's home directory.
pub const HOME: &str = "HOME";

/// The directory alo OS keeps a person's data in.
pub const THE_FOLDER: &str = "alo";

/// The directory inside it that holds indexes, one file per folder.
pub const THE_INDEXES: &str = "finding";

/// What an index file's name ends in.
const THE_EXTENSION: &str = "index";

/// What `$HOME` is followed by when `$XDG_DATA_HOME` says nothing.
const DOT_LOCAL_SHARE: [&str; 2] = [".local", "share"];

/// Where the index of this folder is kept, given what the session says.
///
/// `data_home` is `$XDG_DATA_HOME` and `home` is `$HOME`, each as the process
/// really has it — unset arrives as [`None`]. [`None`] when neither is
/// usable, which is a login with no home directory.
pub(crate) fn where_it_is(
    data_home: Option<&OsStr>,
    home: Option<&OsStr>,
    folder: &Path,
) -> Option<PathBuf> {
    let under = |directory: &Path| {
        directory
            .join(THE_FOLDER)
            .join(THE_INDEXES)
            .join(named_for(folder))
    };
    if let Some(data) = data_home.map(Path::new)
        && data.has_root()
    {
        return Some(under(data));
    }
    match home.map(Path::new) {
        Some(home) if home.has_root() => {
            let mut share = home.to_path_buf();
            for part in DOT_LOCAL_SHARE {
                share.push(part);
            }
            Some(under(&share))
        }
        Some(_) | None => None,
    }
}

/// The file name for this folder's index.
fn named_for(folder: &Path) -> String {
    format!(
        "{:016x}.{THE_EXTENSION}",
        fnv1a(folder.as_os_str().as_encoded_bytes())
    )
}

/// FNV-1a over these bytes, 64 bits wide.
fn fnv1a(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    bytes.iter().fold(OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(PRIME)
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;

    /// `$XDG_DATA_HOME` wins when it is set and absolute; `$HOME/.local/share`
    /// otherwise; and neither usable is nowhere rather than somewhere.
    #[test]
    fn the_rule_is_the_specifications_including_the_part_that_surprises() {
        let documents = Path::new("/home/ada/Documents");
        let name = named_for(documents);
        assert_eq!(name.len(), 16 + 1 + THE_EXTENSION.len(), "{name}");

        let set = where_it_is(
            Some(OsStr::new("/data/ada")),
            Some(OsStr::new("/home/ada")),
            documents,
        )
        .unwrap();
        assert_eq!(set, Path::new("/data/ada/alo/finding").join(&name));

        let relative = where_it_is(
            Some(OsStr::new("data")),
            Some(OsStr::new("/home/ada")),
            documents,
        )
        .unwrap();
        assert_eq!(
            relative,
            Path::new("/home/ada/.local/share/alo/finding").join(&name)
        );

        let empty = where_it_is(
            Some(OsStr::new("")),
            Some(OsStr::new("/home/ada")),
            documents,
        );
        assert_eq!(empty, Some(relative));

        assert_eq!(where_it_is(None, None, documents), None);
        assert_eq!(where_it_is(None, Some(OsStr::new("ada")), documents), None);
    }

    /// Two folders are two files, and the same folder is the same file every
    /// time — checked against the FNV-1a test vectors, so the name is not
    /// merely consistent with itself.
    #[test]
    fn the_name_is_the_folders_and_the_hash_is_the_published_one() {
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a(b"foobar"), 0x8594_4171_f739_67e8);
        assert_eq!(
            named_for(Path::new("/home/ada/Documents")),
            named_for(Path::new("/home/ada/Documents"))
        );
        assert_ne!(
            named_for(Path::new("/home/ada/Documents")),
            named_for(Path::new("/home/ada/Pictures"))
        );
    }
}
