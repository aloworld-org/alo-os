//! **The two names a window's band shows** — its title, then the application's
//! name a person reads (`docs/design/the-window-controls.md` §3: *The next
//! chapter · Notes*).
//!
//! # The application's name is the one its entry gives
//!
//! Until 2026-11-09 the band showed the application's `app_id`, so a window of
//! GNOME Text Editor said `org.gnome.TextEditor` — a string for machines. The
//! name a person knows it by is its desktop entry's `Name`, in their language,
//! which this seat is already told: `crate::Server::what_each_is_called` is
//! what the alo Bar matches and shows, filled from the same reading of the
//! machine. The band reads that one map, so the Bar and the band cannot call
//! one application two things.
//!
//! **The `app_id` is shown only where no entry names it** — an application
//! started from a path, or one whose entry is not installed — because a
//! machine string is still better than a band that says nothing about what is
//! open. An XDG `app_id` is the desktop entry's identifier by the protocol's
//! own definition, so the lookup is exact.
//!
//! # The title is only ever the application's title
//!
//! `crate::Server::the_name_of` falls back to the `app_id` where a window has
//! no title, which is right for a reader and the handle and wrong here: the
//! band would then say `org.gnome.TextEditor · Text Editor`, one application
//! named twice in two spellings. §3 says *no meaningful title → application
//! name only, never duplicated*, so the band's title is the title or nothing.

use std::collections::BTreeMap;

use crate::FrameName;

/// The band's title: what the application called this window, and nothing
/// where it called it nothing.
#[must_use]
pub(crate) fn the_title_to_show(name: &FrameName) -> Option<&str> {
    match name {
        FrameName::Given(title) => Some(title),
        FrameName::ItsClass(_) | FrameName::AnApplication => None,
    }
}

/// The band's application name: what its entry calls it, else its `app_id`.
///
/// [`None`] where the window gave no `app_id`, which is an absence rather than
/// a name to invent.
#[must_use]
pub(crate) fn the_application_to_show(
    application: Option<&alo_dock::AppId>,
    what_each_is_called: &BTreeMap<String, String>,
) -> Option<String> {
    let application = application?;
    Some(
        what_each_is_called
            .get(application.name())
            .filter(|called| !called.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| application.name().to_owned()),
    )
}

impl crate::Server {
    /// The name this window's band gives its application.
    pub(crate) fn the_application_on_the_band(
        &self,
        surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    ) -> Option<String> {
        the_application_to_show(
            self.the_application_of(surface).as_ref(),
            &self.what_each_is_called,
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
#[path = "the_name_on_the_band_tests.rs"]
mod tests;
