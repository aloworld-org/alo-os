//! A path on this machine as the `file://` address the file chooser portal
//! answers with, and a `current_folder` or `current_file` option read back
//! into a path.
//!
//! The specification answers `uris` as `file://` addresses and takes folders
//! and files as **bytes ending in a zero**, because a path on Linux is bytes.
//! This machine's paths are text (`alo_acting`'s line carries no other kind),
//! so bytes that are not text are not a path here and are read as none.

use std::path::{Path, PathBuf};

/// `path` as a `file://` address: every byte outside the unreserved set and
/// `/` written `%XX`.
#[must_use]
pub fn file_uri(path: &Path) -> String {
    let mut uri = String::from("file://");
    for byte in path.to_string_lossy().bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/') {
            uri.push(char::from(byte));
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
    uri
}

/// The bytes of a `current_folder` or `current_file` option as a whole path,
/// or [`None`] — for bytes that are not text, that do not begin at the root,
/// or that hold `.`, `..` or an empty name. One zero at the end is the
/// specification's and is not part of the path.
#[must_use]
pub fn a_path_from(bytes: &[u8]) -> Option<PathBuf> {
    let bytes = bytes.strip_suffix(&[0]).unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).ok()?;
    if text.contains('\0') || !text.starts_with('/') {
        return None;
    }
    if text == "/" {
        return Some(PathBuf::from(text));
    }
    let whole = text
        .strip_prefix('/')?
        .split('/')
        .all(|name| !name.is_empty() && name != "." && name != ".." && !name.chars().any(char::is_control));
    whole.then(|| PathBuf::from(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A space and a letter outside ASCII are written as their bytes.**
    #[test]
    fn a_path_is_written_as_an_address() {
        assert_eq!(
            file_uri(Path::new("/run/user/1000/doc/ab12/notes for März.txt")),
            "file:///run/user/1000/doc/ab12/notes%20for%20M%C3%A4rz.txt"
        );
    }

    /// **Only a whole path from the root is a path**, its one closing zero
    /// set aside.
    #[test]
    fn only_a_whole_path_is_read() {
        assert_eq!(
            a_path_from(b"/home/anna/Documents\0"),
            Some(PathBuf::from("/home/anna/Documents"))
        );
        assert_eq!(a_path_from(b"/\0"), Some(PathBuf::from("/")));
        for not in [
            &b"relative/folder\0"[..],
            b"/home/../etc\0",
            b"/home//anna\0",
            b"/home/anna/\0",
            b"/home/\xff\0",
            b"/home/a\0b\0",
            b"",
        ] {
            assert_eq!(a_path_from(not), None, "{not:?}");
        }
    }
}
