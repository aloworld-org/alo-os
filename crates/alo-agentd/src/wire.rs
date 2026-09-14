//! The port presence advertises, bound; and the socket discovery is answered
//! on.
//!
//! Three libraries decide what happens on that port — `alo-nearby` for a
//! proposal and a confirmation, `alo-corridor` for a verb and an outcome, and
//! this crate's own [`crate::questioned`] for a question — and each of them
//! takes a listener somebody bound or a message somebody read. This is the
//! somebody: one `TcpListener` on [`THE_WIRE_PORT`], one datagram socket that
//! answers *who is here* with that port, and one identity kept at
//! [`THE_IDENTITY`] so that the machine is the same machine after a restart.
//!
//! # One reader, and why
//!
//! Both wires' `Receiving`s accept and read a message themselves, so one port
//! needs either a dispatcher that peeks the request line and hands the
//! connection on, or a way to hand each wire a message already read. It is the
//! second ([`Wire::accept_one`] reads; `alo_nearby::Arrived::carried` and
//! `alo_corridor::Arrived::carried` take): a peeked request line leaves the
//! rest of the request in whatever buffer did the peeking, and a socket-level
//! peek reads no further than the segment that has arrived — so the bytes
//! either go missing between the dispatcher and the door or have to be read
//! twice. One reader, through the framing both wires already share
//! (`alo_nearby::http`), reads every message once with the larger of the two
//! wires' bounds, and each wire still refuses a body over its own.
//!
//! # Nothing here is configurable
//!
//! The port is a constant and so is the identity's path, for task 1's reason:
//! no *advertise as*, no *discovery off*, no setting that would become the
//! trusted-network switch ADR 0003 forbids by the back door. Whoever runs alo
//! OS on one host for a test binds sockets of their own and hands them in
//! ([`Wire::on`]); the machine binds these.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::os::fd::{AsFd as _, BorrowedFd};

use alo_corridor::AT_MOST_A_VERB;
use alo_nearby::http::{self, WHILE_THE_WIRE_ANSWERS};
use alo_nearby::{Answering, MachineId, NotNearby, Presence, THE_ADDRESS, THE_PORT};

use crate::refusing::NotBound;

/// The port a machine answers proposals, verbs and questions on.
///
/// The number every example in this workspace has used since the first
/// discovery test, now the one a machine really binds. It is advertised, so
/// nothing about it is secret, and it is fixed, so nothing about it is a
/// setting.
pub const THE_WIRE_PORT: u16 = 7_610;

/// Where this machine keeps its identity, beside the record.
///
/// The folder `usr/lib/tmpfiles.d/alo.conf` makes for the record, which is
/// the person's to write in. `alo_nearby::MachineId::remembered_at` makes the
/// file the first time and refuses to replace one it cannot read, because a
/// new identity would be a new machine to everything this one had paired with.
pub const THE_IDENTITY: &str = "/var/lib/alo/machine-id";

/// The port bound, the socket discovery is answered on, and who this machine
/// says it is.
pub struct Wire {
    /// Where messages arrive.
    listener: TcpListener,
    /// The socket discovery questions arrive on, as something to wait on.
    ///
    /// A second handle onto the socket [`Answering`] owns: the two share one
    /// open file description, so waiting on this is waiting on that.
    discovery: UdpSocket,
    /// This machine, answering that it exists at the port above.
    answering: Answering,
    /// Where an asking machine's own discovery answers, for looking one up
    /// when its proposal arrives ([`crate::looking`]).
    asking_at: u16,
}

/// One connection, accepted and read, not yet told which wire it is for.
#[derive(Debug)]
pub struct Knocked {
    /// The connection, to write the reply on.
    pub stream: TcpStream,
    /// The address it came from, measured off the connection.
    pub from: IpAddr,
    /// What it carried, or why it could not be read as a message at all.
    pub message: Result<http::Message, NotNearby>,
}

impl Wire {
    /// Bind the port and the discovery socket on this machine.
    ///
    /// The listener on every interface at [`THE_WIRE_PORT`]; the discovery
    /// socket on every interface at `alo_nearby::THE_PORT`, shared with any
    /// other responder on the machine and joined to `alo_nearby::THE_ADDRESS`,
    /// which is what makes a question on the link reach it.
    ///
    /// # Errors
    ///
    /// [`NotBound::NoWire`] when either socket will not bind or the group will
    /// not join, and nothing is listening.
    pub fn bound(here: MachineId) -> Result<Self, NotBound> {
        let listener =
            TcpListener::bind((Ipv4Addr::UNSPECIFIED, THE_WIRE_PORT)).map_err(|why| {
                NotBound::NoWire {
                    what: "the port presence advertises",
                    why,
                }
            })?;
        let discovery =
            crate::unix::a_shared_datagram_socket_on(THE_PORT).map_err(|why| NotBound::NoWire {
                what: "the socket discovery is answered on",
                why,
            })?;
        discovery
            .join_multicast_v4(&THE_ADDRESS, &Ipv4Addr::UNSPECIFIED)
            .map_err(|why| NotBound::NoWire {
                what: "the discovery group",
                why,
            })?;
        Self::on(listener, discovery, here, THE_PORT)
    }

    /// This machine on sockets somebody else bound.
    ///
    /// For a test that puts two machines on one host, and for the reason both
    /// wires take a listener rather than making one. `asking_at` is the port
    /// an asking machine's own discovery answers at — [`THE_PORT`] on a real
    /// network, and whatever the other side of a test bound.
    ///
    /// # Errors
    ///
    /// [`NotBound::NoWire`] when the listener will not say where it is bound
    /// or the discovery socket cannot be waited on beside it.
    pub fn on(
        listener: TcpListener,
        discovery: UdpSocket,
        here: MachineId,
        asking_at: u16,
    ) -> Result<Self, NotBound> {
        let port = listener
            .local_addr()
            .map_err(|why| NotBound::NoWire {
                what: "the port presence advertises",
                why,
            })?
            .port();
        let waiting_on = discovery.try_clone().map_err(|why| NotBound::NoWire {
            what: "the socket discovery is answered on",
            why,
        })?;
        Ok(Self {
            listener,
            discovery: waiting_on,
            answering: Answering::on(discovery, Presence::of(here, port)),
            asking_at,
        })
    }

    /// This machine.
    #[must_use]
    pub const fn here(&self) -> &MachineId {
        self.answering.presence().machine()
    }

    /// The port this machine answers on, which is the one it advertises.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.answering.presence().port()
    }

    /// Where an asking machine's own discovery answers.
    #[must_use]
    pub const fn asking_at(&self) -> u16 {
        self.asking_at
    }

    /// What to wait on for a message on the port.
    #[must_use]
    pub fn waiting_on(&self) -> BorrowedFd<'_> {
        self.listener.as_fd()
    }

    /// What to wait on for a discovery question.
    #[must_use]
    pub fn discovery_waiting_on(&self) -> BorrowedFd<'_> {
        self.discovery.as_fd()
    }

    /// Accept one connection and read what it carries.
    ///
    /// Called once the listener has said somebody is there, so the accept
    /// answers at once; the read waits at most `WHILE_THE_WIRE_ANSWERS`, so a
    /// stranger holding a connection open does not hold this machine. What
    /// arrived but could not be read as a message is carried in
    /// [`Knocked::message`] for `crate::hearing` to answer with a word.
    ///
    /// # Errors
    ///
    /// [`NotNearby::TheNetwork`] when the listener will not accept or the
    /// connection will not take its timeouts — the machine's, and a reason
    /// to stop rather than spin.
    pub fn accept_one(&self) -> Result<Knocked, NotNearby> {
        let (stream, who) = self
            .listener
            .accept()
            .map_err(|why| NotNearby::TheNetwork(why.to_string()))?;
        stream
            .set_read_timeout(Some(WHILE_THE_WIRE_ANSWERS))
            .map_err(|why| NotNearby::TheNetwork(why.to_string()))?;
        stream
            .set_write_timeout(Some(WHILE_THE_WIRE_ANSWERS))
            .map_err(|why| NotNearby::TheNetwork(why.to_string()))?;
        // Every way the read fails is carried rather than answered here: a
        // stranger's bytes and a stranger who sent nothing for ten seconds
        // are both one connection, and neither is a reason to stop serving.
        let message = http::read_message_of_at_most(&stream, AT_MOST_A_VERB);
        Ok(Knocked {
            stream,
            from: who.ip(),
            message,
        })
    }

    /// Answer one discovery question, if what arrived was one.
    ///
    /// Called once the discovery socket has said something is there. The
    /// answer is this machine's identity and the port above, the same whether
    /// anything is paired or a turn is under way — presence never says what a
    /// machine is doing.
    ///
    /// # Errors
    ///
    /// As [`Answering::answer_one`].
    pub fn answer_discovery(&self) -> Result<Option<SocketAddr>, NotNearby> {
        self.answering.answer_one()
    }
}

impl std::fmt::Debug for Wire {
    /// Written by hand because [`Answering`] prints nothing of itself, and
    /// what a reader wants here is which machine this is and which port.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Wire")
            .field("here", self.here())
            .field("port", &self.port())
            .field("asking_at", &self.asking_at)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::io::Write as _;
    use std::net::{Ipv4Addr, TcpListener, TcpStream, UdpSocket};
    use std::time::Duration;

    use alo_nearby::{Looking, MachineId};

    use super::Wire;

    /// This machine, for these tests.
    fn here() -> MachineId {
        MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
    }

    /// A wire on sockets of this test's own, on this host.
    fn a_wire() -> Wire {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let discovery = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        Wire::on(listener, discovery, here(), 0).unwrap()
    }

    /// **Discovery is answered with the port the wire is bound to**, so what
    /// another machine dials is where this one listens.
    #[test]
    fn discovery_is_answered_with_the_port_the_wire_listens_on() {
        let wire = a_wire();
        let at = wire.discovery.local_addr().unwrap();
        let looking = Looking::from(UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap());
        looking.ask(at).unwrap();
        assert!(wire.answer_discovery().unwrap().is_some());
        let found = looking.found(Duration::from_secs(2)).unwrap();
        let one = found.iter().find(|one| one.machine == here()).unwrap();
        assert_eq!(one.port, wire.port());
        assert_eq!(one.where_it_answers().port(), wire.port());
    }

    /// **A message is read once, with where it came from**, and something
    /// that is not a message at all is carried as such rather than dropped.
    #[test]
    fn a_message_is_read_once_and_a_non_message_is_carried_as_one() {
        let wire = a_wire();
        let at = wire.listener.local_addr().unwrap();

        let mut client = TcpStream::connect(at).unwrap();
        client
            .write_all(alo_nearby::http::a_request("/somewhere", "h", "one line\n").as_bytes())
            .unwrap();
        let knocked = wire.accept_one().unwrap();
        assert!(knocked.from.is_loopback());
        let message = knocked.message.unwrap();
        assert_eq!(message.first, "POST /somewhere HTTP/1.1");
        assert_eq!(message.body, "one line\n");

        let mut stranger = TcpStream::connect(at).unwrap();
        stranger.write_all(b"\r\n").unwrap();
        stranger.shutdown(std::net::Shutdown::Write).unwrap();
        let knocked = wire.accept_one().unwrap();
        assert!(knocked.message.is_err(), "a non-message was read as one");
    }
}
