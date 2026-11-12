//! **The *Open with* sheet, as a screen reader is told about it** — frame
//! `25:52`, approved by the owner on 2026-11-12
//! (`docs/design/default-apps-and-open-with.md`).
//!
//! Until now a reader was told nothing when an application asked the person
//! to choose what opens a link: the sheet was drawn and held every key, and
//! the tree said only *the desktop*.
//!
//! # What is read, in order
//!
//! - a **dialogue** named *Open with*, the sheet's title;
//! - a **label**: *{application} asked to open {address}*, the line under it;
//! - a **list** of the applications offered, each **one of a few**, named by
//!   its own name with the line drawn under it — *Your default for web links*
//!   or *Installed* — as its description; the one the arrows are on is
//!   **selected** and has the keyboard, which is the person's default when the
//!   sheet opens, as the frame's note says and the drawing rings it;
//! - **Cancel**, then **Open**, the order they are drawn in.
//!
//! **Every word is one the sheet draws** (`crate::the_open_with_sheets_words`),
//! and an application is called what the drawing calls it
//! ([`crate::Server::what_an_application_is_called`]), so what is heard and
//! what is seen cannot drift (ADR 0089).
//!
//! # A reader presses what a click lands on
//!
//! Each row, *Cancel* and *Open* carry a press, numbered with the sheet it was
//! heard on, and the sheet answers it through the door a click takes
//! (`crate::the_open_with_sheet::TheOpenWithSheet::a_reader_pressed`).
//! The list itself is a box and is not pressed.

use alo_applications::LinkKind;
use alo_strings::{Filling, Strings, Word};

use crate::access_action::{OnTheOpenWithSheet, Pressed};
use crate::access_nodes::{Node, ReadAloudTree};
use crate::access_roles;
use crate::access_sheet_in_front::{ASheetInFront, TheSheet};
use crate::the_open_with_sheet::SheetShows;
use crate::the_open_with_sheets_words as words;

impl ReadAloudTree {
    /// The same tree, with the *Open with* sheet in front while one is up.
    /// `shows` is [`None`] when none is, and the tree is left as it was.
    /// `called` is what each application is called, by its identifier.
    #[must_use]
    pub(crate) fn with_the_open_with_sheet(
        mut self,
        strings: &Strings,
        shows: Option<SheetShows<'_>>,
        called: &dyn Fn(&str) -> String,
    ) -> Self {
        let Some(shows) = shows else {
            return self;
        };
        let pressed = |on| {
            Some(Pressed::OnTheOpenWithSheet {
                sheet: shows.number,
                on,
            })
        };
        let sheet = self.push(
            0,
            Node {
                role: access_roles::number_of(alo_access::Role::Dialogue),
                name: said(strings, words::OPEN_WITH, &Filling::nothing()),
                states: access_roles::in_front(access_roles::words_of(
                    alo_access::State::ReadOnly,
                    None,
                )),
                ..Node::default()
            },
        );
        let asking = called(shows.asking.identifier());
        self.push(
            sheet,
            Node {
                role: access_roles::number_of(alo_access::Role::Label),
                name: said(
                    strings,
                    words::ASKED_TO_OPEN,
                    &Filling::of("application", &asking).and("address", shows.link.as_str()),
                ),
                states: access_roles::words_of(alo_access::State::ReadOnly, None),
                ..Node::default()
            },
        );
        let list = self.push(
            sheet,
            Node {
                role: access_roles::number_of(alo_access::Role::List),
                // **No name**: the frame draws no heading over the list, and a
                // name written here would be a word nobody decided.
                states: access_roles::words_of(alo_access::State::ReadOnly, None),
                ..Node::default()
            },
        );
        let the_default = said(
            strings,
            match shows.link.kind() {
                LinkKind::Web => words::YOUR_DEFAULT_FOR_WEB_LINKS,
                LinkKind::Mail => words::YOUR_DEFAULT_FOR_EMAIL_ADDRESSES,
            },
            &Filling::nothing(),
        );
        let installed = said(strings, words::INSTALLED, &Filling::nothing());
        let mut keyboard = list;
        for (at, application) in shows.offered.iter().enumerate() {
            let on = at == shows.focus;
            let states = access_roles::words_of(alo_access::State::OneOfAFew, Some(on));
            let row = self.push(
                list,
                Node {
                    role: access_roles::number_of(alo_access::Role::ListItem),
                    name: called(application.identifier()),
                    description: if at == 0 {
                        the_default.clone()
                    } else {
                        installed.clone()
                    },
                    states: if on {
                        access_roles::with_the_keyboard(states)
                    } else {
                        states
                    },
                    press: pressed(OnTheOpenWithSheet::TheApplication(at)),
                    ..Node::default()
                },
            );
            if on {
                keyboard = row;
            }
        }
        for (word, on) in [
            (words::CANCEL, OnTheOpenWithSheet::Cancel),
            (words::OPEN, OnTheOpenWithSheet::Open),
        ] {
            self.push(
                sheet,
                Node {
                    role: access_roles::number_of(alo_access::Role::Button),
                    name: said(strings, word, &Filling::nothing()),
                    states: access_roles::words_of(alo_access::State::CanBeUsed, None),
                    press: pressed(on),
                    ..Node::default()
                },
            );
        }
        self.put_in_front(ASheetInFront {
            which: TheSheet::OpenWith(shows.number),
            at: sheet,
            keyboard,
        });
        self
    }
}

impl crate::Server {
    /// **What an application is called on alo's own surfaces**: the name its
    /// entry gives, or its identifier where it gives none. Asked by the
    /// *Open with* sheet's drawing and by its tree, so a reader hears the name
    /// a sighted person reads.
    #[must_use]
    pub(crate) fn what_an_application_is_called(&self, identifier: &str) -> String {
        self.what_each_is_called
            .get(identifier)
            .filter(|called| !called.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| identifier.to_owned())
    }
}

/// One of the sheet's words, filled, in the person's language.
fn said(strings: &Strings, word: Word, filling: &Filling) -> String {
    strings.say(&word.key(), filling).into_text()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_open_with_sheet_as_read_tests.rs"]
mod tests;
