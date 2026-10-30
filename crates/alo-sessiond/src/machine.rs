//! The half that is really a machine: the accounts file on the disk.
//!
//! One method wide, and it implements a trait whose other implementations are
//! fixtures — so everything this crate *decides* is decided in
//! `crate::opening`, on any host, in the gate.
//!
//! # `logind` is not spoken to from here any more
//!
//! **This file used to call `CreateSession`**, with this process's own pid as
//! the session's leader, on `seat0` and `tty1`. Measured inside the image on
//! 2026-10-30 ([ADR 0141](../../../docs/decisions/0141-the-desktop-is-put-inside-the-persons-session-by-what-already-has-the-privilege.md)):
//! `logind` moves the leader into the person's session scope, so this
//! privileged daemon became a member of the person's session and **held their
//! seat**, and the desktop's own session was never made. The person's terminal
//! was then in no session at all, and restart, power off and the time zone
//! asked for a password nothing on the screen could take.
//!
//! The desktop's own unit opens the session now, through `pam_systemd`, and
//! `crate::starting_their_units` starts that unit. ADR 0024's measurement —
//! that `logind` opens a session for a privileged caller and refuses an
//! unprivileged one — still stands as a fact about `logind`; nothing here
//! relies on it.

use std::path::Path;

use alo_accounts::THE_ACCOUNTS;

use crate::accounts::TheAccounts;
use crate::refusing::NotAsked;

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
