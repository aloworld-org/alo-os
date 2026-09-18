//! This machine's printing service, and the one door this crate reaches it by.
//!
//! The service is CUPS (ADR 0011: rented, configured, never patched). It finds
//! printers — on the local network through its DNS-SD backend, over USB through
//! its USB and IPP-over-USB backends — keeps the queues, and turns a document
//! into what a printer takes. What this crate adds is everything a person meets:
//! which printer, set up without a driver being chosen, and what is wrong with
//! it in a sentence.
//!
//! # Only on this machine
//!
//! [`PrintingService`] can be made for the service's socket, or for an address
//! **on this machine** and nowhere else. A printing service on another machine
//! would be a document leaving before anybody decided it could, and a queue
//! set up there would be a place a document can go that this machine does not
//! keep; [`NotThisMachine`] is what an address elsewhere becomes, before any
//! connection is opened.
//!
//! # Who may add a printer is the service's decision
//!
//! Reached over its socket, the service decides from the credentials of the
//! process at the other end, and adding a printer takes membership of its
//! administrative group. Nothing here asks for a password or carries one. An
//! answer saying no is [`Unanswered::NotPermitted`], which is a fact about how
//! this machine is configured and is said as one.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
#[cfg(unix)]
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use crate::http::{self, Failed};
use crate::ipp::Message;

/// Where the service listens on a machine built from the image.
pub const THE_SOCKET: &str = "/run/cups/cups.sock";

/// How long a connection may take to open.
const CONNECTING: Duration = Duration::from_secs(5);

/// How long the service may take to answer.
///
/// Longer than the ten seconds this crate asks the service to spend looking
/// for devices, so a search that is still going is never mistaken for silence.
const ANSWERING: Duration = Duration::from_secs(30);

/// Why the service gave no answer this crate can use.
#[derive(Debug)]
pub enum Unanswered {
    /// Nothing was listening, or the connection could not be opened.
    NotRunning,
    /// It was reached and stopped answering part way, or took too long.
    Silent,
    /// It answered with something that is not an answer to what was asked.
    NotUnderstood,
    /// It answered, and would not let this machine do what was asked.
    NotPermitted,
}

/// An address that is not on this machine, so no service there is this
/// machine's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotThisMachine;

/// How the service is reached.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Reaching {
    /// Its own socket.
    #[cfg(unix)]
    Socket(PathBuf),
    /// The broker asks CUPS to verify root through the socket peer credentials.
    #[cfg(unix)]
    BrokerSocket(PathBuf),
    /// An address on this machine.
    Address(SocketAddr),
}

/// This machine's printing service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintingService {
    /// How it is reached.
    reaching: Reaching,
}

impl PrintingService {
    /// The service at its own socket — how a machine built from the image
    /// reaches it.
    #[cfg(unix)]
    #[must_use]
    pub fn at_its_socket(socket: impl Into<PathBuf>) -> Self {
        Self {
            reaching: Reaching::Socket(socket.into()),
        }
    }

    /// The service as reached by the privileged broker.
    ///
    /// Sends the fixed PeerCred root authentication header over a Unix socket.
    /// CUPS checks the actual peer UID; this constructor grants no authority,
    /// takes no password, and is refused for a caller that is not root.
    #[cfg(unix)]
    #[must_use]
    pub fn for_the_broker(socket: impl Into<PathBuf>) -> Self {
        Self {
            reaching: Reaching::BrokerSocket(socket.into()),
        }
    }

    /// The service at an address, which must be this machine's own.
    ///
    /// # Errors
    /// [`NotThisMachine`] for an address anywhere else.
    pub fn at_this_machines_address(address: SocketAddr) -> Result<Self, NotThisMachine> {
        if !address.ip().is_loopback() {
            return Err(NotThisMachine);
        }
        Ok(Self {
            reaching: Reaching::Address(address),
        })
    }

    /// Send this message to `path` and read the answer to it.
    pub(crate) fn exchange(&self, path: &str, message: &Message) -> Result<Message, Unanswered> {
        self.exchange_carrying(path, message, &mut std::io::empty(), 0)
    }

    /// Send this message to `path` with `length` bytes of `document` after it,
    /// and read the answer to it.
    pub(crate) fn exchange_carrying<D: Read>(
        &self,
        path: &str,
        message: &Message,
        document: &mut D,
        length: u64,
    ) -> Result<Message, Unanswered> {
        let head = message.head().map_err(|_| Unanswered::NotUnderstood)?;
        let body = match &self.reaching {
            #[cfg(unix)]
            Reaching::Socket(socket) | Reaching::BrokerSocket(socket) => {
                let mut stream = std::os::unix::net::UnixStream::connect(socket)
                    .map_err(|_| Unanswered::NotRunning)?;
                stream
                    .set_read_timeout(Some(ANSWERING))
                    .and_then(|()| stream.set_write_timeout(Some(ANSWERING)))
                    .map_err(|_| Unanswered::NotRunning)?;
                over(
                    &mut stream,
                    path,
                    &head,
                    document,
                    length,
                    matches!(&self.reaching, Reaching::BrokerSocket(_)),
                )?
            }
            Reaching::Address(address) => {
                let mut stream = TcpStream::connect_timeout(address, CONNECTING)
                    .map_err(|_| Unanswered::NotRunning)?;
                stream
                    .set_read_timeout(Some(ANSWERING))
                    .and_then(|()| stream.set_write_timeout(Some(ANSWERING)))
                    .map_err(|_| Unanswered::NotRunning)?;
                over(&mut stream, path, &head, document, length, false)?
            }
        };
        let answer = Message::read(&body).map_err(|_| Unanswered::NotUnderstood)?;
        if answer.request_id() != message.request_id() {
            return Err(Unanswered::NotUnderstood);
        }
        Ok(answer)
    }
}

/// One exchange over an open stream, its failures as this file names them.
fn over<S: Read + Write, D: Read>(
    stream: &mut S,
    path: &str,
    head: &[u8],
    document: &mut D,
    length: u64,
    as_root: bool,
) -> Result<Vec<u8>, Unanswered> {
    http::exchange(stream, path, head, document, length, as_root).map_err(|failed| match failed {
        Failed::Io => Unanswered::Silent,
        Failed::NotPermitted => Unanswered::NotPermitted,
        Failed::NotUnderstood | Failed::Status | Failed::TooLarge => Unanswered::NotUnderstood,
    })
}

/// A request identifier no other request from this process has.
pub(crate) fn next_request() -> u32 {
    /// The last one handed out.
    static NEXT: AtomicU32 = AtomicU32::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    /// **A printing service elsewhere is not this machine's.** Refused as a
    /// value before any connection could be opened to it.
    #[test]
    fn a_service_on_another_machine_is_refused_before_anything_is_opened() {
        for elsewhere in [
            SocketAddr::from((Ipv4Addr::new(192, 168, 1, 20), 631)),
            SocketAddr::from((Ipv4Addr::UNSPECIFIED, 631)),
            SocketAddr::from((Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1), 631)),
        ] {
            assert_eq!(
                PrintingService::at_this_machines_address(elsewhere),
                Err(NotThisMachine),
                "{elsewhere}"
            );
        }
        assert!(
            PrintingService::at_this_machines_address(SocketAddr::from((Ipv4Addr::LOCALHOST, 631)))
                .is_ok()
        );
        assert!(
            PrintingService::at_this_machines_address(SocketAddr::from((Ipv6Addr::LOCALHOST, 631)))
                .is_ok()
        );
    }

    /// No two requests share an identifier, so an answer to one is never read
    /// as the answer to another.
    #[test]
    fn no_two_requests_share_an_identifier() {
        let first = next_request();
        let second = next_request();
        assert_ne!(first, second);
    }
}
