//! Do-not-disturb: whether notifications are being held right now, and why.
//!
//! **Four reasons, one of which a person cannot turn off.** Two are theirs —
//! they switched it on, or it is inside the hours they set aside — and two are
//! the machine's: something is reading the screen, or the machine could not
//! find out whether anything is.
//!
//! # The screen wins, and it wins first
//!
//! `docs/features.md` promises an indicator whenever the screen is in use, and
//! the capture plan's `alo-in-use` is the machine's one honest answer to *what
//! is reading my screen*. [`Quiet::now`] asks it **before** it asks the
//! person's settings, so a notification is never shown into a meeting or a
//! recording because do-not-disturb happened to be off. There is no setting
//! here that turns that off and nothing to add one to: the whole value of it is
//! that a person who is about to share their screen does not have to remember
//! anything.
//!
//! # A machine that cannot tell holds too
//!
//! [`Quiet::now`] takes the screen as [`Option`], and [`None`] — the media
//! server could not be asked — holds notifications exactly as a screen that is
//! being recorded does, saying so in its own words. `alo_in_use::InUse` makes
//! the same argument about its own indicator: an empty list means the room is
//! quiet, and it must never also mean the machine could not tell. Here the
//! stakes are the other way up, so the answer is: when in doubt, hold. A
//! private sentence shown into a recording cannot be taken back.
//!
//! # Nothing here reads a clock
//!
//! The time of day is handed in, so that a settings panel previewing quiet
//! hours and the thing obeying them cannot disagree.

use alo_appearance::TimeOfDay;
use alo_in_use::{InUse, Used};
use alo_strings::{Filling, Said, Strings, Word};

use crate::changes::Settings;
use crate::words;

/// Why notifications are being held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Because {
    /// The person turned notifications off themselves.
    YouAskedForQuiet,
    /// It is inside the hours they set aside.
    TheQuietHours,
    /// Something is reading the screen.
    TheScreenIsShared,
    /// The machine could not find out whether anything is reading the screen.
    ItCannotTellAboutTheScreen,
}

impl Because {
    /// All four, in the order [`Quiet::now`] asks them.
    pub const EVERY: [Self; 4] = [
        Self::TheScreenIsShared,
        Self::ItCannotTellAboutTheScreen,
        Self::YouAskedForQuiet,
        Self::TheQuietHours,
    ];

    /// Whether this is a reason the person chose.
    ///
    /// The two that are not are the machine's, and no setting anywhere turns
    /// either of them off.
    #[must_use]
    pub const fn is_the_persons_own(self) -> bool {
        matches!(self, Self::YouAskedForQuiet | Self::TheQuietHours)
    }

    /// The string this crate declares for it.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::YouAskedForQuiet => words::HELD_YOU_ASKED_FOR_QUIET,
            Self::TheQuietHours => words::HELD_QUIET_HOURS,
            Self::TheScreenIsShared => words::HELD_SCREEN_IS_SHARED,
            Self::ItCannotTellAboutTheScreen => words::HELD_CANNOT_TELL_ABOUT_THE_SCREEN,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    ///
    /// It says why nothing was shown and **nothing about what was held** — no
    /// sender, no title, no count — because a reason is read on the same screen
    /// the notifications were kept off.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// Whether notifications are being held right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quiet {
    /// Notifications are shown as they arrive.
    No,
    /// They are being held, for this reason.
    Yes(Because),
}

impl Quiet {
    /// Whether notifications are being held at this moment.
    ///
    /// `screen` is what this machine's media server says is in use, or [`None`]
    /// when it could not be asked. The screen is looked at first and the
    /// person's settings second, so nothing a person can set turns off the
    /// holding that happens while their screen is being read.
    #[must_use]
    pub fn now(settings: &Settings, screen: Option<&InUse>, at: TimeOfDay) -> Self {
        let Some(in_use) = screen else {
            return Self::Yes(Because::ItCannotTellAboutTheScreen);
        };
        if in_use.is_in_use(Used::Screen) {
            return Self::Yes(Because::TheScreenIsShared);
        }
        if settings.do_not_disturb {
            return Self::Yes(Because::YouAskedForQuiet);
        }
        if settings.quiet_hours.is_some_and(|hours| hours.holds(at)) {
            return Self::Yes(Because::TheQuietHours);
        }
        Self::No
    }

    /// Whether notifications are being held.
    #[must_use]
    pub const fn is_holding(self) -> bool {
        matches!(self, Self::Yes(_))
    }

    /// Why, or [`None`] when nothing is being held.
    #[must_use]
    pub const fn because(self) -> Option<Because> {
        match self {
            Self::Yes(because) => Some(because),
            Self::No => None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::quiet_hours::QuietHours;
    use crate::testing::{
        at, in_english, nothing_is_in_use, the_screen_is_being_recorded, what_alo_os_ships,
    };

    /// **A machine nobody changed, with a quiet room, shows notifications.**
    #[test]
    fn a_machine_nobody_changed_shows_notifications() {
        let quiet = Quiet::now(&what_alo_os_ships(), Some(&nothing_is_in_use()), at(15, 0));
        assert_eq!(quiet, Quiet::No);
        assert!(!quiet.is_holding());
        assert_eq!(quiet.because(), None);
    }

    /// **The person turning it on holds everything**, and says it was their
    /// own choice.
    #[test]
    fn the_person_turning_it_on_holds_everything() {
        let mut settings = what_alo_os_ships();
        settings.do_not_disturb = true;
        assert_eq!(
            Quiet::now(&settings, Some(&nothing_is_in_use()), at(15, 0)),
            Quiet::Yes(Because::YouAskedForQuiet)
        );
    }

    /// **Inside the hours they set aside, notifications wait** — and outside
    /// them they do not.
    #[test]
    fn inside_the_hours_they_set_aside_notifications_wait() {
        let mut settings = what_alo_os_ships();
        settings.quiet_hours = Some(QuietHours::these_two_times(at(23, 0), at(7, 0)).unwrap());
        for (hour, minute) in [(23_u8, 0_u8), (2, 30), (6, 59)] {
            assert_eq!(
                Quiet::now(&settings, Some(&nothing_is_in_use()), at(hour, minute)),
                Quiet::Yes(Because::TheQuietHours),
                "{hour:02}:{minute:02}"
            );
        }
        for (hour, minute) in [(7_u8, 0_u8), (15, 0), (22, 59)] {
            assert_eq!(
                Quiet::now(&settings, Some(&nothing_is_in_use()), at(hour, minute)),
                Quiet::No,
                "{hour:02}:{minute:02}"
            );
        }
    }

    /// **A screen that is being read holds notifications whatever the person
    /// set**, and says so as the machine's own doing rather than theirs. This
    /// is the clause with no setting behind it: a person about to share their
    /// screen has to remember nothing.
    #[test]
    fn a_screen_being_read_holds_notifications_whatever_the_person_set() {
        let quiet = Quiet::now(
            &what_alo_os_ships(),
            Some(&the_screen_is_being_recorded()),
            at(15, 0),
        );
        assert_eq!(quiet, Quiet::Yes(Because::TheScreenIsShared));
        assert!(!Because::TheScreenIsShared.is_the_persons_own());

        // And it still holds with everything a person could set turned off.
        let settings = Settings {
            do_not_disturb: false,
            quiet_hours: None,
        };
        assert_eq!(
            Quiet::now(&settings, Some(&the_screen_is_being_recorded()), at(15, 0)),
            Quiet::Yes(Because::TheScreenIsShared)
        );
    }

    /// **A machine that cannot tell holds too.** An answer it could not get is
    /// not an answer that nothing is watching.
    #[test]
    fn a_machine_that_cannot_tell_holds_too() {
        let quiet = Quiet::now(&what_alo_os_ships(), None, at(15, 0));
        assert_eq!(quiet, Quiet::Yes(Because::ItCannotTellAboutTheScreen));
        assert!(!Because::ItCannotTellAboutTheScreen.is_the_persons_own());
    }

    /// **Every reason reads, no two read the same, and none names what was
    /// held.** The sentence is read where the notifications are not.
    #[test]
    fn every_reason_reads_and_none_names_what_was_held() {
        let strings = in_english();
        let mut seen: Vec<String> = Vec::new();
        for because in Because::EVERY {
            let said = because.said(&strings);
            assert!(!said.is_a_bug(), "{because:?} is not declared");
            assert!(said.unfilled().is_empty(), "{because:?}: {said}");
            let text = said.into_text();
            assert!(!seen.contains(&text), "two reasons both read {text}");
            seen.push(text);
        }
        assert_eq!(
            Because::EVERY
                .iter()
                .filter(|because| because.is_the_persons_own())
                .count(),
            2
        );
    }
}
