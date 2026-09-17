//! The port presence advertises, bound; and the socket discovery is answered
//! on.
//!
//! Three libraries decide what happens on that port — `alo-nearby` for a
//! proposal and a confirmation, `alo-corridor` for a verb and an outcome, and
//! this crate's own [`crate::questioned`] for a question — and each of them
//! takes a listener somebody bound or a message somebody read. This is the
//! somebody: the listeners on [`THE_WIRE_PORT`] ([`crate::listeners`]), the
//! sockets that answer *who is here* with that port ([`crate::responding`]),
//! and one identity
//! kept at [`THE_IDENTITY`] so that the machine is the same machine after a
//! restart.
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
//!
//! **What a workspace this machine hosts answers on is not a setting either.**
//! It is a fact about a server root installed, read once at start out of
//! [`crate::hosting`]'s root-owned file and handed in as a port
//! ([`Wire::hosting`]). It adds an answer to the question for workspaces under
//! this machine's own identity, and changes nothing this machine says about
//! itself.
//!
//! # On every network the machine is on
//!
//! Discovery is answered on **one socket per network this machine is on, each
//! held to that network's interface** ([`crate::responding`]), joined to the
//! group on each interface that is up, carries multicast and has an IPv4 address
//! ([`crate::networks`]), and bound again when the kernel says a network
//! appeared. Which networks is not a setting either — it is what the machine is
//! plugged into — and what is said on each is the same bytes, because there is
//! one presence and every socket answers with it. **An answer leaves on the
//! network its question arrived on**, which an answer held to nothing did not: on
//! two networks handing out one private range it went by the route, to whoever
//! was at that address on the other one.
//!
//! # In both families
//!
//! A network nobody configured often has no IPv4 address on it, and every
//! interface on it still has an IPv6 link-local one. So the machine answers
//! discovery on a second socket, IPv6 only, joined at `ff02::fb` on every
//! interface with a link-local address — a responder holding **the same presence
//! and the same workspace** as the IPv4 ones, so the same bytes answer in both
//! families. It is held to no network, because a link-local address carries the
//! interface it was heard on and the kernel answers back out of that one. A kernel with no IPv6 in it is a line in the service log
//! and a machine discovered over IPv4 as before, never a machine that will not
//! start.
//!
//! # And the port is listened on once per network
//!
//! An unheld listener answers every handshake by the route, so on a machine on
//! two networks that hand out one private range only the machine on the route's
//! network could reach this one's port at all. So the port is **one IPv4
//! listener per IPv4 network, each held to that network's interface**, beside
//! one IPv6-only listener — which is `crate::listeners`', with the reasoning and
//! the kernel's measured behaviour. What this file takes from it is
//! [`Knocked::arrived`]: the listener that accepted says which network a
//! connection arrived on, and `crate::hearing` measures a proposal there.

use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::num::NonZeroU16;
use std::os::fd::BorrowedFd;

use alo_nearby::http::Message;
use alo_nearby::{HeardFrom, MachineId, NotNearby, Presence, THE_ADDRESS, THE_PORT};

use crate::arrived_on::ArrivedOn;
use crate::hosting::Hosted;
use crate::interfaces_that_went::Went;
use crate::joining::Joining;
use crate::listeners::{Listeners, Listening};
use crate::networks::Network;
use crate::refusing::NotBound;
use crate::responding::{Responders, Responding};
use crate::told_of_a_move::ToldOfAMove;
use crate::unhosted::Unhosted;
use crate::what_is_advertised::Advertising;

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
    /// Where messages arrive: one listener per network this machine is on, each
    /// held to that network's interface, and one IPv6-only listener beside them
    /// ([`crate::listeners`]).
    listeners: Listeners,
    /// Where discovery is answered: one socket per network this machine is on,
    /// each held to that network's interface, and one held to nothing over IPv6
    /// ([`crate::responding`]). An answer therefore leaves on the network its
    /// question arrived on.
    responders: Responders,
    /// The IPv6 discovery group, joined on every link-local network this machine
    /// is on, and the kernel's notification that a network changed — nothing on
    /// sockets a test handed in.
    joining: Option<Joining>,
    /// Why a workspace installed on this machine is not advertised, when the
    /// start refused its file — kept so the person can be told, and never
    /// found out again by reading the file.
    unhosted: Option<Unhosted>,
    /// Where an asking machine's own discovery answers, for looking one up
    /// when its proposal arrives ([`crate::looking`]).
    asking_at: u16,
    /// Where this machine asks who is here when its person proposes a
    /// pairing by identity: the multicast group on a real machine, and the
    /// socket the other side of a test bound.
    looks_at: SocketAddr,
}

/// One connection, accepted and read, not yet told which wire it is for.
#[derive(Debug)]
pub struct Knocked {
    /// The connection, to write the reply on.
    pub stream: TcpStream,
    /// The address it came from, measured off the connection — with the
    /// interface, for a link-local IPv6 address, and as IPv4 for an IPv4 peer
    /// an IPv6 listener reported as an IPv4-mapped address.
    pub from: HeardFrom,
    /// Which of this machine's networks it arrived on: the network of the
    /// listener that accepted it where that listener is held to one, and what
    /// the connection itself says where it is not (`crate::arrived_on`).
    ///
    /// What a measurement about the machine at the other end is held to, so
    /// that `192.168.1.20` on the cable is not measured against
    /// `192.168.1.20` on the Wi-Fi.
    pub arrived: ArrivedOn,
    /// What it carried, or why it could not be read as a message at all.
    pub message: Result<Message, NotNearby>,
}

impl Wire {
    /// Bind the port and the discovery socket on this machine.
    ///
    /// The listener on every interface at [`THE_WIRE_PORT`], in both families;
    /// the discovery socket on every interface at `alo_nearby::THE_PORT`, shared
    /// with any other responder on the machine and joined to
    /// `alo_nearby::THE_ADDRESS` **on every network this machine is on**
    /// ([`crate::joining`]), which is what makes a question on each of those
    /// links reach it; and a second discovery socket over IPv6, joined to
    /// `alo_nearby::THE_IPV6_ADDRESS` on every interface with a link-local
    /// address. A network that will not join is a line in the service log and
    /// the others are joined; a network that appears later is joined when the
    /// kernel says so ([`Wire::networks_changed`]); a machine that cannot listen
    /// over IPv6 at all is a line in the log and answers over IPv4.
    ///
    /// # Errors
    ///
    /// [`NotBound::NoWire`] when the port or the IPv4 discovery socket will not
    /// bind, and nothing is listening.
    pub fn bound(here: MachineId) -> Result<Self, NotBound> {
        let said = |line: &str| eprintln!("alo-agentd: {line}");
        let listeners = Listeners::bound(THE_WIRE_PORT, &mut |line| said(line))?;
        let presence = Presence::of(here, listeners.port());
        let responders = Responders::bound(THE_PORT, presence, &mut |line| said(line));
        let discovery_ipv6 = match crate::unix::a_shared_ipv6_datagram_socket_on(THE_PORT) {
            Ok(socket) => Some(socket),
            Err(why) => {
                said(&format!(
                    "discovery could not listen over IPv6 ({why}); this machine is found over IPv4 alone, and not on a network with no IPv4 address"
                ));
                None
            }
        };
        let joining = Joining::on_every_network(discovery_ipv6.as_ref(), &mut |line| said(line));
        let mut wire = Self::of(listeners, responders, THE_PORT);
        if let Some(socket) = discovery_ipv6 {
            wire = wire.answering_over_ipv6_on(socket)?;
        }
        wire.joining = Some(joining);
        wire.looks_at = SocketAddr::new(THE_ADDRESS.into(), THE_PORT);
        Ok(wire)
    }

    /// The same machine, also answering discovery over IPv6 on `socket` with the
    /// presence and the workspace it answers with over IPv4 — so what is said is
    /// the same bytes in both families.
    ///
    /// The socket is held to no network, because a question over IPv6 discovery
    /// comes from a link-local address that carries its own interface and the
    /// kernel answers it back out of that one (`crate::responding`).
    ///
    /// [`Wire::bound`] hands in the machine's own socket; a test on one host
    /// hands in one of its own, as it does to [`Wire::on`].
    ///
    /// # Errors
    ///
    /// [`NotBound::NoWire`] when the socket cannot be waited on beside the
    /// others.
    pub fn answering_over_ipv6_on(self, socket: UdpSocket) -> Result<Self, NotBound> {
        let responders =
            self.responders
                .also_answering_on(socket)
                .map_err(|why| NotBound::NoWire {
                    what: "the socket discovery is answered on over IPv6",
                    why,
                })?;
        Ok(Self { responders, ..self })
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
        let listeners = Listeners::on(listener)?;
        let presence = Presence::of(here, listeners.port());
        let responders = Responders::on(discovery, presence).map_err(|why| NotBound::NoWire {
            what: "the socket discovery is answered on",
            why,
        })?;
        Ok(Self::of(listeners, responders, asking_at))
    }

    /// This machine on `listeners` and `responders`, whether the machine bound
    /// them or a test handed them in.
    fn of(listeners: Listeners, responders: Responders, asking_at: u16) -> Self {
        Self {
            listeners,
            responders,
            joining: None,
            unhosted: None,
            asking_at,
            looks_at: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), asking_at),
        }
    }

    /// Where this machine asks who is here, when its person proposes a
    /// pairing by identity.
    #[must_use]
    pub const fn looks_at(&self) -> SocketAddr {
        self.looks_at
    }

    /// This machine.
    #[must_use]
    pub const fn here(&self) -> &MachineId {
        self.responders.presence().machine()
    }

    /// The port this machine answers on, which is the one it advertises.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.responders.presence().port()
    }

    /// The same machine, also answering discovery for the workspace it hosts
    /// at the port the start read — or for none, when there is none or its
    /// file was refused, which is kept so the person can be told why.
    ///
    /// A port and nothing else reaches the responder: the workspace is
    /// advertised under this machine's own identity, the one it was bound with
    /// (`alo_nearby::Answering::hosting_a_workspace_at`). Taken by value before
    /// the service is handed a borrow of the wire, so nothing the service does
    /// can change what is advertised; `src/main.rs` reads the file once, at
    /// start, through [`crate::hosting::advertised`].
    #[must_use]
    pub fn hosting(self, hosted: Hosted) -> Self {
        match hosted {
            Hosted::At(port) => {
                let responders = self
                    .responders
                    .hosting(port, &mut |line| eprintln!("alo-agentd: {line}"));
                Self { responders, ..self }
            }
            Hosted::Refused(why) => Self {
                unhosted: Some(why),
                ..self
            },
            Hosted::Nothing => self,
        }
    }

    /// What this machine advertises on the local network, as the running
    /// service holds it: its identity, the port its presence names, and the
    /// workspace it answers for, that it answers for none, or why one
    /// installed is not advertised.
    ///
    /// Read off what the responders answer with and what the start kept — no
    /// file is read, and nothing about what is advertised moves.
    #[must_use]
    pub fn advertising(&self) -> Advertising {
        let workspace = match (self.responders.workspace(), self.unhosted) {
            (Some(port), _) => Hosted::At(port),
            (None, Some(why)) => Hosted::Refused(why),
            (None, None) => Hosted::Nothing,
        };
        Advertising::of(self.here().clone(), self.port(), workspace)
    }

    /// The port of the workspace this machine answers for, if it hosts one.
    #[must_use]
    pub fn hosts(&self) -> Option<u16> {
        self.responders.workspace().map(NonZeroU16::get)
    }

    /// Where an asking machine's own discovery answers.
    #[must_use]
    pub const fn asking_at(&self) -> u16 {
        self.asking_at
    }

    /// The listeners as they stand, for one round of the service: what to wait
    /// on for a message on the port, and what to accept it from.
    ///
    /// Taken once a round rather than held, so a listener a network change took
    /// away closes when the round that was already waiting on it ends.
    #[must_use]
    pub fn listening(&self) -> Listening {
        Listening::of(&self.listeners)
    }

    /// The networks the port is listened on now — empty on a wire listening
    /// through a listener somebody handed in.
    #[must_use]
    pub fn listened_on(&self) -> Vec<Network> {
        self.listeners.listened_on()
    }

    /// The responders as they stand, for one round of answering discovery: what
    /// to wait on for a question, and what to answer it from.
    ///
    /// Taken once a round rather than held, for [`listening`](Self::listening)'s
    /// reason: a socket a network change took away closes when the round already
    /// waiting on it ends.
    #[must_use]
    pub fn responding(&self) -> Responding {
        Responding::of(&self.responders)
    }

    /// What the thread answering discovery waits on, and empties, for the
    /// responders having moved — nothing on a wire answering through a socket
    /// somebody handed in, whose responders never move.
    ///
    /// The service's own thread reads the kernel's notification and moves the
    /// responders ([`networks_changed`](Self::networks_changed)); this is how the
    /// other thread, asleep on the sockets it took before, hears of it
    /// (`crate::told_of_a_move`).
    #[must_use]
    pub const fn answering_moved(&self) -> Option<&ToldOfAMove> {
        self.responders.moved()
    }

    /// The networks discovery is answered on now — empty on a wire answering
    /// through a socket somebody handed in.
    #[must_use]
    pub fn answered_on(&self) -> Vec<Network> {
        self.responders.answered_on()
    }

    /// What to wait on for the kernel saying a program let go of the port —
    /// nothing on a listener a test handed in, or where the kernel would not
    /// say (`crate::told_of_a_port_let_go`).
    #[must_use]
    pub fn let_go_waiting_on(&self) -> Option<BorrowedFd<'_>> {
        self.listeners.let_go_waiting_on()
    }

    /// The kernel said a TCP socket at the port was destroyed: the port is
    /// tried again on every network it was refused on, with no network having
    /// changed (`crate::listeners`). Nothing discovery says moves.
    pub fn port_let_go_of(&self) {
        self.listeners
            .let_go_of(&mut |line| eprintln!("alo-agentd: {line}"));
    }

    /// What to wait on for the kernel saying one of this machine's networks
    /// appeared, changed or went — nothing on sockets a test handed in, or
    /// where the kernel would not say.
    #[must_use]
    pub fn networks_waiting_on(&self) -> Option<BorrowedFd<'_>> {
        self.joining.as_ref().and_then(Joining::waiting_on)
    }

    /// The kernel said a network changed: join discovery on every network
    /// this machine is on now that it is not yet joined on.
    ///
    /// What is advertised does not move — the same identity, port and
    /// workspace answer on every network — so this changes where this machine
    /// is heard and nothing it says. A failure is a line in the service log.
    pub fn networks_changed(&self) {
        let said = &mut |line: &str| eprintln!("alo-agentd: {line}");
        let went = self
            .joining
            .as_ref()
            .map_or_else(Went::nothing, |joining| joining.changed(said));
        // And where discovery is answered, on the same notification and for the
        // same reason: a machine plugged in after it started is otherwise never
        // found on the network it was plugged into (`crate::responding`). What
        // the kernel said went goes with it: an interface deleted and laid again
        // at its number reads the same as the one that went, and its membership
        // did not survive.
        self.responders.changed(&went, said);
        // And the port, for the same reason and on the same notification: a
        // laptop docked after it started is otherwise unreachable on the wired
        // network all day (`crate::listeners`).
        self.listeners.changed(said);
    }

    /// The networks discovery is joined on now, in the kernel's order — empty
    /// on sockets a test handed in.
    #[must_use]
    pub fn joined(&self) -> Vec<Network> {
        let mut joined = self.responders.joined();
        joined.extend(
            self.joining
                .as_ref()
                .map(Joining::joined)
                .unwrap_or_default(),
        );
        joined
    }

    /// Accept one connection on the listener bound first and read what it
    /// carries, for a wire on a listener somebody handed in.
    ///
    /// A machine listening on one per network accepts through
    /// [`listening`](Self::listening), which is told which of them spoke.
    ///
    /// # Errors
    ///
    /// As [`Listening::accept_one`].
    pub fn accept_one(&self) -> Result<Knocked, NotNearby> {
        self.listening().accept_the_first()
    }

    /// Answer one discovery question, if what arrived was one.
    ///
    /// Called once the discovery socket has said something is there. The
    /// answer to *who is here* is this machine's identity and the port above,
    /// the same whether anything is paired, a turn is under way or a workspace
    /// is hosted — presence never says what a machine is doing. The answer to
    /// *which workspaces are here* is the hosted workspace, and nothing on a
    /// machine hosting none.
    ///
    /// # Errors
    ///
    /// As [`Responding::answer_the_first_that_speaks`].
    pub fn answer_discovery(&self) -> Result<Option<SocketAddr>, NotNearby> {
        self.responding().answer_the_first_that_speaks()
    }
}

impl crate::looking::LookingFor for Wire {
    /// Asked on the link this wire is bound to, at the moment.
    fn look_for(&self, machine: &MachineId) -> Option<alo_nearby::Found> {
        crate::looking::found_by_name(machine, self.looks_at)
    }

    /// Asked on the same link, at the moment, for machines and workspaces.
    fn look_around(&self) -> alo_nearby::Around {
        crate::looking::around_at(self.looks_at)
    }
}

impl std::fmt::Debug for Wire {
    /// Written by hand because `alo_nearby::Answering` prints nothing of itself,
    /// and what a reader wants here is which machine this is and which port.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Wire")
            .field("here", self.here())
            .field("port", &self.port())
            .field("asking_at", &self.asking_at)
            .field("hosts", &self.hosts())
            .field("joined", &self.joined())
            .field("answered_on", &self.answered_on())
            .field("listened_on", &self.listened_on())
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

    /// A wire on sockets of this test's own, on this host, and where its
    /// listener is bound.
    fn a_wire() -> (Wire, std::net::SocketAddr) {
        let (wire, at, _) = a_wire_and_where_discovery_answers();
        (wire, at)
    }

    /// The same, and where its discovery answers.
    fn a_wire_and_where_discovery_answers() -> (Wire, std::net::SocketAddr, std::net::SocketAddr) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = listener.local_addr().unwrap();
        let discovery = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let answers_at = discovery.local_addr().unwrap();
        (
            Wire::on(listener, discovery, here(), 0).unwrap(),
            at,
            answers_at,
        )
    }

    /// **Discovery is answered with the port the wire is bound to**, so what
    /// another machine dials is where this one listens.
    #[test]
    fn discovery_is_answered_with_the_port_the_wire_listens_on() {
        let (wire, _, at) = a_wire_and_where_discovery_answers();
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
        let (wire, at) = a_wire();

        let mut client = TcpStream::connect(at).unwrap();
        client
            .write_all(alo_nearby::http::a_request("/somewhere", "h", "one line\n").as_bytes())
            .unwrap();
        let knocked = wire.accept_one().unwrap();
        assert!(knocked.from.ip().is_loopback());
        assert_eq!(
            knocked.arrived,
            crate::arrived_on::ArrivedOn::ItsOwnNetwork,
            "a connection on a listener held to nothing was not read off the connection"
        );
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

/// An alo machine that hosts a workspace, as `src/main.rs` makes one — its
/// file read, its wire told the port — found and opened by a second daemon
/// over loopback with task 18's request, and the two ways it hosts nothing.
#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod a_hosted_workspace {
    use std::fs::Permissions;
    use std::net::{Ipv4Addr, SocketAddr, TcpListener, UdpSocket};
    use std::num::NonZeroU16;
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
    use std::path::{Path, PathBuf};
    use std::thread::JoinHandle;
    use std::time::Duration;

    use alo_capability::Grants;
    use alo_protocol::ToAPerson;
    use alo_record::{Happened, Record};
    use alo_strings::Filling;

    use super::Wire;
    use crate::answering::what_a_person_said;
    use crate::corridor::Corridor;
    use crate::doing::what_an_agent_said;
    use crate::holding::Holding;
    use crate::hosting::{advertised, hosted_at};
    use crate::network::TheNetwork;
    use crate::pairing::Nearby;
    use crate::rereading::WhatIsGranted;
    use crate::testing::{
        NothingIsRemembered, a_directory_of_our_own, a_message, hour, in_english, noon,
        nothing_has_been_chosen, on_a_machine_that_answers, on_a_machine_with_no_turn, reception,
        the_studio,
    };
    use crate::words::NO_SUCH_WORKSPACE_ON_THE_NETWORK;

    /// The workspace file `text` would be, written at `mode` in a folder of
    /// this test's own — root's, because the loop runs these tests as root.
    fn the_workspace_file(what: &str, text: &str, mode: u32) -> PathBuf {
        let at = a_directory_of_our_own(what).join("workspace.toml");
        std::fs::write(&at, text).unwrap();
        std::fs::set_permissions(&at, Permissions::from_mode(mode)).unwrap();
        assert_eq!(
            std::fs::metadata(&at).unwrap().uid(),
            0,
            "these tests hold root's file and are run as root, as the loop runs them"
        );
        at
    }

    /// Where a workspace client would connect, which nothing may.
    fn a_workspace_listening() -> TcpListener {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        listener
    }

    /// The studio, as a daemon starts it: bound on this host, hosting what
    /// the file at `file` says — answering discovery on a thread until no
    /// question has come for three seconds, and counting what it answered.
    fn the_studio_hosting(file: &Path) -> (SocketAddr, Option<u16>, JoinHandle<u32>) {
        let discovery = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        discovery
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let at = discovery.local_addr().unwrap();
        let mut refused = None;
        let studio = Wire::on(
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
            discovery,
            the_studio(),
            0,
        )
        .unwrap()
        .hosting(advertised(file, |why| refused = Some(why.to_string())));
        if let Some(why) = refused {
            assert!(why.contains("no workspace is advertised"), "{why}");
        }
        let hosts = studio.hosts();
        let answering = std::thread::spawn(move || {
            let mut answered = 0_u32;
            while let Ok(who) = studio.answer_discovery() {
                answered = answered.saturating_add(u32::from(who.is_some()));
            }
            answered
        });
        (at, hosts, answering)
    }

    /// Reception, a second daemon on this host, whose wire looks for who is
    /// here where the studio answers.
    fn reception_looking_at(studio: SocketAddr) -> Wire {
        Wire::on(
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
            UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
            reception(),
            studio.port(),
        )
        .unwrap()
    }

    /// Reception's person sends `line` on reception's door, looked around
    /// through reception's own wire; the answer, and what was written down.
    fn receptions_person_says(
        line: &str,
        network: &TheNetwork,
        wire: &Wire,
    ) -> (ToAPerson, Record) {
        let mut record = Record::default();
        let said = on_a_machine_with_no_turn(
            "a-hosted-workspace",
            &mut record,
            |machine, _, strings, _, _| {
                let mut grants = Grants::default();
                what_a_person_said(
                    &a_message(line),
                    &mut Holding::Nobody(machine),
                    &mut WhatIsGranted::of(&mut grants, &NothingIsRemembered),
                    &Nearby {
                        network,
                        looking: wire,
                        advertising: &wire.advertising(),
                    },
                    strings,
                    noon(),
                )
                .unwrap()
            },
        );
        (said, record)
    }

    /// What reception's shell sends to open the studio's workspace.
    fn opening_the_studio() -> String {
        format!(
            r#"{{"open-workspace":{{"machine":"{}"}}}}"#,
            the_studio().as_str()
        )
    }

    /// **A machine advertising a workspace is found and opened by task 18's
    /// request from a second daemon, over loopback, end to end.** The
    /// studio's port comes off root's file; reception's person lists the
    /// workspaces and sees the studio's under the studio's own identity, at
    /// the address it answered from, then opens it by that identity — written
    /// down, with nothing connected to it and nothing paired.
    #[test]
    fn a_machine_hosting_a_workspace_is_found_and_opened_by_a_second_daemon_over_loopback() {
        let workspace = a_workspace_listening();
        let port = workspace.local_addr().unwrap().port();
        let file = the_workspace_file("hosting-end-to-end", &format!("port = {port}\n"), 0o644);
        let (at, hosts, answering) = the_studio_hosting(&file);
        assert_eq!(hosts, Some(port));

        let reception_wire = reception_looking_at(at);
        let network = TheNetwork::on(reception());

        let (listed, _) = receptions_person_says(r#"{"workspaces":{}}"#, &network, &reception_wire);
        let found = listed.workspaces_found().unwrap();
        assert_eq!(found.len(), 1, "{listed:?}");
        let one = found.first().unwrap();
        assert_eq!(one.machine(), the_studio().as_str());
        assert_eq!(one.answers_at(), format!("127.0.0.1:{port}"));

        let (said, record) =
            receptions_person_says(&opening_the_studio(), &network, &reception_wire);
        let opened = said.opened_workspace().unwrap();
        assert_eq!(opened.machine(), the_studio().as_str());
        assert_eq!(opened.answers_at(), format!("127.0.0.1:{port}"));
        assert_eq!(record.len(), 1, "{record:?}");
        assert!(matches!(
            record.everything().next().unwrap().happened(),
            Happened::WorkspaceOpened { workspace, answers_at }
                if workspace.is(the_studio().as_str())
                    && answers_at.as_str() == format!("127.0.0.1:{port}")
        ));

        // Two looks, each asking both questions, and every one answered.
        assert_eq!(answering.join().unwrap(), 4);
        assert_eq!(
            workspace.accept().map(|_| ()).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock,
            "something connected to the workspace"
        );
        assert!(
            network.locked().pairings().every().is_empty(),
            "hosting paired"
        );
    }

    /// **A machine with no workspace file, and a machine whose file is
    /// refused, host nothing**: each is still found as a machine, and opening
    /// a workspace by its identity is refused as no such workspace on the
    /// network.
    #[test]
    fn a_machine_with_no_workspace_file_or_a_refused_one_is_found_and_hosts_nothing() {
        let nowhere = a_directory_of_our_own("hosting-nothing").join("workspace.toml");
        let loose = the_workspace_file("hosting-loose", "port = 8443\n", 0o666);
        let strings = in_english();
        for file in [nowhere, loose] {
            let (at, hosts, answering) = the_studio_hosting(&file);
            assert_eq!(hosts, None, "{}", file.display());

            let reception_wire = reception_looking_at(at);
            let network = TheNetwork::on(reception());
            let (said, record) =
                receptions_person_says(&opening_the_studio(), &network, &reception_wire);
            let refusal = said.refusal().unwrap();
            assert_eq!(
                refusal.text(),
                strings
                    .say(&NO_SUCH_WORKSPACE_ON_THE_NETWORK.key(), &Filling::nothing())
                    .text()
            );
            assert!(record.is_empty());
            // One look, both questions: the machine's was answered, the
            // workspace question was stepped over.
            assert_eq!(answering.join().unwrap(), 1, "{}", file.display());
        }
    }

    /// **No request on either door writes, names or changes what is
    /// hosted.** Requests shaped as though one could — on the person's door
    /// and on the agent's — are refused; the file is byte for byte what it was
    /// and still says the same port.
    #[test]
    fn no_request_on_either_door_writes_names_or_changes_the_hosted_workspace() {
        let file = the_workspace_file("hosting-doors", "port = 8443\n", 0o644);
        let before = std::fs::read(&file).unwrap();
        let lines = [
            r#"{"host-workspace":{"port":9443}}"#.to_owned(),
            r#"{"workspace":{"port":9443}}"#.to_owned(),
            r#"{"hosting":{"file":"/etc/alo/workspace.toml","port":9443}}"#.to_owned(),
            format!(
                r#"{{"open-workspace":{{"machine":"{}","port":9443}}}}"#,
                the_studio().as_str()
            ),
        ];

        let quiet = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let wire = reception_looking_at(quiet.local_addr().unwrap());
        let network = TheNetwork::on(reception());
        for line in &lines {
            let (said, record) = receptions_person_says(line, &network, &wire);
            assert!(said.refusal().is_some(), "the person's door: {line}");
            assert!(record.is_empty(), "{line}");
        }

        let corridor = Corridor {
            network: &network,
            looking: &wire,
            naming: network.names(),
        };
        let mut questions = nothing_has_been_chosen();
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, grants, strings| {
            for line in &lines {
                let said = what_an_agent_said(
                    &a_message(line),
                    turning,
                    &mut questions,
                    Some(&corridor),
                    grants,
                    strings,
                    hour(),
                    noon(),
                );
                assert!(said.refusal().is_some(), "the agent's door: {line}");
            }
        });

        assert_eq!(std::fs::read(&file).unwrap(), before);
        assert_eq!(hosted_at(&file).unwrap().map(NonZeroU16::get), Some(8_443));
        assert_eq!(wire.hosts(), None);
    }
}
