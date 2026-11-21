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

/// One application alo opened a file in, waited for.
#[derive(Debug)]
pub(crate) struct Awaited {
    /// The application.
    app: alo_dock::AppId,
    /// Until when its window may still come.
    until: Instant,
    /// The names windows went by when the file was opened, so what is said on
    /// the log names only the windows that came since — never the person's
    /// other applications.
    there_before: Vec<String>,
    /// What was last said about why no window was brought in front.
    said: String,
}

/// What one look found: the window to bring in front, or why there is none.
enum Found {
    /// The most recently opened window of the application alo may type into.
    It(WlSurface),
    /// None, and why, for the log.
    Not(String),
}

impl Server {
    /// **alo was approved to open a file in `application`, and it opened**:
    /// its window, once there is one alo may type into, comes in front.
    pub fn alo_opened_a_file_in(&mut self, application: &str, now: Instant) {
        let Ok(app) = alo_dock::AppId::named(application) else {
            return;
        };
        self.alo_opened_a_file_in
            .retain(|awaited| awaited.app != app);
        if self.alo_opened_a_file_in.len() >= AT_MOST_WAITED_FOR {
            self.alo_opened_a_file_in.remove(0);
        }
        let there_before = self
            .the_windows_the_dock_sees()
            .each_as_opened()
            .filter_map(|window| window.app().map(|it| it.name().to_owned()))
            .collect();
        self.alo_opened_a_file_in.push(Awaited {
            app,
            until: now + COMES_WITHIN,
            there_before,
            said: String::new(),
        });
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
        for mut awaited in std::mem::take(&mut self.alo_opened_a_file_in) {
            match self.the_window_alo_may_type_into(&awaited, may) {
                Found::It(surface) => {
                    // **The window alo opened a file in is alo's for the
                    // turn**: it wears the task at work from the moment it
                    // comes (`crate::the_window_alo_acts_in`).
                    let _ = self.alo_acts_in(&surface, awaited.app.name());
                    // **What the keys are taken from is remembered**, so the
                    // card that follows can come in front of this window
                    // rather than rise behind it
                    // (`crate::the_card_comes_in_front_of_what_alo_opened`).
                    self.alo_takes_the_keys_to(crate::window_number::the_window_id_of(&surface));
                    let came = self.the_person_goes_to_this_window(&surface);
                    eprintln!(
                        "alo-shell: the window alo opened a file in, of {}, {}",
                        awaited.app.name(),
                        if came.is_ok() {
                            "came in front with the keyboard"
                        } else {
                            "went before it could come in front"
                        }
                    );
                }
                Found::Not(why) if now >= awaited.until => eprintln!(
                    "alo-shell: no window of {} that alo may type into came within {}s of \
                     opening a file in it, so nothing was brought in front: {why}",
                    awaited.app.name(),
                    COMES_WITHIN.as_secs()
                ),
                Found::Not(why) => {
                    // **Why not, said once each time it changes**: the walk of
                    // 2026-11-14 (run 43679048665) saw VSCodium's window up and
                    // never brought forward, and nothing said whether its name
                    // or its process was the reason.
                    if why != awaited.said {
                        eprintln!(
                            "alo-shell: no window of {} that alo may type into yet: {why}",
                            awaited.app.name()
                        );
                        awaited.said = why;
                    }
                    still.push(awaited);
                }
            }
        }
        self.alo_opened_a_file_in = still;
    }

    /// **The most recently opened window of the application whose process
    /// `may` says may be typed into** — or, in the same one pass, why there is
    /// none: each window of it, with its process and the desktop's answer, and
    /// the names of windows that came since the file was opened under a name
    /// that is not the application's. Names and process numbers only, never a
    /// title, and each name escaped, because a program names its own window.
    fn the_window_alo_may_type_into(&self, awaited: &Awaited, may: &mut dyn MayTypeInto) -> Found {
        let windows = self.the_windows_the_dock_sees();
        let mut of_it = Vec::new();
        let mut came_since = Vec::new();
        let opened: Vec<&alo_dock::Window> = windows.each_as_opened().collect();
        for window in opened.into_iter().rev() {
            if !crate::typing_into_the_window_in_front::a_window_of(window.app(), &awaited.app) {
                let name = window.app().map(|it| it.name().to_owned());
                if !name
                    .as_ref()
                    .is_some_and(|it| awaited.there_before.contains(it))
                {
                    came_since.push(format!("{name:?}"));
                }
                continue;
            }
            let name = window.app().map(alo_dock::AppId::name).unwrap_or_default();
            if window.sits().was_put_aside() {
                of_it.push(format!("{name:?} put aside"));
                continue;
            }
            let surface = self.the_window_numbered(window.id()).cloned();
            let pid = surface
                .as_ref()
                .and_then(|surface| surface.client())
                .and_then(|client| client.get_credentials(&self.display_handle()).ok())
                .and_then(|credentials| u32::try_from(credentials.pid).ok());
            match (surface, pid) {
                (Some(surface), Some(pid)) if may.may_type_into(awaited.app.name(), pid) => {
                    return Found::It(surface);
                }
                (_, Some(pid)) => {
                    of_it.push(format!("{name:?} by process {pid}, not alo's to type into"));
                }
                (_, None) => of_it.push(format!("{name:?} by a process this machine cannot name")),
            }
        }
        Found::Not(format!(
            "windows of it: [{}]; windows that came since, named otherwise: [{}]",
            of_it.join("; "),
            came_since.join(", ")
        ))
    }
}

#[cfg(test)]
#[path = "a_file_alo_opened_comes_in_front_tests.rs"]
mod tests;
