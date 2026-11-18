//! **Taking one picture of an application's window**, for one approval —
//! `accessible.picture_window`, the screen road's picture
//! ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md)
//! §1).
//!
//! This file holds what is decided before the picture is taken, in this order:
//!
//! 1. [`PicturingAWindow::of`] — the grants asked **now**, the application
//!    installed, no adapter of its own (the fallback's reach), then the
//!    person's two choices: *Pictures and keys* switched on for **this**
//!    application, and somebody chosen under *Who reads the pictures*. Both are
//!    read by the caller at the moment of the act and handed in, so this crate
//!    touches no disk.
//! 2. [`PicturingAWindow::may_be_pictured`] — the application's windows,
//!    walked **then**: a password field anywhere means no picture, and a window
//!    that describes itself is read rather than pictured
//!    ([`crate::a_picture_may_be_taken`]).
//!
//! The picture itself is the compositor's (`picture-of`,
//! `docs/contracts/application-door.md`): the window that holds the keyboard,
//! only if it is that application's, drawn alone.
//!
//! # An application the tree knows no window of
//!
//! **Is one that describes nothing**, and is not refused here. A game drawn by
//! SDL is often not on the accessibility bus at all, which is the very
//! application the screen road is for; refusing it as *no window* would make
//! the road reach nothing. Whether a window of it is on the screen and in front
//! is the compositor's to answer, and it answers *not in front* when none is.

use alo_applications::Installed;
use alo_capability::{Authorised, Grants, PicturesAndKeys, Refused, WhoReadsPictures};
use alo_strings::{Filling, Strings, Word};

use crate::a_picture_may_be_taken::{NoPicture, a_picture_may_be_taken};
use crate::accessibility_tree::AccessibilityTree;
use crate::adapters::Adapters;
use crate::fallback_reach::reached;
use crate::fallback_verbs::PICTURE_WINDOW;
use crate::fallback_words as fallback;
use crate::not_walked::NotWalked;
use crate::picture_words as words;
use crate::walking::{Whose, walk};

/// An approved picture of one application's window that has not been taken.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it.
#[derive(Debug)]
pub struct PicturingAWindow {
    /// What may run.
    authorised: Authorised,
    /// The application, as installed.
    application: String,
}

impl PicturingAWindow {
    /// The reach, and the person's two choices, asked before any window is
    /// looked at.
    ///
    /// `pictures` and `reading` are what the person's file says **now**.
    ///
    /// # Errors
    /// [`Refused`], in the words a person is told: the fallback's own refusals
    /// first, then *pictures are off for it*, then *nobody reads pictures*.
    pub fn of(
        authorised: Authorised,
        adapters: &Adapters,
        grants: &Grants,
        installed: &Installed,
        pictures: &PicturesAndKeys,
        reading: &WhoReadsPictures,
        strings: &Strings,
    ) -> Result<Self, Refused> {
        let reached = reached(
            authorised,
            PICTURE_WINDOW,
            adapters,
            grants,
            installed,
            strings,
        )?;
        let picturing = Self {
            authorised: reached.authorised,
            application: reached.application,
        };
        if !pictures.allows(&picturing.application) {
            return Err(picturing.refused(words::NOT_SWITCHED_ON, strings));
        }
        if !reading.may_take_pictures() {
            return Err(picturing.refused(words::NO_ONE_READS, strings));
        }
        Ok(picturing)
    }

    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// This picture refused in these words, with the application filled in.
    #[must_use]
    pub fn refused(&self, word: Word, strings: &Strings) -> Refused {
        Refused::worded_elsewhere(
            self.authorised.call().clone(),
            strings.say(
                &word.key(),
                &Filling::of(fallback::APPLICATION, self.application.clone()),
            ),
        )
    }

    /// Whether the application's windows, as the tree shows them now, may be
    /// pictured: never with a password field, never when they describe
    /// themselves.
    ///
    /// # Errors
    /// [`Refused`], in the words a person is told; and when the tree could not
    /// be read at all, because then nothing can say there is no password field.
    pub fn may_be_pictured(
        &self,
        tree: &dyn AccessibilityTree,
        strings: &Strings,
    ) -> Result<(), Refused> {
        let shown = match walk(tree, Whose::Application(&self.application), None) {
            Ok(walked) => walked.shown,
            // **Not on the bus, or no window there: it describes nothing**
            // (this file's header), and the compositor answers for the screen.
            Err(NotWalked::NoWindow) => return Ok(()),
            Err(why) => {
                return Err(Refused::worded_elsewhere(
                    self.authorised.call().clone(),
                    why.said(&self.application, None, strings),
                ));
            }
        };
        match a_picture_may_be_taken(&shown) {
            Ok(()) | Err(NoPicture::NoWindow) => Ok(()),
            Err(NoPicture::APasswordField) => Err(self.refused(words::A_PASSWORD_FIELD, strings)),
            Err(NoPicture::ItDescribesItself) => {
                Err(self.refused(words::IT_DESCRIBES_ITSELF, strings))
            }
        }
    }

    /// What ran, once it has.
    #[must_use]
    pub fn into_authorised(self) -> Authorised {
        self.authorised
    }
}
