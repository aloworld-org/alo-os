//! What this machine starts once a person's session is open, and as whom.
//!
//! # Why this component starts it at all
//!
//! Until 2026-10-14 it did not: `alo-desktop.service` was
//! `WantedBy=user@1000.service`, so `logind` started the person's user manager
//! on their first session and the user manager pulled the desktop in. A boot
//! showed what that costs. Two sessions existed for the person nine seconds
//! before the desktop started, and the desktop found none for itself, six
//! times, while PAM reported `session_open res=success` every time:
//!
//! ```text
//! [ 1723.840704] systemd-logind: New session c1 of user alo
//! [ 1735.311535] systemd[1]: Started alo-desktop.service
//! [ 1739.766759] alo-desktop: ... libseat found no login session for this process
//! ```
//!
//! `logind` answers *which session is this process in* from the control group,
//! and `user@1000.service` is the user manager, which is in no session. So the
//! desktop was never in the session that was opened for it. [ADR
//! 0115](../../../docs/decisions/0115-the-desktop-runs-inside-the-session-alo-sessiond-opened.md)
//! has the whole measurement.
//!
//! [`crate::machine`] passes its own pid as `CreateSession`'s leader, so
//! `logind` puts *this* process in the session's scope. **A child started after
//! that call inherits the scope and needs nothing else to be inside it.** The
//! order is the entire mechanism: open the session, then start the desktop.
//!
//! # This is not law 2's exec, and the shape is what says so
//!
//! Law 2 forbids a verb that runs an arbitrary command. `THE_DESKTOP` is a
//! constant: no message, no configuration file and no account names the program
//! that runs here, and there is no argument for one. What varies is **who it
//! runs as** and **which directories are theirs**, both of which come from the
//! account and the session rather than from anything a caller supplies.
//!
//! A reviewer checking that should look for a path reaching this module from
//! outside and find none.

use std::path::{Path, PathBuf};

/// The program this machine starts for a person who has signed in.
///
/// A constant, for the reason the module header gives: what starts here is
/// never chosen by a caller.
pub const THE_DESKTOP: &str = "/usr/bin/alo-desktop";

/// The directory name the desktop binds its socket inside.
///
/// It lives in the **session's** runtime directory rather than in
/// `/run/alo-desktop`, which is what the unit file made: a session's runtime
/// directory is the person's, `logind` takes it away when their last session
/// ends, and a desktop that outlived its session could not reach it.
pub const WHERE_THE_SOCKET_GOES: &str = "alo-desktop";

/// Everything needed to start one person's desktop, and nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheDesktopToStart {
    /// The program to start, which is always `THE_DESKTOP`.
    program: PathBuf,
    /// The account number it runs as.
    person: u32,
    /// Their folder, which becomes `HOME`.
    home: PathBuf,
    /// The directory it binds its socket in.
    runtime: PathBuf,
}

impl TheDesktopToStart {
    /// The program, which is always `THE_DESKTOP`.
    #[must_use]
    pub fn program(&self) -> &Path {
        &self.program
    }

    /// The account number it runs as.
    #[must_use]
    pub const fn person(&self) -> u32 {
        self.person
    }

    /// The folder that is theirs, which becomes `HOME`.
    #[must_use]
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// The directory the desktop binds its socket in, which becomes
    /// `RUNTIME_DIRECTORY`.
    #[must_use]
    pub fn runtime(&self) -> &Path {
        &self.runtime
    }
}

/// Describe the desktop to start for the person this session was opened for.
///
/// `home` is their folder, which `crate::the_folder_of` reads from the
/// machine's own user database rather than from the accounts file — see that
/// module for why this does not come from `TheAccounts`. `runtime` is the path
/// `CreateSession` answered with: the person's own runtime directory, which
/// `logind` makes and takes away with their last session.
#[must_use]
pub fn the_desktop_for(person: u32, home: &Path, runtime: &Path) -> TheDesktopToStart {
    TheDesktopToStart {
        program: PathBuf::from(THE_DESKTOP),
        person,
        home: home.to_path_buf(),
        runtime: runtime.join(WHERE_THE_SOCKET_GOES),
    }
}

/// What can start a desktop on this machine.
///
/// A trait for the reason [`crate::Logind`] is one: everything decided about
/// *what* to start is arithmetic the gate can run on any host, and only the
/// starting needs a Linux machine with a session on it.
pub trait StartsADesktop {
    /// Start it, as that person, inside this process's session scope.
    ///
    /// # Errors
    /// [`NotStarted`] — the program is not there, or the machine refused to
    /// run it as that account.
    fn start(&mut self, what: &TheDesktopToStart) -> Result<(), NotStarted>;
}

/// Every way starting a person's desktop can fail.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NotStarted {
    /// The machine would not run it, and this is what it said.
    #[error("this machine would not start {program} for account {person}: {why}")]
    TheMachineRefused {
        /// The program that was to be started.
        program: String,
        /// The account it was to run as.
        person: u32,
        /// What the machine said, kept whole.
        why: String,
    },
}

#[cfg(test)]
#[path = "the_persons_desktop_tests.rs"]
mod tests;
