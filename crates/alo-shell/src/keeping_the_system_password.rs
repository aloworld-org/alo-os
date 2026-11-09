//! **The sign-in screen's keeper**: the password a person just signed in with,
//! handed to `alo-keeping-the-system-password` so polkit and `sudo` ask the
//! same one ([ADR 0150](../../../docs/decisions/0150-administrator-actions-ask-the-persons-password-each-time.md)).
//!
//! `alo_greeting::Greeting` calls it once per successful sign-in, before the
//! session opens. The screen runs as root with no capability, which is enough
//! to open a socket only root may open and nothing more; the keeper on the
//! other end holds the one capability that writes the system's list.
//!
//! **What happened is said in the journal, and never the password.** A keeper
//! that could not be reached does not stop the sign-in: the person is owed
//! their desktop, and the keeper's own state file is what lets a password sheet
//! later say truthfully why their password cannot be checked.

use std::path::{Path, PathBuf};

use alo_administrator::keeping::{THE_KEEPERS_SOCKET, handed_to_the_keeper};
use alo_greeting::KeepingTheSystemPassword;

/// The keeper at a socket: on a machine, [`THE_KEEPERS_SOCKET`].
#[derive(Debug, Clone)]
pub(crate) struct TheSystemsKeeper {
    /// Where the keeper listens.
    at: PathBuf,
}

impl TheSystemsKeeper {
    /// The keeper this machine runs.
    pub(crate) fn on_this_machine() -> Self {
        Self::at(Path::new(THE_KEEPERS_SOCKET))
    }

    /// The keeper listening at `socket`.
    pub(crate) fn at(socket: &Path) -> Self {
        Self {
            at: socket.to_owned(),
        }
    }
}

impl KeepingTheSystemPassword for TheSystemsKeeper {
    fn keep(&self, person: u32, typed: &str) {
        match handed_to_the_keeper(&self.at, person, typed) {
            Ok(()) => eprintln!(
                "alo-shell: the system's password for account {person} is the one they signed in \
                 with"
            ),
            Err(why) => eprintln!(
                "alo-shell: the system's password for account {person} was not kept in step, so \
                 an administrator's password asked for in this session cannot be checked: {why}"
            ),
        }
    }
}
