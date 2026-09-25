//! Asking whether a descriptor really closed, without asking whether it closed
//! *by the next statement*.
//!
//! Two tests in this crate assert that a descriptor the shell handed out has
//! reached end-of-file — that it was closed rather than leaked. Both read the
//! other end of a socket pair and expect zero bytes.
//!
//! **Both were wrong in the same way.** A descriptor is closed by whoever holds
//! the last copy of it, and on this crate's paths that is often somebody else:
//! libinput lets go of its own duplicate during the context's shutdown, and a
//! session device closes inside a scope's unwinding. On a loaded machine the
//! read can arrive before that has happened and be told *not yet* —
//! `WouldBlock` — which is not *still open*. One instance was found in a
//! whole-workspace gate run on 2026-09-23 and the second on 2026-09-25, each
//! having passed dozens of runs of the crate alone.
//!
//! So the read is given a **bounded moment**, in one place rather than in two
//! that would drift. This is not a weaker assertion: a descriptor that is
//! really leaked never reaches end-of-file however long this waits, and the
//! test still fails — it just fails for the reason it was written for.

use std::io::{ErrorKind, Read};

/// How long a close is allowed to take before this is a leak.
///
/// Two seconds, which is far longer than any close on these paths and far
/// shorter than a person waiting on a gate would notice.
const AT_MOST: std::time::Duration = std::time::Duration::from_secs(2);

/// How long to wait between asking again.
const BETWEEN: std::time::Duration = std::time::Duration::from_millis(10);

/// Whether this end of a socket pair has reached end-of-file, waiting for a
/// close that is on its way.
///
/// The peer must already be non-blocking; a blocking read would wait for a
/// byte that is never coming rather than for the close.
pub(crate) fn reached_end_of_file(peer: &mut impl Read) -> std::io::Result<usize> {
    let until = std::time::Instant::now() + AT_MOST;
    loop {
        match peer.read(&mut [0_u8]) {
            Err(again)
                if again.kind() == ErrorKind::WouldBlock && std::time::Instant::now() < until =>
            {
                std::thread::sleep(BETWEEN);
            }
            read => return read,
        }
    }
}
