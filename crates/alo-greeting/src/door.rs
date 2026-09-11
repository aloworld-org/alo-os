//! The opener's door as a real socket: one connection, one line, one answer.
//!
//! The socket is the one `alo-sessiond` binds at
//! `/run/alo-sessiond/sign-in.sock`, and the conversation is the shortest one
//! this repository has — `open <number>` out, `opened` or `refused <key>` back.
//! Both words are that crate's (`alo_sessiond::Knock`, `Answered`), read and
//! written by its own code: a second reader of one line is two accounts of one
//! conversation, and one of the two is a privileged process's.
//!
//! # Nothing here is silent
//!
//! Every way this can fail is a [`NotAnswered`] with a sentence behind it, and
//! that is the whole reason this file is not four lines of `ok()?`. Somebody
//! standing at a sign-in that did nothing cannot tell *your password was
//! refused* from *the service that opens sessions is not running*, and those
//! two call for opposite things — so a connection that could not be made, a
//! knock that could not be delivered, an answer that never came and a line
//! that is not an answer are four values here and two sentences at the screen.
//!
//! # The patience is bounded, and it is the door's own
//!
//! [`PATIENCE`] matches `alo_sessiond::listening::PATIENCE`: the far side gives
//! itself that long to read and to write, so anything shorter here would call a
//! door slow that is answering exactly as fast as it is allowed to. It is long
//! against a machine under load at boot and short against a person watching a
//! screen that appears to have died.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_sessiond::{Answered, Knock};

use crate::knocking::Knocking;
use crate::refusing::NotAnswered;

/// How long one write or one read at the door may take.
///
/// The door's own patience. See this module's header.
pub const PATIENCE: Duration = Duration::from_secs(10);

/// The door `alo-sessiond` listens at, knocked on.
///
/// Holds a path and nothing else. The path is the caller's — `alo-sessiond`'s
/// own `THE_DOOR` is where it is on a machine — because a rule about a socket
/// is only a rule with a test if a test can bind one somewhere it may write,
/// and `/run` is not that. It is the argument `alo_changing::TheDaemonsDoor`
/// makes one crate along.
#[derive(Debug, Clone)]
pub struct TheOpenersDoor {
    /// Where the opener listens, when it is running.
    at: PathBuf,
}

impl TheOpenersDoor {
    /// The door at this socket.
    #[must_use]
    pub fn at(socket: &Path) -> Self {
        Self {
            at: socket.to_owned(),
        }
    }

    /// The door where a machine really has one.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::at(Path::new(alo_sessiond::THE_DOOR))
    }

    /// Where this door is.
    #[must_use]
    pub fn socket(&self) -> &Path {
        &self.at
    }

    /// The path, for a refusal to name.
    fn named(&self) -> String {
        self.at.display().to_string()
    }
}

impl Knocking for TheOpenersDoor {
    /// The whole conversation, with each way it can fail to happen told apart
    /// from the others.
    fn knock(&self, knock: Knock) -> Result<Answered, NotAnswered> {
        let connection = UnixStream::connect(&self.at).map_err(|why| NotAnswered::NobodyThere {
            at: self.named(),
            why: why.to_string(),
        })?;
        for bounded in [
            connection.set_write_timeout(Some(PATIENCE)),
            connection.set_read_timeout(Some(PATIENCE)),
        ] {
            bounded.map_err(|why| NotAnswered::WentAway {
                at: self.named(),
                why: why.to_string(),
            })?;
        }

        let said = format!("{}\n", knock.written());
        (&connection)
            .write_all(said.as_bytes())
            .map_err(|why| NotAnswered::WentAway {
                at: self.named(),
                why: why.to_string(),
            })?;

        let mut back = String::new();
        BufReader::new(&connection)
            .read_line(&mut back)
            .map_err(|why| NotAnswered::NothingCameBack {
                at: self.named(),
                why: why.to_string(),
            })?;
        if back.trim_end_matches(['\r', '\n']).is_empty() {
            // Nothing at all came back, which is what a door that closed the
            // connection without answering looks like. It is not an unreadable
            // line — there is no line — and saying so is what sends whoever
            // maintains the machine at the service rather than at its wire.
            return Err(NotAnswered::NothingCameBack {
                at: self.named(),
                why: "the door closed without answering".to_owned(),
            });
        }
        Answered::read(back.trim_end_matches(['\r', '\n'])).map_err(|why| {
            NotAnswered::NotAnAnswer {
                at: self.named(),
                why: why.to_string(),
            }
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A door that is not there is told apart from a door that refused.**
    /// The one failure this file can show without standing a service up, and
    /// the one a person meets on a machine whose opener is not running.
    #[test]
    fn nothing_listening_is_its_own_answer() {
        let nowhere = std::env::temp_dir().join(format!(
            "alo-greeting-nothing-here-{}.sock",
            std::process::id()
        ));
        drop(std::fs::remove_file(&nowhere));

        let door = TheOpenersDoor::at(&nowhere);
        let refused = door.knock(Knock::on_behalf_of(1000)).unwrap_err();
        assert!(
            matches!(refused, NotAnswered::NobodyThere { .. }),
            "{refused:?}"
        );
        assert_eq!(refused.at(), nowhere.display().to_string());
    }

    /// The door a machine really has is `alo-sessiond`'s own constant, not a
    /// second spelling of it.
    #[test]
    fn the_machines_door_is_the_one_the_opener_binds() {
        assert_eq!(
            TheOpenersDoor::on_this_machine().socket(),
            Path::new(alo_sessiond::THE_DOOR)
        );
    }

    /// The patience is the door's own, so this side never calls a door slow
    /// that is answering exactly as fast as it is allowed to.
    #[test]
    fn the_patience_is_the_doors_own() {
        assert_eq!(PATIENCE, alo_sessiond::listening::PATIENCE);
    }
}
