//! The other side of the door: asking once, and reading the one line back.
//!
//! What issues a token — a turn redeeming a person's approval, or Settings
//! carrying out a person's own choice — asks here. One connection per request,
//! the request's line and its newline, and one answer line read no further than
//! the longest answer there is. Nothing here decides anything about the answer;
//! the caller says what it means to a person.

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use crate::answer::Answer;
use crate::listening::PATIENCE;
use crate::request::Request;

/// How long the asking side waits for the answer.
///
/// Not [`PATIENCE`], which bounds a caller that says nothing. The answer comes
/// after the verb is carried out, and carrying one out takes as long as the
/// rented service takes: joining a network waits for the network manager to
/// associate, authenticate and be given an address, which on a busy access
/// point takes most of a minute, and setting a printer up asks the printing
/// service to spend ten seconds looking and then answers three more requests of
/// up to thirty seconds each. An asker that gave up sooner would tell a person
/// nothing was changed while the change was still being made.
pub const WAITING_FOR_THE_ANSWER: Duration = Duration::from_secs(180);

/// The most bytes an answer may be, newline included: `refused` and the
/// longest reason, with room to spare.
const LONGEST_ANSWER: u64 = 64;

/// Why the door gave no answer.
#[derive(Debug)]
pub enum NotAsked {
    /// Nothing is listening at the door, or it could not be reached.
    NoDoor(std::io::Error),
    /// The request was sent, and what came back was not an answer — or nothing
    /// came back in time. **Whether the verb was carried out is not known**: the
    /// broker writes a request down before carrying it out, so its record says.
    NoAnswer,
}

impl std::fmt::Display for NotAsked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDoor(why) => write!(f, "the broker's door could not be reached: {why}"),
            Self::NoAnswer => write!(f, "the broker's door answered with something else"),
        }
    }
}

impl std::error::Error for NotAsked {}

/// Ask the door at `door` for this request, once.
///
/// # Errors
/// [`NotAsked`]. A refusal is not one of these: it is an [`Answer`].
pub fn ask(door: &Path, request: &Request) -> Result<Answer, NotAsked> {
    let connection = UnixStream::connect(door).map_err(NotAsked::NoDoor)?;
    connection
        .set_read_timeout(Some(WAITING_FOR_THE_ANSWER))
        .and_then(|()| connection.set_write_timeout(Some(PATIENCE)))
        .map_err(NotAsked::NoDoor)?;
    let mut line = request.written().into_bytes();
    line.push(b'\n');
    // A door that refuses a caller unread may answer before the line is taken
    // in, so a failed write is not yet a missing answer: the answer is read
    // either way, and only its absence is refused.
    drop((&connection).write_all(&line));

    let mut answer = Vec::new();
    BufReader::new((&connection).take(LONGEST_ANSWER))
        .read_until(b'\n', &mut answer)
        .map_err(|_| NotAsked::NoAnswer)?;
    if answer.pop() != Some(b'\n') {
        return Err(NotAsked::NoAnswer);
    }
    std::str::from_utf8(&answer)
        .ok()
        .and_then(Answer::read)
        .ok_or(NotAsked::NoAnswer)
}
