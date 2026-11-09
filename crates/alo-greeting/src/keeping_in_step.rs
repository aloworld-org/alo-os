//! **The system's password, kept in step at every sign-in**
//! ([ADR 0150](../../../docs/decisions/0150-administrator-actions-ask-the-persons-password-each-time.md)).
//!
//! The owner, 2026-11-09: *"Keep both in step"* — the password a person signs
//! in with is the one polkit and `sudo` ask for. This crate checks the password
//! against alo's own accounts and nothing else (ADR 0024), so it does not keep
//! the system's list itself: it hands the password, once a sign-in has
//! succeeded, to whatever keeps it, and goes on. On a machine that is
//! `alo_administrator::keeping`, through a socket only root may open.
//!
//! **A keeper that fails never stops a sign-in.** The person is owed their
//! desktop, and what the system's list could not be given is written down
//! where the desktop's password sheet reads it, so a password asked for later
//! is refused truthfully rather than as *not right*.

/// What keeps the system's password in step with the one a person just signed
/// in with.
///
/// Called once per successful sign-in, after the password was checked against
/// alo's own accounts and before a session is asked for, with the person's
/// number and what they typed. It says what happened on its own — the journal,
/// the state file — and answers nothing, because nothing the sign-in does next
/// depends on it.
pub trait KeepingTheSystemPassword {
    /// Keep `password` as `person`'s in the system's list.
    fn keep(&self, person: u32, password: &str);
}
