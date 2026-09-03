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

use std::os::unix::net::UnixStream;
use std::path::Path;

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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

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
