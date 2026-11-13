//! **The GitHub sign-in sheet**: the one-time code GitHub's own command, gh,
//! printed, and what signing in came to — as drawn in the proposal the owner
//! approved on 2026-11-13 (`docs/design/github-sign-in.md`, *The sheet*).
//!
//! # What it shows, in turn
//!
//! 1. **Waiting for the code**, from the moment the person presses *Sign in*
//!    until gh prints one: the sentence, the card saying *Checking…*, and
//!    *Cancel* alone. Not drawn in the proposal; see the design record for
//!    why it offers no *Copy* and no *Open*: neither has anything to act on
//!    yet, and a button that does nothing is a button that lies.
//! 2. **The code**: *Copy*, *Cancel* and *Open* the page. The page was already
//!    opened in the person's browser when the code arrived
//!    (`crate::asking_github`); *Open* opens it again.
//! 3. **Signed in**, as a login: *Done*.
//!
//! **A sign-in that did not work takes the sheet down** and is said on
//! Settings → Accounts, where every other answer about GitHub is said
//! (`crate::settings_accounts`). The *Open with* sheet has no failure of its
//! own to show either; this sheet keeps that shape rather than draw a third
//! state the proposal does not.
//!
//! # Keys
//!
//! Every key is the sheet's while it is up (`crate::a_key_reaches_the_github_sheet`).
//! **Tab and the arrows move between the buttons**, in the order they are
//! drawn and round again; **Enter or Space presses** the one the keyboard is
//! on, which starts on the navy button; **Escape is *Cancel*** — or *Done*,
//! once signed in, because there is nothing left to cancel.
//!
//! # It decides nothing
//!
//! The sheet asks; the server does ([`TheSheetAsks`]): the clipboard, the
//! page, and stopping gh are the server's and the desktop's. **There is never
//! a password field**: GitHub asks for a password on its own page, in the
//! person's own browser.

use crate::asking_github::{AskingGitHub, GitHubAnswered};

/// A button on the sheet: what a press of the pointer, a key, or a screen
/// reader can land on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnTheGitHubSheet {
    /// *Copy*: the code goes on the clipboard.
    Copy,
    /// *Cancel*: signing in stops and the sheet closes.
    Cancel,
    /// *Open github.com/login/device*: GitHub's page opens again.
    OpenThePage,
    /// *Done*: the sheet closes, signed in.
    Done,
}

/// What a press asks of the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TheSheetAsks {
    /// Put this code on the clipboard.
    Copy(String),
    /// Open this page, as the code's own page was opened.
    OpenThePage(String),
    /// Stop the sign-in that is waiting on the person.
    StopSigningIn,
}

/// Where the sign-in is, as the sheet shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TheSheetIs {
    /// Signing in was asked and gh has not printed a code yet.
    WaitingForTheCode,
    /// gh printed this code, to be typed on this page.
    TheCodeIs {
        /// The code, exactly as gh printed it.
        code: String,
        /// The page, exactly as gh printed it.
        page: String,
    },
    /// Signed in, as this login.
    SignedIn(String),
}

impl TheSheetIs {
    /// The buttons drawn, in the order they are drawn and Tab goes through.
    pub(crate) const fn buttons(&self) -> &'static [OnTheGitHubSheet] {
        match self {
            Self::WaitingForTheCode => &[OnTheGitHubSheet::Cancel],
            Self::TheCodeIs { .. } => &[
                OnTheGitHubSheet::Copy,
                OnTheGitHubSheet::Cancel,
                OnTheGitHubSheet::OpenThePage,
            ],
            Self::SignedIn(_) => &[OnTheGitHubSheet::Done],
        }
    }

    /// Where the keyboard starts: the navy button, or *Cancel* while there is
    /// none.
    const fn the_keyboard_starts_on(&self) -> OnTheGitHubSheet {
        match self {
            Self::WaitingForTheCode => OnTheGitHubSheet::Cancel,
            Self::TheCodeIs { .. } => OnTheGitHubSheet::OpenThePage,
            Self::SignedIn(_) => OnTheGitHubSheet::Done,
        }
    }
}

/// A key, as the sheet understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitHubSheetKey {
    /// Tab, Right or Down: the next button, and round to the first.
    Next,
    /// Shift+Tab, Left or Up: the button before, and round to the last.
    Previous,
    /// Home: the first button.
    First,
    /// End: the last button.
    Last,
    /// Enter or Space: press the button the keyboard is on.
    Press,
    /// Escape: *Cancel*, or *Done* once signed in.
    Escape,
    /// Anything else, which does nothing.
    Nothing,
}

impl GitHubSheetKey {
    /// What a key means here, read as Settings reads it — the one reading of a
    /// key for every surface of this window's family.
    pub(crate) const fn of(key: crate::SettingsKey) -> Self {
        use crate::SettingsKey as K;
        match key {
            K::Next | K::More | K::NextSection => Self::Next,
            K::Previous | K::Less | K::PreviousSection => Self::Previous,
            K::First => Self::First,
            K::Last => Self::Last,
            K::Choose => Self::Press,
            K::Close => Self::Escape,
            K::PutBackThisOne | K::PutBackAsShipped | K::Nothing => Self::Nothing,
        }
    }
}

/// The sheet that is up.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Shown {
    /// What it shows.
    is: TheSheetIs,
    /// The button the keyboard is on.
    focus: OnTheGitHubSheet,
    /// Which sheet this is, counted from the first to go up: what a screen
    /// reader's press is held to, so a press heard on one sheet never acts on
    /// the next.
    number: u64,
}

/// What the sheet shows, for drawing it and reading it aloud.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GitHubSheetShows<'a> {
    /// Where the sign-in is.
    pub(crate) is: &'a TheSheetIs,
    /// The button the keyboard is on.
    pub(crate) focus: OnTheGitHubSheet,
    /// Which sheet this is.
    pub(crate) number: u64,
}

/// The GitHub sign-in sheet, up or not.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TheGitHubSheet {
    /// The sheet that is up.
    shown: Option<Shown>,
    /// How many sheets have gone up.
    went_up: u64,
}

impl TheGitHubSheet {
    /// Whether the sheet is up, so every key and press is its.
    pub(crate) const fn is_up(&self) -> bool {
        self.shown.is_some()
    }

    /// What the sheet that is up shows.
    pub(crate) fn shows(&self) -> Option<GitHubSheetShows<'_>> {
        self.shown.as_ref().map(|shown| GitHubSheetShows {
            is: &shown.is,
            focus: shown.focus,
            number: shown.number,
        })
    }

    /// **A sign-in begins**: the sheet goes up waiting for the code. `false`
    /// where one is up already, and nothing is asked twice.
    pub(crate) fn opened(&mut self) -> bool {
        if self.shown.is_some() {
            return false;
        }
        self.went_up = self.went_up.wrapping_add(1);
        let is = TheSheetIs::WaitingForTheCode;
        self.shown = Some(Shown {
            focus: is.the_keyboard_starts_on(),
            is,
            number: self.went_up,
        });
        true
    }

    /// **What the desktop answered**, as the sheet takes it. `true` only for a
    /// code this sheet was waiting for — the one case in which GitHub's page
    /// is opened, so a code that arrives after the person cancelled opens
    /// nothing.
    pub(crate) fn answered(&mut self, answered: &GitHubAnswered) -> bool {
        let Some(shown) = self.shown.as_mut() else {
            return false;
        };
        match answered {
            GitHubAnswered::TheCodeIs { code, page }
                if shown.is == TheSheetIs::WaitingForTheCode =>
            {
                shown.is = TheSheetIs::TheCodeIs {
                    code: code.clone(),
                    page: page.clone(),
                };
                shown.focus = shown.is.the_keyboard_starts_on();
                true
            }
            GitHubAnswered::SignedIn(login) if !matches!(shown.is, TheSheetIs::SignedIn(_)) => {
                shown.is = TheSheetIs::SignedIn(login.clone());
                shown.focus = shown.is.the_keyboard_starts_on();
                false
            }
            // **A sign-in that did not work takes the sheet down**; why is
            // said on the Accounts page.
            GitHubAnswered::DidNotWork {
                asked: AskingGitHub::SignIn | AskingGitHub::StopSigningIn,
                ..
            } if !matches!(shown.is, TheSheetIs::SignedIn(_)) => {
                self.shown = None;
                false
            }
            _ => false,
        }
    }

    /// A key while the sheet is up.
    pub(crate) fn pressed(&mut self, key: GitHubSheetKey) -> Option<TheSheetAsks> {
        let shown = self.shown.as_mut()?;
        let buttons = shown.is.buttons();
        let at = buttons
            .iter()
            .position(|button| *button == shown.focus)
            .unwrap_or(0);
        let last = buttons.len().saturating_sub(1);
        let to = |index: usize| buttons.get(index).copied();
        match key {
            GitHubSheetKey::Next => {
                shown.focus = to(if at >= last { 0 } else { at + 1 })?;
                None
            }
            GitHubSheetKey::Previous => {
                shown.focus = to(if at == 0 { last } else { at - 1 })?;
                None
            }
            GitHubSheetKey::First => {
                shown.focus = to(0)?;
                None
            }
            GitHubSheetKey::Last => {
                shown.focus = to(last)?;
                None
            }
            GitHubSheetKey::Press => {
                let focus = shown.focus;
                self.press(focus)
            }
            GitHubSheetKey::Escape => {
                let leaving = if matches!(shown.is, TheSheetIs::SignedIn(_)) {
                    OnTheGitHubSheet::Done
                } else {
                    OnTheGitHubSheet::Cancel
                };
                self.press(leaving)
            }
            GitHubSheetKey::Nothing => None,
        }
    }

    /// Whether `on` is drawn on the sheet now.
    pub(crate) fn has(&self, on: OnTheGitHubSheet) -> bool {
        self.shown
            .as_ref()
            .is_some_and(|shown| shown.is.buttons().contains(&on))
    }

    /// **A button is pressed**, by the pointer, a key or a reader. A button
    /// not drawn now does nothing: a press is never carried out on a button
    /// the person cannot see.
    pub(crate) fn press(&mut self, on: OnTheGitHubSheet) -> Option<TheSheetAsks> {
        if !self.has(on) {
            return None;
        }
        let shown = self.shown.as_mut()?;
        shown.focus = on;
        match (on, &shown.is) {
            (OnTheGitHubSheet::Copy, TheSheetIs::TheCodeIs { code, .. }) => {
                Some(TheSheetAsks::Copy(code.clone()))
            }
            (OnTheGitHubSheet::OpenThePage, TheSheetIs::TheCodeIs { page, .. }) => {
                Some(TheSheetAsks::OpenThePage(page.clone()))
            }
            (OnTheGitHubSheet::Cancel, _) => {
                self.shown = None;
                Some(TheSheetAsks::StopSigningIn)
            }
            (OnTheGitHubSheet::Done, _) => {
                self.shown = None;
                None
            }
            // Never reached: `has` keeps every press to a button drawn for
            // what the sheet shows.
            _ => None,
        }
    }

    /// **A screen reader pressed `on`, on the sheet numbered `sheet`** — the
    /// door a click takes. Held to the sheet it was heard on: a press for any
    /// other sheet, or for a button not drawn now, does nothing, and says so
    /// with [`None`].
    pub(crate) fn a_reader_pressed(
        &mut self,
        sheet: u64,
        on: OnTheGitHubSheet,
    ) -> Option<Option<TheSheetAsks>> {
        let shown = self.shown.as_ref()?;
        if shown.number != sheet || !self.has(on) {
            return None;
        }
        Some(self.press(on))
    }

    /// **The sheet could not be drawn on this display**: it comes down rather
    /// than hold every key with nothing on the screen — the password sheet's
    /// rule — and a sign-in still waiting on the person is stopped, because
    /// nobody can see its code.
    pub(crate) fn could_not_be_drawn(&mut self) -> Option<TheSheetAsks> {
        let shown = self.shown.take()?;
        match shown.is {
            TheSheetIs::SignedIn(_) => None,
            TheSheetIs::WaitingForTheCode | TheSheetIs::TheCodeIs { .. } => {
                Some(TheSheetAsks::StopSigningIn)
            }
        }
    }
}

#[cfg(test)]
#[path = "the_github_sheet_tests.rs"]
mod tests;
