//! What a machine looks like before anybody changes anything.
//!
//! **The defaults live in the code, not in the file a person's settings are
//! written to** — the same shape as `alo-shortcuts`, and for the same reason. A
//! file that held every setting would freeze the day it was written: a release
//! that shipped a better wallpaper, or moved the schedule an hour, would reach
//! nobody who had ever opened the appearance panel. So what is stored is the
//! difference ([`crate::changes`]), and everything else comes from the release
//! that is running.
//!
//! **A fresh machine is not grey.** `docs/features.md` promises wallpapers in
//! the image, so what ships here is a picture rather than a colour — named
//! [`THE_WALLPAPER`], which is a promise the image has to keep. A machine whose
//! image ships no wallpaper by that name shows nothing behind its windows, and
//! that is the image's bug rather than a case this crate papers over with a
//! colour nobody chose.
//!
//! **What ships is light, all day.** Not because light is the real one — the
//! design brief says to treat the two as equals — but because a machine that
//! changed its appearance on the first evening, before its owner asked it to,
//! would be a machine deciding something on their behalf. The schedule is one
//! switch away and [`Shipped::the_evening_schedule`] is the schedule the switch
//! turns on.

use crate::lock::Lock;
use crate::picture::Picture;
use crate::scheme::{Following, Schedule, Scheme};
use crate::text::TextScale;
use crate::time::TimeOfDay;

/// The name the image gives the wallpaper it ships.
pub const THE_WALLPAPER: &str = "alo";

/// The schedule a person gets when they turn *follow the time of day* on: dark
/// from six in the evening, light again at seven in the morning.
const EVENING: Schedule = Schedule::shipped(TimeOfDay::shipped(18, 0), TimeOfDay::shipped(7, 0));

/// What alo OS looks like out of the box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shipped {
    /// What is behind the windows.
    background: Picture,
    /// What the lock screen shows.
    lock: Lock,
    /// What decides light or dark.
    following: Following,
    /// How big the text is.
    text: TextScale,
}

impl Shipped {
    /// The appearance the image ships.
    #[must_use]
    pub fn of_the_image() -> Self {
        Self {
            background: Picture::unchecked(THE_WALLPAPER),
            lock: Lock::TheDesktop,
            following: Following::Always(Scheme::Light),
            text: TextScale::ordinary(),
        }
    }

    /// A different set of defaults — a release being tried out against a
    /// person's changes, or a test of what a new default would do to them.
    #[must_use]
    pub fn of(background: Picture, lock: Lock, following: Following, text: TextScale) -> Self {
        Self {
            background,
            lock,
            following,
            text,
        }
    }

    /// The schedule *follow the time of day* means when a person turns it on.
    #[must_use]
    pub fn the_evening_schedule() -> Schedule {
        EVENING
    }

    /// What is behind the windows.
    #[must_use]
    pub fn background(&self) -> &Picture {
        &self.background
    }

    /// What the lock screen shows.
    #[must_use]
    pub fn lock(&self) -> &Lock {
        &self.lock
    }

    /// What decides light or dark.
    #[must_use]
    pub fn following(&self) -> Following {
        self.following
    }

    /// How big the text is.
    #[must_use]
    pub fn text(&self) -> TextScale {
        self.text
    }
}

impl Default for Shipped {
    fn default() -> Self {
        Self::of_the_image()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::picture::{Fitting, Of};

    /// **What ships is held to the rules a person is held to.** The wallpaper's
    /// name and the schedule's two times are built by the compiler, so this is
    /// what puts them back through the checks — or the checks are advice.
    #[test]
    fn what_ships_would_be_accepted_from_a_person() {
        assert!(Picture::shipped(THE_WALLPAPER).is_ok());
        let evening = Shipped::the_evening_schedule();
        assert_eq!(
            Schedule::checked(evening.dark_from(), evening.light_from()).unwrap(),
            evening
        );
        assert!(
            TimeOfDay::checked(evening.dark_from().hour(), evening.dark_from().minute()).is_ok()
        );
        assert!(
            TimeOfDay::checked(evening.light_from().hour(), evening.light_from().minute()).is_ok()
        );
    }

    /// A fresh machine shows the picture the image shipped, not a colour and not
    /// nothing.
    #[test]
    fn a_fresh_machine_is_not_grey() {
        let shipped = Shipped::of_the_image();
        assert_eq!(
            shipped.background().of(),
            &Of::Shipped(THE_WALLPAPER.to_owned())
        );
        assert_eq!(shipped.background().fitting(), Fitting::Fill);
        assert_eq!(shipped, Shipped::default());
    }

    /// **A machine does not change its own appearance on the first evening.**
    /// What ships is light all day; the schedule exists and is one switch away.
    #[test]
    fn what_ships_is_light_all_day_until_somebody_asks() {
        let shipped = Shipped::of_the_image();
        for hour in 0..24 {
            assert_eq!(
                shipped.following().at(TimeOfDay::checked(hour, 0).unwrap()),
                Scheme::Light,
                "at {hour} o'clock"
            );
        }
        assert_eq!(shipped.lock(), &Lock::TheDesktop);
        assert_eq!(shipped.text(), TextScale::ordinary());
    }

    /// The evening schedule is dark after six and light at seven, which is what
    /// the switch in the settings panel means.
    #[test]
    fn the_evening_schedule_is_dark_after_six() {
        let evening = Shipped::the_evening_schedule();
        assert_eq!(evening.dark_from().to_string(), "18:00");
        assert_eq!(evening.light_from().to_string(), "07:00");
        assert_eq!(evening.at(TimeOfDay::checked(20, 0).unwrap()), Scheme::Dark);
        assert_eq!(evening.at(TimeOfDay::checked(9, 0).unwrap()), Scheme::Light);
    }

    /// A release trying out different defaults is an ordinary thing to build.
    #[test]
    fn a_different_release_can_ship_different_defaults() {
        let other = Shipped::of(
            Picture::shipped("harbour").unwrap(),
            Lock::TheDesktop,
            Following::from(Shipped::the_evening_schedule()),
            TextScale::percent(125).unwrap(),
        );
        assert_ne!(other, Shipped::of_the_image());
        assert_eq!(other.text().as_percent(), 125);
    }
}
