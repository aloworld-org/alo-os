//! **Clicking once at one point of an application's window**, for one approval
//! — `accessible.click_at`, the screen road's click
//! ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md),
//! amended 2026-11-17: *"Allow clicking by position"*). Dragging stays the
//! person's: nothing here can hold a button.
//!
//! What is decided before anything is clicked, in this order, is the picture's
//! (`crate::picturing_a_window`) with the click's own words:
//!
//! 1. [`ClickingAt::of`] — the grants asked **now**, the application installed,
//!    no adapter of its own; *Pictures and keys* on for **this** application;
//!    and somebody chosen to read pictures, because after every click the
//!    window is pictured and read again, and a click nothing can look at
//!    afterwards is not one alo makes.
//! 2. [`ClickingAt::may_be_clicked`] — the windows walked **then**: a password
//!    field anywhere means no click, and a window that describes itself is
//!    pressed by its names instead ([`crate::a_picture_may_be_taken`]).
//!
//! **The point is a proportion of the window, never pixels**: two whole numbers
//! from 0 to 100, how far across from its left edge and how far down from its
//! top, as the reader of its picture said them.

use alo_applications::Installed;
use alo_capability::{Authorised, Grants, PicturesAndKeys, Refused, Value, WhoReadsPictures};
use alo_strings::{Filling, Strings, Word};

use crate::a_picture_may_be_taken::{NoPicture, a_picture_may_be_taken};
use crate::accessibility_tree::AccessibilityTree;
use crate::adapters::Adapters;
use crate::click_words as words;
use crate::fallback_reach::reached;
use crate::fallback_verbs::{ACROSS, CLICK_AT, DOWN, THE_WHOLE_WAY};
use crate::fallback_words as fallback;
use crate::not_walked::NotWalked;
use crate::walking::{Whose, walk};

/// An approved click at one point that has not happened yet.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it.
#[derive(Debug)]
pub struct ClickingAt {
    /// What may run.
    authorised: Authorised,
    /// The application, as installed.
    application: String,
    /// How far across, in per cent.
    across: u8,
    /// How far down, in per cent.
    down: u8,
}

impl ClickingAt {
    /// The reach, the point, and the person's two choices, asked before any
    /// window is looked at.
    ///
    /// # Errors
    /// [`Refused`], in the words a person is told: the fallback's own refusals
    /// first, then *pictures and keys are off for it*, then *nobody reads
    /// pictures*.
    pub fn of(
        authorised: Authorised,
        adapters: &Adapters,
        grants: &Grants,
        installed: &Installed,
        pictures: &PicturesAndKeys,
        reading: &WhoReadsPictures,
        strings: &Strings,
    ) -> Result<Self, Refused> {
        let reached = reached(authorised, CLICK_AT, adapters, grants, installed, strings)?;
        let call = reached.authorised.call();
        let per_cent = |named: &str| match call.value(named) {
            Some(Value::Count(number)) => u8::try_from(*number)
                .ok()
                .filter(|number| *number <= THE_WHOLE_WAY),
            _ => None,
        };
        let (Some(across), Some(down)) = (per_cent(ACROSS), per_cent(DOWN)) else {
            let said = strings.say(
                &fallback::NOT_THIS_VERB.key(),
                &Filling::of(fallback::VERB, call.verb().to_owned()),
            );
            return Err(Refused::worded_elsewhere(call.clone(), said));
        };
        let clicking = Self {
            authorised: reached.authorised,
            application: reached.application,
            across,
            down,
        };
        if !pictures.allows(&clicking.application) {
            return Err(clicking.refused(words::NOT_SWITCHED_ON, strings));
        }
        if !reading.may_take_pictures() {
            return Err(clicking.refused(words::NO_ONE_READS, strings));
        }
        Ok(clicking)
    }

    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// How far across and down, in per cent of the window.
    #[must_use]
    pub const fn at(&self) -> (u8, u8) {
        (self.across, self.down)
    }

    /// This click refused in these words, with the application filled in.
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
    /// clicked by position: never with a password field, never when they
    /// describe themselves. An application the tree knows no window of
    /// describes nothing, as for a picture.
    ///
    /// # Errors
    /// [`Refused`], in the words a person is told; and when the tree could not
    /// be read at all, because then nothing can say there is no password field.
    pub fn may_be_clicked(
        &self,
        tree: &dyn AccessibilityTree,
        strings: &Strings,
    ) -> Result<(), Refused> {
        let shown = match walk(tree, Whose::Application(&self.application), None) {
            Ok(walked) => walked.shown,
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
