//! **logind and polkit, asked on the system bus.** The few calls the runner
//! and the desktop's password sheet make, named once.
//!
//! Nothing here decides anything — `crate::running` decides, and
//! `crate::what_polkit_said` reads polkit's answer. These are the wires, and
//! they are walked only on a booted machine: no test here has a polkit to
//! ask (ADR 0150, *How this is checked*).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use zbus::blocking::Connection;
use zbus::zvariant::{OwnedObjectPath, Value};

use crate::answering_as_administrator::NotAsAdministrator;
use crate::what_polkit_said::{THE_ACTION, THE_COMMAND_DETAIL, what_it_means};

/// polkit's name on the bus.
pub const POLKIT: &str = "org.freedesktop.PolicyKit1";

/// polkit's authority, where every question is asked.
pub const THE_AUTHORITY_PATH: &str = "/org/freedesktop/PolicyKit1/Authority";

/// The authority's interface.
pub const THE_AUTHORITY: &str = "org.freedesktop.PolicyKit1.Authority";

/// logind's name on the bus, and its manager.
const LOGIND: &str = "org.freedesktop.login1";

/// polkit's flag for *the person may be asked*.
const ALLOW_USER_INTERACTION: u32 = 1;

/// The system bus, with every call bounded by `within`.
///
/// # Errors
/// Why the bus could not be reached, in English, for the journal.
pub fn the_system_bus(within: Duration) -> Result<Connection, String> {
    zbus::blocking::connection::Builder::system()
        .map_err(|why| why.to_string())?
        .method_timeout(within)
        .build()
        .map_err(|why| why.to_string())
}

/// **The session `person` is signed in at on a screen**, as logind names it:
/// the user's `Display` session. [`None`] when they have none — signed out, or
/// signed in only somewhere without a screen.
#[must_use]
pub fn the_session_on_their_screen(bus: &Connection, person: u32) -> Option<String> {
    let user: OwnedObjectPath = bus
        .call_method(
            Some(LOGIND),
            "/org/freedesktop/login1",
            Some("org.freedesktop.login1.Manager"),
            "GetUser",
            &(person,),
        )
        .ok()?
        .body()
        .deserialize()
        .ok()?;
    let reply = bus
        .call_method(
            Some(LOGIND),
            user.as_str(),
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.login1.User", "Display"),
        )
        .ok()?;
    let value: zbus::zvariant::OwnedValue = reply.body().deserialize().ok()?;
    // `(so)`: the session's id and its path, of which the id is wanted.
    let Value::Structure(display) = &*value else {
        return None;
    };
    match display.fields().first()? {
        Value::Str(id) if !id.is_empty() => Some(id.as_str().to_owned()),
        _ => None,
    }
}

/// The subject polkit is asked about: one logind session.
fn a_session(session: &str) -> (&'static str, HashMap<&'static str, Value<'_>>) {
    (
        "unix-session",
        HashMap::from([("session-id", Value::from(session))]),
    )
}

/// **Whether the person in `session` gave their password for `command`**:
/// polkit's `CheckAuthorization` for [`THE_ACTION`], the person allowed to be
/// asked, and the question taken back if `bus`'s bound passes first.
///
/// # Errors
/// What polkit's answer means; [`NotAsAdministrator::NoAnswerInTime`] when the
/// bound passed, and [`NotAsAdministrator::PolkitDidNotAnswer`] when polkit
/// could not be asked.
pub fn the_person_said_yes(
    bus: &Connection,
    session: &str,
    command: &str,
    within: Duration,
) -> Result<(), NotAsAdministrator> {
    let cancellation = format!("alo-{}-{}", std::process::id(), session);
    let details = HashMap::from([(THE_COMMAND_DETAIL, command)]);
    let began = Instant::now();
    let asked = bus.call_method(
        Some(POLKIT),
        THE_AUTHORITY_PATH,
        Some(THE_AUTHORITY),
        "CheckAuthorization",
        &(
            a_session(session),
            THE_ACTION,
            details,
            ALLOW_USER_INTERACTION,
            cancellation.as_str(),
        ),
    );
    match asked {
        Ok(reply) => {
            let (authorised, challenge, said): (bool, bool, HashMap<String, String>) = reply
                .body()
                .deserialize()
                .map_err(|_| NotAsAdministrator::PolkitDidNotAnswer)?;
            what_it_means(authorised, challenge, &said)
        }
        Err(_) if began.elapsed() + Duration::from_secs(1) >= within => {
            // **The question is taken back**, so the sheet goes away rather
            // than standing on the person's screen for a command that will
            // not run.
            let _ = bus.call_method(
                Some(POLKIT),
                THE_AUTHORITY_PATH,
                Some(THE_AUTHORITY),
                "CancelCheckAuthorization",
                &(cancellation.as_str(),),
            );
            Err(NotAsAdministrator::NoAnswerInTime)
        }
        Err(_) => Err(NotAsAdministrator::PolkitDidNotAnswer),
    }
}

/// **Register an authentication agent for `session`**, served at `path` on
/// `bus`, speaking `locale`: what makes polkit ask *this desktop* to put the
/// password sheet up.
///
/// # Errors
/// polkit's refusal, in English, for the journal.
pub fn the_agent_is_registered(
    bus: &Connection,
    session: &str,
    locale: &str,
    path: &str,
) -> Result<(), String> {
    bus.call_method(
        Some(POLKIT),
        THE_AUTHORITY_PATH,
        Some(THE_AUTHORITY),
        "RegisterAuthenticationAgent",
        &(a_session(session), locale, path),
    )
    .map(|_| ())
    .map_err(|why| why.to_string())
}

/// **The session this process is in**, as logind names it: `XDG_SESSION_ID`,
/// which `pam_systemd` sets for the desktop's unit, or logind's own answer
/// about this process when it is not set.
#[must_use]
pub fn the_session_this_process_is_in(bus: &Connection) -> Option<String> {
    if let Some(id) = std::env::var("XDG_SESSION_ID")
        .ok()
        .filter(|id| !id.is_empty())
    {
        return Some(id);
    }
    let session: OwnedObjectPath = bus
        .call_method(
            Some(LOGIND),
            "/org/freedesktop/login1",
            Some("org.freedesktop.login1.Manager"),
            "GetSessionByPID",
            &(std::process::id(),),
        )
        .ok()?
        .body()
        .deserialize()
        .ok()?;
    let reply = bus
        .call_method(
            Some(LOGIND),
            session.as_str(),
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.login1.Session", "Id"),
        )
        .ok()?;
    let value: zbus::zvariant::OwnedValue = reply.body().deserialize().ok()?;
    match &*value {
        Value::Str(id) if !id.is_empty() => Some(id.as_str().to_owned()),
        _ => None,
    }
}
