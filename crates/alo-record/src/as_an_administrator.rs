//! **A command asked for as administrator, and what the person did when their
//! password was asked for**
//! ([ADR 0150](../../../docs/decisions/0150-administrator-actions-ask-the-persons-password-each-time.md)).
//!
//! A stamp on the entry for a command that ran and on the entry for one that
//! did not, so the record says both that the command was an administrator's
//! and whether the person authenticated — the two facts ADR 0150 says every
//! administrator action leaves behind. Never the password, and never anything
//! about it but whether it was given.

use serde::{Deserialize, Serialize};

/// What happened when a command was asked for as administrator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AsAnAdministrator {
    /// The person gave their password, and it ran as administrator.
    TheyGaveTheirPassword,
    /// The person closed the password window.
    TheyCancelled,
    /// The password given was not accepted.
    NotAccepted,
    /// Nothing on the machine could ask for the password.
    NothingCouldAsk,
    /// The account is not an administrator.
    NotAnAdministrator,
    /// Nobody answered within the time the question stood.
    NoAnswerInTime,
    /// The machine could not ask at all.
    CouldNotAsk,
}

impl AsAnAdministrator {
    /// Whether the person authenticated.
    #[must_use]
    pub const fn they_authenticated(self) -> bool {
        matches!(self, Self::TheyGaveTheirPassword)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::SystemTime;

    use alo_capability::Grantee;

    use super::*;
    use crate::Entry;

    /// **A command run as administrator says so, and that the person gave
    /// their password**; a refusal says what they did; anything else carries
    /// no such field at all.
    #[test]
    fn an_administrator_action_is_written_down_as_one() {
        let agent = Grantee::named("alo");
        let ran = Entry::ran_a_command(
            &agent,
            "/bin/sh",
            &["-c".to_owned(), "sudo dnf install -y htop".to_owned()],
            &[],
            Some(0),
            false,
            SystemTime::UNIX_EPOCH,
        )
        .as_an_administrator(AsAnAdministrator::TheyGaveTheirPassword);
        let written = serde_json::to_string(&ran).unwrap();
        assert!(
            written.contains(r#""as_an_administrator":"they-gave-their-password""#),
            "{written}"
        );
        assert_eq!(serde_json::from_str::<Entry>(&written).unwrap(), ran);
        assert!(
            ran.asked_as_an_administrator()
                .unwrap()
                .they_authenticated()
        );

        let refused =
            Entry::a_command_was_not_run(&agent, "/bin/sh", "nothing ran", SystemTime::UNIX_EPOCH)
                .as_an_administrator(AsAnAdministrator::TheyCancelled);
        assert!(
            !refused
                .asked_as_an_administrator()
                .unwrap()
                .they_authenticated()
        );

        let ordinary =
            Entry::a_command_was_not_run(&agent, "/bin/sh", "nothing ran", SystemTime::UNIX_EPOCH);
        assert!(
            !serde_json::to_string(&ordinary)
                .unwrap()
                .contains("as_an_administrator")
        );
        assert_eq!(ordinary.asked_as_an_administrator(), None);
    }
}
