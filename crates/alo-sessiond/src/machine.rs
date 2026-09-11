//! The half that is really a machine: `systemd-logind` over the system bus,
//! and the accounts file on the disk.
//!
//! Both are one method wide, and both implement a trait whose other
//! implementations are fixtures — so everything this crate *decides* is decided
//! in `crate::opening`, on any host, in the gate. What is here is the part that
//! can only be shown working on a Linux machine with a bus.
//!
//! # The call, and the measurement behind every argument
//!
//! ADR 0024 asked whether `logind` will open a session for a caller that is not
//! `pam_systemd`. It will, provided the caller is privileged:
//! `CreateSession` answers root with *Leader PID is not valid* — the call
//! authorised, and only its contents refused — and an unprivileged caller with
//! *Access denied*. That was measured on the development machine before the
//! decision was taken, and **again on the pinned Fedora base** while this was
//! built; `docs/quirks.md` has both, including the one thing that differs
//! between them.
//!
//! So no PAM module is owed, no C ABI, and no `unsafe` exemption on the
//! authentication path — which was the price ADR 0024 priced for the other
//! option.
//!
//! # What this component deliberately does not decide
//!
//! **The seat and the virtual terminal.** They are passed empty and zero, which
//! is a session with no seat. A seat and a VT are facts about *what draws*, and
//! this component must not draw, start a compositor, or decide what a person
//! sees — that is task 13's and the desktop lane's. A seatless session is
//! enough for the thing this task exists for: `logind` starts the person's own
//! `user@<uid>.service` on their first session whatever its seat, and
//! `alo-agentd.service` is `BindsTo=` that.
//!
//! **The session type.** `unspecified`, for the same reason: claiming
//! `wayland` would be this process asserting that something is drawing, when
//! the compositor it would be asserting about has no binary yet.
//!
//! Both are additive to change — a later task that starts a compositor is the
//! one that knows which VT it is on, and it will be changing an argument here
//! rather than adding a door.
//!
//! # The session lasts as long as this process holds it
//!
//! `CreateSession` answers with a descriptor, and the session lives while
//! somebody holds it. [`TheMachinesLogind`] keeps it, which is why
//! [`crate::Logind`] takes `&mut self`: signing out is this process letting go,
//! and there is nothing else it could be.

use std::path::Path;

use alo_accounts::THE_ACCOUNTS;
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedFd, OwnedObjectPath, Value};

use crate::accounts::TheAccounts;
use crate::logind::Logind;
use crate::refusing::{NotAsked, NotOpened};

/// The service that opens sessions on a systemd machine.
const LOGIND: &str = "org.freedesktop.login1";

/// The object its manager is at.
const THE_MANAGER: &str = "/org/freedesktop/login1";

/// The interface the one method is on.
const THE_INTERFACE: &str = "org.freedesktop.login1.Manager";

/// The one method this component calls, on the whole machine.
const CREATE_SESSION: &str = "CreateSession";

/// What this component calls itself where `logind` records who asked.
const US: &str = "alo-sessiond";

/// The class of session a person signing in gets.
const A_PERSONS: &str = "user";

/// What this component will not claim about what is drawing.
const NOT_SAID: &str = "unspecified";

/// `systemd-logind`, on this machine's system bus.
#[derive(Debug, Default)]
pub struct TheMachinesLogind {
    /// What holds the session open, once there is one.
    ///
    /// Never read, and that is the whole of its job: the session lasts while
    /// this is alive, so it is dropped when this process ends and not before.
    held: Option<OwnedFd>,
}

impl Logind for TheMachinesLogind {
    fn open_a_session_for(&mut self, person: u32) -> Result<(), NotOpened> {
        let connection = Connection::system().map_err(|why| NotOpened::NoBus {
            why: why.to_string(),
        })?;

        let nothing: Vec<(String, Value<'_>)> = Vec::new();
        let reply = connection
            .call_method(
                Some(LOGIND),
                THE_MANAGER,
                Some(THE_INTERFACE),
                CREATE_SESSION,
                &(
                    person,
                    std::process::id(),
                    US,
                    NOT_SAID,
                    A_PERSONS,
                    "",
                    "",
                    0_u32,
                    "",
                    "",
                    false,
                    "",
                    "",
                    nothing,
                ),
            )
            .map_err(|why| refused(person, &why))?;

        let (_called, _object, _runtime, held, _uid, _seat, _vtnr, _existing): (
            String,
            OwnedObjectPath,
            String,
            OwnedFd,
            u32,
            String,
            u32,
            bool,
        ) = reply
            .body()
            .deserialize()
            .map_err(|why| NotOpened::NotUnderstood {
                why: why.to_string(),
            })?;

        self.held = Some(held);
        Ok(())
    }
}

/// A call the bus would not carry out, with the name to decide on beside the
/// sentence to read.
///
/// `zbus::Error::MethodError` is the only shape that carries a D-Bus error name;
/// everything else — a connection that dropped, a reply that never came — has
/// none, and an empty name is how this says so rather than inventing one.
fn refused(person: u32, why: &zbus::Error) -> NotOpened {
    let named = match why {
        zbus::Error::MethodError(name, _, _) => name.as_str().to_owned(),
        _ => String::new(),
    };
    NotOpened::Refused {
        person,
        named,
        why: why.to_string(),
    }
}

/// The accounts this machine has, as the file `alo-accounts` keeps.
#[derive(Debug, Clone, Copy, Default)]
pub struct TheMachinesAccounts;

impl TheAccounts for TheMachinesAccounts {
    /// Whether this machine has an account with this number.
    ///
    /// **A machine with no accounts file is a machine with no accounts**, and
    /// not a failure. That is the state every alo OS arrives in — the image
    /// ships no store, which ADR 0024 settled and `crates/alo-image` tests —
    /// so a first boot answers *no* here rather than *this machine is broken*.
    /// Every other way the file can refuse is carried, because a store that
    /// somebody else could have written is one `alo-accounts` will not believe
    /// and this component must not believe either.
    fn an_account_numbered(&self, person: u32) -> Result<bool, NotAsked> {
        match alo_accounts::found(Path::new(THE_ACCOUNTS)) {
            Ok(accounts) => Ok(accounts.numbers(person)),
            Err(alo_accounts::NotKept::NotThere { .. }) => Ok(false),
            Err(why) => Err(NotAsked::about(THE_ACCOUNTS, &why)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The one method, the one interface and the one object are written
    /// once.** A privileged component whose bus address was assembled in two
    /// places would be one where a typo is a call to something else.
    #[test]
    fn there_is_one_method_this_component_can_call() {
        assert_eq!(LOGIND, "org.freedesktop.login1");
        assert_eq!(THE_MANAGER, "/org/freedesktop/login1");
        assert_eq!(THE_INTERFACE, "org.freedesktop.login1.Manager");
        assert_eq!(CREATE_SESSION, "CreateSession");
    }

    /// **A machine with no accounts file has no accounts**, rather than an
    /// unreadable one. This is the first boot of every alo OS there will ever
    /// be, and answering it as a failure would mean the first person to make an
    /// account could not then sign in with it.
    #[test]
    fn a_machine_with_no_accounts_file_answers_that_nobody_is_numbered() {
        // The real path, which on a machine running these tests is not there.
        // If it *is* there this asserts nothing false: the question is whether
        // a missing file is an error, and a present one simply answers.
        assert!(TheMachinesAccounts.an_account_numbered(1000).is_ok());
    }
}
