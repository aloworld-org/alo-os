//! **polkit's authentication agent for this person's session**: the half of
//! the password sheet that speaks to polkit
//! ([ADR 0150](../../../docs/decisions/0150-administrator-actions-ask-the-persons-password-each-time.md)).
//!
//! The owner, 2026-11-09: *"Yes, alo's desktop draws it"*. This registers
//! `org.freedesktop.PolicyKit1.AuthenticationAgent` on the system bus for the
//! session the desktop runs in, so when anything on the machine — alo's runner
//! for a `sudo` command, or anything else — needs an administrator's password,
//! polkit asks *this desktop*, and the sheet `alo_shell` draws comes up.
//!
//! # One question, from polkit's call to its answer
//!
//! 1. polkit calls `BeginAuthentication` with a cookie and the identities it
//!    would accept. This person must be one of them: the sheet never asks for
//!    anybody else's password.
//! 2. The question goes to the sheet ([`alo_shell::ToTheSheet::Ask`]) with a
//!    channel of its own, and the call waits — off the bus's executor, so a
//!    `CancelAuthentication` for it still arrives.
//! 3. What the person typed is checked by polkit's own helper, through
//!    `alo-checking-a-password` (`alo_administrator::the_helpers_conversation`).
//!    The helper tells polkit; this only hears *SUCCESS* or *FAILURE*.
//! 4. A wrong password leaves the sheet up; a password that cannot be checked
//!    here says why (`alo_administrator::in_step`); Escape answers polkit with
//!    *Cancelled*, which polkit reports to its caller as *dismissed*.
//!
//! **Nothing here keeps the password.** It arrives as an
//! [`alo_shell::ThePasswordTyped`], is lent to the helper's conversation, and
//! is overwritten when it goes.

use std::collections::HashMap;
use std::io::BufReader;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use alo_administrator::in_step::{self, InStep};
use alo_administrator::the_helpers_conversation::{Checked, THE_CHECKERS_SOCKET, checked};
use alo_administrator::what_polkit_said::THE_ACTION;
use alo_shell::{FromTheSheet, PasswordAsked, ToTheSheet, WhatItIsFor, WhyNotChecked};
use zbus::zvariant::OwnedValue;

/// Where the agent is served on the system bus.
const THE_AGENTS_PATH: &str = "/org/aloworld/alo/AuthenticationAgent";

/// The most one check of a password may take.
const THE_LONGEST_A_CHECK_TAKES: Duration = Duration::from_secs(30);

/// The errors polkit's agent interface answers with.
#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.freedesktop.PolicyKit1.Error")]
enum AgentError {
    /// The bus itself failed.
    #[zbus(error)]
    ZBus(zbus::Error),
    /// The person was not asked, or could not be: polkit reports *not
    /// authorised*.
    Failed(String),
    /// The person closed the sheet: polkit reports *dismissed*.
    Cancelled(String),
}

/// The agent served on the bus.
struct TheAgent {
    /// Where questions go to the sheet.
    to_the_sheet: Sender<ToTheSheet>,
    /// This desktop's person, by number: the only identity it asks for.
    person: u32,
}

#[zbus::interface(name = "org.freedesktop.PolicyKit1.AuthenticationAgent")]
impl TheAgent {
    /// polkit asks for a password.
    #[allow(
        clippy::too_many_arguments,
        reason = "polkit's interface names six arguments, and this is its signature"
    )]
    async fn begin_authentication(
        &self,
        action_id: String,
        message: String,
        _icon_name: String,
        _details: HashMap<String, String>,
        cookie: String,
        identities: Vec<(String, HashMap<String, OwnedValue>)>,
    ) -> Result<(), AgentError> {
        if !identities
            .iter()
            .any(|identity| is_this_person(identity, self.person))
        {
            eprintln!(
                "alo-desktop: polkit asked for a password that is not this person's, so nobody \
                 is asked"
            );
            return Err(AgentError::Failed(
                "this desktop asks only its own person's password".to_owned(),
            ));
        }
        let what_for = if action_id == THE_ACTION {
            WhatItIsFor::ACommandAloWasAsked(message)
        } else {
            WhatItIsFor::SomethingElse(message)
        };
        let (answer, answered) = std::sync::mpsc::channel();
        self.to_the_sheet
            .send(ToTheSheet::Ask(PasswordAsked {
                cookie: cookie.clone(),
                what_for,
                answer,
            }))
            .map_err(|_| AgentError::Failed("the desktop is not drawing".to_owned()))?;
        let to_the_sheet = self.to_the_sheet.clone();
        let person = self.person;
        blocking::unblock(move || {
            the_conversation(&cookie, &answered, &to_the_sheet, person, Places::on_this_machine())
        })
        .await
    }

    /// polkit takes a question back.
    fn cancel_authentication(&self, cookie: String) {
        let _ = self.to_the_sheet.send(ToTheSheet::Over { cookie });
    }
}

/// Whether one of polkit's identities is this person.
fn is_this_person(identity: &(String, HashMap<String, OwnedValue>), person: u32) -> bool {
    let (kind, details) = identity;
    kind == "unix-user"
        && details
            .get("uid")
            .and_then(|uid| u32::try_from(&**uid).ok())
            == Some(person)
}

/// Where a password is checked, and where whether it can be is written.
#[derive(Debug, Clone, Copy)]
struct Places<'a> {
    /// `alo-checking-a-password`'s socket.
    checker: &'a Path,
    /// The keeper's state folder.
    in_step: &'a Path,
}

impl Places<'static> {
    /// This machine's.
    fn on_this_machine() -> Self {
        Self {
            checker: Path::new(THE_CHECKERS_SOCKET),
            in_step: Path::new(in_step::THE_FOLDER),
        }
    }
}

/// **One question, from the sheet's first answer to polkit's.**
fn the_conversation(
    cookie: &str,
    answered: &Receiver<FromTheSheet>,
    to_the_sheet: &Sender<ToTheSheet>,
    person: u32,
    places: Places<'_>,
) -> Result<(), AgentError> {
    loop {
        let typed = match answered.recv() {
            Ok(FromTheSheet::Typed(typed)) => typed,
            Ok(FromTheSheet::Cancelled) => {
                return Err(AgentError::Cancelled("the person closed the sheet".to_owned()));
            }
            // The sheet let the question go: polkit took it back, or the
            // desktop is ending.
            Err(_) => return Err(AgentError::Cancelled("the question was taken back".to_owned())),
        };
        let told = match checked_once(places.checker, cookie, typed.as_str()) {
            Ok(Checked::Yes) => {
                let _ = to_the_sheet.send(ToTheSheet::Over {
                    cookie: cookie.to_owned(),
                });
                return Ok(());
            }
            Ok(Checked::No) => match in_step::now(places.in_step, person) {
                InStep::Kept => ToTheSheet::NotRight {
                    cookie: cookie.to_owned(),
                },
                InStep::NotKept(_) | InStep::NotKnown => ToTheSheet::CannotBeChecked {
                    cookie: cookie.to_owned(),
                    why: WhyNotChecked::NotInStep,
                },
            },
            Err(why) => {
                eprintln!("alo-desktop: a password could not be checked: {why}");
                ToTheSheet::CannotBeChecked {
                    cookie: cookie.to_owned(),
                    why: WhyNotChecked::NoChecker,
                }
            }
        };
        drop(typed);
        if to_the_sheet.send(told).is_err() {
            return Err(AgentError::Cancelled("the desktop is not drawing".to_owned()));
        }
    }
}

/// One password, checked by polkit's helper through the checker at `socket`.
fn checked_once(
    socket: &Path,
    cookie: &str,
    password: &str,
) -> Result<Checked, alo_administrator::the_helpers_conversation::NotChecked> {
    use alo_administrator::the_helpers_conversation::NotChecked;
    let no_checker = |why: std::io::Error| NotChecked::NoChecker(why.to_string());
    let stream = UnixStream::connect(socket).map_err(no_checker)?;
    stream
        .set_read_timeout(Some(THE_LONGEST_A_CHECK_TAKES))
        .and_then(|()| stream.set_write_timeout(Some(THE_LONGEST_A_CHECK_TAKES)))
        .map_err(no_checker)?;
    let mut writing = stream.try_clone().map_err(no_checker)?;
    let mut reading = BufReader::new(stream);
    checked(&mut reading, &mut writing, cookie, password)
}

/// **The agent, served and registered for this desktop's session**, handing
/// questions to `to_the_sheet`. The connection is returned to be held for the
/// session: dropping it takes the agent off the bus, and polkit then finds
/// nothing that can ask.
///
/// [`None`] — said in the journal — when the bus, the session or polkit would
/// not have it; administrator actions are then refused as *nothing can ask*,
/// never run another way.
pub(crate) fn the_agent_for_this_session(
    to_the_sheet: Sender<ToTheSheet>,
) -> Option<zbus::blocking::Connection> {
    let person = rustix::process::getuid().as_raw();
    let agent = TheAgent {
        to_the_sheet,
        person,
    };
    let bus = zbus::blocking::connection::Builder::system()
        .and_then(|builder| builder.serve_at(THE_AGENTS_PATH, agent))
        .and_then(zbus::blocking::connection::Builder::build);
    let bus = match bus {
        Ok(bus) => bus,
        Err(why) => {
            eprintln!(
                "alo-desktop: the system bus would not take the password sheet, so nothing in \
                 this session can ask for an administrator's password: {why}"
            );
            return None;
        }
    };
    let Some(session) = alo_administrator::on_the_system_bus::the_session_this_process_is_in(&bus)
    else {
        eprintln!(
            "alo-desktop: this desktop is in no session logind knows, so polkit cannot ask it for \
             a password"
        );
        return None;
    };
    let locale = std::env::var("LANG").unwrap_or_else(|_| "C".to_owned());
    match alo_administrator::on_the_system_bus::the_agent_is_registered(
        &bus,
        &session,
        &locale,
        THE_AGENTS_PATH,
    ) {
        Ok(()) => {
            eprintln!("alo-desktop: the password sheet is polkit's agent for session {session}");
            Some(bus)
        }
        Err(why) => {
            eprintln!(
                "alo-desktop: polkit would not take the password sheet as session {session}'s \
                 agent, so nothing in this session can ask for an administrator's password: {why}"
            );
            None
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "asking_for_the_persons_password_tests.rs"]
mod tests;
