//! A question to a machine found on one network, dialled from a socket held to
//! that network's interface.
//!
//! [ADR 0042](../../../docs/decisions/0042-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md).
//! `192.168.1.20` on the wired network and `192.168.1.20` on the Wi-Fi are two
//! machines whenever two routers hand out the same private range, and a socket
//! that is not held to an interface leaves by whatever the route says at the
//! moment it connects. So a question to a paired machine found on one network
//! connects from a socket **held to that network's interface**
//! (`SO_BINDTOIFINDEX`): the kernel sends it out of that interface or not at
//! all, and the boundary around the turn — which was shown the machine on that
//! interface — refuses the same address on any other.
//!
//! The client's own connector is kept for everything else, and deliberately
//! replaced only here: the corridor is plain HTTP on a machine's own port, so
//! there is no proxy and no TLS for this to skip, and a provider's question —
//! whose address is on no one network — never comes this way.
//!
//! A socket is held before it connects and never after: once held, the kernel
//! will not let a process without `CAP_NET_RAW` hold it anywhere else, and
//! `alo-agentd` has no capabilities at all (ADR 0018).

use std::io::{self, Read as _, Write as _};
use std::net::{SocketAddr, TcpStream};
use std::num::NonZeroU32;
use std::time::Duration;

use socket2::{Domain, SockAddr, Socket, Type};
use ureq::unversioned::transport::{
    Buffers, ConnectionDetails, Connector, LazyBuffers, NextTimeout, Transport,
};

/// The longest a connection to a machine on the same network is waited for,
/// when the request's own budget does not say sooner.
///
/// A machine on the link answers a connection in milliseconds or not at all;
/// the question's own wait, which is minutes, is for the model to think.
const WHILE_CONNECTING: Duration = Duration::from_secs(10);

/// A connector that connects only from a socket held to one interface.
#[derive(Debug, Clone, Copy)]
pub(crate) struct HeldTo {
    /// The interface, as the kernel numbers it.
    interface: NonZeroU32,
}

impl HeldTo {
    /// Connections held to the interface the kernel numbers `interface`.
    pub(crate) const fn the_interface(interface: NonZeroU32) -> Self {
        Self { interface }
    }
}

impl Connector for HeldTo {
    type Out = HeldConnection;

    fn connect(
        &self,
        details: &ConnectionDetails,
        // Nothing comes before this in its chain, so there is nothing to pass on.
        _chained: Option<()>,
    ) -> Result<Option<Self::Out>, ureq::Error> {
        let within = details
            .timeout
            .not_zero()
            .map_or(WHILE_CONNECTING, |left| (*left).min(WHILE_CONNECTING));
        let mut refused = None;
        for address in details.addrs.iter() {
            match connected_from(self.interface, *address, within) {
                Ok(stream) => {
                    let buffers = LazyBuffers::new(
                        details.config.input_buffer_size(),
                        details.config.output_buffer_size(),
                    );
                    return Ok(Some(HeldConnection { stream, buffers }));
                }
                Err(why) if why.kind() == io::ErrorKind::TimedOut => {
                    return Err(ureq::Error::Timeout(ureq::Timeout::Connect));
                }
                Err(why) => refused = Some(why),
            }
        }
        Err(ureq::Error::Io(refused.unwrap_or_else(|| {
            io::Error::new(io::ErrorKind::ConnectionRefused, "nowhere was registered")
        })))
    }
}

/// One connection made from a socket held to an interface, as the client reads
/// and writes it.
///
/// The client's own TCP transport is private to it, so this is the few lines of
/// it a request needs: its buffers, a write, a read, and whether the
/// connection is still there. Nothing is pooled — `crate::openai` makes one
/// client per request and drops it.
#[derive(Debug)]
pub(crate) struct HeldConnection {
    /// The connection.
    stream: TcpStream,
    /// What has been read and what is to be written.
    buffers: LazyBuffers,
}

impl Transport for HeldConnection {
    fn buffers(&mut self) -> &mut dyn Buffers {
        &mut self.buffers
    }

    fn transmit_output(&mut self, amount: usize, timeout: NextTimeout) -> Result<(), ureq::Error> {
        self.stream
            .set_write_timeout(timeout.not_zero().map(|left| *left))?;
        let output = self
            .buffers
            .output()
            .get(..amount)
            .ok_or_else(|| io::Error::other("the client asked for more than it buffered"))?;
        self.stream
            .write_all(output)
            .map_err(|why| timed_out_or(why, timeout))
    }

    fn await_input(&mut self, timeout: NextTimeout) -> Result<bool, ureq::Error> {
        self.stream
            .set_read_timeout(timeout.not_zero().map(|left| *left))?;
        let read = self
            .stream
            .read(self.buffers.input_append_buf())
            .map_err(|why| timed_out_or(why, timeout))?;
        self.buffers.input_appended(read);
        Ok(read > 0)
    }

    fn is_open(&mut self) -> bool {
        // Never reused: one request, one connection (`crate::openai`).
        false
    }
}

/// A read or a write that ran out of time is the client's timeout, and
/// anything else is what the kernel said.
fn timed_out_or(why: io::Error, timeout: NextTimeout) -> ureq::Error {
    match why.kind() {
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => ureq::Error::Timeout(timeout.reason),
        _ => ureq::Error::Io(why),
    }
}

/// A connection to `to` from a socket held to `interface`, made within `within`.
///
/// # Errors
/// What the kernel answered: `EACCES` from a boundary that was not shown this
/// address on this interface, and everything a connection can fail with.
fn connected_from(
    interface: NonZeroU32,
    to: SocketAddr,
    within: Duration,
) -> io::Result<TcpStream> {
    let socket = Socket::new(Domain::for_address(to), Type::STREAM, None)?;
    held(&socket, interface, to)?;
    socket.connect_timeout(&SockAddr::from(to), within)?;
    Ok(socket.into())
}

/// Hold `socket` to `interface`, before it connects to `to`.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn held(socket: &Socket, interface: NonZeroU32, to: SocketAddr) -> io::Result<()> {
    match to {
        SocketAddr::V4(_) => socket.bind_device_by_index_v4(Some(interface)),
        SocketAddr::V6(_) => socket.bind_device_by_index_v6(Some(interface)),
    }
}

/// Hold `socket` to `interface` — which a kernel alo OS does not run on is not
/// asked to do, so the question is not put rather than put by the route.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
fn held(_socket: &Socket, _interface: NonZeroU32, _to: SocketAddr) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "a socket can be held to an interface only on Linux",
    ))
}

#[cfg(all(test, target_os = "linux"))]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::io::Read as _;
    use std::net::{Ipv4Addr, TcpListener};
    use std::num::NonZeroU32;
    use std::time::Duration;

    use super::connected_from;

    /// **A connection held to the interface it is on is made**, and the socket
    /// the far end sees is really held there: loopback is interface one on every
    /// Linux network namespace.
    #[test]
    fn a_connection_held_to_its_interface_is_made() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = listener.local_addr().unwrap();
        let loopback = NonZeroU32::new(1).unwrap();
        let stream = connected_from(loopback, at, Duration::from_secs(2)).unwrap();
        assert_eq!(stream.peer_addr().unwrap(), at);
        let (mut accepted, _) = listener.accept().unwrap();
        drop(stream);
        let mut nothing = Vec::new();
        assert_eq!(accepted.read_to_end(&mut nothing).unwrap(), 0);
    }

    /// **A socket held to an interface that is not there connects nowhere** —
    /// the kernel refuses the hold, and nothing is dialled by the route instead.
    #[test]
    fn a_socket_held_to_an_interface_that_is_not_there_connects_nowhere() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = listener.local_addr().unwrap();
        let nowhere = NonZeroU32::new(u32::MAX / 2).unwrap();
        let refused = connected_from(nowhere, at, Duration::from_secs(2));
        assert!(refused.is_err(), "{refused:?}");
        listener.set_nonblocking(true).unwrap();
        assert!(listener.accept().is_err(), "a refused hold still connected");
    }
}
