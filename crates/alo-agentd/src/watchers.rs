//! **The desktops told what this service's indicator shows.**
//!
//! Law 1: *every network egress an agent causes is visible at the moment it
//! happens.* This service opens the connection to a provider and holds the
//! indicator that shows it (ADR 0125), and the desktop is where a person looks.
//! Until 2026-10-27 nothing carried one to the other, so a question sent to
//! OpenAI left a machine whose screen said nothing (ADR 0134).
//!
//! A desktop asks `watch-what-leaves` on the person's door. That connection is
//! given over to this list and answers nothing else, so the person's door is
//! free again for Settings. From then on, every time the indicator changes,
//! **inside the call that changed it** (`alo_egress::Telling`'s rule), each
//! watcher is written the whole list.
//!
//! # A watcher never holds a departure up
//!
//! Each write is non-blocking. A watcher that cannot take the line at once —
//! one that stopped reading, or went — is dropped rather than waited for: the
//! departure the policy permitted still happens and the record still writes it
//! down, and the desktop sees its connection end, says *cannot tell*, and asks
//! again. A screen that froze must not be a way to stop an agent mid-question,
//! and must never be a way to make it leave unseen either — which is why it is
//! the watcher that goes, not the line.

use std::io::Write as _;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

use alo_egress::{Reported, Shown, Telling};
use alo_protocol::ToAPerson;

/// How many desktops are told at once.
///
/// One per display server a person runs — the direct one, and a nested one on
/// a developer's machine — with room to spare. **The oldest goes** when another
/// arrives, because the newest is the one a person just started.
pub const AT_MOST: usize = 4;

/// The desktops told what this service's indicator shows.
///
/// Shared between the indicator, which tells it, and the serving loop, which
/// hands it connections. Cloning shares the one list.
#[derive(Debug, Clone, Default)]
pub struct Watchers(Arc<Mutex<Watching>>);

/// What the list holds.
#[derive(Debug, Default)]
struct Watching {
    /// The connections, oldest first.
    listening: Vec<UnixStream>,
    /// What the indicator showed when it last changed, so a desktop that
    /// arrives between changes is told the list as it stands.
    last: Vec<Reported>,
}

impl Watchers {
    /// Nobody watching yet.
    #[must_use]
    pub fn none_yet() -> Self {
        Self::default()
    }

    /// What the indicator shows now, as a desktop is told it.
    #[must_use]
    pub fn as_it_stands(&self) -> ToAPerson {
        ToAPerson::Leaving {
            lines: self
                .0
                .lock()
                .map(|watching| watching.last.clone())
                .unwrap_or_default(),
        }
    }

    /// Tell this connection every change from now on.
    ///
    /// It has already been told the list as it stands, as the answer to the
    /// request that asked for this. A connection that cannot be made
    /// non-blocking is dropped: a blocking one could hold a departure up.
    pub fn watching(&self, connection: UnixStream) {
        if connection.set_nonblocking(true).is_err() {
            return;
        }
        if let Ok(mut watching) = self.0.lock() {
            watching.listening.push(connection);
            while watching.listening.len() > AT_MOST {
                watching.listening.remove(0);
            }
        }
    }

    /// How many desktops are being told.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.0
            .lock()
            .map(|watching| watching.listening.len())
            .unwrap_or(0)
    }
}

impl Telling for Watchers {
    fn now_showing(&self, showing: &[Shown]) {
        let lines: Vec<Reported> = showing.iter().map(Reported::of).collect();
        let Ok(mut watching) = self.0.lock() else {
            return;
        };
        let written = ToAPerson::Leaving {
            lines: lines.clone(),
        }
        .written();
        watching.last = lines;
        let Ok(mut written) = written else {
            // Nothing a line on this machine's indicator holds is too long for
            // the wire; a list that could not be written is told to nobody and
            // every desktop then says it cannot tell.
            watching.listening.clear();
            return;
        };
        written.push('\n');
        watching
            .listening
            .retain_mut(|connection| connection.write_all(written.as_bytes()).is_ok());
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "watchers_tests.rs"]
mod tests;
