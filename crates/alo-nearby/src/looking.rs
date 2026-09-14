//! Asking who is here on the socket the packets go over, and what comes back.
//!
//! [`Looking`] asks who is here, and collects what comes back. Its other half,
//! this machine answering, is [`crate::answering`]'s.
//!
//! # Nothing here connects to anything it finds
//!
//! Not one line opens a connection to a machine it heard from. Everything is
//! one unconnected datagram socket, and the port a [`Found`] carries is a
//! number that is written down rather than dialled. What it takes to use a
//! machine found this way is ADR 0003's mutual pairing, and the one wire this
//! crate has for that lives in `dialling.rs` and `receiving.rs` and dials
//! [`Found::where_it_answers`] under a proposal a person made; a
//! `TcpStream::connect` appearing in this file or any other would mean that
//! decision had been quietly reversed, and
//! `the_local_network_says_no_more_than_a_machine_exists.rs` fails if one does.
//!
//! # Finding nothing is an answer
//!
//! [`Looking::found`] returns an empty list on a machine with no network, no
//! neighbours, or no route to the multicast address — the way
//! `alo_models::found_on_this_machine` answers with an empty list rather than a
//! refusal. A machine alone in a room is not a machine in trouble. What is an
//! error is a socket that will not listen at all, which is
//! [`NotNearby::TheNetwork`] and is about this machine.
//!
//! # What is owed to a machine
//!
//! The tests in this workspace put both sides on one host over ordinary
//! datagrams, which is what shows the packets are right and the road is walked.
//! **Whether a second physical machine on an office network hears this one is
//! not shown by any test here**, and is owed to two machines. The report says
//! so rather than implying otherwise.

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use crate::advertising::{a_question, a_question_for_workspaces};
use crate::presence::Found;
use crate::reading::{a_machine_in, a_workspace_in};
use crate::refusing::{NotNearby, because};
use crate::workspace::FoundWorkspace;

/// The address every machine on a link listens to for this kind of question.
pub const THE_ADDRESS: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);

/// The port it listens on.
pub const THE_PORT: u16 = 5353;

/// The most bytes one datagram is read into.
///
/// An advertisement from this crate is under two hundred; a stranger's may be
/// anything, and what does not fit is what is not read, which for a packet this
/// crate will refuse anyway costs nothing.
const AT_MOST: usize = 1_500;

/// This machine, asking who else is here.
pub struct Looking {
    /// The socket the question goes out of and answers arrive on.
    socket: UdpSocket,
}

impl Looking {
    /// A search on a socket somebody else opened.
    #[must_use]
    pub const fn from(socket: UdpSocket) -> Self {
        Self { socket }
    }

    /// Ask who is here.
    ///
    /// # Errors
    ///
    /// [`NotNearby::TheNetwork`] if the question cannot be sent, which on a
    /// machine with no route to the multicast address is what happens instead
    /// of an empty answer — and is why [`found`](Self::found) is the thing that
    /// returns nothing rather than this.
    pub fn ask(&self, of: SocketAddr) -> Result<(), NotNearby> {
        let question = a_question()?;
        self.socket
            .send_to(&question, of)
            .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
        Ok(())
    }

    /// Ask which workspaces are here.
    ///
    /// # Errors
    ///
    /// As [`ask`](Self::ask).
    pub fn ask_for_workspaces(&self, of: SocketAddr) -> Result<(), NotNearby> {
        let question = a_question_for_workspaces()?;
        self.socket
            .send_to(&question, of)
            .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
        Ok(())
    }

    /// Every machine that answers within `patience`, each counted once.
    ///
    /// A packet that is not an alo machine's is stepped over rather than
    /// carried out as a refusal — a network has printers on it, and one
    /// printer's answer is not a reason to stop listening for the machine down
    /// the corridor.
    ///
    /// # Errors
    ///
    /// [`NotNearby::TheNetwork`] if a read timeout cannot be set on the socket.
    /// **Not** if nothing answers: that is an empty list.
    pub fn found(&self, patience: Duration) -> Result<Vec<Found>, NotNearby> {
        self.around(patience).map(|around| around.machines)
    }

    /// Every machine and every workspace that answers within `patience`.
    ///
    /// One window for both, so that a workspace and the machine that says it
    /// serves it are heard at the same moment and can be told to have answered
    /// from the same address. A machine is counted once by its identity; a
    /// workspace once by its identity, address and port — two answers claiming
    /// one identity from two addresses are two things heard, and neither is
    /// quietly preferred. Anything that is neither is stepped over.
    ///
    /// # Errors
    ///
    /// As [`found`](Self::found).
    pub fn around(&self, patience: Duration) -> Result<Around, NotNearby> {
        let until = Instant::now().checked_add(patience);
        let mut around = Around::default();
        let mut heard = [0_u8; AT_MOST];
        loop {
            let left = until
                .and_then(|until| until.checked_duration_since(Instant::now()))
                .unwrap_or_default();
            if left.is_zero() {
                return Ok(around);
            }
            self.socket
                .set_read_timeout(Some(left))
                .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
            let Ok((how_many, who)) = self.socket.recv_from(&mut heard) else {
                // Nothing more arrived in the time there was, which is the
                // ordinary end of a search rather than a fault.
                return Ok(around);
            };
            let Some(said) = heard.get(..how_many) else {
                continue;
            };
            if let Ok(found) = a_machine_in(said, who.ip()) {
                if !around
                    .machines
                    .iter()
                    .any(|already| already.machine == found.machine)
                {
                    around.machines.push(found);
                }
            } else if let Ok(found) = a_workspace_in(said, who.ip())
                && !around.workspaces.contains(&found)
            {
                around.workspaces.push(found);
            }
        }
    }
}

/// What answered on the link in one window: machines, and workspaces.
///
/// Both are facts written down and nothing more — every one of them
/// [`crate::Standing::NotPaired`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Around {
    /// The alo machines that said they exist, in the order they answered.
    pub machines: Vec<Found>,
    /// The workspaces whose hosts said they serve one, in the order they
    /// answered.
    pub workspaces: Vec<FoundWorkspace>,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::net::UdpSocket;
    use std::time::Duration;

    use super::Looking;
    use crate::answering::Answering;
    use crate::machine::MachineId;
    use crate::presence::{Presence, Standing};

    /// A socket of this test's own, on this machine and nowhere else.
    fn a_socket() -> UdpSocket {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        socket
    }

    /// **The road, walked.** One machine asks, another answers, and what comes
    /// back is one machine that is not paired with anything.
    #[test]
    fn a_machine_that_asks_is_answered_by_one_that_is_there() {
        let machine = MachineId::made().unwrap();
        let answering = a_socket();
        let at = answering.local_addr().unwrap();
        let answering = Answering::on(answering, Presence::of(machine.clone(), 7_610));
        let answered = std::thread::spawn(move || answering.answer_one());

        let looking = Looking::from(a_socket());
        looking.ask(at).unwrap();
        let found = looking.found(Duration::from_secs(5)).unwrap();

        assert!(answered.join().unwrap().unwrap().is_some());
        assert_eq!(found.len(), 1, "{found:?}");
        let one = found.first().unwrap();
        assert_eq!(one.machine, machine);
        assert_eq!(one.port, 7_610);
        assert_eq!(one.address, at.ip());
        assert_eq!(
            one.where_it_answers(),
            std::net::SocketAddr::new(at.ip(), 7_610)
        );
        assert_eq!(one.standing, Standing::NotPaired);
    }

    /// **A machine with nobody to hear it finds nothing, which is an answer.**
    /// Not a refusal: a machine alone in a room is not a machine in trouble.
    #[test]
    fn a_machine_with_no_neighbours_finds_nothing_rather_than_failing() {
        let looking = Looking::from(a_socket());
        let found = looking.found(Duration::from_millis(200)).unwrap();
        assert!(found.is_empty(), "{found:?}");
    }

    /// **One window hears both**: a machine and a workspace answering the
    /// two questions come back apart, each once, and a machine's own
    /// discovery does not answer the question for workspaces.
    #[test]
    fn one_look_around_hears_machines_and_workspaces_apart() {
        let machine = MachineId::made().unwrap();
        let answering = a_socket();
        let at = answering.local_addr().unwrap();
        let answering = Answering::on(answering, Presence::of(machine.clone(), 7_610));

        let looking = Looking::from(a_socket());
        looking.ask_for_workspaces(at).unwrap();
        // The machine's own discovery hears a question that is not for it.
        assert!(answering.answer_one().unwrap().is_none());
        looking.ask(at).unwrap();
        assert!(answering.answer_one().unwrap().is_some());

        // A workspace host answers twice, which is counted once.
        let workspace = crate::workspace::WorkspacePresence::of(machine.clone(), 8_443);
        let packet = crate::advertising::about_a_workspace(&workspace).unwrap();
        let host = a_socket();
        let to = looking.socket.local_addr().unwrap();
        host.send_to(&packet, to).unwrap();
        host.send_to(&packet, to).unwrap();

        let around = looking.around(Duration::from_millis(500)).unwrap();
        assert_eq!(around.machines.len(), 1, "{around:?}");
        assert_eq!(around.machines.first().unwrap().machine, machine);
        assert_eq!(around.workspaces.len(), 1, "{around:?}");
        let found = around.workspaces.first().unwrap();
        assert_eq!(found.host(), &machine);
        assert_eq!(found.port(), 8_443);
        assert_eq!(found.address(), host.local_addr().unwrap().ip());
    }
}
