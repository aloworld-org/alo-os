//! Which installed application holds a connection to the accessibility tree,
//! as its sandbox says.
//!
//! An application's root in the tree carries a name the application chose, and
//! any program can choose any name — so the name is never how the fallback
//! decides which application it is reading. What decides is the same thing
//! `alo-portals` decides a portal caller by: the bus daemon says which process
//! is behind the connection (`GetConnectionCredentials`, read from the socket),
//! the process is **held** by a descriptor before anything is read about it
//! (`alo_portals::HeldProcess`), and its sandbox names the application
//! (`alo_portals::Sandboxes`). What was read counts only while the process held
//! is still alive afterwards, and — for a daemon that gives a number and no
//! descriptor — while the daemon still says the connection is that process's.
//!
//! A program with no sandbox is nobody, and nobody is never a granted
//! application. That includes the desktop's own shell: nothing an agent is
//! granted names it, so no agent can read or press the approval it is waiting
//! on.
//!
//! The one message sent here is sent as a plain method call, **not through a
//! proxy**, because a proxy asks the bus to deliver signals to it — and this
//! crate asks for nothing to be delivered.

use std::os::fd::AsFd as _;

use alo_portals::{HeldProcess, Sandboxed, Sandboxes};
use zbus::blocking::Connection;
use zbus::fdo::ConnectionCredentials;

/// The application the sandbox of the process behind `holder` names, or
/// [`None`] for a program nothing names, or one gone before it was named.
pub(crate) fn application_of(
    bus: &Connection,
    holder: &str,
    sandboxes: &Sandboxes,
) -> Option<String> {
    let credentials = credentials_of(bus, holder)?;
    let said = credentials.process_id();
    let given = credentials.process_fd().is_some();
    let held = match credentials.process_fd() {
        Some(descriptor) => descriptor
            .as_fd()
            .try_clone_to_owned()
            .ok()
            .and_then(|descriptor| HeldProcess::given(descriptor, said)),
        None => HeldProcess::opened_for(said?),
    }?;
    let sandboxed = sandboxes.application_of(&held);
    if !given {
        let again = credentials_of(bus, holder)?;
        if again.process_id() != Some(held.number()) || !held.is_still_alive() {
            return None;
        }
    }
    match sandboxed {
        Sandboxed::Named(application) => Some(application),
        Sandboxed::Nobody | Sandboxed::Gone => None,
    }
}

/// What the bus daemon says about the process behind `holder`.
fn credentials_of(bus: &Connection, holder: &str) -> Option<ConnectionCredentials> {
    bus.call_method(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        Some("org.freedesktop.DBus"),
        "GetConnectionCredentials",
        &(holder,),
    )
    .ok()?
    .body()
    .deserialize::<ConnectionCredentials>()
    .ok()
}
