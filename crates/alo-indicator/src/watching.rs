//! **Listening to `alo-agentd` say what it is sending, on the person's own
//! socket and nowhere else.**
//!
//! The one constructor of [`ToldByTheDaemon`] (ADR 0134). It connects to the
//! door the service listens on for this person, asks `watch-what-leaves`, and
//! hands over every list it is told. When the connection ends — the service
//! stopped, restarted, or dropped a desktop that fell behind — it hands over
//! [`FromTheDaemon::Unreachable`] at once, so the screen says *cannot tell*
//! rather than go on showing a list that may have stopped being true, and then
//! asks again.
//!
//! # Who it believes, and the edge it cannot see past
//!
//! Before reading a word it checks two things:
//!
//! - **the directory the socket is in belongs to this person and nobody else
//!   may write in it**, so no other account could have put a socket there;
//! - **the process listening is this person's** (`SO_PEERCRED`) — `alo-agentd`
//!   runs as the person (ADR 0125's table measured it), and anything running
//!   as somebody else is refused.
//!
//! What neither check can tell apart is another program **running as the same
//! person** that removed the service's socket and bound its own. That program
//! could already do anything the person can; the same edge is documented for
//! `alo_egress::Departing`. It is named here rather than smoothed over.
//!
//! # It never blocks a frame
//!
//! It runs on its own thread and hands its answers over a channel the desktop
//! drains once a frame. It stops when the desktop drops the other end.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::fd::AsFd as _;
use std::os::unix::fs::MetadataExt as _;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::Duration;

use alo_protocol::{FromAPerson, ToAPerson};

use crate::told::{FromTheDaemon, ToldByTheDaemon};

/// How long to wait before asking again, at first.
const FIRST_WAIT: Duration = Duration::from_secs(1);
/// The longest wait between two attempts.
const LONGEST_WAIT: Duration = Duration::from_secs(10);

/// Why one attempt at listening ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoppedListening {
    /// Nothing was listening there, or the connection could not be made.
    NotThere,
    /// The directory the socket is in is not the person's alone.
    NotThePersonsAlone,
    /// The process listening is not running as this person.
    NotThePersons,
    /// The service ended the connection, or said something that is not a list.
    Ended,
    /// The desktop is no longer listening for answers.
    NobodyToTell,
}

/// Listen once, at the door at `door`, for `person`.
///
/// Every list it is told is handed to `tell` as [`FromTheDaemon::Told`].
/// Answers why it stopped. A `tell` that answers `false` stops it.
pub fn listened_once(
    door: &Path,
    person: u32,
    tell: &mut dyn FnMut(FromTheDaemon) -> bool,
) -> StoppedListening {
    if !the_persons_alone(door, person) {
        return StoppedListening::NotThePersonsAlone;
    }
    let Ok(mut connection) = UnixStream::connect(door) else {
        return StoppedListening::NotThere;
    };
    match rustix::net::sockopt::socket_peercred(connection.as_fd()) {
        Ok(peer) if peer.uid.as_raw() == person => {}
        _ => return StoppedListening::NotThePersons,
    }
    let Ok(mut asked) = FromAPerson::WatchWhatLeaves.written() else {
        return StoppedListening::Ended;
    };
    asked.push('\n');
    if connection.write_all(asked.as_bytes()).is_err() {
        return StoppedListening::Ended;
    }
    let mut lines = BufReader::new(connection);
    loop {
        let mut line = String::new();
        match lines.read_line(&mut line) {
            Ok(0) | Err(_) => return StoppedListening::Ended,
            Ok(_) => {}
        }
        let Some(told) = ToAPerson::read(&line)
            .ok()
            .and_then(|answer| answer.lines_leaving().map(<[_]>::to_vec))
        else {
            // A refusal, or anything else: the service is not telling this
            // desktop what leaves, so the desktop cannot tell.
            return StoppedListening::Ended;
        };
        if !tell(FromTheDaemon::Told(ToldByTheDaemon::heard(told))) {
            return StoppedListening::NobodyToTell;
        }
    }
}

/// Keep listening for as long as the desktop holds the other end of `send`.
///
/// Every time a connection ends, [`FromTheDaemon::Unreachable`] is handed over
/// before waiting — so a service that stopped is *cannot tell* on the next
/// frame — and the wait grows from one second to ten.
pub fn kept_listening(door: PathBuf, person: u32, send: &Sender<FromTheDaemon>) {
    let mut wait = FIRST_WAIT;
    loop {
        let mut heard_anything = false;
        let stopped = listened_once(&door, person, &mut |told| {
            heard_anything = true;
            send.send(told).is_ok()
        });
        if stopped == StoppedListening::NobodyToTell
            || send.send(FromTheDaemon::Unreachable).is_err()
        {
            return;
        }
        if stopped != StoppedListening::NotThere {
            eprintln!(
                "alo-indicator: alo-agentd stopped telling this desktop what leaves: {stopped:?}"
            );
        }
        wait = if heard_anything {
            FIRST_WAIT
        } else {
            (wait * 2).min(LONGEST_WAIT)
        };
        std::thread::sleep(wait);
    }
}

/// Whether the directory the door is in is the person's alone: theirs, and
/// nobody else may write in it.
fn the_persons_alone(door: &Path, person: u32) -> bool {
    let Some(directory) = door.parent() else {
        return false;
    };
    std::fs::metadata(directory)
        .is_ok_and(|held| held.is_dir() && held.uid() == person && held.mode() & 0o022 == 0)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "watching_tests.rs"]
mod tests;
