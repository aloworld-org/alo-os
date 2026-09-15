//! Which application sent a message on the bus, as its sandbox says.
//!
//! Every interface the backend serves starts here, and so does every
//! `SettingChanged` it sends: the bus daemon says which process is behind a
//! connection (`GetConnectionCredentials`), read from the socket and never from
//! anything the caller said, and [`crate::Sandboxes`] reads which application that
//! process's sandbox names.

use alo_capability::Applicant;
use zbus::fdo;
use zbus::names::{BusName, UniqueName};

use crate::request::identified;
use crate::serving::Backend;

/// The application the sandbox of the process behind `connection` names.
///
/// A bus that will not answer, or a process with no sandbox, is nobody.
pub(crate) async fn application_of(
    bus: &zbus::Connection,
    connection: &UniqueName<'_>,
    backend: &Backend,
) -> Option<String> {
    let daemon = fdo::DBusProxy::new(bus).await.ok()?;
    let credentials = daemon
        .get_connection_credentials(BusName::Unique(connection.as_ref()))
        .await
        .ok()?;
    backend
        .sandboxes()
        .application_of(credentials.process_id()?)
}

/// The applicant the record names, when the sandbox named one that is an
/// identifier.
pub(crate) fn named(application: Option<&str>) -> Option<Applicant> {
    application.and_then(|id| identified(id).ok())
}
