//! **What the person's shell starts with**: the environment of their session,
//! what this terminal is called, and nothing of the process that runs it.
//!
//! # Why the session's, and why the whole of it
//!
//! alo's terminal runs inside the desktop (ADR 0114), so a shell started the
//! ordinary way inherits **the desktop's** environment — and the desktop is not
//! the person's session. It is a system unit with variables of its own
//! (`ALO_AGENT`, `ALO_KEYBOARD`, its runtime folder, systemd's
//! `NOTIFY_SOCKET` and `INVOCATION_ID`), and it lacks what the person's
//! session holds: where alo draws, the session bus, Flatpak's folders in
//! `XDG_DATA_DIRS`, and the *applications describe themselves* variables from
//! `environment.d`. A program typed into the terminal then behaved unlike the
//! same program pressed on the Dock — on 2026-11-14 VSCodium crashed for want
//! of a display (`docs/quirks/vscodium-from-alos-terminal-asks-for-x11-and-traps.md`).
//!
//! `docs/features.md` promises the person's shell with *their environment*.
//! Their environment is what their systemd manager holds, because that is what
//! every application they start from the Dock or the bus is given. So, where
//! the session could be asked ([`TheSession::Whole`]), the shell starts with
//! **exactly** that, and `TERM`.
//!
//! # How the desktop's own names are kept out
//!
//! The engine underneath can add variables to a child and cannot take any away
//! (`alacritty_terminal` 0.26, `tty::Options` has only `env`), and it sets four
//! of its own on every child. So the shell is started through
//! `/usr/bin/env -u NAME … -- <their shell>`: every name the session does not
//! hold is named to `env`, which removes it and then becomes the shell — the
//! same process, so nothing that asks about the shell sees `env`.
//!
//! **Only names go on that command line, never values**, because a command
//! line can be read by any account on the machine and an environment cannot.
//! The values travel the way they always did.
//!
//! # When the session cannot be asked
//!
//! [`TheSession::OnTopOfThisProcess`]: the shell inherits this process and is
//! given what it was handed on top, which is how it worked before. The caller
//! says so on its log, because a shell missing the person's session is a fault
//! somebody should be able to find.

use std::collections::{BTreeSet, HashMap};

/// What a shell's environment is built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheSession {
    /// The person's whole session environment, `NAME=value` each, as their
    /// systemd manager holds it. The shell starts with exactly this and `TERM`;
    /// a name given twice takes its last value.
    Whole(Vec<String>),
    /// The session could not be asked: these `NAME=value` on top of what this
    /// process has.
    OnTopOfThisProcess(Vec<String>),
}

/// The program that removes names and then becomes the shell.
pub(crate) const ENV: &str = "/usr/bin/env";

/// What the engine underneath sets on every child itself
/// (`alacritty_terminal` 0.26, `tty/unix.rs`), and which a shell starting with
/// the session's environment must not have unless the session has them.
const THE_ENGINES_OWN: [&str; 4] = ["ALACRITTY_WINDOW_ID", "WINDOWID", "USER", "HOME"];

/// What one shell is started with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WhatTheShellStartsWith {
    /// Variables handed to the shell.
    pub(crate) env: HashMap<String, String>,
    /// The program to run and its arguments — [`ENV`] in front of the
    /// person's shell — or [`None`] for the person's shell as the engine finds
    /// it.
    pub(crate) program: Option<(String, Vec<String>)>,
    /// Why the session was asked for and the desktop's own names could still
    /// not be kept out, for the caller's log.
    pub(crate) not_kept_out: Option<WhyNotKeptOut>,
}

/// Why a shell given the whole session still inherits some or all of this
/// process's own names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyNotKeptOut {
    /// Neither the session nor this process names the person's shell, so there
    /// is nothing to put after `/usr/bin/env`. Every name this process has is kept.
    NoShellNamed,
    /// `/usr/bin/env` is not on this machine. Every name this process has is kept.
    NoEnv,
    /// This many of this process's names are not text, so they cannot be
    /// named to `/usr/bin/env` and are kept. The rest are removed. systemd refuses a
    /// unit environment that is not UTF-8, so on an alo machine this is zero.
    NamesThatAreNotText(usize),
}

impl std::fmt::Display for WhyNotKeptOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoShellNamed => write!(
                f,
                "neither the session nor the desktop names the person's shell, so it inherits \
                 the desktop's own variables"
            ),
            Self::NoEnv => write!(
                f,
                "{ENV} is not on this machine, so the shell inherits the desktop's own variables"
            ),
            Self::NamesThatAreNotText(count) => write!(
                f,
                "{count} of the desktop's own variable names are not text and the shell inherits them"
            ),
        }
    }
}

/// **What a shell starts with**, from the session, the name this terminal goes
/// by (`term`), the names this process has (`this_process`), this process's
/// own `SHELL`, and whether [`ENV`] exists (`env_is_there`).
///
/// A name that is not `NAME=value` with a non-empty name is not a variable and
/// is left out. `TERM` is set last, so no session can change what this
/// terminal claims to understand.
pub(crate) fn what_a_shell_starts_with(
    session: &TheSession,
    term: &str,
    this_process: &[String],
    this_processs_shell: Option<&str>,
    env_is_there: bool,
) -> WhatTheShellStartsWith {
    let (entries, whole) = match session {
        TheSession::Whole(entries) => (entries, true),
        TheSession::OnTopOfThisProcess(entries) => (entries, false),
    };
    let mut env: HashMap<String, String> = entries
        .iter()
        .filter_map(|entry| entry.split_once('='))
        .filter(|(name, _)| !name.is_empty())
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect();
    env.insert("TERM".to_owned(), term.to_owned());
    if !whole {
        return WhatTheShellStartsWith {
            env,
            program: None,
            not_kept_out: None,
        };
    }
    let shell = env
        .get("SHELL")
        .map(String::as_str)
        .or(this_processs_shell)
        .filter(|shell| !shell.is_empty())
        .map(str::to_owned);
    let why = match (shell, env_is_there) {
        (Some(shell), true) => {
            let removed: BTreeSet<&str> = this_process
                .iter()
                .map(String::as_str)
                .chain(THE_ENGINES_OWN)
                .filter(|name| !name.is_empty() && !name.contains('='))
                .filter(|name| !env.contains_key(*name))
                .collect();
            let mut arguments = Vec::with_capacity(removed.len() * 2 + 2);
            for name in removed {
                arguments.push("-u".to_owned());
                arguments.push(name.to_owned());
            }
            arguments.push("--".to_owned());
            arguments.push(shell);
            return WhatTheShellStartsWith {
                env,
                program: Some((ENV.to_owned(), arguments)),
                not_kept_out: None,
            };
        }
        (None, _) => WhyNotKeptOut::NoShellNamed,
        (Some(_), false) => WhyNotKeptOut::NoEnv,
    };
    WhatTheShellStartsWith {
        env,
        program: None,
        not_kept_out: Some(why),
    }
}

#[cfg(test)]
#[path = "what_a_shell_starts_with_tests.rs"]
mod tests;
