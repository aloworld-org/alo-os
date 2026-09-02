//! What a person changed, which is the only part that is written down.
//!
//! The same shape as `alo-shortcuts`: the defaults are in the running release
//! and this is the difference, so a release can improve a wallpaper or a
//! schedule for every machine that never touched it and no machine that did.
//! A file written by an older release still says exactly what its owner decided,
//! however much has moved since.
//!
//! **One background, and per-display exceptions to it.** The alternative — a
//! background *per display and nothing else* — reads the same until somebody
//! plugs in a projector, at which point a machine whose owner chose a photograph
//! shows a stranger's meeting room the default wallpaper, because the projector
//! is a display nobody has chosen for. So the person's choice is the machine's,
//! and a display they have singled out is an exception they made on purpose.
//! A display that is renamed by a driver update loses its exception and falls
//! back to their choice, which is the right way round to fail.

use serde::{Deserialize, Serialize};

use crate::background::Background;
use crate::display::DisplayId;
use crate::lock::Lock;
use crate::scheme::Following;
use crate::text::TextScale;

/// One thing a person can change, for a settings panel that offers *put it
/// back*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Setting {
    /// The background, everywhere. Per-display exceptions are their own, and
    /// [`Changes::forget_display`] is how one of those goes back.
    Background,
    /// What the lock screen shows.
    Lock,
    /// What decides light and dark.
    Following,
    /// How big the text is.
    Text,
}

/// Everything a person has changed about how their machine looks.
///
/// This is what a settings file holds and nothing else: an untouched machine
/// writes an empty one.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Written", into = "Written")]
pub struct Changes {
    /// What they put behind their windows, on every display they have not
    /// singled out.
    background: Option<Background>,
    /// The displays they singled out, oldest first, one entry each.
    displays: Vec<(DisplayId, Background)>,
    /// What they put on the lock screen.
    lock: Option<Lock>,
    /// What they told light and dark to follow.
    following: Option<Following>,
    /// How big they made the text.
    text: Option<TextScale>,
}

impl Changes {
    /// Nothing changed yet.
    #[must_use]
    pub fn untouched() -> Self {
        Self::default()
    }

    /// Whether nothing has been changed at all, which is what a fresh machine
    /// writes to its settings file.
    #[must_use]
    pub fn is_untouched(&self) -> bool {
        self.background.is_none()
            && self.displays.is_empty()
            && self.lock.is_none()
            && self.following.is_none()
            && self.text.is_none()
    }

    /// Put this behind the windows, on every display they have not singled out.
    pub fn set_background(&mut self, background: Background) {
        self.background = Some(background);
    }

    /// Put this behind the windows on one display only.
    ///
    /// Replaces any earlier choice for the same display: two rows for one
    /// display would be a file that disagrees with itself.
    pub fn set_background_on(&mut self, display: DisplayId, background: Background) {
        self.displays.retain(|(named, _)| *named != display);
        self.displays.push((display, background));
    }

    /// Put this on the lock screen.
    pub fn set_lock(&mut self, lock: Lock) {
        self.lock = Some(lock);
    }

    /// Follow this for light and dark.
    pub fn follow(&mut self, following: Following) {
        self.following = Some(following);
    }

    /// Make the text this size.
    pub fn set_text(&mut self, text: TextScale) {
        self.text = Some(text);
    }

    /// Forget that this was ever changed, which puts it back to what the
    /// running release ships.
    ///
    /// Says whether there was anything to forget.
    pub fn forget(&mut self, setting: Setting) -> bool {
        match setting {
            Setting::Background => self.background.take().is_some(),
            Setting::Lock => self.lock.take().is_some(),
            Setting::Following => self.following.take().is_some(),
            Setting::Text => self.text.take().is_some(),
        }
    }

    /// Forget that this display was ever singled out, which puts it back to the
    /// background the person chose for everywhere.
    ///
    /// Says whether there was anything to forget.
    pub fn forget_display(&mut self, display: &DisplayId) -> bool {
        let before = self.displays.len();
        self.displays.retain(|(named, _)| named != display);
        self.displays.len() != before
    }

    /// Forget everything, putting the machine back to what it shipped as.
    pub fn forget_everything(&mut self) {
        *self = Self::untouched();
    }

    /// What they put behind their windows, if they changed it.
    #[must_use]
    pub fn background(&self) -> Option<&Background> {
        self.background.as_ref()
    }

    /// What they put behind their windows on this display, if they singled it
    /// out.
    #[must_use]
    pub fn background_on(&self, display: &DisplayId) -> Option<&Background> {
        self.displays
            .iter()
            .find(|(named, _)| named == display)
            .map(|(_, background)| background)
    }

    /// Every display they singled out, oldest first.
    pub fn displays(&self) -> impl Iterator<Item = (&DisplayId, &Background)> {
        self.displays
            .iter()
            .map(|(display, background)| (display, background))
    }

    /// What they put on the lock screen, if they changed it.
    #[must_use]
    pub fn lock(&self) -> Option<&Lock> {
        self.lock.as_ref()
    }

    /// What they told light and dark to follow, if they changed it.
    #[must_use]
    pub fn following(&self) -> Option<Following> {
        self.following
    }

    /// How big they made the text, if they changed it.
    #[must_use]
    pub fn text(&self) -> Option<TextScale> {
        self.text
    }
}

/// Changes as a settings file holds them: everything untouched is absent rather
/// than present and null, so an untouched machine writes `{}`.
#[derive(Default, Serialize, Deserialize)]
struct Written {
    /// The background, everywhere.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    background: Option<Background>,
    /// The displays singled out.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    displays: Vec<(DisplayId, Background)>,
    /// The lock screen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lock: Option<Lock>,
    /// What light and dark follow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    following: Option<Following>,
    /// How big the text is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    text: Option<TextScale>,
}

impl From<Written> for Changes {
    /// Normalising, because a file is a thing a person can edit: a display named
    /// twice means what the file says last, which is the only reading that does
    /// not throw one of the two away at random.
    fn from(written: Written) -> Self {
        let mut changes = Self {
            background: written.background,
            displays: Vec::new(),
            lock: written.lock,
            following: written.following,
            text: written.text,
        };
        for (display, background) in written.displays {
            changes.set_background_on(display, background);
        }
        changes
    }
}

impl From<Changes> for Written {
    fn from(changes: Changes) -> Self {
        Self {
            background: changes.background,
            displays: changes.displays,
            lock: changes.lock,
            following: changes.following,
            text: changes.text,
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
    use crate::picture::Picture;
    use crate::scheme::Scheme;
    use crate::token::Token;

    /// The laptop's own screen.
    fn laptop() -> DisplayId {
        DisplayId::named("eDP-1").unwrap()
    }

    /// The screen on the desk.
    fn desk() -> DisplayId {
        DisplayId::named("DP-1").unwrap()
    }

    /// A background that is nothing but a colour.
    fn plain(token: Token) -> Background {
        Background::from(token.colour())
    }

    /// Each setting is recorded, replaced and forgotten, and *was it changed*
    /// is answerable for every one of them.
    #[test]
    fn a_change_is_recorded_replaced_and_forgotten() {
        let mut changes = Changes::untouched();
        assert!(changes.is_untouched());

        changes.set_background(plain(Token::Navy));
        changes.set_background(plain(Token::Charcoal));
        assert_eq!(changes.background(), Some(&plain(Token::Charcoal)));
        changes.set_lock(Lock::from(plain(Token::Cream)));
        changes.follow(Following::from(Scheme::Dark));
        changes.set_text(TextScale::percent(125).unwrap());
        assert!(!changes.is_untouched());

        for setting in [
            Setting::Background,
            Setting::Lock,
            Setting::Following,
            Setting::Text,
        ] {
            assert!(changes.forget(setting), "{setting:?} was there to forget");
            assert!(!changes.forget(setting), "and is not there twice");
        }
        assert!(changes.is_untouched());
    }

    /// **A display is an exception to the person's choice**, so a display
    /// nobody singled out has no row of its own and falls back to it.
    #[test]
    fn a_display_is_an_exception_and_can_be_put_back() {
        let mut changes = Changes::untouched();
        changes.set_background(plain(Token::Navy));
        changes.set_background_on(desk(), plain(Token::Charcoal));

        assert_eq!(
            changes.background_on(&desk()),
            Some(&plain(Token::Charcoal))
        );
        assert_eq!(changes.background_on(&laptop()), None);
        assert_eq!(changes.displays().count(), 1);

        changes.set_background_on(desk(), plain(Token::Porcelain));
        assert_eq!(changes.displays().count(), 1, "one display, one row");
        assert!(changes.forget_display(&desk()));
        assert!(!changes.forget_display(&desk()));
        assert_eq!(changes.background_on(&desk()), None);
        assert_eq!(
            changes.background(),
            Some(&plain(Token::Navy)),
            "the choice for everywhere is untouched by putting one display back"
        );
    }

    /// **An untouched machine writes nothing.** The file holds the difference,
    /// so a fresh machine's settings are an empty object rather than a copy of
    /// everything the release ships.
    #[test]
    fn the_file_holds_only_what_was_changed() {
        assert_eq!(serde_json::to_string(&Changes::untouched()).unwrap(), "{}");

        let mut changes = Changes::untouched();
        changes.set_text(TextScale::percent(150).unwrap());
        assert_eq!(serde_json::to_string(&changes).unwrap(), r#"{"text":150}"#);

        changes.set_background(Background::from(Picture::shipped("harbour").unwrap()));
        changes.set_background_on(laptop(), plain(Token::Navy));
        changes.set_lock(Lock::TheDesktop);
        changes.follow(Following::from(Scheme::Dark));
        let written = serde_json::to_string(&changes).unwrap();
        assert_eq!(serde_json::from_str::<Changes>(&written).unwrap(), changes);
    }

    /// A hand-edited file that names one display twice means what it says last,
    /// rather than holding two rows one of which never applies.
    #[test]
    fn a_file_that_repeats_a_display_means_what_it_says_last() {
        let read: Changes = serde_json::from_str(
            r##"{"displays":[["DP-1",{"Colour":"#102A43"}],["DP-1",{"Colour":"#1F2529"}]]}"##,
        )
        .unwrap();
        assert_eq!(read.displays().count(), 1);
        assert_eq!(read.background_on(&desk()), Some(&plain(Token::Charcoal)));
    }

    /// Forgetting everything is one call, and it is the same as never having
    /// touched anything.
    #[test]
    fn everything_can_be_put_back_at_once() {
        let mut changes = Changes::untouched();
        changes.set_background(plain(Token::Navy));
        changes.set_background_on(desk(), plain(Token::Cream));
        changes.set_text(TextScale::percent(200).unwrap());

        changes.forget_everything();
        assert!(changes.is_untouched());
        assert_eq!(changes, Changes::untouched());
    }
}
