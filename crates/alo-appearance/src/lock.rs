//! What is on the screen when nobody is signed in.
//!
//! The lock screen follows the desktop unless a person says otherwise, because
//! two settings that have to be kept in step by hand are two settings that drift
//! apart.
//!
//! **But it does not follow a rotating folder, and that is the decision this
//! file exists for.** The desktop is seen by whoever is signed in; the lock
//! screen is seen by whoever walks past. Those are different audiences, and a
//! person who pointed the background at a folder of their own photographs picked
//! the *folder* — they did not pick, one by one, the pictures that a machine
//! left alone in a room will show to a corridor. So when the desktop rotates,
//! following it means the wallpaper alo OS shipped instead.
//!
//! Nothing is taken away by that: [`Lock::Its`] takes any background at all,
//! including a rotating folder, so a person who wants their photographs on the
//! lock screen says so once and gets them. The rule only decides what *following*
//! means, which is the case where nobody said anything.

use serde::{Deserialize, Serialize};

use crate::background::Background;

/// What the lock screen shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lock {
    /// Whatever the desktop shows — unless the desktop rotates, in which case
    /// the wallpaper alo OS shipped. [`crate::appearance::Appearance::lock_on`]
    /// is where that is resolved, and the reasoning is at the top of this file.
    TheDesktop,
    /// Its own background, whatever the desktop is doing.
    Its(Background),
}

impl Lock {
    /// Whether the person has chosen a lock screen of their own, which is what
    /// a settings panel marks and what *put it back* undoes.
    #[must_use]
    pub fn is_its_own(&self) -> bool {
        matches!(self, Self::Its(_))
    }
}

impl From<Background> for Lock {
    fn from(background: Background) -> Self {
        Self::Its(background)
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
    use crate::rotating::{Every, Rotating};
    use std::path::PathBuf;

    /// A rotating folder, on a path no machine has.
    fn rotating() -> Rotating {
        let folder = if cfg!(windows) {
            PathBuf::from(r"C:\Users\a\Pictures\Rotating")
        } else {
            PathBuf::from("/home/a/Pictures/Rotating")
        };
        Rotating::folder(folder, Every::hours(1).unwrap()).unwrap()
    }

    /// Following is the default, and choosing is a thing a panel can see.
    #[test]
    fn a_lock_screen_follows_or_is_its_own() {
        assert!(!Lock::TheDesktop.is_its_own());
        let mine = Lock::from(Background::from(Picture::shipped("alo").unwrap()));
        assert!(mine.is_its_own());
    }

    /// **A person may put their photographs on the lock screen** — the rule
    /// about rotating folders governs what *following* means, and takes nothing
    /// away from somebody who says what they want.
    #[test]
    fn a_person_may_choose_a_rotating_lock_screen() {
        let mine = Lock::from(Background::from(rotating()));
        assert!(mine.is_its_own());
        assert_eq!(mine, Lock::Its(Background::from(rotating())));
    }

    /// Both kinds survive a settings file unchanged.
    #[test]
    fn a_lock_screen_survives_being_written_down() {
        let each = [
            Lock::TheDesktop,
            Lock::from(Background::from(Picture::shipped("alo").unwrap())),
        ];
        for lock in each {
            let written = serde_json::to_string(&lock).unwrap();
            assert_eq!(serde_json::from_str::<Lock>(&written).unwrap(), lock);
        }
    }
}
