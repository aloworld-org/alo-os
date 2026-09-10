//! The session this process was started into, and the one thing it refuses.
//!
//! `alo-agentd` runs as the signed-in person (ADR 0001 §2) and reaches their
//! keyring over their own session bus (ADR 0022). On a machine that has booted,
//! what puts it in that session is the unit file: it is pulled in by
//! `user@<uid>.service`, bound to it, and given `XDG_RUNTIME_DIR` and
//! `DBUS_SESSION_BUS_ADDRESS` naming `/run/user/<uid>`. `alo-entering` is where
//! those strings are derived, and `alo-image` checks the shipped unit against
//! it.
//!
//! This file is the same question asked from inside the process, once, before
//! anything is served.
//!
//! # It refuses a wrong answer; it never takes an answer from here
//!
//! **Nothing in this file tells anything where to connect.** `alo-secrets`
//! derives the bus from the daemon's own uid and has no parameter an
//! environment variable could arrive through, which is a property of its
//! signatures rather than a promise — and that stays exactly as it was. What
//! this adds is a refusal: if the environment names a session, it must be the
//! person's, or this process stops before it opens a socket.
//!
//! The difference matters more than it looks. A daemon that *read* the variable
//! could be pointed at root's bus by one edited line in a unit file, in the file
//! nobody reviews, on a machine whose whole claim is that the service talking to
//! your agent holds nothing. A daemon that *checks* the variable cannot be
//! pointed anywhere: the worst an edited line can do is stop the service and say
//! so.
//!
//! # An absent variable is not a wrong one
//!
//! A missing `XDG_RUNTIME_DIR` is allowed and is not a stub. Two reasons, and
//! both are about what is really true on a machine:
//!
//! - **Nothing here uses the variables.** Where the bus is comes from the uid.
//!   So an absent one leaves the daemon exactly where it already was, which is
//!   `alo_secrets::NotStored::Unavailable` at the moment a store is opened —
//!   the state `a_session_that_really_ended.rs` measures — rather than a service
//!   that will not start.
//! - **The person's bus appears when their user manager gets to it**, not at the
//!   instant the manager is called started. A service that refused to run
//!   because a socket was three hundred milliseconds late would be a machine
//!   that boots to a sentence about D-Bus, and the unit is deliberately not
//!   restarted.
//!
//! So the rule is exactly: *say nothing, or say the person's session*.

use alo_entering::{TheSessionsEnvironment, WHERE_THE_BUS_IS, WHERE_THE_SESSION_IS};

use crate::caller::Uid;
use crate::refusing::NotStarted;

/// That the environment this process was started with is the person's session,
/// or names no session at all.
///
/// `person` is `[logins].person` from the machine description — the same number
/// `crate::side` divides the socket's two doors by, and the same one
/// `alo_accounts::Session` refuses to differ from.
///
/// # Errors
///
/// [`NotStarted::NotThePersonsSession`], naming the variable, what it said and
/// what it would have to say. Whoever reads it is standing a machine up and the
/// thing they have to change is a line in a unit file.
pub fn in_the_persons_session(person: Uid) -> Result<(), NotStarted> {
    told(
        person,
        std::env::var(WHERE_THE_SESSION_IS).ok().as_deref(),
        std::env::var(WHERE_THE_BUS_IS).ok().as_deref(),
    )
}

/// The same, as a rule about two strings.
///
/// Separated from the reading so that every branch of it is a test rather than
/// a process with a doctored environment: setting a variable is global to a
/// process and `std::env::set_var` is `unsafe` in this edition, which the
/// workspace forbids outright.
fn told(person: Uid, session: Option<&str>, bus: Option<&str>) -> Result<(), NotStarted> {
    let theirs = TheSessionsEnvironment::for_person(person.raw());

    if let Some(said) = session
        && !theirs.names_their_session(said)
    {
        return Err(NotStarted::NotThePersonsSession {
            variable: WHERE_THE_SESSION_IS,
            named: said.to_owned(),
            person: person.raw(),
            theirs: theirs.runtime_directory(),
        });
    }
    if let Some(said) = bus
        && !theirs.names_their_bus(said)
    {
        return Err(NotStarted::NotThePersonsSession {
            variable: WHERE_THE_BUS_IS,
            named: said.to_owned(),
            person: person.raw(),
            theirs: theirs.bus_address(),
        });
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The person this machine is described as belonging to.
    fn the_person() -> Uid {
        Uid::of(1000).unwrap()
    }

    /// **The session the unit really hands the daemon is accepted**, which is
    /// the one that has to work: the two strings `alo-entering` derives, for the
    /// person the description names.
    #[test]
    fn the_persons_own_session_starts() {
        let theirs = TheSessionsEnvironment::for_person(1000);
        assert!(
            told(
                the_person(),
                Some(&theirs.runtime_directory()),
                Some(&theirs.bus_address()),
            )
            .is_ok()
        );
    }

    /// **A machine that says nothing about a session starts**, and goes on to
    /// answer `Unavailable` when a store is opened rather than refusing to run.
    /// An absent variable is the state before anybody has signed in, and it is
    /// not a machine wired wrongly.
    #[test]
    fn a_process_told_nothing_about_a_session_starts() {
        assert!(told(the_person(), None, None).is_ok());
    }

    /// **A runtime directory belonging to another login stops the process**,
    /// which is the edited unit file this check exists for: one line, and a
    /// daemon that runs as the person while pointed at root's session.
    #[test]
    fn somebody_elses_runtime_directory_does_not_start() {
        let refused = told(the_person(), Some("/run/user/0"), None).unwrap_err();

        assert!(
            matches!(
                &refused,
                NotStarted::NotThePersonsSession { variable, named, person: 1000, theirs }
                    if *variable == WHERE_THE_SESSION_IS
                        && named == "/run/user/0"
                        && theirs == "/run/user/1000"
            ),
            "{refused}"
        );
        let said = refused.to_string();
        assert!(said.contains("XDG_RUNTIME_DIR"), "{said}");
        assert!(said.contains("/run/user/0"), "{said}");
        assert!(said.contains("/run/user/1000"), "{said}");
    }

    /// **And a bus belonging to another login stops it too**, separately: the
    /// two variables can be edited one at a time and a machine with one of them
    /// wrong is wired wrongly.
    #[test]
    fn somebody_elses_bus_does_not_start() {
        let theirs = TheSessionsEnvironment::for_person(1000);
        let refused = told(
            the_person(),
            Some(&theirs.runtime_directory()),
            Some("unix:path=/run/user/0/bus"),
        )
        .unwrap_err();

        assert!(
            matches!(
                &refused,
                NotStarted::NotThePersonsSession { variable, theirs, .. }
                    if *variable == WHERE_THE_BUS_IS
                        && theirs == "unix:path=/run/user/1000/bus"
            ),
            "{refused}"
        );
    }

    /// **A bus said as a bare path is the same bus**, and a bare path belonging
    /// to somebody else is refused for the same reason as the address.
    #[test]
    fn the_bus_is_theirs_under_either_spelling() {
        assert!(told(the_person(), None, Some("/run/user/1000/bus")).is_ok());
        assert!(told(the_person(), None, Some("/run/user/0/bus")).is_err());
    }

    /// **A second address hidden behind the first is refused**, which is what a
    /// D-Bus address's `;` means: *and if that fails, try this one*. A rule that
    /// looked for the person's bus anywhere in the text would have accepted a
    /// machine whose daemon falls back to somebody else's session.
    #[test]
    fn an_address_offering_a_second_bus_does_not_start() {
        let refused = told(
            the_person(),
            None,
            Some("unix:path=/run/user/1000/bus;unix:path=/run/user/0/bus"),
        );
        assert!(refused.is_err());
    }

    /// **What this process was really started with is what is asked about.**
    /// The reading half is one line and this is the test of it: whatever
    /// environment the test runner has, the answer is the same as the rule's for
    /// those two strings.
    #[test]
    fn the_environment_this_process_has_is_the_one_asked_about() {
        let person = the_person();
        assert_eq!(
            in_the_persons_session(person).is_ok(),
            told(
                person,
                std::env::var(WHERE_THE_SESSION_IS).ok().as_deref(),
                std::env::var(WHERE_THE_BUS_IS).ok().as_deref(),
            )
            .is_ok()
        );
    }
}
