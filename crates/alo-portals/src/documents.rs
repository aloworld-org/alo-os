//! Materialise selected-file grants in the rented Documents portal, and revoke them.

use crate::{
    Unanswered,
    documents_file::{self, Export, Journal},
    serving::Backend,
};
use std::{
    collections::HashMap,
    os::unix::ffi::OsStrExt,
    path::Path,
    sync::{Mutex, PoisonError},
};
use zbus::zvariant::{Fd, OwnedValue};

/// The rented service that mounts an app-specific view inside Flatpak.
const DOCUMENTS: &str = "org.freedesktop.portal.Documents";
/// The object and interface have the same standard service lifetime.
const OBJECT: &str = "/org/freedesktop/portal/documents";
/// Export no folder, ability to delegate, or ability to delete another file.
const PERMISSIONS: [&str; 2] = ["read", "write"];

/// One frontend owns the bookkeeping; grants remain the authority.
#[derive(Default)]
pub(crate) struct Documents {
    /// Loaded only from the configured private file; a failed read stays a failure.
    journal: Mutex<Option<Journal>>,
}

impl Documents {
    /// Save a newly materialised permission before it is handed to the app.
    fn remember(&self, backend: &Backend, export: Export) -> Result<(), Unanswered> {
        let at = backend
            .machine()
            .document_permissions_file()
            .ok_or(Unanswered::DocumentUnavailable)?;
        let mut held = self.journal.lock().unwrap_or_else(PoisonError::into_inner);
        let before = match held.as_ref() {
            Some(journal) => journal.clone(),
            None => documents_file::read(&at).map_err(|_| Unanswered::DocumentUnavailable)?,
        };
        let mut after = before;
        if !after.exports.contains(&export) {
            after.exports.push(export);
        }
        documents_file::write(&at, &after).map_err(|_| Unanswered::DocumentUnavailable)?;
        *held = Some(after);
        Ok(())
    }

    /// Export exactly one chosen destination, using an O_PATH parent descriptor.
    pub(crate) async fn export(
        &self,
        bus: &zbus::Connection,
        backend: &Backend,
        path: &Path,
        grant: alo_capability::GrantId,
        request: &crate::dialog_request::NativeRequest,
    ) -> Result<(String, Export), Unanswered> {
        let unavailable = || Unanswered::DocumentUnavailable;
        let application = request.asked.application().ok_or_else(unavailable)?;
        if request.cancelled() {
            return Err(unavailable());
        }
        let parent = path.parent().ok_or_else(unavailable)?;
        let name = path.file_name().ok_or_else(unavailable)?;
        let parent_fd = rustix::fs::open(
            parent,
            rustix::fs::OFlags::PATH | rustix::fs::OFlags::DIRECTORY | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        )
        .map_err(|_| unavailable())?;
        let mut filename = name.as_bytes().to_vec();
        filename.push(0);
        let proxy = zbus::Proxy::new(bus, DOCUMENTS, OBJECT, DOCUMENTS)
            .await
            .map_err(|_| unavailable())?;
        // Reuse the exact document; omit persistence and as-needed flags so the
        // answer always names an isolated exported document, never a home path.
        let (document, _extra): (String, HashMap<String, OwnedValue>) = proxy
            .call(
                "AddNamedFull",
                &(
                    Fd::from(&parent_fd),
                    filename,
                    1_u32,
                    "",
                    Vec::<&str>::new(),
                ),
            )
            .await
            .map_err(|_| unavailable())?;
        if document.is_empty()
            || document.len() > 128
            || !document
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return Err(unavailable());
        }
        let export = Export {
            grant,
            application: application.to_owned(),
            document,
        };
        let result = async {
            let mut mount: Vec<u8> = proxy
                .call("GetMountPoint", &())
                .await
                .map_err(|_| unavailable())?;
            if mount.last() == Some(&0) {
                mount.pop();
            }
            let mount = Path::new(std::ffi::OsStr::from_bytes(&mount));
            if !mount.is_absolute() {
                return Err(unavailable());
            }
            let uri = url::Url::from_file_path(mount.join(&export.document).join(name))
                .map_err(|_| unavailable())?
                .into();
            self.remember(backend, export.clone())?;
            // The journal must exist before any sandbox permission does. A
            // crash while preparing an ungranted document exposes nothing.
            let still_granted = backend.machine().grants().is_some_and(|grants| {
                grants
                    .active_at(std::time::SystemTime::now())
                    .any(|held| held.id == grant)
            });
            if request.cancelled() || !still_granted {
                return Err(unavailable());
            }
            proxy
                .call::<_, _, ()>(
                    "GrantPermissions",
                    &(&export.document, application, PERMISSIONS.to_vec()),
                )
                .await
                .map_err(|_| unavailable())?;
            Ok(uri)
        }
        .await;
        match result {
            Ok(uri) => Ok((uri, export)),
            Err(why) => {
                self.withdraw(bus, backend, &export).await;
                Err(why)
            }
        }
    }

    /// Roll back one export without withdrawing another live selection's access.
    pub(crate) async fn withdraw(
        &self,
        bus: &zbus::Connection,
        backend: &Backend,
        export: &Export,
    ) -> bool {
        let snapshot = self
            .journal
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .map(|journal| journal.exports.clone())
            .unwrap_or_default();
        let grants = backend.machine().grants();
        let now = std::time::SystemTime::now();
        let shared = snapshot.iter().any(|other| {
            other != export
                && other.document == export.document
                && other.application == export.application
                && grants
                    .as_ref()
                    .is_some_and(|grants| grants.active_at(now).any(|held| held.id == other.grant))
        });
        if shared {
            true
        } else {
            Self::revoke(bus, export).await
        }
    }

    /// Withdraw a materialisation; failure stays journalled for the next sweep.
    pub(crate) async fn revoke(bus: &zbus::Connection, export: &Export) -> bool {
        let call = async {
            let Ok(proxy) = zbus::Proxy::new(bus, DOCUMENTS, OBJECT, DOCUMENTS).await else {
                return false;
            };
            proxy
                .call::<_, _, ()>(
                    "RevokePermissions",
                    &(&export.document, &export.application, PERMISSIONS.to_vec()),
                )
                .await
                .is_ok()
        };
        futures_lite::future::race(call, async {
            async_io::Timer::after(std::time::Duration::from_secs(2)).await;
            false
        })
        .await
    }

    /// Remove permissions whose exact grants no longer stand, including unread grants.
    pub(crate) async fn refresh(&self, bus: &zbus::Connection, backend: &Backend) {
        let Some(at) = backend.machine().document_permissions_file() else {
            return;
        };
        let snapshot = {
            let mut held = self.journal.lock().unwrap_or_else(PoisonError::into_inner);
            if held.is_none() {
                match documents_file::read(&at) {
                    Ok(journal) => *held = Some(journal),
                    Err(why) => {
                        eprintln!("alo-portald: document permissions could not be read: {why}");
                        return;
                    }
                }
            }
            held.as_ref()
                .map(|journal| journal.exports.clone())
                .unwrap_or_default()
        };
        let now = std::time::SystemTime::now();
        if snapshot.is_empty() {
            return;
        }
        let grants = backend.machine().grants();
        let active = |export: &Export| {
            grants.as_ref().is_some_and(|grants| {
                grants.active_at(now).any(|held| {
                    held.id == export.grant
                        && held.grant.grantee
                            == alo_capability::Applicant::named(&export.application).grantee()
                })
            })
        };
        for export in &snapshot {
            if backend.native_requests().stopped() {
                return;
            }
            if active(export) {
                continue;
            }
            let still_shared = snapshot.iter().any(|other| {
                other.document == export.document
                    && other.application == export.application
                    && active(other)
            });
            if !still_shared && !Self::revoke(bus, export).await {
                continue;
            }
            let mut held = self.journal.lock().unwrap_or_else(PoisonError::into_inner);
            let Some(journal) = held.as_mut() else {
                continue;
            };
            let mut after = journal.clone();
            after.exports.retain(|item| item != export);
            if documents_file::write(&at, &after).is_ok() {
                *journal = after;
            }
        }
    }
}
