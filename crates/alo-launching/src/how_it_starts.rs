//! Which road starts an application: over the bus, or from its `Exec` line.
//!
//! **The entry says, and nothing else does.** `DBusActivatable=true` is the
//! entry's author saying *start me by asking me over the bus*, and the Desktop
//! Entry specification then still requires an `Exec` line *for compatibility*.
//! So a bus-activatable entry is asked over the bus first, and its `Exec` line
//! is carried as what to run when the bus has nothing to start — which on a
//! real machine is a flatpak whose service file the session bus was never told
//! about. The desktop says so on its log when that happens, so the second road
//! is never taken silently.
//!
//! Every other entry is started from its `Exec` line. An entry with neither is
//! refused, by name, and nothing runs.

use crate::the_command::{Filling, WhyNotACommand, the_command_in};
use crate::the_entry::TheEntry;

/// How an application is started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HowItStarts {
    /// `Activate` on its own bus name — and, if the bus cannot start it, this
    /// command, where its entry has a valid one.
    OverTheBus {
        /// The entry's own `Exec` line, as a command, if it has a valid one.
        or_else: Option<Vec<String>>,
    },
    /// Its `Exec` line, as a program and its arguments.
    FromItsCommand(Vec<String>),
}

/// Why an application cannot be started at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhyItCannotStart {
    /// The entry says `Hidden=true`: the specification's *deleted*.
    Deleted,
    /// The entry's `Type` is not `Application`.
    NotAnApplication,
    /// The entry asks to be run inside a terminal, and a press opens none.
    NeedsATerminal,
    /// The entry has no `Exec` and is not started over the bus.
    NothingToStartItWith,
    /// The entry's `Exec` line is not one the specification allows.
    ItsCommandIsMalformed(WhyNotACommand),
}

/// Decide how this entry starts.
///
/// `identifier` is the entry's desktop file ID and `location` where its file
/// is, for `%k`. **The bus road needs an identifier that is a bus name**; one
/// that is not — on an ordinary machine 14 of 15 desktop file IDs, as
/// `docs/contracts/application-door.md` records — cannot be activated whatever
/// its entry says, so it is started from its command.
///
/// # Errors
///
/// [`WhyItCannotStart`] when nothing should run.
pub fn how_it_starts(
    identifier: &str,
    entry: &TheEntry,
    location: Option<&str>,
) -> Result<HowItStarts, WhyItCannotStart> {
    if entry.hidden {
        return Err(WhyItCannotStart::Deleted);
    }
    if entry
        .kind
        .as_deref()
        .is_some_and(|kind| kind != "Application")
    {
        return Err(WhyItCannotStart::NotAnApplication);
    }
    if entry.terminal {
        return Err(WhyItCannotStart::NeedsATerminal);
    }
    let filling = Filling {
        icon: entry.icon.as_deref(),
        name: entry.name.as_deref(),
        location,
    };
    let command = entry
        .exec
        .as_deref()
        .map(|exec| the_command_in(exec, &filling));
    if entry.bus_activatable && is_a_bus_name(identifier) {
        return Ok(HowItStarts::OverTheBus {
            or_else: command.and_then(Result::ok),
        });
    }
    match command {
        Some(Ok(command)) => Ok(HowItStarts::FromItsCommand(command)),
        Some(Err(why)) => Err(WhyItCannotStart::ItsCommandIsMalformed(why)),
        None => Err(WhyItCannotStart::NothingToStartItWith),
    }
}

/// Whether an identifier is a well-known bus name, by the D-Bus specification:
/// at least two elements separated by `.`, each made of `[A-Za-z0-9_-]` and not
/// starting with a digit, at most 255 bytes.
#[must_use]
pub fn is_a_bus_name(identifier: &str) -> bool {
    let mut elements = 0;
    for element in identifier.split('.') {
        let mut letters = element.chars();
        let Some(first) = letters.next() else {
            return false;
        };
        if first.is_ascii_digit() || !is_a_name_letter(first) || !letters.all(is_a_name_letter) {
            return false;
        }
        elements += 1;
    }
    elements >= 2 && identifier.len() <= 255
}

/// One character a bus name element may hold.
const fn is_a_name_letter(letter: char) -> bool {
    letter.is_ascii_alphanumeric() || letter == '_' || letter == '-'
}

#[cfg(test)]
#[path = "how_it_starts_tests.rs"]
mod tests;
