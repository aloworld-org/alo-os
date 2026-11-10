//! Handing a person's own words to the agent, which is a different door from
//! the daemon's.
//!
//! [ADR 0109](../../../docs/decisions/0109-a-persons-words-reach-the-agent-through-a-socket-systemd-opens-as-the-agent.md):
//! `alo-asking.socket` has `Accept=yes` and starts `alo-asking@.service` as
//! `User=alo-agent` for each connection. What goes in is **one line a person
//! wrote**, and what comes back is whatever the agent has to say about it. No
//! framing, no verb names, no arguments — those are the agent's half of
//! `docs/contracts/daemon-protocol.md`, spoken on the daemon's socket by the
//! agent and never by this command.
//!
//! # Why a command does not call a verb itself
//!
//! Because it is not an agent, and the two doors are told apart by
//! `SO_PEERCRED` rather than by what a caller claims. A command that built a
//! `Propose` and wrote it to the daemon would be refused, and should be: the
//! thing that decides a sentence means `open_application` is the agent, with the
//! machine's verbs in front of it, and the thing that decides it happens is the
//! person approving the proposal it makes.
//!
//! **So nothing here approves anything.** This writes a sentence and reads what
//! came back. ADR 0124: *the classifier is not what protects the person — the
//! approval is*, and the approval is the daemon's to put and the person's to
//! give.
//!
//! # Every failure is one sentence, and nothing is retried
//!
//! The same rule `crate::at_the_daemons_door` keeps, for the same reason: a
//! command that fell back to answering a request it could not act on would be
//! the silent substitution ADR 0008 forbids, arriving as robustness.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

/// How long one write or one read may take before the person is told nobody
/// answered.
///
/// Longer than the daemon's door allows, deliberately: behind this one a model
/// is asked and a machine's verbs are looked at, where behind that one a line is
/// parsed. `alo-shell`'s own door to this socket waits the same.
const PATIENCE: Duration = Duration::from_secs(330);

/// Why the agent said nothing.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NothingCameBack {
    /// Nothing is listening where the agent's door should be.
    #[error("nothing is listening at {at}, so alo cannot be asked to do anything")]
    NobodyThere {
        /// Where this knocked.
        at: String,
    },
    /// The connection was made and then would not carry a sentence.
    #[error("the agent's door would not take what you typed: {why}")]
    WouldNotTake {
        /// What the machine said.
        why: String,
    },
    /// It closed without writing a line.
    ///
    /// **Told apart from an empty answer on purpose**: a door that closes is a
    /// machine that would not serve this caller, and a person reads that
    /// differently from an agent that had nothing to say.
    #[error("the agent closed without answering")]
    ClosedWithoutAnswering,
}

/// Hand one sentence to the agent and wait for what it says.
///
/// `at` is the socket; `crate::where_a_persons_words_go` names it on a real
/// machine and a test passes its own.
///
/// # Errors
/// [`NothingCameBack`], one sentence each, none of them retried.
pub fn what_the_agent_says(at: &Path, typed: &str) -> Result<String, NothingCameBack> {
    let mut talking = UnixStream::connect(at).map_err(|_| NothingCameBack::NobodyThere {
        at: at.display().to_string(),
    })?;
    let fault = |why: std::io::Error| NothingCameBack::WouldNotTake {
        why: why.to_string(),
    };
    talking.set_read_timeout(Some(PATIENCE)).map_err(fault)?;
    talking.set_write_timeout(Some(PATIENCE)).map_err(fault)?;

    // **One line, and this end then shuts for writing.** Without it an agent
    // reading to the end of its input waits for this side to go away while this
    // side waits for the agent, and both wait for ever. `alo-shell`'s door to
    // the same socket does the same and says so.
    let one_line: String = typed
        .chars()
        .map(|it| if it == '\n' || it == '\r' { ' ' } else { it })
        .collect();
    talking
        .write_all(format!("{one_line}\n").as_bytes())
        .and_then(|()| talking.flush())
        .map_err(fault)?;
    drop(talking.shutdown(std::net::Shutdown::Write));

    let mut heard = String::new();
    let mut reading = BufReader::new(&talking);
    loop {
        let mut line = String::new();
        match reading.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => heard.push_str(&line),
            Err(why) => return Err(fault(why)),
        }
    }
    // Only its end: the start of the first line is what keeps a model's words
    // from reading as a change waiting (`alo_protocol::what_the_agent_says_back`).
    let heard = heard.trim_end();
    if heard.trim_start().is_empty() {
        return Err(NothingCameBack::ClosedWithoutAnswering);
    }
    Ok(heard.to_owned())
}

/// The request's connection, kept open so that the turn — and a change waiting
/// in it — lasts until this is dropped.
///
/// `alo_agent::holding_the_turn` holds a turn **until the asking side closes**.
/// [`what_the_agent_says`] closes its sending side at once, so a change its
/// request left waiting lapses with the turn the moment the reply is read. The
/// terminal acting without the OK has to answer that change first, so it keeps
/// the connection open and lets go of it only once the daemon has answered —
/// which is exactly what `alo-shell` does while a card is on the screen.
///
/// **Dropping it is the whole of letting go**, and it is also what a Ctrl-C does
/// to this process: the agent sees the close and the turn ends, with nothing
/// approved.
#[derive(Debug)]
pub struct TheTurnHeldOpen {
    /// The connection, which nothing writes to again.
    talking: UnixStream,
}

impl TheTurnHeldOpen {
    /// Let go of the turn: the agent sees the close, and the turn ends.
    ///
    /// Said out loud rather than left to a drop, so that the moment the turn
    /// ends is a line somebody can point at. Dropping it does the same.
    pub fn let_go(self) {
        drop(self.talking.shutdown(std::net::Shutdown::Both));
    }
}

/// Hand one sentence to the agent and wait for what it says, **keeping the turn
/// open** until the returned [`TheTurnHeldOpen`] is dropped.
///
/// It also writes the earlier-conversation marker and the window marker with
/// nothing after either — a first request, about no window — and then where
/// the person's folders are, or that marker alone, so that the agent need not
/// wait to learn there is none (`docs/contracts/asking-socket.md`). Keeping the sending side open makes the
/// agent read a close as a Stop, and that is what it is.
///
/// **The reply is kept as written, only its end trimmed.** Its first line is
/// what says whether a change is waiting
/// (`alo_protocol::what_the_agent_says_back`), and trimming its start would
/// undo the space that keeps a model's words off that line.
///
/// # Errors
/// [`NothingCameBack`], as [`what_the_agent_says`] has them.
pub fn what_the_agent_says_holding_the_turn(
    at: &Path,
    typed: &str,
    their_folders: Option<&alo_instructing::ThePersonsFolders>,
) -> Result<(String, TheTurnHeldOpen), NothingCameBack> {
    let mut talking = UnixStream::connect(at).map_err(|_| NothingCameBack::NobodyThere {
        at: at.display().to_string(),
    })?;
    let fault = |why: std::io::Error| NothingCameBack::WouldNotTake {
        why: why.to_string(),
    };
    talking.set_read_timeout(Some(PATIENCE)).map_err(fault)?;
    talking.set_write_timeout(Some(PATIENCE)).map_err(fault)?;
    let one_line: String = typed
        .chars()
        .map(|it| if it == '\n' || it == '\r' { ' ' } else { it })
        .collect();
    talking
        .write_all(
            format!(
                "{one_line}\n{}\n{}\n{}\n",
                alo_instructing::THE_EARLIER_CONVERSATION,
                alo_instructing::THE_WINDOW_OFFERED,
                alo_instructing::ThePersonsFolders::the_line_for(their_folders)
            )
            .as_bytes(),
        )
        .and_then(|()| talking.flush())
        .map_err(fault)?;

    let mut heard = String::new();
    let mut reading = BufReader::new(&talking);
    loop {
        let mut line = String::new();
        match reading.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => heard.push_str(&line),
            Err(why) => return Err(fault(why)),
        }
    }
    let heard = heard.trim_end();
    if heard.trim_start().is_empty() {
        return Err(NothingCameBack::ClosedWithoutAnswering);
    }
    Ok((heard.to_owned(), TheTurnHeldOpen { talking }))
}

#[cfg(test)]
#[path = "at_the_agents_door_tests.rs"]
mod tests;
