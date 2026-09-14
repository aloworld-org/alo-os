//! Discovery, done honestly, at the moment a proposal arrives.
//!
//! `alo_nearby::Proposals::arrived` refuses a proposal from an address
//! discovery on this machine has never measured — task 7's *a proposal that
//! arrived from an address discovery never measured is refused before anybody
//! is shown anything* — and takes the list of what was measured as an
//! argument. This daemon keeps no such list, because a list kept would age:
//! a machine found this morning at one address may be at another by the
//! afternoon, and a proposal judged against the morning's list would be
//! judged against a stale fact.
//!
//! So the measurement is made at the moment: the machine the proposal came
//! from is asked, at the address the connection came from and the port its
//! own discovery answers at, whether it exists, and what it answers is what
//! the proposal is judged against. A machine that does not answer was not
//! found, and its proposal is refused as such — which is the true thing.
//!
//! **Nothing here is a lookup of anything typed.** The address is the
//! connection's, measured by the kernel; the port is the wire's constant.
//!
//! # And for a proposal the person here makes
//!
//! The person's door proposes a pairing to a machine by its identity
//! (`crate::pairing`), and the identity is all it carries: no address, so
//! nothing typed can be dialled. [`found_by_name`] asks the link who is here
//! — the multicast group on a real machine, the socket the other side of a
//! test bound — and answers with the one machine that said it was the one
//! asked for, at the address it answered from. A machine that does not
//! answer is not found, and nothing is proposed to it. [`LookingFor`] is that
//! question as the door asks it, so the door can be tested against a network
//! with nobody on it.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use alo_nearby::{Found, Looking, MachineId};

/// Somewhere a machine can be looked for by its identity, at the moment.
///
/// One method, answering what discovery measured just now or nothing. The
/// `Wire` is the implementation that ships (`crate::wire`), asking the link
/// it is bound to; a test hands the door a network with nobody on it.
pub trait LookingFor: std::fmt::Debug {
    /// The machine with this identity, if it answered on the network just
    /// now, at the address it answered from.
    fn look_for(&self, machine: &MachineId) -> Option<Found>;
}

/// Ask who is here at `at`, and answer with the machine named `machine` if
/// it answered, at the address it answered from.
///
/// `None` when nothing answered by that name, nothing could be asked, or the
/// socket would not bind — every one of which the caller reads as *not
/// found*: a proposal to a machine this one cannot find is refused before
/// anything is sent, which is the whole of what the measurement is for.
#[must_use]
pub fn found_by_name(machine: &MachineId, at: SocketAddr) -> Option<Found> {
    let here: IpAddr = if at.ip().is_loopback() {
        Ipv4Addr::LOCALHOST.into()
    } else {
        Ipv4Addr::UNSPECIFIED.into()
    };
    let socket = UdpSocket::bind((here, 0)).ok()?;
    let looking = Looking::from(socket);
    looking.ask(at).ok()?;
    looking
        .found(WHILE_LOOKING)
        .unwrap_or_default()
        .into_iter()
        .find(|found| found.machine == *machine)
}

/// How long this machine waits for the machine that proposed to say it
/// exists.
///
/// Two seconds: it is on the same link, it is the machine that just opened a
/// connection here, and a proposal is a person waiting at a screen rather
/// than an agent in a loop.
pub const WHILE_LOOKING: Duration = Duration::from_secs(2);

/// Ask the machine at `from` whether it exists, at the port its discovery
/// answers on, and answer with what was found.
///
/// Empty when nothing answered or nothing could be asked, which the caller
/// reads as *not found*: a proposal from a machine this one cannot find is
/// refused by `alo_nearby::Proposals::arrived`, and refusing it is the whole
/// of what this measurement is for.
#[must_use]
pub fn found_at(from: IpAddr, asking_at: u16) -> Vec<Found> {
    let here: IpAddr = if from.is_loopback() {
        Ipv4Addr::LOCALHOST.into()
    } else {
        Ipv4Addr::UNSPECIFIED.into()
    };
    let Ok(socket) = UdpSocket::bind((here, 0)) else {
        return Vec::new();
    };
    let looking = Looking::from(socket);
    if looking.ask(SocketAddr::new(from, asking_at)).is_err() {
        return Vec::new();
    }
    looking.found(WHILE_LOOKING).unwrap_or_default()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::{Ipv4Addr, UdpSocket};

    use alo_nearby::{Answering, MachineId, Presence};

    use super::found_at;

    /// The machine that proposed, for these tests.
    fn reception() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// **A machine that answers at the address it came from is found there**,
    /// with the port it advertises.
    #[test]
    fn a_machine_that_answers_is_found_at_the_address_it_came_from() {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = socket.local_addr().unwrap();
        let answering = Answering::on(socket, Presence::of(reception(), 7_610));
        let answered = std::thread::spawn(move || answering.answer_one());

        let found = found_at(at.ip(), at.port());
        assert!(answered.join().unwrap().unwrap().is_some());
        let one = found.iter().find(|one| one.machine == reception()).unwrap();
        assert_eq!(one.address, at.ip());
        assert_eq!(one.port, 7_610);
    }

    /// **A machine that does not answer is not found**, and nothing is
    /// invented in its place.
    #[test]
    fn a_machine_that_does_not_answer_is_not_found() {
        let quiet = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = quiet.local_addr().unwrap();
        assert!(found_at(at.ip(), at.port()).is_empty());
    }

    /// **A machine looked for by its identity is found only if it is the one
    /// that answered**: the machine that is there is found at the address it
    /// answered from, and another identity on the same link is not.
    #[test]
    fn a_machine_looked_for_by_name_is_found_only_if_it_is_the_one_that_answered() {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = socket.local_addr().unwrap();
        let answering = Answering::on(socket, Presence::of(reception(), 7_610));
        let answered =
            std::thread::spawn(move || (0..2).map(|_| answering.answer_one()).collect::<Vec<_>>());

        let found = super::found_by_name(&reception(), at).unwrap();
        assert_eq!(found.machine, reception());
        assert_eq!(found.where_it_answers().ip(), at.ip());
        let somebody_else = MachineId::read("99998888777766665555444433332222").unwrap();
        assert!(super::found_by_name(&somebody_else, at).is_none());
        drop(answered.join().unwrap());
    }
}
