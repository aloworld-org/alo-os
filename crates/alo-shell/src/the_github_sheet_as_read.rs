//! **The GitHub sign-in sheet, as a screen reader is told about it**
//! (`docs/design/github-sign-in.md`, *The sheet*) — the *Open with* sheet's
//! pattern (`crate::the_open_with_sheet_as_read`).
//!
//! # What is read, in the order it is drawn
//!
//! Signing in:
//!
//! - a **dialogue** named *Sign in to GitHub*;
//! - a **label**: *GitHub will ask for this code. Enter it on the page that
//!   opens in your browser.*;
//! - a **label** that is the code itself, exactly as gh printed it — or
//!   *Checking…* while there is none yet;
//! - **Copy**, once there is a code;
//! - a **label** named *What it risks*, the owner's sentence as its
//!   description;
//! - **Cancel**, then **Open github.com/login/device** once there is a page.
//!
//! Signed in: the dialogue named *Signed in to GitHub as* the login; the label
//! *Signed in. You can sign out in Settings → Accounts.*; *What it risks*; and
//! **Done**.
//!
//! **The button the keyboard is on holds the keyboard**, so a reader says
//! where Tab went. Every word is one the sheet draws, so what is heard and what
//! is seen cannot drift (ADR 0089).
//!
//! # A reader presses what a click lands on
//!
//! Each button carries a press numbered with the sheet it was heard on, and
//! the server answers it through the door a click takes
//! (`crate::the_github_sheet_on_the_server`).

use alo_strings::{Filling, Strings, Word};

use crate::access_action::Pressed;
use crate::access_nodes::{Node, ReadAloudTree};
use crate::access_roles;
use crate::access_sheet_in_front::{ASheetInFront, TheSheet};
use crate::the_github_sheet::{GitHubSheetShows, OnTheGitHubSheet, TheSheetIs};
use crate::the_github_sheets_words as words;
use crate::the_settings_words as w;

impl ReadAloudTree {
    /// The same tree, with the GitHub sign-in sheet in front while it is up.
    /// `shows` is [`None`] when it is not, and the tree is left as it was.
    #[must_use]
    pub(crate) fn with_the_github_sheet(
        mut self,
        strings: &Strings,
        shows: Option<GitHubSheetShows<'_>>,
    ) -> Self {
        let Some(shows) = shows else {
            return self;
        };
        let read_only = || access_roles::words_of(alo_access::State::ReadOnly, None);
        let label = |name: String, description: String| Node {
            role: access_roles::number_of(alo_access::Role::Label),
            name,
            description,
            states: read_only(),
            ..Node::default()
        };
        let title = match shows.is {
            TheSheetIs::SignedIn(login) => strings
                .say(
                    &words::SIGNED_IN_TO_GITHUB_AS.key(),
                    &Filling::of("login", login.clone()),
                )
                .into_text(),
            TheSheetIs::WaitingForTheCode | TheSheetIs::TheCodeIs { .. } => {
                said(strings, words::SIGN_IN_TO_GITHUB)
            }
        };
        let sheet = self.push(
            0,
            Node {
                role: access_roles::number_of(alo_access::Role::Dialogue),
                name: title,
                states: access_roles::in_front(read_only()),
                ..Node::default()
            },
        );
        let mut keyboard = sheet;
        match shows.is {
            TheSheetIs::SignedIn(_) => {
                self.push(
                    sheet,
                    label(
                        said(strings, words::SIGNED_IN_AND_WHERE_TO_SIGN_OUT),
                        String::new(),
                    ),
                );
            }
            TheSheetIs::WaitingForTheCode | TheSheetIs::TheCodeIs { .. } => {
                self.push(
                    sheet,
                    label(
                        said(strings, w::GITHUB_WILL_ASK_FOR_THIS_CODE),
                        String::new(),
                    ),
                );
                let code = match shows.is {
                    TheSheetIs::TheCodeIs { code, .. } => code.clone(),
                    TheSheetIs::WaitingForTheCode | TheSheetIs::SignedIn(_) => {
                        said(strings, w::CHECKING)
                    }
                };
                self.push(sheet, label(code, String::new()));
                if shows.is.buttons().contains(&OnTheGitHubSheet::Copy) {
                    let at = self.a_button(strings, sheet, shows, OnTheGitHubSheet::Copy);
                    if shows.focus == OnTheGitHubSheet::Copy {
                        keyboard = at;
                    }
                }
            }
        }
        self.push(
            sheet,
            label(
                said(strings, w::WHAT_IT_RISKS),
                said(strings, w::GITHUB_RISKS),
            ),
        );
        for on in shows.is.buttons() {
            if *on == OnTheGitHubSheet::Copy {
                continue;
            }
            let at = self.a_button(strings, sheet, shows, *on);
            if shows.focus == *on {
                keyboard = at;
            }
        }
        self.put_in_front(ASheetInFront {
            which: TheSheet::GitHub(shows.number),
            at: sheet,
            keyboard,
        });
        self
    }

    /// One of the sheet's buttons under `sheet`, holding the keyboard where it
    /// is on it, and its press held to this sheet.
    fn a_button(
        &mut self,
        strings: &Strings,
        sheet: usize,
        shows: GitHubSheetShows<'_>,
        on: OnTheGitHubSheet,
    ) -> usize {
        let name = match on {
            OnTheGitHubSheet::Copy => said(strings, words::COPY),
            OnTheGitHubSheet::Cancel => said(strings, words::CANCEL),
            OnTheGitHubSheet::Done => said(strings, words::DONE),
            OnTheGitHubSheet::OpenThePage => {
                let page = match shows.is {
                    TheSheetIs::TheCodeIs { page, .. } => page.as_str(),
                    TheSheetIs::WaitingForTheCode | TheSheetIs::SignedIn(_) => "",
                };
                strings
                    .say(
                        &words::OPEN_THE_PAGE.key(),
                        &Filling::of("page", page.trim_start_matches("https://")),
                    )
                    .into_text()
            }
        };
        let states = access_roles::words_of(alo_access::State::CanBeUsed, None);
        self.push(
            sheet,
            Node {
                role: access_roles::number_of(alo_access::Role::Button),
                name,
                states: if shows.focus == on {
                    access_roles::with_the_keyboard(states)
                } else {
                    states
                },
                press: Some(Pressed::OnTheGitHubSheet {
                    sheet: shows.number,
                    on,
                }),
                ..Node::default()
            },
        )
    }
}

/// One word, said in the person's language.
fn said(strings: &Strings, word: Word) -> String {
    strings.say(&word.key(), &Filling::nothing()).into_text()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_github_sheet_as_read_tests.rs"]
mod tests;
