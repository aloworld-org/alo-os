//! The real daemon's door: one connection, one line, one answer.
//!
//! Three conversations, each that shortest one: the knock after a grant's
//! file is written, `revoke-pairing` for a row of the one list that is a
//! pairing, and `pairings` for the rows themselves. The last two exist because
//! the pairings file is the daemon's alone — the person's side asks, and never
//! writes it.
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

use alo_nearby::MachineId;
use alo_protocol::{AfterRevoking, FromAPerson, ToAPerson};

use crate::knocking::Knocking;
use crate::seen_pairing::SeenPairing;
use crate::stood::Stood;
use crate::unpairing::{RevokingPairings, Unpaired};

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

    /// Every pairing the running daemon has, as rows of the one list — or
    /// [`None`] when no daemon gave that answer, and a surface reads
    /// `alo_remembering::pairings_remembered` through
    /// [`SeenPairing::remembered`] instead.
    ///
    /// A pairing the daemon names by something that is not a machine identity
    /// is left out rather than listed: it is not a row anybody could revoke.
    #[must_use]
    pub fn pairings(&self) -> Option<Vec<SeenPairing>> {
        let answer = self.asked(&FromAPerson::Pairings).ok()?;
        Some(
            answer
                .paired()?
                .iter()
                .filter_map(SeenPairing::told)
                .collect(),
        )
    }

    /// One line out and one line back.
    fn asked(&self, request: &FromAPerson) -> Result<ToAPerson, Unheard> {
        let stream = UnixStream::connect(&self.socket).map_err(|_| Unheard::NobodyThere)?;
        stream
            .set_write_timeout(Some(PATIENCE))
            .map_err(|_| Unheard::NotAnswered)?;
        stream
            .set_read_timeout(Some(PATIENCE))
            .map_err(|_| Unheard::NotAnswered)?;

        let line = request.written().map_err(|_| Unheard::NotAnswered)?;
        (&stream)
            .write_all(line.as_bytes())
            .map_err(|_| Unheard::NotAnswered)?;
        (&stream)
            .write_all(b"\n")
            .map_err(|_| Unheard::NotAnswered)?;

        let mut answer = String::new();
        BufReader::new(&stream)
            .read_line(&mut answer)
            .map_err(|_| Unheard::NotAnswered)?;
        ToAPerson::read(answer.trim_end()).map_err(|_| Unheard::NotAnswered)
    }

    /// The knock, or [`None`] for every way it can fail to happen — which
    /// the caller reads as the next sign-in.
    fn answered(&self) -> Option<Stood> {
        match self.asked(&FromAPerson::Granted).ok()? {
            ToAPerson::Granted { holding } => Some(Stood::Heard { holding }),
            ToAPerson::Refused(wording) => Some(Stood::TurnedAway {
                told: wording.text().to_owned(),
            }),
            // Any other answer is not one this request can earn; a daemon
            // saying something unrecognisable is treated like no daemon,
            // because the change already stands and the next start reads it.
            ToAPerson::Did(_)
            | ToAPerson::Waiting { .. }
            | ToAPerson::Declined
            | ToAPerson::Pairing(_)
            | ToAPerson::Confirmed { .. }
            | ToAPerson::Revoked { .. }
            | ToAPerson::Pairings { .. }
            | ToAPerson::ChosenToAnswer { .. }
            | ToAPerson::MachineNamed { .. }
            | ToAPerson::Workspaces { .. }
            | ToAPerson::WorkspaceOpened(_)
            | ToAPerson::Advertised(_) => None,
        }
    }
}

/// The two ways a conversation fails to be one, told apart because for a
/// pairing they mean different things: nobody there means nothing was asked;
/// a door that was reached and never answered means nobody knows.
enum Unheard {
    /// The socket would not connect.
    NobodyThere,
    /// Connected, and no readable answer came back within the patience.
    NotAnswered,
}

impl RevokingPairings for TheDaemonsDoor {
    /// Ask over `revoke-pairing`, and carry back what the daemon said: its
    /// outcome, its refusal in its own words, or which way the conversation
    /// failed — never a guess dressed as an answer.
    fn revoke_pairing(&self, with: &MachineId) -> Unpaired {
        let request = FromAPerson::RevokePairing {
            machine: with.as_str().to_owned(),
        };
        match self.asked(&request) {
            Err(Unheard::NobodyThere) => Unpaired::NobodyThere,
            Err(Unheard::NotAnswered) => Unpaired::NotAnswered,
            Ok(ToAPerson::Revoked {
                became: AfterRevoking::Revoked,
            }) => Unpaired::Revoked,
            Ok(ToAPerson::Revoked {
                became: AfterRevoking::RevokedUntilARestart,
            }) => Unpaired::RevokedUntilARestart,
            Ok(ToAPerson::Refused(wording)) => Unpaired::Refused {
                told: wording.text().to_owned(),
            },
            // Any other answer is not one this request can earn, and says
            // nothing about whether the pairing is gone.
            Ok(_) => Unpaired::NotAnswered,
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
