//! The socket the packets go over, and the two things a machine does on it.
//!
//! [`Answering`] holds this machine's presence and replies when somebody asks
//! who is here. [`Looking`] asks, and collects what comes back.
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

use crate::advertising::{a_question, about};
use crate::presence::{Found, Presence};
use crate::reading::{a_machine_in, a_question_in};
use crate::refusing::{NotNearby, because};

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

/// This machine, answering when somebody asks who is here.
pub struct Answering {
    /// The socket questions arrive on and answers go out of.
    socket: UdpSocket,
    /// What this machine says about itself, which does not change.
    presence: Presence,
}

impl Answering {
    /// This machine on a socket somebody else opened.
    ///
    /// The socket is taken rather than made so that a test can put both sides
    /// of the road on one host, and so that whoever runs alo OS decides which
    /// interfaces it speaks on rather than this crate deciding for them.
    #[must_use]
    pub const fn on(socket: UdpSocket, presence: Presence) -> Self {
        Self { socket, presence }
    }

    /// What this machine says about itself, for a caller that wants to see it
    /// without waiting for anybody to ask.
    #[must_use]
    pub const fn presence(&self) -> &Presence {
        &self.presence
    }

    /// Wait for one question and answer it.
    ///
    /// Returns who was answered, or nothing at all if what arrived was not a
    /// question for this service — a printer, a media player, or this machine's
    /// own answer coming back round. Neither is an error.
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
        if !a_question_in(asked) {
            return Ok(None);
        }
        let answer = about(&self.presence)?;
        self.socket
            .send_to(&answer, who)
            .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
        Ok(Some(who))
    }
}

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
        let until = Instant::now().checked_add(patience);
        let mut machines: Vec<Found> = Vec::new();
        let mut heard = [0_u8; AT_MOST];
        loop {
            let left = until
                .and_then(|until| until.checked_duration_since(Instant::now()))
                .unwrap_or_default();
            if left.is_zero() {
                return Ok(machines);
            }
            self.socket
                .set_read_timeout(Some(left))
                .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
            let Ok((how_many, who)) = self.socket.recv_from(&mut heard) else {
                // Nothing more arrived in the time there was, which is the
                // ordinary end of a search rather than a fault.
                return Ok(machines);
            };
            let Some(said) = heard.get(..how_many) else {
                continue;
            };
            if let Ok(found) = a_machine_in(said, who.ip())
                && !machines
                    .iter()
                    .any(|already| already.machine == found.machine)
            {
                machines.push(found);
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::net::UdpSocket;
    use std::time::Duration;

    use super::{Answering, Looking};
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

    /// Something that is not a question is not answered, and is not an error.
    #[test]
    fn a_packet_that_is_not_a_question_is_stepped_over() {
        let answering = a_socket();
        let at = answering.local_addr().unwrap();
        let answering = Answering::on(answering, Presence::of(MachineId::made().unwrap(), 7_610));
        let answered = std::thread::spawn(move || answering.answer_one());

        let saying = a_socket();
        saying.send_to(b"who is there?", at).unwrap();

        assert!(
            answered.join().unwrap().unwrap().is_none(),
            "something that was not a question was answered"
        );
    }
}
