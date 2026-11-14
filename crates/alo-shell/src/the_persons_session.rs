//! **The person's session environment**, as their systemd manager holds it,
//! for the shell in alo's own terminal.
//!
//! Every application a person starts from the Dock or the bus is started by
//! their systemd manager and given its environment: what `environment.d`
//! names (the *applications describe themselves* variables among them),
//! Flatpak's folders in `XDG_DATA_DIRS`, the session bus, and — once the
//! desktop has told it at sign-in — where alo draws. The terminal's shell is
//! started by the desktop instead, so it is handed the same thing, read from
//! the same place, rather than a list somebody wrote down here.
//!
//! **Read once, when the terminal opens**, over the session bus, as
//! `org.freedesktop.systemd1.Manager`'s `Environment`. Nothing here listens, and
//! nothing is read again while the terminal is open: a shell's environment is
//! fixed when it starts, in every terminal.
//!
//! Measured once by hand on 2026-11-14, on Ubuntu's systemd 255 and not on an
//! alo machine: that property held `SHELL`, `PATH`, `XDG_RUNTIME_DIR`,
//! `DBUS_SESSION_BUS_ADDRESS`, `XDG_DATA_DIRS` with both Flatpak export
//! folders, and the `environment.d` variables.

use std::path::Path;

use alo_terminal::TheSession;

/// The person's manager's environment, `NAME=value` each, or why it could not
/// be read.
///
/// # Errors
/// A sentence for the log when there is no session bus, no manager on it, or an
/// answer that is not a list of text.
pub(crate) fn what_their_manager_holds() -> Result<Vec<String>, String> {
    let bus = zbus::blocking::Connection::session().map_err(|why| why.to_string())?;
    let value: zbus::zvariant::OwnedValue = bus
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.systemd1.Manager", "Environment"),
        )
        .and_then(|reply| reply.body().deserialize())
        .map_err(|why| why.to_string())?;
    Vec::<String>::try_from(value).map_err(|why| why.to_string())
}

/// **What the terminal's shell is built from**: the manager's environment with
/// where alo draws (`display`) laid over it, or — where the manager could not
/// be asked — the display alone on top of the desktop's own environment. The
/// second answer carries the sentence the log owes the person.
///
/// The display is laid over the manager's answer, not trusted to be in it,
/// because the desktop's telling the manager at sign-in can fail on its own,
/// and a shell with no display is the fault this exists for.
pub(crate) fn the_session_for_a_terminal(
    manager: Result<Vec<String>, String>,
    display: &Path,
) -> (TheSession, Vec<String>) {
    let mut said = Vec::new();
    let told = alo_launching::what_the_session_is_told(display).unwrap_or_else(|| {
        said.push(format!(
            "alo's own terminal cannot be told the display at {}, so a graphical program \
             started in it will find no display",
            display.display()
        ));
        Vec::new()
    });
    let session = match manager {
        Ok(mut held) => {
            held.extend(told);
            TheSession::Whole(held)
        }
        Err(why) => {
            said.push(format!(
                "alo's own terminal could not read the person's session from their systemd \
                 manager ({why}), so its shell has the desktop's environment and where alo draws"
            ));
            TheSession::OnTopOfThisProcess(told)
        }
    };
    (session, said)
}

#[cfg(test)]
#[path = "the_persons_session_tests.rs"]
mod tests;
