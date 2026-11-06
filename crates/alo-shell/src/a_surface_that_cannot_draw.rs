//! **A window of alo's own that cannot be drawn is closed, and the desktop goes
//! on.**
//!
//! Measured on a booted machine on 2026-11-02 (qemu, 1280 × 800): Tab and Enter
//! in the record window opened the Put back confirmation, the record could not
//! lay it out, and the refusal travelled up the frame as an error. The display
//! loop ended on it, the desktop exited *successfully* — which is the sign-out
//! road — and the person was put back at the sign-in screen without a single
//! window they had open.
//!
//! One surface that cannot be laid out is that surface's problem. The frame is
//! drawn without it (`crate::nested_desktop::frame_pictures` for the record
//! and a question, `crate::direct_desktop` for Settings), and the surface is
//! closed here, so it does not hold the keyboard while drawn nowhere — the
//! fault `Server::the_market_is_taking_the_keys` was written for. The machine's
//! log says which window, on which display, and why.
//!
//! **What the person is shown** is the owner's answer of 2026-11-06: a short
//! notice, in the notification style, naming the window by its own title —
//! `crate::the_closed_windows_notice`. One per closing: a window already
//! closed is not noticed again.

use std::time::Instant;

use crate::RenderError;
use crate::the_closed_windows_notice::AClosedWindow;

impl crate::Server {
    /// The record window could not be drawn on `display` at `now`: close it,
    /// so it takes no keys while it is drawn nowhere, tell the person, and say
    /// why in the log.
    pub(crate) fn the_record_could_not_be_drawn(
        &mut self,
        display: &str,
        why: &RenderError,
        now: Instant,
    ) {
        eprintln!(
            "alo-shell: the record window could not be drawn on {display} ({why:?}), so it has \
             been closed and the rest of the desktop is drawn without it"
        );
        if self.record.is_open() {
            self.closed_notices.closed(AClosedWindow::TheRecord, now);
        }
        self.record.closed();
    }

    /// A question could not be drawn on `display`: say why in the log.
    ///
    /// Not closed here, because a question is not this server's to close: it
    /// is waiting for the person's answer, and the agent that asked it is
    /// still waiting too. It is left out of this frame, and asked again on the
    /// next.
    pub(crate) fn a_question_could_not_be_drawn(display: &str, why: &RenderError) {
        eprintln!(
            "alo-shell: a question could not be drawn on {display} ({why:?}), so this frame is \
             drawn without it"
        );
    }

    /// Settings could not be drawn on `display` at `now`: close it, for the
    /// same reasons, tell the person, and say why in the log.
    pub(crate) fn settings_could_not_be_drawn(
        &mut self,
        display: &str,
        why: &RenderError,
        now: Instant,
    ) {
        eprintln!(
            "alo-shell: Settings could not be drawn on {display} ({why:?}), so it has been \
             closed and the rest of the desktop is drawn without it"
        );
        if self.settings.is_open() {
            self.closed_notices.closed(AClosedWindow::Settings, now);
        }
        self.settings = crate::SettingsWindow::closed();
    }

    /// The notices showing at `now`, worded in `strings`' language.
    pub(crate) fn the_closed_windows_notices(
        &mut self,
        now: Instant,
        strings: &alo_strings::Strings,
    ) -> Vec<String> {
        self.closed_notices.now(now, strings)
    }
}

#[cfg(test)]
#[path = "a_surface_that_cannot_draw_tests.rs"]
mod tests;
