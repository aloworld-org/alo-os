//! The door as a real socket: bound, handed to the greeter's group, and
//! answered one caller at a time.
//!
//! Everything decided here is `crate::opening`'s; this is the part that has to
//! touch a machine. It is small on purpose — a privileged process's I/O is the
//! surface somebody would go looking at first.
//!
//! # One caller at a time, and that is not a shortcut
//!
//! There is one screen, one person in front of it and one session to open. A
//! door that served callers concurrently would be a privileged process holding
//! several half-finished sign-ins, for no case that exists: the second knock
//! this component can receive is the one `crate::Opening` answers with *already
//! signed in*. `alo-agentd` waits on several things at once because an agent, a
//! person and a stop all arrive on different descriptors; nothing here has a
//! second descriptor to wait on.
//!
//! # A caller that says nothing costs one connection and no more
//!
//! Reading and writing are each given [`PATIENCE`]. Without it, anything in the
//! greeter's group could connect, say nothing, and leave the only door to
//! signing in blocked for ever — which on this machine is the whole of what a
//! denial of service against sign-in would look like.
//!
//! # What it does not do
//!
//! **It does not create its directory.** systemd does, from the unit's
//! `RuntimeDirectory=`, and `crate::place` says why. **It does not remove the
//! socket at exit** either: the directory is the unit's and systemd takes the
//! whole of it away. What it does remove is a socket left behind by a previous
//! start, because a `bind` onto an existing path fails and a service that could
//! not restart after a crash is a machine nobody can sign in to.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::accounts::TheAccounts;
use crate::asking::{Answered, Knock};
use crate::logind::Logind;
use crate::opening::Opening;
use crate::words::NOT_YOURS_TO_ASK;

/// How long one read or one write at the door may take.
///
/// Generous against a machine under load at boot, and short against a door that
/// something is sitting on.
pub const PATIENCE: Duration = Duration::from_secs(10);

/// The mode the door is made with: this process, the greeter's group, and
/// nobody else.
const THE_DOORS_MODE: u32 = 0o660;

/// The door this process listens at.
#[derive(Debug)]
pub struct Listening {
    /// Where it is.
    at: PathBuf,
    /// The socket itself.
    socket: UnixListener,
}

impl Listening {
    /// Bind this path and hand it to this group.
    ///
    /// The path is the caller's rather than [`crate::THE_DOOR`], which is
    /// `alo-agentd`'s `place.rs` argument and `alo-boundaryd`'s: a rule about a
    /// socket is only a rule with a test if a test can bind one somewhere it
    /// may write, and `/run` is not that.
    ///
    /// # Errors
    ///
    /// Whatever the machine said about binding, about the mode, or about the
    /// handover. Nothing is left behind on any of those roads: a socket that
    /// was bound and could not be handed over is removed, because a door
    /// standing open at the wrong permissions is worse than no door.
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

    /// Answer one caller, and say what it was answered with.
    ///
    /// # Errors
    ///
    /// Whatever the machine said about accepting, reading or writing. A caller
    /// that went away mid-sentence is one of these and is not a refusal: there
    /// is nobody left to refuse.
    pub fn answer_one<A: TheAccounts, L: Logind>(
        &self,
        opening: &mut Opening<A, L>,
    ) -> Result<Answered, std::io::Error> {
        let (connection, _) = self.socket.accept()?;
        connection.set_read_timeout(Some(PATIENCE))?;
        connection.set_write_timeout(Some(PATIENCE))?;

        let answered = what_to_say(&connection, opening);
        (&connection).write_all(answered.written().as_bytes())?;
        (&connection).write_all(b"\n")?;
        Ok(answered)
    }
}

/// What this connection is answered with.
///
/// A caller whose group the kernel would not say, and a line that is not a
/// knock, are both answered with the one refusal that gives nothing away.
/// Neither is a failure of this machine — they are somebody at the door who is
/// not the sign-in surface — and neither may learn anything more than that.
fn what_to_say<A: TheAccounts, L: Logind>(
    connection: &UnixStream,
    opening: &mut Opening<A, L>,
) -> Answered {
    let Ok(group) = crate::unix::the_group_of(connection) else {
        return Answered::Refused(NOT_YOURS_TO_ASK.key());
    };
    let mut line = String::new();
    if BufReader::new(connection).read_line(&mut line).is_err() {
        return Answered::Refused(NOT_YOURS_TO_ASK.key());
    }
    match Knock::read(line.trim_end_matches(['\r', '\n'])) {
        Ok(knock) => opening.heard(knock, group),
        Err(why) => {
            eprintln!("alo-sessiond: {why}");
            Answered::Refused(NOT_YOURS_TO_ASK.key())
        }
    }
}

/// The door, at the mode and in the group it is meant to be.
fn handed_over(path: &Path, group: u32) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt as _;

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(THE_DOORS_MODE))?;
    crate::unix::give_to_group(path, group)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt as _;

    /// Somewhere this test may bind a socket.
    fn somewhere(what: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!("alo-sessiond-{}-{what}", std::process::id()));
        drop(std::fs::create_dir_all(&at));
        at.join("sign-in.sock")
    }

    /// **The door is made at the mode that is written down**, and handed to the
    /// group it was asked for. The mode is the half the kernel enforces before
    /// anything in this crate is reached.
    #[test]
    fn the_door_is_made_where_only_its_group_can_reach_it() {
        let at = somewhere("mode");
        let door = Listening::at(&at, crate::unix::our_group()).unwrap();

        let mode = std::fs::metadata(door.at_path())
            .unwrap()
            .permissions()
            .mode();

        assert_eq!(mode & 0o777, THE_DOORS_MODE);
        drop(std::fs::remove_file(&at));
    }

    /// **A socket left behind by a previous start is not a service that cannot
    /// start.** It is the ordinary state after a crash, and a machine nobody
    /// can sign in to is the wrong answer to it.
    #[test]
    fn a_door_left_behind_by_a_previous_start_is_replaced() {
        let at = somewhere("again");
        let first = Listening::at(&at, crate::unix::our_group()).unwrap();
        drop(first);

        assert!(
            at.exists(),
            "this test is about a socket that is still there"
        );
        assert!(Listening::at(&at, crate::unix::our_group()).is_ok());
        drop(std::fs::remove_file(&at));
    }
}
