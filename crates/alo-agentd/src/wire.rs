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
//!
//! **What a workspace this machine hosts answers on is not a setting either.**
//! It is a fact about a server root installed, read once at start out of
//! [`crate::hosting`]'s root-owned file and handed in as a port
//! ([`Wire::hosting`]). It adds an answer to the question for workspaces under
//! this machine's own identity, and changes nothing this machine says about
//! itself.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::num::NonZeroU16;
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
        let mut wire = Self::on(listener, discovery, here, THE_PORT)?;
        wire.looks_at = SocketAddr::new(THE_ADDRESS.into(), THE_PORT);
        Ok(wire)
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
            looks_at: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), asking_at),
        })
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
        self.answering.presence().machine()
    }

    /// The port this machine answers on, which is the one it advertises.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.answering.presence().port()
    }

    /// The same machine, also answering discovery for the workspace it hosts
    /// at `workspace` — or for none, when there is none.
    ///
    /// A port and nothing else: the workspace is advertised under this
    /// machine's own identity, the one it was bound with
    /// (`alo_nearby::Answering::hosting_a_workspace_at`). Taken by value before
    /// the service is handed a borrow of the wire, so nothing the service does
    /// can change what is advertised; `src/main.rs` reads the port from
    /// [`crate::hosting`]'s file once, at start.
    #[must_use]
    pub fn hosting(self, workspace: Option<NonZeroU16>) -> Self {
        match workspace {
            Some(port) => Self {
                answering: self.answering.hosting_a_workspace_at(port),
                ..self
            },
            None => self,
        }
    }

    /// The port of the workspace this machine answers for, if it hosts one.
    #[must_use]
    pub fn hosts(&self) -> Option<u16> {
        self.answering
            .workspace()
            .map(alo_nearby::WorkspacePresence::port)
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
    /// answer to *who is here* is this machine's identity and the port above,
    /// the same whether anything is paired, a turn is under way or a workspace
    /// is hosted — presence never says what a machine is doing. The answer to
    /// *which workspaces are here* is the hosted workspace, and nothing on a
    /// machine hosting none.
    ///
    /// # Errors
    ///
    /// As [`Answering::answer_one`].
    pub fn answer_discovery(&self) -> Result<Option<SocketAddr>, NotNearby> {
        self.answering.answer_one()
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
    /// Written by hand because [`Answering`] prints nothing of itself, and
    /// what a reader wants here is which machine this is and which port.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Wire")
            .field("here", self.here())
            .field("port", &self.port())
            .field("asking_at", &self.asking_at)
            .field("hosts", &self.hosts())
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
