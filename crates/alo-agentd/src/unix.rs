//! The only file that asks the kernel anything directly.
//!
//! `ollama.rs` is the only file in `alo-models` that knows Ollama exists, and
//! this is the same rule about a different rented thing: everything else here
//! asks *this* file who is calling, and never the machine.
//!
//! # Why there is a rented crate at all
//!
//! `std::os::unix::net::UnixStream::peer_cred` exists and is **unstable** —
//! feature `peer_credentials_unix_socket`, rust-lang issue #42839 — so on a
//! stable compiler the standard library will not answer this question. The two
//! honest ways past that are a `getsockopt` written out by hand, which needs
//! `unsafe` and `CLAUDE.md` forbids it workspace-wide, or a crate that has
//! already written it. A daemon that only builds on a nightly compiler is a
//! daemon nobody else can build, so it is a crate.
//!
//! `rustix` is the one, and it is named here and nowhere else. If it is ever
//! replaced, this file is the change.
//!
//! # What `SO_PEERCRED` is worth
//!
//! It is the process, user and group of the peer **as the kernel recorded them
//! when the connection was made**. It is not a field in a message, not a
//! header, and not something a client can set: a caller cannot lie about it
//! any more than it can lie about which socket it connected to. That is the
//! whole property [`crate::Sides`] stands on, and it is why the division
//! between an agent's door and a person's is a division rather than a
//! convention.
//!
//! It answers about the process that called `connect`, at the moment it called
//! it. A process that has since exited, or been replaced by something else at
//! the same process id, does not change the answer — which is one more reason
//! [`crate::Caller`] never decides anything on a process id.
//!
//! # And the second question: which of these has something on it
//!
//! `ready` is `poll`, and it is here for the same reason [`who`] is: the
//! standard library has no way to wait on more than one thing at once, and
//! [`crate::Serving`] must wait on the socket, on both connections and on the
//! end a stop arrives on **at the same time**. Waiting on them one after
//! another is the deadlock item 21c cut this decision out of itself rather than
//! take in a hurry — an agent waiting to be told what happened, and the message
//! that would tell it arriving on a connection nobody is reading.
//!
//! It waits with no timeout, so a quiet machine costs nothing at all: the
//! process sleeps in one call until somebody says something, and there is no
//! interval on which it wakes up to discover that nobody has.
//!
//! # And the third: open this, and not whatever it points at
//!
//! `open_not_a_link` is here for the same reason again. `File::open` follows a
//! symbolic link, and a service that looked at a path and then opened it would
//! be answering about two different files whenever somebody could replace the
//! one in between. `O_NOFOLLOW` is how a kernel is asked for both answers at
//! once, and the standard library has no spelling for it either.

use std::os::fd::BorrowedFd;
use std::os::unix::net::UnixStream;
use std::path::Path;

use rustix::event::{PollFd, PollFlags};

use crate::caller::{Caller, Gid, Uid};
use crate::refusing::{NotACaller, NotAUser};

/// The user this process is running as.
///
/// The one thing in this crate that asks about *us* rather than about somebody
/// calling. [`crate::Listening::at`] uses it to refuse a machine whose
/// configuration names a person this process is not, and the daemon itself will
/// use it to refuse to run as root at all (ADR 0001 §2).
///
/// # Errors
///
/// [`NotAUser::NoSuchUser`] cannot arise from a working kernel — `geteuid`
/// always answers with a real user — and is carried rather than assumed away,
/// because the alternative is a `-1` travelling into a `chown` on the strength
/// of a comment.
pub fn us() -> Result<Uid, NotAUser> {
    Uid::of(rustix::process::geteuid().as_raw())
}

/// The group this process is running as.
///
/// Beside [`us`], and for the same readers: a daemon reporting what it is about
/// to do with a socket, and a test that has to name a group this process is
/// really in — handing a path to a group nobody here is a member of is refused
/// by the machine, correctly, and a fixture that did it would be testing the
/// refusal by accident.
///
/// # Errors
///
/// [`NotAUser::NoSuchGroup`], which a working kernel does not answer with. See
/// [`us`].
pub fn our_group() -> Result<Gid, NotAUser> {
    Gid::of(rustix::process::getegid().as_raw())
}

/// Who the kernel says is at the other end of this connection.
///
/// **It takes a connection and not a socket**, and the type is the whole of how
/// that is kept true: `SO_PEERCRED` asked of a socket that has no peer answers
/// with process `0`, and the crate this file rents holds that process id in a
/// non-zero integer. A listening socket is not a connection, and there is no
/// door here to hand it through. `docs/quirks.md` records it.
///
/// # Errors
///
/// [`NotACaller::NotAsked`] if the kernel would not answer, and
/// [`NotACaller::NotAUser`] if what it answered with is not a user. Neither is
/// a caller that gets served: who is calling is the whole of what the door
/// decides on, so not knowing is not a lesser answer than knowing.
pub fn who(connection: &UnixStream) -> Result<Caller, NotACaller> {
    let peer = rustix::net::sockopt::socket_peercred(connection)
        .map_err(|why| NotACaller::NotAsked { why: why.into() })?;
    Ok(Caller::known(
        peer.pid.as_raw_nonzero().get(),
        Uid::of(peer.uid.as_raw())?,
        Gid::of(peer.gid.as_raw())?,
    ))
}

/// Hand this path to a group, leaving its owner alone.
///
/// The agent is a different user from the person, so a socket that only its
/// owner could reach would be a socket the agent could never connect to. The
/// group is how it reaches the path; [`crate::place`] is what makes sure that
/// is the *only* thing reaching it.
///
/// # Errors
///
/// Whatever the machine said, as a `std::io::Error`. The usual one is that the
/// person is not a member of that group, which nothing here can fix.
pub(crate) fn give_to_group(path: &Path, group: Gid) -> Result<(), std::io::Error> {
    rustix::fs::chown(path, None, Some(rustix::fs::Gid::from_raw(group.raw())))
        .map_err(std::io::Error::from)
}

/// Open this path, and refuse it if the last part of it is a symbolic link.
///
/// The third thing the standard library will not do here: `File::open` follows
/// a link, and looking first and opening afterwards is two answers about a path
/// that could have been replaced in between. `O_NOFOLLOW` makes the kernel
/// answer both questions at once — what is opened is what was looked at, or
/// nothing is opened at all.
///
/// It refuses **the last part of the path only**, which is the same limit
/// [`crate::place`] has: a link somewhere along the way is still followed, and
/// what protects against that is who owns the directories, not this call.
///
/// # Errors
///
/// [`NotOpened::ALink`] when the kernel said it was a link, and
/// [`NotOpened::Machine`] for everything else. The two are told apart **here**
/// rather than by the caller, because what the kernel answers with is `ELOOP`
/// and `std::io::ErrorKind` has no stable spelling for it — so a caller would be
/// left comparing a raw number, which is exactly the kind of knowledge this file
/// exists to keep in one place.
pub(crate) fn open_not_a_link(path: &Path) -> Result<std::fs::File, NotOpened> {
    match rustix::fs::open(
        path,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NOFOLLOW,
        rustix::fs::Mode::empty(),
    ) {
        Ok(opened) => Ok(std::fs::File::from(opened)),
        Err(rustix::io::Errno::LOOP) => Err(NotOpened::ALink),
        Err(why) => Err(NotOpened::Machine(std::io::Error::from(why))),
    }
}

/// Why [`open_not_a_link`] opened nothing.
#[derive(Debug)]
pub(crate) enum NotOpened {
    /// The last part of the path is a symbolic link.
    ALink,
    /// The machine would not open it, for any other reason.
    Machine(std::io::Error),
}

/// Sleep until one of these has something on it, and say which ones do.
///
/// The answer is one `bool` per place asked about, in the order they were
/// asked, so a caller reads it by destructuring rather than by matching an
/// index to a meaning. A `None` is something this service is not holding at the
/// moment — no connection on that door — and is never ready.
///
/// **A connection that has been closed is ready**, not quiet. The read that
/// follows answers with nothing, which is how the end of a connection is
/// noticed at all; a `poll` that hid a hangup would leave a service asleep
/// beside a caller that has gone, holding a turn nothing can end.
///
/// **An interrupted wait is resumed rather than reported.** A signal arriving
/// while this sleeps ends the call with `EINTR`, and on this machine the signal
/// *is* the ordinary way a stop arrives (see [`crate::stopping`]) — so treating
/// it as a failure would turn the intended shutdown into a service that says it
/// broke. The byte the handler wrote is still on the socket, and the wait that
/// resumes finds it.
///
/// # Errors
///
/// Whatever the machine said, as a `std::io::Error`. `InvalidInput` when there
/// is nothing at all to wait on: a wait nothing can end is a hang, and a
/// service asking for one is a bug in the service rather than a quiet machine.
pub(crate) fn ready<const N: usize>(
    waiting_on: &[Option<BorrowedFd<'_>>; N],
) -> Result<[bool; N], std::io::Error> {
    let asking: Vec<usize> = waiting_on
        .iter()
        .enumerate()
        .filter_map(|(which, held)| held.map(|_| which))
        .collect();
    let mut polling: Vec<PollFd<'_>> = waiting_on
        .iter()
        .flatten()
        .map(|fd| PollFd::from_borrowed_fd(*fd, PollFlags::IN))
        .collect();
    if polling.is_empty() {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }

    loop {
        match rustix::event::poll(&mut polling, None) {
            Ok(_) => break,
            Err(rustix::io::Errno::INTR) => {
                for polled in &mut polling {
                    polled.clear_revents();
                }
            }
            Err(why) => return Err(std::io::Error::from(why)),
        }
    }

    let mut answered = [false; N];
    for (which, polled) in asking.iter().zip(polling.iter()) {
        if let Some(said) = answered.get_mut(*which) {
            *said = !polled.revents().is_empty();
        }
    }
    Ok(answered)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::io::Write as _;
    use std::os::fd::AsFd as _;

    /// **The credentials are the kernel's, and they are this process's own.**
    /// A pair of connected sockets is made, and what comes back is the process
    /// id and the user this test is really running as — not anything the test
    /// wrote into a message, because there is no message.
    #[test]
    fn the_kernel_answers_with_the_process_that_connected() {
        let (ours, _theirs) = UnixStream::pair().unwrap();
        let caller = who(&ours).unwrap();

        let mine = i32::try_from(std::process::id()).unwrap();
        assert_eq!(caller.process(), mine);
        assert_eq!(caller.user(), us().unwrap());
        assert_eq!(caller.group().raw(), rustix::process::getegid().as_raw());
    }

    /// Both ends of one connection see each other, and on a pair made by one
    /// process both answers are that process. This is the same question the
    /// door asks, asked from the other side.
    #[test]
    fn both_ends_are_asked_the_same_way() {
        let (ours, theirs) = UnixStream::pair().unwrap();
        assert_eq!(who(&ours).unwrap(), who(&theirs).unwrap());
    }

    /// **Only the one that was spoken on is ready**, and the answer is in the
    /// order the places were asked about — which is what lets a service read it
    /// by destructuring rather than by keeping an index and a meaning in step.
    #[test]
    fn only_the_place_something_arrived_on_is_ready() {
        let (quiet, _quiet_other_end) = UnixStream::pair().unwrap();
        let (spoken, mut speaking) = UnixStream::pair().unwrap();
        speaking.write_all(b"something").unwrap();

        assert_eq!(
            ready(&[Some(quiet.as_fd()), Some(spoken.as_fd())]).unwrap(),
            [false, true]
        );
    }

    /// **A door this service is not holding is never ready**, and it does not
    /// move the others along: the `None` keeps its place in the answer, so a
    /// service with no agent connected reads the same positions it reads with
    /// one.
    #[test]
    fn a_place_nothing_is_held_on_keeps_its_place_and_is_quiet() {
        let (spoken, mut speaking) = UnixStream::pair().unwrap();
        speaking.write_all(b"something").unwrap();

        assert_eq!(
            ready(&[None, Some(spoken.as_fd()), None]).unwrap(),
            [false, true, false]
        );
    }

    /// **A caller that has gone is ready, not quiet.** The read that follows
    /// answers with nothing, which is the only way the end of a connection is
    /// noticed — and a wait that treated a hangup as silence would sleep beside
    /// a caller that has gone, holding a turn nothing could end.
    #[test]
    fn a_connection_whose_caller_has_gone_is_ready() {
        let (ours, theirs) = UnixStream::pair().unwrap();
        drop(theirs);

        assert_eq!(ready(&[Some(ours.as_fd())]).unwrap(), [true]);
    }

    /// **A wait with nothing to wait on is refused rather than slept in.**
    /// `poll` with no descriptors and no timeout sleeps until the machine is
    /// turned off, so the one thing that must not happen here is that it
    /// succeeds quietly.
    #[test]
    fn waiting_on_nothing_at_all_is_refused() {
        let refused = ready(&[None::<BorrowedFd<'_>>; 3]).unwrap_err();
        assert_eq!(refused.kind(), std::io::ErrorKind::InvalidInput);
    }

    /// Handing a path to the group it is already in changes nothing and
    /// refuses nothing — the ordinary case on a machine set up correctly.
    #[test]
    fn a_path_can_be_handed_to_the_group_it_is_in() {
        let folder = std::env::temp_dir().join(format!("alo-group-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();

        let ours = Gid::of(rustix::process::getegid().as_raw()).unwrap();
        give_to_group(&folder, ours).unwrap();

        let _ = std::fs::remove_dir(&folder);
    }
}
