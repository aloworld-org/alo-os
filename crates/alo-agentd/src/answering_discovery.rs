//! Discovery, answered beside the service rather than between its rounds.
//!
//! # Why a thread of its own
//!
//! A proposal is judged against a measurement made at the moment it arrives
//! (`crate::looking::found_at`): the asked machine asks the asking machine, at
//! the address its connection came from, whether it exists. **The asking
//! machine is, at that moment, waiting for the answer to its own proposal** —
//! its person's `pair` request dials the other machine and reads the reply
//! inside one round of `crate::serving`. A service that answered discovery only
//! between rounds would hear the question after the reply it was waiting for,
//! the asked machine would hear nothing within `crate::looking::WHILE_LOOKING`,
//! and every proposal between two alo machines would be refused as coming from
//! a machine nobody found. Two daemons could never pair.
//!
//! So discovery is answered here, on a thread of its own for exactly as long as
//! the service runs, in both families. What is answered is unchanged — the same
//! [`Wire`], the same presence and the same workspace, whoever asks and whatever
//! the service is doing — and nothing else moves off the service's thread: the
//! doors, the port, the turn and the kernel's notifications are all still read
//! in one place.
//!
//! # And how it ends
//!
//! The thread sleeps in one `poll` on both discovery sockets and the far end of
//! a pair of sockets the service holds the near end of. The service ending —
//! stopped, or failed — drops the near end, the hangup wakes the thread, and it
//! answers what is already waiting and returns before the service does.
//!
//! A socket that will not read or answer is still the machine's, and still ends
//! the service, as it did when the service answered it itself: the thread keeps
//! the reason, drops its end of a second pair, and the service — which waits on
//! that end beside everything else — stops with [`NotServed::TheWire`].

use std::os::fd::{AsFd as _, BorrowedFd};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

use alo_nearby::NotNearby;

use crate::refusing::NotServed;
use crate::unix::ready;
use crate::wire::Wire;

/// What the service holds of discovery answered beside it: what to wait on for
/// it failing, why it failed, and how many times it answered.
#[derive(Debug)]
pub(crate) struct Answered {
    /// Readable once the answering has stopped for a reason of the machine's.
    told: UnixStream,
    /// That reason, once there is one.
    failed: Mutex<Option<NotNearby>>,
    /// How many questions were answered.
    answered: AtomicU64,
}

impl Answered {
    /// What to wait on for discovery no longer being answered.
    pub(crate) fn failed_waiting_on(&self) -> BorrowedFd<'_> {
        self.told.as_fd()
    }

    /// Why discovery is no longer answered, as the service stops with it.
    pub(crate) fn why(&self) -> NotServed {
        let failed = self
            .failed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        NotServed::TheWire(failed.unwrap_or_else(|| {
            NotNearby::TheNetwork("discovery stopped being answered".to_owned())
        }))
    }

    /// Keep why the answering stopped.
    fn stopped_because(&self, why: NotNearby) {
        *self.failed.lock().unwrap_or_else(PoisonError::into_inner) = Some(why);
    }
}

/// Run `serving` with discovery on `wire` answered beside it on a thread of its
/// own, and hand back what it answered with and how many discovery questions
/// were answered while it ran.
///
/// # Errors
///
/// [`NotServed::NotWaiting`] when the pairs of sockets the thread is ended and
/// heard from through cannot be made — before anything is answered or served.
pub(crate) fn beside<T>(
    wire: &Wire,
    serving: impl FnOnce(&Answered) -> T,
) -> Result<(T, u64), NotServed> {
    let (ending, ended) = UnixStream::pair().map_err(|why| NotServed::NotWaiting { why })?;
    let (told, telling) = UnixStream::pair().map_err(|why| NotServed::NotWaiting { why })?;
    let answered = Answered {
        told,
        failed: Mutex::new(None),
        answered: AtomicU64::new(0),
    };
    let served = std::thread::scope(|scope| {
        let answered = &answered;
        let answering = scope.spawn(move || {
            answer_until_ended(wire, answered, &ended);
            drop(telling);
        });
        let served = serving(answered);
        drop(ending);
        if answering.join().is_err() {
            eprintln!("alo-agentd: the thread answering discovery ended by panicking");
        }
        served
    });
    Ok((served, answered.answered.load(Ordering::Relaxed)))
}

/// One family's way of answering a discovery question on a wire.
type Answer = fn(&Wire) -> Result<Option<std::net::SocketAddr>, NotNearby>;

/// Answer every discovery question on `wire`, in both families, until `ended`
/// hangs up or a socket fails.
fn answer_until_ended(wire: &Wire, answered: &Answered, ended: &UnixStream) {
    loop {
        let waiting_on = [
            Some(ended.as_fd()),
            Some(wire.discovery_waiting_on()),
            wire.discovery_ipv6_waiting_on(),
        ];
        let [stop, over_ipv4, over_ipv6] = match ready(&waiting_on, None) {
            Ok(ready) => ready,
            Err(why) => {
                answered.stopped_because(NotNearby::TheNetwork(why.to_string()));
                return;
            }
        };
        let answering: [(bool, Answer); 2] = [
            (over_ipv4, Wire::answer_discovery),
            (over_ipv6, Wire::answer_discovery_over_ipv6),
        ];
        for (asked, answer) in answering {
            if !asked {
                continue;
            }
            // A question that was not one for this service — a printer's, or
            // this machine's own answer coming back round — is nothing to count.
            match answer(wire) {
                Ok(Some(_)) => {
                    answered.answered.fetch_add(1, Ordering::Relaxed);
                }
                Ok(None) => {}
                Err(why) => {
                    answered.stopped_because(why);
                    return;
                }
            }
        }
        if stop {
            return;
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::{Ipv4Addr, TcpListener, UdpSocket};
    use std::time::Duration;

    use alo_nearby::{Looking, MachineId};

    use super::beside;
    use crate::refusing::NotServed;
    use crate::unix::ready;
    use crate::wire::Wire;

    /// This machine.
    fn here() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// A wire on sockets of this test's own, where its discovery answers, and a
    /// second handle on its discovery socket.
    fn a_wire() -> (Wire, std::net::SocketAddr, UdpSocket) {
        let discovery = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = discovery.local_addr().unwrap();
        let held = discovery.try_clone().unwrap();
        let wire = Wire::on(
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
            discovery,
            here(),
            0,
        )
        .unwrap();
        (wire, at, held)
    }

    /// **Discovery is answered while the service is busy with something else**
    /// — here, a round that does nothing but look for this very machine, as the
    /// asked machine does while the asking one waits on its own proposal — and
    /// the answering ends when the service does, counted.
    #[test]
    fn discovery_is_answered_while_the_service_is_busy() {
        let (wire, at, _held) = a_wire();
        let ((found, failed), answered) = beside(&wire, |answered| {
            let looking = Looking::from(UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap());
            looking.ask(at).unwrap();
            looking.ask(at).unwrap();
            let found = looking.found(Duration::from_secs(2)).unwrap();
            let failed = ready(
                &[Some(answered.failed_waiting_on())],
                Some(Duration::from_millis(50)),
            )
            .unwrap();
            (found, failed)
        })
        .unwrap();
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found.first().unwrap().machine, here());
        assert_eq!(
            failed,
            [false],
            "the answering stopped while the service ran"
        );
        assert_eq!(answered, 2);
    }

    /// **A socket that stops reading ends the service, as the machine's**: the
    /// answering stops, the service is woken on the end it waits on, and what it
    /// stops with is the wire's refusal.
    #[test]
    fn a_discovery_socket_that_fails_wakes_the_service_with_why() {
        let (wire, at, held) = a_wire();
        // A socket shut for sending reads a question and cannot answer it.
        let ((woken, why), _) = beside(&wire, |answered| {
            crate::unix::stopped_sending(&held).unwrap();
            let asking = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
            asking
                .send_to(&alo_nearby::advertising::a_question().unwrap(), at)
                .unwrap();
            let woken = ready(
                &[Some(answered.failed_waiting_on())],
                Some(Duration::from_secs(5)),
            )
            .unwrap();
            (woken, answered.why())
        })
        .unwrap();
        assert_eq!(woken, [true]);
        assert!(matches!(why, NotServed::TheWire(_)), "{why:?}");
    }
}
