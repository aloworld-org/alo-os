//! Which application sent a message on the bus, as its sandbox says.
//!
//! Every interface the backend serves starts here, and so does every
//! `SettingChanged` it sends: the bus daemon says which process is behind a
//! connection (`GetConnectionCredentials`), read from the socket and never from
//! anything the caller said, and [`crate::Sandboxes`] reads which application that
//! process's sandbox names.
//!
//! # The process is held, never its number
//!
//! Between the bus answering and the sandbox being read, a process can end and
//! its number be given to another, whose sandbox would then be read in its
//! place. So the process is **held** by a descriptor ([`HeldProcess`]) before
//! anything is read, and what was read counts only while the process held is
//! still alive afterwards:
//!
//! - **A daemon that gives `ProcessFD`** hands over a descriptor it took from
//!   the socket, so the process held is the connection's by construction. The
//!   number beside it must agree.
//! - **A daemon that gives only `ProcessID`** — the reference `dbus-daemon`
//!   1.14 is one (`docs/quirks.md`) — hands over a number, and a descriptor is
//!   opened for it. That number could already have been reused before it was
//!   opened, so once the sandbox has been read the daemon is asked again, and
//!   the connection must still be there with the same number. A process that
//!   has ended has closed its end of the socket, so a daemon that has noticed
//!   no longer has the connection.
//!
//! A caller whose process could not be held, ended while it was read, or is no
//! longer the connection's is [`Sandboxed::Gone`], never named.

use std::os::fd::AsFd;

use alo_capability::Applicant;
use zbus::fdo;
use zbus::names::{BusName, UniqueName};

use crate::held_process::HeldProcess;
use crate::request::identified;
use crate::sandboxed::Sandboxed;
use crate::serving::Backend;

/// The application the sandbox of the process behind `connection` names.
///
/// A bus that will not answer, a process with no sandbox, and a process that
/// was gone by the time its sandbox was read are all nobody.
pub(crate) async fn application_of(
    bus: &zbus::Connection,
    connection: &UniqueName<'_>,
    backend: &Backend,
) -> Option<String> {
    caller_of(bus, connection, backend).await.into_application()
}

/// Which application the process behind `connection` is, saying apart a
/// caller nobody names and one whose process was gone before it was named.
pub(crate) async fn caller_of(
    bus: &zbus::Connection,
    connection: &UniqueName<'_>,
    backend: &Backend,
) -> Sandboxed {
    let Ok(daemon) = fdo::DBusProxy::new(bus).await else {
        return Sandboxed::Nobody;
    };
    let Ok(credentials) = daemon
        .get_connection_credentials(BusName::Unique(connection.as_ref()))
        .await
    else {
        return Sandboxed::Nobody;
    };
    let said = credentials.process_id();
    let given = credentials.process_fd().is_some();
    let held = match credentials.process_fd() {
        Some(descriptor) => descriptor
            .as_fd()
            .try_clone_to_owned()
            .ok()
            .and_then(|descriptor| HeldProcess::given(descriptor, said)),
        None => match said {
            Some(number) => HeldProcess::opened_for(number),
            None => return Sandboxed::Nobody,
        },
    };
    let Some(held) = held else {
        return Sandboxed::Gone;
    };
    let sandboxed = backend.sandboxes().application_of(&held);
    if sandboxed == Sandboxed::Gone || given {
        return sandboxed;
    }
    match daemon
        .get_connection_credentials(BusName::Unique(connection.as_ref()))
        .await
    {
        Ok(again) if again.process_id() == Some(held.number()) && held.is_still_alive() => {
            sandboxed
        }
        _ => Sandboxed::Gone,
    }
}

/// The applicant the record names, when the sandbox named one that is an
/// identifier.
pub(crate) fn named(application: Option<&str>) -> Option<Applicant> {
    application.and_then(|id| identified(id).ok())
}
