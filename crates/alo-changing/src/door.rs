//! The real daemon's door: one connection, one line, one answer.
//!
//! The socket is the one `alo-agentd` binds at `/run/alo/<uid>/agentd.sock`
//! (ADR 0017), and the conversation is the shortest one the protocol has: the
//! knock goes out as `alo_protocol::FromAPerson::Granted` — carrying nothing,
//! which is the whole design — and what comes back is either the daemon's
//! count of what is in force now, or the daemon's own sentence about why it
//! kept the list it had.
//!
//! # Every failure is the next sign-in
//!
//! Nobody listening, a connection that drops, an answer that never comes or
//! cannot be read — all of them are [`Stood::AtTheNextSignIn`], and none of
//! them is an error, because by the time this file runs **the change is
//! already on the disk**. A daemon that never heard the knock reads the file
//! at its next start exactly as it read it before the knock existed. The one
//! answer that is not flattened this way is the daemon saying, in words, that
//! it could not read the list again: that daemon is running and serving under
//! the old list, which is the one situation a person must not be told looks
//! like any other ([`Stood::TurnedAway`]).
//!
//! # The patience is bounded
//!
//! A knock is a courtesy to a running daemon, not something the person's
//! surface may hang on. Reading and writing are each given `PATIENCE`; a
//! daemon that cannot answer within it is answered by the next sign-in, like
//! a daemon that is not there. The bound is generous against a busy machine
//! and short against a person watching a surface that appears to have died.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_protocol::{FromAPerson, ToAPerson};

use crate::knocking::Knocking;
use crate::stood::Stood;

/// How long one write or one read at the door may take before the change is
/// left for the next sign-in.
const PATIENCE: Duration = Duration::from_secs(5);

/// The socket this machine's agent service listens on, knocked on.
///
/// Holds a path and nothing else. The path is the caller's — the shape is
/// ADR 0017's and lives in `alo-agentd`'s own `Place` — because a rule about
/// a socket is only a rule with a test if a test can bind one somewhere it
/// may write, and `/run` is not that.
#[derive(Debug, Clone)]
pub struct TheDaemonsDoor {
    /// Where the daemon listens, when one is running.
    socket: PathBuf,
}

impl TheDaemonsDoor {
    /// The door at this socket.
    #[must_use]
    pub fn at(socket: &Path) -> Self {
        Self {
            socket: socket.to_owned(),
        }
    }

    /// Where this door is.
    #[must_use]
    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// The whole conversation, or [`None`] for every way it can fail to
    /// happen — which the caller reads as the next sign-in.
    fn answered(&self) -> Option<Stood> {
        let stream = UnixStream::connect(&self.socket).ok()?;
        stream.set_write_timeout(Some(PATIENCE)).ok()?;
        stream.set_read_timeout(Some(PATIENCE)).ok()?;

        let line = FromAPerson::Granted.written().ok()?;
        (&stream).write_all(line.as_bytes()).ok()?;
        (&stream).write_all(b"\n").ok()?;

        let mut answer = String::new();
        BufReader::new(&stream).read_line(&mut answer).ok()?;
        match ToAPerson::read(answer.trim_end()).ok()? {
            ToAPerson::Granted { holding } => Some(Stood::Heard { holding }),
            ToAPerson::Refused(wording) => Some(Stood::TurnedAway {
                told: wording.text().to_owned(),
            }),
            // Any other answer is not one this request can earn; a daemon
            // saying something unrecognisable is treated like no daemon,
            // because the change already stands and the next start reads it.
            ToAPerson::Did(_) | ToAPerson::Waiting { .. } | ToAPerson::Declined => None,
        }
    }
}

impl Knocking for TheDaemonsDoor {
    /// Knock, and read every way the conversation can fail as the next
    /// sign-in — see this module's header for why that is the honest floor.
    fn knock(&self) -> Stood {
        self.answered().unwrap_or(Stood::AtTheNextSignIn)
    }
}
