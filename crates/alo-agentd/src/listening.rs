//! The socket itself: putting it there, and answering the door.
//!
//! # What binding a socket has to deal with first
//!
//! A socket is a name in a directory, and the name may already be taken — by a
//! daemon that is running, by one that was killed, or by a file that has
//! nothing to do with alo OS. The three are told apart by **asking**, and each
//! has a different answer:
//!
//! - something is listening: [`NotBound::AlreadyRunning`], and the socket is
//!   left exactly as it is. Two daemons on one socket would mean approvals
//!   arriving at whichever won the race;
//! - nothing is listening: the name is stale, and it is removed. A socket
//!   nobody answers on is the remains of a daemon that stopped, and the
//!   alternative is a machine that cannot start its agent service again until
//!   somebody logs in and deletes a file;
//! - it is not a socket at all: [`NotBound::NotOurSocket`], and it is left
//!   where it is. Deleting a file because it was in the way is what a daemon
//!   that has taken a machine over from its owner does.
//!
//! # And the socket goes when the daemon does
//!
//! [`Listening`] removes the socket when it is dropped, so an ordinary stop
//! leaves no name behind for the next start to reason about. The stale case
//! above is still needed and always will be: a process that was killed outright
//! runs no destructor.
//!
//! # One question per connection, asked of the kernel
//!
//! [`Listening::next`] accepts, asks `SO_PEERCRED` who is there, and hands back
//! an [`Accepted`] carrying the side, the caller and the connection. Anybody
//! who is neither the person nor the agent is refused, and the refusal is the
//! connection being **closed with nothing written on it** — no sentence, no
//! code, no protocol version.
//!
//! That is the one place this crate parts company with
//! `docs/contracts/daemon-protocol.md`'s *a message that is not a request is
//! refused in words and never dropped*. The contract is about messages, from
//! clients this machine has two doors for; a stranger has sent no message and
//! is not one of them. Answering would tell whoever is knocking that there is
//! an alo OS daemon here and what version it is, and they cannot be told
//! anything else — so they are told nothing.

use std::os::unix::fs::FileTypeExt;
use std::os::unix::net::{UnixListener, UnixStream};

use crate::caller::Caller;
use crate::place::Place;
use crate::refusing::{NotACaller, NotBound};
use crate::side::{Side, Sides};
use crate::unix::{us, who};

/// A connection, and who is on it.
#[derive(Debug)]
pub struct Accepted {
    /// Which of the protocol's two doors this connection is on.
    side: Side,
    /// Who the kernel said is there.
    caller: Caller,
    /// The connection itself.
    connection: UnixStream,
}

impl Accepted {
    /// Which door this connection is on.
    #[must_use]
    pub const fn side(&self) -> Side {
        self.side
    }

    /// Who is at the other end.
    #[must_use]
    pub const fn caller(&self) -> &Caller {
        &self.caller
    }

    /// The connection, to read a request off and write an answer onto.
    #[must_use]
    pub const fn connection(&self) -> &UnixStream {
        &self.connection
    }

    /// The connection, taken out to be served.
    #[must_use]
    pub fn taken(self) -> UnixStream {
        self.connection
    }
}

/// This machine's agent socket, bound and listening.
#[derive(Debug)]
pub struct Listening {
    /// Where the socket is.
    place: Place,
    /// The two users this machine has.
    sides: Sides,
    /// The bound socket.
    listener: UnixListener,
}

impl Listening {
    /// Put the socket where it belongs and listen on it.
    ///
    /// # Errors
    ///
    /// [`NotBound::NotThePerson`] if this process is not running as the user it
    /// was told the person is — a daemon that opened the person's door for
    /// somebody else's login would be one nobody could see was wrong.
    /// [`NotBound::AlreadyRunning`] and [`NotBound::NotOurSocket`] are the two
    /// things in the way that are not ours to move, and the rest are
    /// [`Place::prepared`]'s and the machine's.
    ///
    /// In every one of them nothing is listening, and anything that was already
    /// at the socket's path is still there.
    pub fn at(place: Place, sides: Sides) -> Result<Self, NotBound> {
        let running_as = us()?;
        if running_as != sides.person() {
            return Err(NotBound::NotThePerson {
                us: running_as.raw(),
                told: sides.person().raw(),
            });
        }

        place.prepared(&sides)?;
        clear_the_way(&place)?;

        let listener = UnixListener::bind(place.socket()).map_err(|why| NotBound::NotBoundTo {
            at: place.socket().to_path_buf(),
            why,
        })?;

        // A socket that is bound but not yet shut is a door that is open to
        // whoever is in the directory, so a mode that would not be set means
        // taking the socket away again rather than serving on it.
        if let Err(why) = place.shut_the_socket(&sides) {
            drop(listener);
            drop(std::fs::remove_file(place.socket()));
            return Err(why);
        }

        Ok(Self {
            place,
            sides,
            listener,
        })
    }

    /// Where this socket is.
    #[must_use]
    pub const fn place(&self) -> &Place {
        &self.place
    }

    /// The two users this machine has.
    #[must_use]
    pub const fn sides(&self) -> &Sides {
        &self.sides
    }

    /// Wait for a connection, and answer who is on it.
    ///
    /// # Errors
    ///
    /// [`NotACaller::Stranger`] for anybody who is neither the person nor the
    /// agent, and the connection is closed as this answers — there is no
    /// borrowing of it out of a refusal, so a caller of this method cannot
    /// serve one by accident. [`NotACaller::NotAccepted`],
    /// [`NotACaller::NotAsked`] and [`NotACaller::NotAUser`] are the machine or
    /// the kernel not answering, and none of them is a lesser kind of yes.
    pub fn next(&self) -> Result<Accepted, NotACaller> {
        let (connection, _) = self
            .listener
            .accept()
            .map_err(|why| NotACaller::NotAccepted { why })?;
        let caller = who(&connection)?;
        let side = self.sides.which(&caller)?;
        Ok(Accepted {
            side,
            caller,
            connection,
        })
    }
}

impl Drop for Listening {
    /// Take the socket away with the daemon.
    ///
    /// Whether the file went is not reported, because there is nobody to report
    /// it to: this runs while the process is stopping. What it costs when it
    /// fails is one stale socket, which is exactly the case
    /// [`Listening::at`] already deals with.
    fn drop(&mut self) {
        drop(std::fs::remove_file(self.place.socket()));
    }
}

/// Deal with whatever is already at the socket's path.
///
/// The three cases and their reasoning are this file's own documentation.
fn clear_the_way(place: &Place) -> Result<(), NotBound> {
    let what_is_there = match std::fs::symlink_metadata(place.socket()) {
        Ok(what_is_there) => what_is_there,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(why) => {
            return Err(NotBound::Unreadable {
                at: place.socket().to_path_buf(),
                why,
            });
        }
    };

    if !what_is_there.file_type().is_socket() {
        return Err(NotBound::NotOurSocket {
            at: place.socket().to_path_buf(),
        });
    }

    if UnixStream::connect(place.socket()).is_ok() {
        return Err(NotBound::AlreadyRunning {
            at: place.socket().to_path_buf(),
        });
    }

    std::fs::remove_file(place.socket()).map_err(|why| NotBound::NotRemoved {
        at: place.socket().to_path_buf(),
        why,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::caller::{Gid, Uid};
    use crate::testing::{a_directory_of_our_own, calling_as_the_person, ourselves};
    use std::path::Path;

    /// What is really at a path, without following a link.
    fn what_is_at(path: &Path) -> std::fs::Metadata {
        std::fs::symlink_metadata(path).unwrap()
    }

    /// **The socket is bound, and it is a socket.** The ordinary case, on a
    /// real filesystem, on the machine the tests are running on.
    #[test]
    fn the_socket_is_put_where_it_belongs() {
        let folder = a_directory_of_our_own("bound");
        let place = Place::under(&folder);

        let listening = Listening::at(place.clone(), ourselves()).unwrap();

        assert!(what_is_at(place.socket()).file_type().is_socket());
        assert_eq!(listening.place(), &place);
    }

    /// **A second daemon does not take the socket over.** Something is
    /// listening, so the answer names the path and changes nothing — and the
    /// first daemon is still there afterwards.
    #[test]
    fn a_second_daemon_is_refused_and_the_first_keeps_the_socket() {
        let folder = a_directory_of_our_own("two-daemons");
        let place = Place::under(&folder);
        let first = Listening::at(place.clone(), ourselves()).unwrap();

        let second = Listening::at(place.clone(), ourselves()).unwrap_err();
        assert!(matches!(second, NotBound::AlreadyRunning { .. }));

        assert!(UnixStream::connect(place.socket()).is_ok());
        drop(first);
    }

    /// **A socket nobody is listening on is removed and bound again.** A
    /// machine whose daemon was killed comes back up without somebody logging
    /// in to delete a file.
    #[test]
    fn a_stale_socket_is_taken_over() {
        let folder = a_directory_of_our_own("stale");
        let place = Place::under(&folder);
        place.prepared(&ourselves()).unwrap();
        // A socket that outlived whatever was listening on it, made the way a
        // killed daemon leaves one: bound, then gone with no destructor run.
        drop(UnixListener::bind(place.socket()).unwrap());
        assert!(what_is_at(place.socket()).file_type().is_socket());

        let listening = Listening::at(place.clone(), ourselves()).unwrap();
        assert!(UnixStream::connect(place.socket()).is_ok());
        drop(listening);
    }

    /// **A file that is not a socket is refused and left there.** It belongs to
    /// somebody, and a daemon that deleted it to make room for itself would be
    /// taking the machine from its owner over a name.
    #[test]
    fn a_file_in_the_way_is_refused_and_left_there() {
        let folder = a_directory_of_our_own("in-the-way");
        let place = Place::under(&folder);
        place.prepared(&ourselves()).unwrap();
        std::fs::write(place.socket(), b"not a socket").unwrap();

        let refused = Listening::at(place.clone(), ourselves()).unwrap_err();
        assert!(matches!(refused, NotBound::NotOurSocket { .. }));
        assert_eq!(std::fs::read(place.socket()).unwrap(), b"not a socket");
    }

    /// **The daemon refuses to be a person it is not running as.** A machine
    /// whose configuration names somebody else would otherwise open the
    /// person's door — approvals and all — for a login this process has nothing
    /// to do with.
    #[test]
    fn it_will_not_open_the_door_of_a_person_it_is_not() {
        let folder = a_directory_of_our_own("not-the-person");
        let place = Place::under(&folder);
        let somebody_else = Sides::of(
            Uid::of(4_242_424).unwrap(),
            Uid::of(989).unwrap(),
            Gid::of(989).unwrap(),
        )
        .unwrap();

        let refused = Listening::at(place.clone(), somebody_else).unwrap_err();
        assert!(matches!(
            refused,
            NotBound::NotThePerson {
                told: 4_242_424,
                ..
            }
        ));
        assert!(!place.socket().exists(), "and nothing was bound");
    }

    /// **The socket goes when the daemon does**, so an ordinary stop leaves no
    /// name for the next start to have to reason about.
    #[test]
    fn stopping_takes_the_socket_with_it() {
        let folder = a_directory_of_our_own("stopping");
        let place = Place::under(&folder);

        let listening = Listening::at(place.clone(), ourselves()).unwrap();
        assert!(place.socket().exists());
        drop(listening);

        assert!(!place.socket().exists());
    }

    /// **A connection is answered with which door it is on**, and with the
    /// kernel's own account of who made it. The client here is this test, so
    /// the person is this process — and the process id that comes back is the
    /// one the operating system knows it by.
    #[test]
    fn a_connection_is_answered_with_the_side_it_is_on() {
        let folder = a_directory_of_our_own("who-is-there");
        let place = Place::under(&folder);
        let listening = Listening::at(place.clone(), ourselves()).unwrap();

        let client = UnixStream::connect(place.socket()).unwrap();
        let accepted = listening.next().unwrap();

        assert_eq!(accepted.side(), Side::Person);
        assert_eq!(
            *accepted.caller(),
            calling_as_the_person(),
            "the process, the user and the group are this process's own"
        );
        drop(client);
    }
}
