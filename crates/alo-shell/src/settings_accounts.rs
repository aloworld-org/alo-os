//! **Settings → Accounts, as Settings holds it**: whether this computer is
//! signed in to GitHub, as the desktop last said, and what the page has to say
//! about the last thing asked (`docs/design/github-sign-in.md`).
//!
//! Settings reads nothing itself. Each time it opens it asks the desktop once
//! who is signed in ([`SettingsWindow::who_is_signed_in_to_github_is_asked`]),
//! and until the answer comes the row says *Checking…* and offers no button —
//! a *Sign in* that might be a *Sign out* would be a button that lies.
//!
//! **One button, and what it is follows the answer**: *Sign in* where nobody
//! is, *Sign out* where somebody is. A sign-in in progress is the sheet's
//! (`crate::the_github_sheet`), which shows the code; **what a sign-in came to
//! when it did not work is said here**, as every other answer about GitHub is.

use alo_strings::{Filling, Strings};

use crate::asking_github::{AskingGitHub, GitHubAnswered};
use crate::settings_window::{SettingsRow, SettingsWindow};
use crate::the_settings_words as w;

/// Whether this computer is signed in to GitHub, as the desktop last said.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum GitHubHere {
    /// Not answered yet, since Settings opened.
    #[default]
    NotKnownYet,
    /// Nobody is signed in — or gh could not say, which offers the same
    /// button.
    Nobody,
    /// Signed in as this login.
    SignedInAs(String),
}

/// GitHub's sign-in, while Settings is open.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TheGitHubSignIn {
    /// Who is signed in.
    pub(crate) is: GitHubHere,
    /// What the page says about the last thing asked, line by line: empty
    /// where there is nothing to say.
    pub(crate) told: Vec<String>,
    /// Whether the desktop has been asked who is signed in since Settings
    /// opened.
    asked_who: bool,
}

impl TheGitHubSignIn {
    /// The rows the page offers: one button, or none while it is not known.
    pub(crate) fn rows(&self) -> Vec<SettingsRow> {
        match self.is {
            GitHubHere::NotKnownYet => Vec::new(),
            GitHubHere::Nobody => vec![SettingsRow::SignInToGitHub],
            GitHubHere::SignedInAs(_) => vec![SettingsRow::SignOutOfGitHub],
        }
    }

    /// The line under GitHub's name.
    pub(crate) fn line(&self, strings: &Strings) -> String {
        match &self.is {
            GitHubHere::NotKnownYet => crate::settings_page::said(strings, w::CHECKING),
            GitHubHere::Nobody => crate::settings_page::said(strings, w::NOT_SIGNED_IN),
            GitHubHere::SignedInAs(login) => strings
                .say(
                    &w::SIGNED_IN_AS.key(),
                    &Filling::of(w::THE_LOGIN, login.clone()),
                )
                .into_text(),
        }
    }

    /// What the desktop answered.
    pub(crate) fn answered(&mut self, answered: &GitHubAnswered, strings: &Strings) {
        match answered {
            // **A reading changes who is signed in and says nothing**, so a
            // sentence about the last thing the person did stays under it.
            GitHubAnswered::SignedInAs(Some(login)) => {
                self.is = GitHubHere::SignedInAs(login.clone());
            }
            GitHubAnswered::SignedInAs(None) => self.is = GitHubHere::Nobody,
            GitHubAnswered::SignInWorked(login) => {
                self.is = GitHubHere::SignedInAs(login.clone());
                self.told.clear();
            }
            GitHubAnswered::SignedOut => {
                self.is = GitHubHere::Nobody;
                self.told = vec![crate::settings_page::said(strings, w::SIGNED_OUT)];
            }
            // **The code is the sheet's** (`crate::the_github_sheet`), which
            // shows it, copies it, and names the page on its button — so a
            // person whose computer has no browser to open it in can still
            // read the address there. The page says nothing until it is over.
            GitHubAnswered::TheCodeIs { .. } => self.told.clear(),
            GitHubAnswered::DidNotWork { asked, .. } => {
                // **A reading that did not work offers *Sign in***, which is
                // where the person finds out why: the same failure, said.
                if *asked == AskingGitHub::WhoIsSignedIn && self.is == GitHubHere::NotKnownYet {
                    self.is = GitHubHere::Nobody;
                }
                self.told = answered.said(strings).unwrap_or_default();
            }
        }
    }
}

impl SettingsWindow {
    /// **Whether the desktop is to be asked who is signed in to GitHub** —
    /// `true` once each time Settings opens, and never while it is shut.
    pub(crate) fn who_is_signed_in_to_github_is_asked(&mut self) -> bool {
        let Some(open) = self.open_mut() else {
            return false;
        };
        if open.github.asked_who {
            return false;
        }
        open.github.asked_who = true;
        true
    }

    /// What the desktop answered about GitHub. Nothing while Settings is shut:
    /// it asks again when it opens.
    pub(crate) fn github_answered(&mut self, answered: &GitHubAnswered, strings: &Strings) {
        if let Some(open) = self.open_mut() {
            open.github.answered(answered, strings);
        }
    }
}

#[cfg(test)]
#[path = "settings_accounts_tests.rs"]
mod tests;
