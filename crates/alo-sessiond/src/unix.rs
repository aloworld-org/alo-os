//! The only file here that asks the kernel anything directly.
//!
//! The rule `alo-agentd`'s own `unix.rs` set and `alo-boundaryd`'s kept: what
//! the standard library will not answer on a stable compiler is asked through
//! one rented crate, named in one file, so that replacing it is one change.
//!
//! Three questions, and each is one line:
//!
//! - **Which group is this process in**, which is the group its unit file put
//!   it in and the group the door is handed to. `getegid` has no safe spelling
//!   in `std`.
//! - **Who is at the other end of this connection**, which is `SO_PEERCRED` —
//!   what the kernel recorded about the process that called `connect`, not a
//!   field in a message. `std`'s `peer_cred` is unstable (feature
//!   `peer_credentials_unix_socket`, rust-lang issue #42839), and the honest
//!   alternatives are a `getsockopt` written by hand, which is `unsafe` and
//!   forbidden workspace-wide, or a crate that has already written it.
//! - **Hand this path to a group**, which is how a socket root created becomes
//!   a socket the sign-in surface can connect to.
//!
//! # The handover needs no capability, and that is the point
//!
//! `chown` changing only the group is allowed to the owner of a file when the
//! group is one the process is already in — and this process's group **is** the
//! door's, because its unit says so. So the one privileged-looking thing this
//! component does to the filesystem needs no `CAP_CHOWN`, and its unit can say
//! `CapabilityBoundingSet=` with nothing after it and mean it.

use std::os::unix::net::UnixStream;
use std::path::Path;

/// The group this process is in, which is the group its unit file named.
///
/// The *effective* one rather than the real one, because that is what a newly
/// created file is given and what the kernel decides a `chown` by.
pub fn our_group() -> u32 {
    rustix::process::getegid().as_raw()
}

/// The group the kernel recorded for the process at the other end.
///
/// **It takes a connection and not a socket.** `SO_PEERCRED` asked of a
/// listening socket answers about nothing, and there is no door here to hand
/// one through — the type is what keeps that true, exactly as in `alo-agentd`.
///
/// # Errors
///
/// Whatever the kernel said, as a `std::io::Error`. A caller whose group could
/// not be asked is not a caller that gets served: who is knocking is the whole
/// of what this door decides on, so not knowing is not a lesser answer than
/// knowing.
pub fn the_group_of(connection: &UnixStream) -> Result<u32, std::io::Error> {
    let peer = rustix::net::sockopt::socket_peercred(connection)?;
    Ok(peer.gid.as_raw())
}

/// Hand this path to a group, leaving its owner alone.
///
/// # Errors
///
/// Whatever the machine said. On a correctly written unit it cannot refuse:
/// the group is this process's own, and changing a file's group to one the
/// process is in is allowed to the file's owner.
pub fn give_to_group(path: &Path, group: u32) -> Result<(), std::io::Error> {
    rustix::fs::chown(path, None, Some(rustix::fs::Gid::from_raw(group)))?;
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **The answer is the same every time it is asked**, which is what makes
    /// asking once at start-up honest: the door is handed to what this reads,
    /// and a number that changed afterwards would be a door handed to a group
    /// nobody checks against.
    ///
    /// Which number it is belongs to whoever started the process, so a test
    /// that asserted one would be a test of the machine it ran on.
    #[test]
    fn which_group_this_process_is_in_does_not_change_between_two_questions() {
        assert_eq!(our_group(), our_group());
    }

    /// **The credentials are the kernel's, and on a pair made here they are
    /// this process's own.** Nothing was written into a message, because there
    /// is no message — which is the property the whole door stands on.
    #[test]
    fn the_kernel_answers_with_the_group_that_connected() {
        let Ok((ours, theirs)) = UnixStream::pair() else {
            panic!("a machine that cannot make a pair of sockets");
        };
        assert_eq!(the_group_of(&ours).ok(), Some(our_group()));
        assert_eq!(the_group_of(&theirs).ok(), Some(our_group()));
    }

    /// Handing a path to the group it is already in changes nothing and refuses
    /// nothing — the ordinary case on a machine set up correctly, and the one
    /// this process really performs.
    #[test]
    fn a_path_can_be_handed_to_the_group_it_is_in() {
        let at = std::env::temp_dir().join(format!("alo-sessiond-group-{}", std::process::id()));
        let Ok(()) = std::fs::write(&at, b"") else {
            panic!("a machine that cannot write a temporary file");
        };

        assert!(give_to_group(&at, our_group()).is_ok());

        drop(std::fs::remove_file(&at));
    }
}
