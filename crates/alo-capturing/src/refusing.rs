//! Why no picture was taken, or none was kept.
//!
//! Nine answers, and two of them are somebody else's carried whole. The plan
//! puts two of the seven that are this crate's own into its acceptance — **a
//! window from another person's session, or the lock screen, cannot be
//! captured** — and the shape of this type is what makes those two facts rather
//! than intentions: they are refusals returned by [`crate::Screenshot::of`],
//! which is the constructor, so a screenshot of somebody else's window is not a
//! value that exists and then declines to be taken. It is a value that cannot
//! be built.
//!
//! # Two refusals are carried whole rather than reworded
//!
//! [`NotTaken::NotGrabbed`] is the rented mechanism's own, and
//! [`NotTaken::NotAllowed`] is `alo-portals`' — which is `alo-capability`'s
//! under that. Both keep their own sentence. An application refused a
//! screenshot reads the same words as an application refused a camera, because
//! the thing that refused it is the same thing, and a second wording here would
//! be this crate having an opinion about grants that it is not allowed to have.
//!
//! # Nothing here says what was on the screen
//!
//! No refusal carries a window's title, a file's contents, another person's
//! name, or anything read from the picture. Three of them carry a diagnosis for
//! whoever is fixing the machine, in English, which never reaches a person —
//! the same shape `alo_in_use::NotHeard` uses and for the same reason.

use alo_portals::Refused;
use alo_strings::{Filling, Said, Strings};

use crate::grabs::NotGrabbed;
use crate::words::{self, Word};

/// Why a picture of the screen was not taken, or was not kept.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotTaken {
    /// The lock screen is up, so the screen belongs to nobody yet.
    #[error("the lock screen cannot be captured")]
    TheLockScreen,
    /// The window belongs to somebody else signed in on this machine.
    ///
    /// Carries nothing: being refused a window that is not yours must not tell
    /// you whose it is.
    #[error("that window is in another person's session")]
    AnotherPersonsWindow,
    /// The rectangle asked for has no width or no height, or is not on the
    /// screen.
    #[error("that is not an area of the screen")]
    NotAnAreaOfTheScreen,
    /// The folder chosen to keep pictures in cannot hold them.
    #[error("that is not a folder pictures can be kept in")]
    NotAFolderForPictures,
    /// Every name the moment could make is already a file in that folder.
    #[error("that folder holds a picture under every name this moment can make")]
    NoRoomForAName,
    /// The file could not be written in the folder the person chose.
    #[error("the picture could not be written: {said}")]
    NotWritten {
        /// What the disk said. English, for whoever is fixing it, and never
        /// shown to a person.
        said: String,
    },
    /// The picture was taken and could not be put on the clipboard.
    ///
    /// alo OS's own bug rather than anything the person did. It is a refusal
    /// rather than an unwrap for the reason every list in this workspace is:
    /// a library that panicked over its own table would take the shell down
    /// with it, and a person whose picture did not arrive is better served by
    /// a sentence than by a machine that stopped.
    #[error("the picture could not be put on the clipboard: {said}")]
    NotCopied {
        /// What was wrong. English, for whoever is fixing it, and never shown
        /// to a person.
        said: String,
    },
    /// The rented screen-capture mechanism did not hand a picture back.
    #[error(transparent)]
    NotGrabbed(#[from] NotGrabbed),
    /// An application asked through the portal and its grant does not cover
    /// this.
    #[error("the application that asked is not allowed a picture of the screen")]
    NotAllowed(Refused),
}

impl NotTaken {
    /// The string this crate declares for it, where it has one.
    ///
    /// [`None`] for [`NotTaken::NotAllowed`], whose sentence is
    /// `alo-capability`'s: what a person reads when an application is refused a
    /// screenshot is the same sentence they read when it is refused a camera,
    /// and this crate does not write a second one.
    #[must_use]
    pub const fn word(&self) -> Option<Word> {
        match self {
            Self::TheLockScreen => Some(words::THE_LOCK_SCREEN),
            Self::AnotherPersonsWindow => Some(words::ANOTHER_PERSONS_WINDOW),
            Self::NotAnAreaOfTheScreen => Some(words::NOT_AN_AREA_OF_THE_SCREEN),
            Self::NotAFolderForPictures => Some(words::NOT_A_FOLDER_FOR_PICTURES),
            Self::NoRoomForAName => Some(words::NO_ROOM_FOR_A_NAME),
            Self::NotWritten { .. } => Some(words::NOT_WRITTEN),
            Self::NotCopied { .. } => Some(words::NOT_COPIED),
            Self::NotGrabbed(why) => Some(why.word()),
            Self::NotAllowed(_) => None,
        }
    }

    /// What was said about it in English, for whoever is fixing it, where
    /// anything was.
    ///
    /// Never put in front of a person: what a person reads is [`NotTaken::said`]
    /// and it is answered in their own language.
    #[must_use]
    pub fn diagnosis(&self) -> Option<&str> {
        match self {
            Self::NotWritten { said } | Self::NotCopied { said } => Some(said),
            Self::NotGrabbed(why) => Some(why.diagnosis()),
            Self::TheLockScreen
            | Self::AnotherPersonsWindow
            | Self::NotAnAreaOfTheScreen
            | Self::NotAFolderForPictures
            | Self::NoRoomForAName
            | Self::NotAllowed(_) => None,
        }
    }

    /// Why an application was refused, where one was.
    #[must_use]
    pub const fn refused(&self) -> Option<&Refused> {
        match self {
            Self::NotAllowed(why) => Some(why),
            Self::TheLockScreen
            | Self::AnotherPersonsWindow
            | Self::NotAnAreaOfTheScreen
            | Self::NotAFolderForPictures
            | Self::NoRoomForAName
            | Self::NotWritten { .. }
            | Self::NotCopied { .. }
            | Self::NotGrabbed(_) => None,
        }
    }

    /// What to tell the person, in the language they read.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::capturing_words`] answers with
    /// the key, marked `Said::is_a_bug`, which is the honest answer to *the
    /// shell forgot to declare what this crate can say*.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotAllowed(why) => why.said(strings),
            Self::TheLockScreen
            | Self::AnotherPersonsWindow
            | Self::NotAnAreaOfTheScreen
            | Self::NotAFolderForPictures
            | Self::NoRoomForAName
            | Self::NotWritten { .. }
            | Self::NotCopied { .. }
            | Self::NotGrabbed(_) => match self.word() {
                Some(word) => strings.say(&word.key(), &Filling::nothing()),
                // Unreachable by the match above, and written as an answer
                // rather than as a panic: a library that took the shell down
                // over its own string table would be a worse bug than a
                // sentence nobody declared.
                None => strings.say(&words::NOT_WRITTEN.key(), &Filling::nothing()),
            },
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
    use crate::testing::{in_english, refused_a_screenshot, translated};
    use std::collections::BTreeSet;

    /// Every way a picture does not happen, so no test below quietly skips one.
    fn every_refusal() -> [NotTaken; 9] {
        [
            NotTaken::TheLockScreen,
            NotTaken::AnotherPersonsWindow,
            NotTaken::NotAnAreaOfTheScreen,
            NotTaken::NotAFolderForPictures,
            NotTaken::NoRoomForAName,
            NotTaken::NotWritten {
                said: "there is no room on the disk".to_owned(),
            },
            NotTaken::NotCopied {
                said: "the offer this crate makes is not an offer".to_owned(),
            },
            NotTaken::NotGrabbed(NotGrabbed::NoPicture {
                said: "no such node".to_owned(),
            }),
            NotTaken::NotAllowed(refused_a_screenshot()),
        ]
    }

    /// **Every refusal says something a person could read**, and no two read
    /// the same. Somebody told the same sentence for a lock screen and for a
    /// full disk would go and do the wrong thing.
    #[test]
    fn every_refusal_reads_and_no_two_read_the_same() {
        let strings = in_english();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for refusal in every_refusal() {
            let said = refusal.said(&strings);
            assert!(!said.text().is_empty(), "{refusal:?} says nothing");
            assert!(!said.is_a_bug(), "{refusal:?} is not declared");
            assert!(
                seen.insert(said.text().to_owned()),
                "two refusals both say {said}"
            );
        }
    }

    /// **The application's refusal is the grants' own sentence**, not a second
    /// one written here: an application refused a picture of the screen reads
    /// what it reads when it is refused a camera.
    #[test]
    fn an_application_is_refused_in_the_grants_own_words() {
        let strings = in_english();
        let why = refused_a_screenshot();
        let refusal = NotTaken::NotAllowed(why.clone());
        assert_eq!(refusal.word(), None);
        assert_eq!(refusal.refused(), Some(&why));
        assert_eq!(refusal.said(&strings).text(), why.said(&strings).text());
    }

    /// **Being refused another person's window tells you nothing about them.**
    /// The variant carries no field, so there is nothing to leak, and the
    /// sentence has no gap to put a name in.
    #[test]
    fn another_persons_window_carries_nothing_about_them() {
        let strings = in_english();
        let refusal = NotTaken::AnotherPersonsWindow;
        assert_eq!(refusal.diagnosis(), None);
        assert!(!refusal.said(&strings).text().is_empty());
        assert!(
            words::ANOTHER_PERSONS_WINDOW
                .phrase()
                .unwrap()
                .source()
                .gaps()
                .is_empty()
        );
    }

    /// **What the machine said is kept for whoever is fixing it**, and is never
    /// in the sentence a person reads.
    #[test]
    fn what_the_machine_said_is_kept_and_is_not_the_sentence() {
        let strings = in_english();
        for refusal in every_refusal() {
            let Some(diagnosis) = refusal.diagnosis() else {
                continue;
            };
            assert!(!diagnosis.is_empty(), "{refusal:?}");
            assert!(
                !refusal.said(&strings).text().contains(diagnosis),
                "{refusal:?} puts a diagnostic in front of a person"
            );
        }
    }

    /// **The mechanism's refusal keeps its own sentence**, carried through
    /// rather than reworded, so there is one wording of *the part that reads
    /// the screen is not running* in the whole machine.
    #[test]
    fn the_mechanisms_refusal_keeps_its_own_sentence() {
        let strings = in_english();
        let why = NotGrabbed::NothingReadsTheScreen {
            said: "not on this machine".to_owned(),
        };
        let refusal = NotTaken::from(why.clone());
        assert_eq!(refusal.word(), Some(why.word()));
        assert_eq!(refusal.said(&strings).text(), why.said(&strings).text());
    }

    /// **A refusal arrives in the language the person reads** when somebody has
    /// translated it, and says so.
    #[test]
    fn a_refusal_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::THE_LOCK_SCREEN,
            "Vom Sperrbildschirm kann kein Bild aufgenommen werden. Melden Sie sich an, und \
             nehmen Sie das Bild dann auf",
        )]);
        let said = NotTaken::TheLockScreen.said(&strings);
        assert!(said.is_translated());
        assert!(said.text().starts_with("Vom Sperrbildschirm"));

        // The one nobody translated is still English, and says it is.
        let untranslated = NotTaken::NoRoomForAName.said(&strings);
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug());
    }
}
