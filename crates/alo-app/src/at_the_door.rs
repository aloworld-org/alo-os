//! **The compositor's door on this machine**: `/run/alo/<uid>/shell.sock`,
//! one line in and one line out (`docs/contracts/application-door.md`).
//!
//! A new connection for every question, which is the contract's rule — *one
//! line is one act* — with one exception it makes itself: a count holds its
//! connection for as long as the act runs, because a connection is the one
//! thing a command cannot forget to end. If `alo-app` dies mid-act, the count
//! ends with it.
//!
//! **The door admits only this person's own login**, decided by the kernel
//! from the socket's credentials; nothing here proves anything to it, and
//! nothing needs to (ADR 0108).

use std::io::{BufReader, Write as _};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_acting::{
    Answer, Begun, Counted, InFront, Request, ToCount, WhereAlosTreeIs, alos_tree::ALO_TREE,
    counting::DONE, front::FRONT,
};

use crate::door::{HeldCount, TheDoor, Unanswered};

/// The compositor's socket, in this person's own directory: part of the
/// contract's published surface.
pub const THE_SOCKET: &str = "shell.sock";

/// How long one answer is waited for. The answer is one word on a local
/// socket; a desktop that has not given it in this long is not about to.
const PATIENCE: Duration = Duration::from_secs(5);

/// The door, by where it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtTheDoor {
    /// The socket.
    socket: PathBuf,
}

impl AtTheDoor {
    /// The door in this person's directory — `/run/alo/<uid>` on a machine.
    #[must_use]
    pub fn in_(directory: &Path) -> Self {
        Self {
            socket: directory.join(THE_SOCKET),
        }
    }

    /// A connection, with its patience set.
    fn connected(&self) -> Result<UnixStream, Unanswered> {
        let stream = UnixStream::connect(&self.socket).map_err(|_| Unanswered::NotReached)?;
        stream
            .set_read_timeout(Some(PATIENCE))
            .and_then(|()| stream.set_write_timeout(Some(PATIENCE)))
            .map_err(|_| Unanswered::NotReached)?;
        Ok(stream)
    }

    /// Ask one line and read one line back.
    fn asked(&self, line: &str) -> Result<String, Unanswered> {
        let stream = self.connected()?;
        said(&stream, line).map_err(|_| Unanswered::NotReached)?;
        heard(&mut BufReader::new(&stream)).ok_or(Unanswered::NotUnderstood)
    }
}

/// Write one line.
fn said(stream: &UnixStream, line: &str) -> std::io::Result<()> {
    let mut writing = stream;
    writing.write_all(line.as_bytes())?;
    writing.write_all(b"\n")
}

/// Read one line, or nothing.
fn heard(reading: &mut impl std::io::BufRead) -> Option<String> {
    let mut line = String::new();
    match reading.read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line),
    }
}

impl TheDoor for AtTheDoor {
    fn front(&mut self) -> Result<InFront, Unanswered> {
        let line = self.asked(FRONT)?;
        InFront::read(&line).ok_or(Unanswered::NotUnderstood)
    }

    fn alos_tree(&mut self) -> Result<WhereAlosTreeIs, Unanswered> {
        let line = self.asked(ALO_TREE)?;
        WhereAlosTreeIs::read(&line).ok_or(Unanswered::NotUnderstood)
    }

    fn act(&mut self, request: &Request) -> Result<Answer, Unanswered> {
        let line = self.asked(&request.written())?;
        Answer::read(&line).ok_or(Unanswered::NotUnderstood)
    }

    fn begin_counting(&mut self, asked: &ToCount) -> Option<Box<dyn HeldCount>> {
        let stream = self.connected().ok()?;
        said(&stream, &asked.written()).ok()?;
        // **One reader for the whole count**, so a `stopped` the desktop wrote
        // straight after `counting` is not lost in a buffer thrown away.
        let mut reading = BufReader::new(stream.try_clone().ok()?);
        match Begun::read(&heard(&mut reading)?)? {
            Begun::Counting => Some(Box::new(Counting { stream, reading })),
            Begun::NotInstalled | Begun::CannotCount => None,
        }
    }
}

/// A count being held.
#[derive(Debug)]
struct Counting {
    /// The connection the count lasts for, written to.
    stream: UnixStream,
    /// The same connection, read from.
    reading: BufReader<UnixStream>,
}

impl HeldCount for Counting {
    fn done(mut self: Box<Self>) -> Option<Counted> {
        // **Said whether or not it can be.** The desktop may already have ended
        // the count and written `stopped`, and a closed connection then refuses
        // this line; what it wrote first is still there to read.
        let _ = said(&self.stream, DONE);
        Counted::read(&heard(&mut self.reading)?)
    }
}
