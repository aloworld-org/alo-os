//! Asking the service to stop, and the one thing that has to be true of it.
//!
//! [`Serving`](crate::Serving) spends nearly all of its life asleep inside one
//! `poll`, waiting for somebody to say something. Stopping it means waking it,
//! and the way something asleep on a set of file descriptors is woken is by one
//! of them becoming readable. So a stop is **a byte on a socket** rather than a
//! flag in memory.
//!
//! # Why not a flag
//!
//! A flag can only be noticed by a loop that wakes up to look at it, so a flag
//! means polling with a timeout — a machine that wakes several times a minute
//! forever in order to discover that nothing has happened. On a laptop that is
//! measurable, and it would be spent entirely on a question whose answer is
//! almost always no. A socket answers it while the machine sleeps.
//!
//! # And what stopping will really be
//!
//! On a running machine what asks for a stop is `SIGTERM`, and a signal handler
//! may do almost nothing: it may not allocate, take a lock, or call most of the
//! standard library. Writing one byte to an already-open file descriptor is one
//! of the few things it *may* do. [`Stop::stop`] is that write and nothing else,
//! so the handler the process installs is one call long. Installing it is queue
//! item 21e, and this is the shape it is owed.
//!
//! [`Stop`] can be handed to another thread and kept for the life of the
//! process; it holds one end of a socket pair and nothing else.

use std::io::Write as _;
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::net::UnixStream;

/// The end of the pair the service is listening to.
///
/// Held by [`crate::Serving`] for as long as it runs. Nothing is ever read from
/// it: the service stops when it becomes readable, and what the byte was does
/// not matter.
#[derive(Debug)]
pub struct Waking(UnixStream);

/// The end something else holds in order to stop the service.
///
/// Cheap to keep somewhere a signal handler can reach, and safe for one to use:
/// see this file's header.
#[derive(Debug)]
pub struct Stop(UnixStream);

impl Waking {
    /// A new pair: the end the service waits on, and the end that stops it.
    ///
    /// # Errors
    ///
    /// Whatever the machine said, as a `std::io::Error`. A machine that cannot
    /// make a socket pair cannot make the agent socket either, so there is
    /// nothing here to fall back to.
    pub fn made() -> Result<(Self, Stop), std::io::Error> {
        let (waiting, stopping) = UnixStream::pair()?;
        Ok((Self(waiting), Stop(stopping)))
    }

    /// What the service waits on, beside the socket and its connections.
    pub(crate) fn waiting_on(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl Stop {
    /// Ask the service to stop.
    ///
    /// Answers whether the ask was delivered. `false` means the service has
    /// already gone — there is nothing left to tell — and it is not an error:
    /// a stop that arrives after the thing being stopped has ended did what it
    /// was for.
    ///
    /// Asking twice is harmless. The service stops on the first byte and never
    /// reads either of them.
    pub fn stop(&self) -> bool {
        (&self.0).write_all(&[0]).is_ok()
    }

    /// Another handle onto the same stop.
    ///
    /// For a process that has more than one reason to stop — a signal, and
    /// whatever else 21e finds — so that neither has to own the only copy.
    ///
    /// # Errors
    ///
    /// Whatever the machine said, as a `std::io::Error`.
    pub fn again(&self) -> Result<Self, std::io::Error> {
        Ok(Self(self.0.try_clone()?))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::unix::ready;

    /// **A stop wakes something that is asleep on the socket.** The whole point
    /// of the pair: nothing is polled with a timeout, and the service is still
    /// stoppable.
    ///
    /// Asked beside a connection that has already been spoken on, because
    /// `crate::unix::ready` waits with no timeout: something has to be ready
    /// for it to answer at all, and what is asserted here is that the stop's
    /// own end is quiet until somebody asks and ready afterwards.
    #[test]
    fn a_stop_makes_the_waiting_end_readable() {
        let (waking, stop) = Waking::made().unwrap();
        let (spoken, mut speaking) = UnixStream::pair().unwrap();
        speaking.write_all(b"something").unwrap();

        assert_eq!(
            ready(&[Some(waking.waiting_on()), Some(spoken.as_fd())]).unwrap(),
            [false, true]
        );

        assert!(stop.stop());
        assert_eq!(
            ready(&[Some(waking.waiting_on()), Some(spoken.as_fd())]).unwrap(),
            [true, true]
        );
    }

    /// **Asking twice is harmless**, because nothing reads the byte: a process
    /// with a signal handler and a supervisor both saying stop is an ordinary
    /// morning rather than a bug.
    #[test]
    fn asking_twice_is_the_same_as_asking_once() {
        let (waking, stop) = Waking::made().unwrap();
        let second = stop.again().unwrap();

        assert!(stop.stop());
        assert!(second.stop());
        assert_eq!(ready(&[Some(waking.waiting_on())]).unwrap(), [true]);
        assert_eq!(
            ready(&[Some(waking.waiting_on())]).unwrap(),
            [true],
            "the byte was read, so a service could go back to sleep after being stopped"
        );
    }

    /// **A stop after the service has gone is not an error.** It answers
    /// `false`, and whoever asked has nothing to do about it — what they wanted
    /// has already happened.
    #[test]
    fn stopping_something_that_has_already_stopped_says_so() {
        let (waking, stop) = Waking::made().unwrap();
        drop(waking);

        // The first write may be accepted into a socket buffer nobody will ever
        // read; the second finds the connection gone. Either way, no panic and
        // no error escapes.
        let first = stop.stop();
        let second = stop.stop();
        assert!(!(first && second), "a stop reached a service that is gone");
    }
}
