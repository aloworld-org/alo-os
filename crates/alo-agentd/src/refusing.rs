//! Everything this crate refuses, and who reads it.
//!
//! Eight types, divided by what somebody has to do about them. [`NotDescribed`]
//! is the file a machine is described by, [`NotTwoSides`] and [`NotAUser`] are
//! a machine described wrongly, [`NotBound`] is a machine whose socket cannot be
//! put where it belongs, [`NotACaller`] is one connection that will not be served,
//! [`NotHeard`] is one connection that cannot go on being read, [`NotServed`]
//! is the service itself stopping, and [`NotStarted`] is the process: the one
//! that gathers the rest, because a process ends in exactly one of them.
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

/// Why this machine's description was not believed.
///
/// Every one of these ends with nothing running: a service that started under a
/// description it could not read would be a service running under whatever the
/// last one said, which is the failure this type exists to prevent rather than a
/// degraded mode.
///
/// The first four are about the **file** rather than about what is in it, and
/// they are here for the reason [`NotBound::SomebodyElses`] is: whoever can
/// write the file that names the agent's login can name themselves the agent.
#[derive(Debug, Error)]
pub enum NotDescribed {
    /// The description is a symbolic link.
    #[error(
        "{at} is a symbolic link, so what alo-agentd would really read is decided by whoever can change the link; make it a file"
    )]
    ALink {
        /// The link.
        at: PathBuf,
    },
    /// Something is at that path and it is not an ordinary file.
    #[error("{at} is not an ordinary file; alo-agentd is described by one file and reads no other")]
    NotAFile {
        /// What is in the way.
        at: PathBuf,
    },
    /// The description belongs to somebody who is neither this process nor root.
    #[error(
        "{at} belongs to user {owner}, who is neither the person alo-agentd runs as nor root, and could name themselves this machine's agent; give the file to root or to the person"
    )]
    SomebodyElses {
        /// The description.
        at: PathBuf,
        /// Who owns it.
        owner: u32,
    },
    /// The description can be written by somebody who does not own it.
    #[error(
        "{at} is mode {mode:04o}, so somebody who does not own it can rewrite what this machine says about itself; take the write bits off the group and off everybody else"
    )]
    Loose {
        /// The description.
        at: PathBuf,
        /// The permission bits it really has.
        mode: u32,
    },
    /// The description could not be read.
    #[error("could not read {at}: {why}")]
    Unreadable {
        /// The description.
        at: PathBuf,
        /// What the machine said.
        why: std::io::Error,
    },
    /// The description is not what this alo OS reads.
    #[error(
        "{at} says format {format}, and this alo-agentd reads format {reads}; a description written for a newer alo OS is not guessed at"
    )]
    AnotherFormat {
        /// The description.
        at: PathBuf,
        /// What it says it is.
        format: u32,
        /// What this service reads.
        reads: u32,
    },
    /// The description is not the shape a description is.
    #[error("{at} is not a machine description: {why}")]
    NotUnderstood {
        /// The description.
        at: PathBuf,
        /// What the reader said, naming the line and the key.
        why: Box<toml::de::Error>,
    },
    /// The two logins are not two.
    #[error("{0}")]
    NotTwoSides(#[from] NotTwoSides),
    /// A number in the description is not a user or a group.
    #[error("{0}")]
    NotAUser(#[from] NotAUser),
    /// The agent has no name for the grants to be about.
    #[error(
        "agent.name is empty, so no grant could name this machine's agent; give the agent the name its grants are made to"
    )]
    Anonymous,
    /// A length of time that is no time at all.
    #[error(
        "{what} is 0, and a turn or a proposal that lasts no time at all is refused at the moment it begins rather than served; give it a length in whole seconds"
    )]
    NoTimeAtAll {
        /// The key in the description.
        what: &'static str,
    },
    /// A length of time longer than a turn or a proposal may be.
    #[error(
        "{what} is {seconds} seconds, and the longest either may be is {at_most}; an approval is never a session (CLAUDE.md)"
    )]
    TooLong {
        /// The key in the description.
        what: &'static str,
        /// What it said.
        seconds: u64,
        /// The longest it may be.
        at_most: u64,
    },
    /// A path that is not absolute.
    #[error(
        "{what} is {at}, which is relative to whatever directory alo-agentd happened to be started in; give it a path beginning with /"
    )]
    NotAbsolute {
        /// The key in the description.
        what: &'static str,
        /// What it said.
        at: PathBuf,
    },
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
    /// The directory every person's door goes in is not on this machine.
    ///
    /// ADR 0017 gives it to the image, so this is a machine that booted without
    /// it rather than one nobody has set up: the daemon says which directory and
    /// who makes it, and makes nothing.
    #[error(
        "{at} is not there, and alo-agentd does not create it; the image makes it at boot through tmpfiles.d, 0755 and owned by root, and every person's socket goes in a directory of their own beneath it (ADR 0017)"
    )]
    NoParent {
        /// The directory the image makes.
        at: PathBuf,
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
    /// A thread went into a turn's boundary and could not be brought back out.
    ///
    /// The second refusal here that is a promise rather than a fault, and it is
    /// the worse of the two. There is a thread in this process inside a control
    /// group belonging to a turn that has ended, so the kernel refuses it
    /// everything outside a grant that no longer exists — which fails closed,
    /// and is why `alo-bounding` leaves it in there rather than taking the
    /// kernel's entry away underneath it. A service that carried on would be a
    /// service leaking a thread per turn into a boundary nobody can lift.
    #[error(
        "a thread of this service went into a turn's boundary and could not be brought back out, so nothing further will be done; the reason is above, and alo-agentd has to be started again"
    )]
    AThreadIsInsideATurn,
}

/// Why there is no service on this machine.
///
/// The process's own refusal, and the one type that gathers the others: a
/// process ends in exactly one of these, so this is what a service log holds
/// when `alo-agentd` did not come up. Every one of them leaves nothing running
/// — no socket bound, no turn begun, nothing written down — which is what makes
/// a failed start safe to retry rather than something to clean up after.
///
/// It is **not** how a running service reports trouble. A stranger at the door,
/// a message that is not a request and a caller that hangs up are all served
/// and survived, and [`NotServed`] is the narrower type for the ones that end a
/// service that had already started.
#[derive(Debug, Error)]
pub enum NotStarted {
    /// The machine would not say who this process is running as.
    #[error("could not tell who alo-agentd is running as: {0}")]
    NotAUser(#[from] NotAUser),
    /// This process is root.
    ///
    /// ADR 0001 §2, and the check `crate::side` deliberately left here:
    /// refusing a *number in a file* that says the agent is root is one thing,
    /// and it has been done since item 21e. This is the other — a process that
    /// really is root, whatever any file says.
    #[error(
        "alo-agentd is running as root, and it holds a person's authority rather than a machine's (ADR 0001 §2); start it as a user service of the signed-in person"
    )]
    AsRoot,
    /// This machine's description was not believed.
    #[error("{0}")]
    NotDescribed(#[from] NotDescribed),
    /// alo OS's own words contradict each other.
    ///
    /// Not this machine's fault and not fixable on it — see
    /// `alo_saying::NotCollected`.
    #[error("{0}")]
    NotCollected(#[from] alo_saying::NotCollected),
    /// This crate's own three strings could not be added to the machine's
    /// vocabulary.
    #[error("alo-agentd's own words could not be declared: {0}")]
    NotDeclared(#[from] crate::words::WordsError),
    /// There is no record to write what happens down in.
    ///
    /// The one refusal here that is not this crate's English, and the reason
    /// the vocabulary is loaded before the record is opened: `alo-keeping`
    /// already words a record that will not open, and a second sentence written
    /// here would be this file disagreeing with the one a person is shown.
    #[error(
        "nothing that happens on this machine could be written down: {said}; alo-agentd will not run without a record"
    )]
    NoRecord {
        /// What `alo-keeping` said, in the language this machine loaded.
        said: String,
    },
    /// The socket could not be put where it belongs.
    #[error("{0}")]
    NotBound(#[from] NotBound),
    /// The verbs this machine offers could not be declared.
    #[error("the verbs this machine can carry out could not be declared: {0}")]
    NoVerbs(#[from] alo_files::Declaring),
    /// The catalogue built into this image does not hold.
    ///
    /// English rather than a word from the vocabulary, and for the reason
    /// `alo-models` gives its own `CatalogueError`: the catalogue is a file
    /// signed with the release, so whoever reads this is fixing that file
    /// rather than using the machine. A service that started without one would
    /// be a service that cannot say which models it offers while looking
    /// perfectly healthy.
    #[error(
        "the model catalogue built into this image does not hold: {why}; it is shipped with the release, so this is a build to fix rather than a machine"
    )]
    NoCatalogue {
        /// What `alo-models` said about the file.
        why: String,
    },
    /// There is no boundary to run a turn's work inside.
    ///
    /// ADR 0015's *a turn whose boundary cannot be applied does not run*, asked
    /// of the service rather than of one turn: everything a machine could not
    /// do about it is the same on the next turn as on this one, so a daemon that
    /// started anyway would be a daemon refusing everything an agent asked for
    /// while looking perfectly healthy. What is wrong is underneath the daemon —
    /// a kernel that publishes no type information, one whose security modules
    /// do not include `bpf`, or a service with no control group subtree of its
    /// own — and `docs/hardware.md` has the three checks in the order they fail
    /// in.
    #[error(
        "a turn's work cannot be bounded on this machine: {why}; alo-agentd will not run without a boundary, because a turn that cannot be bounded does not run (ADR 0015)"
    )]
    NoBoundary {
        /// What `alo-bounding` said, in English, for whoever is standing this
        /// machine up.
        why: String,
    },
    /// There is no way to ask this service to stop.
    #[error(
        "could not make the pair a stop arrives on: {why}; alo-agentd is not started without one, because a service that cannot be asked to stop can only be killed"
    )]
    NoStop {
        /// What the machine said.
        why: std::io::Error,
    },
    /// `SIGTERM` could not be made to ask the service to stop.
    #[error(
        "could not arrange for SIGTERM to stop alo-agentd: {why}; without it a stop would kill the service mid-turn"
    )]
    NoHandler {
        /// What the machine said.
        why: std::io::Error,
    },
    /// The service started and then stopped for a reason that is the machine's.
    #[error("{0}")]
    NotServed(#[from] NotServed),
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
            at: PathBuf::from("/run/alo/1000"),
            owner: 1001,
        }
        .to_string();
        assert!(said.contains("/run/alo/1000"));
        assert!(said.contains("1001"));

        let said = NotTwoSides::OneUser { uid: 1000 }.to_string();
        assert!(said.contains("1000"));

        let said = NotACaller::Stranger { uid: 65534 }.to_string();
        assert!(said.contains("65534"));
    }

    /// **And so does every refusal about the description**, which is read by
    /// the same person at the same hour: the file, the number, and the key.
    #[test]
    fn every_refusal_about_a_description_names_what_to_go_and_change() {
        let said = NotDescribed::SomebodyElses {
            at: PathBuf::from("/etc/alo/agentd.toml"),
            owner: 1001,
        }
        .to_string();
        assert!(said.contains("/etc/alo/agentd.toml"), "{said}");
        assert!(said.contains("1001"), "{said}");

        let said = NotDescribed::Loose {
            at: PathBuf::from("/etc/alo/agentd.toml"),
            mode: 0o666,
        }
        .to_string();
        assert!(said.contains("0666"), "{said}");

        let said = NotDescribed::NotAbsolute {
            what: "record.path",
            at: PathBuf::from("record"),
        }
        .to_string();
        assert!(said.contains("record.path"), "{said}");
    }

    /// **A missing `/run/alo` says who makes it**, because whoever reads it is
    /// looking at a machine that booted without something the image was meant to
    /// put there — and *no such file or directory* would send them to the daemon
    /// instead (ADR 0017).
    #[test]
    fn a_missing_root_names_the_directory_and_who_makes_it() {
        let said = NotBound::NoParent {
            at: PathBuf::from("/run/alo"),
        }
        .to_string();
        assert!(said.contains("/run/alo"), "{said}");
        assert!(said.contains("tmpfiles.d"), "{said}");
        assert_ne!(
            said,
            NotBound::NoDirectory {
                at: PathBuf::from("/run/alo/1000"),
                why: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
            }
            .to_string(),
            "the image's directory and the person's are two things to go and change"
        );
    }

    /// The two logins and the two numbers that are not users travel into a
    /// description's refusal as themselves, so what a reader is told about the
    /// file is what `crate::side` and `crate::caller` already said.
    #[test]
    fn what_the_logins_refused_is_carried_into_the_description() {
        let refused = NotDescribed::from(NotTwoSides::OneUser { uid: 1000 });
        assert!(refused.to_string().contains("1000"), "{refused}");

        let refused = NotDescribed::from(NotAUser::NoSuchGroup);
        assert!(refused.to_string().contains("group"), "{refused}");
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

    /// **A process that did not start says what to go and change too**, and
    /// each of these is a different place to go: a service file, a disk, a
    /// kernel that would not take a handler.
    #[test]
    fn a_process_that_did_not_start_says_what_to_go_and_change() {
        let said = NotStarted::AsRoot.to_string();
        assert!(said.contains("root"), "{said}");
        assert!(said.contains("user service"), "{said}");

        let said = NotStarted::NoRecord {
            said: "there is no room left on the disk holding the record".to_owned(),
        }
        .to_string();
        assert!(said.contains("no room left"), "{said}");

        let said = NotStarted::NoHandler {
            why: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        }
        .to_string();
        assert!(said.contains("SIGTERM"), "{said}");
    }

    /// **What a process gathers, it carries whole.** Every refusal it did not
    /// make itself reaches the service log as the sentence whoever made it
    /// wrote — the rule the rest of this workspace keeps about a person's
    /// language, asked here about the one reader who has none.
    #[test]
    fn a_process_carries_the_refusal_somebody_else_made() {
        let stranger = NotACaller::Stranger { uid: 65534 };
        let inside = NotServed::NotTaken(stranger).to_string();
        assert_eq!(
            NotStarted::NotServed(NotServed::NotTaken(NotACaller::Stranger { uid: 65534 }))
                .to_string(),
            inside
        );

        let missing = NotBound::NoParent {
            at: PathBuf::from("/run/alo"),
        };
        assert_eq!(
            NotStarted::NotBound(NotBound::NoParent {
                at: PathBuf::from("/run/alo")
            })
            .to_string(),
            missing.to_string()
        );
    }
}
