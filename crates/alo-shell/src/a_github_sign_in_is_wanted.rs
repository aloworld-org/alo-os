//! **A command alo ran failed because nobody is signed in to GitHub, so the
//! sign-in sheet goes up** — the owner's approval of 2026-11-13: the sheet
//! *appears when alo can't publish because you're not signed in*
//! (`docs/design/github-sign-in.md`).
//!
//! `alo-agentd` tells the desktop once per such failure; the desktop hands it
//! here once a frame (`crate::direct_desktop::TheDesktop::a_command_needed_a_github_sign_in`).
//! Going up is exactly what *Sign in* on Settings → Accounts does
//! (`crate::the_github_sheet_on_the_server`): the sheet, and GitHub's own
//! `gh auth login --web` asked for its code.
//!
//! # Never stacked, and never over another sheet
//!
//! Decided from the *Open with* sheet's rule (`crate::the_open_with_sheet`):
//! **one sheet is up, and another waits behind it — never for ever.**
//!
//! - **While the GitHub sheet is up, another failure is let go.** It is up
//!   already, asking the very thing a second one would ask; a second sheet, or
//!   a second sign-in behind the first, would be one the person has to dismiss
//!   twice.
//! - **While another sheet is up** — *Open with*, or the password sheet — **it
//!   waits**, and goes up the frame that sheet comes down. However many
//!   failures arrive while it waits, **one** sheet goes up.
//! - **It waits at most [`THE_LONGEST`]**, the *Open with* sheet's own bound,
//!   and is let go after it: a sheet that rose a quarter of an hour after the
//!   person's request would be about something they had stopped looking at.
//!
//! # What it never does
//!
//! **It never runs the command again.** Signing in is the person's to do or
//! dismiss, and asking alo again is theirs too — alo's answer says so — because
//! a command run again by itself would be a second run nobody approved.

use std::time::Instant;

/// The longest a sign-in waits behind another sheet before it is let go —
/// the *Open with* sheet's own bound.
pub(crate) const THE_LONGEST: std::time::Duration = crate::the_open_with_sheet::THE_LONGEST;

/// Whether a sign-in is waiting to go up, and since when.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ASignInWanted {
    /// When the first failure still waiting arrived, and [`None`] when none
    /// is.
    since: Option<Instant>,
}

/// What a frame should do about a wanted sign-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItGoesUp {
    /// Put the sheet up now.
    Now,
    /// Nothing, this frame.
    NotNow,
}

impl ASignInWanted {
    /// **A command needed a sign-in.** Let go while the GitHub sheet is up;
    /// otherwise it waits, and a failure while one already waits changes
    /// nothing — so many failures are one sheet.
    pub(crate) fn heard(&mut self, the_github_sheet_is_up: bool, now: Instant) {
        if the_github_sheet_is_up {
            return;
        }
        if self.since.is_none() {
            self.since = Some(now);
        }
    }

    /// **Whether the sheet goes up this frame**: only when one is waiting, it
    /// has waited no longer than [`THE_LONGEST`], and no sheet is up. One that
    /// goes up, or has waited too long, is no longer waiting; the GitHub sheet
    /// being up already answers it.
    pub(crate) fn this_frame(
        &mut self,
        the_github_sheet_is_up: bool,
        another_sheet_is_up: bool,
        now: Instant,
    ) -> ItGoesUp {
        let Some(since) = self.since else {
            return ItGoesUp::NotNow;
        };
        if the_github_sheet_is_up || now.saturating_duration_since(since) > THE_LONGEST {
            self.since = None;
            return ItGoesUp::NotNow;
        }
        if another_sheet_is_up {
            return ItGoesUp::NotNow;
        }
        self.since = None;
        ItGoesUp::Now
    }
}

impl crate::Server {
    /// **A command alo ran needed a GitHub sign-in**, as the desktop was told.
    pub(crate) fn a_command_needed_a_github_sign_in(&mut self, now: Instant) {
        let up = self.github_sheet.is_up();
        self.github_sign_in_wanted.heard(up, now);
    }

    /// **Once a frame: the sheet goes up for a sign-in that is waiting**, when
    /// no other sheet is up — by the same road as *Sign in* on Settings →
    /// Accounts, so it is the sheet the owner approved and nothing else.
    pub(crate) fn a_wanted_github_sign_in_goes_up(
        &mut self,
        another_sheet_is_up: bool,
        now: Instant,
    ) {
        let up = self.github_sheet.is_up();
        if self
            .github_sign_in_wanted
            .this_frame(up, another_sheet_is_up, now)
            == ItGoesUp::Now
        {
            self.a_github_sign_in_begins();
        }
    }
}

#[cfg(test)]
#[path = "a_github_sign_in_is_wanted_tests.rs"]
mod tests;
