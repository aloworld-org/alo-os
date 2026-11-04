//! Authenticate the portal's process before the private socket reveals sources.

use std::{io, os::fd::OwnedFd, os::unix::net::UnixStream};
use zbus::{
    blocking::{Connection, fdo::DBusProxy},
    names::{BusName, OwnedUniqueName},
};

/// A held process and its bus owner; neither a uid nor a claimed app ID suffices.
pub(super) struct Authority {
    /// The person's session bus, reached only on a capture worker thread.
    bus: Connection,
    /// This exact service incarnation; a restarted frontend must choose again.
    owner: OwnedUniqueName,
    /// Pins the kernel process identity against PID reuse.
    process: OwnedFd,
}

impl Authority {
    /// Admit only the process the bus authenticates as its public portal owner.
    pub(super) fn identify(stream: &UnixStream) -> io::Result<Self> {
        let peer = rustix::net::sockopt::socket_peercred(stream)?;
        if peer.uid != rustix::process::geteuid() {
            return Err(refused());
        }
        let process = rustix::process::pidfd_open(peer.pid, rustix::process::PidfdFlags::empty())?;
        let bus = Connection::session().map_err(io::Error::other)?;
        let daemon = DBusProxy::new(&bus).map_err(io::Error::other)?;
        let owner = daemon
            .get_name_owner(
                BusName::try_from("org.freedesktop.portal.Desktop").map_err(io::Error::other)?,
            )
            .map_err(io::Error::other)?;
        let pid = daemon
            .get_connection_unix_process_id(BusName::Unique(owner.as_ref()))
            .map_err(io::Error::other)?;
        if pid != peer.pid.as_raw_pid() as u32 {
            return Err(refused());
        }
        let authority = Self {
            bus,
            owner,
            process,
        };
        authority.current().then_some(authority).ok_or_else(refused)
    }

    /// A process exit or public service replacement ends all of its frame feeds.
    pub(super) fn current(&self) -> bool {
        let mut processes = [rustix::event::PollFd::new(
            &self.process,
            rustix::event::PollFlags::IN,
        )];
        let timeout = rustix::event::Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        if rustix::event::poll(&mut processes, Some(&timeout)) != Ok(0) {
            return false;
        }
        DBusProxy::new(&self.bus)
            .and_then(|daemon| {
                daemon.get_name_owner(BusName::try_from("org.freedesktop.portal.Desktop")?)
            })
            .is_ok_and(|owner| owner == self.owner)
    }
}

/// A deliberately uninformative refusal to an unauthorised local process.
fn refused() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "capture requires the current portal service",
    )
}
