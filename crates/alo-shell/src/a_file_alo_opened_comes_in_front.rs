//! **A file alo was approved to open comes in front, in the window alo will
//! type into, with the keyboard** — `open_file_in_application` (2026-11-14),
//! ADR 0153's keyboard road, and `docs/design/where-a-new-window-opens.md`
//! rule 8.
//!
//! # Why this window may take the keyboard
//!
//! Rule 8 reads *a person-initiated opening activates the new window.
//! Background applications and alo must not steal focus … **without an
//! authorised interaction that calls for it***. An approved
//! `open_file_in_application` is that interaction: `docs/features.md` promises
//! that what alo makes is *in front of the person in the application they
//! would have used themselves*, and ADR 0153 types only into the window that
//! holds the keyboard and **never moves the keyboard to type**. So the
//! keyboard goes there once, when the file the person approved opening has
//! opened, and typing never moves it again. A window alo did not open for an
//! approved request still opens behind, as it did.
//!
//! # Which window
//!
//! A window of that application **whose process the desktop says may be typed
//! into** (`crate::typing_into_the_window_in_front::MayTypeInto`) — at the
//! default, only alo's own copy, so the person's own VSCodium window is never
//! the one brought forward; where the person chose *alo may work in your own
//! window*, theirs, which is where the file opened. The most recently opened
//! such window, because that is the one a new start made.
//!
//! **A copy just started has no window yet**, so the ask is remembered for
//! [`COMES_WITHIN`] and looked at once a frame until one comes. A copy already
//! running already has one, which comes in front on the next frame.

use std::time::{Duration, Instant};

use smithay::reexports::wayland_server::Resource as _;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::Server;
use crate::typing_into_the_window_in_front::MayTypeInto;

/// How long a window alo opened a file in may take to come: a cold start of an
/// Electron editor from a Flatpak, which is measured in seconds, with room.
pub const COMES_WITHIN: Duration = Duration::from_secs(90);

/// How many applications the shell waits for at once; alo opens one at a time,
/// so the bound is for a caller with a bug, which must not grow a list for ever.
const AT_MOST_WAITED_FOR: usize = 8;

impl Server {
    /// **alo was approved to open a file in `application`, and it opened**:
    /// its window, once there is one alo may type into, comes in front.
    pub fn alo_opened_a_file_in(&mut self, application: &str, now: Instant) {
        let Ok(app) = alo_dock::AppId::named(application) else {
            return;
        };
        self.alo_opened_a_file_in
            .retain(|(waiting, _)| *waiting != app);
        if self.alo_opened_a_file_in.len() >= AT_MOST_WAITED_FOR {
            self.alo_opened_a_file_in.remove(0);
        }
        self.alo_opened_a_file_in.push((app, now + COMES_WITHIN));
    }

    /// **Bring in front each window alo opened a file in that has come**, and
    /// forget the ones that did not come in time, saying so on the log.
    ///
    /// Called once a frame, after the desktop has answered this frame's
    /// opening (`crate::direct_desktop`).
    pub fn what_alo_opened_comes_in_front(&mut self, may: &mut dyn MayTypeInto, now: Instant) {
        if self.alo_opened_a_file_in.is_empty() {
            return;
        }
        let mut still = Vec::new();
        for (app, until) in std::mem::take(&mut self.alo_opened_a_file_in) {
            match self.the_window_alo_may_type_into(&app, may) {
                Some(surface) => {
                    let came = self.the_person_goes_to_this_window(&surface);
                    eprintln!(
                        "alo-shell: the window alo opened a file in, of {}, {}",
                        app.name(),
                        if came.is_ok() {
                            "came in front with the keyboard"
                        } else {
                            "went before it could come in front"
                        }
                    );
                }
                None if now >= until => eprintln!(
                    "alo-shell: no window of {} that alo may type into came within {}s of \
                     opening a file in it, so nothing was brought in front",
                    app.name(),
                    COMES_WITHIN.as_secs()
                ),
                None => still.push((app, until)),
            }
        }
        self.alo_opened_a_file_in = still;
    }

    /// The most recently opened window of `app` whose process `may` says may
    /// be typed into.
    fn the_window_alo_may_type_into(
        &self,
        app: &alo_dock::AppId,
        may: &mut dyn MayTypeInto,
    ) -> Option<WlSurface> {
        let windows = self.the_windows_the_dock_sees();
        let theirs: Vec<alo_dock::WindowId> = windows
            .each_as_opened()
            .filter(|window| window.app() == Some(app) && !window.sits().was_put_aside())
            .map(alo_dock::Window::id)
            .collect();
        theirs.into_iter().rev().find_map(|id| {
            let surface = self.the_window_numbered(id)?.clone();
            let credentials = surface
                .client()?
                .get_credentials(&self.display_handle())
                .ok()?;
            let pid = u32::try_from(credentials.pid).ok()?;
            may.may_type_into(app.name(), pid).then_some(surface)
        })
    }
}

#[cfg(test)]
#[path = "a_file_alo_opened_comes_in_front_tests.rs"]
mod tests;
