//! **A window the person just opened comes in front and takes the keyboard.**
//!
//! `docs/design/where-a-new-window-opens.md`, rule 8: *a person-initiated
//! opening activates the new window. Background applications and alo must not
//! steal focus.* And the owner's decision of 2026-10-31,
//! `crate::which_window_is_in_front`: *the window you go to comes in front.*
//!
//! # What was wrong
//!
//! Measured on a booted machine on 2026-11-01 (image of `c4e6cee0`): Dolphin
//! was started from the alo Bar and the log said it started, and nothing
//! appeared until Super+Tab. Two faults, both here:
//!
//! - a new toplevel was put at the **bottom** of the applications' stack
//!   (`Surfaces::new_toplevel` pushes, and the front is index 0), and nothing
//!   raised it when it mapped;
//! - the applications stayed behind alo's terminal, which the person had used
//!   last, and nothing gave the new window the keyboard.
//!
//! # Which new windows count as the person's
//!
//! Two, and no others:
//!
//! - **the first window of an application the person started** — a Dock
//!   press, Enter in the alo Bar, or a chord — found by the identifier it was
//!   started under, which is the one the Dock already groups windows by
//!   (`Server::the_application_of`). The ask is remembered when it is sent
//!   ([`Server::the_person_started`]) and forgotten when the window comes, or
//!   when the person goes to some other window first: somebody who started an
//!   application and went back to typing elsewhere has chosen where to be.
//! - **a new window of the application the person is in**, while the
//!   applications are in front: a dialog a press in that application opened.
//!   Its own window holds the keyboard, so the new one is the person's too.
//!
//! **An application the agent opened is not brought in front**, which is
//! `crate::which_window_is_in_front`'s *only a person's own road moves it*. A
//! window no one asked for opens behind and without the keyboard, one
//! Super+Tab away — background applications must not steal focus.

use smithay::reexports::wayland_server::Resource as _;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// How many new windows the shell remembers between two looks.
///
/// A frame empties the list, so a real session holds one or two. The bound is
/// for a fixture that never looks: it must not grow for ever.
const AT_MOST_REMEMBERED: usize = 64;

impl crate::surfaces::Surfaces {
    /// **A window came onto the screen**, remembered until the server looks.
    pub(crate) fn it_just_mapped(&mut self, surface: &WlSurface) {
        if self.just_mapped.len() >= AT_MOST_REMEMBERED {
            self.just_mapped.remove(0);
        }
        self.just_mapped.push(surface.clone());
    }
}

impl crate::Server {
    /// **The person started this application**, so its first window will be
    /// theirs to go to. Called when the start was sent and did not fail.
    pub(crate) fn the_person_started(&mut self, application: alo_dock::AppId) {
        if self.the_person_is_waiting_for.contains(&application) {
            return;
        }
        if self.the_person_is_waiting_for.len() >= AT_MOST_REMEMBERED {
            self.the_person_is_waiting_for.remove(0);
        }
        self.the_person_is_waiting_for.push(application);
    }

    /// **The person went somewhere else**, so nothing they started earlier
    /// takes them away from it when its window comes.
    pub(crate) fn the_person_waits_for_nothing(&mut self) {
        self.the_person_is_waiting_for.clear();
    }

    /// **Bring in front the windows the person just opened**, and give the
    /// last of them the keyboard.
    ///
    /// Called once a frame, after clients have been heard
    /// (`crate::direct_desktop`). Every window that came onto the screen since
    /// the last call is looked at once and forgotten; the module header says
    /// which of them are brought in front.
    pub fn what_the_person_opened_comes_in_front(&mut self) {
        for surface in std::mem::take(&mut self.surfaces.just_mapped) {
            if !surface.alive() {
                continue;
            }
            let started = self.the_application_of(&surface).and_then(|application| {
                self.the_person_is_waiting_for
                    .iter()
                    .position(|waiting| *waiting == application)
            });
            if let Some(at) = started {
                self.the_person_is_waiting_for.remove(at);
            } else if !self.a_window_of_the_application_the_person_is_in(&surface) {
                continue;
            }
            // Going to a window forgets what else the person was waiting for,
            // which is right for a window they chose and wrong here: two
            // applications started together are both still coming.
            let still_waiting = std::mem::take(&mut self.the_person_is_waiting_for);
            // A refusal is a window that went away since it mapped, and
            // there is nothing to bring in front.
            let _ = self.the_person_goes_to_this_window(&surface);
            self.the_person_is_waiting_for = still_waiting;
        }
    }

    /// Whether `surface` is a new window of the application that holds the
    /// keyboard while the applications are in front.
    fn a_window_of_the_application_the_person_is_in(&self, surface: &WlSurface) -> bool {
        if self.the_kind_in_front() != crate::which_window_is_in_front::AKindOfWindow::Applications
        {
            return false;
        }
        let Some(focused) = self
            .surfaces
            .keyboard
            .as_ref()
            .and_then(|keyboard| keyboard.handle.current_focus())
        else {
            return false;
        };
        focused != *surface
            && focused
                .client()
                .is_some_and(|client| surface.client().is_some_and(|other| other == client))
    }

    /// **Whether the applications are drawn in front of alo's terminal and
    /// task windows**, for a reader and for a test.
    #[must_use]
    pub fn the_applications_are_in_front(&self) -> bool {
        self.which_is_in_front().top_to_bottom().first()
            == Some(&crate::which_window_is_in_front::AKindOfWindow::Applications)
    }

    /// **The person asked for this application to start**, by a road of
    /// their own: the trusted entry point a press or a key ends in.
    ///
    /// Recorded rather than started, for `Server::asked_to_start`'s reason;
    /// [`Self::start_what_was_asked_for`] sends it.
    pub fn the_person_asked_to_start(&mut self, application: alo_dock::AppId) {
        self.asked_to_start.push(application);
    }
}

#[cfg(test)]
#[path = "a_window_the_person_opened_tests.rs"]
mod tests;
