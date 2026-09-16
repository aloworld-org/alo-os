//! A path, as the file address an application is handed.
//!
//! `file://` and the path, every byte that is not unreserved or a `/`
//! percent-encoded (RFC 3986, RFC 8089). **Byte by byte, not character by
//! character**: a filename on Linux is bytes, and a name that is not UTF-8 is
//! still a file a person may have granted — rewriting it lossily would hand the
//! application an address for a different file from the one approved.

use std::fmt::Write as _;
use std::path::Path;

/// The file address of a full path, or [`None`] for one that is not full or
/// whose bytes this platform cannot read.
#[must_use]
pub fn of(path: &Path) -> Option<String> {
    let bytes = bytes_of(path)?;
    if bytes.first() != Some(&b'/') {
        return None;
    }
    let mut address = String::from("file://");
    for byte in bytes {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'.' | b'_' | b'~') {
            address.push(char::from(*byte));
        } else {
            // Writing to a `String` cannot fail.
            let _ = write!(address, "%{byte:02X}");
        }
    }
    Some(address)
}

/// A path's bytes, as the kernel holds them.
#[cfg(unix)]
fn bytes_of(path: &Path) -> Option<&[u8]> {
    use std::os::unix::ffi::OsStrExt as _;
    Some(path.as_os_str().as_bytes())
}

/// A path's bytes, where the platform keeps paths as something other than
/// bytes: only when it is text, and never a lossy rendering of it.
#[cfg(not(unix))]
fn bytes_of(path: &Path) -> Option<&[u8]> {
    path.to_str().map(str::as_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_becomes_one_address_with_everything_unsafe_encoded() {
        assert_eq!(
            of(Path::new("/home/anna/Notes/march.txt")).as_deref(),
            Some("file:///home/anna/Notes/march.txt")
        );
        assert_eq!(
            of(Path::new("/home/anna/Mes notes/été #1?.txt")).as_deref(),
            Some("file:///home/anna/Mes%20notes/%C3%A9t%C3%A9%20%231%3F.txt")
        );
    }

    #[test]
    fn a_path_that_is_not_full_has_no_address() {
        assert_eq!(of(Path::new("notes.txt")), None);
        assert_eq!(of(Path::new("")), None);
    }
}
