//! Where connections come from, and which door each one is on.
//!
//! [`crate::Listening`] answers this by asking the kernel, and it is the only
//! implementation that ships. The trait exists for the reason
//! `alo_files::Resolving`'s does: a decision this crate makes has to be
//! testable on a machine that cannot arrange the situation the decision is
//! about.
//!
//! # The situation one process cannot arrange
//!
//! Which door a connection is on is decided by **which user made it**, and a
//! test process is one user. So a test that connected to a real socket twice
//! would get the person's door twice, and *what the service does while an agent
//! is connected and a person answers* — which is the whole of what
//! [`crate::serving`] decides — could not be written down as a test at all.
//! Item 21c recorded that limit and could live with it, because it was testing
//! the mapping from a user to a door and that is a value. The service is not.
//!
//! So the service takes its connections from a [`Knocking`], the real one is
//! the socket, and a test's is a socket too — the same accepting, the same
//! reading, the same closing — differing only in that it is told which login
//! each connection would have come from. Nothing about the mapping is faked:
//! [`crate::Sides`] is still the only thing that decides a door on a running
//! machine, and it is still the kernel it asks.

use std::os::fd::BorrowedFd;
use std::os::unix::net::UnixStream;

use crate::listening::Listening;
use crate::refusing::NotACaller;
use crate::side::Side;

/// Somewhere connections arrive from, each on one of the two doors.
pub trait Knocking {
    /// What to wait on while nobody is knocking.
    ///
    /// One descriptor: this is a socket somebody connects to, and a service
    /// that had to poll several of them to find out whether anybody had would
    /// be a service with more than one front door.
    fn waiting_on(&self) -> BorrowedFd<'_>;

    /// Take the next connection, and say which door it is on.
    ///
    /// # Errors
    ///
    /// [`NotACaller`]. [`NotACaller::Stranger`] is one connection that will not
    /// be served and the service goes on; the rest are the machine, and
    /// [`NotACaller::is_only_this_connection`] is what tells them apart.
    fn next(&self) -> Result<(Side, UnixStream), NotACaller>;
}

impl Knocking for Listening {
    /// The socket this machine's agent service listens on.
    fn waiting_on(&self) -> BorrowedFd<'_> {
        Listening::waiting_on(self)
    }

    /// The kernel's own answer, and nothing that arrived in a message.
    fn next(&self) -> Result<(Side, UnixStream), NotACaller> {
        let accepted = Listening::next(self)?;
        Ok((accepted.side(), accepted.taken()))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::place::Place;
    use crate::testing::{a_directory_of_our_own, ourselves};

    /// **The socket is a `Knocking`, and what it says is what the kernel
    /// said.** The service is written against the trait, so this is the test
    /// that the thing it is really handed on a machine is the same shape.
    #[test]
    fn the_real_socket_answers_with_the_door_the_kernel_chose() {
        let folder = a_directory_of_our_own("knocking");
        let place = Place::under(&folder);
        let listening = Listening::at(place.clone(), ourselves()).unwrap();

        let client = UnixStream::connect(place.socket()).unwrap();
        let (side, connection) = Knocking::next(&listening).unwrap();

        assert_eq!(side, Side::Person, "the tests run as the person");
        assert!(
            connection.peer_addr().is_ok(),
            "a connection came back to be served on"
        );
        drop(client);
    }
}
