//! The person's session at this machine: open, or locked.
//!
//! [`Seat`] is the one value every locking decision is asked of. It is made
//! from a session a sign-in opened and from nothing else, it is locked by
//! [`Seat::locked`], and it is unlocked by `crate::unlocking` — through the
//! greeting, and through nothing weaker.
//!
//! # Locked is not signed out
//!
//! Both states hold the session. There is no third state, no method that
//! consumes a seat into nothing and no call anywhere in this crate that could
//! end a session: that is `alo-sessiond`'s and the person's, through signing
//! out. `tests/nothing_here_ends_a_session.rs` reads this crate's source and
//! dependencies for it rather than trusting this paragraph.

use std::time::SystemTime;

use alo_accounts::Session;
use alo_appearance::{Appearance, DisplayId};
use alo_egress::Indicator;
use alo_indicator::Lamp;
use alo_overlay::Summoning;

use crate::arriving::Arrived;
use crate::battery::Battery;
use crate::locked::Locked;
use crate::screen::LockScreen;

/// The person's session: open, or behind the lock screen.
///
/// `N` is what a notification is — decided by whatever delivers them, and held
/// here without being looked at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Seat<N> {
    /// Signed in and unlocked: the person is at their desktop.
    Open(Session),
    /// Locked: still signed in, still running, and showing only the lock screen.
    Locked(Locked<N>),
}

impl<N> Seat<N> {
    /// The seat a sign-in opened.
    #[must_use]
    pub const fn opened(session: Session) -> Self {
        Self::Open(session)
    }

    /// Whether the lock screen is up.
    #[must_use]
    pub const fn is_locked(&self) -> bool {
        matches!(self, Self::Locked(_))
    }

    /// Whose session this is, whichever state it is in — for the shell's own
    /// wiring, never for a lock screen.
    #[must_use]
    pub const fn session(&self) -> &Session {
        match self {
            Self::Open(session) => session,
            Self::Locked(locked) => locked.session(),
        }
    }

    /// Lock the screen.
    ///
    /// The session carries on — nothing running is handed in here, so nothing
    /// running can be stopped (`crate::running`). The agent's overlay, if it
    /// is open, closes: it is the signed-in person's, and a lock screen with
    /// the agent still up on it would be the agent answering a stranger.
    ///
    /// Locking a seat that is already locked changes nothing, and keeps what
    /// it was holding.
    #[must_use]
    pub fn locked(self, summoning: &mut Summoning) -> Self {
        if summoning.is_open() {
            // `dismissed` refuses only an overlay that is not open, which the
            // line above has just ruled out.
            let _closed = summoning.dismissed();
        }
        match self {
            Self::Open(session) => Self::Locked(Locked::over(session)),
            Self::Locked(locked) => Self::Locked(locked),
        }
    }

    /// A notification has arrived for this person.
    ///
    /// On an open seat it is handed straight back to be shown. On a locked
    /// one it is held, nothing comes back to draw, and the person is given it
    /// when they unlock.
    pub fn arrives(&mut self, notification: N) -> Arrived<N> {
        match self {
            Self::Open(_) => Arrived::ToShow(notification),
            Self::Locked(locked) => {
                locked.holds(notification);
                Arrived::Held
            }
        }
    }

    /// What the lock screen on this display shows at this moment — or
    /// [`None`] when the seat is open and there is no lock screen.
    ///
    /// Everything it may show is a parameter, and nothing the seat holds is:
    /// the lock image is `alo-appearance`'s for this display, the battery is
    /// whatever reads it, and the egress light is this machine's own
    /// indicator, reduced to a lamp that names nothing.
    #[must_use]
    pub fn lock_screen(
        &self,
        now: SystemTime,
        appearance: &Appearance,
        display: &DisplayId,
        battery: Option<Battery>,
        indicator: &Indicator,
    ) -> Option<LockScreen> {
        match self {
            Self::Open(_) => None,
            Self::Locked(_) => Some(LockScreen::of(
                now,
                appearance.lock_on(display),
                battery,
                Lamp::of(indicator),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use alo_overlay::{Compositor, Pressed, SurfaceRefused, SurfaceRequest};

    use super::*;
    use crate::testing::{a_notification, anna};

    /// A compositor with room for the overlay.
    struct Room;

    impl Compositor for Room {
        fn honour(&mut self, _asked: SurfaceRequest) -> Result<(), SurfaceRefused> {
            Ok(())
        }
    }

    /// **A locked seat is still the same session.** Locking changes where the
    /// person is, not who is signed in.
    #[test]
    fn a_locked_seat_is_still_the_same_session() {
        let seat = Seat::<String>::opened(anna());
        let locked = seat.clone().locked(&mut Summoning::closed());
        assert!(locked.is_locked());
        assert_eq!(locked.session(), seat.session());
    }

    /// **The agent's overlay, open when the machine locks, is closed** — and a
    /// seat locked twice is still one lock, holding what it held.
    #[test]
    fn an_overlay_open_when_the_machine_locks_is_closed() {
        let mut summoning = Summoning::closed();
        assert_eq!(summoning.press(Some(&mut Room)), Pressed::Summoned);

        let mut seat = Seat::opened(anna()).locked(&mut summoning);
        assert!(!summoning.is_open());

        assert_eq!(seat.arrives(a_notification("one")), Arrived::Held);
        let again = seat.clone().locked(&mut summoning);
        assert_eq!(again, seat);
    }

    /// **An open seat hands a notification back to be shown**, so the rule
    /// about holding is a rule about being locked and nothing wider.
    #[test]
    fn an_open_seat_shows_what_arrives() {
        let mut seat = Seat::opened(anna());
        assert_eq!(
            seat.arrives(a_notification("hello")),
            Arrived::ToShow(a_notification("hello"))
        );
    }
}
