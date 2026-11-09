//! Whether every window's controls are always on its band, and which window
//! wears the focused look.
//!
//! **The owner's ruling of 2026-11-09** (`docs/design/the-window-controls.md`):
//! the title is always on the band, the controls come as the pointer nears, and
//! Settings offers *Always show window controls* — `alo_appearance::WindowControls`,
//! told to this seat at stand-up and again whenever Settings keeps a new answer.
//! The focused look is the 2026-11-01 answer's, kept, and is asked of the
//! keyboard, because *focused* on the board `557:456` means *the window your
//! typing goes to*.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::which_window_is_in_front::AKindOfWindow;

impl crate::Server {
    /// When windows' controls are shown, as the person last said.
    ///
    /// **Told rather than read**, for the reason `Server::the_shortcuts_are`
    /// gives: a compositor that opened a person's settings file would be a
    /// compositor measuring. The desktop reads the file at sign-in and says
    /// what it holds; Settings says it again when the person changes it.
    pub fn the_window_controls_are(&mut self, controls: alo_appearance::WindowControls) {
        self.window_controls = controls;
    }

    /// When windows' controls are shown.
    #[must_use]
    pub const fn the_window_controls(&self) -> alo_appearance::WindowControls {
        self.window_controls
    }

    /// Whether `surface` is the window the person's typing goes to.
    ///
    /// **Both halves, because either alone is wrong.** The seat's keyboard can
    /// rest on an application while the person has gone to a task window or
    /// the terminal — `crate::which_window_is_in_front` — and then the keys go
    /// there, not to the application, so the application is not the one in
    /// focus however the seat's own field reads.
    #[must_use]
    pub(crate) fn is_the_window_typing_goes_to(&self, surface: &WlSurface) -> bool {
        self.the_kind_in_front() == AKindOfWindow::Applications
            && self
                .surfaces
                .keyboard
                .as_ref()
                .and_then(|keyboard| keyboard.handle.current_focus())
                .is_some_and(|focused| focused == *surface)
    }
}

#[cfg(test)]
#[path = "the_window_controls_shown_tests.rs"]
mod tests;
