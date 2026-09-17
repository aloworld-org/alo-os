//! The only file here that asks the kernel anything directly.
//!
//! `alo-agentd`'s and `alo-sessiond`'s rule: what the standard library will not
//! answer on a stable compiler is asked through one rented crate, named in one
//! file. Two questions:
//!
//! - **Who is at the other end of this connection** — `SO_PEERCRED`, the user
//!   the kernel recorded for the process that called `connect`. `std`'s
//!   `peer_cred` is unstable (rust-lang issue #42839) and a `getsockopt` by hand
//!   is `unsafe`, which this workspace forbids.
//! - **Hand this path to a group** — how a socket the broker made becomes one
//!   `alo-agentd` can connect to at all.

use std::os::unix::net::UnixStream;
use std::path::Path;

/// The user the kernel recorded for the process at the other end.
///
/// It takes a connection and not a listening socket, which has no other end to
/// ask about — the type is what keeps that true.
///
/// # Errors
/// Whatever the kernel said. The door treats a caller it could not name as a
/// caller it does not hear.
pub fn the_user_of(connection: &UnixStream) -> Result<u32, std::io::Error> {
    let peer = rustix::net::sockopt::socket_peercred(connection)?;
    Ok(peer.uid.as_raw())
}

/// Hand this path to a group, leaving its owner alone.
///
/// # Errors
/// Whatever the machine said.
pub fn give_to_group(path: &Path, group: u32) -> Result<(), std::io::Error> {
    rustix::fs::chown(path, None, Some(rustix::fs::Gid::from_raw(group)))?;
    Ok(())
}

/// The user this process runs as, which is who a test's own connection is.
#[must_use]
pub fn our_user() -> u32 {
    rustix::process::geteuid().as_raw()
}

/// The group this process runs in.
#[must_use]
pub fn our_group() -> u32 {
    rustix::process::getegid().as_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The kernel answers with the user that connected**, and on a pair made
    /// here that is this process — nothing was written into a message.
    #[test]
    fn the_kernel_names_the_user_that_connected() {
        let pair = UnixStream::pair();
        assert!(pair.is_ok(), "a machine that cannot make a pair of sockets");
        if let Ok((ours, theirs)) = pair {
            assert_eq!(the_user_of(&ours).ok(), Some(our_user()));
            assert_eq!(the_user_of(&theirs).ok(), Some(our_user()));
        }
    }
}
