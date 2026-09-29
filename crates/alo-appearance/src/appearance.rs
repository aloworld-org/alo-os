//! What this machine looks like: what the release ships, what the person
//! changed, and the two of them resolved at the moment of asking.
//!
//! Nothing is baked at load time. A change takes effect on the next frame rather
//! than at the next sign-in, and a settings panel previewing *what would this
//! look like at seven in the evening* asks the same question the compositor asks
//! at seven in the evening.
//!
//! **Three resolutions live here, and each of them is a decision rather than a
//! lookup.**
//!
//! 1. **A background is the person's choice, and a display can be an
//!    exception.** [`Appearance::background_on`] answers with the exception if
//!    they made one for that display, their choice if they made one at all, and
//!    the shipped wallpaper otherwise — so plugging in a screen shows what they
//!    chose rather than what we chose.
//! 2. **The lock screen follows the desktop, but never a rotating folder.** The
//!    desktop is seen by whoever is signed in and the lock screen by whoever
//!    walks past; a person who pointed the background at a folder of their
//!    photographs picked the folder, not the picture a machine alone in a room
//!    shows to a corridor. The reasoning is in [`crate::lock`], and
//!    [`Appearance::lock_on`] is where it happens.
//! 3. **Light and dark are answered at a time of day that is passed in**, never
//!    read, so the answer is testable and the panel and the compositor cannot
//!    disagree about what the schedule says.
//! 4. **The accent follows light and dark.** A person picks one of the five
//!    (ADR 0010), and which of its two values is showing is decided by the
//!    scheme at the moment of asking — so an accent chosen in the morning is
//!    still readable at eight in the evening. [`Appearance::accent_at`] is
//!    where that happens, and it is the only place the two questions meet.

use crate::accent::Accent;
use crate::background::Background;
use crate::changes::{Changes, Setting};
use crate::colour::Colour;
use crate::display::DisplayId;
use crate::lock::Lock;
use crate::scheme::{Following, Scheme};
use crate::shipped::Shipped;
use crate::text::TextScale;
use crate::time::TimeOfDay;

/// How this machine looks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Appearance {
    /// What this release ships.
    shipped: Shipped,
    /// What the person changed, which is the only part written down.
    changes: Changes,
}

impl Appearance {
    /// A machine as it looks before anybody changes anything.
    #[must_use]
    pub fn shipped() -> Self {
        Self::default()
    }

    /// The same, over a set of defaults that is not the shipped one.
    #[must_use]
    pub fn over(shipped: Shipped) -> Self {
        Self {
            shipped,
            changes: Changes::untouched(),
        }
    }

    /// The changes read out of a settings file, applied over these defaults.
    ///
    /// **A file is not a settings panel**: what it says is what its owner
    /// decided, and nothing here second-guesses it. What it *cannot* say is
    /// anything the types refuse — a colour that is not a colour, a rotation
    /// faster than a minute, text nobody could read — because each of those is
    /// checked where the file is read rather than here.
    #[must_use]
    pub fn with(mut self, changes: Changes) -> Self {
        self.changes = changes;
        self
    }

    /// What has been changed, which is what gets written down.
    #[must_use]
    pub fn changes(&self) -> &Changes {
        &self.changes
    }

    /// What this release ships.
    #[must_use]
    pub fn shipped_appearance(&self) -> &Shipped {
        &self.shipped
    }

    /// Put this behind the windows, on every display the person has not singled
    /// out.
    pub fn set_background(&mut self, background: Background) {
        self.changes.set_background(background);
    }

    /// Put this behind the windows on one display only.
    pub fn set_background_on(&mut self, display: DisplayId, background: Background) {
        self.changes.set_background_on(display, background);
    }

    /// Put this on the lock screen.
    pub fn set_lock(&mut self, lock: Lock) {
        self.changes.set_lock(lock);
    }

    /// Follow this for light and dark.
    pub fn follow(&mut self, following: Following) {
        self.changes.follow(following);
    }

    /// Make the text this size.
    pub fn set_text(&mut self, text: TextScale) {
        self.changes.set_text(text);
    }

    /// Make this the accent the shell follows.
    pub fn set_accent(&mut self, accent: Accent) {
        self.changes.set_accent(accent);
    }

    /// Put one setting back to what this release ships.
    ///
    /// Says whether there was anything to put back.
    pub fn put_back(&mut self, setting: Setting) -> bool {
        self.changes.forget(setting)
    }

    /// Stop singling this display out, putting it back to the background the
    /// person chose for everywhere.
    ///
    /// Says whether there was anything to put back.
    pub fn put_display_back(&mut self, display: &DisplayId) -> bool {
        self.changes.forget_display(display)
    }

    /// Put everything back to what this release ships.
    pub fn put_everything_back(&mut self) {
        self.changes.forget_everything();
    }

    /// What is behind the windows on this display.
    ///
    /// The exception the person made for this display, or the choice they made
    /// for everywhere, or the wallpaper the image shipped — in that order.
    #[must_use]
    pub fn background_on(&self, display: &DisplayId) -> Background {
        if let Some(background) = self.changes.background_on(display) {
            return *background;
        }
        self.their_background()
    }

    /// What the lock screen shows on this display.
    ///
    /// Their own if they set one, and otherwise the desktop — **which is now all
    /// there is to it.** This used to have a third case: a desktop that rotated
    /// was not followed, and the wallpaper the image shipped was shown instead,
    /// because a folder of somebody's photographs is not a thing they chose to
    /// show to whoever walks past a locked machine.
    ///
    /// ADR 0075 removed the folder and the wallpaper both, and said of that rule
    /// that it *existed only because there was a picture to betray*. There is no
    /// longer a choice to betray, and following simply follows. What replaces it
    /// is simpler and was always the part that mattered: the lock screen shows
    /// the surface and no client's pixels.
    #[must_use]
    pub fn lock_on(&self, display: &DisplayId) -> Background {
        match self.changes.lock().unwrap_or_else(|| self.shipped.lock()) {
            Lock::Its(background) => *background,
            Lock::TheDesktop => self.background_on(display),
        }
    }

    /// What decides light and dark.
    #[must_use]
    pub fn following(&self) -> Following {
        self.changes.following().unwrap_or(self.shipped.following())
    }

    /// Light or dark, at this time of day.
    #[must_use]
    pub fn scheme_at(&self, now: TimeOfDay) -> Scheme {
        self.following().at(now)
    }

    /// How big the text is.
    #[must_use]
    pub fn text(&self) -> TextScale {
        self.changes.text().unwrap_or(self.shipped.text())
    }

    /// Which accent the shell follows.
    #[must_use]
    pub fn accent(&self) -> Accent {
        self.changes.accent().unwrap_or(self.shipped.accent())
    }

    /// The accent as a colour, at this time of day.
    ///
    /// **The fourth resolution, and the reason the accent is a name rather than
    /// a hex.** An accent has a value for a light ground and one for a dark, and
    /// which of them is showing follows the same schedule everything else does —
    /// so a machine that turns dark at six turns its accent with it, and nobody
    /// has to remember to. The time is passed in, like every other question this
    /// crate answers about a clock it does not read.
    #[must_use]
    pub fn accent_at(&self, now: TimeOfDay) -> Colour {
        self.accent().on(self.scheme_at(now))
    }

    /// The background the person chose for everywhere, or the shipped one.
    fn their_background(&self) -> Background {
        self.changes
            .background()
            .copied()
            .unwrap_or_else(|| self.shipped.background())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::token::Token;

    /// The laptop's own screen.
    fn laptop() -> DisplayId {
        DisplayId::named("eDP-1").unwrap()
    }

    /// The screen on the desk.
    fn desk() -> DisplayId {
        DisplayId::named("DP-1").unwrap()
    }

    /// A time of day.
    fn at(hour: u8) -> TimeOfDay {
        TimeOfDay::checked(hour, 0).unwrap()
    }

    /// What a fresh machine shows: the surface the palette names, which is what
    /// `Shipped::of_the_image` carries.
    ///
    /// It was `the_shipped_wallpaper` and answered with a picture until ADR
    /// 0075. Three tests still ask the question, so it keeps a name rather than
    /// being inlined into each of them.
    fn the_shipped_surface() -> Background {
        Background::Colour(crate::shipped::THE_SURFACE.colour())
    }

    /// A fresh machine shows what the image shipped, on every screen, at every
    /// hour, and has written nothing down.
    #[test]
    fn a_fresh_machine_shows_what_the_image_shipped() {
        let appearance = Appearance::shipped();
        assert_eq!(appearance.background_on(&laptop()), the_shipped_surface());
        assert_eq!(appearance.background_on(&desk()), the_shipped_surface());
        assert_eq!(appearance.lock_on(&laptop()), the_shipped_surface());
        assert_eq!(appearance.scheme_at(at(22)), Scheme::Light);
        assert_eq!(appearance.text(), TextScale::ordinary());
        assert_eq!(appearance.accent(), Accent::Indigo);
        assert!(appearance.changes().is_untouched());
    }

    /// **The accent follows light and dark.** One choice, two values, and the
    /// one showing is decided by the scheme at the moment of asking rather than
    /// at the moment it was chosen — so an accent picked in the morning is still
    /// readable in the evening, on the ground that is actually behind it.
    #[test]
    fn the_accent_is_the_value_for_the_ground_it_is_on() {
        let mut appearance = Appearance::shipped();
        appearance.set_accent(Accent::Rose);
        appearance.follow(Following::from(Shipped::the_evening_schedule()));

        assert_eq!(appearance.accent(), Accent::Rose);
        assert_eq!(appearance.accent_at(at(9)), Accent::Rose.on(Scheme::Light));
        assert_eq!(appearance.accent_at(at(20)), Accent::Rose.on(Scheme::Dark));
        assert_ne!(appearance.accent_at(at(9)), appearance.accent_at(at(20)));

        assert!(appearance.put_back(Setting::Accent));
        assert!(!appearance.put_back(Setting::Accent));
        assert_eq!(
            appearance.accent_at(at(20)),
            Accent::Indigo.on(Scheme::Dark),
            "and putting it back is the shipped accent, on the right ground"
        );
    }

    /// **Deep teal is not reachable from here.** Whatever a person has chosen,
    /// and at whatever hour it is asked, the accent is never the colour that
    /// means the agent is present or acting (ADR 0010).
    #[test]
    fn the_accent_is_never_the_agents_colour() {
        let mut appearance = Appearance::shipped();
        appearance.follow(Following::from(Shipped::the_evening_schedule()));
        for accent in Accent::ALL {
            appearance.set_accent(accent);
            for hour in 0..24 {
                assert_ne!(
                    appearance.accent_at(at(hour)),
                    Token::DeepTeal.colour(),
                    "{} at {hour} o'clock",
                    accent.word().says()
                );
            }
        }
    }

    /// **A display nobody singled out shows what the person chose**, which is
    /// the whole point of one background with exceptions: plugging a projector
    /// in does not put our wallpaper on somebody else's wall.
    #[test]
    fn a_new_screen_shows_the_persons_choice_rather_than_ours() {
        let mut appearance = Appearance::shipped();
        appearance.set_background(Background::from(Token::Navy.colour()));
        assert_eq!(
            appearance.background_on(&desk()),
            Background::from(Token::Navy.colour()),
            "a screen never seen before"
        );

        appearance.set_background_on(desk(), Background::from(Token::Charcoal.colour()));
        assert_eq!(
            appearance.background_on(&desk()),
            Background::from(Token::Charcoal.colour())
        );
        assert_eq!(
            appearance.background_on(&laptop()),
            Background::from(Token::Navy.colour()),
            "and the exception is for one screen only"
        );

        assert!(appearance.put_display_back(&desk()));
        assert_eq!(
            appearance.background_on(&desk()),
            Background::from(Token::Navy.colour())
        );
    }

    /// A lock screen that follows a desktop that does not rotate shows the
    /// desktop, including the exception made for that display.
    #[test]
    fn a_lock_screen_that_follows_shows_that_screens_desktop() {
        let mut appearance = Appearance::shipped();
        appearance.set_background(Background::from(Token::Navy.colour()));
        appearance.set_background_on(desk(), Background::from(Token::Charcoal.colour()));

        assert_eq!(
            appearance.lock_on(&desk()),
            Background::from(Token::Charcoal.colour())
        );
        assert_eq!(
            appearance.lock_on(&laptop()),
            Background::from(Token::Navy.colour())
        );
    }

    /// **Dark after six**, asked at whatever hour the caller names rather than
    /// at whatever hour it happens to be.
    #[test]
    fn the_scheme_is_answered_at_a_time_that_is_given() {
        let mut appearance = Appearance::shipped();
        appearance.follow(Following::from(Shipped::the_evening_schedule()));

        assert_eq!(appearance.scheme_at(at(17)), Scheme::Light);
        assert_eq!(appearance.scheme_at(at(18)), Scheme::Dark);
        assert_eq!(appearance.scheme_at(at(3)), Scheme::Dark);
        assert_eq!(appearance.scheme_at(at(7)), Scheme::Light);

        appearance.follow(Following::from(Scheme::Dark));
        assert_eq!(appearance.scheme_at(at(12)), Scheme::Dark);
    }

    /// Every setting goes back, and putting one back never touches another.
    #[test]
    fn one_setting_goes_back_without_disturbing_the_others() {
        let mut appearance = Appearance::shipped();
        appearance.set_background(Background::from(Token::Navy.colour()));
        appearance.set_text(TextScale::percent(200).unwrap());
        appearance.follow(Following::from(Scheme::Dark));

        assert!(appearance.put_back(Setting::Text));
        assert!(!appearance.put_back(Setting::Text));
        assert_eq!(appearance.text(), TextScale::ordinary());
        assert_eq!(appearance.scheme_at(at(12)), Scheme::Dark, "untouched");
        assert_eq!(
            appearance.background_on(&laptop()),
            Background::from(Token::Navy.colour()),
            "untouched"
        );

        appearance.put_everything_back();
        assert!(appearance.changes().is_untouched());
        assert_eq!(appearance.background_on(&laptop()), the_shipped_surface());
        assert_eq!(appearance.scheme_at(at(12)), Scheme::Light);
    }

    /// **A better default reaches a machine that never touched it.** The
    /// changes are the whole file, so a release that ships a different
    /// wallpaper and schedule reaches a person who set only the text size.
    #[test]
    fn a_new_release_reaches_what_the_person_never_touched() {
        let mut theirs = Appearance::shipped();
        theirs.set_text(TextScale::percent(125).unwrap());
        let written = serde_json::to_string(theirs.changes()).unwrap();

        let next_release = Shipped::of(
            Background::from(Token::Cream.colour()),
            Lock::TheDesktop,
            Following::from(Shipped::the_evening_schedule()),
            TextScale::ordinary(),
            Accent::Moss,
        );
        let after =
            Appearance::over(next_release).with(serde_json::from_str::<Changes>(&written).unwrap());

        assert_eq!(
            after.background_on(&laptop()),
            Background::from(Token::Cream.colour()),
            "the new wallpaper reaches them"
        );
        assert_eq!(
            after.scheme_at(at(20)),
            Scheme::Dark,
            "and the new schedule"
        );
        assert_eq!(
            after.text(),
            TextScale::percent(125).unwrap(),
            "and what they chose is untouched"
        );
        assert_eq!(
            after.accent(),
            Accent::Moss,
            "and a release that changes the accent reaches somebody who never picked one"
        );
    }

    /// The shipped appearance is readable, so a settings panel can mark what is
    /// the person's own and offer to put it back.
    #[test]
    fn what_ships_is_readable_beside_what_was_changed() {
        let appearance = Appearance::shipped();
        assert_eq!(appearance.shipped_appearance(), &Shipped::of_the_image());
        assert_eq!(appearance.following(), Following::from(Scheme::Light));
    }
}
