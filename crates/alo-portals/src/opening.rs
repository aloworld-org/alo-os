//! Asking the application that opens a file to open it, over the bus.
//!
//! The freedesktop *D-Bus activation* specification: an application whose
//! desktop entry says `DBusActivatable=true` owns its identifier as a bus name,
//! serves `org.freedesktop.Application` at its identifier written as a path,
//! and opens files it is sent with `Open(as uris, a{sv} platform_data)`. The bus
//! starts it if it is not running. That is the whole of how this backend opens
//! a file: **a message to an application, never a command run**. Nothing here
//! reads an `Exec` line, starts a process or passes anything to a shell.
//!
//! An application that is not activatable this way — no name on the bus and
//! nothing to start one — is not opened, and the request is answered with the
//! portal's refusal and recorded as `NotOpened`, naming it. Starting such an
//! application from its `Exec` line is the launcher's work, and not this
//! backend's.

use std::collections::HashMap;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use zbus::names::WellKnownName;
use zbus::zvariant::{ObjectPath, Value};

/// The interface an activatable application serves.
const AN_APPLICATION: &str = "org.freedesktop.Application";

/// Ask `opener` to open the file at `path`, and whether it said it did.
pub(crate) async fn opened_in(connection: &zbus::Connection, opener: &str, path: &Path) -> bool {
    let (Ok(name), Ok(object)) = (
        WellKnownName::try_from(opener),
        ObjectPath::try_from(object_of(opener)),
    ) else {
        return false;
    };
    let platform_data: HashMap<&str, Value<'_>> = HashMap::new();
    connection
        .call_method(
            Some(name),
            object,
            Some(AN_APPLICATION),
            "Open",
            &(vec![file_uri(path)], platform_data),
        )
        .await
        .is_ok()
}

/// The object an activatable application serves itself at: its identifier
/// with every `.` a `/` and every `-` a `_`.
fn object_of(identifier: &str) -> String {
    let path: String = identifier
        .chars()
        .map(|letter| match letter {
            '.' => '/',
            '-' => '_',
            other => other,
        })
        .collect();
    format!("/{path}")
}

/// A `file://` URI for `path`, with every byte that is not unreserved written
/// as `%XX` — so a space, a `#` or a `?` in a file name is part of the name
/// and not of the URI.
fn file_uri(path: &Path) -> String {
    let mut uri = String::from("file://");
    for byte in path.as_os_str().as_bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(byte) {
            uri.push(char::from(*byte));
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
    uri
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A path is written into a URI so that it means only itself.**
    #[test]
    fn a_path_becomes_a_uri_that_names_only_it() {
        assert_eq!(
            file_uri(Path::new("/home/anna/Invoices/march.pdf")),
            "file:///home/anna/Invoices/march.pdf"
        );
        assert_eq!(
            file_uri(Path::new("/home/anna/a b#c?d%é.pdf")),
            "file:///home/anna/a%20b%23c%3Fd%25%C3%A9.pdf"
        );
    }

    /// **The object is the one the activation specification names.**
    #[test]
    fn an_application_is_found_at_its_identifier_as_a_path() {
        assert_eq!(object_of("org.gnome.Papers"), "/org/gnome/Papers");
        assert_eq!(object_of("org.gnome.Text-Editor"), "/org/gnome/Text_Editor");
    }
}
