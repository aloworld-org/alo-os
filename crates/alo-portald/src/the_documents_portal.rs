//! **Handing a chosen file to one application**, through upstream's documents
//! portal on the person's own bus —
//! [ADR 0159](../../../docs/decisions/0159-a-chosen-file-reaches-an-application-through-the-documents-portal.md).
//!
//! `org.freedesktop.portal.Documents` is upstream's `xdg-document-portal`, a
//! pinned engine this image carries for this alone, with upstream's frontend
//! masked so `alo-portald` keeps `org.freedesktop.portal.Desktop`. It is asked
//! exactly as upstream's own file chooser asks it:
//!
//! | | |
//! |---|---|
//! | opening | `AddFull(ah fds, u flags, s app_id, as permissions) -> (as ids, a{sv} extra)`, the file held by an `O_PATH` descriptor |
//! | saving | `AddNamedFull(h folder, ay name, u flags, s app_id, as permissions) -> (s id, a{sv} extra)`, the folder held the same way, because the file may not exist yet |
//!
//! The flags are *reuse an export already made*, *keep it*, and *only where the
//! application's sandbox does not already reach it*. The permissions are
//! `read`, `write` when the application may write, and `grant-permissions` —
//! upstream's own set, so an application that hands the file on (to print it,
//! say) can. Saving adds `delete`, as upstream's does, because saving is
//! often writing a new file and moving it over the old one.
//!
//! The application opens the file at `<mountpoint>/<id>/<name>`. **An empty
//! id means its sandbox already reaches the file**, and it is handed the file's
//! own path.
//!
//! # Only the file it is given
//!
//! This is handed exactly one path, already chosen by the person and already
//! recorded (`alo_portals`' `the_person_chooses_a_file`). It never looks
//! for, widens or guesses at another.

use std::os::fd::AsFd as _;
use std::path::{Path, PathBuf};

use alo_portals::{ExportedFor, ExportsDocuments, NotExported};
use rustix::fs::{Mode, OFlags};
use zbus::zvariant::{Fd, OwnedValue};

/// The documents portal's name on the bus.
const DOCUMENTS: &str = "org.freedesktop.portal.Documents";
/// Its object.
const ITS_OBJECT: &str = "/org/freedesktop/portal/documents";

/// Reuse an export already made of the same file.
const REUSE_EXISTING: u32 = 1;
/// Keep the export after the application stops.
const PERSISTENT: u32 = 1 << 1;
/// Make no export where the application's sandbox already reaches the file.
const AS_NEEDED_BY_APP: u32 = 1 << 2;

/// The documents portal on the person's bus at `address`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheDocumentsPortal {
    /// The bus's address.
    address: String,
}

impl TheDocumentsPortal {
    /// The documents portal on the bus at `address`.
    #[must_use]
    pub fn on(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
        }
    }

    /// A connection to the bus, or why not.
    fn bus(&self) -> Result<zbus::blocking::Connection, NotExported> {
        zbus::blocking::connection::Builder::address(self.address.as_str())
            .and_then(zbus::blocking::connection::Builder::build)
            .map_err(|_| NotExported::NoDocumentsPortal)
    }
}

/// The permissions an application is given over a file it is handed.
#[must_use]
pub fn permissions(what: ExportedFor) -> Vec<&'static str> {
    match what {
        ExportedFor::Opening { writable: true } => vec!["read", "write", "grant-permissions"],
        ExportedFor::Opening { writable: false } => vec!["read", "grant-permissions"],
        ExportedFor::Saving => vec!["read", "write", "grant-permissions", "delete"],
    }
}

/// Where the application opens what was exported: `<mountpoint>/<id>/<name>`,
/// or `file` itself for an empty id.
#[must_use]
pub fn where_it_opens(mountpoint: &Path, id: &str, file: &Path) -> Option<PathBuf> {
    if id.is_empty() {
        return Some(file.to_path_buf());
    }
    if id.contains('/') || id == "." || id == ".." {
        return None;
    }
    Some(mountpoint.join(id).join(file.file_name()?))
}

/// `path` held by an `O_PATH` descriptor, which names it and cannot read it.
fn held(path: &Path) -> Result<std::os::fd::OwnedFd, NotExported> {
    rustix::fs::open(path, OFlags::PATH | OFlags::CLOEXEC, Mode::empty())
        .map_err(|_| NotExported::NotThere)
}

/// The mountpoint, as the portal says it: bytes ending in a zero.
fn the_mountpoint(bus: &zbus::blocking::Connection) -> Result<PathBuf, NotExported> {
    let reply = bus
        .call_method(
            Some(DOCUMENTS),
            ITS_OBJECT,
            Some(DOCUMENTS),
            "GetMountPoint",
            &(),
        )
        .map_err(|_| NotExported::NoDocumentsPortal)?;
    let bytes: Vec<u8> = reply
        .body()
        .deserialize()
        .map_err(|_| NotExported::Refused)?;
    let bytes = bytes.strip_suffix(&[0]).unwrap_or(&bytes);
    let text = std::str::from_utf8(bytes).map_err(|_| NotExported::Refused)?;
    if !text.starts_with('/') {
        return Err(NotExported::Refused);
    }
    Ok(PathBuf::from(text))
}

impl ExportsDocuments for TheDocumentsPortal {
    fn exported(
        &self,
        application: &str,
        file: &Path,
        what: ExportedFor,
    ) -> Result<PathBuf, NotExported> {
        let bus = self.bus()?;
        let mountpoint = the_mountpoint(&bus)?;
        let flags = REUSE_EXISTING | PERSISTENT | AS_NEEDED_BY_APP;
        let permissions = permissions(what);
        let id = match what {
            ExportedFor::Opening { .. } => {
                let fd = held(file)?;
                let reply = bus
                    .call_method(
                        Some(DOCUMENTS),
                        ITS_OBJECT,
                        Some(DOCUMENTS),
                        "AddFull",
                        &(vec![Fd::from(fd.as_fd())], flags, application, &permissions),
                    )
                    .map_err(|_| NotExported::Refused)?;
                let (ids, _extra): (Vec<String>, std::collections::HashMap<String, OwnedValue>) =
                    reply
                        .body()
                        .deserialize()
                        .map_err(|_| NotExported::Refused)?;
                // **Exactly one id for exactly one file**: anything else is
                // not an answer to what was asked.
                match <[String; 1]>::try_from(ids) {
                    Ok([id]) => id,
                    Err(_) => return Err(NotExported::Refused),
                }
            }
            ExportedFor::Saving => {
                let (Some(folder), Some(name)) = (file.parent(), file.file_name()) else {
                    return Err(NotExported::NotThere);
                };
                let fd = held(folder)?;
                let mut named = name.as_encoded_bytes().to_vec();
                named.push(0);
                let reply = bus
                    .call_method(
                        Some(DOCUMENTS),
                        ITS_OBJECT,
                        Some(DOCUMENTS),
                        "AddNamedFull",
                        &(
                            Fd::from(fd.as_fd()),
                            named,
                            flags,
                            application,
                            &permissions,
                        ),
                    )
                    .map_err(|_| NotExported::Refused)?;
                let (id, _extra): (String, std::collections::HashMap<String, OwnedValue>) = reply
                    .body()
                    .deserialize()
                    .map_err(|_| NotExported::Refused)?;
                id
            }
        };
        where_it_opens(&mountpoint, &id, file).ok_or(NotExported::Refused)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **An empty id is the file's own path; any other is under the
    /// mountpoint, by its own name — and an id that would lead elsewhere is
    /// not one.**
    #[test]
    fn where_an_exported_file_opens() {
        let mountpoint = Path::new("/run/user/1000/doc");
        let file = Path::new("/home/anna/the letter.txt");
        assert_eq!(
            where_it_opens(mountpoint, "", file),
            Some(file.to_path_buf())
        );
        assert_eq!(
            where_it_opens(mountpoint, "a1b2c3", file),
            Some(PathBuf::from("/run/user/1000/doc/a1b2c3/the letter.txt"))
        );
        for not in ["../../etc", "..", "."] {
            assert_eq!(where_it_opens(mountpoint, not, file), None, "{not}");
        }
    }

    /// **Upstream's own permissions**: writing only where it may, and
    /// deleting only to save.
    #[test]
    fn the_permissions_given() {
        assert_eq!(
            permissions(ExportedFor::Opening { writable: false }),
            ["read", "grant-permissions"]
        );
        assert!(permissions(ExportedFor::Opening { writable: true }).contains(&"write"));
        assert!(!permissions(ExportedFor::Opening { writable: true }).contains(&"delete"));
        assert!(permissions(ExportedFor::Saving).contains(&"delete"));
    }

    /// **With no documents portal on the bus, nothing is handed over**, and it
    /// says which.
    #[test]
    fn no_bus_is_no_documents_portal() {
        let nowhere = TheDocumentsPortal::on("unix:path=/nonexistent/alo-no-bus");
        assert_eq!(
            nowhere.exported(
                "org.gnome.TextEditor",
                Path::new("/etc/hostname"),
                ExportedFor::Opening { writable: false }
            ),
            Err(NotExported::NoDocumentsPortal)
        );
    }
}
