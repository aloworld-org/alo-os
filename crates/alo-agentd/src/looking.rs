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

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use alo_nearby::{Found, Looking};

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
}
