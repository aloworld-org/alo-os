//! **polkit's answer, read as what the person did.**
//!
//! `CheckAuthorization` answers three things — authorised, a challenge, and a
//! map of details — and with interaction allowed they mean, read from polkit
//! 126's `polkitbackendinteractiveauthority.c` on 2026-11-09:
//!
//! | authorised | challenge | details | what happened |
//! |---|---|---|---|
//! | yes | — | — | the person gave their password |
//! | no | yes | — | **no agent could ask**: polkit found no password sheet for the session, and returns the challenge it could not put |
//! | no | no | `polkit.dismissed = true` | the person closed the sheet (the agent answered *Cancelled*) |
//! | no | no | otherwise | it was not accepted |
//!
//! The second row is the one a careless reader gets wrong: *no agent* is not
//! an error from polkit, it is a challenge nobody was there to answer, and it
//! must reach the person as *nothing could ask you* rather than as a wrong
//! password.

use std::collections::HashMap;

use crate::answering_as_administrator::NotAsAdministrator;

/// The action alo asks polkit about: one command, run as administrator.
/// Declared in `image/usr/share/polkit-1/actions/org.aloworld.alo.policy`,
/// `auth_admin` at every setting and never `auth_admin_keep`, so every command
/// asks again (the owner's *"Ask my password each time"*).
pub const THE_ACTION: &str = "org.aloworld.alo.run-as-administrator";

/// The detail the command is handed to polkit under, which the action's
/// message shows the person as `$(command)`.
pub const THE_COMMAND_DETAIL: &str = "command";

/// The detail polkit sets when the agent said the person cancelled.
const DISMISSED: &str = "polkit.dismissed";

/// What polkit's three answers mean.
///
/// # Errors
/// [`NotAsAdministrator::NothingCanAsk`],
/// [`NotAsAdministrator::TheyCancelled`] or
/// [`NotAsAdministrator::NotAccepted`], as the table says.
pub fn what_it_means(
    authorised: bool,
    challenge: bool,
    details: &HashMap<String, String>,
) -> Result<(), NotAsAdministrator> {
    if authorised {
        return Ok(());
    }
    if challenge {
        return Err(NotAsAdministrator::NothingCanAsk);
    }
    if details.get(DISMISSED).is_some_and(|said| said == "true") {
        return Err(NotAsAdministrator::TheyCancelled);
    }
    Err(NotAsAdministrator::NotAccepted)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Each of polkit's answers is the person's**, as the table says.
    #[test]
    fn each_answer_is_what_the_person_did() {
        let none = HashMap::new();
        let dismissed = HashMap::from([(DISMISSED.to_owned(), "true".to_owned())]);
        assert_eq!(what_it_means(true, false, &none), Ok(()));
        assert_eq!(what_it_means(true, false, &dismissed), Ok(()));
        assert_eq!(
            what_it_means(false, true, &none),
            Err(NotAsAdministrator::NothingCanAsk)
        );
        assert_eq!(
            what_it_means(false, false, &dismissed),
            Err(NotAsAdministrator::TheyCancelled)
        );
        assert_eq!(
            what_it_means(false, false, &none),
            Err(NotAsAdministrator::NotAccepted)
        );
    }
}
