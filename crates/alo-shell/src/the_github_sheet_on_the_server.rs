//! **What the GitHub sign-in sheet asks, carried out by the server** — the
//! sheet decides nothing itself (`crate::the_github_sheet`).
//!
//! - **A sign-in begins** with the sheet going up and the desktop asked to
//!   run `gh auth login --web`, in that order and on one call, so there is
//!   never a sign-in running with no sheet to show its code.
//! - ***Copy*** puts the code on the clipboard (`crate::the_shell_copies`).
//! - ***Open*** opens GitHub's page by the road the code's own page took
//!   (`crate::asking_github`): only a page on `https://github.com/`, in the
//!   browser Settings → Default apps says opens web links.
//! - ***Cancel***, Escape, and a sheet no display could draw ask the desktop
//!   to stop the sign-in, which it records and answers as cancelled — said on
//!   Settings → Accounts.

use crate::asking_github::AskingGitHub;
use crate::the_github_sheet::{GitHubSheetKey, OnTheGitHubSheet, TheSheetAsks};

impl crate::Server {
    /// **The person asked to sign in to GitHub**: the sheet goes up waiting
    /// for the code, and the desktop is asked to sign in. Nothing where a
    /// sheet is up already, so one press never starts two sign-ins.
    pub(crate) fn a_github_sign_in_begins(&mut self) {
        if self.github_sheet.opened() {
            self.github_is_asked(AskingGitHub::SignIn);
        }
    }

    /// A key reached the sheet.
    pub(crate) fn the_github_sheet_key(&mut self, key: GitHubSheetKey) {
        let asks = self.github_sheet.pressed(key);
        self.the_github_sheet_asked(asks);
    }

    /// A press of the pointer landed on `on`.
    pub(crate) fn the_github_sheet_pressed(&mut self, on: OnTheGitHubSheet) {
        let asks = self.github_sheet.press(on);
        self.the_github_sheet_asked(asks);
    }

    /// **A screen reader pressed `on`, on the sheet numbered `sheet`** —
    /// `false` where that sheet is no longer up or does not draw `on` now.
    pub(crate) fn a_reader_pressed_on_the_github_sheet(
        &mut self,
        sheet: u64,
        on: OnTheGitHubSheet,
    ) -> bool {
        match self.github_sheet.a_reader_pressed(sheet, on) {
            Some(asks) => {
                self.the_github_sheet_asked(asks);
                true
            }
            None => false,
        }
    }

    /// The sheet could not be drawn on the display the person is working on.
    pub(crate) fn the_github_sheet_could_not_be_drawn(&mut self) {
        let asks = self.github_sheet.could_not_be_drawn();
        self.the_github_sheet_asked(asks);
    }

    /// Carry out what the sheet asked.
    fn the_github_sheet_asked(&mut self, asks: Option<TheSheetAsks>) {
        match asks {
            Some(TheSheetAsks::Copy(code)) => self.the_shell_copies(&code),
            Some(TheSheetAsks::OpenThePage(page)) => self.githubs_page_is_opened(&page),
            Some(TheSheetAsks::StopSigningIn) => {
                self.github_is_asked(AskingGitHub::StopSigningIn);
            }
            // The person pressed *Sign in* on the sheet that rose by itself:
            // the sheet is waiting for the code, and gh is asked for it now.
            Some(TheSheetAsks::SignIn) => self.github_is_asked(AskingGitHub::SignIn),
            None => {}
        }
    }
}

#[cfg(test)]
#[path = "the_github_sheet_on_the_server_tests.rs"]
mod tests;
