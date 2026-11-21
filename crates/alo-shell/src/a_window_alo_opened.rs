//! **A window alo opened is placed as a window the person opened is**: in
//! empty space on the canvas, covering nothing, with the view moving there
//! when what the person is looking at has no room.
//!
//! The owner, 2026-11-21, on where a window alo opens goes: *"on the canvas
//! like figma, xd, miro so it can be really be hard to fill it all but when
//! filled we go with your recommendation"*, the recommendation being the
//! person's own rule (`docs/design/where-a-new-window-opens.md`, amended that
//! day). Until then a window alo opened kept rule 5, in front and offset, and
//! on the owner's screenshot of 2026-11-20 alo's copy of VSCodium lay under
//! the task window.
//!
//! # Which new windows are alo's
//!
//! **The first window of an application alo started**, by either of its two
//! roads: an approved `open_application` (`Server::its_agent_asked_to_start`,
//! sent by `crate::an_application_is_started`) or an approved
//! `open_file_in_application` (`Server::alo_opened_a_file_in`). The ask is
//! remembered when it is sent and spent by the first window of that
//! application to be placed, or forgotten after [`COMES_WITHIN`]: a file
//! opened in a copy that is already running brings no new window.
//!
//! **Only where it goes.** Whether it comes in front with the keyboard is
//! still rule 8's (`crate::a_window_the_person_opened`,
//! `crate::a_file_alo_opened_comes_in_front`); this file says nothing about
//! focus.

use std::time::{Duration, Instant};

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// How long a window alo started may take to come: the same as
/// `crate::a_file_alo_opened_comes_in_front::COMES_WITHIN`, a cold start of an
/// Electron editor from a Flatpak, with room.
pub(crate) const COMES_WITHIN: Duration = crate::a_file_alo_opened_comes_in_front::COMES_WITHIN;

/// How many applications alo is waited for at once. alo opens one at a time,
/// so the bound is for a caller with a bug.
const AT_MOST_WAITED_FOR: usize = 8;

impl crate::Server {
    /// **alo started `application`**, so its first window is placed as a
    /// window the person opened is.
    pub(crate) fn alo_started(&mut self, application: alo_dock::AppId, now: Instant) {
        self.alo_is_waiting_for
            .retain(|(waiting, _)| *waiting != application);
        if self.alo_is_waiting_for.len() >= AT_MOST_WAITED_FOR {
            self.alo_is_waiting_for.remove(0);
        }
        self.alo_is_waiting_for
            .push((application, now + COMES_WITHIN));
    }

    /// **Whether alo opened this window**, which is waiting to be placed — and
    /// the ask is spent, because only an application's first window is the
    /// one alo started.
    pub(crate) fn alo_opened_this_one(&mut self, surface: &WlSurface, now: Instant) -> bool {
        self.alo_forgets_what_is_late(now);
        let Some(application) = self.the_application_of(surface) else {
            return false;
        };
        let Some(at) = self
            .alo_is_waiting_for
            .iter()
            .position(|(waiting, _)| *waiting == application)
        else {
            return false;
        };
        self.alo_is_waiting_for.remove(at);
        true
    }

    /// **Forget every application whose window is later than it may be**: a
    /// file opened in a copy already running brings no new window, and a
    /// later window of that application is not alo's.
    pub(crate) fn alo_forgets_what_is_late(&mut self, now: Instant) {
        self.alo_is_waiting_for.retain(|(_, until)| now <= *until);
    }
}

#[cfg(test)]
#[path = "a_window_alo_opened_tests.rs"]
mod tests;
