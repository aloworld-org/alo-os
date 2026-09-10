//! Where a person's session is, and whether an environment names theirs.
//!
//! Three strings and one unit name, all of them derived from one number: the
//! uid of the person who signed in. Nothing here reads a variable, opens a
//! socket or asks the machine anything — it is the derivation, and
//! `crate` says why that is the whole of it.

use alo_accounts::Session;

/// Where `logind` puts a person's runtime directory.
///
/// The same fact `alo_secrets::WHERE_SESSIONS_ARE` states, and there is a test
/// at the bottom of this file that the two say one thing. It is repeated here
/// rather than imported because that crate is Linux-only and this one is read
/// by `alo-image`, which checks a unit file on whatever host somebody is
/// working on.
pub const WHERE_SESSIONS_ARE: &str = "/run/user";

/// The variable that names the session's runtime directory.
pub const WHERE_THE_SESSION_IS: &str = "XDG_RUNTIME_DIR";

/// The variable that names the session's bus.
pub const WHERE_THE_BUS_IS: &str = "DBUS_SESSION_BUS_ADDRESS";

/// One person's session, as the machine spells it.
///
/// A value of this type is a **derivation and not an observation**: holding one
/// says where that person's session would be, never that they are signed in.
/// [`crate`] says why the two are kept apart — the bus that is really there is
/// `alo-secrets`' question, asked of the kernel, and a type here that answered
/// it would be a second place a daemon could learn where to connect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheSessionsEnvironment {
    /// Whose session it is.
    person: u32,
}

impl TheSessionsEnvironment {
    /// The session this sign-in opens.
    ///
    /// The only constructor that takes a fact about a person rather than a
    /// number, and the reason this crate depends on `alo-accounts`: a
    /// [`Session`] cannot exist carrying a uid the machine description does not
    /// name, so an environment derived from one is the described person's
    /// session or it is nothing.
    #[must_use]
    pub const fn of(session: &Session) -> Self {
        Self {
            person: session.uid(),
        }
    }

    /// The same, for the person a machine is described as belonging to.
    ///
    /// This is what a *check* takes: `alo-image` reads `[logins].person` out of
    /// `/etc/alo/agentd.toml` and asks whether the unit file agrees with it, and
    /// there is no sign-in anywhere near that question. It derives exactly what
    /// [`TheSessionsEnvironment::of`] derives, because the number is the same
    /// number — which is the whole of what `Session::opened` refuses to let
    /// differ.
    #[must_use]
    pub const fn for_person(person: u32) -> Self {
        Self { person }
    }

    /// Whose session this is.
    #[must_use]
    pub const fn person(&self) -> u32 {
        self.person
    }

    /// The session's runtime directory: `/run/user/<uid>`.
    ///
    /// Text rather than a `PathBuf`, and deliberately: what this is *for* is the
    /// value of an environment variable and the right-hand side of a line in a
    /// unit file. Both are text, and a type that had to be rendered back into
    /// text at every use would be a second spelling waiting to happen.
    #[must_use]
    pub fn runtime_directory(&self) -> String {
        format!("{WHERE_SESSIONS_ARE}/{}", self.person)
    }

    /// The person's bus, at `/run/user/<uid>/bus`.
    #[must_use]
    pub fn bus(&self) -> String {
        format!("{}/bus", self.runtime_directory())
    }

    /// The address a client is given for that bus.
    #[must_use]
    pub fn bus_address(&self) -> String {
        addressed(&self.bus())
    }

    /// The systemd unit that *is* this person's session on the machine:
    /// `user@<uid>.service`.
    ///
    /// The daemon's unit is pulled in by this one and bound to it, which is how
    /// starting under a session and stopping when it ends are one decision
    /// rather than two pieces of code. `alo-image` checks that the shipped unit
    /// really says so.
    #[must_use]
    pub fn their_manager(&self) -> String {
        format!("user@{}.service", self.person)
    }

    /// Everything a process started into this session is told, and nothing else.
    ///
    /// Two variables. A daemon that needed a third would be a daemon whose
    /// environment had become a configuration file, and where alo OS is
    /// configured is `/etc/alo/agentd.toml`.
    #[must_use]
    pub fn variables(&self) -> [(&'static str, String); 2] {
        [
            (WHERE_THE_SESSION_IS, self.runtime_directory()),
            (WHERE_THE_BUS_IS, self.bus_address()),
        ]
    }

    /// Whether this text names *this* person's runtime directory.
    ///
    /// Equality, not a prefix and not a parse. A rule that pulled a uid out of
    /// whatever it was handed would be a rule with an opinion about strings
    /// nobody on this machine writes; a rule that compares against the one
    /// spelling this machine uses refuses everything else without having to
    /// understand any of it.
    #[must_use]
    pub fn names_their_session(&self, said: &str) -> bool {
        said == self.runtime_directory()
    }

    /// Whether this text names *this* person's bus.
    ///
    /// Both spellings are accepted — the bare path and the D-Bus address — and
    /// nothing else is, for [`TheSessionsEnvironment::names_their_session`]'s
    /// reason. Two are here because both really occur: a unit file writes the
    /// address, and a shell that exported the path would be pointing a daemon
    /// at the same socket by the other name.
    #[must_use]
    pub fn names_their_bus(&self, said: &str) -> bool {
        said == self.bus_address() || said == self.bus()
    }
}

/// A socket path, as a D-Bus address.
///
/// One line, in one place, because `alo-secrets` hands a client the same
/// sentence for a bus it has checked and the two must not drift — the test
/// below is where that is asked of both.
fn addressed(path: &str) -> String {
    format!("unix:path={path}")
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A person, signed in, at this number.
    fn signed_in_at(uid: u32) -> Session {
        let mut store = alo_accounts::Accounts::none().unwrap();
        store
            .created("ada", uid, "correct horse battery staple")
            .unwrap();
        let who = store
            .signs_in("ada", "correct horse battery staple")
            .unwrap();
        Session::opened(who, uid).unwrap()
    }

    /// **The environment is the session's**, every string of it, from the value
    /// a sign-in produced rather than from a number this test typed.
    #[test]
    fn what_a_session_hands_a_daemon_is_derived_from_the_session() {
        let session = signed_in_at(1000);
        let environment = TheSessionsEnvironment::of(&session);

        assert_eq!(environment.person(), 1000);
        assert_eq!(environment.runtime_directory(), "/run/user/1000");
        assert_eq!(environment.bus(), "/run/user/1000/bus");
        assert_eq!(environment.bus_address(), "unix:path=/run/user/1000/bus");
        assert_eq!(environment.their_manager(), "user@1000.service");
        assert_eq!(
            environment.variables(),
            [
                ("XDG_RUNTIME_DIR", "/run/user/1000".to_owned()),
                (
                    "DBUS_SESSION_BUS_ADDRESS",
                    "unix:path=/run/user/1000/bus".to_owned()
                ),
            ]
        );
    }

    /// **A sign-in and a machine description that name one person derive one
    /// environment.** The two constructors exist because two callers have two
    /// facts, not because there are two answers.
    #[test]
    fn the_described_person_and_the_signed_in_one_are_one_session() {
        let session = signed_in_at(1000);
        assert_eq!(
            TheSessionsEnvironment::of(&session),
            TheSessionsEnvironment::for_person(1000)
        );
        assert_ne!(
            TheSessionsEnvironment::of(&session),
            TheSessionsEnvironment::for_person(1001)
        );
    }

    /// **Somebody else's session is not theirs**, whichever of the two names it
    /// arrives under — which is the refusal the daemon is built on: a unit file
    /// edited to point the person's daemon at root's bus is a machine that must
    /// not start rather than one that quietly connects.
    #[test]
    fn another_persons_session_is_refused_under_either_name() {
        let theirs = TheSessionsEnvironment::for_person(1000);

        assert!(theirs.names_their_session("/run/user/1000"));
        assert!(theirs.names_their_bus("/run/user/1000/bus"));
        assert!(theirs.names_their_bus("unix:path=/run/user/1000/bus"));

        assert!(!theirs.names_their_session("/run/user/0"));
        assert!(!theirs.names_their_bus("unix:path=/run/user/0/bus"));
        assert!(!theirs.names_their_bus("/run/user/0/bus"));
    }

    /// **And a name that is nearly theirs is not theirs.** A prefix rule would
    /// have accepted every one of these, and each of them is a different
    /// directory on a real machine.
    #[test]
    fn a_session_that_merely_begins_the_same_is_refused() {
        let theirs = TheSessionsEnvironment::for_person(100);

        for nearly in [
            "/run/user/1000",
            "/run/user/100/",
            "/run/user/100/../0",
            "/run/user/1001",
            "",
        ] {
            assert!(
                !theirs.names_their_session(nearly),
                "{nearly} was taken for uid 100's session"
            );
        }
        for nearly in [
            "unix:path=/run/user/1000/bus",
            "unix:path=/run/user/100/bus,guid=x",
            "unix:abstract=/run/user/100/bus",
            "",
        ] {
            assert!(
                !theirs.names_their_bus(nearly),
                "{nearly} was taken for uid 100's bus"
            );
        }
    }

    /// **This crate and `alo-secrets` say one thing about where a bus is**, on
    /// the host where both exist. Two crates deriving the same path is the
    /// mistake this test is here to catch the day somebody changes one of them.
    #[cfg(target_os = "linux")]
    #[test]
    fn where_a_bus_is_agrees_with_the_crate_that_opens_one() {
        use alo_secrets::TheBus;

        for person in [0, 1000, 60989] {
            let environment = TheSessionsEnvironment::for_person(person);
            assert_eq!(
                environment.bus(),
                TheBus::of(person).to_string_lossy(),
                "the two crates disagree about where uid {person}'s bus is"
            );
            assert_eq!(
                environment.runtime_directory(),
                format!("{}/{person}", alo_secrets::WHERE_SESSIONS_ARE)
            );
        }
    }

    /// **And one thing about how a bus is said to a client.** `alo-secrets`
    /// hands a *checked* bus to whatever connects, so the address is asked of a
    /// real socket this test owns rather than of a formatting rule copied out of
    /// that crate.
    #[cfg(target_os = "linux")]
    #[test]
    fn how_a_bus_is_said_agrees_with_the_crate_that_says_it() {
        use alo_secrets::TheBus;

        let place = std::env::temp_dir().join(format!("alo-entering-{}", std::process::id()));
        drop(std::fs::remove_dir_all(&place));
        std::fs::create_dir_all(&place).unwrap();
        let at = place.join("bus");
        let listening = std::os::unix::net::UnixListener::bind(&at).unwrap();

        let ours = rustix::process::getuid().as_raw();
        let found = TheBus::at(&at, ours).unwrap();
        assert_eq!(found.as_an_address(), addressed(at.to_str().unwrap()));

        drop(listening);
        drop(std::fs::remove_dir_all(&place));
    }
}
