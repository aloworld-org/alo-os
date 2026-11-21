//! **Clicking once on one described thing in an application's window**, for
//! one approval — `accessible.click_at`, the screen road's click
//! ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md),
//! amended 2026-11-17: *"Allow clicking by position"*, and 2026-11-21: *"Yes,
//! click by description"*). Dragging stays the person's: nothing here can hold
//! a button.
//!
//! What is decided before anything is clicked, in this order, is the picture's
//! (`crate::picturing_a_window`) with the click's own words:
//!
//! 1. [`ClickingAt::of`] — the grants asked **now**, the application installed,
//!    no adapter of its own; *Pictures and keys* on for **this** application;
//!    and somebody chosen to read pictures, because the reader is who finds the
//!    thing, and who reads the window again after the click.
//! 2. [`ClickingAt::may_be_clicked`] — the windows walked **then**: a password
//!    field anywhere means no click, and a window that describes itself is
//!    pressed by its names instead ([`crate::a_picture_may_be_taken`]).
//!
//! **The model gives no point.** It describes the thing as a person sees it,
//! and the reader the person chose says where it is
//! (`crate::where_a_thing_is`): a proportion of the window, never pixels.

use alo_applications::Installed;
use alo_capability::{Authorised, Grants, PicturesAndKeys, Refused, Value, WhoReadsPictures};
use alo_strings::{Filling, Said, Strings, Word};

use crate::a_picture_may_be_taken::{NoPicture, a_picture_may_be_taken};
use crate::accessibility_tree::AccessibilityTree;
use crate::adapters::Adapters;
use crate::click_words as words;
use crate::fallback_reach::reached;
use crate::fallback_verbs::{CLICK_AT, DESCRIPTION};
use crate::fallback_words as fallback;
use crate::not_walked::NotWalked;
use crate::walking::{Whose, walk};

/// An approved click on one described thing that has not happened yet.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it.
#[derive(Debug)]
pub struct ClickingAt {
    /// What may run.
    authorised: Authorised,
    /// The application, as installed.
    application: String,
    /// The application, as a person calls it.
    named: String,
    /// The thing to click, as the call described it.
    description: String,
}

impl ClickingAt {
    /// The reach, the description, and the person's two choices, asked before
    /// any window is looked at. `called` says what a person calls an
    /// application, for every sentence they are told.
    ///
    /// # Errors
    /// [`Refused`], in the words a person is told: the fallback's own refusals
    /// first, then *pictures and keys are off for it*, then *nobody reads
    /// pictures*.
    #[expect(
        clippy::too_many_arguments,
        reason = "the authorisation, the three things it is asked against, the person's two \
                  choices and how a person names an application; a struct would exist only to \
                  be unpacked"
    )]
    pub fn of(
        authorised: Authorised,
        adapters: &Adapters,
        grants: &Grants,
        installed: &Installed,
        pictures: &PicturesAndKeys,
        reading: &WhoReadsPictures,
        called: &dyn Fn(&str) -> String,
        strings: &Strings,
    ) -> Result<Self, Refused> {
        let reached = reached(authorised, CLICK_AT, adapters, grants, installed, strings)?;
        let call = reached.authorised.call();
        let description = match call.value(DESCRIPTION) {
            Some(Value::Name(described) | Value::Text(described))
                if !described.trim().is_empty() =>
            {
                described.trim().to_owned()
            }
            _ => {
                let said = strings.say(
                    &fallback::NOT_THIS_VERB.key(),
                    &Filling::of(fallback::VERB, call.verb().to_owned()),
                );
                return Err(Refused::worded_elsewhere(call.clone(), said));
            }
        };
        let clicking = Self {
            named: called(&reached.application),
            authorised: reached.authorised,
            application: reached.application,
            description,
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

    /// The application, as a person calls it.
    #[must_use]
    pub fn named(&self) -> &str {
        &self.named
    }

    /// The thing to click, as the call described it.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// This click refused in these words, with the application — as a person
    /// calls it — and the description filled in.
    #[must_use]
    pub fn refused(&self, word: Word, strings: &Strings) -> Refused {
        self.refused_saying(strings.say(
            &word.key(),
            &Filling::of(fallback::APPLICATION, self.named.clone()).and(
                crate::where_a_thing_is::DESCRIPTION,
                self.description.clone(),
            ),
        ))
    }

    /// This click refused in a sentence already said.
    #[must_use]
    pub fn refused_saying(&self, said: Said) -> Refused {
        Refused::worded_elsewhere(self.authorised.call().clone(), said)
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
                return Err(self.refused_saying(why.said(&self.named, None, strings)));
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
