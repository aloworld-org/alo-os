//! **The file chooser, as a screen reader is told about it**
//! (`docs/design/the-file-chooser.md`, *What a screen reader hears*).
//!
//! # What is read, in order
//!
//! - a **dialogue** named by the title — *Open a file* or *Save a file*;
//! - a **label**: *{application} asked to open a file* (or *to save a file*);
//! - a **label**: *In {folder}*;
//! - a **list** of **every** row, not only the page drawn — a reader walks the
//!   list, it does not look at a page — each named by its name with *The
//!   folder above*, *Folder* or *File* as its description; the one the
//!   keyboard is on is **selected** and has the keyboard while the keyboard is
//!   on the rows;
//! - when saving, a **text field** named *Name*, holding the name typed as its
//!   description, which has the keyboard while the keyboard is in it;
//! - a **label** saying what just happened, when anything did, announced;
//! - **Cancel**, then **Open** or **Save**.
//!
//! **Every word is one the sheet draws** (`crate::the_file_choosers_words`),
//! so what is heard and what is seen cannot drift (ADR 0089).
//!
//! # A reader presses what a click lands on
//!
//! A row, the name, *Cancel* and the button carry a press, numbered with the
//! sheet it was heard on, and the sheet answers it
//! (`crate::the_file_chooser::TheFileChooser::a_reader_pressed`). A row
//! pressed by a reader is used at once, as Enter on it is.

use alo_file_chooser::{How, Kind, Where};
use alo_strings::{Filling, Strings, Word};

use crate::access_action::Pressed;
use crate::access_nodes::{Node, ReadAloudTree};
use crate::access_roles;
use crate::access_sheet_in_front::{ASheetInFront, TheSheet};
use crate::the_file_chooser::{FileChooserShows, OnTheFileChooser};
use crate::the_file_choosers_words as words;

impl ReadAloudTree {
    /// The same tree, with the file chooser in front while one is up.
    /// `shows` is [`None`] when none is, and the tree is left as it was.
    /// `called` is what each application is called, by its identifier.
    #[must_use]
    pub(crate) fn with_the_file_chooser(
        mut self,
        strings: &Strings,
        shows: Option<FileChooserShows<'_>>,
        called: &dyn Fn(&str) -> String,
    ) -> Self {
        let Some(shows) = shows else {
            return self;
        };
        let chooser = shows.chooser;
        let said = |word: Word, filling: &Filling| strings.say(&word.key(), filling).into_text();
        let read_only = || access_roles::words_of(alo_access::State::ReadOnly, None);
        let pressed = |on| {
            Some(Pressed::OnTheFileChooser {
                sheet: shows.number,
                on,
            })
        };
        let (title, asked, button) = match chooser.how() {
            How::Open => (words::OPEN_A_FILE, words::ASKED_TO_OPEN, words::OPEN),
            How::Save => (words::SAVE_A_FILE, words::ASKED_TO_SAVE, words::SAVE),
        };
        let sheet = self.push(
            0,
            Node {
                role: access_roles::number_of(alo_access::Role::Dialogue),
                name: said(title, &Filling::nothing()),
                states: access_roles::in_front(read_only()),
                ..Node::default()
            },
        );
        let asking = called(shows.asking.identifier());
        let folder = chooser.at().to_string_lossy().into_owned();
        for line in [
            said(asked, &Filling::of("application", asking)),
            said(words::IN_THE_FOLDER, &Filling::of("folder", folder)),
        ] {
            self.push(
                sheet,
                Node {
                    role: access_roles::number_of(alo_access::Role::Label),
                    name: line,
                    states: read_only(),
                    ..Node::default()
                },
            );
        }
        let list = self.push(
            sheet,
            Node {
                role: access_roles::number_of(alo_access::Role::List),
                // **No name**: the design note draws no heading over the list.
                states: read_only(),
                ..Node::default()
            },
        );
        let mut keyboard = list;
        for (at, row) in chooser.rows().iter().enumerate() {
            let on = at == chooser.focus();
            let states = access_roles::words_of(alo_access::State::OneOfAFew, Some(on));
            let has_the_keyboard = on && chooser.on() == Where::TheRows;
            let node = self.push(
                list,
                Node {
                    role: access_roles::number_of(alo_access::Role::ListItem),
                    name: row.name.to_owned(),
                    description: said(
                        match row.kind {
                            Kind::Above => words::THE_FOLDER_ABOVE,
                            Kind::Folder => words::FOLDER,
                            Kind::File => words::FILE,
                        },
                        &Filling::nothing(),
                    ),
                    states: if has_the_keyboard {
                        access_roles::with_the_keyboard(states)
                    } else {
                        states
                    },
                    press: pressed(OnTheFileChooser::TheRow(at)),
                    ..Node::default()
                },
            );
            if has_the_keyboard {
                keyboard = node;
            }
        }
        if chooser.how() == How::Save {
            let states = access_roles::words_of(alo_access::State::CanBeUsed, None);
            let in_it = chooser.on() == Where::TheName;
            let field = self.push(
                sheet,
                Node {
                    role: access_roles::number_of(alo_access::Role::Entry),
                    name: said(words::NAME, &Filling::nothing()),
                    description: chooser.name().to_owned(),
                    states: if in_it {
                        access_roles::with_the_keyboard(states)
                    } else {
                        states
                    },
                    press: pressed(OnTheFileChooser::TheName),
                    ..Node::default()
                },
            );
            if in_it {
                keyboard = field;
            }
        }
        if let Some(note) = chooser.note() {
            self.push(
                sheet,
                Node {
                    role: access_roles::number_of(alo_access::Role::Label),
                    name: words::said(strings, note),
                    states: read_only(),
                    announced: true,
                    ..Node::default()
                },
            );
        }
        for (word, on) in [
            (words::CANCEL, OnTheFileChooser::Cancel),
            (button, OnTheFileChooser::TheButton),
        ] {
            self.push(
                sheet,
                Node {
                    role: access_roles::number_of(alo_access::Role::Button),
                    name: said(word, &Filling::nothing()),
                    states: access_roles::words_of(alo_access::State::CanBeUsed, None),
                    press: pressed(on),
                    ..Node::default()
                },
            );
        }
        self.put_in_front(ASheetInFront {
            which: TheSheet::FileChooser(shows.number),
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
#[path = "the_file_chooser_as_read_tests.rs"]
mod tests;
