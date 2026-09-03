//! Where the socket goes, and what has to be true of the directory it goes in.
//!
//! # Whoever owns the directory owns the socket
//!
//! A Unix socket is a name in a directory. Anybody who can write that directory
//! can delete the name and bind their own socket to it, and from then on every
//! client on the machine that looks for `alo-agentd` finds them — with the
//! person's approvals going to it. So the directory is not somewhere the socket
//! happens to be put; it is the first thing this crate checks and the first
//! thing it sets.
//!
//! [`Place::prepared`] refuses three shapes outright and fixes one:
//!
//! - **a symbolic link** is refused, because where the socket really goes would
//!   then be decided by whoever can change the link, and that is a different
//!   question from who owns the directory we looked at;
//! - **anything that is not a directory** is refused, and is left where it is;
//! - **a directory belonging to somebody else** is refused, and nothing in this
//!   crate will chmod, chown or empty it — a daemon that took a directory over
//!   because it wanted the name would be doing what this check exists to stop;
//! - **the mode and the group are set**, to `0750` and to the group the agent
//!   is in, every time, because a directory that was right yesterday is not an
//!   argument about today.
//!
//! # `0750`, and why the socket's own mode is the second lock
//!
//! The person owns the directory and the agent's group may enter it. Nobody
//! else can reach the socket at all — not to connect to it, not to see whether
//! it is there.
//!
//! That is also what makes the moment between binding a socket and setting its
//! permissions harmless. For those few instructions the socket carries whatever
//! the process's umask left it with, which on an ordinary machine is readable
//! and writable by everybody; reaching it still means traversing a directory
//! created `0750` **before** the socket existed. The socket's own `0660` is set
//! straight afterwards and is the second lock rather than the first.
//!
//! # Where the directory itself comes from is not decided here
//!
//! [`Place::under`] is handed one. On a real machine it is
//! `$XDG_RUNTIME_DIR`, which is per-person, `0700`, and emptied when the
//! session ends — but reading an environment variable, and deciding what to do
//! when it is not set, belongs to the process (queue item 21d) rather than to a
//! library that would then be untestable anywhere. What is decided here is the
//! name: `alo/agentd.sock` beneath whatever it was handed, and that name is
//! part of `docs/contracts/daemon-protocol.md` because it is how somebody
//! else's client finds this machine.

use std::fs::{DirBuilder, Permissions};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::refusing::NotBound;
use crate::side::Sides;
use crate::unix::give_to_group;

/// The directory the socket lives in, beneath the one this crate is handed.
pub const THE_DIRECTORY: &str = "alo";

/// The socket's name. Part of the daemon protocol's public surface.
pub const THE_SOCKET: &str = "agentd.sock";

/// The person owns it, the agent's group may enter it, nobody else exists.
const THE_DIRECTORY_MODE: u32 = 0o750;

/// The person and the agent may talk on it, nobody else may.
const THE_SOCKET_MODE: u32 = 0o660;

/// Where this machine's agent socket goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// The directory the socket lives in, which this crate makes and owns.
    directory: PathBuf,
    /// The socket itself.
    socket: PathBuf,
}

impl Place {
    /// The place beneath this directory.
    ///
    /// Nothing is made or looked at: this is two paths joined, and
    /// [`Place::prepared`] is what touches a disk.
    #[must_use]
    pub fn under(runtime: &Path) -> Self {
        let directory = runtime.join(THE_DIRECTORY);
        let socket = directory.join(THE_SOCKET);
        Self { directory, socket }
    }

    /// The directory the socket lives in.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// The socket itself.
    #[must_use]
    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// Make the directory if it is not there, and make sure it is ours.
    ///
    /// Answers once the directory exists, belongs to the person, is entered by
    /// the agent's group and by nobody else, and is not a link to somewhere
    /// that story is not true of.
    ///
    /// # Errors
    ///
    /// [`NotBound::ALink`], [`NotBound::NotADirectory`] and
    /// [`NotBound::SomebodyElses`] are the three refusals this file exists for,
    /// and in all three nothing has been changed. [`NotBound::NoDirectory`],
    /// [`NotBound::Unreadable`], [`NotBound::NotOurGroup`] and
    /// [`NotBound::NotShutTo`] are the machine saying it would not.
    pub fn prepared(&self, sides: &Sides) -> Result<(), NotBound> {
        self.make_it_if_it_is_missing()?;
        self.check_it_is_the_persons(sides)?;
        self.shut_it_to_everybody_else(sides)
    }

    /// Make the directory, `0750` from the moment it exists.
    ///
    /// The mode is given to the call that creates it rather than set
    /// afterwards, so there is no moment at which the directory exists and
    /// anybody can write it.
    fn make_it_if_it_is_missing(&self) -> Result<(), NotBound> {
        match DirBuilder::new()
            .mode(THE_DIRECTORY_MODE)
            .create(&self.directory)
        {
            Ok(()) => Ok(()),
            Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            Err(why) => Err(NotBound::NoDirectory {
                at: self.directory.clone(),
                why,
            }),
        }
    }

    /// Refuse anything that is not a directory of the person's own.
    ///
    /// The metadata is read **without following a link**, because a link is one
    /// of the three things being refused and following it would answer about
    /// whatever it points at instead.
    fn check_it_is_the_persons(&self, sides: &Sides) -> Result<(), NotBound> {
        let what_is_there =
            std::fs::symlink_metadata(&self.directory).map_err(|why| NotBound::Unreadable {
                at: self.directory.clone(),
                why,
            })?;

        if what_is_there.file_type().is_symlink() {
            return Err(NotBound::ALink {
                at: self.directory.clone(),
            });
        }
        if !what_is_there.is_dir() {
            return Err(NotBound::NotADirectory {
                at: self.directory.clone(),
            });
        }
        if what_is_there.uid() != sides.person().raw() {
            return Err(NotBound::SomebodyElses {
                at: self.directory.clone(),
                owner: what_is_there.uid(),
            });
        }
        Ok(())
    }

    /// Set the group and the mode, whatever they were before.
    fn shut_it_to_everybody_else(&self, sides: &Sides) -> Result<(), NotBound> {
        give_to_group(&self.directory, sides.shared()).map_err(|why| NotBound::NotOurGroup {
            at: self.directory.clone(),
            group: sides.shared().raw(),
            why,
        })?;
        std::fs::set_permissions(&self.directory, Permissions::from_mode(THE_DIRECTORY_MODE))
            .map_err(|why| NotBound::NotShutTo {
                at: self.directory.clone(),
                why,
            })
    }

    /// Shut the socket itself to everybody but the person and the agent.
    ///
    /// Called once the socket is bound. The directory is what makes the moment
    /// before this harmless; see this file's own documentation.
    ///
    /// # Errors
    ///
    /// [`NotBound::NotOurGroup`] if the person is not in the agent's group, and
    /// [`NotBound::NotShutTo`] if the mode would not be set. In both, the
    /// socket exists and nothing is listening on it yet.
    pub(crate) fn shut_the_socket(&self, sides: &Sides) -> Result<(), NotBound> {
        give_to_group(&self.socket, sides.shared()).map_err(|why| NotBound::NotOurGroup {
            at: self.socket.clone(),
            group: sides.shared().raw(),
            why,
        })?;
        std::fs::set_permissions(&self.socket, Permissions::from_mode(THE_SOCKET_MODE)).map_err(
            |why| NotBound::NotShutTo {
                at: self.socket.clone(),
                why,
            },
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::caller::{Gid, Uid};
    use crate::testing::{a_directory_of_our_own, ourselves};

    /// Who a directory was refused as belonging to, if that is what happened.
    ///
    /// A helper rather than a `let … else { panic!(…) }`, because this
    /// workspace denies `clippy::panic` everywhere including its tests.
    fn whose(refused: &NotBound) -> Option<u32> {
        match refused {
            NotBound::SomebodyElses { owner, .. } => Some(*owner),
            _ => None,
        }
    }

    /// The mode a path really has on the disk, without the file type bits.
    fn mode_of(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    /// The two names are decided here, and a client that looked for something
    /// else would not find this machine.
    #[test]
    fn the_socket_is_where_the_contract_says_it_is() {
        let place = Place::under(Path::new("/run/user/1000"));
        assert_eq!(place.directory(), Path::new("/run/user/1000/alo"));
        assert_eq!(place.socket(), Path::new("/run/user/1000/alo/agentd.sock"));
    }

    /// Making the place makes the directory, and makes it `0750` — the person,
    /// the agent's group, and nobody at all beyond that.
    #[test]
    fn a_missing_directory_is_made_shut() {
        let folder = a_directory_of_our_own("made-shut");
        let place = Place::under(&folder);

        place.prepared(&ourselves()).unwrap();

        assert_eq!(mode_of(place.directory()), 0o750);
    }

    /// **A directory that was left open is shut**, rather than accepted because
    /// it existed. It was right yesterday is not an argument about today.
    #[test]
    fn a_directory_left_open_is_shut_again() {
        let folder = a_directory_of_our_own("left-open");
        let place = Place::under(&folder);
        DirBuilder::new()
            .mode(0o777)
            .create(place.directory())
            .unwrap();

        place.prepared(&ourselves()).unwrap();

        assert_eq!(mode_of(place.directory()), 0o750);
    }

    /// Preparing a place twice is preparing it once: a daemon restarting on a
    /// machine it already ran on is the ordinary case, not an error.
    #[test]
    fn preparing_it_twice_is_preparing_it_once() {
        let folder = a_directory_of_our_own("twice");
        let place = Place::under(&folder);

        place.prepared(&ourselves()).unwrap();
        place.prepared(&ourselves()).unwrap();
    }

    /// **A directory belonging to somebody else is refused**, and the refusal
    /// names who owns it. Nothing is chmodded, chowned or emptied on the way
    /// out.
    #[test]
    fn somebody_elses_directory_is_refused_and_left_alone() {
        let folder = a_directory_of_our_own("not-ours");
        let place = Place::under(&folder);
        place.prepared(&ourselves()).unwrap();

        // The same directory, described as belonging to a person this process
        // is not. Nothing about the disk changed; what changed is who the
        // machine says the person is.
        let strangers = Sides::of(
            Uid::of(4_242_424).unwrap(),
            Uid::of(989).unwrap(),
            Gid::of(989).unwrap(),
        )
        .unwrap();

        let refused = place.prepared(&strangers).unwrap_err();
        assert_eq!(whose(&refused).unwrap(), ourselves().person().raw());
        assert!(place.directory().is_dir(), "it is still there");
    }

    /// **A file where the directory belongs is refused and left there.** A
    /// daemon that deleted whatever was in its way would be the thing this
    /// check exists to stop, wearing our name.
    #[test]
    fn a_file_in_the_way_is_refused_and_left_there() {
        let folder = a_directory_of_our_own("a-file");
        let place = Place::under(&folder);
        std::fs::write(place.directory(), b"somebody's file").unwrap();

        let refused = place.prepared(&ourselves()).unwrap_err();
        assert!(matches!(refused, NotBound::NotADirectory { .. }));
        assert_eq!(
            std::fs::read(place.directory()).unwrap(),
            b"somebody's file",
            "and it is still theirs"
        );
    }

    /// **A symbolic link is refused even when it points somewhere we own**,
    /// because where the socket really goes would be decided by whoever can
    /// change the link rather than by who owns what we looked at.
    #[test]
    fn a_link_is_refused_even_pointing_somewhere_ours() {
        let folder = a_directory_of_our_own("a-link");
        let elsewhere = folder.join("really-here");
        DirBuilder::new().mode(0o750).create(&elsewhere).unwrap();

        let place = Place::under(&folder);
        std::os::unix::fs::symlink(&elsewhere, place.directory()).unwrap();

        let refused = place.prepared(&ourselves()).unwrap_err();
        assert!(matches!(refused, NotBound::ALink { .. }));
    }

    /// The socket's own mode is `0660`: the person and the agent, and nobody
    /// else, on top of a directory nobody else can enter.
    #[test]
    fn the_socket_is_shut_to_everybody_but_the_two() {
        let folder = a_directory_of_our_own("socket-mode");
        let place = Place::under(&folder);
        place.prepared(&ourselves()).unwrap();
        std::fs::write(place.socket(), b"").unwrap();
        std::fs::set_permissions(place.socket(), Permissions::from_mode(0o666)).unwrap();

        place.shut_the_socket(&ourselves()).unwrap();

        assert_eq!(mode_of(place.socket()), 0o660);
    }
}
