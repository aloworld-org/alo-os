//! What the lock screen may show, which is four things.
//!
//! **The time, the lock image, the battery, and that the machine is locked.**
//! The lock screen is the one surface a stranger at the desk is guaranteed to
//! see, and on most systems it leaks: a notification's first line, the name of
//! the document somebody was writing, the agent's last question, an approval
//! waiting to be tapped. So what may be on it is a type rather than a policy,
//! and the type has room for exactly those four and for nothing a setting could
//! add later.
//!
//! And for one more thing that is not a leak but a law: **the egress light**.
//! *Nothing leaves silently* does not pause because the screen is covered, so a
//! departure while locked lights it. It is carried as an `alo_indicator::Lamp`,
//! which is dark or lit for a number and has no line in it — so it says that
//! something is leaving and never what, where to or for whom.
//!
//! # Nothing else fits
//!
//! There is no field for a notification, a name, a window or a question, and
//! no generic parameter a caller could fill with one: a [`LockScreen`] is the
//! same type whatever the seat is holding for later. Nor is there a setting
//! that shows a preview:
//!
//! ```compile_fail
//! let previews = alo_locking::LockScreen::with_previews;
//! ```
//!
//! ```compile_fail
//! let previews = alo_locking::Previews::Shown;
//! ```
//!
//! A person who wants to read their messages unlocks.
//!
//! # Nothing here draws
//!
//! Where the clock goes, how large, in which typeface and in what format are
//! the shell's and `alo-formats`'. This is what the shell is permitted to put
//! there.

use std::time::SystemTime;

use alo_appearance::Background;
use alo_indicator::Lamp;
use alo_strings::{Said, Strings};

use crate::battery::Battery;
use crate::refusing::NotWhileLocked;

/// Everything the lock screen of one display may show, at one moment.
///
/// Made only by [`crate::Seat::lock_screen`], and only for a seat that is
/// locked. It has no public field and no constructor of its own:
///
/// ```compile_fail
/// let display = alo_appearance::DisplayId::named("eDP-1").expect("a display");
/// let screen = alo_locking::LockScreen {
///     at: std::time::SystemTime::UNIX_EPOCH,
///     image: alo_appearance::Appearance::shipped().lock_on(&display),
///     battery: None,
///     lamp: alo_indicator::Lamp::of(&alo_egress::Indicator::default()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockScreen {
    /// The moment it shows.
    at: SystemTime,
    /// What is behind the time — `alo-appearance`'s answer for this display.
    image: Background,
    /// The battery, on a machine that has one.
    battery: Option<Battery>,
    /// Whether anything is leaving this machine, and how many things.
    lamp: Lamp,
}

impl LockScreen {
    /// The lock screen at this moment. `pub(crate)`: the seat is the only
    /// thing that knows whether there is a lock screen at all.
    pub(crate) const fn of(
        at: SystemTime,
        image: Background,
        battery: Option<Battery>,
        lamp: Lamp,
    ) -> Self {
        Self {
            at,
            image,
            battery,
            lamp,
        }
    }

    /// The time to show.
    #[must_use]
    pub const fn at(&self) -> SystemTime {
        self.at
    }

    /// The lock image for this display.
    #[must_use]
    pub const fn image(&self) -> &Background {
        &self.image
    }

    /// The battery, or [`None`] on a machine without one.
    #[must_use]
    pub const fn battery(&self) -> Option<Battery> {
        self.battery
    }

    /// The egress light: dark, or lit for how many things are leaving — never
    /// what they are.
    #[must_use]
    pub const fn lamp(&self) -> Lamp {
        self.lamp
    }

    /// That the machine is locked, in the language the person reads.
    #[must_use]
    pub fn locked_said(&self, strings: &Strings) -> Said {
        NotWhileLocked.said(strings)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_appearance::{Appearance, DisplayId};
    use alo_capability::Grantee;
    use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};

    use crate::battery::Battery;
    use crate::testing::{a_notification, anna, in_english, noon, the_laptop};
    use crate::{Arrived, Seat};

    /// **The lock screen shows the time, the lock image, the battery, and that
    /// the machine is locked** — and the lock image is `alo-appearance`'s
    /// answer for that display, read.
    #[test]
    fn the_lock_screen_shows_the_time_the_image_the_battery_and_that_it_is_locked() {
        let appearance = Appearance::shipped();
        let battery = Battery::reading(64, false).unwrap();
        let seat = Seat::<String>::opened(anna()).locked(&mut alo_overlay::Summoning::closed());

        let screen = seat
            .lock_screen(
                noon(),
                &appearance,
                &the_laptop(),
                Some(battery),
                &Indicator::default(),
            )
            .unwrap();

        assert_eq!(screen.at(), noon());
        assert_eq!(screen.image(), &appearance.lock_on(&the_laptop()));
        assert_eq!(screen.battery(), Some(battery));
        assert!(screen.lamp().is_dark());
        assert_eq!(
            screen.locked_said(&in_english()).text(),
            "This machine is locked"
        );
    }

    /// **An unlocked seat has no lock screen at all**, so nothing can be drawn
    /// as one over a desktop that is in use.
    #[test]
    fn an_unlocked_seat_has_no_lock_screen() {
        let seat = Seat::<String>::opened(anna());
        assert!(
            seat.lock_screen(
                noon(),
                &Appearance::shipped(),
                &the_laptop(),
                None,
                &Indicator::default()
            )
            .is_none()
        );
    }

    /// **Nobody's name is on it.** The session's owner is the first thing a
    /// stranger would learn from a lock screen that greeted them by name.
    #[test]
    fn the_lock_screen_does_not_name_whose_machine_it_is() {
        let seat = Seat::<String>::opened(anna()).locked(&mut alo_overlay::Summoning::closed());
        let screen = seat
            .lock_screen(
                noon(),
                &Appearance::shipped(),
                &the_laptop(),
                None,
                &Indicator::default(),
            )
            .unwrap();
        let everything = format!("{screen:?}");
        assert!(!everything.contains("anna"), "{everything}");
    }

    /// **A different display shows its own lock image** — the lock screen is
    /// per display, and each is `alo-appearance`'s answer for that display.
    #[test]
    fn each_display_has_the_lock_image_appearance_gives_it() {
        let appearance = Appearance::shipped();
        let projector = DisplayId::named("HDMI-A-1 Projector").unwrap();
        let seat = Seat::<String>::opened(anna()).locked(&mut alo_overlay::Summoning::closed());
        let screen = seat
            .lock_screen(noon(), &appearance, &projector, None, &Indicator::default())
            .unwrap();
        assert_eq!(screen.image(), &appearance.lock_on(&projector));
    }

    /// **The egress light still fires while locked, and names nothing.** A
    /// departure that begins behind the lock screen lights the lamp on it; the
    /// lock screen carries the count and no line, so the destination, the
    /// agent and the reason are nowhere in it — not in its sentence and not in
    /// anything it holds.
    #[test]
    fn the_egress_light_is_lit_while_locked_without_naming_what_left() {
        let mut summoning = alo_overlay::Summoning::closed();
        let seat = Seat::<String>::opened(anna()).locked(&mut summoning);
        let mut indicator = Indicator::default();
        let departing = indicator
            .beginning(
                &EgressPolicy::Anywhere,
                Leaving::because(
                    &Grantee::named("@files"),
                    Why::Fetching,
                    Destination::at("invoices.example").unwrap(),
                ),
                noon(),
            )
            .unwrap();

        let screen = seat
            .lock_screen(
                noon(),
                &Appearance::shipped(),
                &the_laptop(),
                None,
                &indicator,
            )
            .unwrap();
        assert!(screen.lamp().is_lit());
        assert_eq!(screen.lamp().how_many(), 1);

        let strings =
            alo_strings::Strings::of(alo_saying::everything_this_machine_can_say().unwrap());
        let said = screen.lamp().said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        for private in ["invoices.example", "@files"] {
            assert!(!said.text().contains(private), "{said}");
            assert!(!format!("{screen:?}").contains(private), "{screen:?}");
        }

        // And it goes dark when the departure ends, locked or not.
        indicator.ended(departing);
        let after = seat
            .lock_screen(
                noon(),
                &Appearance::shipped(),
                &the_laptop(),
                None,
                &indicator,
            )
            .unwrap();
        assert!(after.lamp().is_dark());
    }

    /// **A notification arriving while locked is held and never drawn there.**
    /// The lock screen after three notifications is the lock screen before
    /// them, value for value.
    #[test]
    fn a_notification_arriving_while_locked_changes_nothing_on_the_lock_screen() {
        let mut seat = Seat::opened(anna()).locked(&mut alo_overlay::Summoning::closed());
        let before = seat
            .lock_screen(
                noon(),
                &Appearance::shipped(),
                &the_laptop(),
                None,
                &Indicator::default(),
            )
            .unwrap();

        for sent in ["Re: the Lisbon offer", "Your test results", "Anna, call me"] {
            assert_eq!(seat.arrives(a_notification(sent)), Arrived::Held);
        }

        let after = seat
            .lock_screen(
                noon(),
                &Appearance::shipped(),
                &the_laptop(),
                None,
                &Indicator::default(),
            )
            .unwrap();
        assert_eq!(before, after);
        assert!(!format!("{after:?}").contains("Lisbon"));
    }
}
