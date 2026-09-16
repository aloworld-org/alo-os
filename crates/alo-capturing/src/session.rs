//! Whose session a capture is taken in, and whether the lock screen is up.
//!
//! Two of the plan's refusals live on this one value, and they are the two the
//! whole task is judged on: **a window from another person's session, or the
//! lock screen, cannot be captured.**
//!
//! # Why they are one type and not two checks
//!
//! Because both are answers to *whose screen is this, right now*. A machine
//! that several people are signed into has one screen and several sessions on
//! it, and a lock screen is the state where the screen belongs to nobody yet.
//! Kept as two flags passed around separately, one of them is eventually
//! forgotten at one call site; kept here, a capture cannot be built without
//! having been handed both.
//!
//! # The lock screen is refused for every kind of capture
//!
//! Not only for a window. While the lock screen is up, the whole screen *is*
//! the lock screen, a region of it is a region of the lock screen, and a window
//! underneath it is a window the person at the keyboard has not proved they may
//! see. So the refusal is taken before what is being captured is even looked at
//! — [`crate::Screenshot::of`] asks this first.
//!
//! # And the other person is never named
//!
//! [`crate::NotTaken::AnotherPersonsWindow`] carries nothing. Somebody trying
//! to photograph a window that is not theirs does not learn whose it is from
//! being refused, which is the same instinct ADR 0004 states about a managed
//! machine: no administrator can watch a screen or act as a person, and no
//! person can read another's from an error message.

/// Whose session something belongs to.
///
/// The name the machine knows a signed-in person by, compared and never shown.
/// A newtype rather than a `String` so that a window's owner and a session's
/// owner cannot be crossed with any other name the machine holds.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WhoseSession(String);

impl WhoseSession {
    /// The session belonging to the person the machine knows by this name.
    #[must_use]
    pub fn of(whose: &str) -> Self {
        Self(whose.trim().to_owned())
    }

    /// The name itself, for comparing and for writing down.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The session a capture is taken in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Whose it is.
    whose: WhoseSession,
    /// Whether the lock screen is up over it.
    the_lock_screen_is_up: bool,
}

impl Session {
    /// A session belonging to this person, with the lock screen down.
    #[must_use]
    pub fn of(whose: &WhoseSession) -> Self {
        Self {
            whose: whose.clone(),
            the_lock_screen_is_up: false,
        }
    }

    /// The same session, with the lock screen up over it.
    #[must_use]
    pub fn behind_the_lock_screen(whose: &WhoseSession) -> Self {
        Self {
            whose: whose.clone(),
            the_lock_screen_is_up: true,
        }
    }

    /// Whose it is.
    #[must_use]
    pub const fn whose(&self) -> &WhoseSession {
        &self.whose
    }

    /// Whether the lock screen is up over it.
    #[must_use]
    pub const fn the_lock_screen_is_up(&self) -> bool {
        self.the_lock_screen_is_up
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A session is whose it is and whether the lock screen is up**, and a
    /// session made the ordinary way is not locked: a capture is never refused
    /// for a lock screen nobody said was there.
    #[test]
    fn a_session_is_whose_it_is_and_whether_the_lock_screen_is_up() {
        let anna = WhoseSession::of("anna");
        let session = Session::of(&anna);
        assert_eq!(session.whose(), &anna);
        assert!(!session.the_lock_screen_is_up());

        let locked = Session::behind_the_lock_screen(&anna);
        assert_eq!(locked.whose(), &anna);
        assert!(locked.the_lock_screen_is_up());
    }

    /// **Two people are two sessions**, which is the comparison the refusal for
    /// another person's window is made on.
    #[test]
    fn two_people_are_two_sessions() {
        assert_ne!(WhoseSession::of("anna"), WhoseSession::of("bo"));
        assert_ne!(Session::of(&WhoseSession::of("anna")), {
            Session::of(&WhoseSession::of("bo"))
        });
    }

    /// **A name is the name, with whatever a caller padded it with removed**,
    /// so `anna` and `anna ` are not two people on one machine.
    #[test]
    fn a_name_is_the_name_whatever_it_arrived_wrapped_in() {
        assert_eq!(WhoseSession::of("  anna  ").as_str(), "anna");
        assert_eq!(WhoseSession::of("  anna  "), WhoseSession::of("anna"));
    }
}
