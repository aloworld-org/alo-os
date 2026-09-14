//! This machine, answering when somebody on the link asks who is here — and,
//! where it hosts a workspace, which workspaces are.
//!
//! [`Answering`] holds this machine's [`Presence`] and replies to the question
//! for [`SERVICE`](crate::SERVICE) with it. It was the only thing it answered
//! until a workspace could be served on an alo machine: task 17's contract
//! says such a workspace is advertised under **that machine's own identity**,
//! and the service that holds the identity is the one that answers for it. A
//! second responder racing this one for the same socket would have to invent
//! the identity for itself, which is a machine saying something about itself
//! that the service holding its identity did not say.
//!
//! # What hosting a workspace changes, and what it does not
//!
//! [`Answering::hosting_a_workspace_at`] takes **a port and nothing else**.
//! The identity in the workspace answer is the one in the presence this
//! responder was made with, so there is no road by which another identity
//! reaches it:
//!
//! ```compile_fail
//! use alo_nearby::{Answering, MachineId, Presence, WorkspacePresence};
//! use std::num::NonZeroU16;
//! let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
//! let here = MachineId::made().unwrap();
//! let somebody_else = MachineId::made().unwrap();
//! // A workspace presence under another identity is not something a responder
//! // can be handed: hosting takes a port.
//! let _ = Answering::on(socket, Presence::of(here, 7_610))
//!     .hosting_a_workspace_at(WorkspacePresence::of(somebody_else, 8_443));
//! ```
//!
//! What it does not change is **anything this machine says about itself**. The
//! answer to the question for machines is the same bytes whether or not a
//! workspace is hosted — presence that changed shape when a server was
//! installed would tell a network watching it that one had been — and the
//! question for workspaces is stepped over, exactly as a printer's is, by a
//! machine that hosts none. Hosting grants nothing and pairs nothing: this is
//! two more records on a datagram, and nothing here connects to anything.

use std::net::{SocketAddr, UdpSocket};
use std::num::NonZeroU16;

use crate::advertising::{about, about_a_workspace};
use crate::presence::Presence;
use crate::reading::{a_question_for_workspaces_in, a_question_in};
use crate::refusing::{NotNearby, because};
use crate::workspace::WorkspacePresence;

/// The most bytes one datagram is read into.
///
/// A question from this crate is under fifty; a stranger's may be anything,
/// and what does not fit is what is not read.
const AT_MOST: usize = 1_500;

/// This machine, answering when somebody asks who is here.
pub struct Answering {
    /// The socket questions arrive on and answers go out of.
    socket: UdpSocket,
    /// What this machine says about itself, which does not change.
    presence: Presence,
    /// The workspace this machine hosts, under its own identity, if it hosts
    /// one.
    workspace: Option<WorkspacePresence>,
}

impl Answering {
    /// This machine on a socket somebody else opened, hosting no workspace.
    ///
    /// The socket is taken rather than made so that a test can put both sides
    /// of the road on one host, and so that whoever runs alo OS decides which
    /// interfaces it speaks on rather than this crate deciding for them.
    #[must_use]
    pub const fn on(socket: UdpSocket, presence: Presence) -> Self {
        Self {
            socket,
            presence,
            workspace: None,
        }
    }

    /// The same machine, also answering that it hosts a workspace at `port`.
    ///
    /// The workspace is advertised under this machine's own identity — the
    /// one in the presence it was made with — because a port is all this
    /// takes. Zero is not a port a workspace answers on, and the type says so.
    #[must_use]
    pub fn hosting_a_workspace_at(self, port: NonZeroU16) -> Self {
        let workspace = WorkspacePresence::of(self.presence.machine().clone(), port.get());
        Self {
            workspace: Some(workspace),
            ..self
        }
    }

    /// What this machine says about itself, for a caller that wants to see it
    /// without waiting for anybody to ask.
    #[must_use]
    pub const fn presence(&self) -> &Presence {
        &self.presence
    }

    /// The workspace this machine says it hosts, if it says it hosts one.
    #[must_use]
    pub const fn workspace(&self) -> Option<&WorkspacePresence> {
        self.workspace.as_ref()
    }

    /// Wait for one packet and answer whatever it asked that this machine
    /// answers.
    ///
    /// A question for machines is answered with this machine's presence; a
    /// question for workspaces with the hosted workspace, and only where there
    /// is one; a packet asking both gets both, one answer each. Returns who was
    /// answered, or nothing at all if nothing was — a printer, a media player,
    /// this machine's own answer coming back round, or a question for
    /// workspaces on a machine hosting none. None of those is an error.
    ///
    /// # Errors
    ///
    /// [`NotNearby::TheNetwork`] if the socket will not read or will not send,
    /// which includes the timeout a caller set on it: a caller that wants to
    /// stop waiting sets one and gets it back here.
    pub fn answer_one(&self) -> Result<Option<SocketAddr>, NotNearby> {
        let mut heard = [0_u8; AT_MOST];
        let (how_many, who) = self
            .socket
            .recv_from(&mut heard)
            .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
        let asked = heard.get(..how_many).ok_or(NotNearby::CutShort)?;
        let mut answered = None;
        if a_question_in(asked) {
            self.send(&about(&self.presence)?, who)?;
            answered = Some(who);
        }
        if let Some(workspace) = &self.workspace
            && a_question_for_workspaces_in(asked)
        {
            self.send(&about_a_workspace(workspace)?, who)?;
            answered = Some(who);
        }
        Ok(answered)
    }

    /// One answer, to whoever asked.
    fn send(&self, answer: &[u8], to: SocketAddr) -> Result<(), NotNearby> {
        self.socket
            .send_to(answer, to)
            .map(|_| ())
            .map_err(|why| NotNearby::TheNetwork(because(&why)))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::net::UdpSocket;
    use std::num::NonZeroU16;
    use std::time::Duration;

    use super::Answering;
    use crate::advertising::{a_question, a_question_for_workspaces, about, about_a_workspace};
    use crate::machine::MachineId;
    use crate::presence::Presence;
    use crate::reading::a_workspace_in;
    use crate::wire::{IN, kind, write_name, write_sixteen};
    use crate::workspace::WorkspacePresence;

    /// A socket of this test's own, on this machine and nowhere else.
    fn a_socket() -> UdpSocket {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        socket
    }

    /// This machine.
    fn here() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// The port a test's workspace answers on.
    fn a_port() -> NonZeroU16 {
        NonZeroU16::new(8_443).unwrap()
    }

    /// `answering` on a socket of its own, asked `question` once by a socket
    /// of the test's; what answer_one said and every packet that came back
    /// within a short wait.
    fn asked(answering: &Answering, asking: &UdpSocket, question: &[u8]) -> (bool, Vec<Vec<u8>>) {
        let at = answering.socket.local_addr().unwrap();
        asking.send_to(question, at).unwrap();
        let answered = answering.answer_one().unwrap().is_some();
        asking
            .set_read_timeout(Some(Duration::from_millis(300)))
            .unwrap();
        let mut back = Vec::new();
        let mut heard = [0_u8; 1_500];
        while let Ok((how_many, _)) = asking.recv_from(&mut heard) {
            back.push(heard.get(..how_many).unwrap().to_vec());
        }
        (answered, back)
    }

    /// **A machine that hosts a workspace answers the question for workspaces
    /// with exactly the closed advertisement**, under its own identity.
    #[test]
    fn a_machine_hosting_a_workspace_answers_the_question_for_workspaces_with_the_closed_advertisement()
     {
        let answering =
            Answering::on(a_socket(), Presence::of(here(), 7_610)).hosting_a_workspace_at(a_port());
        let asking = a_socket();

        let (answered, back) = asked(&answering, &asking, &a_question_for_workspaces().unwrap());

        assert!(answered);
        assert_eq!(back.len(), 1, "{back:?}");
        let expected = about_a_workspace(&WorkspacePresence::of(here(), 8_443)).unwrap();
        assert_eq!(back.first().unwrap(), &expected);
        let found = a_workspace_in(&expected, std::net::Ipv4Addr::LOCALHOST.into()).unwrap();
        assert_eq!(found.host(), &here());
        assert_eq!(answering.workspace().unwrap().host(), &here());
    }

    /// **A machine that hosts no workspace steps over the question for
    /// workspaces**: nothing is sent, and it is not an error.
    #[test]
    fn a_machine_hosting_no_workspace_steps_over_the_question_for_workspaces() {
        let answering = Answering::on(a_socket(), Presence::of(here(), 7_610));
        let asking = a_socket();

        let (answered, back) = asked(&answering, &asking, &a_question_for_workspaces().unwrap());

        assert!(!answered, "a question for workspaces was answered");
        assert!(back.is_empty(), "{back:?}");
        assert!(answering.workspace().is_none());
    }

    /// **The machine's own presence answer is unchanged byte for byte either
    /// way**: hosting a workspace does not change what a machine says about
    /// itself, and a question for machines gets no workspace beside it.
    #[test]
    fn the_presence_answer_is_the_same_bytes_whether_or_not_a_workspace_is_hosted() {
        let presence = Presence::of(here(), 7_610);
        let expected = about(&presence).unwrap();

        let hosting_none = Answering::on(a_socket(), presence.clone());
        let hosting_one =
            Answering::on(a_socket(), presence.clone()).hosting_a_workspace_at(a_port());
        let asking = a_socket();

        let (answered, without) = asked(&hosting_none, &asking, &a_question().unwrap());
        assert!(answered);
        let (answered, with) = asked(&hosting_one, &asking, &a_question().unwrap());
        assert!(answered);

        assert_eq!(without, vec![expected.clone()]);
        assert_eq!(with, vec![expected]);
    }

    /// **A packet asking both is answered with both, one answer each**, and
    /// the presence answer is still those same bytes.
    #[test]
    fn one_packet_asking_both_questions_gets_both_answers_apart() {
        let presence = Presence::of(here(), 7_610);
        let answering =
            Answering::on(a_socket(), presence.clone()).hosting_a_workspace_at(a_port());
        let asking = a_socket();

        let mut both = Vec::new();
        write_sixteen(&mut both, 0);
        write_sixteen(&mut both, 0);
        write_sixteen(&mut both, 2);
        write_sixteen(&mut both, 0);
        write_sixteen(&mut both, 0);
        write_sixteen(&mut both, 0);
        for service in [crate::SERVICE, crate::WORKSPACE_SERVICE] {
            write_name(&mut both, service).unwrap();
            write_sixteen(&mut both, kind::PTR);
            write_sixteen(&mut both, IN);
        }

        let (answered, back) = asked(&answering, &asking, &both);
        assert!(answered);
        assert_eq!(
            back,
            vec![
                about(&presence).unwrap(),
                about_a_workspace(&WorkspacePresence::of(here(), 8_443)).unwrap()
            ]
        );
    }

    /// Something that is not a question is not answered, and is not an error
    /// — on a machine hosting a workspace too.
    #[test]
    fn a_packet_that_is_not_a_question_is_stepped_over() {
        let answering =
            Answering::on(a_socket(), Presence::of(here(), 7_610)).hosting_a_workspace_at(a_port());
        let asking = a_socket();
        let (answered, back) = asked(&answering, &asking, b"who is there?");
        assert!(!answered, "something that was not a question was answered");
        assert!(back.is_empty());
    }
}
