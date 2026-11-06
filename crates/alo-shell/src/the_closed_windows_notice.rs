//! **The short notice a person is shown when one of alo's own windows could
//! not be drawn and was closed** — the owner's answer of 2026-11-06 to the
//! question `crate::a_surface_that_cannot_draw` left open: *a brief
//! notification in the existing notification style*, naming the window by its
//! own title.
//!
//! One notice per closing: a window is noticed only when it was open, so the
//! same refusal on a second display in the same frame — or the frames after,
//! when the window is already closed — adds nothing. Each stays for
//! [`A_NOTICE_STAYS`] and is then gone; nothing about it is kept.
//!
//! **Brief is eight seconds**, a reading taken here and the owner's to
//! change: long enough to read one sentence twice at a slow pace, short enough
//! that it is not still there when the person has moved on.

use std::time::{Duration, Instant};

/// How long one notice stays on the screen.
pub(crate) const A_NOTICE_STAYS: Duration = Duration::from_secs(8);

/// Which of alo's own windows was closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AClosedWindow {
    /// The record window, *What alo did*.
    TheRecord,
    /// Settings.
    Settings,
}

impl AClosedWindow {
    /// The window's own title, as it is shown on it.
    const fn its_title(self) -> alo_strings::Word {
        match self {
            Self::TheRecord => crate::the_records_words::WHAT_ALO_DID,
            Self::Settings => crate::the_settings_words::SETTINGS,
        }
    }

    /// The notice, in `strings`' language.
    pub(crate) fn said(self, strings: &alo_strings::Strings) -> String {
        let nothing = alo_strings::Filling::nothing();
        let title = strings.say(&self.its_title().key(), &nothing).into_text();
        strings
            .say(
                &crate::the_closed_windows_words::WAS_CLOSED.key(),
                &alo_strings::Filling::of("window", title),
            )
            .into_text()
    }
}

/// The notices showing now, each with when it began.
#[derive(Debug, Default)]
pub(crate) struct TheClosedWindowsNotices {
    /// Which window, and since when.
    showing: Vec<(AClosedWindow, Instant)>,
}

impl TheClosedWindowsNotices {
    /// `which` was closed at `now`.
    pub(crate) fn closed(&mut self, which: AClosedWindow, now: Instant) {
        self.showing.push((which, now));
    }

    /// The notices still showing at `now`, in the order they began, worded.
    /// Those that have stayed their time are let go.
    pub(crate) fn now(&mut self, now: Instant, strings: &alo_strings::Strings) -> Vec<String> {
        self.showing
            .retain(|(_, since)| now.saturating_duration_since(*since) < A_NOTICE_STAYS);
        self.showing
            .iter()
            .map(|(which, _)| which.said(strings))
            .collect()
    }
}
