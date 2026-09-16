//! Something to wait on that says a set of sockets moved.
//!
//! The service's own thread waits on the kernel's routing socket and moves the
//! sockets that follow it (`crate::wire::Wire::networks_changed`). The thread
//! that answers discovery beside it (`crate::answering_discovery`) cannot wait
//! on that routing socket too — two readers of one socket each take the other's
//! notifications — and a thread asleep on the responders it took before a cable
//! was plugged in is asleep on the wrong sockets: a question on the new cable
//! arrives at a socket nobody is waiting on, and is answered only when somebody
//! on another network happens to ask something. Measured by
//! `crate::a_cable_pulled_and_plugged_in_again`, which failed until this existed.
//!
//! So the responders **say** when they move: [`ToldOfAMove::tell`] writes one
//! byte into a pair of connected sockets, and the answering thread waits on the
//! other end beside everything else, empties it with [`ToldOfAMove::heard`], and
//! takes the responders again. Nothing wakes on an interval: a machine whose
//! networks do not change is told nothing and costs nothing for it.
//!
//! Both ends are non-blocking. A byte already waiting is a thread already about
//! to be woken, so a write that would block has said everything it needed to.

use std::io::{ErrorKind, Read as _, Write as _};
use std::os::fd::{AsFd as _, BorrowedFd};
use std::os::unix::net::UnixStream;

/// A pair of sockets: one end told that something moved, the other waited on.
#[derive(Debug)]
pub struct ToldOfAMove {
    /// The end written to when something moved.
    telling: UnixStream,
    /// The end waited on and emptied.
    told: UnixStream,
}

impl ToldOfAMove {
    /// A pair nothing has been said on yet.
    ///
    /// # Errors
    ///
    /// What the kernel said when the pair could not be made or made
    /// non-blocking.
    pub fn made() -> Result<Self, std::io::Error> {
        let (telling, told) = UnixStream::pair()?;
        telling.set_nonblocking(true)?;
        told.set_nonblocking(true)?;
        Ok(Self { telling, told })
    }

    /// Say that something moved.
    ///
    /// A byte that will not fit because bytes are already waiting has nothing
    /// to add: whoever waits is woken either way. Any other failure is the
    /// machine's, and is handed back for the service log.
    ///
    /// # Errors
    ///
    /// What the kernel said, other than that the pair is already full.
    pub fn tell(&self) -> Result<(), std::io::Error> {
        match (&self.telling).write(&[1]) {
            Ok(_) => Ok(()),
            Err(why) if why.kind() == ErrorKind::WouldBlock => Ok(()),
            Err(why) => Err(why),
        }
    }

    /// What to wait on for something having moved.
    #[must_use]
    pub fn waiting_on(&self) -> BorrowedFd<'_> {
        self.told.as_fd()
    }

    /// Empty what was said, and say whether anything was.
    ///
    /// Emptied **before** the moved sockets are taken again, so a move that
    /// happens after the taking leaves a byte behind and wakes the next wait at
    /// once, rather than being read away unseen.
    pub fn heard(&self) -> bool {
        let mut anything = false;
        let mut said = [0_u8; 64];
        loop {
            match (&self.told).read(&mut said) {
                Ok(0) => return anything,
                Ok(_) => anything = true,
                Err(why) if why.kind() == ErrorKind::Interrupted => {}
                // Nothing more waiting, or a pair that failed: either way there
                // is nothing more to read, and the caller takes the sockets
                // again regardless.
                Err(_) => return anything,
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use super::ToldOfAMove;
    use crate::unix::ready;

    /// Whether something is waiting to be heard, without waiting for it.
    fn waiting(moved: &ToldOfAMove) -> bool {
        let [said] = ready(&[Some(moved.waiting_on())], Some(Duration::ZERO)).unwrap();
        said
    }

    /// **Nothing said is nothing to wake on**, and nothing heard.
    #[test]
    fn nothing_said_is_nothing_to_wake_on() {
        let moved = ToldOfAMove::made().unwrap();
        assert!(!waiting(&moved), "a pair nothing was said on woke a wait");
        assert!(!moved.heard());
    }

    /// **A move said wakes the wait, and hearing it quiets it again.**
    #[test]
    fn a_move_said_wakes_the_wait_and_hearing_it_quiets_it() {
        let moved = ToldOfAMove::made().unwrap();
        moved.tell().unwrap();
        assert!(waiting(&moved));
        assert!(moved.heard());
        assert!(!waiting(&moved), "a move heard still wakes the wait");
        assert!(!moved.heard());
    }

    /// **Saying it more often than anybody listens never blocks and never
    /// fails**: the pair fills, the rest has nothing to add, and one hearing
    /// empties the lot.
    #[test]
    fn saying_it_into_a_full_pair_neither_blocks_nor_fails() {
        let moved = ToldOfAMove::made().unwrap();
        for _ in 0..100_000 {
            moved.tell().unwrap();
        }
        assert!(waiting(&moved));
        assert!(moved.heard());
        assert!(!waiting(&moved));
    }
}
