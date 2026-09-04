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
//! # `/run/alo/<uid>`, and why it is not in the person's session (ADR 0017)
//!
//! The socket was `$XDG_RUNTIME_DIR/alo/agentd.sock` and could not be reached
//! by the thing it exists for. `logind` makes `/run/user/<uid>` **`0700`, owned
//! by the person**, and the agent is a login of its own (ADR 0001 §5) — so a
//! correct `0750` directory inside it is a locked room inside a locked
//! building, and every connection from the agent's login was refused by the
//! *parent* before either of the two locks below was consulted.
//!
//! So the door is ours: **`/run/alo/<uid>/agentd.sock`**, named for the person
//! whose door it is because a machine may have more than one person on it.
//!
//! - **`/run/alo` is the image's**, made at boot through `tmpfiles.d`. This
//!   crate never creates it — [`NotBound::NoParent`] is what a machine without
//!   it is told, and it names the directory and who makes it. Creating it here
//!   would mean a service deciding the mode of a directory every person's door
//!   goes in.
//! - **`/run/alo/<uid>` is this crate's**, made when a session starts and taken
//!   away by `Place::taken_away` when it ends. That is allowed where making
//!   `/run/user/<uid>` was not: reacting to a session that already exists is
//!   not standing in for `logind` and inventing one.
//!
//! # Where the root itself comes from is not decided here
//!
//! [`Place::for_person`] is the real machine's, and [`Place::beneath`] takes
//! the root as an argument. That is the same argument this file made when the
//! root came from the environment: a rule about a directory is only a rule with
//! a test if a test can be run against a directory it may write in, and `/run`
//! is not one. What is decided here is the shape — the person's number, then
//! `agentd.sock` — and it is part of `docs/contracts/daemon-protocol.md`
//! because it is how somebody else's client finds this machine.

use std::fs::{DirBuilder, Permissions};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::caller::Uid;
use crate::refusing::NotBound;
use crate::side::Sides;
use crate::unix::give_to_group;

/// The directory every person's door goes in, which the image makes at boot.
///
/// Part of the daemon protocol's public surface, with ADR 0017 behind it.
pub const THE_ROOT: &str = "/run/alo";

/// The socket's name. Part of the daemon protocol's public surface.
pub const THE_SOCKET: &str = "agentd.sock";

/// The person owns it, the agent's group may enter it, nobody else exists.
const THE_DIRECTORY_MODE: u32 = 0o750;

/// The person and the agent may talk on it, nobody else may.
const THE_SOCKET_MODE: u32 = 0o660;

/// Where this machine's agent socket goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// The directory every person's door goes in, which the image makes.
    root: PathBuf,
    /// This person's directory, which this crate makes and owns.
    directory: PathBuf,
    /// The socket itself.
    socket: PathBuf,
}

impl Place {
    /// This person's door on a real machine, beneath [`THE_ROOT`].
    #[must_use]
    pub fn for_person(person: Uid) -> Self {
        Self::beneath(Path::new(THE_ROOT), person)
    }

    /// This person's door beneath this root.
    ///
    /// Nothing is made or looked at: this is three paths joined, and
    /// [`Place::prepared`] is what touches a disk.
    #[must_use]
    pub fn beneath(root: &Path, person: Uid) -> Self {
        let directory = root.join(person.raw().to_string());
        let socket = directory.join(THE_SOCKET);
        Self {
            root: root.to_path_buf(),
            directory,
            socket,
        }
    }

    /// The directory every person's door goes in.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
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
    /// and in all three nothing has been changed. [`NotBound::NoParent`] is the
    /// image not having made [`THE_ROOT`], and [`NotBound::NoDirectory`],
    /// [`NotBound::Unreadable`], [`NotBound::NotOurGroup`] and
    /// [`NotBound::NotShutTo`] are the machine saying it would not.
    pub fn prepared(&self, sides: &Sides) -> Result<(), NotBound> {
        self.make_it_if_it_is_missing()?;
        self.check_it_is_the_persons(sides)?;
        self.shut_it_to_everybody_else(sides)
    }

    /// Make this person's directory, `0750` from the moment it exists.
    ///
    /// The mode is given to the call that creates it rather than set
    /// afterwards, so there is no moment at which the directory exists and
    /// anybody can write it.
    ///
    /// **Only this person's directory.** A missing parent is
    /// [`NotBound::NoParent`] rather than a second `mkdir`, because
    /// [`THE_ROOT`] is the image's (ADR 0017) and a service that made it would
    /// be choosing the mode of the directory every person's door goes in — on a
    /// machine where it is missing because something is wrong with the boot
    /// rather than because nobody thought of it.
    fn make_it_if_it_is_missing(&self) -> Result<(), NotBound> {
        match DirBuilder::new()
            .mode(THE_DIRECTORY_MODE)
            .create(&self.directory)
        {
            Ok(()) => Ok(()),
            Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Err(NotBound::NoParent {
                at: self.root.clone(),
            }),
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

    /// Take the door away when the session ends.
    ///
    /// The socket first, then the directory it was in. ADR 0017 gives this
    /// crate the per-person directory for as long as the session lasts, and a
    /// directory left behind is a door standing open on a machine nobody is
    /// signed in to — the socket outliving the session, which is the thing that
    /// can go wrong now that the place is ours rather than `logind`'s.
    ///
    /// **The directory is removed only if it is empty**, and `/run/alo` is
    /// never touched. Whatever else somebody put in there is theirs, and a
    /// service that emptied a directory on the way out would be doing what
    /// [`Place::prepared`]'s three refusals exist to stop, one step later.
    ///
    /// Nothing is reported, because this runs while the process is stopping and
    /// there is nobody to report to. What it costs when it fails is a stale
    /// socket or an empty directory, and both are cases
    /// [`crate::Listening::at`] already deals with on the next start.
    pub(crate) fn taken_away(&self) {
        drop(std::fs::remove_file(&self.socket));
        drop(std::fs::remove_dir(&self.directory));
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::caller::Gid;
    use crate::testing::{a_directory_of_our_own, a_place_of_our_own, ourselves};

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

    /// The shape is decided here, and a client that looked for something else
    /// would not find this machine. ADR 0017: the person's number beneath
    /// `/run/alo`, and the socket beneath that.
    #[test]
    fn the_socket_is_where_the_contract_says_it_is() {
        let place = Place::for_person(Uid::of(1000).unwrap());
        assert_eq!(place.root(), Path::new("/run/alo"));
        assert_eq!(place.directory(), Path::new("/run/alo/1000"));
        assert_eq!(place.socket(), Path::new("/run/alo/1000/agentd.sock"));
    }

    /// **One person's door is not another's.** A machine may have more than one
    /// person on it, and the two doors have nothing in common but the root the
    /// image made.
    #[test]
    fn two_people_on_one_machine_have_two_doors() {
        let one = Place::for_person(Uid::of(1000).unwrap());
        let other = Place::for_person(Uid::of(1001).unwrap());

        assert_ne!(one.socket(), other.socket());
        assert_eq!(one.root(), other.root());
    }

    /// Making the place makes the directory, and makes it `0750` — the person,
    /// the agent's group, and nobody at all beyond that.
    #[test]
    fn a_missing_directory_is_made_shut() {
        let place = a_place_of_our_own("made-shut");

        place.prepared(&ourselves()).unwrap();

        assert_eq!(mode_of(place.directory()), 0o750);
    }

    /// **The root is the image's and is not made here.** A machine whose
    /// `/run/alo` is missing is told which directory is missing rather than
    /// having one invented for it at a mode this service chose — and nothing is
    /// left behind on the way out.
    #[test]
    fn a_missing_root_is_refused_and_names_the_directory_the_image_makes() {
        let folder = a_directory_of_our_own("no-root");
        let root = folder.join("never-made");
        let place = Place::beneath(&root, ourselves().person());

        let refused = place.prepared(&ourselves()).unwrap_err();

        assert!(matches!(refused, NotBound::NoParent { .. }), "{refused}");
        assert!(!root.exists(), "the root was made after all");
    }

    /// **A directory that was left open is shut**, rather than accepted because
    /// it existed. It was right yesterday is not an argument about today.
    #[test]
    fn a_directory_left_open_is_shut_again() {
        let place = a_place_of_our_own("left-open");
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
        let place = a_place_of_our_own("twice");

        place.prepared(&ourselves()).unwrap();
        place.prepared(&ourselves()).unwrap();
    }

    /// **A directory belonging to somebody else is refused**, and the refusal
    /// names who owns it. Nothing is chmodded, chowned or emptied on the way
    /// out.
    #[test]
    fn somebody_elses_directory_is_refused_and_left_alone() {
        let place = a_place_of_our_own("not-ours");
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
        let place = a_place_of_our_own("a-file");
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
        let place = a_place_of_our_own("a-link");
        let elsewhere = place.root().join("really-here");
        DirBuilder::new().mode(0o750).create(&elsewhere).unwrap();

        std::os::unix::fs::symlink(&elsewhere, place.directory()).unwrap();

        let refused = place.prepared(&ourselves()).unwrap_err();
        assert!(matches!(refused, NotBound::ALink { .. }));
    }

    /// The socket's own mode is `0660`: the person and the agent, and nobody
    /// else, on top of a directory nobody else can enter.
    #[test]
    fn the_socket_is_shut_to_everybody_but_the_two() {
        let place = a_place_of_our_own("socket-mode");
        place.prepared(&ourselves()).unwrap();
        std::fs::write(place.socket(), b"").unwrap();
        std::fs::set_permissions(place.socket(), Permissions::from_mode(0o666)).unwrap();

        place.shut_the_socket(&ourselves()).unwrap();

        assert_eq!(mode_of(place.socket()), 0o660);
    }

    /// **The door goes when the session does**, which is what this crate owes
    /// for being given a directory outside the person's session: the socket and
    /// the directory it was in, and the root the image made left alone.
    #[test]
    fn taking_the_door_away_leaves_the_root_the_image_made() {
        let place = a_place_of_our_own("taken-away");
        place.prepared(&ourselves()).unwrap();
        std::fs::write(place.socket(), b"").unwrap();

        place.taken_away();

        assert!(!place.socket().exists());
        assert!(!place.directory().exists());
        assert!(place.root().is_dir(), "and /run/alo is not ours to remove");
    }

    /// **Whatever else is in the directory is somebody's**, so the socket goes
    /// and the directory stays. A service that emptied a directory on the way
    /// out would be doing what the three refusals above exist to stop, one step
    /// later.
    #[test]
    fn a_directory_with_something_else_in_it_is_left_where_it_is() {
        let place = a_place_of_our_own("not-emptied");
        place.prepared(&ourselves()).unwrap();
        std::fs::write(place.socket(), b"").unwrap();
        let theirs = place.directory().join("somebody-elses");
        std::fs::write(&theirs, b"not ours to remove").unwrap();

        place.taken_away();

        assert!(!place.socket().exists(), "the socket is ours and it went");
        assert_eq!(std::fs::read(&theirs).unwrap(), b"not ours to remove");
    }
}
