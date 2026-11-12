//! **The password sheet, as a screen reader is told about it** (ADR 0150).
//!
//! Until now a reader was told nothing at all when `sudo` asked for the
//! person's password: the sheet was drawn, every key went to it, and the tree
//! said only *the desktop*. Somebody who cannot see the screen had a field
//! eating their keys and no way to know why.
//!
//! # What is read, in order
//!
//! - a **dialogue**, named by the sheet's heading — *alo is asking to run this
//!   as administrator:* or *Something on this machine is asking…*;
//! - a **label**: the command, or polkit's own sentence, whole;
//! - a **password field**, which has the keyboard, named by the line drawn
//!   under it — how to answer, *Checking your password…*, or what went wrong.
//!
//! **Every word is one the sheet draws**, asked of the same
//! `crate::the_password_sheets_words` choice the drawing asks, so what is
//! heard and what is seen are one sentence (ADR 0089). The field has no label
//! of its own on the sheet; the line under it is the only drawn sentence that
//! says what it is for, so it is the field's name, and when it changes — the
//! password was not right — the reader is told the new one
//! (`crate::access_events`).
//!
//! # Nothing about the password crosses
//!
//! The field is a **password field**, and nothing on this tree answers `Text`,
//! so no reader, agent or anything else on the bus can ask what is in it — not
//! even whether anything is typed. Nothing here can be pressed: Enter and
//! Escape are the person's keys and already the sheet's.

use alo_strings::{Filling, Strings, Word};

use crate::access_nodes::{Node, ReadAloudTree};
use crate::access_roles;
use crate::access_sheet_in_front::{ASheetInFront, TheSheet};
use crate::the_password_sheet::SheetShows;
use crate::the_password_sheets_words as words;

impl ReadAloudTree {
    /// The same tree, with the password sheet in front while a question is up.
    /// `shows` is [`None`] when none is, and the tree is left as it was.
    #[must_use]
    pub(crate) fn with_the_password_sheet(
        mut self,
        strings: &Strings,
        shows: Option<SheetShows<'_>>,
    ) -> Self {
        let Some(shows) = shows else {
            return self;
        };
        let (heading, what) = words::the_question(shows.what_for);
        let sheet = self.push(
            0,
            Node {
                role: access_roles::number_of(alo_access::Role::Dialogue),
                name: said(strings, heading),
                states: access_roles::in_front(access_roles::words_of(
                    alo_access::State::ReadOnly,
                    None,
                )),
                ..Node::default()
            },
        );
        self.push(
            sheet,
            Node {
                role: access_roles::number_of(alo_access::Role::Label),
                name: what.to_owned(),
                states: access_roles::words_of(alo_access::State::ReadOnly, None),
                ..Node::default()
            },
        );
        let field = self.push(
            sheet,
            Node {
                role: access_roles::number_of(alo_access::Role::PasswordEntry),
                name: said(strings, words::the_line_under(shows.under)),
                states: access_roles::with_the_keyboard(access_roles::words_of(
                    alo_access::State::CanBeUsed,
                    None,
                )),
                ..Node::default()
            },
        );
        self.put_in_front(ASheetInFront {
            which: TheSheet::Password(shows.asked),
            at: sheet,
            keyboard: field,
        });
        self
    }
}

/// One of the sheet's words, in the person's language.
fn said(strings: &Strings, word: Word) -> String {
    strings
        .say(&word.key(), &Filling::nothing())
        .text()
        .to_owned()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_password_sheet_as_read_tests.rs"]
mod tests;
