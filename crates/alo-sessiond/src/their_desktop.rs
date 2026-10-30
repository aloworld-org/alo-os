//! The one thing a sign-in is for: the person's desktop, started inside a
//! login session of its own.
//!
//! [ADR 0141](../../../docs/decisions/0141-the-desktop-is-put-inside-the-persons-session-by-what-already-has-the-privilege.md)
//! is the decision. **This component used to open the session itself**, by
//! calling `CreateSession` with its own pid as the leader, and a measurement
//! inside the image showed what that did: `logind` moved `alo-sessiond` into
//! the person's session scope and gave it the seat, so the desktop's own
//! session was never made and the person's terminal was in no session at all.
//! Restart, power off and the time zone then asked for a password nothing on
//! the screen could take.
//!
//! So it asks systemd to start `alo-desktop@<number>.service` instead. That
//! unit's PAM stack opens the session with `pam_systemd`, pid 1 becomes the
//! person, and the desktop's own process is the one `logind` puts in the scope.
//! **No capability is gained by anything**: starting a unit is authorised by
//! being root, the shape ADR 0121 and ADR 0135 already use.
//!
//! This is the trait for the reason [`crate::TheirFolder`] is: which knock
//! starts a desktop, and when a person counts as signed in, is arithmetic the
//! workspace gate runs on any host. `crate::starting_their_units` is the half
//! that is really systemd.

use crate::refusing::NotOpened;

/// What can start a person's desktop, and say whether it is still running.
pub trait TheirDesktop {
    /// Start account `person`'s desktop, and answer once it is running.
    ///
    /// `&mut self` because a fixture remembers what it was asked; the machine's
    /// own implementation holds nothing, since **the desktop's unit is what
    /// holds the session**, for exactly as long as it runs (ADR 0087's one
    /// owner).
    ///
    /// # Errors
    ///
    /// [`NotOpened`], which is every way the machine can answer no: no bus, a
    /// unit systemd would not start, or a desktop that failed before it ran.
    fn started_for(&mut self, person: u32) -> Result<(), NotOpened>;

    /// Whether account `person`'s desktop is still running.
    ///
    /// **Asked rather than remembered**, because signing out is the desktop
    /// ending, and this component is not told when that happens. A flag set at
    /// sign-in and never cleared would answer *already signed in* to the next
    /// correct password for ever.
    ///
    /// # Errors
    ///
    /// [`NotOpened`] when systemd cannot be asked.
    fn is_running_for(&self, person: u32) -> Result<bool, NotOpened>;
}

/// What one reading of a desktop unit's state says about a sign-in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Starting {
    /// It is running: the person's session is open and their desktop is in it.
    Running,
    /// It failed, and systemd's word for how is carried.
    Failed(String),
    /// Not settled yet: the job may not have begun, or PAM is still opening
    /// the session.
    StillGoing,
}

impl Starting {
    /// Read systemd's `ActiveState`, and the unit's `Result` beside it.
    ///
    /// **`inactive` is still going, not an end**, because `StartUnit` answers
    /// when the job is queued and the unit reads `inactive` until the job
    /// begins. Only `failed` is an end that is not a desktop.
    #[must_use]
    pub fn from_systemd(active_state: &str, result: &str) -> Self {
        match active_state {
            "active" => Self::Running,
            "failed" => Self::Failed(result.to_owned()),
            _ => Self::StillGoing,
        }
    }

    /// What a state still unsettled at the longest a person waits means.
    ///
    /// **`activating` is a desktop on its way**, and answering *signing in did
    /// not finish* over a desktop that appears a second later is the fault ADR
    /// 0115's amendment records. The unit owns what happens next: its
    /// `OnFailure=` puts the sign-in screen back if it never runs. Anything else
    /// — a job that never began — is a sign-in that did not happen.
    #[must_use]
    pub fn at_the_longest_a_person_waits(active_state: &str) -> bool {
        active_state == "activating"
    }
}

/// Whether systemd's `ActiveState` is a desktop that has not ended.
///
/// `deactivating` counts, because a desktop on its way out still holds the
/// seat, and starting a second one beside it would be refused by `logind`.
#[must_use]
pub fn has_not_ended(active_state: &str) -> bool {
    matches!(
        active_state,
        "active" | "activating" | "reloading" | "deactivating"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Only `active` is a desktop, and only `failed` is the end of one.**
    #[test]
    fn a_state_is_read_as_running_failed_or_still_going() {
        assert_eq!(Starting::from_systemd("active", "success"), Starting::Running);
        assert_eq!(
            Starting::from_systemd("failed", "exit-code"),
            Starting::Failed("exit-code".to_owned())
        );
        for going in ["inactive", "activating", "deactivating", "reloading"] {
            assert_eq!(
                Starting::from_systemd(going, "success"),
                Starting::StillGoing,
                "{going} was read as settled"
            );
        }
    }

    /// **A desktop still activating at the ceiling is let through, and a job
    /// that never began is not.**
    #[test]
    fn only_a_desktop_on_its_way_is_answered_as_a_sign_in_at_the_ceiling() {
        assert!(Starting::at_the_longest_a_person_waits("activating"));
        for not in ["inactive", "failed", "deactivating", ""] {
            assert!(
                !Starting::at_the_longest_a_person_waits(not),
                "{not:?} was answered as a desktop on its way"
            );
        }
    }

    /// **A desktop that ended lets the next person in, and one that has not
    /// does not.**
    #[test]
    fn a_desktop_has_ended_only_when_systemd_says_inactive_or_failed() {
        for running in ["active", "activating", "reloading", "deactivating"] {
            assert!(has_not_ended(running), "{running} was read as ended");
        }
        for ended in ["inactive", "failed"] {
            assert!(!has_not_ended(ended), "{ended} was read as still running");
        }
    }
}
