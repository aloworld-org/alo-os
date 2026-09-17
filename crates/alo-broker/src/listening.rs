//! The door as a real socket: bound, handed to `alo-agentd`'s group, answered
//! one request at a time.
//!
//! Everything decided is `crate::broker`'s; this is the part that touches a
//! machine, and it is small on purpose.
//!
//! **One socket, one line in, one line out, one caller at a time.** A system
//! verb is a person's approval crossing a privilege boundary, which happens at
//! the speed people approve things; a broker serving callers concurrently would
//! be a privileged process holding half-finished requests for no case that
//! exists. **The kernel is asked who is calling before the line is read**, and
//! a caller it does not name as the agent's service is written down and
//! answered without a byte of what it sent being looked at. **Reading and
//! writing each have [`PATIENCE`]**, so a caller that connects and says nothing
//! costs one connection and cannot hold the only door shut. **A line is read
//! up to [`crate::LONGEST`] bytes and no further**, so a caller cannot make a
//! privileged process hold an arbitrarily large buffer.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::answer::Answer;
use crate::broker::Broker;
use crate::keeping::{Carrying, Recording};
use crate::request::LONGEST;

/// How long one read or one write at the door may take.
pub const PATIENCE: Duration = Duration::from_secs(10);

/// The mode the door is made with: its owner and its group, nobody else.
const THE_DOORS_MODE: u32 = 0o660;

/// The door this process listens at.
#[derive(Debug)]
pub struct Listening {
    /// Where it is.
    at: PathBuf,
    /// The socket.
    socket: UnixListener,
}

impl Listening {
    /// Bind this path and hand it to this group — the group `alo-agentd` runs in.
    ///
    /// The path is the caller's, so a test can bind somewhere it may write.
    ///
    /// # Errors
    /// Whatever the machine said about binding, the mode or the handover. A
    /// socket bound and not handed over is removed: a door standing open at the
    /// wrong permissions is worse than none.
    pub fn at(path: &Path, group: u32) -> Result<Self, std::io::Error> {
        drop(std::fs::remove_file(path));
        let socket = UnixListener::bind(path)?;
        match handed_over(path, group) {
            Ok(()) => Ok(Self {
                at: path.to_owned(),
                socket,
            }),
            Err(why) => {
                drop(std::fs::remove_file(path));
                Err(why)
            }
        }
    }

    /// Where this door is.
    #[must_use]
    pub fn at_path(&self) -> &Path {
        &self.at
    }

    /// Answer one caller, and say what it was answered.
    ///
    /// # Errors
    /// Whatever the machine said about accepting or writing. A caller that went
    /// away before its answer is one of these; what it asked was still written
    /// down.
    pub fn answer_one<R: Recording, C: Carrying>(
        &self,
        broker: &mut Broker<R, C>,
    ) -> Result<Answer, std::io::Error> {
        let (connection, _) = self.socket.accept()?;
        connection.set_read_timeout(Some(PATIENCE))?;
        connection.set_write_timeout(Some(PATIENCE))?;

        let caller = crate::unix::the_user_of(&connection).ok();
        let line = if broker.hears(caller) {
            one_line(&connection)
        } else {
            Vec::new()
        };
        let answer = broker.heard(caller, &line, SystemTime::now());
        (&connection).write_all(answer.written().as_bytes())?;
        (&connection).write_all(b"\n")?;
        Ok(answer)
    }
}

/// One line, without its newline, read no further than just past the longest a
/// request may be.
///
/// Anything that is not a whole line inside that bound — a caller that stopped
/// before its newline, a line too long, a read the machine refused — comes back
/// empty, and an empty line is not a request.
fn one_line(connection: &UnixStream) -> Vec<u8> {
    let bound = u64::try_from(LONGEST.saturating_add(1)).unwrap_or(u64::MAX);
    let mut line = Vec::new();
    let read = BufReader::new(connection.take(bound)).read_until(b'\n', &mut line);
    if read.is_err() || line.pop() != Some(b'\n') {
        return Vec::new();
    }
    line
}

/// The door, at its mode and in its group.
fn handed_over(path: &Path, group: u32) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt as _;

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(THE_DOORS_MODE))?;
    crate::unix::give_to_group(path, group)
}
