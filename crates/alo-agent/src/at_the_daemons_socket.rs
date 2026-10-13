//! [`TheDoor`] against the real one: `alo-agentd`'s agent socket.
//!
//! One line written, one line read, for as long as the turn lasts.
//!
//! # One connection is one turn, and that is not an optimisation
//!
//! `docs/contracts/daemon-protocol.md`: **nothing names a turn** — *which turn
//! a message belongs to is answered by the connection it arrived on. A number
//! for it would be a number an agent could change.* So this holds the stream
//! for the life of the turn and does not reconnect between requests: a second
//! connection is a second turn, with its own grant and its own context, and a
//! loop that reconnected per request would be asking a fresh turn each time
//! and wondering why nothing it had read was still true.
//!
//! It follows that [`AtTheDaemonsSocket`] is made once per turn and dropped
//! with it. Closing it ends the turn, which is the daemon's business and
//! happens whether this process meant it or not — so there is no `close` here
//! to get wrong.
//!
//! # Nothing here decides which door it is on
//!
//! *Which side a connection is on is answered by `SO_PEERCRED` on the accepted
//! socket … nothing a client sends takes part in that decision, and there is
//! nothing it could send.* The agent is **its own Unix user**, and a machine
//! where the person and the agent are one login has no socket at all — the
//! daemon refuses to start, because both doors would be one and the side that
//! proposed a change could approve it.
//!
//! So this file opens a path and writes lines. If the process running it is not
//! the agent's user, the daemon closes the connection **with nothing written on
//! it**, and that arrives here as [`NotAnswered::NotReached`] — which is the
//! truth: this is not a machine this process may ask.
//!
//! # A turn does not wait for ever
//!
//! Both timeouts are set before anything is written, for `alo-changing`'s
//! reason one door along: a daemon that stopped answering would otherwise hold
//! this process until somebody noticed. [`AtTheDaemonsSocket::PATIENCE`] is
//! generous, because the thing on the other end may be putting a question to a
//! model and a model takes seconds.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use alo_protocol::{FromAnAgent, ToAnAgent};

use crate::the_door::{NotAnswered, TheDoor};

/// The daemon's agent socket, held open for one turn.
pub struct AtTheDaemonsSocket {
    /// The connection. **One of these is one turn** — see this module's header.
    stream: UnixStream,
}

impl AtTheDaemonsSocket {
    /// How long one request may take before this gives up on the machine.
    ///
    /// Minutes rather than seconds, because an `ask` on the other side is a
    /// model thinking and `alo_models`' own wait for one is five minutes. A
    /// shorter patience here would make this file the thing that decides how
    /// long a model may take, which is `alo-models`' and not ours.
    pub const PATIENCE: Duration = Duration::from_secs(300);

    /// Open a turn at this socket.
    ///
    /// # Errors
    /// [`NotAnswered::NotReached`] when there is nothing there, when this
    /// process is not the agent's user — the daemon closes such a connection
    /// with nothing written on it — or when the timeouts could not be set,
    /// which is refused rather than carried on without because a turn with no
    /// patience is a process that can hang for ever.
    pub fn opened(at: &Path) -> Result<Self, NotAnswered> {
        let not_reached = |why: &dyn std::fmt::Display| NotAnswered::NotReached {
            why: format!("{at}: {why}", at = at.display()),
        };
        let stream = UnixStream::connect(at).map_err(|why| not_reached(&why))?;
        stream
            .set_write_timeout(Some(Self::PATIENCE))
            .map_err(|why| not_reached(&why))?;
        stream
            .set_read_timeout(Some(Self::PATIENCE))
            .map_err(|why| not_reached(&why))?;
        Ok(Self { stream })
    }
}

impl TheDoor for AtTheDaemonsSocket {
    fn asking(&mut self, request: &FromAnAgent) -> Result<ToAnAgent, NotAnswered> {
        // Written by the protocol's own writer. This file composes no JSON, so
        // there is no second account of what a message looks like.
        let line = request
            .written()
            .map_err(|why| NotAnswered::NotTheProtocol {
                why: format!("this request could not be written: {why}"),
            })?;

        let mut writing = &self.stream;
        writing
            .write_all(line.as_bytes())
            .and_then(|()| writing.write_all(b"\n"))
            .map_err(|why| NotAnswered::NotReached {
                why: format!("the request could not be sent: {why}"),
            })?;

        let mut answer = String::new();
        let read = BufReader::new(&self.stream)
            .read_line(&mut answer)
            .map_err(|why| NotAnswered::NotReached {
                why: format!("no answer came back: {why}"),
            })?;
        // **Nothing read is the connection gone, not an empty answer.** A
        // daemon that closed the door — which is what it does to a stranger,
        // with nothing written — reads as zero bytes here, and treating that
        // as a message would be reading an answer out of silence.
        if read == 0 {
            return Err(NotAnswered::NotReached {
                why: "the daemon closed the connection without answering".to_owned(),
            });
        }
        ToAnAgent::read(answer.trim_end()).map_err(|why| NotAnswered::NotTheProtocol {
            // `Debug` and not `Display`, because `alo_protocol::NotUnderstood`
            // deliberately has no `Display`: *a `Display` is one `to_string()`
            // from a screen whose author had no reason to think about
            // language*, and the only road to words is `NotUnderstood::said`.
            // What goes here is read by whoever is fixing the machine, never
            // shown to a person, so `Debug` is the honest rendering rather
            // than a way round the rule.
            why: format!("{why:?}"),
        })
    }
}
