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
        Self::on(stream, Connection::session().map_err(io::Error::other)?)
    }

    /// Keep the authenticated bus explicit so the same checks run on a private bus.
    fn on(stream: &UnixStream, bus: Connection) -> io::Result<Self> {
        let peer = rustix::net::sockopt::socket_peercred(stream)?;
        if peer.uid != rustix::process::geteuid() {
            return Err(refused());
        }
        let process = rustix::process::pidfd_open(peer.pid, rustix::process::PidfdFlags::empty())?;
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
                Ok(daemon.get_name_owner(BusName::try_from("org.freedesktop.portal.Desktop")?)?)
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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "unexpected socket or bus failures fail these tests"
)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    /// A current public portal owner is admitted, and loses access with its name.
    #[test]
    fn capture_tracks_the_exact_public_service_incarnation() {
        let private =
            alo_keyring_fixture::AKeyringOfOurOwn::a_bus_with_no_keyring_on_it("capture-authority");
        let owner = zbus::blocking::connection::Builder::address(private.address().as_str())
            .unwrap()
            .name("org.freedesktop.portal.Desktop")
            .unwrap()
            .build()
            .unwrap();
        let (stream, _peer) = UnixStream::pair().unwrap();
        let authority = Authority::on(&stream, owner.clone()).unwrap();
        assert!(authority.current());
        owner
            .release_name("org.freedesktop.portal.Desktop")
            .unwrap();
        assert!(!authority.current());
    }

    /// Matching uid is insufficient: the socket peer must be the bus owner's PID.
    #[test]
    fn another_process_cannot_claim_the_portals_capture_connection() {
        let private =
            alo_keyring_fixture::AKeyringOfOurOwn::a_bus_with_no_keyring_on_it("capture-peer");
        let owner = zbus::blocking::connection::Builder::address(private.address().as_str())
            .unwrap()
            .name("org.freedesktop.portal.Desktop")
            .unwrap()
            .build()
            .unwrap();
        let folder = tempfile::tempdir().unwrap();
        let socket = folder.path().join("peer");
        let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "capture_authority::tests::socket_peer_child",
                "--nocapture",
            ])
            .env("ALO_CAPTURE_PEER_CHECK", &socket)
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let (mut stream, _) = listener.accept().unwrap();
        assert!(Authority::on(&stream, owner).is_err());
        stream.write_all(&[1]).unwrap();
        assert!(child.wait().unwrap().success());
    }

    /// Reexecuted test helper creates a socket with a genuinely different PID.
    #[test]
    fn socket_peer_child() {
        let Some(path) = std::env::var_os("ALO_CAPTURE_PEER_CHECK") else {
            return;
        };
        let mut socket = UnixStream::connect(path).unwrap();
        socket.read_exact(&mut [0]).unwrap();
    }
}
