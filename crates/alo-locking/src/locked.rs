//! A locked session: still the person's, still running, and holding what
//! arrived for them until they are back.
//!
//! [`Locked`] is the session as it was when the screen locked, and the
//! notifications that have arrived since, in the order they arrived. It has no
//! method that shows one of them, counts them or names their sender — the only
//! road they leave by is `crate::Seat::unlocks`, and it hands them to the
//! person who has just proved who they are.
//!
//! It is generic over what a notification is because this crate does not
//! decide that: the notification plan's `alo-notifying` does (plan task 6), and
//! this crate's promise — *held, never drawn on the lock screen* — is the same
//! whatever a notification turns out to carry.

use alo_accounts::Session;

/// A session behind the lock screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locked<N> {
    /// Whose session it is. Kept, never shown: the lock screen does not name
    /// the person whose machine it is.
    session: Session,
    /// What arrived while locked, oldest first.
    held: Vec<N>,
}

impl<N> Locked<N> {
    /// This session, locked, with nothing held yet.
    pub(crate) const fn over(session: Session) -> Self {
        Self {
            session,
            held: Vec::new(),
        }
    }

    /// Whose session is locked — for the unlock, and for nothing on a screen.
    pub(crate) const fn session(&self) -> &Session {
        &self.session
    }

    /// Keep this for the person, behind the lock.
    pub(crate) fn holds(&mut self, notification: N) {
        self.held.push(notification);
    }

    /// The session and everything held for it, handed back at an unlock.
    pub(crate) fn into_parts(self) -> (Session, Vec<N>) {
        (self.session, self.held)
    }
}
