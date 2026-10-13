//! Where the compositor's application door goes, and the five things it
//! refuses rather than serve.
//!
//! `docs/contracts/application-door.md` is the published surface and this file
//! is its transport half. [ADR 0108](../../../docs/decisions/0108-the-compositor-carries-an-application-verb-out-and-needs-no-token-to.md)
//! is the argument for the door existing at all.
//!
//! # The one thing this file is for
//!
//! Everything protecting this door is the **mode of the path it lives at**.
//! There is no token on the wire (ADR 0108) and no secret to check, so a socket
//! one group too wide is a socket the agent can reach, and the whole decision
//! comes apart. So the modes are set by the call that creates each thing rather
//! than afterwards, and the checks happen before anything is bound.
//!
//! # Why the person's directory is not ours
//!
//! `/run/alo/<uid>` is the **daemon's**, by ADR 0017 and
//! `docs/contracts/daemon-protocol.md`: it makes it when the person's session
//! starts and takes it away when the session ends. **This file never creates it
//! and never sets its mode.** A directory two services both made would carry
//! whichever mode lost the race, and the mode of the directory a socket lives
//! in is precisely what decides who may replace the socket.
//!
//! So a compositor that finds it missing serves **no door** and says which
//! directory is missing and who makes it. It keeps running: a window system is
//! not a thing to make depend on a service, and with no daemon there is no turn
//! to have authorised anything anyway.
//!
//! # And why the socket is `0600` where the daemon's is `0660`
//!
//! The daemon hands its socket to the group the agent is in, because the agent
//! has to reach it. This one is handed to nobody. The person and the agent are
//! two Unix logins (ADR 0001 §5), so a socket only the person's uid can open is
//! a socket the kernel keeps the agent out of, with nothing on the wire taking
//! part in the decision.

use std::fs::Permissions;
use std::io;
use std::os::unix::fs::{FileTypeExt as _, MetadataExt as _, PermissionsExt as _};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

/// The directory every person's door goes in, which the image makes at boot.
///
/// The same `/run/alo` the daemon's door is under. **Declared here rather than
/// borrowed from `alo-agentd`**, because the compositor must not depend on the
/// daemon — and held to the contract by
/// `tests/the_contract_describes_this_door.rs` rather than to the other crate's
/// copy, since two constants agree with each other whatever either of them
/// says.
pub const THE_ROOT: &str = "/run/alo";

/// The socket's name, after this crate's binary as the daemon's is after its.
///
/// Part of `docs/contracts/application-door.md`'s published surface.
pub const THE_SOCKET: &str = "shell.sock";

/// The person, and nobody else at all — not even the agent's group.
const THE_SOCKET_MODE: u32 = 0o600;

/// Where this machine's application door goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThePlace {
    /// This person's directory, which the **daemon** makes and owns.
    directory: PathBuf,
    /// The socket itself, which this crate makes and owns.
    socket: PathBuf,
}

impl ThePlace {
    /// This person's application door on a real machine, beneath [`THE_ROOT`].
    #[must_use]
    pub fn for_person(person: u32) -> Self {
        Self::beneath(Path::new(THE_ROOT), person)
    }

    /// The same, beneath a root a test can write to.
    ///
    /// `for_person` is the real machine and this is how every refusal below is
    /// tested without being root: the five of them are about a directory and a
    /// path, and a directory under a temporary one answers them all.
    #[must_use]
    pub fn beneath(root: &Path, person: u32) -> Self {
        let directory = root.join(person.to_string());
        Self {
            socket: directory.join(THE_SOCKET),
            directory,
        }
    }

    /// The socket this door would be served on.
    #[must_use]
    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// The person's directory this door goes in, which somebody else owns.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Check the directory, bind the socket, and shut it to everybody else.
    ///
    /// The listener comes back **not blocking**, because the one place it will
    /// be read from is a compositor's frame loop: a window system that stopped
    /// drawing to wait for a caller would be a window system a caller could
    /// freeze.
    ///
    /// # Errors
    ///
    /// [`NoDoor`], and every one of them means no socket exists and nothing is
    /// listening. A compositor reads any of them as *serve no door and keep
    /// running*, which is this file's own documentation.
    pub fn prepared(&self, person: u32) -> Result<UnixListener, NoDoor> {
        self.check_the_directory_is_the_persons(person)?;
        self.take_away_a_socket_of_our_own()?;
        let listening = UnixListener::bind(&self.socket).map_err(|why| NoDoor::NotBound {
            at: self.socket.clone(),
            why,
        })?;
        // **Anything that goes wrong from here takes the socket away**, and
        // that is the whole reason this is not three calls in a row. A bound
        // socket at the default mode is reachable by the agent's group, so a
        // refusal that left one behind would hand out exactly what ADR 0108
        // relies on the kernel withholding — and it would do it while
        // answering that the door was not served.
        if let Err(why) = self.shut_the_socket().and_then(|()| {
            listening
                .set_nonblocking(true)
                .map_err(|why| NoDoor::NotBound {
                    at: self.socket.clone(),
                    why,
                })
        }) {
            drop(listening);
            self.taken_away();
            return Err(why);
        }
        Ok(listening)
    }

    /// Take the socket away. Called when the compositor stops.
    ///
    /// Nothing is reported, because this runs while the process is ending and
    /// there is nobody to tell. **The directory is not touched**: it is the
    /// daemon's, and a compositor that removed it would take the agent's door
    /// with it.
    pub fn taken_away(&self) {
        drop(std::fs::remove_file(&self.socket));
    }

    /// Refuse anything that is not a directory of this person's own.
    ///
    /// The metadata is read **without following a link**, because a link is one
    /// of the three things being refused and following it would answer about
    /// whatever it points at instead.
    fn check_the_directory_is_the_persons(&self, person: u32) -> Result<(), NoDoor> {
        let what_is_there = match std::fs::symlink_metadata(&self.directory) {
            Ok(what_is_there) => what_is_there,
            Err(why) if why.kind() == io::ErrorKind::NotFound => {
                return Err(NoDoor::NoDirectory {
                    at: self.directory.clone(),
                });
            }
            Err(why) => {
                return Err(NoDoor::Unreadable {
                    at: self.directory.clone(),
                    why,
                });
            }
        };
        if what_is_there.file_type().is_symlink() {
            return Err(NoDoor::ALink {
                at: self.directory.clone(),
            });
        }
        if !what_is_there.is_dir() {
            return Err(NoDoor::NotADirectory {
                at: self.directory.clone(),
            });
        }
        if what_is_there.uid() != person {
            return Err(NoDoor::SomebodyElses {
                at: self.directory.clone(),
                owner: what_is_there.uid(),
            });
        }
        Ok(())
    }

    /// Clear a socket this crate left behind, and refuse anything else.
    ///
    /// A compositor that stopped without running [`Self::taken_away`] — killed,
    /// or the machine lost power — leaves a socket nothing is listening on, and
    /// `bind` refuses a path that exists. So one that **is a socket** is
    /// removed.
    ///
    /// **Anything that is not a socket is refused rather than removed**, and
    /// that is the point of looking at all. The alternative is a compositor that
    /// unlinks whatever is at a path in a directory, which is a worse thing to
    /// own than a door that will not open. A link is refused for the same
    /// reason, unfollowed: removing it would remove the link and bind over it,
    /// and what it pointed at would be untouched and unexplained.
    fn take_away_a_socket_of_our_own(&self) -> Result<(), NoDoor> {
        let what_is_there = match std::fs::symlink_metadata(&self.socket) {
            Ok(what_is_there) => what_is_there,
            Err(why) if why.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(why) => {
                return Err(NoDoor::Unreadable {
                    at: self.socket.clone(),
                    why,
                });
            }
        };
        if !what_is_there.file_type().is_socket() {
            return Err(NoDoor::SomethingElseIsThere {
                at: self.socket.clone(),
            });
        }
        std::fs::remove_file(&self.socket).map_err(|why| NoDoor::NotBound {
            at: self.socket.clone(),
            why,
        })
    }

    /// Shut the socket to everybody but the person.
    ///
    /// Called once it is bound. The directory being `0750` and the person's is
    /// what makes the moment before this harmless — nobody else can enter it to
    /// find the socket in the first place.
    fn shut_the_socket(&self) -> Result<(), NoDoor> {
        std::fs::set_permissions(&self.socket, Permissions::from_mode(THE_SOCKET_MODE)).map_err(
            |why| NoDoor::NotShutTo {
                at: self.socket.clone(),
                why,
            },
        )
    }
}

/// Why the compositor is serving no application door.
///
/// Every one of these leaves **no socket and nothing listening**. None of them
/// stops a compositor: the window system runs, and `alo-agentd` finding no
/// socket is what makes the three verbs answer `could-not-ask`.
#[derive(Debug, thiserror::Error)]
pub enum NoDoor {
    /// The person's directory is not there.
    ///
    /// It is the daemon's to make (ADR 0017), so this says so rather than
    /// making it — on a machine where it is missing because no daemon is
    /// running, which is a machine where no turn can have authorised anything.
    #[error("{at} is not there; alo-agentd makes it when the person signs in")]
    NoDirectory {
        /// The directory that is missing.
        at: PathBuf,
    },
    /// Something is at the directory's path and it is a symbolic link.
    #[error("{at} is a symbolic link, and whoever can replace it can replace the socket")]
    ALink {
        /// Where the link is.
        at: PathBuf,
    },
    /// Something is at the directory's path and it is not a directory.
    #[error("{at} is not a directory")]
    NotADirectory {
        /// What was looked at.
        at: PathBuf,
    },
    /// The directory belongs to somebody else.
    #[error("{at} belongs to user {owner}, not to the person signed in here")]
    SomebodyElses {
        /// The directory.
        at: PathBuf,
        /// Who owns it.
        owner: u32,
    },
    /// Something is at the socket's path and it is not a socket.
    ///
    /// Told apart from a stale socket, which is removed, because a compositor
    /// that unlinked whatever it found at a path is a worse thing to own than a
    /// door that will not open.
    #[error("{at} is not a socket, and this is not the thing to remove it")]
    SomethingElseIsThere {
        /// What is in the way.
        at: PathBuf,
    },
    /// The machine would not say what is at a path.
    #[error("{at} could not be read: {why}")]
    Unreadable {
        /// What was asked about.
        at: PathBuf,
        /// What the machine said.
        why: io::Error,
    },
    /// The socket would not bind.
    #[error("{at} would not bind: {why}")]
    NotBound {
        /// Where it was to be.
        at: PathBuf,
        /// What the machine said.
        why: io::Error,
    },
    /// The mode would not be set, so the socket exists and is too wide.
    ///
    /// [`ThePlace::prepared`] removes the socket before answering with this, so
    /// by the time a caller reads it there is nothing bound and nothing at the
    /// path: a socket at the default mode is one the agent's group can reach,
    /// and that is the single thing this file exists to prevent.
    #[error("{at} could not be shut to everybody else: {why}")]
    NotShutTo {
        /// The socket.
        at: PathBuf,
        /// What the machine said.
        why: io::Error,
    },
}
