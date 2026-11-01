//! Whether every window's title is on its edge, and which window wears the
//! focused look — the two things a person tells windows apart by.
//!
//! **The owner's answer of 2026-11-01**: *window titles are always visible on
//! every window's edge, with the app's icon; Settings offers "show the title
//! only when I point at it". The focused window has its own look.* The first
//! half is `alo_appearance::WindowTitles`, told to this seat at stand-up and
//! again whenever Settings keeps a new answer; the second is asked of the
//! keyboard, because *focused* on the board `557:456` means *the window your
//! typing goes to*.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::which_window_is_in_front::AKindOfWindow;

impl crate::Server {
    /// When windows' titles are shown, as the person last said.
    ///
    /// **Told rather than read**, for the reason `Server::the_shortcuts_are`
    /// gives: a compositor that opened a person's settings file would be a
    /// compositor measuring. The desktop reads the file at sign-in and says
    /// what it holds; Settings says it again when the person changes it.
    pub fn the_window_titles_are(&mut self, titles: alo_appearance::WindowTitles) {
        self.window_titles = titles;
    }

    /// When windows' titles are shown.
    #[must_use]
    pub const fn the_window_titles(&self) -> alo_appearance::WindowTitles {
        self.window_titles
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
#[path = "the_window_titles_shown_tests.rs"]
mod tests;
