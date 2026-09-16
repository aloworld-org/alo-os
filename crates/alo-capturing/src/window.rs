//! One window, as much of it as a capture needs to know.
//!
//! The number the compositor records it under, whose session it belongs to,
//! whether it is the lock screen's own window, and where it sits on the screen.
//! Nothing else: not its title, not what application drew it, not what is in
//! it. A capture crops to a rectangle and hands the bytes on, and a window's
//! title is exactly the kind of thing the plan says a screenshot's file name
//! must not carry.
//!
//! # Why a window carries its own rectangle
//!
//! Because the rented mechanism is told a rectangle, and a window *is* a
//! rectangle on a screen. Reaching for a window by a handle and letting the
//! mechanism work out where it is would put the question *which pixels* in two
//! places — here, where it can be tested with no machine, and there, where it
//! cannot.
//!
//! # And why the lock screen is a window as well as a session state
//!
//! [`crate::Session::the_lock_screen_is_up`] is the machine's state; this is
//! the window itself. They are different facts and both have to be refused: a
//! compositor can hold the lock screen's surface while the session behind it is
//! unlocked — during the moment it is coming down — and a capture asked for in
//! that instant must not get it.

use crate::region::Region;
use crate::session::WhoseSession;

/// The number the compositor records one window under.
///
/// Not stable across a restart, and nothing here pretends otherwise. It is
/// carried so that what a person points at and what the compositor knows about
/// are the same thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(u32);

impl WindowId {
    /// The number the compositor recorded.
    #[must_use]
    pub const fn recorded(number: u32) -> Self {
        Self(number)
    }

    /// The number itself, for showing and for pointing at.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// One window on the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// Which window the compositor says this is.
    at: WindowId,
    /// Whose session it belongs to.
    whose: WhoseSession,
    /// Where it sits on the screen.
    where_it_is: Region,
    /// Whether it is the lock screen's own window.
    is_the_lock_screen: bool,
}

impl Window {
    /// An ordinary window of somebody's, at this place on the screen.
    #[must_use]
    pub fn of(at: WindowId, whose: &WhoseSession, where_it_is: Region) -> Self {
        Self {
            at,
            whose: whose.clone(),
            where_it_is,
            is_the_lock_screen: false,
        }
    }

    /// The lock screen's own window.
    ///
    /// A constructor of its own rather than a flag on the one above, so that
    /// the compositor saying *this is the lock screen* is a deliberate sentence
    /// and not a `true` in a fourth argument position.
    #[must_use]
    pub fn the_lock_screen(at: WindowId, whose: &WhoseSession, where_it_is: Region) -> Self {
        Self {
            at,
            whose: whose.clone(),
            where_it_is,
            is_the_lock_screen: true,
        }
    }

    /// Which window the compositor says this is.
    #[must_use]
    pub const fn at(&self) -> WindowId {
        self.at
    }

    /// Whose session it belongs to.
    #[must_use]
    pub const fn whose(&self) -> &WhoseSession {
        &self.whose
    }

    /// Where it sits on the screen.
    #[must_use]
    pub const fn where_it_is(&self) -> Region {
        self.where_it_is
    }

    /// Whether it is the lock screen's own window.
    #[must_use]
    pub const fn is_the_lock_screen(&self) -> bool {
        self.is_the_lock_screen
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Where the windows in these tests sit.
    fn somewhere() -> Region {
        Region::of(10, 20, 300, 200).unwrap()
    }

    /// **A window is its number, its owner and its rectangle**, and an ordinary
    /// one is not the lock screen: nothing is refused for a lock screen nobody
    /// said was there.
    #[test]
    fn a_window_is_its_number_its_owner_and_where_it_is() {
        let anna = WhoseSession::of("anna");
        let window = Window::of(WindowId::recorded(7), &anna, somewhere());
        assert_eq!(window.at(), WindowId::recorded(7));
        assert_eq!(window.at().as_u32(), 7);
        assert_eq!(window.whose(), &anna);
        assert_eq!(window.where_it_is(), somewhere());
        assert!(!window.is_the_lock_screen());
    }

    /// **The lock screen's window says so**, and it takes its own constructor
    /// to say it — a compositor cannot produce one by accident.
    #[test]
    fn the_lock_screens_window_says_that_is_what_it_is() {
        let anna = WhoseSession::of("anna");
        let locked = Window::the_lock_screen(WindowId::recorded(1), &anna, somewhere());
        assert!(locked.is_the_lock_screen());
        assert_ne!(
            locked,
            Window::of(WindowId::recorded(1), &anna, somewhere())
        );
    }

    /// **Two windows of two people are two windows**, which is the comparison
    /// the refusal for another person's window is made on.
    #[test]
    fn a_window_carries_whose_session_it_is_in() {
        let annas = Window::of(
            WindowId::recorded(7),
            &WhoseSession::of("anna"),
            somewhere(),
        );
        let bos = Window::of(WindowId::recorded(8), &WhoseSession::of("bo"), somewhere());
        assert_ne!(annas.whose(), bos.whose());
    }
}
