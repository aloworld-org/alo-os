//! **What the desktop is asked to do with GitHub's own command, `gh`, and what
//! it answered** — the owner's approval of 2026-11-13 of Settings → Accounts
//! and the GitHub sign-in sheet (`docs/design/github-sign-in.md`).
//!
//! This compositor runs no program. Signing in, signing out and asking who is
//! signed in are the desktop's (`alo-desktop`'s `signing_in_to_github`), so an
//! ask is held here until the desktop takes it on the next frame, and the
//! answer comes back on a later one — the road the *Screen reader* switch
//! takes (`crate::the_screen_reader_switch`).
//!
//! # What a sign-in is
//!
//! `gh auth login --web`: gh prints a one-time code and the page it is typed
//! on, the desktop opens that page in the person's default browser, and the
//! person types the code there. **alo reads the code from gh's own words and
//! fills nothing in**, and there is never a password field — GitHub asks for
//! the password on its own page, in the person's own browser.

use alo_strings::Strings;

use crate::the_settings_words as w;

/// One thing the desktop is asked to do with GitHub's own command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskingGitHub {
    /// Who is signed in: read from gh's own settings, without reaching
    /// GitHub.
    WhoIsSignedIn,
    /// Sign in, with `gh auth login --web`.
    SignIn,
    /// Stop a sign-in that is waiting for the person to type the code.
    StopSigningIn,
    /// Sign out, with `gh auth logout`.
    SignOut,
}

/// What an ask came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitHubAnswered {
    /// Who is signed in, read: the login, or [`None`] for nobody.
    SignedInAs(Option<String>),
    /// A sign-in began: the one-time code gh printed and the page it is typed
    /// on, which the desktop has opened in the person's browser.
    TheCodeIs {
        /// The code, exactly as gh printed it.
        code: String,
        /// The page, exactly as gh printed it.
        page: String,
    },
    /// The sign-in finished, as this login.
    SignInWorked(String),
    /// Signed out.
    SignedOut,
    /// What was asked did not happen.
    DidNotWork {
        /// What was asked.
        asked: AskingGitHub,
        /// Why.
        why: WhyGitHubDidNotWork,
    },
}

/// Why something asked of gh did not happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhyGitHubDidNotWork {
    /// gh is not on this computer.
    GhIsNotHere,
    /// gh stopped with an error: its own last line, where it said one.
    GhFailed(Option<String>),
    /// A sign-in was started and gh never printed a code.
    NoCodeArrived,
    /// The person cancelled the sign-in.
    Cancelled,
}

impl GitHubAnswered {
    /// **What the person is told about an ask that did not happen**, in their
    /// language — the sentence, and gh's own last line under it where it said
    /// one. [`None`] for an answer that worked.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Vec<String>> {
        let Self::DidNotWork { asked, why } = self else {
            return None;
        };
        let (word, gh) = match (asked, why) {
            (_, WhyGitHubDidNotWork::GhIsNotHere) => (w::GH_IS_NOT_HERE, None),
            (AskingGitHub::SignOut, WhyGitHubDidNotWork::GhFailed(gh)) => {
                (w::SIGN_OUT_FAILED, gh.clone())
            }
            (_, WhyGitHubDidNotWork::GhFailed(gh)) => (w::GH_FAILED, gh.clone()),
            (_, WhyGitHubDidNotWork::NoCodeArrived) => (w::NO_CODE_ARRIVED, None),
            (_, WhyGitHubDidNotWork::Cancelled) => (w::SIGN_IN_CANCELLED, None),
        };
        let mut said = vec![crate::settings_page::said(strings, word)];
        said.extend(gh.filter(|line| !line.trim().is_empty()));
        Some(said)
    }
}

impl crate::Server {
    /// Ask the desktop something about GitHub, on the next frame.
    pub(crate) fn github_is_asked(&mut self, asked: AskingGitHub) {
        self.github_to_ask.push(asked);
    }

    /// Everything asked of GitHub since the desktop last took it, in order.
    pub(crate) fn the_github_asks_to_tell(&mut self) -> Vec<AskingGitHub> {
        std::mem::take(&mut self.github_to_ask)
    }

    /// What the desktop answered, told to Settings and to the sign-in sheet
    /// (`crate::the_github_sheet`) — and, for a code the sheet was waiting
    /// for, GitHub's page opened in the person's default browser. **A code
    /// that arrives after the person cancelled opens nothing.**
    pub(crate) fn github_answered(&mut self, answered: &GitHubAnswered, strings: &Strings) {
        self.settings.github_answered(answered, strings);
        let waited_for = self.github_sheet.answered(answered);
        if waited_for && let GitHubAnswered::TheCodeIs { page, .. } = answered {
            self.githubs_page_is_opened(page);
        }
    }

    /// **GitHub's page, opened the way a link a person followed is** — in the
    /// browser Settings → Default apps says opens web links now, started as a
    /// press starts it (`crate::a_link_is_opened`).
    ///
    /// gh is not sandboxed, and the link portal refuses a caller that is not,
    /// so the desktop opens the page itself rather than letting gh ask. **Only
    /// a page on `https://github.com/`**: the address is read from gh's
    /// output, and nothing that output says may send a person anywhere else.
    /// Where nothing opens web links the page says the code, and the person
    /// opens the page by hand.
    pub(crate) fn githubs_page_is_opened(&mut self, page: &str) {
        if !page.starts_with(THE_ONLY_PAGES_OPENED) {
            return;
        }
        let Some(opener) = self
            .settings
            .open()
            .and_then(|open| open.default_apps.opens_now(alo_applications::LinkKind::Web))
        else {
            return;
        };
        if let (Ok(app), Ok(link)) = (
            alo_dock::AppId::named(&opener),
            alo_applications::Link::checked(page),
        ) {
            self.asked_to_open_a_link.push((app, link));
        }
    }
}

/// Where a page gh names must be, to be opened.
const THE_ONLY_PAGES_OPENED: &str = "https://github.com/";

#[cfg(test)]
#[path = "asking_github_tests.rs"]
mod tests;
