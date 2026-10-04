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
//! **A fresh machine is not grey**, and since ADR 0075 it is not a photograph
//! either. alo OS ships no wallpaper: the canvas plane's own surface is the
//! desktop, so what ships here is that surface — [`Token::Porcelain`], which
//! `token.rs` already named *the workspace canvas* long before this crate had to
//! choose one.
//!
//! **That is a colour the palette chose, not a colour this file picked.** The
//! difference matters: a default invented here would be one more thing to keep in
//! step with the design, and the whole argument for defaults living in code is
//! that a release can move them. This one moves when the palette does.
//!
//! **What ships is light, all day.** Not because light is the real one — the
//! design brief says to treat the two as equals — but because a machine that
//! changed its appearance on the first evening, before its owner asked it to,
//! would be a machine deciding something on their behalf. The schedule is one
//! switch away and [`Shipped::the_evening_schedule`] is the schedule the switch
//! turns on.

use crate::accent::Accent;
use crate::background::Background;
use crate::lock::Lock;
use crate::scheme::{Following, Schedule, Scheme};
use crate::text::TextScale;
use crate::time::TimeOfDay;
use crate::token::Token;

/// The surface a machine shows before anybody changes anything.
///
/// The palette's own name for it. `THE_WALLPAPER` stood here until 2026-09-29
/// and named a picture the image installed; ADR 0075 removed it.
/// **It was [`Token::Porcelain`] until 2026-10-04, and that was the wrong
/// name for it.** This is the workspace canvas — the desktop a person's windows
/// sit on — which the design file calls `bg/canvas`. Resolving `porcelain` by
/// role rather than by resemblance is what ADR 0092 did, and the name
/// `Porcelain` now carries `bg/surface`, which is white. Shipping that here
/// would have shipped a pure white desktop instead of the design's canvas, so
/// this follows the role and not the name it used to share.
pub const THE_SURFACE: Token = Token::Cream;

/// The schedule a person gets when they turn *follow the time of day* on: dark
/// from six in the evening, light again at seven in the morning.
const EVENING: Schedule = Schedule::shipped(TimeOfDay::shipped(18, 0), TimeOfDay::shipped(7, 0));

/// What alo OS looks like out of the box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shipped {
    /// The surface a person works on.
    background: Background,
    /// What the lock screen shows.
    lock: Lock,
    /// What decides light or dark.
    following: Following,
    /// How big the text is.
    text: TextScale,
    /// Which of the five accents the shell follows.
    accent: Accent,
}

impl Shipped {
    /// The appearance the image ships.
    #[must_use]
    pub fn of_the_image() -> Self {
        Self {
            background: Background::Colour(THE_SURFACE.colour()),
            lock: Lock::TheDesktop,
            following: Following::Always(Scheme::Light),
            text: TextScale::ordinary(),
            accent: Accent::default(),
        }
    }

    /// A different set of defaults — a release being tried out against a
    /// person's changes, or a test of what a new default would do to them.
    #[must_use]
    pub fn of(
        background: Background,
        lock: Lock,
        following: Following,
        text: TextScale,
        accent: Accent,
    ) -> Self {
        Self {
            background,
            lock,
            following,
            text,
            accent,
        }
    }

    /// The schedule *follow the time of day* means when a person turns it on.
    #[must_use]
    pub fn the_evening_schedule() -> Schedule {
        EVENING
    }

    /// The surface a person works on.
    #[must_use]
    pub const fn background(&self) -> Background {
        self.background
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

    /// Which accent the shell follows.
    #[must_use]
    pub fn accent(&self) -> Accent {
        self.accent
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

    /// **What ships is held to the rules a person is held to.** The wallpaper's
    /// name and the schedule's two times are built by the compiler, so this is
    /// what puts them back through the checks — or the checks are advice.
    #[test]
    fn what_ships_would_be_accepted_from_a_person() {
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

    /// A fresh machine shows the surface the palette named, not a colour this
    /// crate chose and not nothing.
    ///
    /// It showed a photograph until ADR 0075. What replaced it is the workspace
    /// canvas — so this asserts against the palette rather than against a
    /// literal, and a release that moves that colour moves this with it.
    ///
    /// **It asserted `Token::Porcelain` until 2026-10-04.** That name now
    /// carries `bg/surface`, which is white, and a fresh machine does not show
    /// a pure white desktop. [`THE_SURFACE`] followed the role rather than the
    /// name (ADR 0092), and so does this.
    #[test]
    fn a_fresh_machine_is_not_grey() {
        let shipped = Shipped::of_the_image();
        assert_eq!(shipped.background().colour(), Token::Cream.colour());
        assert_eq!(
            shipped.background(),
            Background::Colour(THE_SURFACE.colour())
        );
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

    /// **What ships is verdigris, and it is one of the five.** A default that
    /// was not in the offered set would be an accent a person could lose by
    /// touching the setting and never get back.
    #[test]
    fn what_ships_is_an_accent_a_person_could_also_have_chosen() {
        let shipped = Shipped::of_the_image();
        assert_eq!(shipped.accent(), Accent::Indigo);
        assert!(Accent::ALL.contains(&shipped.accent()));
        assert_eq!(
            Accent::of_colour(shipped.accent().on(Scheme::Light)),
            Ok(shipped.accent())
        );
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
            Background::from(Token::Cream.colour()),
            Lock::TheDesktop,
            Following::from(Shipped::the_evening_schedule()),
            TextScale::percent(125).unwrap(),
            Accent::Indigo,
        );
        assert_ne!(other, Shipped::of_the_image());
        assert_eq!(other.text().as_percent(), 125);
        assert_eq!(other.accent(), Accent::Indigo);
    }
}
