//! Everything this crate refuses, and who reads it.
//!
//! Six types, divided by what somebody has to do about them. [`NotTwoSides`]
//! and [`NotAUser`] are a machine described wrongly, [`NotBound`] is a machine
//! whose socket cannot be put where it belongs, [`NotACaller`] is one
//! connection that will not be served, [`NotHeard`] is one connection that
//! cannot go on being read, and [`NotServed`] is the service itself stopping.
//!
//! # The line between one connection and the service
//!
//! [`NotACaller`] and [`NotHeard`] are about somebody at the other end of a
//! socket, and the service survives every one of them: the connection goes and
//! the machine goes on. [`NotServed`] is the other kind — a turn that could not
//! be begun, a wait the kernel would not take, a record that cannot be written
//! — and it is what ends the process. [`NotACaller::is_only_this_connection`]
//! is where the first of those two questions is answered, because a service
//! that guessed would either die on a stranger or serve on with no evidence.
//!
//! # These are English, and that is the decision rather than an omission
//!
//! Every refusal in the rest of this workspace is an `alo_strings::Said`,
//! because a person in front of the machine reads it. Nothing here is read by
//! that person. *This directory belongs to somebody else*, *this login is not
//! in that group*, *another daemon is already listening*: those are read out of
//! a service log by whoever is standing the machine up, and they are
//! `alo_shortcuts::DefaultsError`'s reader — the one fixing a release rather
//! than the one using it. `crates/alo-agentd/src/lib.rs` has the whole
//! argument, including what happens to [`NotACaller::Stranger`] on the day
//! there is a shell to show it on.
//!
//! # They say what to do
//!
//! `alo-models`' rule, and every sentence below is held to it: a refusal names
//! the path, the user or the group somebody has to go and change, because a
//! daemon that says *permission denied* has told whoever is on call the one
//! thing they already knew.

use std::path::PathBuf;

use thiserror::Error;

/// A number that is not a user or a group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum NotAUser {
    /// `-1`, which is what a Unix call answers with when there is no user.
    #[error(
        "-1 is what a Unix call answers with when there is no user at all; name the user alo-agentd is to run as"
    )]
    NoSuchUser,
    /// `-1`, arriving where a group belongs.
    #[error(
        "-1 is what a Unix call answers with when there is no group at all; name the group the agent is in"
    )]
    NoSuchGroup,
}

/// Why this machine cannot be described as having two sides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum NotTwoSides {
    /// The person and the agent are one login.
    #[error(
        "the person and the agent are both user {uid}, so the socket would have one door and not two; give the agent a user of its own"
    )]
    OneUser {
        /// The user both were named as.
        uid: u32,
    },
    /// The agent is root.
    #[error(
        "the agent is named as root, which is authority the person themselves does not have (ADR 0001 §2); give the agent an unprivileged user"
    )]
    AgentAsRoot,
}

/// Why there is no socket to listen on.
///
/// In every one of these, nothing is listening and no socket belonging to
/// anybody else has been removed.
#[derive(Debug, Error)]
pub enum NotBound {
    /// The machine would not say who this process is running as.
    #[error("could not tell who alo-agentd is running as: {0}")]
    NotAUser(#[from] NotAUser),
    /// This process is not the user it was told the person is.
    #[error(
        "alo-agentd is running as user {us} but was told the person is user {told}; it must run as the person it opens the person's door for (ADR 0001 §2)"
    )]
    NotThePerson {
        /// The user this process is really running as.
        us: u32,
        /// The user it was told the person is.
        told: u32,
    },
    /// The directory the socket goes in could not be made.
    #[error("could not make the directory {at} for the socket: {why}")]
    NoDirectory {
        /// Where it would have gone.
        at: PathBuf,
        /// What the machine said.
        why: std::io::Error,
    },
    /// Something is at that path and it is not a directory.
    #[error(
        "{at} is not a directory; move whatever is there, and the socket will be made beside it"
    )]
    NotADirectory {
        /// What is in the way.
        at: PathBuf,
    },
    /// The directory is a symbolic link, so where the socket really goes is
    /// whatever the link says today.
    #[error(
        "{at} is a symbolic link, so where the socket would really go is decided by whoever can change the link; make it a directory"
    )]
    ALink {
        /// The link.
        at: PathBuf,
    },
    /// The directory belongs to somebody else.
    #[error(
        "{at} belongs to user {owner}, who could replace the socket in it and be talked to instead of alo-agentd; give the person a directory of their own"
    )]
    SomebodyElses {
        /// The directory.
        at: PathBuf,
        /// Who owns it.
        owner: u32,
    },
    /// Whatever is at the path could not be looked at.
    #[error("could not read {at}: {why}")]
    Unreadable {
        /// The path.
        at: PathBuf,
        /// What the machine said.
        why: std::io::Error,
    },
    /// The path could not be handed to the agent's group.
    #[error(
        "could not give {at} to group {group}: {why}; the person alo-agentd runs as has to be a member of that group"
    )]
    NotOurGroup {
        /// The path.
        at: PathBuf,
        /// The group it was to be handed to.
        group: u32,
        /// What the machine said.
        why: std::io::Error,
    },
    /// The permissions could not be set.
    #[error("could not set the permissions on {at}: {why}")]
    NotShutTo {
        /// The path.
        at: PathBuf,
        /// What the machine said.
        why: std::io::Error,
    },
    /// Another daemon answered on that socket.
    #[error(
        "something is already listening on {at}; stop the alo-agentd that is running before starting another"
    )]
    AlreadyRunning {
        /// The socket.
        at: PathBuf,
    },
    /// Something that is not a socket is where the socket goes.
    #[error(
        "{at} is not a socket, and alo-agentd will not delete a file it did not make; move it out of the way"
    )]
    NotOurSocket {
        /// What is in the way.
        at: PathBuf,
    },
    /// A socket nobody is listening on could not be removed.
    #[error("nothing is listening on the socket {at} and it could not be removed either: {why}")]
    NotRemoved {
        /// The socket.
        at: PathBuf,
        /// What the machine said.
        why: std::io::Error,
    },
    /// The socket could not be bound.
    #[error("could not listen on {at}: {why}")]
    NotBoundTo {
        /// The socket.
        at: PathBuf,
        /// What the machine said.
        why: std::io::Error,
    },
}

/// Why a connection will not be served.
#[derive(Debug, Error)]
pub enum NotACaller {
    /// The connection could not be accepted at all.
    #[error("could not accept a connection: {why}")]
    NotAccepted {
        /// What the machine said.
        why: std::io::Error,
    },
    /// The kernel would not say who was at the other end — so nobody is
    /// served, because who is calling is the whole of what the door decides on.
    #[error("the kernel would not say who is at the other end of a connection: {why}")]
    NotAsked {
        /// What the machine said.
        why: std::io::Error,
    },
    /// What the kernel said was not a user.
    #[error("the kernel answered with something that is not a user: {0}")]
    NotAUser(#[from] NotAUser),
    /// Somebody who is neither the person nor the agent.
    #[error(
        "user {uid} is neither the person nor the agent on this machine, so the connection was closed without an answer"
    )]
    Stranger {
        /// Who it was.
        uid: u32,
    },
}

impl NotACaller {
    /// Whether this is one connection going wrong rather than the machine.
    ///
    /// Only a stranger. A machine that would not accept, a kernel that would
    /// not say who is calling and a number that is not a user are all the
    /// machine, and a service that carried on through them would be one
    /// answering the door without knowing which door it is — which is the whole
    /// of what [`crate::Sides`] decides.
    ///
    /// It is a method rather than a `matches!` at the one place that asks,
    /// because the day a fourth way of not being a caller is added, this is
    /// where somebody has to answer whether the service lives through it.
    #[must_use]
    pub const fn is_only_this_connection(&self) -> bool {
        matches!(self, Self::Stranger { .. })
    }
}

/// Why a connection cannot go on being read.
///
/// All three end the connection, and two of them are **answered in words
/// first**: `docs/contracts/daemon-protocol.md` says a message that is not
/// acted on is refused and never dropped, and a line this service cannot go on
/// reading is still a message somebody sent. The words are
/// `alo_protocol::NotUnderstood`'s rather than this crate's — see
/// [`NotHeard::what_to_say`] — so nothing here is a sentence a person reads.
#[derive(Debug, Error)]
pub enum NotHeard {
    /// A line with no ending inside the bound this machine reads to.
    #[error(
        "a message of {was} bytes arrived with no line ending in it; send one message per line"
    )]
    TooLong {
        /// How much was read before the read gave up.
        was: usize,
    },
    /// Bytes that are not text.
    #[error("a message arrived that is not text; send UTF-8")]
    NotText,
    /// The machine would not read or write the connection.
    #[error("a connection could not be read or answered on: {0}")]
    Broken(std::io::Error),
}

/// Why the service stopped.
///
/// Every one of these is **the machine** rather than a client: a stranger, a
/// message that is not a request and a caller that hangs up mid-message are all
/// served and survived, and none of them reaches this type.
#[derive(Debug, Error)]
pub enum NotServed {
    /// A turn could not be begun, so the connection that arrived cannot be
    /// served and neither can the next one.
    ///
    /// The grant an invocation makes is the one thing a turn cannot start
    /// without, and nothing about the next agent would be different.
    #[error(
        "a turn could not be begun for the agent this machine has: {why:?}; check the agent's name and the grants it holds"
    )]
    NoTurn {
        /// What the capability model said.
        why: alo_capability::GrantError,
    },
    /// The kernel would not wait on the socket and its connections.
    #[error("could not wait on the socket and its connections: {why}")]
    NotWaiting {
        /// What the machine said.
        why: std::io::Error,
    },
    /// A connection could not be taken, for a reason that is not one caller.
    #[error("could not take a connection: {0}")]
    NotTaken(#[from] NotACaller),
    /// Something happened and there is no record of it.
    ///
    /// The one refusal here that is a promise rather than a fault. `CLAUDE.md`
    /// asks that every execution and every refusal leaves a record, and a
    /// service that went on acting once it could not write one would be doing
    /// exactly what that sentence exists to prevent. What is wrong is on the
    /// disk and `alo-keeping` has already said what; this is the service
    /// refusing to be the thing that carries on regardless.
    #[error(
        "something happened on this machine and could not be written down, so nothing further will be done; make room on the disk holding the record and start alo-agentd again"
    )]
    NothingIsWrittenDown,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A refusal names the thing somebody has to go and change.** Every one
    /// of these is read by a person on call at an hour they did not choose, and
    /// a path or a number they can act on is the difference between a fix and
    /// an investigation.
    #[test]
    fn every_refusal_names_what_to_go_and_change() {
        let said = NotBound::SomebodyElses {
            at: PathBuf::from("/run/user/1000/alo"),
            owner: 1001,
        }
        .to_string();
        assert!(said.contains("/run/user/1000/alo"));
        assert!(said.contains("1001"));

        let said = NotTwoSides::OneUser { uid: 1000 }.to_string();
        assert!(said.contains("1000"));

        let said = NotACaller::Stranger { uid: 65534 }.to_string();
        assert!(said.contains("65534"));
    }

    /// A number that is not a user arrives at a connection as a refusal of that
    /// connection, rather than as something a caller could be served under.
    #[test]
    fn a_number_that_is_not_a_user_refuses_the_connection() {
        let refusal = NotACaller::from(NotAUser::NoSuchUser);
        assert!(matches!(
            refusal,
            NotACaller::NotAUser(NotAUser::NoSuchUser)
        ));
    }

    /// The two configuration refusals say which of the two sides is wrong,
    /// because *the agent is root* and *both are one login* are different
    /// mornings for whoever is reading them.
    #[test]
    fn the_two_configuration_refusals_are_told_apart() {
        assert_ne!(
            NotTwoSides::AgentAsRoot.to_string(),
            NotTwoSides::OneUser { uid: 0 }.to_string()
        );
        assert!(NotTwoSides::AgentAsRoot.to_string().contains("ADR 0001"));
    }

    /// **A stranger is the only one connection here**, and every other way of
    /// not being a caller stops the service. A machine that would not say who
    /// is calling cannot answer the door at all, because which door it is *is*
    /// who is calling — so carrying on would be serving somebody unidentified.
    #[test]
    fn only_a_stranger_is_one_connection_rather_than_the_machine() {
        assert!(NotACaller::Stranger { uid: 65534 }.is_only_this_connection());

        for why in [
            NotACaller::NotAccepted {
                why: std::io::Error::from(std::io::ErrorKind::ConnectionAborted),
            },
            NotACaller::NotAsked {
                why: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
            },
            NotACaller::NotAUser(NotAUser::NoSuchUser),
        ] {
            assert!(!why.is_only_this_connection(), "{why}");
        }
    }

    /// **A service that stopped says which kind of thing stopped it**, because
    /// *make room on the disk* and *give the agent a name the grants know* send
    /// whoever is on call to two different places.
    #[test]
    fn a_service_that_stopped_says_what_to_go_and_change() {
        let said = NotServed::NothingIsWrittenDown.to_string();
        assert!(said.contains("written down"));
        assert!(said.contains("disk"), "{said}");

        let said = NotServed::NotTaken(NotACaller::Stranger { uid: 65534 }).to_string();
        assert!(said.contains("65534"), "{said}");
    }

    /// **A line that cannot be read says how much arrived**, which is what
    /// tells whoever is looking at it apart from a client sending nothing at
    /// all.
    #[test]
    fn a_line_that_could_not_be_read_says_how_much_of_it_there_was() {
        let said = NotHeard::TooLong { was: 1_048_577 }.to_string();
        assert!(said.contains("1048577"), "{said}");
        assert_ne!(said, NotHeard::NotText.to_string());
    }
}
