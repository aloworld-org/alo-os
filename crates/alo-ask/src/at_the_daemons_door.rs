//! Asking the daemon to put the person's question to their provider.
//!
//! One connection, one line, one answer — the same shape
//! `alo-changing::TheDaemonsDoor` uses on the same socket, and for the same
//! reason: ADR 0017's door is one per signed-in person and the conversation on
//! it is the shortest the protocol has.
//!
//! # Why a command does not ask a provider itself
//!
//! Not law 1. A person's own question goes on no indicator — they chose their
//! provider in Settings
//! ([ADR 0126](../../../docs/decisions/0126-a-persons-own-question-is-a-third-authority-on-the-indicator-and-in-the-record.md))
//! — so this is not about showing anything. It is about two things only the
//! service can reach, measured 2026-10-19:
//!
//! - **the organisation's rule**, read with the checks that guard the file
//!   naming which login is the agent, which is 1710 lines inside `alo-agentd`
//!   and nothing depends on that crate;
//! - **signing in to a proxy that asks who this machine is**, whose password is
//!   a systemd credential belonging to the daemon's unit (ADR 0059).
//!
//! A command that went straight out instead would repeat a fault already found
//! and fixed here: `alo-agentd`'s `the_road_out` records *a machine read its
//! organisation's proxy off the disk and then asked a provider directly.*
//!
//! # A local answer never comes through here
//!
//! `main.rs` answers a model on this machine itself, as it always has: that
//! causes no egress, needs no credential and needs no proxy, so there is
//! nothing about it the service could do that this process cannot. **Two roads,
//! and which one a question takes is decided by the person's settings** rather
//! than by what happens to be running.
//!
//! # Every failure is one sentence, and nothing is retried
//!
//! A daemon that is not there, a connection that drops, an answer that cannot
//! be read: each is [`Unanswered`], each is said once, and none of them is
//! followed by a second attempt anywhere else. An `alo-ask` that fell back to a
//! local model here would be the silent substitution ADR 0008 forbids, arriving
//! as robustness.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_protocol::{FromAPerson, ToAPerson, where_the_door_is};
use alo_strings::{Filling, Said, Strings};

use alo_protocol::words::{NO_SERVICE_TO_ASK_WITH, THE_SERVICE_DID_NOT_ANSWER};

/// How long one write or one read may take before the person is told nobody
/// answered.
///
/// Generous against a busy machine and short against a person watching a
/// terminal that appears to have died. Longer than `alo-changing`'s five
/// seconds, because that one is a courtesy knock and this one is a question
/// going to a provider and back.
const PATIENCE: Duration = Duration::from_secs(120);

/// Why the question was not answered by the service.
///
/// Two, and they are two because they are different things for a person to do:
/// one means start the service, the other means something went wrong while it
/// was running.
#[derive(Debug, PartialEq, Eq)]
pub enum Unanswered {
    /// Nothing is listening at the door.
    NobodyThere,
    /// Something is, and the conversation did not finish.
    NotAnswered,
}

impl Unanswered {
    /// What the person reads, and what to do about it.
    ///
    /// **Both name the next step**, which is the whole point of having two: a
    /// sentence that says only that something went wrong is a sentence that
    /// leaves a person stuck.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let word = match self {
            Self::NobodyThere => NO_SERVICE_TO_ASK_WITH,
            Self::NotAnswered => THE_SERVICE_DID_NOT_ANSWER,
        };
        strings.say(&word.key(), &Filling::nothing())
    }
}

/// The daemon's door, at a path.
///
/// Holds a path and nothing else. The path is this type's to work out — unlike
/// `alo-changing`'s, whose header says *the path is the caller's* because
/// `alo-shell` is its caller. A command a person typed has no caller to hand it
/// one.
#[derive(Debug, Clone)]
pub struct TheDoor {
    /// Where the daemon listens, when one is running.
    socket: PathBuf,
}

impl TheDoor {
    /// This person's own door, from the login number the kernel reports.
    ///
    /// `rustix::process::getuid` cannot fail and takes no argument, so there is
    /// nothing here a caller could get wrong and nothing to check. It is the
    /// same call `alo-agentd` reads its own login number with, so the two
    /// cannot come to disagree about whose door this is.
    #[must_use]
    pub fn for_this_person() -> Self {
        Self {
            socket: where_the_door_is::the_door_of(rustix::process::getuid().as_raw()),
        }
    }

    /// The same door somewhere else, which is a test's.
    ///
    /// Points a **reader** somewhere, never a writer: the worst a wrong path
    /// can do here is say nobody answered.
    #[must_use]
    pub fn at(socket: &Path) -> Self {
        Self {
            socket: socket.to_owned(),
        }
    }

    /// Where this door is, for whoever has to say where they looked.
    #[must_use]
    pub fn where_it_is(&self) -> &Path {
        &self.socket
    }

    /// Put the person's question, and read what comes back.
    ///
    /// # Errors
    ///
    /// [`Unanswered`] for both ways this can fail to happen. A refusal from the
    /// daemon is **not** one of them: that arrives as `ToAPerson::Refused` and
    /// is the service answering.
    pub fn asked(&self, question: &str) -> Result<ToAPerson, Unanswered> {
        let stream = UnixStream::connect(&self.socket).map_err(|_| Unanswered::NobodyThere)?;
        stream
            .set_write_timeout(Some(PATIENCE))
            .map_err(|_| Unanswered::NotAnswered)?;
        stream
            .set_read_timeout(Some(PATIENCE))
            .map_err(|_| Unanswered::NotAnswered)?;

        let line = FromAPerson::TheirOwnQuestion {
            question: question.to_owned(),
        }
        .written()
        .map_err(|_| Unanswered::NotAnswered)?;
        (&stream)
            .write_all(line.as_bytes())
            .map_err(|_| Unanswered::NotAnswered)?;
        (&stream)
            .write_all(b"\n")
            .map_err(|_| Unanswered::NotAnswered)?;

        let mut answer = String::new();
        BufReader::new(&stream)
            .read_line(&mut answer)
            .map_err(|_| Unanswered::NotAnswered)?;
        ToAPerson::read(answer.trim_end()).map_err(|_| Unanswered::NotAnswered)
    }
}
