//! **The sheet alo asks for an application on, as a screen reader is told
//! about it** (`docs/design/when-alo-asks-for-an-application.md`).
//!
//! # What is read, in order
//!
//! - a **dialogue** named *Let alo use {application}?*, the heading;
//! - a **label**: *alo asked to use {application}. Choose how long.*;
//! - a **list** of the three lengths, each **one of a few**, named *This
//!   task*, *This session* or *Remember* with the line drawn under it as its
//!   description; the one the arrows are on is **selected** and has the
//!   keyboard, which is *This task* when the sheet opens;
//! - a **label**: the note naming Escape;
//! - **Not now**, then **Let alo use {application}**, the order they are drawn.
//!
//! **Every word is one the sheet draws** (`crate::the_application_sheets_words`),
//! so what is heard and what is seen cannot drift (ADR 0089).
//!
//! # A reader presses what a click lands on
//!
//! Each length and button carries a press numbered with the sheet it was heard
//! on, and the server answers it through the door a click takes
//! (`crate::the_application_sheet_on_the_server`). **A press on a length grants
//! nothing**, exactly as a click on one does.

use alo_strings::{Filling, Strings, Word};

use crate::access_action::Pressed;
use crate::access_nodes::{Node, ReadAloudTree};
use crate::access_roles;
use crate::access_sheet_in_front::{ASheetInFront, TheSheet};
use crate::the_application_sheet::{
    ApplicationSheetShows, HowLongForAnApplication, OnTheApplicationSheet,
};
use crate::the_application_sheets_words as words;

impl ReadAloudTree {
    /// The same tree, with the sheet alo asks for an application on in front
    /// while one is up. `shows` is [`None`] when none is, and the tree is left
    /// as it was.
    #[must_use]
    pub(crate) fn with_the_application_sheet(
        mut self,
        strings: &Strings,
        shows: Option<ApplicationSheetShows<'_>>,
    ) -> Self {
        let Some(shows) = shows else {
            return self;
        };
        let named = Filling::of(words::THE_APPLICATION, shows.wanted.called().to_owned());
        let said = |word: Word, filling: &Filling| strings.say(&word.key(), filling).into_text();
        let read_only = || access_roles::words_of(alo_access::State::ReadOnly, None);
        let pressed = |on| {
            Some(Pressed::OnTheApplicationSheet {
                sheet: shows.number,
                on,
            })
        };
        let sheet = self.push(
            0,
            Node {
                role: access_roles::number_of(alo_access::Role::Dialogue),
                name: said(words::LET_ALO_USE, &named),
                states: access_roles::in_front(read_only()),
                ..Node::default()
            },
        );
        self.push(
            sheet,
            Node {
                role: access_roles::number_of(alo_access::Role::Label),
                name: said(words::ALO_ASKED, &named),
                states: read_only(),
                ..Node::default()
            },
        );
        let list = self.push(
            sheet,
            Node {
                role: access_roles::number_of(alo_access::Role::List),
                // **No name**: the frame draws no heading over the lengths.
                states: read_only(),
                ..Node::default()
            },
        );
        let mut keyboard = list;
        for how_long in HowLongForAnApplication::ALL {
            let on = how_long == shows.focus;
            let (name, under) = crate::the_application_sheet_drawn::the_words_of(how_long);
            let states = access_roles::words_of(alo_access::State::OneOfAFew, Some(on));
            let row = self.push(
                list,
                Node {
                    role: access_roles::number_of(alo_access::Role::ListItem),
                    name: said(name, &Filling::nothing()),
                    description: said(under, &Filling::nothing()),
                    states: if on {
                        access_roles::with_the_keyboard(states)
                    } else {
                        states
                    },
                    press: pressed(OnTheApplicationSheet::For(how_long)),
                    ..Node::default()
                },
            );
            if on {
                keyboard = row;
            }
        }
        self.push(
            sheet,
            Node {
                role: access_roles::number_of(alo_access::Role::Label),
                name: said(words::ONLY_THIS_APPLICATION, &named),
                states: read_only(),
                ..Node::default()
            },
        );
        for (word, on) in [
            (words::NOT_NOW, OnTheApplicationSheet::NotNow),
            (words::LET_ALO_USE_IT, OnTheApplicationSheet::LetAloUseIt),
        ] {
            self.push(
                sheet,
                Node {
                    role: access_roles::number_of(alo_access::Role::Button),
                    name: said(word, &named),
                    states: access_roles::words_of(alo_access::State::CanBeUsed, None),
                    press: pressed(on),
                    ..Node::default()
                },
            );
        }
        self.put_in_front(ASheetInFront {
            which: TheSheet::Application(shows.number),
            at: sheet,
            keyboard,
        });
        self
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_application_sheet_as_read_tests.rs"]
mod tests;
